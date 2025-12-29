// Copyright © 2019 Interstellar & Stellar Development Foundation
// Copyright © 2025 Twilight Project Contributors
// SPDX-License-Identifier: Apache-2.0
//
// Portions of this file are derived from the `spacesuit` crate in the
// `stellar/slingshot` project (Apache-2.0).

//! Range proof construction using Bulletproofs R1CS framework.
//!
//! This crate provides a pure Rust implementation of range proofs based on the
//! [Bulletproofs](https://crypto.stanford.edu/bulletproofs/) zero-knowledge proof system.
//! It enables proving that a committed value lies within a specified range without
//! revealing the actual value.
//!
//! # Features
//!
//! - **Range Proofs**: Prove values are in range [0, 2^n) for any n ≤ 64
//! - **R1CS Integration**: Built on Bulletproofs R1CS constraint system
//! - **Signed Integer Support**: Handle both positive and negative values
//! - **Pedersen Commitments**: Secure value commitment without revealing secrets
//! - **Batch Verification**: Efficient verification of multiple proofs
//!
//! # Origin
//!
//! Portions of this code are derived from the
//! [`stellar/slingshot`](https://github.com/stellar/slingshot/tree/main/spacesuit) project (Apache-2.0).
//!
//! # Example
//!
//! ```rust
//! use rangeproof::{range_proof, BitRange, SignedInteger};
//! use bulletproofs::r1cs::{Prover, Verifier};
//! use bulletproofs::{BulletproofGens, PedersenGens};
//! use merlin::Transcript;
//! use curve25519_dalek::scalar::Scalar;
//! use rand::thread_rng;
//!
//! // Create generators
//! let pc_gens = PedersenGens::default();
//! let bp_gens = BulletproofGens::new(128, 1);
//!
//! // Prover creates a range proof
//! let (proof, commitment) = {
//!     let mut prover_transcript = Transcript::new(b"RangeProofExample");
//!     let mut rng = thread_rng();
//!     let mut prover = Prover::new(&pc_gens, &mut prover_transcript);
//!
//!     // Commit to a value in range [0, 2^32)
//!     let value = SignedInteger::from(12345u64);
//!     let (com, var) = prover.commit(value.into(), Scalar::random(&mut rng));
//!     
//!     // Create range proof for 32-bit range
//!     let bit_range = BitRange::new(32).unwrap();
//!     range_proof(&mut prover, var.into(), Some(value), bit_range).unwrap();
//!
//!     let proof = prover.prove(&bp_gens).unwrap();
//!     (proof, com)
//! };
//!
//! // Verifier checks the range proof
//! let mut verifier_transcript = Transcript::new(b"RangeProofExample");
//! let mut verifier = Verifier::new(&mut verifier_transcript);
//!
//! let var = verifier.commit(commitment);
//! let bit_range = BitRange::new(32).unwrap();
//! range_proof(&mut verifier, var.into(), None, bit_range).unwrap();
//!
//! assert!(verifier.verify(&proof, &pc_gens, &bp_gens).is_ok());
//! ```
//!
//! # Algorithm
//!
//! The range proof works by:
//!
//! 1. **Binary Decomposition**: Express the value as a sum of powers of 2
//! 2. **Bit Constraints**: Ensure each bit is either 0 or 1
//! 3. **Range Enforcement**: Prove the value equals the sum of its bits
//!
//! For a value `v` in range [0, 2^n), the proof creates `n` multipliers
//! and `2n+1` constraints to ensure the binary representation is valid.
//!

#![deny(missing_docs)]

/// Bit range specification for range proofs.
mod bit_range;
/// Core range proof implementation using R1CS constraints.
mod range_proof;
/// Signed integer arithmetic with overflow protection.
mod signed_integer;
/// Value types and commitment utilities.
mod value;

pub use crate::bit_range::BitRange;
pub use crate::range_proof::range_proof;
pub use crate::signed_integer::SignedInteger;
pub use crate::value::{AllocatedValue, CommittedValue, Value};

// TBD: figure out if we need to export these at all
pub use crate::value::{ProverCommittable, VerifierCommittable};
