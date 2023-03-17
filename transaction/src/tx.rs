#![allow(non_snake_case)]
//#![deny(missing_docs)]

use crate::proof::{DarkTxProof, ShuffleTxProof};
use crate::types::{Input, Output, OutputData, TransactionType, TxEntry, TxLog, Witness};
use crate::util::{Address, Network};
use merlin::Transcript;
// use serde_derive::{Deserialize, Serialize};
use serde::{Deserialize, Serialize};

use curve25519_dalek::scalar::Scalar;
use quisquislib::{
    accounts::prover::Prover,
    accounts::verifier::Verifier,
    accounts::Account,
    keys::PublicKey,
    ristretto::{RistrettoPublicKey, RistrettoSecretKey},
    shuffle::Shuffle,
};

/// A complete twilight Transactiont valid for a specific network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Defines the Tx type.
    pub tx_type: TransactionType,
    /// The Tx data corresponding to the Tx type.
    pub tx: TransactionData,
}

impl Transaction {
    /// Create a input of Dark Coin which is valid on the given network.
    pub fn transaction_transfer(data: TransactionData) -> Transaction {
        Transaction {
            tx_type: TransactionType::default(),
            tx: data,
        }
    }
}

///
/// Store for TransactionTransfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferTransaction {
    //transaction header
    pub(crate) version: u64,
    pub(crate) maturity: u64,
    //lengths of vectors to come
    pub(crate) input_count: u8,
    pub(crate) output_count: u8,
    pub(crate) witness_count: u8,
    //List of inputs and outputs
    pub(crate) inputs: Vec<Input>,
    pub(crate) outputs: Vec<Output>,
    //Dark Tx proof
    pub(crate) proof: DarkTxProof,
    //input and output shuffle proof
    pub(crate) shuffle_proof: Option<ShuffleTxProof>,
    //required for lit to dark case. contains same value proof
    pub(crate) witness: Option<Vec<Witness>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionData {
    TransactionTransfer(TransferTransaction),
    //TransactionTransition,
    //TransactionCreate,
    //TransactionVault,
}

impl TransactionData {
    /// Downcasts Transaction to `Transfer` type.
    pub fn to_transfer(self) -> Result<TransferTransaction, &'static str> {
        match self {
            TransactionData::TransactionTransfer(x) => Ok(x),
            _ => Err("Invalid Transfer Transaction"),
        }
    }
}

// //creates empty Transfer Transaction
// //impl Default for Transaction {
//   //  fn default() -> Self {
//     //    Transfer{ version: 0u64, byte_price: 0u64, price: 0u64, maturity: 0u64, input_count: 0u8, output_count: 0u8, witness_count: 0u8, inputs: Vec::new(), outputs: Vec::new(),
//       //  }
//     //    String::Opaque(Vec::new())
//   //  }
// //}

impl TransferTransaction {
    // Private constructor
    fn set_tranfer_transaction(
        version: u64,
        maturity: u64,
        input_count: u8,
        output_count: u8,
        witness_count: u8,
        inputs: Vec<Input>,
        outputs: Vec<Output>,
        proof: DarkTxProof,
        shuffle_proof: Option<ShuffleTxProof>,
        witness: Option<Vec<Witness>>,
    ) -> TransferTransaction {
        TransferTransaction {
            version,
            maturity,
            input_count,
            output_count,
            witness_count,
            inputs,
            outputs,
            proof,
            shuffle_proof,
            witness,
        }
    }

    pub fn create_dark_transaction(
        value_vector: &[i64],
        account_vector: &[Account],
        sender_updated_balance: &[u64],
        reciever_updated_balance: &[u64],
        input_vector: &[Input],
        sender_sk: &[RistrettoSecretKey],
        senders_count: usize,
        receivers_count: usize,
        tx_log: &mut TxLog,
    ) -> Result<(TransferTransaction), &'static str> {
        //convert the valur vector into scalar type to create the proof
        let mut value_vector_scalar = Vec::<Scalar>::new();
        for v in value_vector.iter() {
            if v >= &0 {
                value_vector_scalar.push(Scalar::from(*v as u64));
            } else {
                value_vector_scalar.push(-Scalar::from((-*v) as u64));
            }
        }
        //create base pk for epsilon accounts
        let base_pk = RistrettoPublicKey::generate_base_pk();

