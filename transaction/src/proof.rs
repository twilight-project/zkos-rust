#![allow(non_snake_case)]
#![deny(missing_docs)]

//! Zero-knowledge proof structures for confidential transactions.
//!
//! This module provides the core proof types used in ZkOS confidential transactions:
//! - **RevealProof**: Proves knowledge of hidden values (used in burn operations)
//! - **DarkTxProof**: Complete privacy proofs for Dark transactions
//! - **ShuffleTxProof**: Shuffle proofs for QuisQuis transactions
//!
//! # Overview
//!
//! The proof system ensures transaction privacy while maintaining verifiability:
//! - **Delta/Epsilon Accounts**: Core privacy mechanism for value hiding
//! - **DLEQ Proofs**: Prove same value commitment across different bases
//! - **Range Proofs**: Verify values are non-negative using Bulletproofs
//! - **Shuffle Proofs**: Prove input/output permutation correctness
//!
//! # Example
//! ```
//! use transaction::proof::{DarkTxProof, RevealProof};
//! use quisquislib::accounts::prover::Prover;
//! use curve25519_dalek::scalar::Scalar;
//! use quisquislib::ristretto::RistrettoPublicKey;
//!
//! // Create a reveal proof for burning
//! let reveal_proof = RevealProof::new(encrypt_scalar, amount);
//! assert!(reveal_proof.verify(encryption, initial_pk));
//!
//! // Create a Dark transaction proof
//! let dark_proof = DarkTxProof::create_dark_tx_proof(
//!     &mut prover,
//!     &value_vector,
//!     &delta_accounts,
//!     &epsilon_accounts,
//!     &delta_rscalar,
//!     &sender_updated_delta_account,
//!     &updated_delta_accounts,
//!     &sender_updated_balance,
//!     &receiver_value_balance,
//!     &sender_sk,
//!     senders_count,
//!     receivers_count,
//!     base_pk,
//!     None, // QuisQuis transaction
//! );
//! ```

use bulletproofs::PedersenGens;
use bulletproofs::RangeProof;
use curve25519_dalek::scalar::Scalar;
use quisquislib::shuffle::shuffle::ROWS;
use quisquislib::{
    accounts::prover::{Prover, SigmaProof},
    accounts::verifier::Verifier,
    accounts::Account,
    elgamal::ElGamalCommitment,
    keys::PublicKey,
    pedersen::vectorpedersen::VectorPedersenGens,
    ristretto::{RistrettoPublicKey, RistrettoSecretKey},
    shuffle::{Shuffle, ShuffleProof, ShuffleStatement},
};

use serde::{Deserialize, Serialize};

/// Proof for revealing hidden values in burn operations.
///
/// This proof allows the owner of an encrypted asset to prove they know
/// the encryption scalar and amount without revealing them to others.
/// It's used primarily in burn operations to prove asset destruction.
///
/// # Fields
/// - `encrypt_scalar`: The encryption scalar used to hide the amount
/// - `amount`: The actual amount being revealed
///
/// # Example
/// ```
/// use transaction::proof::RevealProof;
/// use curve25519_dalek::scalar::Scalar;
/// use quisquislib::elgamal::ElGamalCommitment;
/// use quisquislib::ristretto::RistrettoPublicKey;
///
/// let proof = RevealProof::new(encrypt_scalar, 100);
/// assert!(proof.verify(encryption, initial_pk));
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RevealProof {
    pub encrypt_scalar: Scalar,
    pub amount: u64,
}

impl RevealProof {
    /// Creates a new reveal proof.
    ///
    /// # Arguments
    /// * `encrypt_scalar` - The encryption scalar used to hide the amount
    /// * `amount` - The actual amount being revealed
    ///
    /// # Returns
    /// A new `RevealProof` instance
    pub fn new(encrypt_scalar: Scalar, amount: u64) -> Self {
        RevealProof {
            encrypt_scalar,
            amount,
        }
    }

    /// Returns the encryption scalar.
    pub fn get_encrypt_scalar(&self) -> Scalar {
        self.encrypt_scalar
    }

    /// Returns the revealed amount.
    pub fn get_amount(&self) -> u64 {
        self.amount
    }

