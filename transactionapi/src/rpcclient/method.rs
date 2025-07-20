//! RPC method definitions and response handling for ZkOS Transaction API.
//!
//! This module defines all available JSON-RPC methods and their corresponding
//! response structures for transaction operations, UTXO queries, and blockchain
//! state management.
//!
//! ## Available Methods
//!
//! ### Transaction Operations
//! - `txCommit` - Submit transaction to blockchain
//! - `txStatus` - Query transaction status
//! - `TxQueue` - Queue transaction for processing
//!
//! ### UTXO Queries
//! - `getUtxos` - Get coin UTXOs by address
//! - `getMemoUtxos` - Get memo UTXOs by address  
//! - `getStateUtxos` - Get state UTXOs by address
//! - `allUtxos` - Get all coin UTXOs
//! - `allMemoUtxos` - Get all memo UTXOs
//! - `allSateUtxos` - Get all state UTXOs
//!
//! ### Output Queries
//! - `allOutputs` - Get all coin outputs
//! - `getOutput` - Get specific coin output
//! - `getMemoOutput` - Get specific memo output
//! - `getStateOutput` - Get specific state output
//!
//! ### Database Queries
//! - `getUtxosFromDB` - Query UTXOs from PostgreSQL with filtering

use serde::{Deserialize, Serialize};

/// JSON-RPC methods available in the ZkOS Transaction API
///
/// These methods correspond to the endpoints documented in the API specification.
/// Each method has specific parameter requirements and response formats.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
pub enum Method {
    /// Sends a transaction and immediately returns transaction hash.
    ///
    /// **Parameters**: `[hex_encoded_transaction]`
    /// **Response**: Transaction hash and status
    TxQueue,

    /// Sends a transaction and waits until transaction is fully complete.
    ///
    /// **Parameters**: `[hex_encoded_transaction]` or `[hex_encoded_transaction, twilight_address]` for message transactions
    /// **Response**: Transaction hash and confirmation status
    txCommit,

    /// Queries status of a transaction by hash and returns the final transaction result.
    ///
    /// **Parameters**: `[transaction_hash]`
    /// **Response**: Transaction status and details
    TxStatus,

    /// Get coin UTXOs by address
    ///
    /// **Parameters**: `[hex_encoded_address]`
    /// **Response**: Array of coin UTXOs with block heights
    getUtxos,

    /// Get memo UTXOs by address
    ///
    /// **Parameters**: `[hex_encoded_address]`
    /// **Response**: Array of memo UTXOs with encrypted data
    getMemoUtxos,

    /// Get state UTXOs by address
    ///
    /// **Parameters**: `[hex_encoded_address]`
    /// **Response**: Array of state UTXOs with encrypted state data
    getStateUtxos,

    /// Get all coin UTXOs in the system
    ///
    /// **Parameters**: `[]`
    /// **Response**: Array of all coin UTXOs
    allUtxos,

    /// Get all memo UTXOs in the system
    ///
    /// **Parameters**: `[]`
    /// **Response**: Array of all memo UTXOs
    allMemoUtxos,

    /// Get all state UTXOs in the system
    ///
    /// **Parameters**: `[]`
    /// **Response**: Array of all state UTXOs
    allSateUtxos,

    /// Get all coin outputs in the system
    ///
    /// **Parameters**: `[]`
    /// **Response**: Array of all coin outputs
    allOutputs,

    /// Get specific coin output by UTXO key
    ///
    /// **Parameters**: `[hex_encoded_utxo_key]`
    /// **Response**: Coin output details
    getOutput,

    /// Get specific memo output by UTXO key
    ///
    /// **Parameters**: `[hex_encoded_utxo_key]`
    /// **Response**: Memo output details
    getMemoOutput,

    /// Get specific state output by UTXO key
    ///
    /// **Parameters**: `[hex_encoded_utxo_key]`
    /// **Response**: State output details
    getStateOutput,

