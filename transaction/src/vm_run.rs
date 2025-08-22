//! Virtual machine execution for script transactions with R1CS proof generation.
//!
//! This module provides the core virtual machine functionality for executing
//! script transactions in the ZkOS blockchain. It handles both proof generation
//! (for provers) and proof verification (for verifiers) using Bulletproofs R1CS.
//!
//! # Overview
//!
//! The VM execution system consists of two main components:
//!
//! - **Prover**: Executes programs and generates R1CS proofs
//! - **Verifier**: Verifies R1CS proofs without executing programs
//!
//! # Architecture
//!
//! The VM integrates with the ZkVM system to:
//!
//! 1. **Parse Instructions**: Convert bytecode to executable instructions
//! 2. **Execute Programs**: Run programs with constraint generation
//! 3. **Generate Proofs**: Create R1CS proofs for program execution
//! 4. **Verify Proofs**: Validate proofs without re-execution
//!
//! # Example
//!
//! ```rust
//! use transaction::vm_run::Prover;
//! use zkvm::{Program, zkos_types::{Input, Output}};
//! use bulletproofs::BulletproofGens;
//!
//! // Create a program
//! let program = Program::new(vec![/* instructions */]);
//! let inputs = vec![/* input data */];
//! let outputs = vec![/* output data */];
//!
//! // Generate proof
//! let (bytecode, proof) = Prover::build_proof(
//!     program,
//!     &inputs,
//!     &outputs,
//!     false, // not contract deployment
//!     None,  // no transaction data
//! ).unwrap();
//!
//! // Verify proof
//! let is_valid = Verifier::verify_r1cs_proof(
//!     &proof,
//!     &bytecode,
//!     &inputs,
//!     &outputs,
//!     false,
//!     None,
//! ).unwrap();
//!
//! assert!(is_valid);
//! ```

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

/// Entry point API for creating R1CS proofs for script transactions.
///
/// The `Prover` struct handles the execution of programs and generation
/// of zero-knowledge proofs using Bulletproofs R1CS. It integrates with
/// the ZkVM system to execute programs while building constraint systems.
///
/// # Example
/// ```
/// use transaction::vm_run::Prover;
/// use zkvm::{Program, zkos_types::{Input, Output}};
///
/// let program = Program::new(vec![/* instructions */]);
/// let inputs = vec![/* input data */];
/// let outputs = vec![/* output data */];
///
/// let (bytecode, proof) = Prover::build_proof(
///     program,
///     &inputs,
///     &outputs,
///     false, // not contract deployment
///     None,  // no transaction data
/// ).unwrap();
/// ```
pub struct Prover<'g> {
    /// The R1CS constraint system prover
    cs: r1cs::Prover<'g, Transcript>,
}

/// Running state for the prover's program execution.
///
/// This struct maintains the state of program execution during proof
/// generation, including the instruction queue and execution context.
pub struct ProverRun {
    /// Queue of instructions to be executed
    program: VecDeque<Instruction>,
}

impl<'t, 'g> VMRun<r1cs::Prover<'g, Transcript>> for Prover<'g> {
    type RunType = ProverRun;

    /// Commits a variable to the constraint system.
    ///
    /// This method takes a commitment and creates a corresponding variable
    /// in the R1CS constraint system, returning both the compressed point
    /// and the variable reference.
    ///
    /// # Arguments
    /// * `com` - The commitment to commit
    ///
    /// # Returns
    /// `Ok((CompressedRistretto, Variable))` if successful, `Err` otherwise
    ///
    /// # Errors
    /// * `VMError::WitnessMissing` - If the commitment lacks witness data
    fn commit_variable(
        &mut self,
        com: &Commitment,
    ) -> Result<(CompressedRistretto, r1cs::Variable), VMError> {
        let (v, v_blinding) = com.witness().ok_or(VMError::WitnessMissing)?;
        Ok(self.cs.commit(v.into(), v_blinding))
    }

    /// Retrieves the next instruction from the program.
    ///
    /// This method pops the next instruction from the program queue,
    /// returning `None` when the program is complete.
    ///
    /// # Arguments
    /// * `run` - The current program execution state
    ///
    /// # Returns
    /// `Ok(Some(Instruction))` if an instruction is available, `Ok(None)` if complete
    fn next_instruction(&mut self, run: &mut ProverRun) -> Result<Option<Instruction>, VMError> {
        Ok(run.program.pop_front())
    }

    /// Creates a new program execution run.
    ///
    /// This method converts a program item into an executable program
    /// and creates the initial execution state.
    ///
    /// # Arguments
    /// * `data` - The program item to execute
    ///
    /// # Returns
    /// `Ok(ProverRun)` if successful, `Err` otherwise
    fn new_run(&self, data: ProgramItem) -> Result<ProverRun, VMError> {
        Ok(ProverRun {
            program: data.to_program()?.to_vec().into(),
        })
    }

