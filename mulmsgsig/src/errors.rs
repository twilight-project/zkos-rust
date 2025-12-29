//! Error types for multi-message signature operations.
//!
//! This module defines the error types used throughout the multi-message
//! signature scheme, providing detailed error information for debugging
//! and error handling.
//!
use thiserror::Error;

/// Errors that can occur in key aggregation, signing, or verification..
///
/// This enum provides detailed error information for various failure modes
/// in the multi-message signature scheme, including key aggregation,
/// signing, and verification operations.
///
#[derive(Error, Clone, Debug, Eq, PartialEq)]
pub enum MusigError {
    /// Occurs when a point cannot be decoded as a valid compressed Ristretto point.
    ///
    /// This error typically happens when:
    /// - The point data is corrupted or malformed
    /// - The point is not in the correct compressed format
    /// - The point is not on the Ristretto255 curve
    #[error("Point decoding failed")]
    InvalidPoint,

    /// Occurs when a signature share fails to verify against its corresponding public key.
    ///
    /// This error includes the public key that failed verification to help
    /// identify which specific share is problematic in a multi-signature scheme.
    ///
    /// # Fields
    /// * `pubkey` - The 32-byte public key that failed verification
    #[error("Share #{pubkey:?} failed to verify correctly")]
    ShareError {
        /// The public key corresponding to the share that failed to verify
        pubkey: [u8; 32],
    },

    /// Occurs when an individual point operation (addition, multiplication, etc.) fails.
    ///
    /// This is a catch-all error for cryptographic point operations that
    /// fail for reasons other than invalid input data.
    #[error("Point operation failed")]
    PointOperationFailed,

    /// Occurs when function arguments are invalid or inconsistent.
    ///
    /// This error is raised when:
    /// - The number of private keys doesn't match the number of messages
    /// - Empty collections are provided where non-empty is required
    /// - Arguments are out of valid ranges
    #[error("Bad arguments")]
    BadArguments,
}