    /// Query UTXOs from PostgreSQL database with filtering
    ///
    /// **Parameters**: `{start_block, end_block, limit, pagination, io_type}`
    /// **Response**: Filtered UTXO array
    getUtxosFromDB,
}

impl Method {}

/// Response structure for all coin outputs query
///
/// Used with the `allOutputs` method to retrieve all coin-type outputs
/// in the system for transaction output analysis.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AllOutputsResponse {
    /// Array of all coin outputs in the system
    pub all_output: Vec<zkvm::zkos_types::Output>,
}

impl AllOutputsResponse {
    /// Extracts coin outputs from RPC response
    ///
    /// # Arguments
    /// * `resp` - RPC response containing hex-encoded output data
    ///
    /// # Returns
    /// Vector of coin outputs deserialized from response
    pub fn get_response(
        resp: crate::rpcclient::txrequest::RpcResponse<serde_json::Value>,
    ) -> Vec<zkvm::zkos_types::Output> {
        let mut result: Vec<zkvm::zkos_types::Output> = Vec::new();

        let tx_hash: Vec<zkvm::zkos_types::Output> = match resp.result {
            Ok(response) => match response {
                serde_json::Value::String(tx_hex_data) => match hex::decode(tx_hex_data) {
                    Ok(u8_bytes) => match bincode::deserialize(&u8_bytes) {
                        Ok(output_vec) => {
                            result = output_vec;
                            result
                        }
                        Err(_args) => result,
                    },
                    Err(_args) => result,
                },
                _ => result,
            },
            Err(_arg) => result,
        };
        tx_hash
    }
}

/// Response structure for coin UTXOs query
///
/// Used with the `getUtxos` method to retrieve coin-type UTXOs
/// for a specific address, showing transaction history.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetUtxosResponse {
    /// Array of coin UTXOs for the queried address
    pub all_utxo: Vec<zkvm::zkos_types::Utxo>,
}

impl GetUtxosResponse {
    /// Extracts coin UTXOs from RPC response
    ///
    /// # Arguments
    /// * `resp` - RPC response containing UTXO data
    ///
    /// # Returns
    /// Vector of coin UTXOs for the address
    pub fn get_response(
        resp: crate::rpcclient::txrequest::RpcResponse<serde_json::Value>,
    ) -> Vec<zkvm::zkos_types::Utxo> {
        let utxo_vec: Vec<zkvm::zkos_types::Utxo> = match resp.result {
            Ok(response) => {
                let response_result = match serde_json::from_value(response) {
                    Ok(response) => response,
                    Err(_) => Vec::new(),
                };
                response_result
            }
            Err(_arg) => Vec::new(),
        };
        utxo_vec
    }
}

/// Response structure for memo UTXOs query
///
/// Used with the `getMemoUtxos` method to retrieve memo-type UTXOs
/// containing encrypted message data for a specific address.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetMemoUtxosResponse {
    /// Array of memo UTXOs for the queried address
    pub all_utxo: Vec<zkvm::zkos_types::Utxo>,
}

impl GetMemoUtxosResponse {
    /// Extracts memo UTXOs from RPC response
    ///
    /// # Arguments
    /// * `resp` - RPC response containing memo UTXO data
    ///
    /// # Returns
    /// Vector of memo UTXOs for the address
    pub fn get_response(
        resp: crate::rpcclient::txrequest::RpcResponse<serde_json::Value>,
    ) -> Vec<zkvm::zkos_types::Utxo> {
        let utxo_vec: Vec<zkvm::zkos_types::Utxo> = match resp.result {
            Ok(response) => {
                let response_result = match serde_json::from_value(response) {
                    Ok(response) => response,
                    Err(_) => Vec::new(),
                };
                response_result
            }
            Err(_arg) => Vec::new(),
        };
        utxo_vec
    }
}

