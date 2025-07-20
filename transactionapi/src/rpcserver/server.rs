#![allow(warnings)]
//! JSON-RPC server implementation for ZkOS transaction API.
//!
//! This module provides a comprehensive JSON-RPC 2.0 server with endpoints for:
//! - Transaction submission and verification (txCommit)
//! - UTXO queries by address and type (getUtxos, getMemoUtxos, getStateUtxos)
//! - Output retrieval and search functionality
//! - Database queries with pagination
//! - Test commands for snapshot management and storage operations

use crate::rpcserver::types::UtxoDetailResponse;

use super::service;
use jsonrpc_core::types::error::Error as JsonRpcError;
use jsonrpc_core::*;
use jsonrpc_http_server::jsonrpc_core::{MetaIoHandler, Metadata, Params};
use jsonrpc_http_server::{hyper, ServerBuilder};
use zkvm::IOType;

use super::types::UtxoRequest;
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
use utxo_in_memory::{ADDRESS_TO_UTXO, UTXO_STORAGE};

/***************** POstgreSQL Insert Code *********/
use utxo_in_memory::pgsql::{
    get_utxo_from_db_by_block_height_range, QueryUtxoFromDB, TestCommand, TestCommandString,
    UtxoHexEncodedResult,
};
/**************** POstgreSQL Insert Code End **********/

use zkvm::zkos_types::{MessageType, Utxo};

/// Metadata structure for RPC requests
#[derive(Default, Clone, Debug)]
struct Meta {
    metadata: HashMap<String, Option<String>>,
}

impl Metadata for Meta {}

