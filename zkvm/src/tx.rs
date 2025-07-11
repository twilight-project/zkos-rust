use bulletproofs::r1cs::R1CSProof;
use bulletproofs::BulletproofGens;
use curve25519_dalek::ristretto::CompressedRistretto;
use merlin::Transcript;
use mulmsgsig::{Signature, VerificationKey};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::contract::{Contract, ContractID};
use crate::encoding::*;
use crate::errors::VMError;
use crate::fees::FeeRate;
use crate::merkle::{Hash, MerkleItem, MerkleTree};
use crate::transcript::TranscriptProtocol;
use crate::verifier::Verifier;
use crate::zkos_types::{Input, Output};



/// Instance of a transaction that contains all necessary data to validate it.
#[derive(Clone, Serialize, Deserialize)]
pub struct Tx {
    /// Header metadata
    pub header: TxHeader,

    /// Program representing the transaction
    pub program: Vec<u8>,

    /// Aggregated signature of the txid
    pub signature: Signature,

    /// Constraint system proof for all the constraints
    pub proof: R1CSProof,
}



impl Tx {
    /// Computes the TxID and TxLog without verifying the transaction.
    pub fn precompute(
        &self,
        inputs: &[Input],
        outputs: &[Output],
    ) -> Result<PrecomputedTx, VMError> {
        Verifier::precompute(self, inputs, outputs)
    }

    /// Performs stateless verification of the transaction:
    /// logic, signatures and ZK R1CS proof.
    pub fn verify(
        &self,
        bp_gens: &BulletproofGens,
        inputs: &[Input],
        outputs: &[Output],
    ) -> Result<VerifiedTx, VMError> {
        self.precompute(inputs, outputs)?.verify(bp_gens)
    }

    /// Serializes the tx into a byte array.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.encode_to_vec()
    }

    /// Deserializes the tx from a byte slice.
    ///
    /// Returns an error if the byte slice cannot be parsed into a `Tx`.
    pub fn from_bytes(mut slice: &[u8]) -> Result<Tx, VMError> {
        slice
            .read_all(|r| Self::decode(r))
            .map_err(|_| VMError::InvalidFormat)
    }
}

impl PrecomputedTx {
    /// Completes verification of the transaction,
    /// performing expensive checks of the R1CS proof, Schnorr signatures
    /// and other Ristretto255 operations.
    pub fn verify(self, bp_gens: &BulletproofGens) -> Result<VerifiedTx, VMError> {
        Verifier::verify_tx(self, bp_gens)
    }

    /// Verifies a batch of transactions, typically coming from a Block.
    pub fn verify_batch(
        txs: impl IntoIterator<Item = Self>,
        bp_gens: &BulletproofGens,
    ) -> Result<Vec<VerifiedTx>, VMError> {
        // TODO: implement and adopt a batch verification API for R1CS proofs.

        txs.into_iter().map(|tx| tx.verify(bp_gens)).collect()
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::merkle::{Hasher, Path};

    fn txlog_helper() -> Vec<TxEntry> {
        vec![
            TxEntry::Header(TxHeader {
                mintime_ms: 0,
                maxtime_ms: 0,
                version: 0,
            }),
            TxEntry::Issue(
                CompressedRistretto::from_slice(&[0u8; 32]),
                CompressedRistretto::from_slice(&[1u8; 32]),
            ),
            TxEntry::Data(vec![0u8]),
            TxEntry::Data(vec![1u8]),
            TxEntry::Data(vec![2u8]),
        ]
    }

    #[test]
    fn valid_txid_proof() {
        let hasher = Hasher::new(b"ZkVM.txid");
        let (entry, txid, path) = {
            let entries = txlog_helper();
            let index = 3;
            let path = Path::new(&entries, index, &hasher).unwrap();
            (entries[index].clone(), TxID::from_log(&entries), path)
        };
        assert!(path.verify_root(&txid.0, &entry, &hasher));
    }

    #[test]
    fn invalid_txid_proof() {
        let hasher = Hasher::new(b"ZkVM.txid");
        let (entry, txid, path) = {
            let entries = txlog_helper();
            let index = 3;
            let path = Path::new(&entries, index, &hasher).unwrap();
            (entries[index + 1].clone(), TxID::from_log(&entries), path)
        };
        assert!(path.verify_root(&txid.0, &entry, &hasher) == false);
    }
}
