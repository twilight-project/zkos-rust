use super::service;
// use crate::rpcserver::types::*;
use crate::{TransactionStatusId, TxResponse};
use jsonrpc_core::futures_util::future::ok;
use jsonrpc_core::types::error::Error as JsonRpcError;
use jsonrpc_core::*;
use jsonrpc_http_server::jsonrpc_core::{MetaIoHandler, Metadata, Params};
use jsonrpc_http_server::{hyper, ServerBuilder};
use utxo_in_memory::{init_utxo, zk_oracle_subscriber};
use std::collections::HashMap;
use transaction::Transaction;
use utxo_in_memory::blockoperations::blockprocessing::{verify_utxo, search_coin_type_utxo_by_address, search_coin_type_utxo_by_utxo_key};
use transaction::reference_tx::verify_transaction;
use quisquislib::ristretto::RistrettoPublicKey;
use zkvm::zkos_types::Utxo;
#[derive(Default, Clone, Debug)]
struct Meta {
    metadata: HashMap<String, Option<String>>,
}
impl Metadata for Meta {}

pub fn rpcserver() {
    // let mut io = IoHandler::default();
    let mut io = MetaIoHandler::default();

    io.add_method_with_meta("TxCommit", move |params: Params, _meta: Meta| async move {
        let tx: transaction::Transaction;
        tx = match params.parse::<Vec<u8>>() {
            Ok(txx) => match bincode::deserialize(&txx) {
                Ok(value) => value,
                Err(args) => {
                    let err =
                        JsonRpcError::invalid_params(format!("Invalid parameters, {:?}", args));
                    return Err(err);
                }
            },
            Err(args) => {
                let err = JsonRpcError::invalid_params(format!("Invalid parameters, {:?}", args));
                return Err(err);
            }
        };

        let response_body = format!("{{ \"Error\": \"\"}}");

        let utxo_verified = verify_utxo(tx.clone());
        if utxo_verified == false {
            let response_body = "{{ \"Error\": \"failed to verify utxo\"}}".to_string();
        }
        else{
            let tx_verified = verify_transaction(tx.clone());
            match tx_verified {
                Ok(()) => {
                    let response_body = service::tx_commit(tx);
                },
                Err(err_msg) => {
                    let response_body = format!("{{ \"Error\": \"{}\"}}", err_msg);
                },
            }
        }

        let response_body = serde_json::Value::String(response_body);
        Ok(response_body)
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
                if hex_address.trim().is_empty(){
                    let err = JsonRpcError::invalid_params("Expected hex string.".to_string());
                    return Err(err);
                }
                hex_address
            },
            Err(args) => {
                let err = JsonRpcError::invalid_params(format!("Expected a hex string, {:?}", args));
                return Err(err);
            }
        };
    
        println!("Received hex string: {}", hex_str);
        address = match address::Standard::from_hex_with_error(&hex_str) {
            Ok(addr) => addr,
            Err(e) => {
                let err = JsonRpcError::invalid_params( e.to_string());
                return Err(err)
            }
        };

        let utxos = search_coin_type_utxo_by_address(address);
        if utxos.len() > 0 {
            println!("{}", hex::encode(utxos[0].to_bytes()));
            let response_body = serde_json::to_value(&utxos).expect("Failed to serialize to JSON");
            Ok(response_body)
        }
        else {
            let result = format!("{{ Error: Utxo not available for provided address}}");
            let response_body = serde_json::to_value(result).expect("Failed to serialize to JSON");
            Ok(response_body)
        }       
    });


    io.add_method_with_meta("getOutput", move |params: Params, _meta: Meta| async move {

        let hex_str = match params.parse::<Vec<String>>() {
            Ok(vec) => {
                if vec.is_empty() {
                    let err = JsonRpcError::invalid_params("Expected hex string.".to_string());
                    return Err(err);
                }
                let hex_utxo = vec[0].clone();
                if hex_utxo.trim().is_empty(){
                    let err = JsonRpcError::invalid_params("Expected hex string.".to_string());
                    return Err(err);
                }
                hex_utxo
            },
            Err(args) => {
                let err = JsonRpcError::invalid_params(format!("Expected a hex string, {:?}", args));
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

        let output = search_coin_type_utxo_by_utxo_key(utxo);
        let response_body = serde_json::to_value(&output).expect("Failed to serialize to JSON");
        Ok(response_body)
    });
    
    

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