    /// Verifies the reveal proof by reconstructing the encryption.
    ///
    /// This method recreates the ElGamal commitment using the provided
    /// encryption scalar and amount, then compares it with the original
    /// encryption to verify the proof is valid.
    ///
    /// # Arguments
    /// * `encryption` - The original ElGamal encryption
    /// * `initial_pk` - The public key used for encryption
    ///
    /// # Returns
    /// `true` if the proof is valid, `false` otherwise
    ///
    /// # Example
    /// ```
    /// use transaction::proof::RevealProof;
    /// use quisquislib::elgamal::ElGamalCommitment;
    /// use quisquislib::ristretto::RistrettoPublicKey;
    ///
    /// let proof = RevealProof::new(encrypt_scalar, 100);
    /// let is_valid = proof.verify(encryption, initial_pk);
    /// assert!(is_valid);
    /// ```
    pub fn verify(&self, encryption: ElGamalCommitment, initial_pk: RistrettoPublicKey) -> bool {
        // Recreate encryption using reveal proof commitment scalar and initial pk
        let recreated_enc = ElGamalCommitment::generate_commitment(
            &initial_pk,
            self.encrypt_scalar,
            Scalar::from(self.amount),
        );

        // Compare the encryptions
        encryption == recreated_enc
    }
}

/// Zero-knowledge proof for Dark transactions.
///
/// Dark transactions provide complete privacy by hiding all transaction
/// details behind cryptographic proofs. This proof structure contains
/// all the necessary components to verify a Dark transaction without
/// revealing amounts or relationships.
///
/// # Components
/// - **Delta/Epsilon Accounts**: Core privacy mechanism for value hiding
/// - **DLEQ Proofs**: Prove same value commitment across different bases
/// - **Range Proofs**: Verify values are non-negative using Bulletproofs
/// - **Updated Output Proofs**: Optional proofs for Dark transaction outputs
///
/// # Example
/// ```
/// use transaction::proof::DarkTxProof;
/// use quisquislib::accounts::prover::Prover;
/// use curve25519_dalek::scalar::Scalar;
/// use quisquislib::ristretto::RistrettoPublicKey;
///
/// let dark_proof = DarkTxProof::create_dark_tx_proof(
///     &mut prover,
///     &value_vector,
///     &delta_accounts,
///     &epsilon_accounts,
///     &delta_rscalar,
///     &sender_updated_delta_account,
///     &updated_delta_accounts,
///     &sender_updated_balance,
///     &receiver_value_balance,
///     &sender_sk,
///     senders_count,
///     receivers_count,
///     base_pk,
///     None, // QuisQuis transaction
/// );
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DarkTxProof {
    pub(super) delta_accounts: Vec<Account>,
    pub(super) epsilon_accounts: Vec<Account>,
    pub(super) updated_delta_accounts: Vec<Account>,
    pub(super) delta_dleq: SigmaProof,
    pub(super) updated_sender_epsilon_accounts: Vec<Account>,
    pub(super) sender_account_dleq: SigmaProof,
    pub(super) range_proof: Vec<RangeProof>,
    // Proof only needed for Dark Tx. None in case of Quisquis Tx
    pub(super) updated_output_proof: Option<SigmaProof>,
    // ONLY FOR TESTING PURPOSES
    pub(super) receivers_count: usize, // SHOULD BE REMOVED LATER
}

