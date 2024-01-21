#![allow(non_snake_case)]
//#![deny(missing_docs)]

use crate::proof::{DarkProof, ShuffleTxProof};
use merlin::Transcript;
use zkvm::zkos_types::{Input, Output, Witness};

use serde::{Deserialize, Serialize};
use std::sync::Arc;
//use bulletproofs::r1cs::R1CSProof;
use curve25519_dalek::scalar::Scalar;
use quisquislib::{
    accounts::prover::Prover,
    accounts::Account,
    accounts::{verifier::Verifier, SigmaProof},
    keys::PublicKey,
    ristretto::{RistrettoPublicKey, RistrettoSecretKey},
    shuffle::{Permutation, Shuffle},
};

///
/// Store for TransactionTransfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferTransaction {
    //transaction header
    pub(crate) version: u64,
    pub(crate) maturity: u64,
    pub(crate) fee: u64,
    //lengths of vectors to come
    pub(crate) input_count: u8,
    pub(crate) output_count: u8,
    pub(crate) witness_count: u8,
    //List of inputs and outputs
    pub(crate) inputs: Vec<Input>,
    pub(crate) outputs: Vec<Output>,
    //Dark Tx proof
    pub(crate) proof: DarkProof,
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

/// Utility functions for Creating the Zero balance proof as witness for newly minted reciver accounts
fn reciever_zero_balance_proof(
  // prover: &mut Prover,
    input_vector: &[Input],
    scalar_vector: &[Scalar],
    senders_count: usize,
    receivers_count: usize,
) -> Vec<Witness> {
    let mut witnesses = Vec::<Witness>::new();
    let mut scalar_index = 0;
    let reciever_inputs = input_vector[senders_count..senders_count + receivers_count].to_vec();
    for inp in reciever_inputs.iter() {
        // check if utxo exists
        if inp.get_utxo() == zkvm::Utxo::default() {
            // UTXO does not exist. So create a witness proof for the reciever account
            // get the account
            let rec = inp.to_quisquis_account().unwrap();
            //create proof
            let witness_proof =
                Prover::zero_balance_account_prover(rec, scalar_vector[scalar_index]);
            scalar_index += 1;
            witnesses.push(Witness::Proof(witness_proof));
        }
    }
    witnesses
}

/// Utility function to verify the zero balance proof for newly minted reciever accounts
///
fn verify_zero_balance_witness(
   // verifier: &mut Verifier,
    inputs: &[Input],
    witness: &Option<Vec<Witness>>,
) -> Result<(), &'static str> {
    for inp in inputs.iter() {
        if inp.get_utxo() == zkvm::Utxo::default() {
            // UTXO does not exist. Check the witness proof
            // get the account
            let rec = inp.to_quisquis_account()?;

            // witness is present
            match witness {
                Some(witnesses) => {
                    let index = inp.get_witness_index();
                    if index as usize >= witnesses.len() {
                        return Err("Tx Verification failed. Witness index is not valid.");
                    }
                    let witness_proof: SigmaProof = match witnesses[index as usize].clone() {
                        Witness::Proof(proof) => proof,
                        _ => return Err("Tx Verification failed. Witness is not valid."),
                    };
                    let (z_vector, x) = witness_proof.get_dlog();
                    Verifier::zero_balance_account_verifier(rec, z_vector[0], x)?;
                }
                None => (),// Do nothing if the witness is not present. This is the case for reciever accounts with existing utxo
            }
        }
    }
    Ok(())
}

