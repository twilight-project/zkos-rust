use serde::{Deserialize, Serialize};

/// Serialized as the "method" field of JSON-RPC/HTTP requests.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
pub enum Method {
    /// Sends a transaction and immediately returns transaction hash.
    TxQueue,

    /// Sends a transaction and waits until transaction is fully complete.
    TxCommit,

    /// Queries status of a transaction by hash and returns the final transaction result.
    TxStatus,
}
