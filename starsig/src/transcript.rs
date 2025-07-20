//! Extension trait for Merlin transcripts for use with Schnorr signatures.
//!
//! This module provides the [`TranscriptProtocol`] trait, which extends the Merlin [`Transcript`] API
//! with methods for committing scalars and points and generating challenge scalars, as used in the starsig protocol.
//!
//! # Example
//! ```
//! use merlin::Transcript;
//! use starsig::TranscriptProtocol;
//! let mut t = Transcript::new(b"example");
//! t.starsig_domain_sep();
//! // ...
//! ```

use curve25519_dalek::ristretto::CompressedRistretto;
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;

/// Extension trait to the Merlin transcript API that allows committing scalars and points and
/// generating challenges as scalars for the starsig protocol.
pub trait TranscriptProtocol {
    /// Commit a domain separator for a single-message signature protocol.
    fn starsig_domain_sep(&mut self);
    /// Commit a `scalar` with the given `label`.
    fn append_scalar(&mut self, label: &'static [u8], scalar: &Scalar);
    /// Commit a `point` with the given `label`.
    fn append_point(&mut self, label: &'static [u8], point: &CompressedRistretto);
    /// Compute a `labeled` challenge variable as a scalar.
    fn challenge_scalar(&mut self, label: &'static [u8]) -> Scalar;
}

impl TranscriptProtocol for Transcript {
    fn starsig_domain_sep(&mut self) {
        self.append_message(b"dom-sep", b"starsig v1");
    }

    fn append_scalar(&mut self, label: &'static [u8], scalar: &Scalar) {
        self.append_message(label, scalar.as_bytes());
    }

    fn append_point(&mut self, label: &'static [u8], point: &CompressedRistretto) {
        self.append_message(label, point.as_bytes());
    }

    fn challenge_scalar(&mut self, label: &'static [u8]) -> Scalar {
        let mut buf = [0u8; 64];
        self.challenge_bytes(label, &mut buf);
        Scalar::from_bytes_mod_order_wide(&buf)
    }
}
