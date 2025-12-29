//! Bit range specification for range proofs.
//!
//! This module provides the `BitRange` type which specifies the bit width
//! for range proofs, ensuring values are in the range [0, 2^n) where n ≤ 64.
//!
//! # Example
//! ```
//! use rangeproof::BitRange;
//!
//! // Invalid range (too large)
//! assert!(BitRange::new(65).is_none());
//!
//! ```

/// Represents a bit range for range proofs with value in [0, 64].
///
/// This type ensures that range proofs are created with valid bit widths,
/// preventing potential issues with oversized ranges that could cause
/// performance or security problems.
///
/// # Example
/// ```
/// use rangeproof::BitRange;
///
/// // Valid ranges
/// assert!(BitRange::new(8).is_some());   // 8-bit range
/// assert!(BitRange::new(32).is_some());  // 32-bit range
/// assert!(BitRange::new(64).is_some());  // 64-bit range
///
/// // Invalid ranges
/// assert!(BitRange::new(65).is_none());  // Too large
/// assert!(BitRange::new(0).is_some());   // 0-bit range (valid but empty)
/// ```
#[derive(Copy, Clone, Debug)]
pub struct BitRange(usize);

impl BitRange {
    /// Creates a new bit range if the value is ≤ 64.
    ///
    /// # Arguments
    /// * `n` - The bit width for the range [0, 2^n)
    ///
    /// # Returns
    /// `Some(BitRange)` if n ≤ 64, `None` otherwise
    ///
    /// # Example
    /// ```
    /// use rangeproof::BitRange;
    ///
    /// assert_eq!(usize::from(BitRange::new(32).unwrap()), 32usize);
    /// assert!(BitRange::new(65).is_none());
    /// ```
    pub fn new(n: usize) -> Option<Self> {
        if n > 64 {
            None
        } else {
            Some(BitRange(n))
        }
    }

    /// Returns the maximum bit range (64 bits).
    ///
    /// This creates a range for values in [0, 2^64).
    ///
    /// # Example
    /// ```
    /// use rangeproof::BitRange;
    ///
    /// let max_range = BitRange::max();
    /// assert_eq!(usize::from(max_range), 64usize);
    /// ```
    pub fn max() -> Self {
        BitRange(64)
    }
}

impl From<BitRange> for usize {
    fn from(val: BitRange) -> Self {
        val.0
    }
}

impl From<BitRange> for u8 {
    fn from(val: BitRange) -> Self {
        val.0 as u8
    }
}