impl DarkTxProof {
    /// Serializes the proof into a byte array.
    ///
    /// # Returns
    /// Byte representation of the proof
    ///
    /// # Note
    /// This is a placeholder implementation. The actual serialization
    /// format needs to be designed to handle all proof components.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(64);
        // DESIGN BYTE STREAM FOR PROOF CATERING FOR VECTORS
        let (pk, _enc) = self.delta_accounts[0].get_account();
        buf.extend_from_slice(&pk.as_bytes());
        // TODO: Implement complete serialization
        buf
    }

    /// Deserializes the proof from a byte slice.
    ///
    /// # Arguments
    /// * `_slice` - Byte slice to deserialize from
    ///
    /// # Note
    /// This is a placeholder implementation. The actual deserialization
    /// format needs to be designed to handle all proof components.
    pub fn from_bytes(_slice: &[u8]) /*-> Result<DarkTxProof, &'static str>*/
    {
        // TODO: Implement complete deserialization
    }

    /// Creates a Dark transaction proof for the prover.
    ///
    /// This method generates all the necessary zero-knowledge proofs
    /// for a Dark transaction, including delta/epsilon account proofs,
    /// range proofs, and optional output update proofs.
    ///
    /// # Arguments
    /// * `prover` - The QuisQuis prover instance
    /// * `value_vector` - Transfer amounts as scalars
    /// * `delta_accounts` - Delta accounts for privacy
    /// * `epsilon_accounts` - Epsilon accounts for privacy
    /// * `delta_rscalar` - Random scalars for delta accounts
    /// * `sender_updated_delta_account` - Updated delta accounts for senders
    /// * `updated_delta_account` - All updated delta accounts
    /// * `sender_updated_balance` - New balances for sender accounts
    /// * `receiver_value_balance` - Final balances for receiver accounts
    /// * `sender_sk` - Secret keys for sender accounts
    /// * `senders_count` - Number of sender accounts
    /// * `receivers_count` - Number of receiver accounts
    /// * `base_pk` - Base public key for account generation
    /// * `update_outputs_statement` - Optional output update statement for Dark transactions
    ///
    /// # Returns
    /// A complete `DarkTxProof` instance
    ///
    /// # Example
    /// ```
    /// use transaction::proof::DarkTxProof;
    /// use quisquislib::accounts::prover::Prover;
    ///
    /// let dark_proof = DarkTxProof::create_dark_tx_proof(
    ///     &mut prover,
    ///     &value_vector,
    ///     &delta_accounts,
    ///     &epsilon_accounts,
    ///     &delta_rscalar,
    ///     &sender_updated_delta_account,
    ///     &updated_delta_accounts,
    ///     &sender_updated_balance,
    ///     &receiver_value_balance,
    ///     &sender_sk,
    ///     senders_count,
    ///     receivers_count,
    ///     base_pk,
    ///     None, // QuisQuis transaction
    /// );
    /// ```
    pub fn create_dark_tx_proof(
        prover: &mut quisquislib::accounts::Prover,
        value_vector: &[Scalar],
        delta_accounts: &[Account],
        epsilon_accounts: &[Account],
        delta_rscalar: &[Scalar],
        sender_updated_delta_account: &[Account],
        updated_delta_account: &[Account],
        sender_updated_balance: &[u64],
        reciever_value_balance: &[u64],
        sender_sk: &[RistrettoSecretKey],
        senders_count: usize,
        receivers_count: usize,
        base_pk: RistrettoPublicKey,
        update_outputs_statement: Option<(&[Account], Scalar, Scalar)>,
    ) -> DarkTxProof {
        // Create DLEQ proof for same balance value committed in epsilon and delta accounts
        let delta_dleq = Prover::verify_delta_compact_prover(
            delta_accounts,
            epsilon_accounts,
            delta_rscalar,
            value_vector,
            prover,
        );

        // Create proof for sender account updates and remaining balances
        let (updated_sender_epsilon_accounts, epsilon_sender_rscalar_vector, sender_account_dleq) =
            Prover::verify_account_prover(
                sender_updated_delta_account,
                sender_updated_balance,
                sender_sk,
                prover,
                base_pk,
            );

        // Create range proofs for sender and receiver accounts
        let bl_rp_vector: Vec<u64> = sender_updated_balance
            .iter()
            .cloned()
            .chain(reciever_value_balance.iter().cloned())
            .collect();

        // Extract receiver rscalars from delta_rscalar
        let rec_rscalars_slice = &delta_rscalar[senders_count..senders_count + receivers_count];

        let scalars_bp_vector: Vec<Scalar> = epsilon_sender_rscalar_vector
            .iter()
            .cloned()
            .chain(rec_rscalars_slice.iter().cloned())
            .collect();

        // Generate range proof over sender/receiver account values (balance >= 0 for all)
        let range_proof =
            prover.verify_non_negative_sender_receiver_prover(&bl_rp_vector, &scalars_bp_vector);

        // Handle Dark vs QuisQuis transaction differences
        match update_outputs_statement {
            // Dark transaction - create updated output proof
            Some((updated_outputs, updated_out_pk_rscalar, updated_out_comm_rscalar)) => {
                let updated_output_proof = Prover::verify_update_account_dark_tx_prover(
                    updated_delta_account,
                    updated_outputs,
                    updated_out_pk_rscalar,
                    updated_out_comm_rscalar,
                    prover,
                );
                DarkTxProof {
                    delta_accounts: delta_accounts.to_vec(),
                    epsilon_accounts: epsilon_accounts.to_vec(),
                    updated_delta_accounts: updated_delta_account.to_vec(),
                    delta_dleq,
                    updated_sender_epsilon_accounts,
                    sender_account_dleq,
                    range_proof,
                    updated_output_proof: Some(updated_output_proof),
                    receivers_count,
                }
            }
            // QuisQuis transaction - no output update proof needed
            None => DarkTxProof {
                delta_accounts: delta_accounts.to_vec(),
                epsilon_accounts: epsilon_accounts.to_vec(),
                updated_delta_accounts: updated_delta_account.to_vec(),
                delta_dleq,
                updated_sender_epsilon_accounts,
                sender_account_dleq,
                range_proof,
                updated_output_proof: None,
                receivers_count,
            },
        }
    }

    /// Verifies the Dark transaction proof.
    ///
    /// This method performs comprehensive verification of all proof components:
    /// - Delta/epsilon account identity verification
    /// - DLEQ proof verification for value consistency
    /// - Delta account update verification
    /// - Sender account balance and secret key verification
    /// - Range proof verification for non-negative values
    /// - Optional output update proof verification for Dark transactions
    ///
    /// # Arguments
    /// * `verifier` - The QuisQuis verifier instance
    /// * `updated_input` - Updated input accounts (input for Dark Tx, input' for QuisQuis Tx)
    /// * `update_output_accounts` - Optional output accounts (Some for Dark Tx, None for QuisQuis Tx)
    ///
    /// # Returns
    /// `Ok(())` if verification succeeds, `Err` otherwise
    ///
    /// # Errors
    /// * Various verification errors if any proof component fails
    ///
    /// # Example
    /// ```
    /// use transaction::proof::DarkTxProof;
    /// use quisquislib::accounts::verifier::Verifier;
    ///
    /// let result = dark_proof.verify(
    ///     &mut verifier,
    ///     &updated_input,
    ///     update_output_accounts, // Some(&outputs) for Dark Tx, None for QuisQuis Tx
    /// );
    /// assert!(result.is_ok());
    /// ```
    pub fn verify(
        &self,
        verifier: &mut Verifier,
        updated_input: &[Account],
        update_output_accounts: Option<&[Account]>,
    ) -> Result<(), &'static str> {
        let base_pk = RistrettoPublicKey::generate_base_pk();

        // Verify epsilon account construction using correct rscalars
        Verifier::verify_delta_identity_check(&self.epsilon_accounts)?;

        // Verify DLEQ proof for same balance value commitment in epsilon and delta accounts
        let delta_dleq = self.delta_dleq.clone();
        let (zv_vector, zr1_vector, zr2_vector, x) = delta_dleq.get_dleq();
        Verifier::verify_delta_compact_verifier(
            &self.delta_accounts,
            &self.epsilon_accounts,
            &zv_vector,
            &zr1_vector,
            &zr2_vector,
            &x,
            verifier,
        )?;

        // Verify delta account updates
        Account::verify_delta_update(
            &self.updated_delta_accounts,
            &self.delta_accounts,
            updated_input,
        )?;

        // Verify sender account balance and secret key
        let (zv_sender_acc, zsk_sender_acc, zr_sender_acc, x_sender_acc) =
            self.sender_account_dleq.clone().get_dleq();

        let senders_count: usize = self.updated_sender_epsilon_accounts.len();
        let updated_delta_account_sender = &self.updated_delta_accounts[..senders_count];

        Verifier::verify_account_verifier_bulletproof(
            updated_delta_account_sender,
            &self.updated_sender_epsilon_accounts,
            &base_pk,
            &zv_sender_acc,
            &zsk_sender_acc,
            &zr_sender_acc,
            x_sender_acc,
            verifier,
        )?;

        // Verify range proofs for sender + receiver accounts (balance >= 0 for all)
        let reciever_epsilon_accounts_slice =
            &self.epsilon_accounts[senders_count..senders_count + self.receivers_count].to_vec();

        let bp_epsilon_vec: Vec<Account> = self
            .updated_sender_epsilon_accounts
            .iter()
            .cloned()
            .chain(reciever_epsilon_accounts_slice.iter().cloned())
            .collect();

        // Choose verification method based on proof type
        match self.range_proof.len() {
            // Batched bulletproof (number of prover values is power of 2)
            1 => verifier
                .verify_non_negative_sender_receiver_bulletproof_batch_verifier(
                    &bp_epsilon_vec,
                    &self.range_proof[0],
                )
                .map_err(|_| "Range Proof Verification Failed")?,
            // Vector proof (number of prover values is not power of 2)
            _ => verifier
                .verify_non_negative_sender_receiver_bulletproof_vector_verifier(
                    &bp_epsilon_vec,
                    &self.range_proof,
                )
                .map_err(|_| "Range Proof Verification Failed")?,
        }

        // Verify updated output proof for Dark transactions
        match update_output_accounts {
            Some(updated_outputs) => {
                let updated_output_proof = self.updated_output_proof.clone().unwrap();
                let (z_vector, x) = updated_output_proof.get_dlog();
                Verifier::verify_update_account_dark_tx_verifier(
                    &self.updated_delta_accounts,
                    updated_outputs,
                    &z_vector,
                    &x,
                    verifier,
                )?;
            }
            None => {
                // QuisQuis transaction - update and shuffle proof handled separately
            }
        }

        Ok(())
    }
}

