#![allow(warnings)]
//! JSON-RPC server implementation for ZkOS transaction API.
//!
//! This module provides a comprehensive JSON-RPC 2.0 server with endpoints for:
//! - Transaction submission and verification (txCommit)
//! - UTXO queries by address and type (getUtxos, getMemoUtxos, getStateUtxos)
//! - Output retrieval and search functionality
//! - Database queries with pagination
//! - Test commands for snapshot management and storage operations

use crate::rpcclient::method::GetTxCommit;

use super::error::{RpcError, RpcResult};
use super::service;
use super::types::{UtxoDetailResponse, UtxoRequest};
use address::Standard;
use jsonrpc_core::{MetaIoHandler, Metadata, Params, Value};
use jsonrpc_http_server::{hyper, ServerBuilder};
use std::collections::HashMap;
use transaction::{TransactionData, TransactionType};
use utxo_in_memory::blockoperations::blockprocessing::{
    all_coin_type_output, all_coin_type_utxo, all_memo_type_utxo, all_state_type_utxo,
    search_coin_type_utxo_by_address, search_coin_type_utxo_by_utxo_key,
    search_memo_type_utxo_by_address, search_memo_type_utxo_by_utxo_key,
    search_state_type_utxo_by_address, search_state_type_utxo_by_utxo_key,
    search_utxo_by_utxo_key_bytes, verify_utxo,
};
use utxo_in_memory::db::LocalDBtrait;
use utxo_in_memory::pgsql::{
    get_utxo_from_db_by_block_height_range, QueryUtxoFromDB, TestCommand, TestCommandString,
    UtxoHexEncodedResult,
};
use utxo_in_memory::{ADDRESS_TO_UTXO, UTXO_STORAGE};
use zkvm::zkos_types::{MessageType, Utxo};
use zkvm::IOType;
lazy_static! {
    pub static ref ZKOS_RPC_SERVER_URL_PORT: String =
        std::env::var("ZKOS_RPC_SERVER_URL_PORT").unwrap_or_else(|_| "0.0.0.0:3030".to_string());
}

/// Metadata structure for RPC requests
#[derive(Default, Clone, Debug)]
struct Meta {
    metadata: HashMap<String, Option<String>>,
}

impl Metadata for Meta {}

