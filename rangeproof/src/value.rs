//! Value types and commitment utilities for range proofs.
//!
//! This module provides types for representing values in range proofs,
//! including both secret values and their Pedersen commitments. It also
//! provides traits for committing values to R1CS constraint systems.
//!
//! # Key Types
//!
//! - **`Value`**: Secret value with quantity and flavor
//! - **`CommittedValue`**: Pedersen commitments to a value
//! - **`AllocatedValue`**: R1CS variables representing a value
//!
//! # Example
//! ```
//! use rangeproof::{Value, AllocatedValue, CommittedValue};
//! use rangeproof::{ProverCommittable, VerifierCommittable};
//! use bulletproofs::r1cs::Prover;
//! use bulletproofs::{BulletproofGens, PedersenGens};
//! use merlin::Transcript;
//! use curve25519_dalek::scalar::Scalar;
//! use rand::thread_rng;
//!
//! let pc_gens = PedersenGens::default();
//! let mut transcript = Transcript::new(b"Example");
//! let mut rng = thread_rng();
//! let mut prover = Prover::new(&pc_gens, &mut transcript);
//!
//! // Create a secret value
//! let value = Value {
//!     q: 42u64.into(),
//!     f: Scalar::from(1u64),
//! };
//!
//! // Commit the value to the constraint system
//! let (commitment, allocated) = value.commit(&mut prover, &mut rng);
//! ```

use bulletproofs::r1cs::{ConstraintSystem, Prover, R1CSError, Variable, Verifier};
use core::borrow::BorrowMut;
use curve25519_dalek::ristretto::CompressedRistretto;
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;
use rand::{CryptoRng, Rng};

use crate::signed_integer::SignedInteger;

/// A secret value consisting of a quantity and flavor.
///
/// This type represents a value in the range proof system, where:
/// - `q` is the secret quantity (signed integer)
/// - `f` is the secret flavor (scalar for asset type)
///
/// Values are typically committed using Pedersen commitments to hide
/// the actual values while allowing range proofs to be constructed.
///
/// # Example
/// ```
/// use rangeproof::Value;
/// use curve25519_dalek::scalar::Scalar;
///
/// let value = Value {
///     q: 100u64.into(),
///     f: Scalar::from(1u64),
/// };
///
/// let zero_value = Value::zero();
/// assert_eq!(zero_value.q.to_u64(), Some(0));
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Value {
    /// Secret quantity
    pub q: SignedInteger,
    /// Secret flavor
    pub f: Scalar,
}

/// Pedersen commitments to a secret value.
///
/// This type contains Pedersen commitments to both the quantity and
/// flavor of a value, allowing verification of range proofs without
/// revealing the actual values.
pub struct CommittedValue {
    /// Pedersen commitment to a quantity
    pub q: CompressedRistretto,
    /// Pedersen commitment to a flavor
    pub f: CompressedRistretto,
}

/// R1CS variables representing a value in the constraint system.
///
/// This type contains R1CS variables for both quantity and flavor,
/// along with an optional assignment of the actual values (for provers).
///
#[derive(Copy, Clone, Debug)]
pub struct AllocatedValue {
    /// R1CS variable representing the quantity
    pub q: Variable,
    /// R1CS variable representing the flavor
    pub f: Variable,
    /// Secret assignment to the above variables
    pub assignment: Option<Value>,
}

impl Value {
    /// Returns a zero value with zero quantity and flavor.
    ///
    /// # Example
    /// ```
    /// use rangeproof::Value;
    ///
    /// let zero = Value::zero();
    /// assert_eq!(zero.q.to_u64(), Some(0));
    /// ```
    pub fn zero() -> Value {
        Value {
            q: 0u64.into(),
            f: Scalar::zero(),
        }
    }