/// Zero-knowledge proof for QuisQuis transaction shuffles.
///
/// QuisQuis transactions provide privacy by shuffling inputs and outputs
/// with existing UTXO set accounts. This proof structure contains all
/// the necessary components to verify the shuffle operations.
///
/// # Components
/// - **Input Shuffle Proof**: Proves input permutation correctness
/// - **Output Shuffle Proof**: Proves output permutation correctness
/// - **Updated Delta DLOG Proof**: Proves anonymity account updates
/// - **Shuffle Statements**: Metadata for shuffle verification
///
/// # Example
/// ```
/// use transaction::proof::ShuffleTxProof;
/// use quisquislib::accounts::prover::Prover;
/// use quisquislib::shuffle::Shuffle;
///
/// let shuffle_proof = ShuffleTxProof::create_shuffle_proof(
///     &mut prover,
///     &input_dash_accounts_slice,
///     &updated_delta_accounts_slice,
///     &rscalars_slice,
///     &input_shuffle,
///     &output_shuffle,
/// );
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShuffleTxProof {
    pub(super) input_dash_accounts: Vec<Account>, // Updated input accounts
    pub(super) input_shuffle_proof: ShuffleProof,
    pub(super) input_shuffle_statement: ShuffleStatement,
    pub(super) updated_delta_dlog: SigmaProof,
    pub(super) output_shuffle_proof: ShuffleProof,
    pub(super) output_shuffle_statement: ShuffleStatement,
}

