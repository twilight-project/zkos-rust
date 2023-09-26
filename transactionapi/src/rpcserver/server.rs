use super::service;
// use crate::rpcserver::types::*;
use crate::{TransactionStatusId, TxResponse};
use bincode::deserialize;
use jsonrpc_core::futures_util::future::ok;
use jsonrpc_core::types::error::Error as JsonRpcError;
use jsonrpc_core::*;
use jsonrpc_http_server::jsonrpc_core::{MetaIoHandler, Metadata, Params};
use jsonrpc_http_server::{hyper, ServerBuilder};
use quisquislib::ristretto::RistrettoPublicKey;
use std::collections::HashMap;
use transaction::{Transaction, TransactionData};
use utxo_in_memory::blockoperations::blockprocessing::{
    all_coin_type_output, all_coin_type_utxo, search_coin_type_utxo_by_address,
    search_coin_type_utxo_by_utxo_key, verify_utxo,
};
use utxo_in_memory::db::{LocalDBtrait, LocalStorage};
use utxo_in_memory::UTXO_STORAGE;
/***************** POstgreSQL Insert Code *********/
use utxo_in_memory::pgsql::{
    get_utxo_from_db_by_block_height_range, QueryUtxoFromDB, TestCommand, TestCommandString,
    UtxoHexDecodeResult, UtxoHexEncodedResult, UtxoOutputRaw,
};
/**************** POstgreSQL Insert Code End **********/

use utxo_in_memory::{init_utxo, zk_oracle_subscriber};
use zkvm::zkos_types::Utxo;
#[derive(Default, Clone, Debug)]
struct Meta {
    metadata: HashMap<String, Option<String>>,
}
impl Metadata for Meta {}

pub fn rpcserver() {
    // let mut io = IoHandler::default();
    let mut io = MetaIoHandler::default();

    io.add_method_with_meta("txCommit", move |params: Params, _meta: Meta| async move {
        let tx: transaction::Transaction;
        let hex_tx = match params.parse::<Vec<String>>() {
            Ok(vec) => {
                if vec.is_empty() {
                    let err = JsonRpcError::invalid_params("Expected hex string.".to_string());
                    return Err(err);
                }
                let hex_tx = vec[0].clone();
                if hex_tx.trim().is_empty() {
                    let err = JsonRpcError::invalid_params("Expected hex string.".to_string());
                    return Err(err);
                }
                hex_tx
            }
            Err(args) => {
                let err =
                    JsonRpcError::invalid_params(format!("Expected a hex string, {:?}", args));
                return Err(err);
            }
        };

        let tx_bytes = match hex::decode(hex_tx) {
            Ok(bytes) => bytes,
            Err(e) => {
                let err =
                    JsonRpcError::invalid_params(format!("Expected a valid hex string, {:?}", e));
                return Err(err);
            }
        };

        tx = match bincode::deserialize(&tx_bytes) {
            Ok(t) => t,
            Err(e) => {
                let err = JsonRpcError::invalid_params(format!("Expected a valid Tx, {:?}", e));
                return Err(err);
            }
        };

        let utxo_verified = verify_utxo(tx.clone());
        if utxo_verified == false {
            let response_body = "Error: failed to verify utxo".to_string();
            let response_body = serde_json::Value::String(response_body);
            Ok(response_body)
        } else {
            let transfer_tx = TransactionData::to_transfer(tx.clone().tx).unwrap();
            let tx_verified = transfer_tx.verify();
            //let tx_verified = verify_transaction(tx.clone());
            match tx_verified {
                Ok(()) => {
                    let response_body = service::tx_commit(tx).await;
                    let response_body = serde_json::Value::String(response_body);
                    Ok(response_body)
                }
                Err(err_msg) => {
                    let response_body = format!("Verification Error: {}", err_msg);
                    let response_body = serde_json::Value::String(response_body);
                    Ok(response_body)
                }
            }
        }
    });

    io.add_method_with_meta("getUtxos", move |params: Params, _meta: Meta| async move {
        let mut address: address::Standard;

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
            let result = format!("{{ Error: Utxo not available for provided address}}");
            let response_body = serde_json::to_value(result).expect("Failed to serialize to JSON");
            Ok(response_body)
        }
    });

    io.add_method_with_meta("allUtxos", move |params: Params, _meta: Meta| async move {
        let utxos = all_coin_type_utxo();
        if utxos.len() > 0 {
            let response_body = serde_json::to_value(&utxos).expect("Failed to serialize to JSON");
            Ok(response_body)
        } else {
            let result = format!("{{ Error: UTXO do not exist for this type}}");
            let response_body = serde_json::to_value(result).expect("Failed to serialize to JSON");
            Ok(response_body)
        }
    });

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

    io.add_method_with_meta(
        "TestCommand",
        move |params: Params, _meta: Meta| async move {
            match params.parse::<TestCommand>() {
                Ok(queryparams) => match queryparams.test_command {
                    TestCommandString::TakeSnapshotLevelDB => {
                        let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
                        utxo_storage.take_snapshot();
                        Ok(serde_json::to_value("".to_string()).unwrap())
                    }
                    TestCommandString::TakeSnapshotPostgreSQL => {
                        utxo_in_memory::db::takesnapshotfrom_memory_to_postgresql_bulk();
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

    eprintln!("Starting jsonRPC server @ 127.0.0.1:3030");
    let server = ServerBuilder::new(io)
        .threads(5)
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
    println!("started api server");
    server.wait();
}