    /// Returns a mutable reference to the constraint system.
    ///
    /// This allows the VM to add constraints during program execution.
    fn cs(&mut self) -> &mut r1cs::Prover<'g, Transcript> {
        &mut self.cs
    }
}

impl<'g> Prover<'g> {
    /// Builds a complete R1CS proof for a script transaction.
    ///
    /// This is the main entry point for creating proofs for script transactions.
    /// It executes the program, builds the constraint system, and generates
    /// a zero-knowledge proof that can be verified without revealing the
    /// execution details.
    ///
    /// # Arguments
    /// * `program` - The program to execute
    /// * `inputs` - Input values for the program
    /// * `outputs` - Expected output values
    /// * `contract_deploy_flag` - Whether this is a contract deployment
    /// * `tx_data` - Optional transaction data
    ///
    /// # Returns
    /// `Ok((Vec<u8>, R1CSProof))` containing the bytecode and proof
    ///
    /// # Errors
    /// * `VMError::InvalidR1CSProof` - If proof generation fails
    /// * `VMError::WitnessMissing` - If required witness data is missing
    /// * Other VM errors from program execution
    ///
    /// # Example
    /// ```
    /// use transaction::vm_run::Prover;
    /// use zkvm::{Program, zkos_types::{Input, Output}};
    ///
    /// let program = Program::new(vec![/* instructions */]);
    /// let inputs = vec![/* input data */];
    /// let outputs = vec![/* output data */];
    ///
    /// let (bytecode, proof) = Prover::build_proof(
    ///     program,
    ///     &inputs,
    ///     &outputs,
    ///     false, // not contract deployment
    ///     None,  // no transaction data
    /// ).unwrap();
    /// ```
    pub fn build_proof(
        program: Program,
        inputs: &[Input],
        outputs: &[Output],
        contract_deploy_flag: bool,
        tx_data: Option<zkvm::String>,
    ) -> Result<(Vec<u8>, R1CSProof), VMError> {
        // Prepare the constraint system with appropriate generators
        let bp_gens = BulletproofGens::new(256, 1);
        let pc_gens = PedersenGens::default();
        let cs = r1cs::Prover::new(&pc_gens, Transcript::new(b"ZkVM.r1cs"));

        // Serialize the program to bytecode
        let mut bytecode = Vec::new();
        program.encode(&mut bytecode)?;

        let mut prover = Prover { cs };

        // Create the VM script executor
        let mut vm: VMScript<'_, r1cs::Prover<'_, Transcript>, Prover<'_>> = VMScript::new(
            ProverRun {
                program: program.to_vec().into(),
            },
            &mut prover,
            inputs,
            outputs,
            tx_data,
        );

        // Initialize the VM stack based on transaction type
        match contract_deploy_flag {
            false => {
                // Standard script execution
                vm.initialize_stack()?;
            }
            true => {
                // Contract deployment execution
                vm.initialize_deploy_contract_stack()?;
            }
        }

        // Execute the program to build the constraint system
        vm.run()?;

        // Generate the R1CS proof
        let proof = prover
            .cs
            .prove(&bp_gens)
            .map_err(|_| VMError::InvalidR1CSProof)?;

        Ok((bytecode, proof))
    }
}

/// Entry point API for verifying R1CS proofs.
///
/// The `Verifier` struct handles the verification of R1CS proofs without
/// re-executing the original program. It validates that the proof is
/// consistent with the program, inputs, and outputs.
///
/// # Example
/// ```
/// use transaction::vm_run::Verifier;
/// use zkvm::zkos_types::{Input, Output};
/// use bulletproofs::R1CSProof;
///
/// let is_valid = Verifier::verify_r1cs_proof(
///     &proof,
///     &bytecode,
///     &inputs,
///     &outputs,
///     false, // not contract deployment
///     None,  // no transaction data
/// ).unwrap();
///
/// assert!(is_valid);
/// ```
pub struct Verifier {
    /// The R1CS constraint system verifier
    cs: r1cs::Verifier<Transcript>,
}

/// Running state for the verifier's program execution.
///
/// This struct maintains the state of program verification, including
/// the bytecode and current instruction offset.
pub struct VerifierRun {
    /// The program bytecode
    program: Vec<u8>,
    /// Current instruction offset in the bytecode
    offset: usize,
}

impl VMRun<r1cs::Verifier<Transcript>> for Verifier {
    type RunType = VerifierRun;

    /// Commits a variable to the constraint system.
    ///
    /// For verification, this method creates a variable from the commitment
    /// point without requiring witness data.
    ///
    /// # Arguments
    /// * `com` - The commitment to commit
    ///
    /// # Returns
    /// `Ok((CompressedRistretto, Variable))` if successful, `Err` otherwise
    fn commit_variable(
        &mut self,
        com: &Commitment,
    ) -> Result<(CompressedRistretto, r1cs::Variable), VMError> {
        let point = com.to_point();
        let var = self.cs.commit(point);
        Ok((point, var))
    }

