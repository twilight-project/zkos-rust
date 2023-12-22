use super::service;
// use crate::rpcserver::types::*;
use jsonrpc_core::types::error::Error as JsonRpcError;
use jsonrpc_core::*;
use jsonrpc_http_server::jsonrpc_core::{MetaIoHandler, Metadata, Params};
use jsonrpc_http_server::{hyper, ServerBuilder};

use std::collections::HashMap;
use transaction::{TransactionData, TransactionType};
use utxo_in_memory::blockoperations::blockprocessing::{
    all_coin_type_output, all_coin_type_utxo, all_memo_type_utxo, all_state_type_utxo,
    search_coin_type_utxo_by_address, search_coin_type_utxo_by_utxo_key,
    search_memo_type_utxo_by_utxo_key, search_state_type_utxo_by_utxo_key, verify_utxo,
};
use utxo_in_memory::db::LocalDBtrait;
use utxo_in_memory::UTXO_STORAGE;
/***************** POstgreSQL Insert Code *********/
use utxo_in_memory::pgsql::{
    get_utxo_from_db_by_block_height_range, QueryUtxoFromDB, TestCommand, TestCommandString,
    UtxoHexEncodedResult,
};
/**************** POstgreSQL Insert Code End **********/

use zkvm::zkos_types::{MessageType, Utxo};
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
        // extract the params vector from the request
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
        // extract the tx hex string
        let hex_tx = vector_params[0].clone();
        if hex_tx.trim().is_empty() {
            let err = JsonRpcError::invalid_params("Expected hex string.".to_string());
            return Err(err);
        }

        //let hex_tx = match params.parse::<Vec<String>>() {
        // Ok(vec) => {
        //    if vec.is_empty() {
        //     let err = JsonRpcError::invalid_params("Expected hex string.".to_string());
        //     return Err(err);
        //   }
        //    let hex_tx = vec[0].clone();
        //  if hex_tx.trim().is_empty() {
        //    let err = JsonRpcError::invalid_params("Expected hex string.".to_string());
        //    return Err(err);
        //  }
        //  hex_tx
        // }
        // Err(args) => {
        //  let err =
        //    JsonRpcError::invalid_params(format!("Expected a hex string, {:?}", args));
        //  return Err(err);
        // }
        // };
        // Decode the tx hex string to bytes
        let tx_bytes = match hex::decode(hex_tx) {
            Ok(bytes) => bytes,
            Err(e) => {
                let err =
                    JsonRpcError::invalid_params(format!("Expected a valid hex string, {:?}", e));
                return Err(err);
            }
        };
        // reconstruct the tx from bytes
        tx = match bincode::deserialize(&tx_bytes) {
            Ok(t) => t,
            Err(e) => {
                let err = JsonRpcError::invalid_params(format!("Expected a valid Tx, {:?}", e));
                return Err(err);
            }
        };

        // check if tx is message type
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

        println!("{:?}", twilight_address);

        // verify the inputs from utxo set for the tx
        let utxo_verified = verify_utxo(tx.clone());
        if utxo_verified == false {
            let response_body = "Error: failed to verify utxo".to_string();
            let response_body = serde_json::Value::String(response_body);
            Ok(response_body)
        } else {
            // verify the tx
            //let transfer_tx = TransactionData::to_transfer(tx.clone().tx).unwrap();
            let tx_verified = tx.clone().verify();

            //let tx_verified = verify_transaction(tx.clone());
            match tx_verified {
                Ok(()) => {
                    // get the tx fee from verified tx
                    let fee = tx.get_tx_fee();
                    // commit the tx
                    // check if transaction is Transfer/BurnMessage
                    match tx.tx_type {
                        TransactionType::Transfer | TransactionType::Script => {
                            println!("Transfer Tx / Script tx");
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
                                    // send the ZkOS burn tx to the Zkos Oracle
                                    let result = service::tx_commit(tx.clone(), fee).await;
                                    //match result {
                                    // Ok(_) => {
                                    println!("ZkOS burn tx submitted to Zkos Oracle");
                                    // The ZkOS burn tx was sucessfully submitted.
                                    // Now the Zkos server needs to send the MintorBurnTx after some delay to the oracle
                                    // The oracle will send the MintorBurnTx to the chain
                                    // seleep the process for 5 seconds
                                    //  std::thread::sleep(std::time::Duration::from_secs(5));
                                    // send the MintorBurnTx initialization to the oracle
                                    // let account = message.input.to_quisquis_account().unwrap();
                                    // let result = service::mint_burn_tx_initiate(message.proof.amount,
                                    //   &account, &message.proof.encrypt_scalar, twilight_address).await;
                                    let response_body = match result {
                                        Ok(response_body) => response_body,
                                        Err(err) => err.to_string(),
                                    };
                                    let response_body = serde_json::Value::String(response_body);
                                    return Ok(response_body);
                                    // }
                                    // Err(err) => {
                                    // let err = JsonRpcError::invalid_params(format!(
                                    //  "Burn Message Error: The burn ZkOS tx was not commited properly"
                                    // ));
                                    // return Err(err);
                                    // }
                                    //}
                                    // let response_body = serde_json::Value::String(response_body);
                                    // Ok(response_body)
                                }
                                _ => {
                                    let err = JsonRpcError::invalid_params(format!(
                                        "Expected a valid Burn Message"
                                    ));
                                    return Err(err);
                                }
                            }
                            // let response_body = service::tx_commit(tx.clone()).await;
                            // let response_body = serde_json::Value::String(response_body);
                            // Ok(response_body)
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
            let result = format!("{{ Error: Utxo not available for provided address}}");
            let response_body = serde_json::to_value(result).expect("Failed to serialize to JSON");
            Ok(response_body)
        }
    });

    io.add_method_with_meta("allUtxos", move |params: Params, _meta: Meta| async move {
        let utxos: Vec<String> = all_coin_type_utxo();
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
        "allMemoUtxos",
        move |params: Params, _meta: Meta| async move {
            let utxos = all_memo_type_utxo();
            if utxos.len() > 0 {
                let response_body =
                    serde_json::to_value(&utxos).expect("Failed to serialize to JSON");
                Ok(response_body)
            } else {
                let result = format!("{{ Error: UTXO do not exist for this type}}");
                let response_body =
                    serde_json::to_value(result).expect("Failed to serialize to JSON");
                Ok(response_body)
            }
        },
    );
    io.add_method_with_meta(
        "allSateUtxos",
        move |params: Params, _meta: Meta| async move {
            let utxos = all_state_type_utxo();
            if utxos.len() > 0 {
                let response_body =
                    serde_json::to_value(&utxos).expect("Failed to serialize to JSON");
                Ok(response_body)
            } else {
                let result = format!("{{ Error: UTXO do not exist for this type}}");
                let response_body =
                    serde_json::to_value(result).expect("Failed to serialize to JSON");
                Ok(response_body)
            }
        },
    );
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
                    TestCommandString::TakeSnapshotintoLevelDB => {
                        let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
                        let _res = utxo_storage.take_snapshot();
                        Ok(serde_json::to_value("".to_string()).unwrap())
                    }
                    TestCommandString::LoadBackupFromLevelDB => {
                        let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
                        let _ = utxo_storage.load_from_snapshot();
                        Ok(serde_json::to_value("".to_string()).unwrap())
                    }
                    TestCommandString::TakeSnapshotintoPostgreSQL => {
                        utxo_in_memory::db::takesnapshotfrom_memory_to_postgresql_bulk();
                        Ok(serde_json::to_value("".to_string()).unwrap())
                    }
                    TestCommandString::UtxoCoinDbLength => {
                        let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
                        let mut length_count = Vec::new();
                        for (i, v) in utxo_storage.data.get_mut(&0).unwrap().iter() {
                            length_count.push(v);
                        }
                        println!(
                            "State length : {}",
                            // utxo_storage.data.get_mut(&2).unwrap().len()
                            length_count.len()
                        );

                        Ok(serde_json::to_value("".to_string()).unwrap())
                    }
                    TestCommandString::UtxoMemoDbLength => {
                        let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
                        let mut length_count = Vec::new();
                        for (i, v) in utxo_storage.data.get_mut(&1).unwrap().iter() {
                            length_count.push(v);
                        }
                        println!(
                            "State length : {}",
                            // utxo_storage.data.get_mut(&2).unwrap().len()
                            length_count.len()
                        );
                        Ok(serde_json::to_value("".to_string()).unwrap())
                    }
                    TestCommandString::UtxoStateDbLength => {
                        let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
                        let mut length_count = Vec::new();
                        for (i, v) in utxo_storage.data.get_mut(&2).unwrap().iter() {
                            length_count.push(v);
                        }
                        println!(
                            "State length : {}",
                            // utxo_storage.data.get_mut(&2).unwrap().len()
                            length_count.len()
                        );
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
