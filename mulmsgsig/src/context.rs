//use curve25519_dalek::ristretto::RistrettoPoint;
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;
use starsig::{TranscriptProtocol as StarsigTranscriptProtocol, VerificationKey};

use super::TranscriptProtocol;

/// The context for signing - can either be a Multikey or Multimessage context.
pub trait MusigContext {
    /// Takes a mutable transcript, and commits the internal context to the transcript.
    fn commit(&self, transcript: &mut Transcript);

    /// Takes an index of a public key and mutable transcript,
    /// and returns the suitable challenge for that public key.
    fn challenge(&self, index: usize, transcript: &mut Transcript) -> Scalar;

    /// Length of the number of pubkeys in the context
    fn len(&self) -> usize;

    /// Returns the pubkey for the index i
    fn key(&self, index: usize) -> VerificationKey;
}

/// MuSig multimessage context
#[derive(Clone)]
pub struct Multimessage<M: AsRef<[u8]>> {
    pairs: Vec<(VerificationKey, M)>,
}

impl<M: AsRef<[u8]>> Multimessage<M> {
    /// Constructs a new multimessage context
    pub fn new(pairs: Vec<(VerificationKey, M)>) -> Self {
        Self { pairs }
    }
}

impl<M: AsRef<[u8]>> MusigContext for Multimessage<M> {
    fn commit(&self, transcript: &mut Transcript) {
        transcript.musig_multimessage_domain_sep(self.pairs.len());
        for (key, msg) in &self.pairs {
            transcript.append_point(b"X", key.as_point());
            transcript.append_message(b"m", msg.as_ref());
        }
    }

    fn challenge(&self, i: usize, transcript: &mut Transcript) -> Scalar {
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
