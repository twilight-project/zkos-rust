//! Message-based transaction implementation.
//!
//! This module provides message transaction functionality for operations like
//! burning assets with reveal proofs. Messages allow for special transaction
//! types that don't follow standard transfer patterns.
//!
//! # Overview
//!
//! Message transactions support:
//! - **Burn Operations**: Destroy assets with cryptographic proof of destruction
//! - **Reveal Proofs**: Prove knowledge of hidden values without revealing them
//! - **Signature Verification**: Cryptographic authorization for message operations
//!
//! # Example
//! ```
//! use transaction::Message;
//! use zkvm::zkos_types::{Input, MessageType};
//! use curve25519_dalek::scalar::Scalar;
//! use quisquislib::ristretto::RistrettoSecretKey;
//!
//! // Create a burn message
//! let burn_message = Message::create_burn_message(
//!     input,
//!     amount,
//!     encrypt_scalar,
//!     secret_key,
//!     initial_address,
//! );
//!
//! // Verify the message
//! assert!(burn_message.verify().is_ok());
//! ```

use address::{Address, AddressType};
use curve25519_dalek::scalar::Scalar;
use quisquislib::{
    keys::PublicKey,
    ristretto::{RistrettoPublicKey, RistrettoSecretKey},
};
use zkvm::zkos_types::{Input, MessageType, Witness};

use crate::proof::RevealProof;
use serde::{Deserialize, Serialize};

/// Message transaction for special operations like burning assets.
///
/// Messages provide a way to perform non-standard transactions that require
/// cryptographic proofs and signatures for authorization.
///
/// # Fields
/// - `msg_type`: Type of message (e.g., Burn)
/// - `version`: Message version
/// - `fee`: Transaction fee
/// - `input`: Input being operated on
/// - `msg_data`: Message-specific data (currently String, will be enum in future)
/// - `proof`: Cryptographic proof (currently RevealProof, will be enum in future)
/// - `signature`: Authorization signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub msg_type: MessageType,
    pub version: u64,
    pub fee: u64,
    pub input: Input,
    // Initially String to support Burn for now. Should be an enum in future
    pub msg_data: String,
    // Initially RevealProof to support Burn for now. Should be an enum in future
    pub proof: RevealProof,
    pub signature: Witness,
}

impl Message {
    /// Creates a new message with the specified parameters.
    ///
    /// # Arguments
    /// * `msg_type` - Type of message operation
    /// * `version` - Message version
    /// * `fee` - Transaction fee
    /// * `input` - Input to operate on
    /// * `msg_data` - Message-specific data
    /// * `proof` - Cryptographic proof
    /// * `signature` - Authorization signature
    ///
    /// # Returns
    /// A new `Message` instance
    ///
    /// # Example
    /// ```
    /// use transaction::Message;
    /// use zkvm::zkos_types::{Input, MessageType, Witness};
    /// use crate::proof::RevealProof;
    ///
    /// let message = Message::new(
    ///     MessageType::Burn,
    ///     0,
    ///     2,
    ///     input,
    ///     "data".to_string(),
    ///     reveal_proof,
    ///     signature,
    /// );
    /// ```
    pub fn new(
        msg_type: MessageType,
        version: u64,
        fee: u64,
        input: Input,
        msg_data: String,
        proof: RevealProof,
        signature: Witness,
    ) -> Message {
        Message {
            msg_type,
            version,
            fee,
            input,
            msg_data,
            proof,
            signature,
        }
    }