impl ShuffleTxProof {
    /// Creates a shuffle proof for QuisQuis transactions.
    ///
    /// This method generates proofs for both input and output shuffles,
    /// along with the necessary DLOG proofs for anonymity account updates.
    ///
    /// # Arguments
    /// * `prover` - The QuisQuis prover instance
    /// * `input_dash_accounts_slice` - Input' anonymity account set
    /// * `updated_delta_accounts_slice` - Output' anonymity account set
    /// * `rscalars_slice` - Random scalars for delta anonymity accounts
    /// * `input_shuffle` - Input shuffle configuration
    /// * `output_shuffle` - Output shuffle configuration
    ///
    /// # Returns
    /// A complete `ShuffleTxProof` instance
    ///
    /// # Example
    /// ```
    /// use transaction::proof::ShuffleTxProof;
    /// use quisquislib::accounts::prover::Prover;
    /// use quisquislib::shuffle::Shuffle;
    ///
    /// let shuffle_proof = ShuffleTxProof::create_shuffle_proof(
    ///     &mut prover,
    ///     &input_dash_accounts_slice,
    ///     &updated_delta_accounts_slice,
    ///     &rscalars_slice,
    ///     &input_shuffle,
    ///     &output_shuffle,
    /// );
    /// ```
    pub fn create_shuffle_proof(
        prover: &mut quisquislib::accounts::Prover,
        input_dash_accounts_slice: &[Account],
        updated_delta_accounts_slice: &[Account],
        rscalars_slice: &[Scalar],
        input_shuffle: &Shuffle,
        output_shuffle: &Shuffle,
    ) -> ShuffleTxProof {
        // Generate Xcommit generator points of length m+1
        let xpc_gens = VectorPedersenGens::new(ROWS + 1);
        let pc_gens = PedersenGens::default();

        // Step 1: Create proof for input shuffle
        let (input_shuffle_proof, input_shuffle_statement) =
            ShuffleProof::create_shuffle_proof(prover, input_shuffle, &pc_gens, &xpc_gens);

        // Step 2: Generate DLOG proof on anonymity accounts in updated delta accounts
        // Prove that anonymity delta accounts have zero balance and are created using correct rscalars
        let updated_delta_dlog = Prover::verify_update_account_prover(
            input_dash_accounts_slice,
            updated_delta_accounts_slice,
            rscalars_slice,
            prover,
        );

        let (output_shuffle_proof, output_shuffle_statement) =
            ShuffleProof::create_shuffle_proof(prover, output_shuffle, &pc_gens, &xpc_gens);

        ShuffleTxProof {
            input_dash_accounts: input_shuffle.get_outputs_vector(),
            input_shuffle_proof,
            input_shuffle_statement,
            updated_delta_dlog,
            output_shuffle_proof,
            output_shuffle_statement,
        }
    }

