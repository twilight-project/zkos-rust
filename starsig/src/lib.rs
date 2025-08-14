// Copyright © 2019 Interstellar & Stellar Development Foundation
// Copyright © 2025 Twilight Project Contributors
// SPDX-License-Identifier: Apache-2.0
//
// Portions of this file are derived from the `starsig` crate in the
// `stellar/slingshot` project (Apache-2.0).

#![deny(missing_docs)]
#![allow(non_snake_case)]
//! Schnorr signature implementation for Ristretto255.
//!
//! This crate provides a pure Rust implementation of Schnorr signatures over the Ristretto255 curve,
//! with batch verification, transcript-based signing, and serialization support.
//!
//! # Features
//! - Schnorr signature generation and verification
//! - Batch verification for efficient signature checking
//! - Transcript-based API using Merlin for domain separation
//! - Key generation and serialization utilities
//! - Error types for robust error handling
//!
//! # Origin
//! Portions of this code are derived from the
//! [`stellar/slingshot`](https://github.com/stellar/slingshot/tree/main/starsig) project (Apache-2.0).
//!
//! # Example
//! ```
//! use starsig::{Signature, SigningKey, VerificationKey};
//! use merlin::Transcript;
//! use curve25519_dalek::scalar::Scalar;
//!
//! let privkey = Scalar::from(1u64);
//! let sig = Signature::sign(&mut Transcript::new("example".as_bytes()), privkey);
//! let pubkey = VerificationKey::from_secret(&privkey);
//! assert!(sig.verify(&mut Transcript::new("example".as_bytes()), pubkey).is_ok());
//! ```

/// Batch verification traits and types for Schnorr signatures.
mod batch;
/// Error types for Schnorr signature operations.
mod errors;
/// Key types and utilities for Schnorr signatures over Ristretto255.
mod key;
/// Serialization and deserialization for Schnorr signatures.
mod serialization;
/// Schnorr signature type and signing/verification routines.
mod signature;
/// Extension trait for Merlin transcripts for use with Schnorr signatures.
mod transcript;

#[cfg(test)]
mod tests;

pub use self::batch::{BatchVerification, BatchVerifier, SingleVerifier};
pub use self::errors::StarsigError;
pub use self::key::{SigningKey, VerificationKey};
pub use self::signature::Signature;
pub use self::transcript::TranscriptProtocol;
