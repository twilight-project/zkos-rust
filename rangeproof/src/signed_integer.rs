//! Range-preserving arithmetic on signed integers with u64 absolute value.
//!
//! This module provides the `SignedInteger` type which represents signed integers
//! with absolute values in the 64-bit range. It includes safe arithmetic operations
//! that prevent overflow and maintain the range constraints.
//!
//! # Range
//!
//! The `SignedInteger` type supports values in the range [-2^64+1, 2^64-1],
//! which allows for both positive and negative values while maintaining
//! compatibility with u64-based range proofs.
//!
//! # Example
//! ```
//! use rangeproof::SignedInteger;
//!
//! // Create signed integers
//! let positive = SignedInteger::from(42u64);
//! let negative = -SignedInteger::from(100u64);
//!
//! // Safe arithmetic
//! let sum = positive + negative; // Some(SignedInteger(-58))
//! let product = positive * negative; // Some(SignedInteger(-4200))
//!
//! // Overflow protection
//! let max_val = SignedInteger::from(u64::MAX);
//! let overflow = max_val + SignedInteger::from(1u64); // None
//!
//! // Conversion to u64 (only for non-negative values)
//! assert_eq!(positive.to_u64(), Some(42));
//! assert_eq!(negative.to_u64(), None);
//! ```

use core::ops::Neg;
use curve25519_dalek::scalar::Scalar;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Mul};
use subtle::{Choice, ConditionallySelectable};

/// Represents a signed integer with absolute value in the 64-bit range.
///
/// This type provides safe arithmetic operations for signed integers while
/// maintaining compatibility with range proofs that operate on u64 values.
/// The internal representation uses i128 to handle the full range efficiently.
///
/// # Range
///
/// Values are constrained to the range [-2^64+1, 2^64-1] to ensure
/// compatibility with u64-based cryptographic operations.
///
/// # Example
/// ```
/// use rangeproof::SignedInteger;
///
/// // Valid values
/// let pos = SignedInteger::from(42u64);
/// let neg = -SignedInteger::from(100u64);
/// let zero = SignedInteger::from(0u64);
///
/// // Arithmetic operations
/// let sum = (pos + neg).unwrap();
/// let product = (pos * neg).unwrap();
///
/// // Conversion
/// assert_eq!(pos.to_u64(), Some(42));
/// assert_eq!(neg.to_u64(), None);
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct SignedInteger(i128);

impl SignedInteger {
    /// Converts the integer to u64 if it's non-negative.
    ///
    /// Returns `Some(u64)` if the value is ≥ 0, `None` otherwise.
    /// This is useful for range proofs which require non-negative values.
    ///
    /// # Example
    /// ```
    /// use rangeproof::SignedInteger;
    ///
    /// let pos = SignedInteger::from(42u64);
    /// let neg = -SignedInteger::from(100u64);
    ///
    /// assert_eq!(pos.to_u64(), Some(42));
    /// assert_eq!(neg.to_u64(), None);
    /// ```
    pub fn to_u64(&self) -> Option<u64> {
        if self.0 < 0 {
            None
        } else {
            Some(self.0 as u64)
        }
    }

    /// Converts the integer to a cryptographic scalar.
    ///
    /// This method handles the conversion to the field element used
    /// in cryptographic operations, properly handling negative values.
    ///
    /// # Example
    /// ```
    /// use rangeproof::SignedInteger;
    /// use curve25519_dalek::scalar::Scalar;
    ///
    /// let pos = SignedInteger::from(42u64);
    /// let neg = -SignedInteger::from(100u64);
    ///
    /// let pos_scalar = pos.to_scalar();
    /// let neg_scalar = neg.to_scalar();
    /// ```
    pub fn to_scalar(self) -> Scalar {
        self.into()
    }
}

impl From<u64> for SignedInteger {
    fn from(u: u64) -> SignedInteger {
        SignedInteger(u as i128)
    }
}

impl From<SignedInteger> for Scalar {
    fn from(val: SignedInteger) -> Self {
        if val.0 < 0 {
            Scalar::zero() - Scalar::from((-val.0) as u64)
        } else {
            Scalar::from(val.0 as u64)
        }
    }
}

