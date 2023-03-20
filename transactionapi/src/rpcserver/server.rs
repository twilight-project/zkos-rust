use super::service;
use crate::rpcserver::types::*;
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

    io.add_method_with_meta("tx_queue", move |params: Params, _meta: Meta| async move {
        let tx: Transaction = params.parse::<Transaction>().unwrap();
        service::tx_queue(tx);

        Ok(serde_json::to_value("transaction ID").unwrap())
    });
    io.add_method_with_meta("tx_commit", move |params: Params, _meta: Meta| async move {
        let tx: Transaction = params.parse::<Transaction>().unwrap();
        service::tx_commit(tx);
        Ok(serde_json::to_value("Hello world").unwrap())
    });
    io.add_method_with_meta("tx_status", move |params: Params, _meta: Meta| async move {
        let tx: Transaction = params.parse::<Transaction>().unwrap();
        service::tx_status(tx);
        Ok(serde_json::to_value("Hello world").unwrap())
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
    server.wait();
}
