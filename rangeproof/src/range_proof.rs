// Copyright © 2019 Interstellar & Stellar Development Foundation
// Copyright © 2025 Twilight Project Contributors
// SPDX-License-Identifier: Apache-2.0
//
// Portions of this file are derived from the `spacesuit` crate in the
// `stellar/slingshot` project (Apache-2.0).

//! Core range proof implementation using R1CS constraints.
//!
//! This module provides the main `range_proof` function that creates R1CS
//! constraints to prove a value is in the range [0, 2^n) for a given bit width n.
//!
//! # Algorithm
//!
//! The range proof works by:
//!
//! 1. **Binary Decomposition**: Express value v as sum of bits: v = Σ(b_i * 2^i)
//! 2. **Bit Constraints**: Ensure each bit b_i is either 0 or 1
//! 3. **Range Enforcement**: Prove v equals the sum of its bits
//!
//! For each bit position i, we create:
//! - Multiplier: a_i * b_i = 0 (ensures one of a_i, b_i is zero)
//! - Constraint: a_i = 1 - b_i (ensures a_i, b_i are complementary)
//! - Contribution: v -= b_i * 2^i (builds the sum)
//!
//! # Example
//! ```
//! use rangeproof::{range_proof, BitRange, SignedInteger};
//! use bulletproofs::r1cs::{Prover, Verifier};
//! use bulletproofs::{BulletproofGens, PedersenGens};
//! use merlin::Transcript;
//! use curve25519_dalek::scalar::Scalar;
//! use rand::thread_rng;
//!
//! let pc_gens = PedersenGens::default();
//! let bp_gens = BulletproofGens::new(128, 1);
//!
//! // Prover creates range proof
//! let (proof, commitment) = {
//!     let mut prover_transcript = Transcript::new(b"RangeProofTest");
//!     let mut rng = thread_rng();
//!     let mut prover = Prover::new(&pc_gens, &mut prover_transcript);
//!
//!     let value = SignedInteger::from(12345u64);
//!     let (com, var) = prover.commit(value.into(), Scalar::random(&mut rng));
//!     
//!     let bit_range = BitRange::new(16).unwrap();
//!     range_proof(&mut prover, var.into(), Some(value), bit_range).unwrap();
//!
//!     let proof = prover.prove(&bp_gens).unwrap();
//!     (proof, com)
//! };
//!
//! // Verifier checks range proof
//! let mut verifier_transcript = Transcript::new(b"RangeProofTest");
//! let mut verifier = Verifier::new(&mut verifier_transcript);
//!
//! let var = verifier.commit(commitment);
//! let bit_range = BitRange::new(16).unwrap();
//! range_proof(&mut verifier, var.into(), None, bit_range).unwrap();
//!
//! assert!(verifier.verify(&proof, &pc_gens, &bp_gens).is_ok());
//! ```

use crate::bit_range::BitRange;
use bulletproofs::r1cs::{ConstraintSystem, LinearCombination, R1CSError};
use curve25519_dalek::scalar::Scalar;

use crate::signed_integer::SignedInteger;