impl TransferTransaction {
    // Private constructor
    fn set_transfer_transaction(
        version: u64,
        maturity: u64,
        fee: u64,
        input_count: u8,
        output_count: u8,
        witness_count: u8,
        inputs: Vec<Input>,
        outputs: Vec<Output>,
        proof: DarkProof,
        shuffle_proof: Option<ShuffleTxProof>,
        witness: Option<Vec<Witness>>,
    ) -> TransferTransaction {
        TransferTransaction {
            version,
            maturity,
            fee,
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

    /// Option<Scalar> carries the final scalar used in the reciever output encryption
    /// This is required to process burnMessage 
    /// is Some if the reciever is zero balance in input
    pub fn create_private_transaction(
        value_vector: &[i64],
        account_vector: &[Account],
        sender_updated_balance: &[u64],
        reciever_value_balance: &[u64],
        input_vector: &[Input],
        sender_sk: &[RistrettoSecretKey],
        senders_count: usize,
        receivers_count: usize,
        // carries the witness for zero balance reciever accounts if they exist. otherwise none
        // setting the witness index properly in the input is the resposibility of the client
        witness_comm_scalar: Option<&[Scalar]>,
        fee: u64,
    ) -> Result<(TransferTransaction, Option<Vec<Scalar>>), &'static str> {
        
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
        let dark_tx_proof = DarkProof::create_dark_ordered_proof(
            //&mut prover,
            &value_vector_scalar,
            &delta_accounts,
            &epsilon_accounts,
            &delta_rscalar,
            sender_updated_delta_account,
            &updated_delta_accounts,
            &sender_updated_balance,
            &reciever_value_balance,
            &sender_sk,
            senders_count,
            receivers_count,
            base_pk,
            Some((&output_accounts, pk_update_scalar, comm_update_scalar)),
        );

        //create vec of Outputs -- Senders + Recievers in this case
        let mut outputs: Vec<Output> = Vec::new();
        for out in output_accounts.iter() {
            outputs.push(Output::from_quisquis_account(
                out.clone(),
                address::Network::default(),
            ));
        }

        let version = 1u64;
        let maturity = 0u64;
        let input_count = input_vector.len();
        let output_count = outputs.len();

        // Create Zero account proof for Reciever accounts as witness in Tx
        // required if new account has been created for the reciever.
        // Not required if the account used for reciever is already present in the UTXO Set
        // get the reciever inputs
        match witness_comm_scalar {
            Some(scalar_vector) => {
               // create Output_account_commitment_scalar for reciever accounts. Returned back to the client. Required for burnMessage/Script Tx(esp. Order/Lend)
               // create reference for delta_rscalar of recievers
               let delta_rscalar_receiver = &delta_rscalar[senders_count..senders_count+receivers_count];
               // output account commitment scalar = input_commitment_scalar + delta_rscalar + comm_update_scalar
               let encrypt_scalar_sum_vector = delta_rscalar_receiver.iter().zip(scalar_vector.iter()).map(|(x,y)| x+y+comm_update_scalar).collect::<Vec<Scalar>>();
                 // create proof zero balance commitment for reciever accounts
                let witnesses = reciever_zero_balance_proof(
                    //&mut prover,
                    &input_vector,
                    scalar_vector,
                    senders_count,
                    receivers_count,
                );

                Ok((
                    TransferTransaction::set_transfer_transaction(
                        version,
                        maturity,
                        fee, // fee is zero for dark tx for NOW
                        input_count as u8,
                        output_count as u8,
                        witnesses.len() as u8,
                        input_vector.to_vec(),
                        outputs,
                        dark_tx_proof,
                        None,
                        Some(witnesses.to_vec()),
                    ),
                    Some(encrypt_scalar_sum_vector),
                ))
            }
            None => Ok((
                TransferTransaction::set_transfer_transaction(
                    version,
                    maturity,
                    fee, // fee is zero for dark tx for NOW
                    input_count as u8,
                    output_count as u8,
                    0u8,
                    input_vector.to_vec(),
                    outputs,
                    dark_tx_proof,
                    None,
                    None,
                ),
                None,
            )),
        }
    }
    /// Uses native threads to achieve parallelism
    pub fn create_private_transaction_parallel(
        value_vector: &[i64],
        account_vector: &[Account],
        sender_updated_balance: &[u64],
        reciever_value_balance: &[u64],
        input_vector: &[Input],
        sender_sk: &[RistrettoSecretKey],
        senders_count: usize,
        receivers_count: usize,
        // carries the witness for zero balance reciever accounts if they exist. otherwise none
        // setting the witness index properly in the input is the resposibility of the client
        witness_comm_scalar: Option<&[Scalar]>,
        fee: u64,
    ) -> Result<(TransferTransaction, Option<Vec<Scalar>>), &'static str> {
        
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

        //create delta_and_epsilon_accounts
        let (delta_accounts, epsilon_accounts, delta_rscalar) =
            Account::create_delta_and_epsilon_accounts(
                &account_vector,
                &value_vector_scalar,
                base_pk,
            );
        // create reference for delta_rscalar of recievers to create output_account_commitment_scalar
        let delta_rscalar_receiver = &delta_rscalar[senders_count..senders_count+receivers_count].to_vec();
        
        //identity check function to verify the construction of epsilon accounts using correct rscalars
        Verifier::verify_delta_identity_check(&epsilon_accounts)?;

        //update delta_accounts to reflect the change in balance
        //updated_delta_accounts = Output account for DarkTx
        let updated_delta_accounts =
            Account::update_delta_accounts(&account_vector, &delta_accounts)?;
        let sender_updated_delta_account = &updated_delta_accounts[..senders_count];

        // update the delta_updated_accounts to create output for dark tx
        let mut transcript_rng = rand::thread_rng();
        let pk_update_scalar = Scalar::random(&mut transcript_rng);
        let comm_update_scalar = Scalar::random(&mut transcript_rng);

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
        // convert vectors into Arc for shared referencing in threads
        let value_vector_scalar = Arc::new(value_vector_scalar);
        let delta_accounts = Arc::new(delta_accounts);
        let epsilon_accounts = Arc::new(epsilon_accounts);
        let delta_rscalar: Arc<Vec<Scalar>> = Arc::new(delta_rscalar);
        let sender_updated_delta_account = Arc::new(sender_updated_delta_account.to_vec());
        let updated_delta_accounts = Arc::new(updated_delta_accounts);
        let sender_sk = Arc::new(sender_sk.to_vec());
        let output_accounts_proof = Arc::new(output_accounts.clone());

        let dark_tx_proof = DarkProof::create_dark_ordered_proof_parallel(
            value_vector_scalar,
            delta_accounts,
            epsilon_accounts,
            delta_rscalar,
            sender_updated_delta_account,
            updated_delta_accounts,
            &sender_updated_balance,
            &reciever_value_balance,
            sender_sk,
            senders_count,
            receivers_count,
            base_pk,
            Some((output_accounts_proof, pk_update_scalar, comm_update_scalar)),
        )?;

        //create vec of Outputs -- Senders + Recievers in this case
        let mut outputs: Vec<Output> = Vec::new();
        for out in output_accounts.iter() {
            outputs.push(Output::from_quisquis_account(
                out.clone(),
                address::Network::default(),
            ));
        }

        let version = 1u64;
        let maturity = 0u64;
        let input_count = input_vector.len();
        let output_count = outputs.len();

        // Create Zero account proof for Reciever accounts as witness in Tx
        // required if new account has been created for the reciever.
        // Not required if the account used for reciever is already present in the UTXO Set
        // get the reciever inputs
        match witness_comm_scalar {
            Some(scalar_vector) => {
                // create Output_account_commitment_scalar for reciever accounts. Returned back to the client. Required for burnMessage/Script Tx(esp. Order/Lend)
               // output account commitment scalar = input_commitment_scalar + delta_rscalar + comm_update_scalar
               let encrypt_scalar_sum_vector = delta_rscalar_receiver.iter().zip(scalar_vector.iter()).map(|(x,y)| x+y+comm_update_scalar).collect::<Vec<Scalar>>();
                 // create proof zero balance commitment for reciever accounts
                let witnesses = reciever_zero_balance_proof(
                    //&mut prover,
                    &input_vector,
                    scalar_vector,
                    senders_count,
                    receivers_count,
                );

                Ok((
                    TransferTransaction::set_transfer_transaction(
                        version,
                        maturity,
                        fee, // fee is zero for dark tx for NOW
                        input_count as u8,
                        output_count as u8,
                        witnesses.len() as u8,
                        input_vector.to_vec(),
                        outputs,
                        dark_tx_proof,
                        None,
                        Some(witnesses.to_vec()),
                    ),
                    Some(encrypt_scalar_sum_vector),
                ))
            }
            None => Ok((
                TransferTransaction::set_transfer_transaction(
                    version,
                    maturity,
                    fee, // fee is zero for dark tx for NOW
                    input_count as u8,
                    output_count as u8,
                    0u8,
                    input_vector.to_vec(),
                    outputs,
                    dark_tx_proof,
                    None,
                    None,
                ),
                None,
            )),
        }
    }
    pub fn verify_dark_tx(
        &self,
        input_accounts: &[Account],
        // Outpus is updated in case of Dark Tx. So we need to pass the updated output accounts
        // None in case of Quisquis Tx
        output_accounts: Option<&[Account]>,
    ) -> Result<(), &'static str> {

