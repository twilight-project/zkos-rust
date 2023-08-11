use super::service;
// use crate::rpcserver::types::*;
use crate::{TransactionStatusId, TxResponse};
use jsonrpc_core::futures_util::future::ok;
use jsonrpc_core::types::error::Error as JsonRpcError;
use jsonrpc_core::*;
use jsonrpc_http_server::jsonrpc_core::{MetaIoHandler, Metadata, Params};
use jsonrpc_http_server::{hyper, ServerBuilder};
use std::collections::HashMap;
use transaction::Transaction;
use utxo_in_memory::blockoperations::blockprocessing::{verify_utxo, search_coin_type_utxo_by_public_key};
use transaction::reference_tx::verify_transaction;
use quisquislib::ristretto::RistrettoPublicKey;
use transaction::util::Address;
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

        // TODO : add verification here
        let utxo_verified = verify_utxo(tx.clone());
        let tx_verified = verify_transaction(tx.clone());
        let response_body = format!("{{ \"Error\": \"\"}}");

        match tx_verified {
            Ok(()) => {
                let response_body = service::tx_commit(tx);
            },
            Err(err_msg) => {
                let response_body = format!("{{ \"Error\": \"{}\"}}", err_msg);
            },
        }

        let response_body = serde_json::Value::String(response_body);
        Ok(response_body)
    });


    io.add_method_with_meta("getUtxo", move |params: Params, _meta: Meta| async move {
        let address: Address;
    
        // First, get the hex string from params
        match params.parse::<String>() {
            Ok(hex_str) => {
                // Convert the hex string to Vec<u8>
                match hex::decode(&hex_str) {
                    Ok(data) => {
                        println!("inside match OK");
                        // Deserialize the Vec<u8> to RistrettoPublicKey
                        match bincode::deserialize(&data) {
                            Ok(value) => {println!("inside match OK"); address = value;},
                            Err(args) => {
                                let err = JsonRpcError::invalid_params(format!("Deserialization error, {:?}", args));
                                return Err(err);
                            }
                        }
                    },
                    Err(args) => {
                        let err = JsonRpcError::invalid_params(format!("Hex decode error, {:?}", args));
                        return Err(err);
                    }
                }
            },
            Err(args) => {
                let err = JsonRpcError::invalid_params(format!("Invalid parameters, {:?}", args));
                return Err(err);
            }
        };
    
        let utxos = search_coin_type_utxo_by_public_key(address);
        let response_body = serde_json::to_value(&utxos).expect("Failed to serialize to JSON");
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
