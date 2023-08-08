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

        let response_body = service::tx_commit(tx);
        let response_body = serde_json::Value::String(response_body);
        Ok(response_body)
    });
    // io.add_method_with_meta("TxCommit", move |params: Params, _meta: Meta| async move {
    //     let tx: transaction::Transaction;
    //     match params.parse::<Vec<u8>>() {
    //         Ok(txx) => tx = bincode::deserialize(&txx).unwrap(),
    //         Err(args) => {
    //             let err = JsonRpcError::invalid_params(format!("Invalid parameters, {:?}", args));
    //             return Err(err);
    //         }
    //     }
    //     service::tx_commit(tx);
    //     // Ok(serde_json::to_value("please wait while we proccess your request").unwrap())
    //     Ok(serde_json::to_value(&TxResponse::new(
    //         "please wait while we proccess your request".to_string(),
    //     ))
    //     .unwrap())
    // });
    // io.add_method_with_meta("TxStatus", move |params: Params, _meta: Meta| async move {
    //     match params.parse::<TransactionStatusId>() {
    //         Ok(txx) => service::tx_status(txx),
    //         Err(args) => {
    //             let err = JsonRpcError::invalid_params(format!("Invalid parameters, {:?}", args));
    //             return Err(err);
    //         }
    //     }
    //     Ok(serde_json::to_value(&TxResponse::new(
    //         "Checking for status.. Your order updated successfully".to_string(),
    //     ))
    //     .unwrap())
    // });

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
    server.wait();
}

use std::fs::File;
use std::io::prelude::*;
