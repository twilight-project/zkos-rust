//! Transcript protocol extensions for multi-message signatures.
//!
//! This module defines extension traits for Merlin transcripts that provide
//! domain separation and challenge generation specific to multi-message
//! signature schemes.

use merlin::Transcript;
use starsig::TranscriptProtocol as StarsigTranscriptProtocol;

/// Extension trait for Merlin transcripts in multi-message signature protocols.
///
/// This trait extends the basic `StarsigTranscriptProtocol` with additional
/// methods specific to multi-message signature schemes, providing proper
/// domain separation and protocol versioning.
///
/// # Example
/// ```
/// use mulmsgsig::TranscriptProtocol;
/// use merlin::Transcript;
///
/// let mut transcript = Transcript::new(b"example");
/// transcript.musig_multimessage_domain_sep(3); // For 3 participants
/// ```
pub trait TranscriptProtocol: StarsigTranscriptProtocol {
    /// Commits a domain separator for multi-message signature protocols.
    ///
    /// This method adds protocol-specific domain separation to prevent
    /// cross-protocol attacks and ensure proper versioning of the
    /// multi-message signature scheme.
    ///
    /// # Arguments
    /// * `n` - The number of participants (public keys) in the multi-message scheme
    ///
    /// # Example
    /// ```
    /// use mulmsgsig::TranscriptProtocol;
    /// use merlin::Transcript;
    ///
    /// let mut transcript = Transcript::new(b"test");
    /// transcript.musig_multimessage_domain_sep(2); // 2 participants
    /// ```
    fn musig_multimessage_domain_sep(&mut self, n: usize);
}

impl TranscriptProtocol for Transcript {
    fn musig_multimessage_domain_sep(&mut self, n: usize) {
        // Add protocol identifier and version
        self.append_message(b"dom-sep", b"musig-multimessage v1");
        // Add number of participants for domain separation
        self.append_u64(b"n", n as u64);
    }
}
