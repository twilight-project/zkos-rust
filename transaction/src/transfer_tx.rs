#![allow(non_snake_case)]
//#![deny(missing_docs)]

use crate::proof::{DarkTxProof, ShuffleTxProof};
use crate::script_tx::ScriptTransaction;
use merlin::Transcript;
use zkvm::zkos_types::{Input, Output, Witness};

use serde::{Deserialize, Serialize};

//use bulletproofs::r1cs::R1CSProof;
use curve25519_dalek::scalar::Scalar;
use quisquislib::{
    accounts::prover::Prover,
    accounts::verifier::Verifier,
    accounts::Account,
    keys::PublicKey,
    ristretto::{RistrettoPublicKey, RistrettoSecretKey},
    // shuffle::Shuffle,
};

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
        let sender_updated_delta_account = &updated_delta_accounts[..senders_count];

        // update the delta_updated_accounts to create output for dark tx
        let pk_update_scalar = Scalar::random(&mut rand::rngs::OsRng);
        let comm_update_scalar = Scalar::random(&mut rand::rngs::OsRng);

        let output_accounts = updated_delta_accounts
            .iter()
            .map(|account| {
                Account::update_account(
                    *account,
                    Scalar::zero(),
                    pk_update_scalar,
                    comm_update_scalar,
                )
            })
            .collect::<Vec<Account>>();
        // create dark tx proof including the updated output accounts proof
        let dark_tx_proof = DarkTxProof::create_dark_tx_proof(
            &mut prover,
            &value_vector_scalar,
            &delta_accounts,
            &epsilon_accounts,
            &delta_rscalar,
            sender_updated_delta_account,
            &updated_delta_accounts,
            &sender_updated_balance,
            &reciever_updated_balance,
            &sender_sk,
            senders_count,
            receivers_count,
            base_pk,
            Some((&output_accounts, pk_update_scalar, comm_update_scalar)),
        );

        // Create ZERO account proof as witness in Tx
        // create proof for recievers.
        //CONFIRM FROM GAURAV

        //create vec of Outputs -- Senders + Recievers in this case
        let mut outputs: Vec<Output> = Vec::new();
        for out in output_accounts.iter() {
            outputs.push(Output::from(out.clone()));
        }

        let version = 1u64;
        let maturity = 0u64;
        let input_count = input_vector.len();
        let output_count = outputs.len();

        Ok(TransferTransaction::set_tranfer_transaction(
            version,
            maturity,
            input_count as u8,
            output_count as u8,
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
        // Outpus is updated in case of Dark Tx. So we need to pass the updated output accounts
        // None in case of Quisquis Tx
        outputs: Option<&[Account]>,
    ) -> Result<(), &'static str> {
        //create DarkTx Prover merlin transcript
        let mut transcript = Transcript::new(b"TxProof");
        let mut verifier = Verifier::new(b"DarkTx", &mut transcript);

        //find the number of senders
        // let senders_count = self.proof.updated_sender_epsilon_accounts.len();
        //create updated senders delta account slice
        // let updated_senders_delta_account = &self.proof.delta_accounts[..senders_count];
        //TODO::CONVERT INPUS AND OUTPUTS TO ACCOUNTS
        //verify the proof
        self.proof
            .verify(&mut verifier, &inputs.to_vec(), outputs)?;
        Ok(())
    }

    // pub fn create_quisquis_transaction(
    //     utxo_vector: &[Utxo],
    //     value_vector: &[i64],
    //     account_vector: &[Account],
    //     sender_updated_balance: &[u64],
    //     reciever_updated_balance: &[u64],
    //     sender_sk: &[RistrettoSecretKey],
    //     senders_count: usize,
    //     receivers_count: usize,
    //     anonymity_comm_scalar: &[Scalar],
    //     anonymity_account_diff: usize,
    // ) -> Result<TransferTransaction, &'static str> {
    //     //convert the valur vector into scalar type to create the proof
    //     let mut value_vector_scalar = Vec::<Scalar>::new();
    //     for v in value_vector.iter() {
    //         if v >= &0 {
    //             value_vector_scalar.push(Scalar::from(*v as u64));
    //         } else {
    //             value_vector_scalar.push(-Scalar::from((-*v) as u64));
    //         }
    //     }
    //     //create base pk for epsilon accounts
    //     let base_pk = RistrettoPublicKey::generate_base_pk();
    //     //Step 1. update & shuffle input accounts
    //     let input_shuffle = Shuffle::input_shuffle(account_vector)?;
    //     let updated_accounts = input_shuffle.get_outputs_vector();
    //     //create Inputs from shuffle
    //     //create vec of Inputs
    //     let input_account_vector = input_shuffle.get_inputs_vector();

    //     //THE UTXO SHOULD BE SHUFFLED ACCORDINGLY LATER. USING UTXO as zero for the time being
    //     let mut inputs: Vec<Input> = Vec::new();
    //     for (i, input) in input_account_vector.iter().enumerate() {
    //         //create inputs
    //         let (pk, enc) = input.get_account();
    //         let out_coin = OutputCoin {
    //             encrypt: enc,
    //             owner: Address::standard_address(Network::default(), pk).as_hex(),
    //         };
    //         let inp = Input::coin(InputData::coin(
    //             utxo_vector[i],
    //             // Address::coin_address(Network::default(), pk).as_hex(),
    //             // enc,
    //             out_coin,
    //             0,
    //         ));
    //         inputs.push(inp.clone());
    //     }

    //     //create QuisQuisTx Prover merlin transcript
    //     let mut transcript = Transcript::new(b"TxProof");
    //     let mut prover = Prover::new(b"QuisQuisTx", &mut transcript);

    //     //create delta_and_epsilon_accounts
    //     let (delta_accounts, epsilon_accounts, delta_rscalar) =
    //         Account::create_delta_and_epsilon_accounts(
    //             &updated_accounts,
    //             &value_vector_scalar,
    //             base_pk,
    //         );

    //     //identity check function to verify the construction of epsilon accounts using correct rscalars
    //     Verifier::verify_delta_identity_check(&epsilon_accounts)?;

    //     //update delta_accounts to reflect the change in balance
    //     //updated_delta_accounts = Output account for DarkTx
    //     let updated_delta_accounts =
    //         Account::update_delta_accounts(&updated_accounts, &delta_accounts)?;

    //     let sender_updated_delta_account = &updated_delta_accounts[..senders_count];
    //     //create Dark Proof
    //     let dark_tx_proof = DarkTxProof::create_dark_tx_proof(
    //         &mut prover,
    //         &value_vector_scalar,
    //         &delta_accounts,
    //         &epsilon_accounts,
    //         &delta_rscalar,
    //         &sender_updated_delta_account,
    //         &sender_updated_balance,
    //         &reciever_updated_balance,
    //         &sender_sk,
    //         senders_count,
    //         receivers_count,
    //         base_pk,
    //         None,
    //     );

    //     let anonymity_index = 9 - anonymity_account_diff;
    //     let updated_accounts_slice = &updated_accounts[anonymity_index..9];
    //     let updated_delta_accounts_slice = &updated_delta_accounts[anonymity_index..9];
    //     let rscalars_slice = &delta_rscalar[anonymity_index..9];
    //     //for anonymity zero account proof
    //     let input_anonymity_account_slice = &account_vector[anonymity_index..9];
    //     //Shuffle accounts
    //     let output_shuffle = Shuffle::output_shuffle(&updated_delta_accounts)?;

    //     let shuffle_proof = ShuffleTxProof::create_shuffle_proof(
    //         &mut prover,
    //         &updated_accounts,
    //         &updated_delta_accounts,
    //         &delta_rscalar,
    //         &input_anonymity_account_slice,
    //         &anonymity_comm_scalar,
    //         &input_shuffle,
    //         &output_shuffle,
    //     );

    //     let output_final = output_shuffle.get_outputs_vector();
    //     //create vec of Outputs -- Recievers in this case
    //     let mut outputs: Vec<Output> = Vec::new();
    //     for out in output_final.iter() {
    //         //create address
    //         let (pk, comm) = out.get_account();
    //         let coin: OutputCoin = OutputCoin {
    //             encrypt: comm,
    //             owner: Address::standard_address(Network::default(), pk).as_hex(),
    //         };
    //         let out = Output::coin(OutputData::coin(coin));
    //         outputs.push(out.clone());
    //     }

    //     Ok(TransferTransaction::set_tranfer_transaction(
    //         0u64,
    //         0u64,
    //         senders_count as u8,
    //         receivers_count as u8,
    //         0u8,
    //         inputs,
    //         outputs,
    //         dark_tx_proof,
    //         Some(shuffle_proof),
    //         None,
    //     ))
    // }

    // pub fn verify_quisquis_tx(
    //     &self,
    //     inputs: &[Account],
    //     outputs: &[Account],
    // ) -> Result<(), &'static str> {
    //     //create QuisQUisTx Prover merlin transcript
    //     let mut transcript = Transcript::new(b"TxProof");
    //     let mut verifier = Verifier::new(b"QuisQuisTx", &mut transcript);

    //     //find the number of senders
    //     // let senders_count = self.proof.updated_sender_epsilon_accounts.len();
    //     //create updated senders delta account slice
    //     // let updated_senders_delta_account = &self.proof.delta_accounts[..senders_count];
    //     //extract shuffle proof
    //     let shuffle_proof = self.shuffle_proof.as_ref().unwrap();
    //     //verify the Dark Proof first
    //     self.proof.verify(
    //         &mut verifier,
    //         &shuffle_proof.input_dash_accounts,
    //         //  &updated_senders_delta_account,
    //         &shuffle_proof.updated_delta_accounts,
    //     )?;
    //     let anonymity_index = self.proof.range_proof.len();
    //     //verify the shuffle proof
    //     shuffle_proof.verify(&mut verifier, &inputs, &outputs, anonymity_index)?;

    //     Ok(())
    // }

    //created for utxo-in-memory
    pub fn get_input_values(&self) -> Vec<Input> {
        self.inputs.clone()
    }
    pub fn get_output_values(&self) -> Vec<Output> {
        self.outputs.clone()
    }

    pub fn verify(&self) -> Result<(), &'static str> {
        //convert Inputs and Outputs to Just Accounts
        let inputs = self.get_input_values();
        let outputs = self.get_output_values();

        let mut input_accounts = Vec::<Account>::new();
        let mut output_accounts = Vec::<Account>::new();

        for (inp, out) in inputs.iter().zip(outputs.iter()) {
            let inp_acc = inp.to_quisquis_account()?;
            let out_acc = out.to_quisquis_account()?;
            input_accounts.push(inp_acc);
            output_accounts.push(out_acc);
        }

        if self.shuffle_proof.is_none() {
            //verify Dark transaction
            self.verify_dark_tx(&input_accounts, Some(&output_accounts))
        } else {
            //verify QQ Transaction
            // tx_data.verify_quisquis_tx(&inputs, &outputs)
            Err("Tx Verification failed. Transaction Type is not valid.")
        }
    }
}
