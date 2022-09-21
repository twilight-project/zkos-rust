#![deny(missing_docs)]
#![allow(non_snake_case)]
//! Mulmsgsig implementation

mod context;
mod multisignature;
mod errors;
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