/// Adds a method to the `MetaIoHandler` that wraps a logic function returning `RpcResult`.
///
/// This helper translates the internal `RpcResult` into a client-facing response.
/// To maintain backward compatibility, it formats any `Err` variants into the legacy
/// string format inside a successful `Ok` response, thus avoiding a breaking change.
fn add_rpc_method<F, T>(io: &mut MetaIoHandler<Meta>, name: &'static str, method: F)
where
    F: Fn(Params) -> T + Send + Sync + 'static,
    T: std::future::Future<Output = RpcResult<Value>> + Send + 'static,
{
    io.add_method_with_meta(name, move |params, _meta| {
        let fut = method(params);
        async {
            match fut.await {
                Ok(val) => Ok(val),
                Err(e) => {
                    // Format the error into the legacy string format to maintain compatibility.
                    let error_string = format!("{{ Error: {} }}", e);
                    // Safely convert the error string to a serde_json::Value and wrap in Ok.
                    // This is considered a "successful" response from the client's perspective.
                    Ok(serde_json::Value::String(error_string))
                }
            }
        }
    });
}

/// Starts the JSON-RPC server with all registered endpoints.
pub fn rpcserver() {
    println!("Starting rpc server");
    let mut io = MetaIoHandler::default();

    // Register all RPC methods using the helper
    add_rpc_method(&mut io, "txCommit", tx_commit_logic);
    add_rpc_method(&mut io, "get_utxos_id", get_utxos_id_logic);
    add_rpc_method(&mut io, "get_utxos_detail", get_utxos_detail_logic);
    add_rpc_method(&mut io, "getUtxos", get_coin_utxos_logic);
    add_rpc_method(&mut io, "getMemoUtxos", get_memo_utxos_logic);
    add_rpc_method(&mut io, "getStateUtxos", get_state_utxos_logic);
    add_rpc_method(&mut io, "allCoinUtxos", all_coin_utxos_logic);
    add_rpc_method(&mut io, "allMemoUtxos", all_memo_utxos_logic);
    add_rpc_method(&mut io, "allStateUtxos", all_state_utxos_logic);
    add_rpc_method(&mut io, "allOutputs", all_outputs_logic);
    add_rpc_method(&mut io, "get_output", get_output_by_key_logic);
    add_rpc_method(&mut io, "getOutput", get_coin_output_logic);
    add_rpc_method(&mut io, "getMemoOutput", get_memo_output_logic);
    add_rpc_method(&mut io, "getStateOutput", get_state_output_logic);
    add_rpc_method(&mut io, "getUtxosFromDB", get_utxos_from_db_logic);
    add_rpc_method(&mut io, "TestCommand", test_command_logic);

    // Start the HTTP server
    eprintln!(
        "Starting jsonRPC server @ {}",
        ZKOS_RPC_SERVER_URL_PORT.as_str()
    );
    let server = ServerBuilder::new(io)
        .threads(20)
        .start_http(
            &ZKOS_RPC_SERVER_URL_PORT
                .parse()
                .expect("Invalid socket address"),
        )
        .expect("Failed to start RPC server");

    println!("started rpc api server");
    server.wait();
}

// --- RPC Method Logic ---

async fn tx_commit_logic(params: Params) -> RpcResult<Value> {
    let vector_params: Vec<String> = params
        .parse()
        .map_err(|e| RpcError::InvalidParams(format!("Could not parse parameters: {}", e)))?;

    if vector_params.is_empty() {
        return Err(RpcError::InvalidParams(
            "Expected a hex-encoded transaction string.".to_string(),
        ));
    }

    let hex_tx = vector_params[0].clone();
    let tx_bytes = hex::decode(hex_tx)?;
    let tx: transaction::Transaction = bincode::deserialize(&tx_bytes)?;

    if !verify_utxo(tx.clone()) {
        return Err(RpcError::TxVerificationError(
            "Failed to verify the input UTXO.".to_string(),
        ));
    }

    tx.clone()
        .verify()
        .map_err(|e| RpcError::TxVerificationError(e.to_string()))?;

    let fee = tx.get_tx_fee();
    match tx.tx_type {
        TransactionType::Transfer | TransactionType::Script => {
            println!("Transfer Tx / Script tx submitted to Zkos Oracle");
            let response = service::tx_commit(tx, fee)
                .await
                .map_err(|e| RpcError::InternalError(e.to_string()))?;
            let response_string = GetTxCommit { txHash: response };
            let response_string = serde_json::to_string(&response_string)?;
            Ok(serde_json::to_value(response_string)?)
        }
        TransactionType::Message => {
            let message = match tx.tx {
                TransactionData::Message(ref message) => message,
                _ => {
                    return Err(RpcError::InvalidParams("Expected a valid Message".into()));
                }
            };

            match message.msg_type {
                MessageType::Burn => {
                    println!("ZkOS burn tx submitted to Zkos Oracle");
                    let response = service::tx_commit(tx, fee)
                        .await
                        .map_err(|e| RpcError::InternalError(e.to_string()))?;
                    let response_string = GetTxCommit { txHash: response };
                    let response_string = serde_json::to_string(&response_string)?;
                    Ok(serde_json::to_value(response_string)?)
                }
                _ => Err(RpcError::InvalidParams(
                    "Expected a valid Burn Message".into(),
                )),
            }
        }
        _ => Err(RpcError::InvalidParams(
            "Expected a valid Transfer/Burn Message".into(),
        )),
    }
}

async fn get_utxos_id_logic(params: Params) -> RpcResult<Value> {
    let utxo_request: UtxoRequest = params
        .parse()
        .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
    let mut address_to_utxo_storage = ADDRESS_TO_UTXO.read().map_err(|_| {
        RpcError::InternalError("Failed to acquire read lock on UTXO storage".to_string())
    })?;

    let utxo_id = address_to_utxo_storage
        .get_utxo_id_by_address(utxo_request.address_or_id, utxo_request.input_type)
        .ok_or(RpcError::UtxoNotFound)?;

    Ok(serde_json::to_value(utxo_id)?)
}

async fn get_utxos_detail_logic(params: Params) -> RpcResult<Value> {
    let utxo_request: UtxoRequest = params
        .parse()
        .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
    let mut address_to_utxo_storage = ADDRESS_TO_UTXO.read().map_err(|_| {
        RpcError::InternalError("Failed to acquire read lock on UTXO storage".to_string())
    })?;

    let utxo_id_hex = address_to_utxo_storage
        .get_utxo_id_by_address(utxo_request.address_or_id, utxo_request.input_type)
        .ok_or(RpcError::UtxoNotFound)?;

    drop(address_to_utxo_storage);

    let utxo_id_bytes = hex::decode(&utxo_id_hex)?;
    let utxo_id: Utxo = bincode::deserialize(&utxo_id_bytes)?;

    let output = search_utxo_by_utxo_key_bytes(utxo_id_bytes, utxo_request.input_type)
        .map_err(|e| RpcError::InternalError(format!("Failed to find UTXO by key bytes: {}", e)))?;

    Ok(serde_json::to_value(&UtxoDetailResponse::new(
        utxo_id, output,
    ))?)
}

/// Generic logic for retrieving UTXOs by address.
async fn get_utxos_by_address_logic<F, R>(params: Params, search_fn: F) -> RpcResult<Value>
where
    F: Fn(address::Standard) -> R,
    R: serde::Serialize,
{
    let vector_params: Vec<String> = params
        .parse()
        .map_err(|e| RpcError::InvalidParams(format!("Could not parse params: {}", e)))?;

    let hex_str = vector_params.get(0).ok_or_else(|| {
        RpcError::InvalidParams("Expected hex string address as first parameter.".to_string())
    })?;

    if hex_str.trim().is_empty() {
        return Err(RpcError::InvalidParams(
            "Address cannot be empty.".to_string(),
        ));
    }

    let address = address::Standard::from_hex_with_error(hex_str)
        .map_err(|e| RpcError::InvalidParams(e.to_string()))?;

    let utxos = search_fn(address);
    Ok(serde_json::to_value(&utxos)?)
}

async fn get_coin_utxos_logic(params: Params) -> RpcResult<Value> {
    get_utxos_by_address_logic(params, search_coin_type_utxo_by_address).await
}

async fn get_memo_utxos_logic(params: Params) -> RpcResult<Value> {
    get_utxos_by_address_logic(params, search_memo_type_utxo_by_address).await
}

async fn get_state_utxos_logic(params: Params) -> RpcResult<Value> {
    get_utxos_by_address_logic(params, search_state_type_utxo_by_address).await
}

/// Generic logic for retrieving all UTXOs of a certain type.
async fn all_utxos_logic<F, R>(search_fn: F) -> RpcResult<Value>
where
    F: Fn() -> R,
    R: serde::Serialize,
{
    let utxos = search_fn();
    Ok(serde_json::to_value(&utxos)?)
}

async fn all_coin_utxos_logic(_: Params) -> RpcResult<Value> {
    all_utxos_logic(all_coin_type_utxo).await
}

async fn all_memo_utxos_logic(_: Params) -> RpcResult<Value> {
    all_utxos_logic(all_memo_type_utxo).await
}

async fn all_state_utxos_logic(_: Params) -> RpcResult<Value> {
    all_utxos_logic(all_state_type_utxo).await
}

async fn all_outputs_logic(_: Params) -> RpcResult<Value> {
    let outputs_hex = all_coin_type_output();
    Ok(serde_json::to_value(&outputs_hex)?)
}

async fn get_output_by_key_logic(params: Params) -> RpcResult<Value> {
    let utxo_request: UtxoRequest = params
        .parse()
        .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
    let utxo_bytes = hex::decode(utxo_request.address_or_id)?;

    let output =
        search_utxo_by_utxo_key_bytes(utxo_bytes, utxo_request.input_type).map_err(|e| {
            RpcError::InternalError(format!("Failed to search UTXO by key bytes: {}", e))
        })?;

    Ok(serde_json::to_value(&output)?)
}

/// Generic logic for retrieving a single output by its UTXO key.
async fn get_output_by_utxo_logic<F, R>(params: Params, search_fn: F) -> RpcResult<Value>
where
    F: Fn(Utxo) -> Result<R, &'static str>, // Add lifetime specifier
    R: serde::Serialize,
{
    let vector_params: Vec<String> = params
        .parse()
        .map_err(|e| RpcError::InvalidParams(format!("Could not parse params: {}", e)))?;
    let hex_str = vector_params.get(0).ok_or_else(|| {
        RpcError::InvalidParams("Expected hex string UTXO key as first parameter.".to_string())
    })?;

    let utxo_bytes = hex::decode(hex_str)?;
    let utxo = Utxo::from_bytes(&utxo_bytes)
        .ok_or_else(|| RpcError::InvalidParams("Invalid UTXO byte sequence.".to_string()))?;

    let output = search_fn(utxo).map_err(|e| RpcError::ResponseError(e.to_string()))?;
    Ok(serde_json::to_value(&output)?)
}

async fn get_coin_output_logic(params: Params) -> RpcResult<Value> {
    get_output_by_utxo_logic(params, search_coin_type_utxo_by_utxo_key).await
}

async fn get_memo_output_logic(params: Params) -> RpcResult<Value> {
    get_output_by_utxo_logic(params, search_memo_type_utxo_by_utxo_key).await
}

async fn get_state_output_logic(params: Params) -> RpcResult<Value> {
    get_output_by_utxo_logic(params, search_state_type_utxo_by_utxo_key).await
}

async fn get_utxos_from_db_logic(params: Params) -> RpcResult<Value> {
    let query_params: QueryUtxoFromDB = params
        .parse()
        .map_err(|e| RpcError::InvalidParams(e.to_string()))?;

    if query_params.limit > 10001 {
        return Err(RpcError::InvalidParams("Max limit is 10000".to_string()));
    }

    let result = get_utxo_from_db_by_block_height_range(
        query_params.start_block,
        query_params.end_block,
        query_params.limit,
        query_params.pagination,
        query_params.io_type,
    )
    .map_err(|e| RpcError::InternalError(format!("Database query failed: {:?}", e)))?;

    let hex_encoded = UtxoHexEncodedResult::encode_to_hex(result.result);
    Ok(serde_json::to_value(&hex_encoded)?)
}

async fn test_command_logic(params: Params) -> RpcResult<Value> {
    let command_params: TestCommand = params
        .parse()
        .map_err(|e| RpcError::InvalidParams(e.to_string()))?;

    match command_params.test_command {
        TestCommandString::TakeSnapshotintoLevelDB => {
            let mut utxo_storage = UTXO_STORAGE
                .write()
                .map_err(|_| RpcError::InternalError("Failed to acquire write lock".to_string()))?;
            utxo_storage
                .take_snapshot()
                .map_err(|e| RpcError::InternalError(format!("Failed to take snapshot: {}", e)))?;
        }
        TestCommandString::LoadBackupFromLevelDB => {
            let mut utxo_storage = UTXO_STORAGE
                .write()
                .map_err(|_| RpcError::InternalError("Failed to acquire write lock".to_string()))?;
            utxo_storage
                .load_from_snapshot()
                .map_err(|e| RpcError::InternalError(format!("Failed to load snapshot: {}", e)))?;
        }
        TestCommandString::TakeSnapshotintoPostgreSQL => {
            utxo_in_memory::db::takesnapshotfrom_memory_to_postgresql_bulk();
        }
        TestCommandString::UtxoCoinDbLength => {
            let utxo_storage = UTXO_STORAGE
                .read()
                .map_err(|_| RpcError::InternalError("Failed to acquire read lock".to_string()))?;
            let coin_db = utxo_storage
                .data
                .get(&0)
                .ok_or_else(|| RpcError::InternalError("Coin UTXO DB not found".to_string()))?;
            return Ok(serde_json::to_value(coin_db.len())?);
        }
        _ => {
            return Err(RpcError::InvalidParams(
                "Unsupported test command".to_string(),
            ));
        }
    }

    Ok(serde_json::to_value("Success")?)
}