/// Response structure for state UTXOs query
///
/// Used with the `getStateUtxos` method to retrieve state-type UTXOs
/// containing smart contract state data for a specific address.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetStateUtxosResponse {
    /// Array of state UTXOs for the queried address
    pub all_utxo: Vec<zkvm::zkos_types::Utxo>,
}

impl GetStateUtxosResponse {
    /// Extracts state UTXOs from RPC response
    ///
    /// # Arguments
    /// * `resp` - RPC response containing state UTXO data
    ///
    /// # Returns
    /// Vector of state UTXOs for the address
    pub fn get_response(
        resp: crate::rpcclient::txrequest::RpcResponse<serde_json::Value>,
    ) -> Vec<zkvm::zkos_types::Utxo> {
        let utxo_vec: Vec<zkvm::zkos_types::Utxo> = match resp.result {
            Ok(response) => {
                let response_result = match serde_json::from_value(response) {
                    Ok(response) => response,
                    Err(_) => Vec::new(),
                };
                response_result
            }
            Err(_arg) => Vec::new(),
        };
        utxo_vec
    }
}

/// Response structure for all UTXOs query
///
/// Used with the `allUtxos`, `allMemoUtxos`, and `allSateUtxos` methods
/// to retrieve all UTXOs of a specific type in the system.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AllUtxoResponse {
    /// Array of all UTXOs of the specified type
    pub all_utxo: Vec<String>,
}

impl AllUtxoResponse {
    /// Extracts all UTXOs from RPC response
    ///
    /// # Arguments
    /// * `resp` - RPC response containing UTXO data
    ///
    /// # Returns
    /// Vector of UTXO identifiers as hex strings
    pub fn get_response(
        resp: crate::rpcclient::txrequest::RpcResponse<serde_json::Value>,
    ) -> AllUtxoResponse {
        let utxo_vec: Vec<String> = match resp.result {
            Ok(response) => {
                let response_result = match serde_json::from_value(response) {
                    Ok(response) => response,
                    Err(_) => Vec::new(),
                };
                response_result
            }
            Err(_arg) => Vec::new(),
        };
        AllUtxoResponse { all_utxo: utxo_vec }
    }
}

/// Response structure for coin output query
///
/// Used with the `getOutput` method to retrieve a specific coin output
/// by its UTXO key for detailed transaction analysis.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetCoinOutputResponse {
    /// Optional coin output data
    pub all_utxo: Option<zkvm::zkos_types::Output>,
}

impl GetCoinOutputResponse {
    /// Extracts coin output from RPC response
    ///
    /// # Arguments
    /// * `resp` - RPC response containing output data
    ///
    /// # Returns
    /// Optional coin output if found
    pub fn get_response(
        resp: crate::rpcclient::txrequest::RpcResponse<serde_json::Value>,
    ) -> GetCoinOutputResponse {
        let utxo_vec: Option<zkvm::zkos_types::Output> = match resp.result {
            Ok(response) => {
                let response_result = match serde_json::from_value(response) {
                    Ok(response) => Some(response),
                    Err(_) => None,
                };
                response_result
            }
            Err(_arg) => None,
        };
        GetCoinOutputResponse { all_utxo: utxo_vec }
    }
}

/// Response structure for memo output query
///
/// Used with the `getMemoOutput` method to retrieve a specific memo output
/// by its UTXO key for detailed message analysis.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetMemoOutputResponse {
    /// Optional memo output data
    pub all_utxo: Option<zkvm::zkos_types::Output>,
}

impl GetMemoOutputResponse {
    /// Extracts memo output from RPC response
    ///
    /// # Arguments
    /// * `resp` - RPC response containing memo output data
    ///
    /// # Returns
    /// Optional memo output if found
    pub fn get_response(
        resp: crate::rpcclient::txrequest::RpcResponse<serde_json::Value>,
    ) -> GetMemoOutputResponse {
        let utxo_vec: Option<zkvm::zkos_types::Output> = match resp.result {
            Ok(response) => {
                let response_result = match serde_json::from_value(response) {
                    Ok(response) => Some(response),
                    Err(_) => None,
                };
                response_result
            }
            Err(_arg) => None,
        };
        GetMemoOutputResponse { all_utxo: utxo_vec }
    }
}