    /// Creates a burn message for destroying assets.
    ///
    /// A burn message allows the owner of an asset to prove they are destroying
    /// it by revealing the encryption scalar and amount, while providing a
    /// cryptographic proof that the asset existed.
    ///
    /// # Arguments
    /// * `input` - The input asset to burn
    /// * `amount` - Amount being burned
    /// * `encrypt_scalar` - Encryption scalar for the asset
    /// * `secret_key` - Owner's secret key for signing
    /// * `initial_address` - Address of the initial account
    ///
    /// # Returns
    /// A `Message` configured for burning the specified asset
    ///
    /// # Example
    /// ```
    /// use transaction::Message;
    /// use zkvm::zkos_types::Input;
    /// use curve25519_dalek::scalar::Scalar;
    /// use quisquislib::ristretto::RistrettoSecretKey;
    ///
    /// let burn_message = Message::create_burn_message(
    ///     input,
    ///     100, // amount to burn
    ///     encrypt_scalar,
    ///     secret_key,
    ///     "initial_account_address".to_string(),
    /// );
    /// ```
    pub fn create_burn_message(
        input: Input,
        amount: u64,
        encrypt_scalar: Scalar,
        secret_key: RistrettoSecretKey,
        initial_address: String,
    ) -> Message {
        // Create reveal proof showing knowledge of the asset
        let proof = RevealProof::new(encrypt_scalar, amount);

        // Create signature on the input to prove ownership
        let sign_data = input.as_input_for_signing();
        let message = bincode::serialize(&sign_data).unwrap();

        let pubkey: RistrettoPublicKey =
            Address::from_hex(input.as_owner_address().unwrap(), AddressType::default())
                .unwrap()
                .into();

        let sign = pubkey.sign_msg(&message, &secret_key, ("Signature").as_bytes());
        let signature = Witness::from(sign);

        Message {
            msg_type: MessageType::Burn,
            version: 0,
            fee: 2u64,
            input,
            msg_data: initial_address,
            proof,
            signature,
        }
    }

    /// Verifies the message validity.
    ///
    /// For burn messages, this verifies:
    /// 1. The reveal proof is valid
    /// 2. The signature authorizes the operation
    /// 3. The input encryption matches the proof
    ///
    /// # Returns
    /// `Ok(())` if verification succeeds, `Err` otherwise
    ///
    /// # Errors
    /// * `"BurnError::InvalidRevealProof"` - If reveal proof verification fails
    /// * `"BurnError::InvalidEncryption in Input"` - If input lacks encryption
    /// * `"Burn Message: Invalid Owner Address"` - If owner address is invalid
    /// * `"Burn Message: Invalid Signature"` - If signature format is invalid
    /// * `"Burn Message: Signature verification failed"` - If signature verification fails
    ///
    /// # Example
    /// ```
    /// use transaction::Message;
    ///
    /// let message = Message::create_burn_message(/* args */);
    /// match message.verify() {
    ///     Ok(()) => println!("Burn message verified successfully"),
    ///     Err(e) => println!("Verification failed: {}", e),
    /// }
    /// ```
    pub fn verify(&self) -> Result<(), &'static str> {
        // Reconstruct initial public key from message data
        let init_address = Address::from_hex(&self.msg_data, AddressType::default())?;
        let initial_pk: RistrettoPublicKey = init_address.into();

        // Extract encryption from input and verify reveal proof
        let enc = self.input.as_encryption();
        match enc {
            Some(enc) => {
                if !self.proof.verify(enc, initial_pk) {
                    return Err("BurnError::InvalidRevealProof");
                }
            }
            None => return Err("BurnError::InvalidEncryption in Input"),
        }

        // Verify signature authorizing the burn operation
        let sign_data = self.input.as_input_for_signing();
        let message = bincode::serialize(&sign_data)
            .map_err(|_| "Burn Message Verification: Serialization Failed: Invalid Sign Data")?;

        let owner_address = match self.input.as_owner_address() {
            Some(address) => address,
            None => return Err("Burn Message: Invalid Owner Address"),
        };

        let address = Address::from_hex(owner_address, AddressType::default())?;
        let pubkey: RistrettoPublicKey = address.into();

        let signature = self
            .signature
            .clone()
            .to_signature()
            .map_err(|_| "Burn Message: Invalid Signature")?;

        let verify_sig = pubkey.verify_msg(&message, &signature, ("Signature").as_bytes());
        if verify_sig.is_err() {
            return Err("Burn Message: Signature verification failed");
        }

        Ok(())
    }
}