    /// Verifies the shuffle proof.
    ///
    /// This method verifies all shuffle proof components:
    /// - Input shuffle proof verification
    /// - Updated delta DLOG proof verification
    /// - Output shuffle proof verification
    ///
    /// # Arguments
    /// * `verifier` - The QuisQuis verifier instance
    /// * `input_accounts` - Original input accounts
    /// * `output_accounts` - Final output accounts
    /// * `updated_delta_accounts` - Updated delta accounts
    ///
    /// # Returns
    /// `Ok(())` if verification succeeds, `Err` otherwise
    ///
    /// # Errors
    /// * Various verification errors if any proof component fails
    ///
    /// # Example
    /// ```
    /// use transaction::proof::ShuffleTxProof;
    /// use quisquislib::accounts::verifier::Verifier;
    ///
    /// let result = shuffle_proof.verify(
    ///     &mut verifier,
    ///     &input_accounts,
    ///     &output_accounts,
    ///     &updated_delta_accounts,
    /// );
    /// assert!(result.is_ok());
    /// ```
    pub fn verify(
        &self,
        verifier: &mut Verifier,
        input_accounts: &[Account],
        output_accounts: &[Account],
        updated_delta_accounts: &[Account],
    ) -> Result<(), &'static str> {
        // Recreate Pedersen Commitment (PC) Generator and Extended PC (XPC) Gens
        let xpc_gens = VectorPedersenGens::new(ROWS + 1);
        let pc_gens = PedersenGens::default();

        // Verify the input shuffle
        self.input_shuffle_proof.verify(
            verifier,
            &self.input_shuffle_statement,
            input_accounts,
            &self.input_dash_accounts,
            &pc_gens,
            &xpc_gens,
        )?;

        // Verify DLOG proof on Anonymity accounts in Updated Delta accounts
        let (z_vector, x) = self.updated_delta_dlog.clone().get_dlog();
        let num_anonymity_accounts = z_vector.len();
        let anonymity_index = updated_delta_accounts.len() - num_anonymity_accounts;
        let updated_accounts_slice = &self.input_dash_accounts[anonymity_index..9];
        let updated_delta_accounts_slice = &updated_delta_accounts[anonymity_index..9];

        //verify the dlog proof
        Verifier::verify_update_account_verifier(
            updated_accounts_slice,
            updated_delta_accounts_slice,
            &z_vector,
            &x,
            verifier,
        )?;

        //verify the output shuffle
        self.output_shuffle_proof.verify(
            verifier,
            &self.output_shuffle_statement,
            updated_delta_accounts,
            output_accounts,
            &pc_gens,
            &xpc_gens,
        )?;

