use bulletproofs::r1cs;
use bulletproofs::r1cs::ConstraintSystem;
use bulletproofs::r1cs::R1CSProof;
use bulletproofs::{BulletproofGens, PedersenGens};
use curve25519_dalek::ristretto::CompressedRistretto;
use merlin::Transcript;
use mulmsgsig::{Multisignature, VerificationKey};
//use starsig::VerificationKey;
use crate::constraints::Commitment;
use crate::contract::ContractID;
use crate::encoding::{ExactSizeEncodable, Reader};
use crate::errors::VMError;
use crate::fees::FeeRate;
use crate::ops::Instruction;
use crate::predicate::Predicate;
use crate::program::ProgramItem;
use crate::tx::{PrecomputedTx, Tx, TxHeader, VerifiedTx};
use crate::vm::{Delegate, VM};
use crate::zkos_types::{Input, Output};

/// This is the entry point API for verifying a transaction.
/// Verifier passes the `Tx` object through the VM,
/// verifies an aggregated transaction signature (see `signtx` instruction),
/// verifies a R1CS proof and returns a `VerifiedTx` with the log of changes
/// to be applied to the blockchain state.
pub struct Verifier {
    signtx_items: Vec<(VerificationKey, ContractID)>,
    cs: r1cs::Verifier<Transcript>,
    batch: starsig::BatchVerifier<rand::rngs::ThreadRng>,
}

/// Verifier's implementation of the running state of the program.
pub struct VerifierRun {
    program: Vec<u8>,
    offset: usize,
}

impl Delegate<r1cs::Verifier<Transcript>> for Verifier {
    type RunType = VerifierRun;
    type BatchVerifier = starsig::BatchVerifier<rand::rngs::ThreadRng>;

    fn commit_variable(
        &mut self,
        com: &Commitment,
    ) -> Result<(CompressedRistretto, r1cs::Variable), VMError> {
        let point = com.to_point();
        let var = self.cs.commit(point);
        Ok((point, var))
    }

    fn process_tx_signature(
        &mut self,
        pred: Predicate,
        contract_id: ContractID,
    ) -> Result<(), VMError> {
        let key = pred.to_verification_key()?;
        Ok(self.signtx_items.push((key, contract_id)))
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

    fn batch_verifier(&mut self) -> &mut Self::BatchVerifier {
        &mut self.batch
    }
}

impl Verifier {
    /// Precomputes the TxID and TxLog.
    /// This is a private API until we have a nicer composable API with precomputed tx.
    /// See public API `Tx::precompute() that wraps with method`
    /// One obstacle towards that is relation between CS and the transcript: the CS
    /// only holds a &mut of the transcript that can only be parked in the lexical scope,
    /// but not in the struct. And we need CS instance both for building tx and for verifying.
    pub(crate) fn precompute(
        tx: &Tx,
        _inputs: &[Input],
        _outputs: &[Output],
    ) -> Result<PrecomputedTx, VMError> {
        let cs = r1cs::Verifier::new(Transcript::new(b"ZkVM.r1cs"));

        let mut verifier = Verifier {
            signtx_items: Vec::new(),
            cs: cs,
            batch: starsig::BatchVerifier::new(rand::thread_rng()),
        };

        let vm = VM::new(
            tx.header,
            VerifierRun::new(tx.program.clone()),
            &mut verifier,
        );

        let (id, log, fee) = vm.run()?;

        Ok(PrecomputedTx {
            header: tx.header,
            id,
            log,
            feerate: FeeRate::new(fee, tx.encoded_size()),
            signature: tx.signature.clone(),
            proof: tx.proof.clone(),
            verifier,
        })
    }

