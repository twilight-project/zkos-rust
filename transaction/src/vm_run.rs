use bulletproofs::r1cs::{self, R1CSProof};
use bulletproofs::{BulletproofGens, PedersenGens};
use curve25519_dalek::ristretto::CompressedRistretto;
use merlin::Transcript;
use std::collections::VecDeque;
use zkvm::encoding::{Encodable, Reader};

use zkvm::constraints::Commitment;
use zkvm::errors::VMError;
use zkvm::ops::Instruction;
use zkvm::program::{Program, ProgramItem};
use zkvm::vm::{VMRun, VMScript};
use zkvm::zkos_types::{Input, Output};

/// This is the entry point API for creating a proof for Script transaction.
/// Prover passes the list of instructions through the VM,
/// creates a R1CS proof and returns the full proof
pub struct Prover<'g> {
    cs: r1cs::Prover<'g, Transcript>,
}

pub struct ProverRun {
    program: VecDeque<Instruction>,
}

impl<'t, 'g> VMRun<r1cs::Prover<'g, Transcript>> for Prover<'g> {
    type RunType = ProverRun;

    fn commit_variable(
        &mut self,
        com: &Commitment,
    ) -> Result<(CompressedRistretto, r1cs::Variable), VMError> {
        let (v, v_blinding) = com.witness().ok_or(VMError::WitnessMissing)?;
        Ok(self.cs.commit(v.into(), v_blinding))
    }

    fn next_instruction(&mut self, run: &mut ProverRun) -> Result<Option<Instruction>, VMError> {
        Ok(run.program.pop_front())
    }

    fn new_run(&self, data: ProgramItem) -> Result<ProverRun, VMError> {
        Ok(ProverRun {
            program: data.to_program()?.to_vec().into(),
        })
    }

    fn cs(&mut self) -> &mut r1cs::Prover<'g, Transcript> {
        &mut self.cs
    }
}

impl<'g> Prover<'g> {
    /// Builds a proof with a given list of instructions
    /// Fails if the input program is malformed, or some witness data is missing.
    pub fn build_proof(
        program: Program,
        inputs: &[Input],
        outputs: &[Output],
        contract_deploy_flag: bool,
        tx_data: Option<zkvm::String>,
    ) -> Result<(Vec<u8>, R1CSProof), VMError> {
        // Prepare the constraint system
        let bp_gens = BulletproofGens::new(256, 1);
        let pc_gens = PedersenGens::default();
        let cs = r1cs::Prover::new(&pc_gens, Transcript::new(b"ZkVM.r1cs"));

        // Serialize the tx program
        let mut bytecode = Vec::new();

        program.encode(&mut bytecode)?;

        let mut prover = Prover { cs };

        let mut vm: VMScript<'_, r1cs::Prover<'_, Transcript>, Prover<'_>> = VMScript::new(
            ProverRun {
                program: program.to_vec().into(),
            },
            &mut prover,
            inputs,
            outputs,
            tx_data,
            // contract_init_flag,
        );

        // initialize the Stack with inputs and outputs
        match contract_deploy_flag {
            false => {
                let init_result = vm.initialize_stack()?;
                println!("\n Default VM initialized result {:?}", init_result);
            }
            true => {
                let init_result = vm.initialize_deploy_contract_stack()?;
                println!("\n Contract VM initialized result {:?}", init_result);
            }
        }
        // let init_result = vm.initialize_stack()?;
        // println!("VM initialized result {:?}", init_result);

        // run the program to create a R1CS circuit
        let run_result = vm.run()?;
        println!("Vm run result {:?}", run_result);
        // Commit txid so that the proof is bound to the entire transaction, not just the constraint system.
        //COMMIT TXID TO THE PROOF TO MAKE IT BOUND TO THE ENTIRE TRANSACTION
        // prover.cs.transcript().append_message(b"ZkVM.txid", b"ZKOS");

        // Generate the R1CS proof
        let proof = prover
            .cs
            .prove(&bp_gens)
            .map_err(|_| VMError::InvalidR1CSProof)?;
        // Defer signing of the transaction to the UnsignedTx API.
        Ok((bytecode, proof))
    }
}

/// This is the entry point API for verifying a R1CS proof.
/// verifies a R1CS proof and returns a `Result`
///
pub struct Verifier {
    cs: r1cs::Verifier<Transcript>,
}

/// Verifier's implementation of the running state of the program.
pub struct VerifierRun {
    program: Vec<u8>,
    offset: usize,
}

impl VMRun<r1cs::Verifier<Transcript>> for Verifier {
    type RunType = VerifierRun;

    fn commit_variable(
        &mut self,
        com: &Commitment,
    ) -> Result<(CompressedRistretto, r1cs::Variable), VMError> {
        let point = com.to_point();
        let var = self.cs.commit(point);
        Ok((point, var))
    }

    fn next_instruction(
        &mut self,
        run: &mut Self::RunType,
    ) -> Result<Option<Instruction>, VMError> {
        if run.offset == run.program.len() {
            return Ok(None);
        }
        let mut reader = &run.program[run.offset..];
        let instr = Instruction::parse(&mut reader)?;
        run.offset = run.program.len() - reader.remaining_bytes();
        Ok(Some(instr))
    }

    fn new_run(&self, prog: ProgramItem) -> Result<Self::RunType, VMError> {
        Ok(VerifierRun::new(prog.to_bytecode()?))
    }

    fn cs(&mut self) -> &mut r1cs::Verifier<Transcript> {
        &mut self.cs
    }
}

impl Verifier {
    /// verify_proof is a simple function that just verifies a R1CS proof instead of a whole ZKVM tx
    pub fn verify_r1cs_proof(
        proof: &R1CSProof,
        program: &Vec<u8>,
        inputs: &[Input],
        outputs: &[Output],
        contract_deploy_flag: bool,
        tx_data: Option<zkvm::String>,
    ) -> Result<bool, VMError> {
        let bp_gens = BulletproofGens::new(256, 1);
        //print!("BP Gens in verify_proof {:?}", bp_gens);
        let pc_gens = PedersenGens::default();
        let cs = r1cs::Verifier::new(Transcript::new(b"ZkVM.r1cs"));

        let mut verifier = Verifier { cs };

        let mut vm = VMScript::new(
            VerifierRun::new(program.clone()),
            &mut verifier,
            inputs,
            outputs,
            tx_data,
            // contract_init_flag,
        );

        // initialize the Stack with inputs and outputs
        match contract_deploy_flag {
            false => {
                let _init_result = vm.initialize_stack()?;
            }
            true => {
                let _init_result = vm.initialize_deploy_contract_stack()?;
            }
        }
        //let _init_result = vm.initialize_stack()?;
        // run the program to create a proof
        let _run_result = vm.run()?;

        // Commit txid so that the proof is bound to the entire transaction, not just the constraint system.
        //COMMIT TXID TO THE PROOF TO MAKE IT BOUND TO THE ENTIRE TRANSACTION
        // verifier
        //     .cs
        //     .transcript()
        //     .append_message(b"ZkVM.txid", b"ZKOS");

        // Verify the R1CS proof
        verifier
            .cs
            .verify(&proof, &pc_gens, &bp_gens)
            .map_err(|_| VMError::InvalidR1CSProof)?;

        Ok(true)
    }
}

impl VerifierRun {
    fn new(program: Vec<u8>) -> Self {
        VerifierRun { program, offset: 0 }
    }
}
