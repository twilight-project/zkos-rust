#![allow(non_snake_case)]

//! Confidential transfer transaction implementation.
//!
//! This module provides transfer transaction functionality with two privacy modes:
//! - **Dark Transactions**: Fully confidential transfers with delta/epsilon proofs
//! - **QuisQuis Transactions**: Shuffled transfers with enhanced privacy through UTXO set mixing
//!
//! # Overview
//!
//! Transfer transactions enable confidential value transfers while maintaining:
//! - **Privacy**: Amounts and sender/receiver relationships are hidden
//! - **Verifiability**: Cryptographic proofs ensure transaction validity
//! - **Balance Conservation**: Total input equals total output
//!
//! # Example
//! ```
//! use transaction::TransferTransaction;
//! use quisquislib::ristretto::RistrettoSecretKey;
//! use zkvm::zkos_types::{Input, Output};
//!
//! // Create a QuisQuis transfer
//! let transfer_tx = TransferTransaction::create_quisquis_transaction(
//!     &inputs,
//!     &value_vector,
//!     &account_vector,
//!     &sender_updated_balance,
//!     &receiver_value_balance,
//!     &sender_sk,
//!     senders_count,
//!     receivers_count,
//!     anonymity_account_diff,
//!     witness_comm_scalar,
//!     fee,
//! ).unwrap();
//!
//! // Verify the transaction
//! assert!(transfer_tx.verify().is_ok());
//! ```

use crate::proof::{DarkTxProof, ShuffleTxProof};
use merlin::Transcript;
use zkvm::zkos_types::{Input, Output, Witness};

use serde::{Deserialize, Serialize};

use curve25519_dalek::scalar::Scalar;
use quisquislib::{
    accounts::prover::Prover,
    accounts::Account,
    accounts::{verifier::Verifier, SigmaProof},
    keys::PublicKey,
    ristretto::{RistrettoPublicKey, RistrettoSecretKey},
    shuffle::{Permutation, Shuffle},
};

/// Confidential transfer transaction with privacy proofs.
///
/// Supports both Dark and QuisQuis transaction types with zero-knowledge
/// proofs for amount privacy and balance conservation.
///
/// # Fields
/// - `version`: Transaction version
/// - `maturity`: Block height when transaction becomes valid
/// - `fee`: Transaction fee
/// - `inputs/outputs`: Transaction inputs and outputs
/// - `proof`: Dark transaction proof (delta/epsilon accounts)
/// - `shuffle_proof`: Optional shuffle proof for QuisQuis transactions
/// - `witness`: Optional witness proofs for new receiver accounts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferTransaction {
    pub(crate) version: u64,
    pub(crate) maturity: u64,
    pub(crate) fee: u64,
    pub(crate) input_count: u8,
    pub(crate) output_count: u8,
    pub(crate) witness_count: u8,
    pub(crate) inputs: Vec<Input>,
    pub(crate) outputs: Vec<Output>,
    pub(crate) proof: DarkTxProof,
    pub(crate) shuffle_proof: Option<ShuffleTxProof>,
    pub(crate) witness: Option<Vec<Witness>>,
}

/// Creates zero balance proofs for newly minted receiver accounts.
///
/// When a receiver account doesn't exist in the UTXO set, we need to prove
/// it has zero balance before receiving funds.
fn reciever_zero_balance_proof(
    prover: &mut Prover,
    input_vector: &[Input],
    scalar_vector: &[Scalar],
    senders_count: usize,
    receivers_count: usize,
) -> Vec<Witness> {
    let mut witnesses = Vec::<Witness>::new();
    let mut scalar_index = 0;
    let reciever_inputs = input_vector[senders_count..senders_count + receivers_count].to_vec();

    for inp in reciever_inputs.iter() {
        if inp.get_utxo() == zkvm::Utxo::default() {
            // UTXO does not exist. So create a witness proof for the reciever account
            // get the account
            let rec = inp.to_quisquis_account().unwrap();
            let witness_proof =
                Prover::zero_balance_account_prover(rec, scalar_vector[scalar_index], prover);
            scalar_index += 1;
            witnesses.push(Witness::Proof(witness_proof));
        }
    }
    witnesses
}

