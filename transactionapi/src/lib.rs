//! ZkOS Transaction API for blockchain interaction and UTXO management.
//!
//! This crate provides a JSON-RPC interface for interacting with the ZkOS blockchain,
//! including transaction submission, UTXO management, and blockchain state queries.
//!
//! ## Overview
//!
//! The TransactionAPI enables:
//! - **Transaction Operations**: Submit and query transactions (txCommit, txStatus, TxQueue)
//! - **UTXO Queries**: Retrieve UTXOs by address and type (Coin, Memo, State)
//! - **Output Queries**: Get transaction outputs and their details
//! - **Database Queries**: Advanced UTXO filtering with pagination
//!
//! ## Base URLs
//! - **Production**: `https://nykschain.twilight.rest/zkos/`
//! - **Development**: `http://localhost:3030/`
//!
//! ## Quick Start
//!
//! ```rust
//! use transactionapi::{rpcclient, rpcserver};
//!
//! // Create RPC client
//! let client = rpcclient::txrequest::RpcBody::new(
//!     transaction,
//!     rpcclient::method::Method::txCommit
//! );
//!
//! // Send transaction
//! let response = client.send("http://localhost:3030/".to_string())?;
//! ```
//!
//! ## API Methods
//!
//! ### Transaction Operations
//! - `txCommit` - Submit transaction to blockchain
//! - `txStatus` - Query transaction status  
//! - `TxQueue` - Queue transaction for processing
//!
//! ### UTXO Operations
//! - `getUtxos` - Get UTXOs by address
//! - `getMemoUtxos` - Get memo UTXOs by address
//! - `getStateUtxos` - Get state UTXOs by address
//! - `allUtxos` - Get all UTXOs
//! - `allOutputs` - Get all outputs
//!
//! ## Error Handling
//!
//! All API methods return `Result<T, E>` where errors include:
//! - Invalid parameters
//! - Network errors
//! - Transaction verification failures
//! - UTXO not found errors

pub mod rpcclient;
pub mod rpcserver;
#[macro_use]
extern crate lazy_static;
use serde_derive::{Deserialize, Serialize};

/// Transaction status identifier for querying transaction state
///
/// Used with the `txStatus` method to retrieve transaction confirmation status
/// and final transaction results from the blockchain.
///
/// # Example
/// ```rust
/// use transactionapi::TransactionStatusId;
///
/// let status_id = TransactionStatusId {
///     txid: "A1B2C3D4E5F6789012345678901234567890ABCDEF".to_string()
/// };
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TransactionStatusId {
    /// Transaction hash for status querying
    pub txid: String,
}

/// Response wrapper for transaction API calls
///
/// Contains the response data from various API methods including
/// transaction submission results, UTXO queries, and error messages.
///
/// # Example
/// ```rust
/// use transactionapi::TxResponse;
///
/// let response = TxResponse::new("Transaction submitted successfully".to_string());
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TxResponse {
    /// Response content from API call
    pub response: String,
}

impl TxResponse {
    /// Creates a new transaction response
    ///
    /// # Arguments
    /// * `response` - Response string from API call
    ///
    /// # Returns
    /// New TxResponse instance
    pub fn new(response: String) -> Self {
        TxResponse { response: response }
    }
}