    /// Creates R1CS variables for this value in the constraint system.
    ///
    /// This method allocates variables for both quantity and flavor,
    /// returning an `AllocatedValue` that can be used in range proofs.
    ///
    /// # Arguments
    /// * `cs` - The constraint system to allocate variables in
    ///
    /// # Returns
    /// `Ok(AllocatedValue)` if allocation succeeds, or an error if it fails
    ///
    pub fn allocate<CS: ConstraintSystem>(&self, cs: &mut CS) -> Result<AllocatedValue, R1CSError> {
        let q_u64 = self.q.into();
        let (q_var, f_var, _) = cs.allocate_multiplier(Some((q_u64, self.f)))?;

        Ok(AllocatedValue {
            q: q_var,
            f: f_var,
            assignment: Some(*self),
        })
    }
}
//Changes by . Commented because this is only used in CloaK instruction
/*
impl AllocatedValue {
    /// Creates an unassigned allocated value.
    pub(crate) fn unassigned<CS: ConstraintSystem>(
        cs: &mut CS,
    ) -> Result<AllocatedValue, R1CSError> {
        let (q, f, _) = cs.allocate_multiplier(None)?;

        Ok(Self {
            q,
            f,
            assignment: None,
        })
    }

    /// Creates a list of unassigned allocated values.
    pub(crate) fn unassigned_vec<CS: ConstraintSystem>(
        cs: &mut CS,
        n: usize,
    ) -> Result<Vec<AllocatedValue>, R1CSError> {
        (0..n).map(|_| Self::unassigned(cs)).collect()
    }
}*/

/// Extension trait for committing values to the Prover's constraint system.
///
/// This trait provides a convenient interface for committing secret values
/// to R1CS constraint systems, returning both the commitments and allocated
/// variables.
/// TBD: make this private by refactoring the benchmarks.
pub trait ProverCommittable {
    /// Result of committing Self to a constraint system.
    type Output;

    /// Commits the type to a constraint system.
    fn commit<T: BorrowMut<Transcript>, R: Rng + CryptoRng>(
        &self,
        prover: &mut Prover<T>,
        rng: &mut R,
    ) -> Self::Output;
}

impl ProverCommittable for Value {
    /// Result type of committing to a constraint system.
    type Output = (CommittedValue, AllocatedValue);
    /// Commits the value to a prover's constraint system.
    ///
    /// # Arguments
    /// * `prover` - The prover's constraint system
    /// * `rng` - Random number generator for blinding factors
    ///
    /// # Returns
    /// The commitment and allocated variables
    fn commit<T: BorrowMut<Transcript>, R: Rng + CryptoRng>(
        &self,
        prover: &mut Prover<T>,
        rng: &mut R,
    ) -> Self::Output {
        let (q_commit, q_var) = prover.commit(self.q.into(), Scalar::random(rng));
        let (f_commit, f_var) = prover.commit(self.f, Scalar::random(rng));
        let commitments = CommittedValue {
            q: q_commit,
            f: f_commit,
        };
        let vars = AllocatedValue {
            q: q_var,
            f: f_var,
            assignment: Some(*self),
        };
        (commitments, vars)
    }
}

impl ProverCommittable for Vec<Value> {
    type Output = (Vec<CommittedValue>, Vec<AllocatedValue>);

    fn commit<T: BorrowMut<Transcript>, R: Rng + CryptoRng>(
        &self,
        prover: &mut Prover<T>,
        rng: &mut R,
    ) -> Self::Output {
        self.iter().map(|value| value.commit(prover, rng)).unzip()
    }
}

/// Extension trait for committing values to the Verifier's constraint system.
///
/// This trait provides a convenient interface for committing public values
/// (commitments) to R1CS constraint systems for verification.
/// TBD: make this private by refactoring the benchmarks.
pub trait VerifierCommittable {
    /// Result of committing Self to a constraint system.
    type Output;
    /// Commits the value to a verifier's constraint system.
    ///
    /// # Arguments
    /// * `verifier` - The verifier's constraint system
    ///
    /// # Returns
    /// The allocated variables
    fn commit<T: BorrowMut<Transcript>>(&self, verifier: &mut Verifier<T>) -> Self::Output;
}

impl VerifierCommittable for CommittedValue {
    type Output = AllocatedValue;

    fn commit<T: BorrowMut<Transcript>>(&self, verifier: &mut Verifier<T>) -> Self::Output {
        AllocatedValue {
            q: verifier.commit(self.q),
            f: verifier.commit(self.f),
            assignment: None,
        }
    }
}

impl VerifierCommittable for Vec<CommittedValue> {
    type Output = Vec<AllocatedValue>;

    fn commit<T: BorrowMut<Transcript>>(&self, verifier: &mut Verifier<T>) -> Self::Output {
        self.iter().map(|value| value.commit(verifier)).collect()
    }
}