/// Verifies zero balance proofs for receiver accounts.
fn verify_zero_balance_witness(
    verifier: &mut Verifier,
    inputs: &[Input],
    witness: &Option<Vec<Witness>>,
) -> Result<(), &'static str> {
    for inp in inputs.iter() {
        if inp.get_utxo() == zkvm::Utxo::default() {
            // UTXO does not exist. Check the witness proof
            // get the account
            let rec = inp.to_quisquis_account()?;

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
                    Verifier::zero_balance_account_verifier(rec, z_vector[0], x, verifier)?;
                }
                None => {
                    // Receiver accounts already exist in UTXO set
                }
            }
        }
    }
    Ok(())
}

impl TransferTransaction {
    /// Private constructor for transfer transactions.
    fn set_transfer_transaction(
        version: u64,
        maturity: u64,
        fee: u64,
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

    /// Creates a Dark transaction (fully confidential transfer).
    ///
    /// Dark transactions provide complete by hiding all transaction
    /// input and output values behind encryption.
    ///
    /// # Arguments
    /// * `value_vector` - Transfer amounts (positive for senders, negative for receivers)
    /// * `account_vector` - QuisQuis accounts for all participants
    /// * `sender_updated_balance` - New balances for sender accounts
    /// * `receiver_value_balance` - Final balances for receiver accounts
    /// * `input_vector` - Transaction inputs
    /// * `sender_sk` - Secret keys for sender accounts
    /// * `senders_count` - Number of sender accounts
    /// * `receivers_count` - Number of receiver accounts
    /// * `witness_comm_scalar` - Optional scalars for new receiver accounts
    /// * `fee` - Transaction fee
    ///
    /// # Returns
    /// `Ok((TransferTransaction, Option<Vec<Scalar>>))` - Transaction and optional encryption scalars
    ///    Option<Scalar> carries the final scalar used in the reciever output encryption
    /// This is required to process burnMessage
    /// is Some if the reciever is zero balance in input
    ///
    /// # Example
    /// ```
    /// use transaction::TransferTransaction;
    /// use quisquislib::ristretto::RistrettoSecretKey;
    ///
    /// let (tx, encrypt_scalars) = TransferTransaction::create_private_transfer_transaction(
    ///     &value_vector,
    ///     &account_vector,
    ///     &sender_updated_balance,
    ///     &receiver_value_balance,
    ///     &input_vector,
    ///     &sender_sk,
    ///     senders_count,
    ///     receivers_count,
    ///     witness_comm_scalar,
    ///     fee,
    /// ).unwrap();
    /// ```
    pub fn create_private_transfer_transaction(
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
        // Convert value vector to scalars
        let mut value_vector_scalar = Vec::<Scalar>::new();
        for v in value_vector.iter() {
            if v >= &0 {
                value_vector_scalar.push(Scalar::from(*v as u64));
            } else {
                value_vector_scalar.push(-Scalar::from((-*v) as u64));
            }
        }

        let base_pk = RistrettoPublicKey::generate_base_pk();
        let mut transcript = Transcript::new(b"TxProof");
        let mut prover = Prover::new(b"DarkTx", &mut transcript);

        // Create delta and epsilon accounts
        let (delta_accounts, epsilon_accounts, delta_rscalar) =
            Account::create_delta_and_epsilon_accounts(
                account_vector,
                &value_vector_scalar,
                base_pk,
            );

        //identity check function to verify the construction of epsilon accounts using correct rscalars
        Verifier::verify_delta_identity_check(&epsilon_accounts)?;

        // Update delta accounts for output
        let updated_delta_accounts =
            Account::update_delta_accounts(account_vector, &delta_accounts)?;
        let sender_updated_delta_account = &updated_delta_accounts[..senders_count];

        // Create Dark transaction proof
        let dark_tx_proof = DarkTxProof::create_dark_tx_proof(
            &mut prover,
            &value_vector_scalar,
            &delta_accounts,
            &epsilon_accounts,
            &delta_rscalar,
            sender_updated_delta_account,
            &updated_delta_accounts,
            sender_updated_balance,
            reciever_value_balance,
            sender_sk,
            senders_count,
            receivers_count,
            base_pk,
            None,
        );

        // Create outputs from updated delta accounts
        let mut outputs: Vec<Output> = Vec::new();
        for out in updated_delta_accounts.iter() {
            outputs.push(Output::from_quisquis_account(
                *out,
                address::Network::default(),
            ));
        }

        let version = 1u64;
        let maturity = 0u64;
        let input_count = input_vector.len() as u8;
        let output_count = outputs.len() as u8;

        // Create Zero account proof for Reciever accounts as witness in Tx
        // required if new account has been created for the reciever.
        // Not required if the account used for reciever is already present in the UTXO Set
        // get the reciever inputs
        let witness_proof_encrypt_scalar = match witness_comm_scalar {
            Some(scalar_vector) => {
                // create Output_account_commitment_scalar for reciever accounts. Returned back to the client. Required for burnMessage/Script Tx(esp. Order/Lend)
                // create reference for delta_rscalar of recievers
                let delta_rscalar_receiver =
                    &delta_rscalar[senders_count..senders_count + receivers_count];
                // output account commitment scalar = input_commitment_scalar + delta_rscalar + comm_update_scalar                              //x+y+comm_update_scalar
                let encrypt_scalar_sum_vector = delta_rscalar_receiver
                    .iter()
                    .zip(scalar_vector.iter())
                    .map(|(x, y)| x + y)
                    .collect::<Vec<Scalar>>();
                // create proof zero balance commitment for reciever accounts
                let witnesses = reciever_zero_balance_proof(
                    &mut prover,
                    input_vector,
                    scalar_vector,
                    senders_count,
                    receivers_count,
                );
                (Some(witnesses), Some(encrypt_scalar_sum_vector))
            }
            None => (None, None),
        };

        let (witness_count, witness) = match witness_proof_encrypt_scalar.0 {
            Some(witnesses) => (witnesses.len() as u8, Some(witnesses)),
            None => (0u8, None),
        };

        Ok((
            TransferTransaction::set_transfer_transaction(
                version,
                maturity,
                fee,
                input_count,
                output_count,
                witness_count,
                input_vector.to_vec(),
                outputs,
                dark_tx_proof,
                None,
                witness,
            ),
            witness_proof_encrypt_scalar.1,
        ))
    }

    /// Verifies a Dark transaction.
    ///
    /// # Arguments
    /// * `input_accounts` - Input accounts for verification
    /// * `output_accounts` - Optional output accounts (Updated outputs for Dark transactions/ None for QuisQuis transactions)
    ///
    /// # Returns
    /// `Ok(())` if verification succeeds, `Err` otherwise
    pub fn verify_private_transfer_tx(
        &self,
        input_accounts: &[Account],
        output_accounts: Option<&[Account]>,
    ) -> Result<(), &'static str> {
        let mut transcript = Transcript::new(b"TxProof");
        let mut verifier = Verifier::new(b"DarkTx", &mut transcript);

        // Verify the Dark transaction proof
        self.proof
            .verify(&mut verifier, input_accounts, output_accounts)?;

        // Verify witness proofs for new receiver accounts
        let inputs = self.inputs.clone();
        verify_zero_balance_witness(&mut verifier, &inputs, &self.witness)?;

        Ok(())
    }

