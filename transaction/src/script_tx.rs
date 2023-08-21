use address::{Address, Network};
use merlin::Transcript;
use quisquislib::{keys::PublicKey, ristretto::RistrettoSecretKey};
use serde::{Deserialize, Serialize};
use zkvm::{
    zkos_types::{Input, InputData, Output, OutputCoin, OutputData, Utxo, Witness},
    IOType, Program,
};

use bulletproofs::r1cs::R1CSProof;
use bulletproofs::BulletproofGens;
use curve25519_dalek::ristretto::CompressedRistretto;
use curve25519_dalek::scalar::Scalar;

use bincode;
use zkschnorr::{Signature, VerificationKey};
use std::fmt;
use zkvm::merkle::{CallProof, Hash, MerkleItem, MerkleTree};

///
/// Store for TransactionScript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptTransaction {
    //transaction header
    pub(crate) version: u64,
    pub(crate) fee: u64,
    pub(crate) maturity: u64,

    //lengths of vectors to come
    pub(crate) input_count: u8,
    pub(crate) output_count: u8,
    pub(crate) witness_count: u8,

    //List of inputs and outputs
    pub(crate) inputs: Vec<Input>,
    pub(crate) outputs: Vec<Output>,

    //Script program to be executed by the VM
    pub(crate) program: Vec<u8>,
    //Call Proof for program Merkle tree inclusion
    pub(crate) call_proof: CallProof,

    //Script proof for computations in tx
    pub(crate) proof: R1CSProof,

    //required for lit to dark case. contains same value proof
    pub(crate) witness: Option<Vec<Witness>>,
    // Transaction data. e.g., supporting data for a script transaction.
    pub(crate) data: Vec<u8>,
}

// /// Represents a precomputed, but not verified transaction.
// pub struct PrecomputedTx {
//     /// Transaction header
//     pub header: TxHeader,

//     /// Transaction ID
//     pub id: TxID,

//     /// Transaction log: a list of changes to the blockchain state (UTXOs to delete/insert, etc.)
//     pub log: TxLog,

//     /// Fee rate of the transaction
//     pub feerate: FeeRate,

//     /// Verifier to continue verification of the transaction
//     pub(crate) verifier: Verifier,

//     /// Schnorr signature
//     pub(crate) signature: Signature,

//     /// R1CS proof
//     pub(crate) proof: R1CSProof,
// }

impl ScriptTransaction {
    /// run the program and create a proof
    pub fn create_script_tx_without_witness(prog: Program, inputs: &[Input], outputs: &[Output]) {

        //Run the program and create a proof
        

    }
    /// Set a script transaction
    /// 
    pub fn set_script_transaction(version: u64,
        fee: u64,
        maturity: u64,
        input_count: u8,
        output_count: u8,
        witness_count: u8,
        inputs: Vec<Input>,
        outputs: Vec<Output>,
        program: Vec<u8>,
        call_proof: CallProof,
        proof: R1CSProof,
        data: Vec<u8>,
        witness: Option<Vec<Witness>>,) -> Self{
        ScriptTransaction {
            version,
            fee,
            maturity,
            input_count,
            output_count,
            witness_count,
            inputs,
            outputs,
            program,
            call_proof,
            proof,
            data,
            witness,
        }
    }
    ///create signatures and zero balance proofs for all inputs
    // pub fn create_witness_without_tx(inputs: &[Input], sk_list: &[Scalar]) -> Vec<Witness> {
    //     let mut witness: Vec<Witness> = Vec::with_capacity(inputs.len());
    //     //iterate over Inputs and check its type
    //     for (i, inp) in inputs.iter().enumerate() {
    //         // create signature over input
    //         //extract public key of input
    //         let pk = address::Standard::from_hex(inp.input.owner().unwrap());
    //         //serialize input
    //         let inp_bytes: Vec<u8> = bincode::serialize(inp).unwrap();
    //         //create signature
    //         let sign = Signature::sign_message(
    //             ("ZKOS.Sign").as_bytes(),
    //             &inp_bytes,
    //             VerificationKey::from_bytes(pk.public_key.as_bytes().as_slice()).unwrap(),
    //             sk_list[i],
    //         );
    //         //if coin mark witness as Signature
    //         match inp.in_type {
    //             IOType::Coin => {
    //                 witness.push(Witness::Signature(sign));
    //             }
    //             //if data mark witness as ZeroBalanceProof
    //             IOType::Memo => {
    //                 witness.push(Witness::Signature(sign));
    //             }
    //             IOType::State => {
    //                 witness.push(Witness::Signature(sign));
    //             }
    //         }
    //     }
    //     witness
    // }
    ///DUMMY TX FOR UTXO SET VERIFICATIO
    /// 
    pub fn create_utxo_script_transaction(
        inputs: &[Input],
        outputs: &[Output],
    ) -> ScriptTransaction {
        let program:Vec<u8> = vec![b'0'; 32];
        ScriptTransaction::set_script_transaction(
            0u64,
            0u64,
            0u64,
            inputs.len() as u8,
            outputs.len() as u8,
            0u8,
            inputs.to_vec(),
            outputs.to_vec(),
            program,
            CallProof::default(),
            R1CSProof::from_bytes(&[0u8; 32]).unwrap(),
            vec![b'0'; 32],
            None,
        )
    }

    pub fn verify_script_tx(
        &self,
        inputs: &[Input],
        outputs: &[Output],
    ) -> Result<(), &'static str> {
        //create QuisQUisTx Prover merlin transcript
        let mut transcript = Transcript::new(b"TxProof");
       // let mut verifier = Verifier::new(b"QuisQuisTx", &mut transcript);

        //verify the Dark Proof first
        //self.script_sig.verify(&mut verifier, &inputs, &outputs)?;

        Ok(())
    }

    //created for utxo-in-memory
    pub fn get_input_values(&self) -> Vec<Input> {
        self.inputs.clone()
    }
    pub fn get_output_values(&self) -> Vec<Output> {
        self.outputs.clone()
    }

}
