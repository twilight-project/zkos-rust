use thiserror::Error;
/// Represents an error in Transaction creation, proving and verification.
#[derive(Error, Clone, Debug, Eq, PartialEq)]
pub enum TxError {
    /// This error occurs when a Tx is not valid
    #[error("Transaction is invalid")]
    InvalidTx,

    /// This error occurs when a Tx proof verification fails
    #[error("Tx proof failed")]
    InvalidProof,
}