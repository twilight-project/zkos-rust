use address::{Address, CoinAddress, Network};
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
use elgamalsign::{Signature, VerificationKey};
use std::fmt;
use zkvm::merkle::{CallProof, Hash, MerkleItem, MerkleTree};

///
/// Store for TransactionScript
#[derive(Debug, Clone)]
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
    ///create signatures and zero balance proofs for all inputs
    pub fn create_witness_without_tx(inputs: &[Input], sk_list: &[Scalar]) -> Vec<Witness> {
        let mut witness: Vec<Witness> = Vec::with_capacity(inputs.len());
        //iterate over Inputs and check its type
        for (i, inp) in inputs.iter().enumerate() {
            // create signature over input
            //extract public key of input
            let pk = CoinAddress::from_hex(inp.input.owner().unwrap());
            //serialize input
            let inp_bytes: Vec<u8> = bincode::serialize(inp).unwrap();
            //create signature
            let sign = Signature::sign_message(
                ("ZKOS.Sign").as_bytes(),
                &inp_bytes,
                VerificationKey::from_bytes(pk.public_key.as_bytes().as_slice()).unwrap(),
                sk_list[i],
            );
            //if coin mark witness as Signature
            match inp.in_type {
                IOType::Coin => {
                    witness.push(Witness::Signature(sign));
                }
                //if data mark witness as ZeroBalanceProof
                IOType::Memo => {
                    witness.push(Witness::Signature(sign));
                }
                IOType::State => {
                    witness.push(Witness::Signature(sign));
                }
            }
        }
        witness
    }
}