    /// Sets up input/output shuffling for QuisQuis transactions.
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
        let permutation = inverse_permutation.as_row_major();

        // Shuffle inputs using permutation matrix
        let shuffled_inputs: Vec<_> = (0..initial_inputs.len())
            .map(|i| initial_inputs[permutation[i] - 1].clone())
            .collect();

        // Verify permutation correctness
        for i in 0..shuffled_inputs.len() {
            let sender = input_account_vector[i];
            let sender_input = shuffled_inputs[i].clone();
            let sender_input_account = sender_input.to_quisquis_account().unwrap();
            assert_eq!(sender, sender_input_account);
        }

        // Create outputs
        let mut outputs: Vec<Output> = Vec::new();
        for out in output_account_vector.iter() {
            let out = Output::from_quisquis_account(*out, network);
            outputs.push(out.clone());
        }

        (shuffled_inputs, outputs)
    }

    /// Creates a QuisQuis transaction (shuffled confidential transfer).
    ///
    /// QuisQuis transactions provide privacy by mixing with existing UTXO set
    /// accounts, making it harder to trace individual transactions.
    ///
    /// # Arguments
    /// * `inputs` - Transaction inputs (may include zero UTXOs for receivers)
    /// * `value_vector` - Transfer amounts
    /// * `account_vector` - QuisQuis accounts
    /// * `sender_updated_balance` - New sender balances
    /// * `receiver_value_balance` - Final receiver balances
    /// * `sender_sk` - Sender secret keys
    /// * `senders_count` - Number of senders
    /// * `receivers_count` - Number of receivers
    /// * `anonymity_account_diff` - Number of anonymity accounts to use
    /// * `witness_comm_scalar` - Optional witness scalars for new receivers
    /// * `fee` - Transaction fee
    ///
    /// # Returns
    /// `Ok(TransferTransaction)` - The created transaction
    ///
    /// # Example
    /// ```
    /// use transaction::TransferTransaction;
    /// use quisquislib::ristretto::RistrettoSecretKey;
    ///
    /// let tx = TransferTransaction::create_quisquis_transaction(
    ///     &inputs,
    ///     &value_vector,
    ///     &account_vector,
    ///     &sender_updated_balance,
    ///     &receiver_value_balance,
    ///     &sender_sk,
    ///     senders_count,
    ///     receivers_count,
    ///     anonymity_account_diff,
    ///     witness_comm_scalar,
    ///     fee,
    /// ).unwrap();
    /// ```
    pub fn create_quisquis_transaction(
        inputs: &[Input],
        value_vector: &[i64],
        account_vector: &[Account],
        sender_updated_balance: &[u64],
        reciever_value_balance: &[u64],
        sender_sk: &[RistrettoSecretKey],
        senders_count: usize,
        receivers_count: usize,
        anonymity_account_diff: usize,

        // carries the witness proofs for zero balance reciever accounts if they exist. otherwise none
        // setting the witness index properly in the input is the resposibility of the client
        witness_comm_scalar: Option<&[Scalar]>,
        fee: u64,
    ) -> Result<TransferTransaction, &'static str> {
        // Convert value vector to scalars
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

        // Step 1: Update and shuffle input accounts
        let input_shuffle = Shuffle::input_shuffle(account_vector)?;

        //get vec of Input Accounts arranged randomly
        let input_account_vector = input_shuffle.get_inputs_vector();

        // get vector of Input' accounts updated and arranged as [sender..reciever..anonymity]
        let input_dash_accounts = input_shuffle.get_outputs_vector();

        //create QuisQuisTx Prover merlin transcript
        let mut transcript = Transcript::new(b"TxProof");
        let mut prover = Prover::new(b"QuisQuisTx", &mut transcript);

        // Step 2: Create delta and epsilon accounts
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
        let dark_tx_proof = DarkTxProof::create_dark_tx_proof(
            &mut prover,
            &value_vector_scalar,
            &delta_accounts,
            &epsilon_accounts,
            &delta_rscalar,
            sender_updated_delta_account,
            &updated_delta_accounts,
            sender_updated_balance,
            reciever_value_balance,
            sender_sk,
            senders_count,
            receivers_count,
            base_pk,
            // Should always be none for Quisquis Tx. Only required for Dark Tx
            None,
        );
        // assuming the number of accounts to be 9
        let anonymity_index = 9 - anonymity_account_diff;
        let input_dash_accounts_anonymity_slice = &input_dash_accounts[anonymity_index..9];
        // get a list of anonymity accounts in the updated delta accounts vector
        let updated_delta_accounts_anonymity_slice = &updated_delta_accounts[anonymity_index..9];
        // get of list of scalar witnesses for anonymity accounts in delta and epsilon accounts vector
        let rscalars_anonymity_slice = &delta_rscalar[anonymity_index..9];
        //for anonymity zero account proof. Not needed anymore
        //let input_anonymity_account_slice = &account_vector[anonymity_index..9];
        //Shuffle accounts
        let output_shuffle = Shuffle::output_shuffle(&updated_delta_accounts)?;

        let shuffle_proof = ShuffleTxProof::create_shuffle_proof(
            &mut prover,
            input_dash_accounts_anonymity_slice,
            updated_delta_accounts_anonymity_slice,
            rscalars_anonymity_slice,
            &input_shuffle,
            &output_shuffle,
        );

        let output_final = output_shuffle.get_outputs_vector();

        // Handle witness proofs and create final transaction
        match witness_comm_scalar {
            Some(scalar_vector) => {
                let witnesses = reciever_zero_balance_proof(
                    &mut prover,
                    inputs,
                    scalar_vector,
                    senders_count,
                    receivers_count,
                );

                // create vec of shuffled Inputs and Outputs.
                // This comes after Witnesses are created because the witness index is set in the input for recievers
                let (shuffled_inputs, outputs) = Self::set_quisquis_input_output_prover(
                    &output_final,
                    &input_account_vector,
                    inputs,
                    input_shuffle.get_permutation().to_owned(),
                    address::Network::default(),
                );

                Ok(TransferTransaction::set_transfer_transaction(
                    0u64,
                    0u64,
                    fee,
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
                    inputs,
                    input_shuffle.get_permutation().to_owned(),
                    address::Network::default(),
                );

                Ok(TransferTransaction::set_transfer_transaction(
                    0u64,
                    0u64,
                    fee,
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

    /// Verifies a QuisQuis transaction.
    ///
    /// # Arguments
    /// * `inputs` - Input accounts for verification
    /// * `outputs` - Output accounts for verification
    ///
    /// # Returns
    /// `Ok(())` if verification succeeds, `Err` otherwise
    pub fn verify_quisquis_tx(
        &self,
        inputs: &[Account],
        outputs: &[Account],
    ) -> Result<(), &'static str> {
        let mut transcript = Transcript::new(b"TxProof");
        let mut verifier = Verifier::new(b"QuisQuisTx", &mut transcript);

        let shuffle_proof = self.shuffle_proof.as_ref().unwrap();

        // Verify Dark transaction proof
        self.proof
            .verify(&mut verifier, &shuffle_proof.input_dash_accounts, None)?;

        // Verify shuffle proof
        shuffle_proof.verify(
            &mut verifier,
            inputs,
            outputs,
            &self.proof.updated_delta_accounts,
        )?;

        // Verify witness proofs
        let inputs = self.inputs.clone();
        //verify the zero balance proof for reciever accounts
        verify_zero_balance_witness(&mut verifier, &inputs, &self.witness)?;

        Ok(())
    }

    /// Returns the input values for this transaction.
    pub fn get_input_values(&self) -> Vec<Input> {
        self.inputs.clone()
    }

    /// Returns the output values for this transaction.
    pub fn get_output_values(&self) -> Vec<Output> {
        self.outputs.clone()
    }

    /// Verifies the transaction validity.
    ///
    /// Automatically determines whether this is a Dark or QuisQuis transaction
    /// and performs the appropriate verification.
    ///
    /// # Returns
    /// `Ok(())` if verification succeeds, `Err` otherwise
    ///
    /// # Example
    /// ```
    /// use transaction::TransferTransaction;
    ///
    /// let tx = TransferTransaction::create_quisquis_transaction(/* args */).unwrap();
    /// assert!(tx.verify().is_ok());
    /// ```
    pub fn verify(&self) -> Result<(), &'static str> {
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
            // Verify Dark transaction
            self.verify_private_transfer_tx(&input_accounts, None)
        } else {
            // Verify QuisQuis transaction
            self.verify_quisquis_tx(&input_accounts, &output_accounts)
        }
    }
}