impl Add for SignedInteger {
    type Output = Option<SignedInteger>;

    /// Adds two signed integers with overflow protection.
    ///
    /// Returns `Some(result)` if the addition doesn't overflow the
    /// valid range [-2^64+1, 2^64-1], `None` otherwise.
    ///
    /// # Example
    /// ```
    /// use rangeproof::SignedInteger;
    ///
    /// let a = SignedInteger::from(42u64);
    /// let b = SignedInteger::from(58u64);
    /// assert_eq!((a + b).unwrap(), SignedInteger::from(100u64));
    ///
    /// // Overflow protection
    /// let max_val = SignedInteger::from(u64::MAX);
    /// assert_eq!(max_val + SignedInteger::from(1u64), None);
    /// ```
    fn add(self, rhs: SignedInteger) -> Option<SignedInteger> {
        let max = u64::MAX as i128;
        let s = self.0 + rhs.0;
        if s <= max && s >= -max {
            Some(SignedInteger(s))
        } else {
            None
        }
    }
}

impl Mul for SignedInteger {
    type Output = Option<SignedInteger>;

    /// Multiplies two signed integers with overflow protection.
    ///
    /// Returns `Some(result)` if the multiplication doesn't overflow the
    /// valid range [-2^64+1, 2^64-1], `None` otherwise.
    ///
    /// # Example
    /// ```
    /// use rangeproof::SignedInteger;
    ///
    /// let a = SignedInteger::from(6u64);
    /// let b = SignedInteger::from(7u64);
    /// assert_eq!((a * b).unwrap(), SignedInteger::from(42u64));
    ///
    /// // Overflow protection
    /// let large = SignedInteger::from(u64::MAX);
    /// assert_eq!(large * SignedInteger::from(2u64), None);
    /// ```
    fn mul(self, rhs: SignedInteger) -> Option<SignedInteger> {
        self.0.checked_mul(rhs.0).and_then(|p| {
            let max = u64::MAX as i128;
            if p <= max && p >= -max {
                Some(SignedInteger(p))
            } else {
                None
            }
        })
    }
}

impl ConditionallySelectable for SignedInteger {
    /// Conditionally selects between two values based on a choice bit.
    ///
    /// This implementation provides constant-time selection for cryptographic
    /// operations that require side-channel resistance.
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        SignedInteger(i128::conditional_select(&a.0, &b.0, choice))
    }
}

impl Neg for SignedInteger {
    type Output = SignedInteger;

    /// Negates the signed integer.
    ///
    /// # Example
    /// ```
    /// use rangeproof::SignedInteger;
    ///
    /// let pos = SignedInteger::from(42u64);
    /// let neg = -pos;
    /// assert_eq!(neg.to_u64(), None);
    /// ```
    fn neg(self) -> SignedInteger {
        SignedInteger(-self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_overflow() {
        let a = SignedInteger::from(u64::max_value());
        let b = SignedInteger::from(0u64);
        assert_eq!((a + b).unwrap(), SignedInteger::from(u64::max_value()));

        let a = SignedInteger::from(u64::max_value());
        let b = SignedInteger::from(1u64);
        assert_eq!(a + b, None);
    }

    #[test]
    fn mul_overflow() {
        let a = SignedInteger::from(u64::max_value());
        let b = SignedInteger::from(1u64);
        assert_eq!((a * b).unwrap(), SignedInteger::from(u64::max_value()));

        let a = SignedInteger::from(u64::max_value());
        let b = -SignedInteger::from(1u64);
        assert_eq!((a * b).unwrap(), -SignedInteger::from(u64::max_value()));

        let a = SignedInteger::from(u64::max_value());
        let b = SignedInteger::from(2u64);
        assert_eq!(a * b, None);

        let a = SignedInteger::from(u64::max_value());
        let b = -SignedInteger::from(2u64);
        assert_eq!(a * b, None);

        let a = SignedInteger::from(u64::max_value());
        let b = SignedInteger::from(u64::max_value());
        assert_eq!(a * b, None);

        let a = SignedInteger::from(u64::max_value());
        let b = -SignedInteger::from(u64::max_value());
        assert_eq!(a * b, None);
    }
}
