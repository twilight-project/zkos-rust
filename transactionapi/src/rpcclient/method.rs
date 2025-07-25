use std::process::Output;

use serde::{Deserialize, Serialize};

/// Serialized as the "method" field of JSON-RPC/HTTP requests.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
pub enum Method {
    /// Sends a transaction and immediately returns transaction hash.
    TxQueue,

    /// Sends a transaction and waits until transaction is fully complete.
    // TxCommit,
    txCommit,
    /// Queries status of a transaction by hash and returns the final transaction result.
    TxStatus,
    getUtxos,
    getMemoUtxos,
    getStateUtxos,
    allUtxos,
    allMemoUtxos,
    allSateUtxos,
    allOutputs,
    getOutput,
    getMemoOutput,
    getStateOutput,
    getUtxosFromDB,
    // TestCommand,
}
impl Method {}

// allOutputs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AllOutputsResponse {
    pub all_output: Vec<zkvm::zkos_types::Output>,
}
impl AllOutputsResponse {
    pub fn get_response(
        resp: crate::rpcclient::txrequest::RpcResponse<serde_json::Value>,
    ) -> Vec<zkvm::zkos_types::Output> {
        let mut result: Vec<zkvm::zkos_types::Output> = Vec::new();

        let tx_hash: Vec<zkvm::zkos_types::Output> = match resp.result {
            Ok(response) => match response {
                serde_json::Value::String(tx_hex_data) => {
                    match hex::decode(tx_hex_data) {
                        Ok(u8_bytes) => match bincode::deserialize(&u8_bytes) {
                            Ok(output_vec) => {
                                result = output_vec;
                                result
                            }
                            Err(_args) => result,
                        },
                        // Ok(hex_data) => Ok(hex_data),
                        Err(_args) => result,
                    }
                }
                _ => result,
            },
            Err(_arg) => result,
        };
        tx_hash
    }
}

// getUtxos
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetUtxosResponse {
    pub all_utxo: Vec<zkvm::zkos_types::Utxo>,
}
impl GetUtxosResponse {
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
// getMemoUtxos
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetMemoUtxosResponse {
    pub all_utxo: Vec<zkvm::zkos_types::Utxo>,
}
impl GetMemoUtxosResponse {
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
// getStateUtxos
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetStateUtxosResponse {
    pub all_utxo: Vec<zkvm::zkos_types::Utxo>,
}
impl GetStateUtxosResponse {
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
// thread 'THREADPOOL_ZKOS-0' panicked at /usr/local/cargo/git/checkouts/zkos-83774292048624fe/13fb0ce/transactionapi/src/rpcclient/method.rs:107:62:
// called `Result::unwrap()` on an `Err` value: Error("invalid type: string \"{ Error: Utxo not available for provided address}\", expected a sequence", line: 0, column: 0)
// stack backtrace:
// allUtxos ,allMemoUtxos, allSateUtxos,  allOutputs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AllUtxoResponse {
    pub all_utxo: Vec<String>,
}
impl AllUtxoResponse {
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

// getOutput
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetCoinOutputResponse {
    pub all_utxo: Option<zkvm::zkos_types::Output>,
}
impl GetCoinOutputResponse {
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

// getMemoOutput
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetMemoOutputResponse {
    pub all_utxo: Option<zkvm::zkos_types::Output>,
}
impl GetMemoOutputResponse {
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

// getStateOutput
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetStateOutputResponse {
    pub all_utxo: Option<zkvm::zkos_types::Output>,
}
impl GetStateOutputResponse {
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

// getUtxosFromDB
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetUtxosFromDBResponse {
    pub utxo_vec: Vec<utxo_in_memory::pgsql::UtxoOutputRaw>,
}
impl GetUtxosFromDBResponse {
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
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetTxCommit {
    pub txHash: String,
}
impl GetTxCommit {
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
