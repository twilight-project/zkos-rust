use address::{Address, AddressType};
//use merlin::Transcript;
use curve25519_dalek::scalar::Scalar;
use quisquislib::{
    keys::PublicKey,
    ristretto::{RistrettoPublicKey, RistrettoSecretKey},
};
use zkvm::zkos_types::{Input, MessageType, Witness};

use crate::proof::RevealProof;
use serde::{Deserialize, Serialize};

/// Message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub msg_type: MessageType,
    pub version: u64,
    pub fee: u64,
    pub input: Input,
    pub msg_data: Vec<u8>,
    pub proof: RevealProof,
    pub signature: Witness,
}

impl Message {
    /// Create a new message
    pub fn new(
        msg_type: MessageType,
        version: u64,
        fee: u64,
        input: Input,
        msg_data: Vec<u8>,
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
    /// create a burn message
    /// burn message is a message with a input and its reveal proof
    pub fn create_burn_message(
        input: Input,
        amount: u64,
        encrypt_scalar: Scalar,
        secret_key: RistrettoSecretKey,
    ) -> Message {
        // create reveal proof
        let proof = RevealProof::new(encrypt_scalar, amount);
        // create signature on the input
        let sign_data = input.as_input_for_signing();
        //create message bytes using input_state
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
            fee: 0,
            input,
            msg_data: vec![],
            proof,
            signature,
        }
    }
    pub fn verify(&self) -> Result<(), &'static str> {
        // convert input to quisquis account
        let account = self.input.to_quisquis_account()?;
        // verify reveal proof
        if self.proof.verify(account) == false {
            return Err("BurnError::InvalidRevealProof");
        }
        // verify siignature
        let sign_data = self.input.as_input_for_signing();
        let message = bincode::serialize(&sign_data).unwrap();
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