        //verify the dark tx proof
        self.proof
            .verify(&input_accounts, output_accounts)?;

        //verify the witnesses if they exist
        // check for inputs with utxo::default()
        // get inputs first
        let inputs = self.inputs.clone();
        //verify the zero balance proof for reciever accounts
        verify_zero_balance_witness(&inputs, &self.witness)?; // existance check is inside the function
        Ok(())
    }

    fn set_quisquis_input_output_prover(
        output_account_vector: &[Account],
        // shuffled input accounts
        input_account_vector: &[Account],
        // initial inputs recieved from Client. Arranged as [sender..reciever..anonymity]
        initial_inputs: &[Input],
        // permutation function applied for input shuffling
        input_permutation: Permutation,
        // network type for addresses
        network: address::Network,
    ) -> (Vec<Input>, Vec<Output>) {
        //get the permutation that was applied on accounts to create input accounts
        let inverse_permutation = input_permutation.invert_permutation();
        // apply the permutation on initial Inputs
        //Convert Matrix to Vector in row major order
        let permutation = inverse_permutation.as_row_major();

        //shuffle Input accounts using permutation matrix
        let shuffled_inputs: Vec<_> = (0..initial_inputs.len())
            .map(|i| initial_inputs[permutation[i] - 1].clone())
            .collect();

        // check if permutation is correct
        for i in 0..shuffled_inputs.len() {
            let sender = input_account_vector[i].clone();
            //let reciever = account_vector[senders_count].clone();
            let sender_input = shuffled_inputs[i].clone();
            let sender_input_account = sender_input.to_quisquis_account().unwrap();
            assert_eq!(sender, sender_input_account);
        }

        //create vec of Outputs -- Recievers in this case
        let mut outputs: Vec<Output> = Vec::new();
        for out in output_account_vector.iter() {
            let out = Output::from_quisquis_account(out.clone(), network);
            outputs.push(out.clone());
        }
        (shuffled_inputs, outputs)
    }
    /// Create a Quisquis tx .
    /// This is a special case of Transfer Tx where the anonymity set is obtained from utxo set itself
    pub fn create_quisquis_transaction(
        inputs: &[Input], // input vector as received from the client (may include zero utxo for reciever/s)
        value_vector: &[i64],
        account_vector: &[Account],
        sender_updated_balance: &[u64],
        reciever_value_balance: &[u64],
        sender_sk: &[RistrettoSecretKey],
        senders_count: usize,
        receivers_count: usize,
        //anonymity_comm_scalar: &[Scalar],
        anonymity_account_diff: usize,
        // carries the witness proofs for zero balance reciever accounts if they exist. otherwise none
        // setting the witness index properly in the input is the resposibility of the client
        witness_comm_scalar: Option<&[Scalar]>,
        fee: u64,
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

        //get vec of Input Accounts arranged randomly
        let input_account_vector = input_shuffle.get_inputs_vector();

        // get vector of Input' accounts updated and arranged as [sender..reciever..anonymity]
        let input_dash_accounts = input_shuffle.get_outputs_vector();

        //create QuisQuisTx Prover merlin transcript
     //   let mut transcript = Transcript::new(b"TxProof");
      //  let mut prover = Prover::new(b"QuisQuisTx", &mut transcript);

        // Step 2. Create delta_and_epsilon_accounts
        let (delta_accounts, epsilon_accounts, delta_rscalar) =
            Account::create_delta_and_epsilon_accounts(
                &input_dash_accounts,
                &value_vector_scalar,
                base_pk,
            );

        //Step 3. identity check function to verify the construction of epsilon accounts using correct rscalars
        Verifier::verify_delta_identity_check(&epsilon_accounts)?;

        // Step 4. update delta_accounts to reflect the change in balance
        let updated_delta_accounts =
            Account::update_delta_accounts(&input_dash_accounts, &delta_accounts)?;

        let sender_updated_delta_account = &updated_delta_accounts[..senders_count];

        // Step 5. create Dark Proof. Entails proofs for
        // 1. correct construction of epsilon and delta accounts (DLEQ)
        // 2. correct construction of updated delta accounts
        // 3. Knowledge of secret key for senders and correct update to their balance (DLOG)
        // 4. Range proof on the updated sender balance and reciever values
        // 5. Zero balance proof in case of new account creation for reciever
        let dark_tx_proof = DarkProof::create_dark_ordered_proof(
            &value_vector_scalar,
            &delta_accounts,
            &epsilon_accounts,
            &delta_rscalar,
            &sender_updated_delta_account,
            &updated_delta_accounts,
            &sender_updated_balance,
            &reciever_value_balance,
            &sender_sk,
            senders_count,
            receivers_count,
            base_pk,
            // Should always be none for Quisquis Tx. Only required for Dark Tx
            None,
        );
        // assuming the number of accounts to be 9
        let anonymity_index = 9 - anonymity_account_diff;
        // println!("anonymity index: {}", anonymity_index);
        // get a list of anonymity accounts in the input' vector
        let input_dash_accounts_anonymity_slice = &input_dash_accounts[anonymity_index..9];
        // get a list of anonymity accounts in the updated delta accounts vector
        let updated_delta_accounts_anonymity_slice = &updated_delta_accounts[anonymity_index..9];
        // get of list of scalar witnesses for anonymity accounts in delta and epsilon accounts vector
        let rscalars_anonymity_slice = &delta_rscalar[anonymity_index..9];
        //for anonymity zero account proof. Not needed anymore
        //let input_anonymity_account_slice = &account_vector[anonymity_index..9];
        //Shuffle accounts
        let output_shuffle = Shuffle::output_shuffle(&updated_delta_accounts)?;

        let shuffle_proof = ShuffleTxProof::create_shuffle_proof_serial(
            &input_dash_accounts_anonymity_slice,
            &updated_delta_accounts_anonymity_slice,
            &rscalars_anonymity_slice,
            input_shuffle.clone(),
            output_shuffle.clone(),
        );

        let output_final = output_shuffle.get_outputs_vector();
        // Create Zero account proof for Reciever accounts as witness in Tx
        // required if new account has been created for the reciever.
        // Not required if the account used for reciever is already present in the UTXO Set
        // get the reciever inputs
        match witness_comm_scalar {
            Some(scalar_vector) => {
                let witnesses = reciever_zero_balance_proof(
                 //   &mut prover,
                    &inputs,
                    scalar_vector,
                    senders_count,
                    receivers_count,
                );

                // create vec of shuffled Inputs and Outputs.
                // This comes after Witnesses are created because the witness index is set in the input for recievers
                let (shuffled_inputs, outputs) = Self::set_quisquis_input_output_prover(
                    &output_final,
                    &input_account_vector,
                    &inputs,
                    input_shuffle.get_permutation().to_owned(),
                    address::Network::default(),
                );
                Ok(TransferTransaction::set_transfer_transaction(
                    0u64,
                    0u64,
                    fee, // fee is zero for quisquis tx for NOW
                    9u8,
                    9u8,
                    witnesses.len() as u8,
                    shuffled_inputs,
                    outputs,
                    dark_tx_proof,
                    Some(shuffle_proof),
                    Some(witnesses.to_vec()),
                ))
            }
            None => {
                // create vec of shuffled Inputs and Outputs.
                // This comes after Witnesses are created because the witness index is set in the input for recievers
                let (shuffled_inputs, outputs) = Self::set_quisquis_input_output_prover(
                    &output_final,
                    &input_account_vector,
                    &inputs,
                    input_shuffle.get_permutation().to_owned(),
                    address::Network::default(),
                );
                Ok(TransferTransaction::set_transfer_transaction(
                    0u64,
                    0u64,
                    fee, // fee is zero for quisquis tx for NOW
                    9u8,
                    9u8,
                    0u8,
                    shuffled_inputs,
                    outputs,
                    dark_tx_proof,
                    Some(shuffle_proof),
                    None,
                ))
            }
        }
    }

      /// This is a special case of Transfer Tx where the anonymity set is obtained from utxo set itself
      pub fn create_quisquis_transaction_parallel(
        inputs: &[Input], // input vector as received from the client (may include zero utxo for reciever/s)
        value_vector: &[i64],
        account_vector: &[Account],
        sender_updated_balance: &[u64],
        reciever_value_balance: &[u64],
        sender_sk: &[RistrettoSecretKey],
        senders_count: usize,
        receivers_count: usize,
        //anonymity_comm_scalar: &[Scalar],
        anonymity_account_diff: usize,
        // carries the witness proofs for zero balance reciever accounts if they exist. otherwise none
        // setting the witness index properly in the input is the resposibility of the client
        witness_comm_scalar: Option<&[Scalar]>,
        fee: u64,
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

        //get vec of Input Accounts arranged randomly
        let input_account_vector = input_shuffle.get_inputs_vector();

        // get vector of Input' accounts updated and arranged as [sender..reciever..anonymity]
        let input_dash_accounts = input_shuffle.get_outputs_vector();

        //create QuisQuisTx Prover merlin transcript
     //   let mut transcript = Transcript::new(b"TxProof");
      //  let mut prover = Prover::new(b"QuisQuisTx", &mut transcript);

        // Step 2. Create delta_and_epsilon_accounts
        let (delta_accounts, epsilon_accounts, delta_rscalar) =
            Account::create_delta_and_epsilon_accounts(
                &input_dash_accounts,
                &value_vector_scalar,
                base_pk,
            );
        let delta_rscalar_clone= delta_rscalar.clone();
        //Step 3. identity check function to verify the construction of epsilon accounts using correct rscalars
        Verifier::verify_delta_identity_check(&epsilon_accounts)?;

        // Step 4. update delta_accounts to reflect the change in balance
        let updated_delta_accounts =
            Account::update_delta_accounts(&input_dash_accounts, &delta_accounts)?;
            let updated_delta_accounts_clone = updated_delta_accounts.clone();

        let sender_updated_delta_account = &updated_delta_accounts[..senders_count];

        // Step 5. create Dark Proof. Entails proofs for
        // 1. correct construction of epsilon and delta accounts (DLEQ)
        // 2. correct construction of updated delta accounts
        // 3. Knowledge of secret key for senders and correct update to their balance (DLOG)
        // 4. Range proof on the updated sender balance and reciever values
        // 5. Zero balance proof in case of new account creation for reciever
         // convert vectors into Arc for shared referencing in threads
         let value_vector_scalar = Arc::new(value_vector_scalar);
         let delta_accounts = Arc::new(delta_accounts);
         let epsilon_accounts = Arc::new(epsilon_accounts);
         let delta_rscalar_dark: Arc<Vec<Scalar>> = Arc::new(delta_rscalar);
         let sender_updated_delta_account = Arc::new(sender_updated_delta_account.to_vec());
         let updated_delta_accounts_dark = Arc::new(updated_delta_accounts);
         let sender_sk = Arc::new(sender_sk.to_vec());
        let dark_tx_proof = DarkProof::create_dark_ordered_proof_parallel(
            value_vector_scalar,
            delta_accounts,
            epsilon_accounts,
            delta_rscalar_dark,
            sender_updated_delta_account,
            updated_delta_accounts_dark,
            sender_updated_balance,
            reciever_value_balance,
            sender_sk,
            senders_count,
            receivers_count,
            base_pk,
            // Should always be none for Quisquis Tx. Only required for Dark Tx
            None,
        )?;
        // assuming the number of accounts to be 9
        let anonymity_index = 9 - anonymity_account_diff;
        // println!("anonymity index: {}", anonymity_index);
        // get a list of anonymity accounts in the input' vector
        let input_dash_accounts_anonymity_slice = &input_dash_accounts[anonymity_index..9];
        // get a list of anonymity accounts in the updated delta accounts vector
        let updated_delta_accounts_anonymity_slice = &updated_delta_accounts_clone[anonymity_index..9];
        // get of list of scalar witnesses for anonymity accounts in delta and epsilon accounts vector
        let rscalars_anonymity_slice = &delta_rscalar_clone[anonymity_index..9];
        //for anonymity zero account proof. Not needed anymore
        //let input_anonymity_account_slice = &account_vector[anonymity_index..9];
        //Shuffle accounts
        let output_shuffle = Shuffle::output_shuffle(&updated_delta_accounts_clone)?;

        let shuffle_proof = ShuffleTxProof::create_shuffle_proof_parallel(
            &input_dash_accounts_anonymity_slice,
            &updated_delta_accounts_anonymity_slice,
            &rscalars_anonymity_slice,
            input_shuffle.clone(),
            output_shuffle.clone(),
        )?;

        let output_final = output_shuffle.get_outputs_vector();
        // Create Zero account proof for Reciever accounts as witness in Tx
        // required if new account has been created for the reciever.
        // Not required if the account used for reciever is already present in the UTXO Set
        // get the reciever inputs
        match witness_comm_scalar {
            Some(scalar_vector) => {
                let witnesses = reciever_zero_balance_proof(
                 //   &mut prover,
                    &inputs,
                    scalar_vector,
                    senders_count,
                    receivers_count,
                );

                // create vec of shuffled Inputs and Outputs.
                // This comes after Witnesses are created because the witness index is set in the input for recievers
                let (shuffled_inputs, outputs) = Self::set_quisquis_input_output_prover(
                    &output_final,
                    &input_account_vector,
                    &inputs,
                    input_shuffle.get_permutation().to_owned(),
                    address::Network::default(),
                );
                Ok(TransferTransaction::set_transfer_transaction(
                    0u64,
                    0u64,
                    fee, // fee is zero for quisquis tx for NOW
                    9u8,
                    9u8,
                    witnesses.len() as u8,
                    shuffled_inputs,
                    outputs,
                    dark_tx_proof,
                    Some(shuffle_proof),
                    Some(witnesses.to_vec()),
                ))
            }
            None => {
                // create vec of shuffled Inputs and Outputs.
                // This comes after Witnesses are created because the witness index is set in the input for recievers
                let (shuffled_inputs, outputs) = Self::set_quisquis_input_output_prover(
                    &output_final,
                    &input_account_vector,
                    &inputs,
                    input_shuffle.get_permutation().to_owned(),
                    address::Network::default(),
                );
                Ok(TransferTransaction::set_transfer_transaction(
                    0u64,
                    0u64,
                    fee, // fee is zero for quisquis tx for NOW
                    9u8,
                    9u8,
                    0u8,
                    shuffled_inputs,
                    outputs,
                    dark_tx_proof,
                    Some(shuffle_proof),
                    None,
                ))
            }
        }
    }

    pub fn verify_quisquis_tx(
        &self,
        inputs: &[Account],
        outputs: &[Account],
    ) -> Result<(), &'static str> {
        //create QuisQUisTx Prover merlin transcript
       // let mut transcript = Transcript::new(b"TxProof");
       // let mut verifier = Verifier::new(b"QuisQuisTx", &mut transcript);

        //find the number of senders
        // let senders_count = self.proof.updated_sender_epsilon_accounts.len();
        //create updated senders delta account slice
        // let updated_senders_delta_account = &self.proof.delta_accounts[..senders_count];
        //extract shuffle proof
        let shuffle_proof = self.shuffle_proof.as_ref().unwrap();

        //verify the Dark Proof first
        self.proof
            .verify(&shuffle_proof.input_dash_accounts, None)?;
        //let anonymity_index = self.proof.range_proof.len();
        //verify the shuffle proof
        shuffle_proof.verify(
            &inputs,
            &outputs,
            &self.proof.updated_delta_accounts,
            // anonymity_index,
        )?;
        //verify the witnesses if they exist
        // check for inputs with utxo::default()
        // get inputs first
        let inputs = self.inputs.clone();
        //verify the zero balance proof for reciever accounts
        verify_zero_balance_witness(&inputs, &self.witness)?;

        Ok(())
    }

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
            self.verify_quisquis_tx(&input_accounts, &output_accounts)
            //Err("Tx Verification failed. Transaction Type is not valid.")
        }
    }
}
