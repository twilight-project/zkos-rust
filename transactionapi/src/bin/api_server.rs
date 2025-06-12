#![allow(warnings)]
use rpcclient::method::Method;
use rpcclient::txrequest::{Resp, RpcBody, RpcRequest};
use rpcserver::*;
use serde_json::to_string;
use std::thread;
use std::time::Duration;
use transaction::Transaction;
use transactionapi::TransactionStatusId;
use transactionapi::{rpcclient, rpcserver};
#[macro_use]
extern crate lazy_static;
use transaction::reference_tx::{
    create_dark_reference_transaction, create_qq_reference_transaction,
};
use utxo_in_memory::blockoperations::blockprocessing::read_telemetry_stats_from_file;
use utxo_in_memory::{init_utxo};
#[macro_use]
extern crate rocket;
use prometheus::{register_counter, register_gauge, Counter, Encoder, Gauge, TextEncoder};
use rocket::data::{Limits, ToByteUnit};
use rocket::{response::content, State};

fn main() {
    init_utxo(); // Execute synchronously
    read_telemetry_stats_from_file();

    let rpc_server_thread = thread::spawn(|| {
        rpcserver();
    });

    // Now start the async part
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async_main());

    rpc_server_thread.join().unwrap();
}

async fn async_main() {
    let telemetry_server_thread = std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            run_telemetry_server().await;
        })
    });

    telemetry_server_thread.join();
}

async fn run_telemetry_server() {
    println!("starting telemetry server");

    let figment = rocket::Config::figment()
        .merge(("address", "0.0.0.0"))
        .merge(("port", 2500))
        .merge(("limits", Limits::new().limit("json", 2.mebibytes())));

    let rocket = rocket::custom(figment).mount("/", routes![telemetry_metrics]);

    // Await the launch of the server
    if let Err(e) = rocket.launch().await {
        println!("Failed to launch server: {}", e);
    }
}

#[get("/metrics")]
fn telemetry_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    let result = String::from_utf8(buffer).unwrap();
    return result;
}
