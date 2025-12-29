//! Multi-message signature implementation.
//!
//! This module provides the core implementation of multi-message signatures,
//! extending the basic Schnorr signature functionality with the ability to
//! sign and verify multiple messages with different keys using a single
//! signature.

use core::borrow::Borrow;
use core::iter;
use curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;
use starsig::{
    BatchVerification, Signature, SingleVerifier, StarsigError, TranscriptProtocol, VerificationKey,
};

use super::{Multimessage, MusigContext, MusigError};

/// Extension trait for multi-message signature operations on `starsig::Signature`.
///
/// This trait extends the basic Schnorr signature functionality with
/// multi-message capabilities, allowing a single signature to verify
/// multiple messages signed by different keys.
///
/// # Example
/// ```
/// use mulmsgsig::Multisignature;
/// use starsig::{Signature, VerificationKey};
/// use merlin::Transcript;
/// use curve25519_dalek::scalar::Scalar;
///
/// let privkey1 = Scalar::from(1u64);
/// let privkey2 = Scalar::from(2u64);
/// let pubkey1 = VerificationKey::from_secret(&privkey1);
/// let pubkey2 = VerificationKey::from_secret(&privkey2);
///
/// let messages = vec![
///     (pubkey1, b"message1"),
///     (pubkey2, b"message2"),
/// ];
///
/// let mut transcript = Transcript::new(b"example");
/// let signature = Signature::sign_multi(
///     vec![privkey1, privkey2],
///     messages.clone(),
///     &mut transcript,
/// ).unwrap();
///
/// let mut verify_transcript = Transcript::new(b"example");
/// assert!(signature.verify_multi(&mut verify_transcript, messages).is_ok());
/// ```
pub trait Multisignature {
    /// Creates a multi-message signature for multiple keys and messages.
    ///
    /// This method generates a single signature that can verify multiple
    /// messages, each signed by a different private key. The signature
    /// is deterministic and uses the provided transcript for domain separation.
    ///
    /// # Arguments
    /// * `privkeys` - Iterator over private keys (scalars) for signing
    /// * `messages` - Vector of (public key, message) pairs to sign
    /// * `transcript` - Merlin transcript for domain separation
    ///
    /// # Returns
    /// A `Signature` that can verify all the provided messages, or an error
    /// if the operation fails
    ///
    /// # Errors
    /// * `MusigError::BadArguments` - If the number of private keys doesn't match
    ///   the number of messages, or if empty collections are provided
    ///
    /// # Example
    /// ```
    /// use mulmsgsig::Multisignature;
    /// use starsig::{Signature, VerificationKey};
    /// use merlin::Transcript;
    /// use curve25519_dalek::scalar::Scalar;
    ///
    /// let privkeys = vec![Scalar::from(1u64), Scalar::from(2u64)];
    /// let pubkeys = vec![
    ///     VerificationKey::from_secret(&privkeys[0]),
    ///     VerificationKey::from_secret(&privkeys[1]),
    /// ];
    /// let messages = vec![
    ///     (pubkeys[0], b"message1"),
    ///     (pubkeys[1], b"message2"),
    /// ];
    ///
    /// let mut transcript = Transcript::new(b"example");
    /// let signature = Signature::sign_multi(privkeys, messages, &mut transcript).unwrap();
    /// ```
    fn sign_multi<P, M>(
        privkeys: P,
        messages: Vec<(VerificationKey, M)>,
        transcript: &mut Transcript,
    ) -> Result<Signature, MusigError>
    where
        M: AsRef<[u8]>,
        P: IntoIterator,
        P::Item: Borrow<Scalar>,
        P::IntoIter: ExactSizeIterator;

    /// Verifies a multi-message signature against the provided messages.
    ///
    /// This method verifies that the signature is valid for all the
    /// provided (public key, message) pairs.
    ///
    /// # Arguments
    /// * `transcript` - Merlin transcript for domain separation (must match signing transcript)
    /// * `messages` - Vector of (public key, message) pairs to verify
    ///
    /// # Returns
    /// `Ok(())` if verification succeeds, or an error if verification fails
    ///
    /// # Errors
    /// * `StarsigError` - If signature verification fails
    fn verify_multi<M: AsRef<[u8]>>(
        &self,
        transcript: &mut Transcript,
        messages: Vec<(VerificationKey, M)>,
    ) -> Result<(), StarsigError>;

