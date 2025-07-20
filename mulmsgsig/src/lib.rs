// Copyright © 2019 Interstellar & Stellar Development Foundation
// Copyright © 2025 Twilight Project Contributors
// SPDX-License-Identifier: Apache-2.0
//
// Portions of this file are derived from the `mugsig` crate in the
// `stellar/slingshot` project (Apache-2.0).

#![deny(missing_docs)]
#![allow(non_snake_case)]

//! Multi-message multi-signature scheme for Ristretto255.
//!
//! This crate provides a pure Rust implementation of multi-message multi-signatures
//! over the Ristretto255 curve, allowing a single signature to verify multiple
//! messages signed by different keys.
//!
//! # Features
//! - Multi-message multi-signature generation and verification
//! - Batch verification support for efficient signature checking
//! - Transcript-based API using Merlin for domain separation
//! - Integration with the `starsig` crate for Schnorr signatures
//! - Error types for robust error handling
//!
//! # Origin
//! Portions of this code are derived from the
//! [`stellar/slingshot`](https://github.com/stellar/slingshot/tree/main/mugsig) project (Apache-2.0).
//!
//! # Example
//! ```
//! use mulmsgsig::{Multisignature, MusigContext, Multimessage};
//! use starsig::{Signature, VerificationKey};
//! use merlin::Transcript;
//! use curve25519_dalek::scalar::Scalar;
//!
//! // Create signing keys
//! let privkey1 = Scalar::from(1u64);
//! let privkey2 = Scalar::from(2u64);
//! let pubkey1 = VerificationKey::from_secret(&privkey1);
//! let pubkey2 = VerificationKey::from_secret(&privkey2);
//!
//! // Create messages for each key
//! let messages = vec![
//!     (pubkey1, b"message1"),
//!     (pubkey2, b"message2"),
//! ];
//!
//! // Sign multiple messages with multiple keys
//! let mut transcript = Transcript::new(b"example");
//! let signature = Signature::sign_multi(
//!     vec![privkey1, privkey2],
//!     messages.clone(),
//!     &mut transcript,
//! ).unwrap();
//!
//! // Verify the multi-message signature
//! let mut verify_transcript = Transcript::new(b"example");
//! assert!(signature.verify_multi(&mut verify_transcript, messages).is_ok());
//! ```

/// Context management for multi-message signatures.
mod context;
/// Error types for multi-message signature operations.
mod errors;
/// Multi-message signature implementation.
mod multisignature;
/// Transcript protocol extensions for multi-message signatures.
mod transcript;

#[cfg(test)]
mod tests;

// Convenience re-exports from `starsig` crate.
pub use starsig::TranscriptProtocol as StarsigTranscriptProtocol;
pub use starsig::{
    BatchVerification, BatchVerifier, Signature, SingleVerifier, StarsigError, VerificationKey,
};

pub use self::context::{Multimessage, MusigContext};
pub use self::errors::MusigError;
pub use self::multisignature::Multisignature;
pub use self::transcript::TranscriptProtocol;