/// Response structure for state output query
///
/// Used with the `getStateOutput` method to retrieve a specific state output
/// by its UTXO key for detailed smart contract state analysis.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetStateOutputResponse {
    /// Optional state output data
    pub all_utxo: Option<zkvm::zkos_types::Output>,
}

impl GetStateOutputResponse {
    /// Extracts state output from RPC response
    ///
    /// # Arguments
    /// * `resp` - RPC response containing state output data
    ///
    /// # Returns
    /// Optional state output if found
    pub fn get_response(
        resp: crate::rpcclient::txrequest::RpcResponse<serde_json::Value>,
    ) -> GetStateOutputResponse {
        let utxo_vec: Option<zkvm::zkos_types::Output> = match resp.result {
            Ok(response) => {
                let response_result = match serde_json::from_value(response) {
                    Ok(response) => Some(response),
                    Err(_) => None,
                };
                response_result
            }
            Err(_arg) => None,
        };
        GetStateOutputResponse { all_utxo: utxo_vec }
    }
}

/// Response structure for database UTXO query
///
/// Used with the `getUtxosFromDB` method to query UTXOs from PostgreSQL
/// with specific filtering parameters including block range, pagination, and UTXO type.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetUtxosFromDBResponse {
    /// Array of UTXOs from database query
    pub utxo_vec: Vec<utxo_in_memory::pgsql::UtxoOutputRaw>,
}

impl GetUtxosFromDBResponse {
    /// Extracts database UTXOs from RPC response
    ///
    /// # Arguments
    /// * `resp` - RPC response containing database UTXO data
    ///
    /// # Returns
    /// Vector of UTXOs from database query
    pub fn get_response(
        resp: crate::rpcclient::txrequest::RpcResponse<serde_json::Value>,
    ) -> GetUtxosFromDBResponse {
        let utxo_vec: Vec<utxo_in_memory::pgsql::UtxoOutputRaw> = match resp.result {
            Ok(response) => {
                // println!("i am here 1 : {:?}", response);
                let data: utxo_in_memory::pgsql::UtxoHexEncodedResult =
                    serde_json::from_value(response).unwrap();
                match data.result {
                    Some(vec_utxo) => {
                        utxo_in_memory::pgsql::UtxoHexDecodeResult::decode_from_hex(vec_utxo).result
                    }
                    None => Vec::new(),
                }
            }
            Err(_arg) => Vec::new(),
        };
        GetUtxosFromDBResponse { utxo_vec: utxo_vec }
    }
}

/// Response structure for transaction commit
///
/// Used with the `txCommit` method to extract transaction hash
/// from successful transaction submission.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetTxCommit {
    /// Transaction hash from commit response
    pub txHash: String,
}

impl GetTxCommit {
    /// Extracts transaction hash from commit response
    ///
    /// # Arguments
    /// * `resp` - RPC response from transaction commit
    ///
    /// # Returns
    /// Transaction hash string or error message
    pub fn get_txhash(
        resp: crate::rpcclient::txrequest::RpcResponse<serde_json::Value>,
    ) -> Result<String, String> {
        let tx_hash: Result<String, String> = match resp.result {
            Ok(response) => match response {
                serde_json::Value::String(txHash) => {
                    match serde_json::from_str::<GetTxCommit>(&txHash) {
                        Ok(value) => Ok(value.txHash),
                        Err(_) => Err(txHash),
                    }
                }
                _ => Err("errror".to_string()),
            },
            Err(arg) => Err(arg.to_string()),
        };
        tx_hash
    }
}