    /// Retrieves the next instruction from the bytecode.
    ///
    /// This method parses the next instruction from the bytecode at the
    /// current offset, advancing the offset as it reads.
    ///
    /// # Arguments
    /// * `run` - The current verification state
    ///
    /// # Returns
    /// `Ok(Some(Instruction))` if an instruction is available, `Ok(None)` if complete
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

    /// Creates a new verification run from a program item.
    ///
    /// This method converts a program item to bytecode and creates
    /// the initial verification state.
    ///
    /// # Arguments
    /// * `prog` - The program item to verify
    ///
    /// # Returns
    /// `Ok(VerifierRun)` if successful, `Err` otherwise
    fn new_run(&self, prog: ProgramItem) -> Result<Self::RunType, VMError> {
        Ok(VerifierRun::new(prog.to_bytecode()?))
    }

    /// Returns a mutable reference to the constraint system.
    ///
    /// This allows the VM to add constraints during verification.
    fn cs(&mut self) -> &mut r1cs::Verifier<Transcript> {
        &mut self.cs
    }
}

impl Verifier {
    /// Verifies an R1CS proof for a script transaction.
    ///
    /// This is the main entry point for verifying proofs of script transactions.
    /// It reconstructs the constraint system from the program, inputs, and outputs,
    /// then verifies that the proof satisfies all constraints.
    ///
    /// # Arguments
    /// * `proof` - The R1CS proof to verify
    /// * `program` - The program bytecode
    /// * `inputs` - Input values used in the proof
    /// * `outputs` - Output values from the proof
    /// * `contract_deploy_flag` - Whether this was a contract deployment
    /// * `tx_data` - Optional transaction data
    ///
    /// # Returns
    /// `Ok(bool)` - `true` if verification succeeds, `false` otherwise
    ///
    /// # Errors
    /// * `VMError::InvalidR1CSProof` - If proof verification fails
    /// * Other VM errors from constraint system reconstruction
    ///
    /// # Example
    /// ```
    /// use transaction::vm_run::Verifier;
    /// use zkvm::zkos_types::{Input, Output};
    /// use bulletproofs::R1CSProof;
    ///
    /// let is_valid = Verifier::verify_r1cs_proof(
    ///     &proof,
    ///     &bytecode,
    ///     &inputs,
    ///     &outputs,
    ///     false, // not contract deployment
    ///     None,  // no transaction data
    /// ).unwrap();
    ///
    /// if is_valid {
    ///     println!("Proof verification successful");
    /// } else {
    ///     println!("Proof verification failed");
    /// }
    /// ```
    pub fn verify_r1cs_proof(
        proof: &R1CSProof,
        program: &Vec<u8>,
        inputs: &[Input],
        outputs: &[Output],
        contract_deploy_flag: bool,
        tx_data: Option<zkvm::String>,
    ) -> Result<bool, VMError> {
        // Prepare the constraint system with appropriate generators
        let bp_gens = BulletproofGens::new(256, 1);
        let pc_gens = PedersenGens::default();
        let cs = r1cs::Verifier::new(Transcript::new(b"ZkVM.r1cs"));

        let mut verifier = Verifier { cs };

        // Create the VM script verifier
        let mut vm = VMScript::new(
            VerifierRun::new(program.clone()),
            &mut verifier,
            inputs,
            outputs,
            tx_data,
        );

        // Initialize the VM stack based on transaction type
        match contract_deploy_flag {
            false => {
                // Standard script verification
                vm.initialize_stack()?;
            }
            true => {
                // Contract deployment verification
                vm.initialize_deploy_contract_stack()?;
            }
        }

        // Reconstruct the constraint system from the program
        vm.run()?;
        // Commit txid so that the proof is bound to the entire transaction, not just the constraint system.
        //COMMIT TXID TO THE PROOF TO MAKE IT BOUND TO THE ENTIRE TRANSACTION
        // verifier
        //     .cs
        //     .transcript()
        //     .append_message(b"ZkVM.txid", b"ZKOS");

        // Verify the R1CS proof
        verifier
            .cs
            .verify(proof, &pc_gens, &bp_gens)
            .map_err(|_| VMError::InvalidR1CSProof)?;

        Ok(true)
    }
}

impl VerifierRun {
    /// Creates a new verification run from bytecode.
    ///
    /// # Arguments
    /// * `program` - The program bytecode
    ///
    /// # Returns
    /// A new `VerifierRun` instance
    fn new(program: Vec<u8>) -> Self {
        VerifierRun { program, offset: 0 }
    }
}
