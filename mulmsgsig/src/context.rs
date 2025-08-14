//! Context management for multi-message signatures.
//!
//! This module provides the core abstractions for managing multi-signature contexts,
//! including the `MusigContext` trait and the `Multimessage` implementation.
//! These types handle the coordination between multiple signers and their
//! respective messages in a multi-message signature scheme.
//!
//use curve25519_dalek::ristretto::RistrettoPoint;
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;
use starsig::{TranscriptProtocol as StarsigTranscriptProtocol, VerificationKey};

use super::TranscriptProtocol;

/// Context for multi-message signature operations.
///
/// This trait defines the interface for different types of multi-signature contexts,
/// such as multi-key or multi-message contexts. It provides methods for committing
/// context data to transcripts and generating challenges for individual signers.
///
/// # Example
/// ```
/// use mulmsgsig::{MusigContext, Multimessage};
/// use starsig::VerificationKey;
/// use merlin::Transcript;
/// use curve25519_dalek::scalar::Scalar;
///
/// let pubkey = VerificationKey::from_secret(&Scalar::from(1u64));
/// let context = Multimessage::new(vec![(pubkey, b"message")]);
/// let mut transcript = Transcript::new(b"test");
/// context.commit(&mut transcript);
/// ```
pub trait MusigContext {
    /// Commits the internal context data to the transcript.
    ///
    /// This method should serialize all relevant context information (public keys,
    /// messages, etc.) into the transcript to ensure proper domain separation
    /// and prevent signature forgery.
    ///
    /// # Arguments
    /// * `transcript` - The Merlin transcript to commit context data to
    fn commit(&self, transcript: &mut Transcript);

    /// Generates a challenge scalar for a specific signer.
    ///
    /// Creates a deterministic challenge for the signer at the given index,
    /// incorporating the signer's position in the multi-signature scheme.
    ///
    /// # Arguments
    /// * `index` - The index of the signer (0-based)
    /// * `transcript` - The Merlin transcript to derive the challenge from
    ///
    /// # Returns
    /// A scalar challenge value for the specified signer
    fn challenge(&self, index: usize, transcript: &mut Transcript) -> Scalar;

    /// Returns the number of participants in the multi-signature scheme.
    ///
    /// # Returns
    /// The total number of public keys/messages in the context
    fn len(&self) -> usize;

    /// Retrieves the public key for a specific signer.
    ///
    /// # Arguments
    /// * `index` - The index of the signer (0-based)
    ///
    /// # Returns
    /// The verification key for the specified signer
    ///
    /// # Panics
    /// Panics if `index >= self.len()`
    fn key(&self, index: usize) -> VerificationKey;
}

/// Multi-message context for signing multiple messages with different keys.
///
/// This struct implements the `MusigContext` trait for scenarios where each
/// signer has their own unique message to sign. The context maintains a list
/// of (public key, message) pairs and provides the necessary transcript
/// operations for multi-message signatures.
///
/// # Example
/// ```
/// use mulmsgsig::Multimessage;
/// use starsig::VerificationKey;
/// use curve25519_dalek::scalar::Scalar;
///
/// let pubkey1 = VerificationKey::from_secret(&Scalar::from(1u64));
/// let pubkey2 = VerificationKey::from_secret(&Scalar::from(2u64));
/// let context = Multimessage::new(vec![
///     (pubkey1, b"message1"),
///     (pubkey2, b"message2"),
/// ]);
/// ```
#[derive(Clone)]
pub struct Multimessage<M: AsRef<[u8]>> {
    /// Pairs of (public key, message) for each signer
    pairs: Vec<(VerificationKey, M)>,
}

impl<M: AsRef<[u8]>> Multimessage<M> {
    /// Creates a new multi-message context.
    ///
    /// # Arguments
    /// * `pairs` - Vector of (public key, message) pairs for each signer
    ///
    /// # Example
    /// ```
    /// use mulmsgsig::Multimessage;
    /// use starsig::VerificationKey;
    /// use curve25519_dalek::scalar::Scalar;
    ///
    /// let pubkey = VerificationKey::from_secret(&Scalar::from(1u64));
    /// let context = Multimessage::new(vec![(pubkey, b"hello world")]);
    /// ```
    pub fn new(pairs: Vec<(VerificationKey, M)>) -> Self {
        Self { pairs }
    }
}

impl<M: AsRef<[u8]>> MusigContext for Multimessage<M> {
    fn commit(&self, transcript: &mut Transcript) {
        // Add domain separator for multi-message protocol
        transcript.musig_multimessage_domain_sep(self.pairs.len());

        // Commit each (public key, message) pair to the transcript
        for (key, msg) in &self.pairs {
            transcript.append_point(b"X", key.as_point());
            transcript.append_message(b"m", msg.as_ref());
        }
    }

    fn challenge(&self, i: usize, transcript: &mut Transcript) -> Scalar {
        // Create a transcript copy for this specific signer
        let mut transcript_i = transcript.clone();
        transcript_i.append_u64(b"i", i as u64);
        transcript_i.challenge_scalar(b"c")

        // TBD: Do we want to add a domain separator to the transcript?
    }

    fn len(&self) -> usize {
        self.pairs.len()
    }

    fn key(&self, index: usize) -> VerificationKey {
        self.pairs[index].0
    }
}