    /// Verifies a multi-message signature as part of a batch verification.
    ///
    /// This method is used for efficient batch verification of multiple
    /// signatures. It adds the verification equation to the provided
    /// batch verifier without performing the actual verification.
    ///
    /// # Arguments
    /// * `transcript` - Merlin transcript for domain separation
    /// * `messages` - Vector of (public key, message) pairs to verify
    /// * `batch` - Batch verifier to add this signature's verification to
    ///
    fn verify_multi_batched<M: AsRef<[u8]>>(
        &self,
        transcript: &mut Transcript,
        messages: Vec<(VerificationKey, M)>,
        batch: &mut impl BatchVerification,
    );
}

impl Multisignature for Signature {
    fn sign_multi<P, M>(
        privkeys: P,
        messages: Vec<(VerificationKey, M)>,
        transcript: &mut Transcript,
    ) -> Result<Signature, MusigError>
    where
        M: AsRef<[u8]>,
        P: IntoIterator,
        P::Item: Borrow<Scalar>,
        P::IntoIter: ExactSizeIterator,
    {
        let mut privkeys = privkeys.into_iter().peekable();

        // Validate input arguments
        if messages.len() != privkeys.len() {
            return Err(MusigError::BadArguments);
        }
        if privkeys.len() == 0 {
            return Err(MusigError::BadArguments);
        }

        // Create multi-message context
        let context = Multimessage::new(messages);

        // Build deterministic RNG from transcript
        let mut rng = transcript
            .build_rng()
            // Use one key that has enough entropy to seed the RNG.
            // We can call unwrap because we know that the privkeys length is > 0.
            .rekey_with_witness_bytes(b"x_i", privkeys.peek().unwrap().borrow().as_bytes())
            .finalize(&mut rand::thread_rng());

        // Generate ephemeral keypair (r, R). r is a random nonce.
        let r = Scalar::random(&mut rng);
        // R = generator * r
        let R = (RISTRETTO_BASEPOINT_POINT * r).compress();

        // Commit the context, and commit the nonce sum with label "R"
        context.commit(transcript);
        transcript.append_point(b"R", &R);

        // Generate signature: s = r + sum{c_i * x_i}
        let mut s = r;
        for (i, x_i) in privkeys.enumerate() {
            let mut t = transcript.clone();
            let c_i = context.challenge(i, &mut t);
            s += c_i * x_i.borrow();
        }

        Ok(Signature { s, R })
    }

    /// Verifies a signature for a multi-message context.
    ///
    /// This implementation uses the single verifier for individual verification.
    fn verify_multi<M: AsRef<[u8]>>(
        &self,
        transcript: &mut Transcript,
        messages: Vec<(VerificationKey, M)>,
    ) -> Result<(), StarsigError> {
        SingleVerifier::verify(|verifier| self.verify_multi_batched(transcript, messages, verifier))
    }

    /// Verifies a multi-message signature in batch mode.
    ///
    /// This implementation adds the verification equation to the batch verifier
    /// for efficient batch processing of multiple signatures.
    fn verify_multi_batched<M: AsRef<[u8]>>(
        &self,
        transcript: &mut Transcript,
        messages: Vec<(VerificationKey, M)>,
        batch: &mut impl BatchVerification,
    ) {
        let context = Multimessage::new(messages);
        context.commit(transcript);
        transcript.append_point(b"R", &self.R);

        // Form the final linear combination:
        // `s * G = R + sum{c_i * X_i}`
        //      ->
        // `0 == (-s * G) + (1 * R) + sum{c_i * X_i}`
        let n = context.len();
        batch.append(
            -self.s,
            iter::once(Scalar::one())
                .chain((0..n).map(|i| context.challenge(i, &mut transcript.clone()))),
            iter::once(self.R.decompress())
                .chain((0..n).map(|i| context.key(i).into_point().decompress())),
        );
    }
}