        //create DarkTx Prover merlin transcript
        let mut transcript = Transcript::new(b"TxProof");
        let mut prover = Prover::new(b"DarkTx", &mut transcript);

        //create delta_and_epsilon_accounts
        let (delta_accounts, epsilon_accounts, delta_rscalar) =
            Account::create_delta_and_epsilon_accounts(
                &account_vector,
                &value_vector_scalar,
                base_pk,
            );

        //identity check function to verify the construction of epsilon accounts using correct rscalars
        Verifier::verify_delta_identity_check(&epsilon_accounts)?;

        //update delta_accounts to reflect the change in balance
        //updated_delta_accounts = Output account for DarkTx
        let updated_delta_accounts =
            Account::update_delta_accounts(&account_vector, &delta_accounts)?;
        let dark_tx_proof = DarkTxProof::create_dark_tx_proof(
            &mut prover,
            &value_vector_scalar,
            &delta_accounts,
            &epsilon_accounts,
            &delta_rscalar,
            &sender_updated_balance,
            &reciever_updated_balance,
            &sender_sk,
            senders_count,
            receivers_count,
            base_pk,
        );

        //create vec of Outputs -- Recievers in this case
        let mut outputs: Vec<Output> = Vec::new();
        for i in senders_count..senders_count + receivers_count {
            //create address
            let (pk, comm) = updated_delta_accounts[i].get_account();
            let out = Output::coin(OutputData::coin(
                Address::standard(Network::default(), pk),
                comm,
            ));
            outputs.push(out.clone());
            tx_log.push(TxEntry::Output(out));
        }