/// Starts the JSON-RPC server with all registered endpoints
///
/// The server runs on 127.0.0.1:3030 and provides endpoints for:
/// - Transaction operations (txCommit)
/// - UTXO queries (getUtxos, getMemoUtxos, getStateUtxos)
/// - Output retrieval (getOutput, getMemoOutput, getStateOutput)
/// - Database queries (getUtxosFromDB)
/// - Test commands (TestCommand)
pub fn rpcserver() {
    println!("Starting rpc server");
    let mut io = MetaIoHandler::default();

    // Transaction commit endpoint
    io.add_method_with_meta("txCommit", move |params: Params, _meta: Meta| async move {
        let tx: transaction::Transaction;

        // Parse transaction parameters
        let vector_params: Vec<String> = match params.parse::<Vec<String>>() {
            Ok(vec) => {
                if vec.is_empty() {
                    let err = JsonRpcError::invalid_params("Expected hex string.".to_string());
                    return Err(err);
                }
                vec
            }
            Err(args) => {
                let err = JsonRpcError::invalid_params(format!(
                    "Incorrect Parameters: Expected a Vec hex string from client, {:?}",
                    args
                ));
                return Err(err);
            }
        };

        // Extract and decode transaction hex string
        let hex_tx = vector_params[0].clone();
        if hex_tx.trim().is_empty() {
            let err = JsonRpcError::invalid_params("Expected hex string.".to_string());
            return Err(err);
        }

        let tx_bytes = match hex::decode(hex_tx) {
            Ok(bytes) => bytes,
            Err(e) => {
                let err =
                    JsonRpcError::invalid_params(format!("Expected a valid hex string, {:?}", e));
                return Err(err);
            }
        };

        // Deserialize transaction from bytes
        tx = match bincode::deserialize(&tx_bytes) {
            Ok(t) => t,
            Err(e) => {
                let err = JsonRpcError::invalid_params(format!("Expected a valid Tx, {:?}", e));
                return Err(err);
            }
        };

        // Handle message type transactions with twilight address
        let twilight_address = if tx.tx_type == TransactionType::Message {
            let address = vector_params[1].clone();
            if address.trim().is_empty() {
                let err = JsonRpcError::invalid_params("Expected hex string.".to_string());
                return Err(err);
            }
            address
        } else {
            "".to_string()
        };

        // Verify UTXO inputs
        let utxo_verified = verify_utxo(tx.clone());
        if utxo_verified == false {
            let response_body = "Error: Failed to verify the Input Utxo".to_string();
            let response_body = serde_json::Value::String(response_body);
            Ok(response_body)
        } else {
            // Verify transaction
            let tx_verified = tx.clone().verify();

            match tx_verified {
                Ok(()) => {
                    let fee = tx.get_tx_fee();

                    // Handle different transaction types
                    match tx.tx_type {
                        TransactionType::Transfer | TransactionType::Script => {
                            println!("Transfer Tx / Script tx submitted to Zkos Oracle");
                            let result = service::tx_commit(tx.clone(), fee).await;
                            let response: String = match result {
                                Ok(response_body) => response_body,
                                Err(err) => err.to_string(),
                            };
                            let response_body = serde_json::Value::String(response);
                            Ok(response_body)
                        }
                        TransactionType::Message => {
                            println!("Message tx");
                            let message = match tx.tx.clone() {
                                TransactionData::Message(message) => message,
                                _ => {
                                    let err = JsonRpcError::invalid_params(format!(
                                        "Expected a valid Message"
                                    ));
                                    return Err(err);
                                }
                            };

                            match message.msg_type {
                                MessageType::Burn => {
                                    let result = service::tx_commit(tx.clone(), fee).await;
                                    println!("ZkOS burn tx submitted to Zkos Oracle");
                                    // The ZkOS burn tx was sucessfully submitted.
                                    // Now the Zkos server needs to send the MintorBurnTx after some delay to the oracle
                                    // The oracle will send the MintorBurnTx to the chain

                                    let response_body = match result {
                                        Ok(response_body) => response_body,
                                        Err(err) => err.to_string(),
                                    };
                                    let response_body = serde_json::Value::String(response_body);
                                    return Ok(response_body);
                                }
                                _ => {
                                    let err = JsonRpcError::invalid_params(format!(
                                        "Expected a valid Burn Message"
                                    ));
                                    return Err(err);
                                }
                            }
                        }
                        _ => {
                            let err = JsonRpcError::invalid_params(format!(
                                "Expected a valid Transfer/Burn Message"
                            ));
                            return Err(err);
                        }
                    }
                }
                Err(err_msg) => {
                    let response_body = format!("Verification Error: {}", err_msg);
                    let response_body = serde_json::Value::String(response_body);
                    Ok(response_body)
                }
            }
        }
    });

    // UTXO ID retrieval endpoint
    io.add_method_with_meta(
        "get_utxos_id",
        move |params: Params, _meta: Meta| async move {
            let utxo_request: UtxoRequest = match params.parse::<UtxoRequest>() {
                Ok(utxo_request) => utxo_request,
                Err(args) => {
                    let err =
                        JsonRpcError::invalid_params(format!("Expected a hex string, {:?}", args));
                    return Err(err);
                }
            };

            let mut address_to_utxo_storage = ADDRESS_TO_UTXO.lock().unwrap();
            let utxo_id_option = address_to_utxo_storage
                .get_utxo_id_by_address(utxo_request.address_or_id, utxo_request.input_type);

            drop(address_to_utxo_storage);

            match utxo_id_option {
                Some(utxo_id) => {
                    let response_body =
                        serde_json::to_value(&utxo_id).expect("Failed to serialize to JSON");
                    Ok(response_body)
                }
                None => {
                    let result = format!(
                        "{{ Error: {:?} Utxo ID not available for provided address}}",
                        utxo_request.input_type
                    );
                    let response_body =
                        serde_json::to_value(result).expect("Failed to serialize to JSON");
                    Ok(response_body)
                }
            }
        },
    );

    // UTXO detail retrieval endpoint
    io.add_method_with_meta(
        "get_utxos_detail",
        move |params: Params, _meta: Meta| async move {
            let utxo_request: UtxoRequest = match params.parse::<UtxoRequest>() {
                Ok(utxo_request) => utxo_request,
                Err(args) => {
                    let err =
                        JsonRpcError::invalid_params(format!("Expected a hex string, {:?}", args));
                    return Err(err);
                }
            };

            let mut address_to_utxo_storage = ADDRESS_TO_UTXO.lock().unwrap();
            let utxo_id_option = address_to_utxo_storage
                .get_utxo_id_by_address(utxo_request.address_or_id, utxo_request.input_type);

            drop(address_to_utxo_storage);

            match utxo_id_option {
                Some(utxo_id) => {
                    let utxo_bytes = match hex::decode(utxo_id) {
                        Ok(bytes) => bytes,
                        Err(args) => {
                            let err =
                                JsonRpcError::invalid_params(format!("invalid Hex, {:?}", args));
                            return Err(err);
                        }
                    };

                    let response_body = match bincode::deserialize(&utxo_bytes) {
                        Ok(utxo_id) => {
                            match search_utxo_by_utxo_key_bytes(utxo_bytes, utxo_request.input_type)
                            {
                                Ok(output) => {
                                    serde_json::to_value(&UtxoDetailResponse::new(utxo_id, output))
                                        .expect("Failed to serialize to JSON")
                                }
                                Err(err) => {
                                    serde_json::to_value(&err).expect("Failed to serialize to JSON")
                                }
                            }
                        }
                        Err(err) => {
                            let err = JsonRpcError::invalid_params(format!(
                                "Failed to serialize to JSON {:?}",
                                err
                            ));
                            serde_json::to_value(&err).expect("Failed to serialize to JSON")
                        }
                    };

                    Ok(response_body)
                }
                None => {
                    let result = format!(
                        "{{ Error: {:?} Utxo ID not available for provided address}}",
                        utxo_request.input_type
                    );
                    let response_body =
                        serde_json::to_value(result).expect("Failed to serialize to JSON");
                    Ok(response_body)
                }
            }
        },
    );

    // Coin UTXOs by address endpoint
    io.add_method_with_meta("getUtxos", move |params: Params, _meta: Meta| async move {
        let address: address::Standard;

        let hex_str = match params.parse::<Vec<String>>() {
            Ok(vec) => {
                if vec.is_empty() {
                    let err = JsonRpcError::invalid_params("Expected hex string.".to_string());
                    return Err(err);
                }
                let hex_address = vec[0].clone();
                if hex_address.trim().is_empty() {
                    let err = JsonRpcError::invalid_params("Expected hex string.".to_string());
                    return Err(err);
                }
                hex_address
            }
            Err(args) => {
                let err =
                    JsonRpcError::invalid_params(format!("Expected a hex string, {:?}", args));
                return Err(err);
            }
        };

        address = match address::Standard::from_hex_with_error(&hex_str) {
            Ok(addr) => addr,
            Err(e) => {
                let err = JsonRpcError::invalid_params(e.to_string());
                return Err(err);
            }
        };

        let utxos = search_coin_type_utxo_by_address(address);
        if utxos.len() > 0 {
            let response_body = serde_json::to_value(&utxos).expect("Failed to serialize to JSON");
            Ok(response_body)
        } else {
            let result = format!("{{ Error: Coin Utxo ID not available for provided address}}");
            let response_body = serde_json::to_value(result).expect("Failed to serialize to JSON");
            Ok(response_body)
        }
    });

    // Memo UTXOs by address endpoint
    io.add_method_with_meta(
        "getMemoUtxos",
        move |params: Params, _meta: Meta| async move {
            let address: address::Standard;

            let hex_str = match params.parse::<Vec<String>>() {
                Ok(vec) => {
                    if vec.is_empty() {
                        let err = JsonRpcError::invalid_params("Expected hex string.".to_string());
                        return Err(err);
                    }
                    let hex_address = vec[0].clone();
                    if hex_address.trim().is_empty() {
                        let err = JsonRpcError::invalid_params("Expected hex string.".to_string());
                        return Err(err);
                    }
                    hex_address
                }
                Err(args) => {
                    let err =
                        JsonRpcError::invalid_params(format!("Expected a hex string, {:?}", args));
                    return Err(err);
                }
            };

            address = match address::Standard::from_hex_with_error(&hex_str) {
                Ok(addr) => addr,
                Err(e) => {
                    let err = JsonRpcError::invalid_params(e.to_string());
                    return Err(err);
                }
            };

            let utxos = search_memo_type_utxo_by_address(address);
            if utxos.len() > 0 {
                let response_body =
                    serde_json::to_value(&utxos).expect("Failed to serialize to JSON");
                Ok(response_body)
            } else {
                let result = format!("{{ Error: Memo Utxo ID not available for provided address}}");
                let response_body =
                    serde_json::to_value(result).expect("Failed to serialize to JSON");
                Ok(response_body)
            }
        },
    );

    // State UTXOs by address endpoint
    io.add_method_with_meta(
        "getStateUtxos",
        move |params: Params, _meta: Meta| async move {
            let address: address::Standard;

            let hex_str = match params.parse::<Vec<String>>() {
                Ok(vec) => {
                    if vec.is_empty() {
                        let err = JsonRpcError::invalid_params("Expected hex string.".to_string());
                        return Err(err);
                    }
                    let hex_address = vec[0].clone();
                    if hex_address.trim().is_empty() {
                        let err = JsonRpcError::invalid_params("Expected hex string.".to_string());
                        return Err(err);
                    }
                    hex_address
                }
                Err(args) => {
                    let err =
                        JsonRpcError::invalid_params(format!("Expected a hex string, {:?}", args));
                    return Err(err);
                }
            };

            address = match address::Standard::from_hex_with_error(&hex_str) {
                Ok(addr) => addr,
                Err(e) => {
                    let err = JsonRpcError::invalid_params(e.to_string());
                    return Err(err);
                }
            };

            let utxos = search_state_type_utxo_by_address(address);
            if utxos.len() > 0 {
                let response_body =
                    serde_json::to_value(&utxos).expect("Failed to serialize to JSON");
                Ok(response_body)
            } else {
                let result =
                    format!("{{ Error: State Utxo ID not available for provided address}}");
                let response_body =
                    serde_json::to_value(result).expect("Failed to serialize to JSON");
                Ok(response_body)
            }
        },
    );

    // All coin UTXOs endpoint
    io.add_method_with_meta(
        "allCoinUtxos",
        move |params: Params, _meta: Meta| async move {
            let utxos: Vec<String> = all_coin_type_utxo();
            if utxos.len() > 0 {
                let response_body =
                    serde_json::to_value(&utxos).expect("Failed to serialize to JSON");
                Ok(response_body)
            } else {
                let result = format!("{{ Error: UTXO do not exist for Coin type}}");
                let response_body =
                    serde_json::to_value(result).expect("Failed to serialize to JSON");
                Ok(response_body)
            }
        },
    );

    // All memo UTXOs endpoint
    io.add_method_with_meta(
        "allMemoUtxos",
        move |params: Params, _meta: Meta| async move {
            let utxos = all_memo_type_utxo();
            if utxos.len() > 0 {
                let response_body =
                    serde_json::to_value(&utxos).expect("Failed to serialize to JSON");
                Ok(response_body)
            } else {
                let result = format!("{{ Error: UTXO do not exist for Memo type}}");
                let response_body =
                    serde_json::to_value(result).expect("Failed to serialize to JSON");
                Ok(response_body)
            }
        },
    );

    // All state UTXOs endpoint
    io.add_method_with_meta(
        "allStateUtxos",
        move |params: Params, _meta: Meta| async move {
            let utxos = all_state_type_utxo();
            if utxos.len() > 0 {
                let response_body =
                    serde_json::to_value(&utxos).expect("Failed to serialize to JSON");
                Ok(response_body)
            } else {
                let result = format!("{{ Error: UTXO do not exist for State type}}");
                let response_body =
                    serde_json::to_value(result).expect("Failed to serialize to JSON");
                Ok(response_body)
            }
        },
    );

    // All outputs endpoint
    io.add_method_with_meta(
        "allOutputs",
        move |params: Params, _meta: Meta| async move {
            let outputs_hex = all_coin_type_output();
            if outputs_hex.len() > 0 {
                let response_body =
                    serde_json::to_value(&outputs_hex).expect("Failed to serialize to JSON");
                Ok(response_body)
            } else {
                let result = format!("{{ Error: Outputs do not exist for this type}}");
                let response_body =
                    serde_json::to_value(result).expect("Failed to serialize to JSON");
                Ok(response_body)
            }
        },
    );

    // Get output by UTXO key endpoint
    io.add_method_with_meta(
        "get_output",
        move |params: Params, _meta: Meta| async move {
            let utxo_request: UtxoRequest = match params.parse::<UtxoRequest>() {
                Ok(utxo_request) => utxo_request,
                Err(args) => {
                    let err =
                        JsonRpcError::invalid_params(format!("Expected a hex string, {:?}", args));
                    return Err(err);
                }
            };

            let utxo_bytes = match hex::decode(utxo_request.address_or_id) {
                Ok(bytes) => bytes,
                Err(args) => {
                    let err = JsonRpcError::invalid_params(format!("invalid Hex, {:?}", args));
                    return Err(err);
                }
            };

            let response_body =
                match search_utxo_by_utxo_key_bytes(utxo_bytes, utxo_request.input_type) {
                    Ok(output) => {
                        serde_json::to_value(&output).expect("Failed to serialize to JSON")
                    }
                    Err(err) => serde_json::to_value(&err).expect("Failed to serialize to JSON"),
                };

            Ok(response_body)
        },
    );

    // Get coin output endpoint
    io.add_method_with_meta("getOutput", move |params: Params, _meta: Meta| async move {
        let hex_str = match params.parse::<Vec<String>>() {
            Ok(vec) => {
                if vec.is_empty() {
                    let err = JsonRpcError::invalid_params("Expected hex string.".to_string());
                    return Err(err);
                }
                let hex_utxo = vec[0].clone();
                if hex_utxo.trim().is_empty() {
                    let err = JsonRpcError::invalid_params("Expected hex string.".to_string());
                    return Err(err);
                }
                hex_utxo
            }
            Err(args) => {
                let err =
                    JsonRpcError::invalid_params(format!("Expected a hex string, {:?}", args));
                return Err(err);
            }
        };

        let utxo = match hex::decode(hex_str) {
            Ok(bytes) => match Utxo::from_bytes(&bytes) {
                Some(utxo) => utxo,
                None => {
                    let err = JsonRpcError::invalid_params(format!("invalid Hex"));
                    return Err(err);
                }
            },
            Err(args) => {
                let err = JsonRpcError::invalid_params(format!("invalid Hex, {:?}", args));
                return Err(err);
            }
        };

        let response_body = match search_coin_type_utxo_by_utxo_key(utxo) {
            Ok(output) => serde_json::to_value(&output).expect("Failed to serialize to JSON"),
            Err(err) => serde_json::to_value(&err).expect("Failed to serialize to JSON"),
        };

        Ok(response_body)
    });

    // Get memo output endpoint
    io.add_method_with_meta(
        "getMemoOutput",
        move |params: Params, _meta: Meta| async move {
            let hex_str = match params.parse::<Vec<String>>() {
                Ok(vec) => {
                    if vec.is_empty() {
                        let err = JsonRpcError::invalid_params("Expected hex string.".to_string());
                        return Err(err);
                    }
                    let hex_utxo = vec[0].clone();
                    if hex_utxo.trim().is_empty() {
                        let err = JsonRpcError::invalid_params("Expected hex string.".to_string());
                        return Err(err);
                    }
                    hex_utxo
                }
                Err(args) => {
                    let err =
                        JsonRpcError::invalid_params(format!("Expected a hex string, {:?}", args));
                    return Err(err);
                }
            };

            let utxo = match hex::decode(hex_str) {
                Ok(bytes) => match Utxo::from_bytes(&bytes) {
                    Some(utxo) => utxo,
                    None => {
                        let err = JsonRpcError::invalid_params(format!("invalid Hex"));
                        return Err(err);
                    }
                },
                Err(args) => {
                    let err = JsonRpcError::invalid_params(format!("invalid Hex, {:?}", args));
                    return Err(err);
                }
            };

            let response_body = match search_memo_type_utxo_by_utxo_key(utxo) {
                Ok(output) => serde_json::to_value(&output).expect("Failed to serialize to JSON"),
                Err(err) => serde_json::to_value(&err).expect("Failed to serialize to JSON"),
            };

            Ok(response_body)
        },
    );

    // Get state output endpoint
    io.add_method_with_meta(
        "getStateOutput",
        move |params: Params, _meta: Meta| async move {
            let hex_str = match params.parse::<Vec<String>>() {
                Ok(vec) => {
                    if vec.is_empty() {
                        let err = JsonRpcError::invalid_params("Expected hex string.".to_string());
                        return Err(err);
                    }
                    let hex_utxo = vec[0].clone();
                    if hex_utxo.trim().is_empty() {
                        let err = JsonRpcError::invalid_params("Expected hex string.".to_string());
                        return Err(err);
                    }
                    hex_utxo
                }
                Err(args) => {
                    let err =
                        JsonRpcError::invalid_params(format!("Expected a hex string, {:?}", args));
                    return Err(err);
                }
            };

            let utxo = match hex::decode(hex_str) {
                Ok(bytes) => match Utxo::from_bytes(&bytes) {
                    Some(utxo) => utxo,
                    None => {
                        let err = JsonRpcError::invalid_params(format!("invalid Hex"));
                        return Err(err);
                    }
                },
                Err(args) => {
                    let err = JsonRpcError::invalid_params(format!("invalid Hex, {:?}", args));
                    return Err(err);
                }
            };

            let response_body = match search_state_type_utxo_by_utxo_key(utxo) {
                Ok(output) => serde_json::to_value(&output).expect("Failed to serialize to JSON"),
                Err(err) => serde_json::to_value(&err).expect("Failed to serialize to JSON"),
            };

            Ok(response_body)
        },
    );

    // Database UTXO query endpoint with pagination
    io.add_method_with_meta(
        "getUtxosFromDB",
        move |params: Params, _meta: Meta| async move {
            match params.parse::<QueryUtxoFromDB>() {
                Ok(queryparams) => {
                    if queryparams.limit < 10001 {
                        match get_utxo_from_db_by_block_height_range(
                            queryparams.start_block,
                            queryparams.end_block,
                            queryparams.limit,
                            queryparams.pagination,
                            queryparams.io_type,
                        ) {
                            Ok(value) => Ok(serde_json::to_value(
                                &UtxoHexEncodedResult::encode_to_hex(value.result),
                            )
                            .unwrap()),
                            Err(args) => {
                                let err =
                                    JsonRpcError::invalid_params(format!("Error: , {:?}", args));
                                Err(err)
                            }
                        }
                    } else {
                        let err = JsonRpcError::invalid_params(format!(
                            "Invalid parameters, max limit : 10000"
                        ));
                        Err(err)
                    }
                }
                Err(args) => {
                    let err =
                        JsonRpcError::invalid_params(format!("Invalid parameters, {:?}", args));
                    Err(err)
                }
            }
        },
    );

    // Test command endpoint for administrative operations
    io.add_method_with_meta(
        "TestCommand",
        move |params: Params, _meta: Meta| async move {
            match params.parse::<TestCommand>() {
                Ok(queryparams) => match queryparams.test_command {
                    TestCommandString::TakeSnapshotintoLevelDB => {
                        let mut utxo_storage = UTXO_STORAGE.write().unwrap();
                        let _res = utxo_storage.take_snapshot();
                        Ok(serde_json::to_value("".to_string()).unwrap())
                    }
                    TestCommandString::LoadBackupFromLevelDB => {
                        let mut utxo_storage = UTXO_STORAGE.write().unwrap();
                        let _ = utxo_storage.load_from_snapshot();
                        Ok(serde_json::to_value("".to_string()).unwrap())
                    }
                    TestCommandString::TakeSnapshotintoPostgreSQL => {
                        utxo_in_memory::db::takesnapshotfrom_memory_to_postgresql_bulk();
                        Ok(serde_json::to_value("".to_string()).unwrap())
                    }
                    TestCommandString::UtxoCoinDbLength => {
                        let mut utxo_storage = UTXO_STORAGE.write().unwrap();
                        let mut length_count = Vec::new();
                        for (i, v) in utxo_storage.data.get_mut(&0).unwrap().iter() {
                            length_count.push(v);
                        }
                        println!("State length : {}", length_count.len());
                        Ok(serde_json::to_value("".to_string()).unwrap())
                    }
                    TestCommandString::UtxoMemoDbLength => {
                        let mut utxo_storage = UTXO_STORAGE.write().unwrap();
                        let mut length_count = Vec::new();
                        for (i, v) in utxo_storage.data.get_mut(&1).unwrap().iter() {
                            length_count.push(v);
                        }
                        println!("State length : {}", length_count.len());
                        Ok(serde_json::to_value("".to_string()).unwrap())
                    }
                    TestCommandString::UtxoStateDbLength => {
                        let mut utxo_storage = UTXO_STORAGE.write().unwrap();
                        let mut length_count = Vec::new();
                        for (i, v) in utxo_storage.data.get_mut(&2).unwrap().iter() {
                            length_count.push(v);
                        }
                        println!("State length : {}", length_count.len());
                        Ok(serde_json::to_value("".to_string()).unwrap())
                    }
                    _ => {
                        let err = JsonRpcError::invalid_params(format!(
                            "Invalid parameters, enum not exist"
                        ));
                        Err(err)
                    }
                },
                Err(args) => {
                    let err =
                        JsonRpcError::invalid_params(format!("Invalid parameters, {:?}", args));
                    Err(err)
                }
            }
        },
    );

    // Start the HTTP server
    eprintln!("Starting jsonRPC server @ 127.0.0.1:3030");
    let server = ServerBuilder::new(io)
        .threads(20)
        .meta_extractor(|req: &hyper::Request<hyper::Body>| {
            let auth = req
                .headers()
                .get(hyper::header::CONTENT_TYPE)
                .map(|h| h.to_str().unwrap_or("").to_owned());
            let relayer = req
                .headers()
                .get("Relayer")
                .map(|h| h.to_str().unwrap_or("").to_owned());

            Meta {
                metadata: {
                    let mut hashmap = HashMap::new();
                    hashmap.insert(String::from("CONTENT_TYPE"), auth);
                    hashmap.insert(String::from("transaction_key"), relayer);
                    hashmap
                },
            }
        })
        .start_http(&"0.0.0.0:3030".parse().unwrap())
        .unwrap();
    println!("started rpc api server");
    server.wait();
}