/// Creates R1CS constraints to prove a value is in range [0, 2^n).
///
/// This function generates the necessary R1CS constraints to prove that
/// a committed value `v` lies within the specified bit range without
/// revealing the actual value.
///
/// # Arguments
///
/// * `cs` - The constraint system to add constraints to
/// * `v` - Linear combination representing the committed value
/// * `v_assignment` - Optional assignment of the actual value (for prover)
/// * `n` - Bit range specifying the maximum value as 2^n
///
/// # Returns
///
/// `Ok(())` if constraints were added successfully, or an error if the
/// operation failed.
///
/// # Algorithm Details
///
/// For each bit position i from 0 to n-1:
///
/// 1. **Create multiplier**: `(a_i, b_i, o_i)` where `a_i * b_i = o_i`
/// 2. **Enforce bit constraint**: `o_i = 0` (ensures one of a_i, b_i is zero)
/// 3. **Enforce complementarity**: `a_i = 1 - b_i` (ensures a_i, b_i are 0/1)
/// 4. **Build sum**: `v -= b_i * 2^i` (contributes to the binary decomposition)
///
/// Finally, constrain `v = 0` to ensure the value equals the sum of its bits.
///
/// # Example
///
/// ```rust
/// use rangeproof::{range_proof, BitRange, SignedInteger};
/// use bulletproofs::r1cs::Prover;
/// use bulletproofs::{BulletproofGens, PedersenGens};
/// use merlin::Transcript;
/// use curve25519_dalek::scalar::Scalar;
/// use rand::thread_rng;
///
/// let pc_gens = PedersenGens::default();
/// let mut transcript = Transcript::new(b"Example");
/// let mut rng = thread_rng();
/// let mut prover = Prover::new(&pc_gens, &mut transcript);
///
/// let value = SignedInteger::from(42u64);
/// let (_, var) = prover.commit(value.into(), Scalar::random(&mut rng));
/// let bit_range = BitRange::new(8).unwrap();
///
/// // Create range proof for 8-bit range [0, 256)
/// range_proof(&mut prover, var.into(), Some(value), bit_range).unwrap();
/// ```
///
/// # Errors
///
/// * `R1CSError::GadgetError` - If constraint creation fails
/// * `R1CSError::InvalidAssignment` - If value assignment is invalid
pub fn range_proof<CS: ConstraintSystem>(
    cs: &mut CS,
    mut v: LinearCombination,
    v_assignment: Option<SignedInteger>,
    n: BitRange,
) -> Result<(), R1CSError> {
    let mut exp_2 = Scalar::one();
    let n_usize: usize = n.into();

    // For each bit position from 0 to n-1
    for i in 0..n_usize {
        // Create multiplier variables (a_i, b_i, o_i) where a_i * b_i = o_i
        let (a, b, o) = cs.allocate_multiplier(v_assignment.and_then(|q| {
            q.to_u64().map(|q| {
                let bit: u64 = (q >> i) & 1;
                ((1 - bit).into(), bit.into())
            })
        }))?;

        // Enforce a_i * b_i = 0, so one of (a_i, b_i) is zero
        cs.constrain(o.into());

        // Enforce that a_i = 1 - b_i, so they are complementary (both 0/1 or 1/0)
        cs.constrain(a + (b - 1u64));

        // Add `-b_i * 2^i` to the linear combination
        // This builds the constraint: v = Σ(b_i * 2^i) for i = 0..n-1
        v = v - b * exp_2;

        // Double the exponent for next bit position
        exp_2 = exp_2 + exp_2;
    }

    // Final constraint: v = Σ(b_i * 2^i) for i = 0..n-1
    // This ensures the value equals the sum of its binary representation
    cs.constrain(v);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bulletproofs::r1cs::{Prover, Verifier};
    use bulletproofs::{BulletproofGens, PedersenGens};
    use merlin::Transcript;

    #[test]
    fn range_proof_gadget() {
        use rand::thread_rng;
        use rand::Rng;

        let mut rng = thread_rng();
        let m = 3; // number of values to test per `n`

        for n in [2, 10, 32, 63].iter() {
            let (min, max) = (0u64, ((1u128 << n) - 1) as u64);
            let values: Vec<u64> = (0..m).map(|_| rng.gen_range(min, max)).collect();
            for v in values {
                assert!(range_proof_helper(v.into(), *n).is_ok());
            }
            assert!(range_proof_helper((max + 1).into(), *n).is_err());
        }
    }

    fn range_proof_helper(v_val: SignedInteger, n: usize) -> Result<(), R1CSError> {
        // Common setup
        let pc_gens = PedersenGens::default();
        let bp_gens = BulletproofGens::new(128, 1);
        let bit_width = BitRange::new(n).ok_or(R1CSError::GadgetError {
            description: "Invalid Bitrange; Bitrange must be between 0 and 64".to_string(),
        })?;

        // Prover's scope
        let (proof, commitment) = {
            // Prover makes a `ConstraintSystem` instance representing a range proof gadget
            let mut prover_transcript = Transcript::new(b"RangeProofTest");
            let mut rng = rand::thread_rng();

            let mut prover = Prover::new(&pc_gens, &mut prover_transcript);

            let (com, var) = prover.commit(v_val.into(), Scalar::random(&mut rng));
            assert!(range_proof(&mut prover, var.into(), Some(v_val), bit_width).is_ok());

            let proof = prover.prove(&bp_gens)?;

            (proof, com)
        };

        // Verifier makes a `ConstraintSystem` instance representing a merge gadget
        let mut verifier_transcript = Transcript::new(b"RangeProofTest");
        let mut verifier = Verifier::new(&mut verifier_transcript);

        let var = verifier.commit(commitment);

        // Verifier adds constraints to the constraint system
        assert!(range_proof(&mut verifier, var.into(), None, bit_width).is_ok());

        // Verifier verifies proof
        Ok(verifier.verify(&proof, &pc_gens, &bp_gens)?)
    }
}