    /// Verifies the `Tx` object by executing the VM and returns the `VerifiedTx`.
    /// Returns an error if the program is malformed or any of the proofs are not valid.
    pub fn verify_tx(
        verifiable_tx: PrecomputedTx,
        bp_gens: &BulletproofGens,
    ) -> Result<VerifiedTx, VMError> {
        let pc_gens = PedersenGens::default();

        let PrecomputedTx {
            header,
            id,
            log,
            feerate,
            signature,
            proof,
            mut verifier,
        } = verifiable_tx;

        // Commit txid so that the proof is bound to the entire transaction, not just the constraint system.
        verifier.cs.transcript().append_message(b"ZkVM.txid", &id);

        // Verify the R1CS proof
        verifier
            .cs
            .verify(&proof, &pc_gens, bp_gens)
            .map_err(|_| VMError::InvalidR1CSProof)?;

        // Verify the signatures over txid
        let mut signtx_transcript = Transcript::new(b"ZkVM.signtx");
        signtx_transcript.append_message(b"txid", &id);

        if verifier.signtx_items.len() != 0 {
            signature.verify_multi_batched(
                &mut signtx_transcript,
                verifier.signtx_items,
                &mut verifier.batch,
            );
        }

        // Verify all deferred crypto operations.
        verifier
            .batch
            .verify()
            .map_err(|_| VMError::BatchSignatureVerificationFailed)?;

        Ok(VerifiedTx {
            header,
            id,
            log,
            feerate,
        })
    }

    /// verify_proof is a simple function that just verifies a R1CS proof instead of a whole ZKVM tx
    pub fn verify_proof(
        proof: R1CSProof,
        header: TxHeader,
        program: Vec<u8>,
    ) -> Result<bool, VMError> {
        let bp_gens = BulletproofGens::new(256, 1);
        //print!("BP Gens in verify_proof {:?}", bp_gens);
        let pc_gens = PedersenGens::default();
        let cs = r1cs::Verifier::new(Transcript::new(b"ZkVM.r1cs"));

        let mut verifier = Verifier {
            signtx_items: Vec::new(),
            cs: cs,
            batch: starsig::BatchVerifier::new(rand::thread_rng()),
        };

        let vm = VM::new(header, VerifierRun::new(program.clone()), &mut verifier);

        let (id, _log, _fee) = vm.run()?;

        // Commit txid so that the proof is bound to the entire transaction, not just the constraint system.
        verifier.cs.transcript().append_message(b"ZkVM.txid", &id);

        // Verify the R1CS proof
        verifier
            .cs
            .verify(&proof, &pc_gens, &bp_gens)
            .map_err(|_| VMError::InvalidR1CSProof)?;

        Ok(true)
    }

    // verify_proof is a simple function that just verifies a R1CS proof instead of a whole ZKVM tx
    // pub fn verify_proof_new(
    //     proof: R1CSProof,
    //     header: TxHeader,
    //     program: Vec<u8>,
    //     inputs: &[Input],
    //     outputs: &[Output],
    // ) -> Result<bool, VMError> {
    //     let bp_gens = BulletproofGens::new(256, 1);
    //     //print!("BP Gens in verify_proof {:?}", bp_gens);
    //     let pc_gens = PedersenGens::default();
    //     let cs = r1cs::Verifier::new(Transcript::new(b"ZkVM.r1cs"));

    //     let mut verifier = Verifier {
    //         signtx_items: Vec::new(),
    //         cs: cs,
    //         batch: starsig::BatchVerifier::new(rand::thread_rng()),
    //     };

    //     let vm = crate::vm::VMScript::new(
    //         VerifierRun::new(program.clone()),
    //         &mut verifier,
    //         inputs,
    //         outputs,
    //     );

    //     let _fee = vm.run()?;

    //     // Commit txid so that the proof is bound to the entire transaction, not just the constraint system.
    //     verifier
    //         .cs
    //         .transcript()
    //         .append_message(b"ZkVM.txid", b"ZKOS");

    //     // Verify the R1CS proof
    //     verifier
    //         .cs
    //         .verify(&proof, &pc_gens, &bp_gens)
    //         .map_err(|_| VMError::InvalidR1CSProof)?;

    //     Ok(true)
    // }
}

impl VerifierRun {
    fn new(program: Vec<u8>) -> Self {
        VerifierRun { program, offset: 0 }
    }
}
