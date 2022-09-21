#![deny(missing_docs)]
#![allow(non_snake_case)]
//! Schnorr signature implementation.

mod batch;
mod errors;
mod key;
mod serialization;
mod signature;
mod transcript;

#[cfg(test)]
mod tests;

pub use self::batch::{BatchVerification, BatchVerifier, SingleVerifier};
pub use self::errors::StarsigError;
pub use self::key::{SigningKey, VerificationKey};
pub use self::signature::Signature;
pub use self::transcript::TranscriptProtocol;