        Ok(TransferTransaction::set_tranfer_transaction(
            0u64,
            0u64,
            senders_count as u8,
            receivers_count as u8,
            0u8,
            input_vector.to_vec(),
            outputs,
            dark_tx_proof,
            None,
            None,
        ))
    }

    pub fn verify_dark_tx(
        &self,
        inputs: &[Account],
        outputs: &[Account],
    ) -> Result<(), &'static str> {
        //create DarkTx Prover merlin transcript
        let mut transcript = Transcript::new(b"TxProof");
        let mut verifier = Verifier::new(b"DarkTx", &mut transcript);

        //find the number of senders
        let senders_count = self.proof.updated_sender_epsilon_accounts.len();
        //create updated senders delta account slice
        let updated_senders_delta_account = &self.proof.delta_accounts[..senders_count];
        //TODO::CONVERT INPUS AND OUTPUTS TO ACCOUNTS
        //verify the proof
        self.proof.verify(
            &mut verifier,
            &inputs.to_vec(),
            &updated_senders_delta_account,
            &outputs.to_vec(),
        )?;
        Ok(())
    }

    pub fn create_quisquis_transaction(
        input_vector: &[Input],
        value_vector: &[i64],
        account_vector: &[Account],
        sender_updated_balance: &[u64],
        reciever_updated_balance: &[u64],
        sender_sk: &[RistrettoSecretKey],
        senders_count: usize,
        receivers_count: usize,
        anonymity_comm_scalar: &[Scalar],
        anonymity_account_diff: usize,
        tx_log: &mut TxLog,
    ) -> Result<TransferTransaction, &'static str> {
        //convert the valur vector into scalar type to create the proof
        let mut value_vector_scalar = Vec::<Scalar>::new();
        for v in value_vector.iter() {
            if v >= &0 {
                value_vector_scalar.push(Scalar::from(*v as u64));
            } else {
                value_vector_scalar.push(-Scalar::from((-*v) as u64));
            }
        }
        //create base pk for epsilon accounts
        let base_pk = RistrettoPublicKey::generate_base_pk();
        //Step 1. update & shuffle input accounts
        let input_shuffle = Shuffle::input_shuffle(account_vector)?;
        let updated_accounts = input_shuffle.get_outputs_vector();
        //create QuisQuisTx Prover merlin transcript
        let mut transcript = Transcript::new(b"TxProof");
        let mut prover = Prover::new(b"QuisQuisTx", &mut transcript);

        //create delta_and_epsilon_accounts
        let (delta_accounts, epsilon_accounts, delta_rscalar) =
            Account::create_delta_and_epsilon_accounts(
                &updated_accounts,
                &value_vector_scalar,
                base_pk,
            );

        //identity check function to verify the construction of epsilon accounts using correct rscalars
        Verifier::verify_delta_identity_check(&epsilon_accounts)?;

        //update delta_accounts to reflect the change in balance
        //updated_delta_accounts = Output account for DarkTx
        let updated_delta_accounts =
            Account::update_delta_accounts(&updated_accounts, &delta_accounts)?;

        ///create Dark Proof
        let dark_tx_proof = DarkTxProof::create_dark_tx_proof(
            &mut prover,
            &value_vector_scalar,
            &delta_accounts,
            &epsilon_accounts,
            &delta_rscalar,
            &sender_updated_balance,
            &reciever_updated_balance,
            &sender_sk,
            senders_count,
            receivers_count,
            base_pk,
        );

        let anonymity_index = 9 - anonymity_account_diff;
        let updated_accounts_slice = &updated_accounts[anonymity_index..9];
        let updated_delta_accounts_slice = &updated_delta_accounts[anonymity_index..9];
        let rscalars_slice = &delta_rscalar[anonymity_index..9];

        //Shuffle accounts
        let output_shuffle = Shuffle::output_shuffle(&updated_delta_accounts)?;

        let shuffle_proof = ShuffleTxProof::create_shuffle_proof(
            &mut prover,
            &updated_accounts,
            &updated_delta_accounts,
            &delta_rscalar,
            &anonymity_comm_scalar,
            anonymity_index,
            senders_count,
            receivers_count,
            base_pk,
            &input_shuffle,
            &output_shuffle,
        );

        let output_final = output_shuffle.get_outputs_vector();
        //create vec of Outputs -- Recievers in this case
        let mut outputs: Vec<Output> = Vec::new();
        for out in output_final.iter() {
            //create address
            let (pk, comm) = out.get_account();
            let output = Output::coin(OutputData::coin(
                Address::standard(Network::default(), pk),
                comm,
            ));
            outputs.push(output.clone());
            tx_log.push(TxEntry::Output(output));
        }

        Ok(TransferTransaction::set_tranfer_transaction(
            0u64,
            0u64,
            senders_count as u8,
            receivers_count as u8,
            0u8,
            input_vector.to_vec(),
            outputs,
            dark_tx_proof,
            Some(shuffle_proof),
            None,
        ))
    }

    pub fn verify_quisquis_tx(
        &self,
        inputs: &[Account],
        outputs: &[Account],
    ) -> Result<(), &'static str> {
        //create QuisQUisTx Prover merlin transcript
        let mut transcript = Transcript::new(b"TxProof");
        let mut verifier = Verifier::new(b"QuisQuisTx", &mut transcript);

        //find the number of senders
        let senders_count = self.proof.updated_sender_epsilon_accounts.len();
        //create updated senders delta account slice
        let updated_senders_delta_account = &self.proof.delta_accounts[..senders_count];
        //extract shuffle proof
        let shuffle_proof = self.shuffle_proof.as_ref().unwrap();
        //verify the Dark Proof first
        self.proof.verify(
            &mut verifier,
            &shuffle_proof.input_dash_accounts,
            &updated_senders_delta_account,
            &shuffle_proof.updated_delta_accounts,
        )?;
        let anonymity_index = self.proof.range_proof.len();
        //verify the shuffle proof
        shuffle_proof.verify(&mut verifier, &inputs, &outputs, anonymity_index)?;

        Ok(())
    }
}