        Ok(())
    }

    /// Serializes the proof into a byte array.
    ///
    /// # Returns
    /// Byte representation of the proof
    ///
    /// # Note
    /// This is a placeholder implementation. The actual serialization
    /// format needs to be designed to handle all proof components.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(64);
        //DESIGN BYTE STREAM FOR PROOF CATERING FOR VECTORS
        let (pk, _enc) = self.input_dash_accounts[0].get_account();
        buf.extend_from_slice(&pk.as_bytes());
        // TODO: Implement complete serialization
        buf
    }
}
//TODO: Implement complete serialization for all proof components
//TODO: Implement complete deserialization for all proof components
// ------------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use merlin::Transcript;

    #[test]
    fn test_create_dark_tx_proof() {
        let base_pk = RistrettoPublicKey::generate_base_pk();

        let value_vector: Vec<Scalar> = vec![-Scalar::from(-(-5i64) as u64), Scalar::from(5u64)];

        let mut updated_accounts: Vec<Account> = Vec::new();
        let mut sender_sk: Vec<RistrettoSecretKey> = Vec::new();

        for i in 0..2 {
            let (updated_account, sk) =
                Account::generate_random_account_with_value(Scalar::from(10u64));
            updated_accounts.push(updated_account);

            if i == 0 {
                sender_sk.push(sk);
            }
        }

        let (delta_accounts, epsilon_accounts, delta_rscalar_vector) =
            Account::create_delta_and_epsilon_accounts(&updated_accounts, &value_vector, base_pk);

        let updated_delta_accounts =
            Account::update_delta_accounts(&updated_accounts, &delta_accounts).unwrap();

        let sender_updated_balance: Vec<u64> = vec![5u64];
        let reciever_value_balance: Vec<u64> = vec![5u64];

        let sender_updated_delta_account = &updated_delta_accounts[..1];
        let mut transcript = Transcript::new(b"TxProof");
        let mut prover = Prover::new(b"DarkTx", &mut transcript);

        // Create outputs for Dark transaction
        let pk_update_scalar = Scalar::random(&mut rand::rngs::OsRng);
        let comm_update_scalar = Scalar::random(&mut rand::rngs::OsRng);

        let outputs = updated_delta_accounts
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

        // Create proof for Dark transaction variant
        let dark_tx_proof = DarkTxProof::create_dark_tx_proof(
            &mut prover,
            &value_vector,
            &delta_accounts,
            &epsilon_accounts,
            &delta_rscalar_vector,
            &sender_updated_delta_account,
            &updated_delta_accounts,
            &sender_updated_balance,
            &reciever_value_balance,
            &sender_sk,
            1,
            1,
            base_pk,
            Some((&outputs, pk_update_scalar, comm_update_scalar)),
        );

        // Verify the proof
        let mut v_transcript = Transcript::new(b"TxProof");
        let mut verifier = Verifier::new(b"DarkTx", &mut v_transcript);

        let verify = dark_tx_proof.verify(&mut verifier, &updated_accounts, Some(&outputs));
        assert!(verify.is_ok());
    }

    #[test]
    fn create_scalar_test() {
        let iin: i64 = -5;
        let uin: i64 = 5;

        println!(
            "0 - Scalar {:#?}",
            Scalar::zero() - Scalar::from(-iin as u64)
        );
        println!(" -  {:#?}", -Scalar::from(-iin as u64));
        println!(" -Scalar {:#?}", -Scalar::from(iin as u64));
        println!("Scalar {:#?}", Scalar::from(uin as u64));
        println!("Scalar {:#?}", -Scalar::from(uin as u64));
        println!("Scalar {:#?}", Scalar::from(-iin as u64));
    }

    #[test]
    fn test_reveal_proof() {
        // Create public key randomly
        let base_pk = RistrettoPublicKey::generate_base_pk();

        // Create an encryption
        let enc_scalar = Scalar::random(&mut rand::rngs::OsRng);
        let enc: ElGamalCommitment =
            ElGamalCommitment::generate_commitment(&base_pk, enc_scalar, Scalar::from(100u64));

        let proof = RevealProof::new(enc_scalar.clone(), 100u64);

        // Verify
        let verify = proof.verify(enc.clone(), base_pk);
        assert!(verify);
    }
}
