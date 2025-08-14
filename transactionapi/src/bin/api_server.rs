#![allow(warnings)]
//! ZkOS Transaction API Server binary entry point.
//!
//! This module provides the main server binary that initializes and runs:
//! - UTXO storage system
//! - ZkOS Oracle subscriber for blockchain events
//! - JSON-RPC server for transaction API endpoints
//! - Telemetry server with Prometheus metrics
//!
//! The server runs multiple concurrent components:
//! - Main thread: Initialization and coordination
//! - ZkOS subscriber thread: Blockchain event processing
//! - RPC server thread: JSON-RPC request handling
//! - Telemetry server: Metrics endpoint on port 2500

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
use utxo_in_memory::{init_utxo, zk_oracle_subscriber};
#[macro_use]
extern crate rocket;
use prometheus::{register_counter, register_gauge, Counter, Encoder, Gauge, TextEncoder};
use rocket::data::{Limits, ToByteUnit};
use rocket::{response::content, State};

/// Main entry point for the TransactionAPI server.
///
/// Initializes the UTXO storage system, starts the ZkOS Oracle subscriber,
/// launches the JSON-RPC server, and runs the telemetry server for metrics.
///
/// # Components Started
/// - UTXO storage initialization
/// - ZkOS Oracle subscriber thread
/// - JSON-RPC server thread (port 3030)
/// - Telemetry server (port 2500)
fn main() {
    // Initialize UTXO storage synchronously
    init_utxo();

    // Load telemetry statistics from file
    read_telemetry_stats_from_file();

    // Start ZkOS Oracle subscriber in separate thread
    let zk_subscriber_thread = thread::spawn(|| {
        zk_oracle_subscriber();
    });

    // Start JSON-RPC server in separate thread
    let rpc_server_thread = thread::spawn(|| {
        rpcserver();
    });

    // Start async telemetry server
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async_main());

    // Wait for all threads to complete
    zk_subscriber_thread.join().unwrap();
    rpc_server_thread.join().unwrap();
}

/// Async main function for running the telemetry server.
///
/// Creates a separate thread for the telemetry server and waits for it to complete.
/// The telemetry server provides Prometheus metrics on port 2500.
async fn async_main() {
    let telemetry_server_thread = std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            run_telemetry_server().await;
        })
    });

    telemetry_server_thread.join();
}

/// Runs the telemetry server with Prometheus metrics endpoint.
///
/// Configures and starts a Rocket server on port 2500 that serves
/// Prometheus metrics at the `/metrics` endpoint.
///
/// # Configuration
/// - Address: 0.0.0.0
/// - Port: 2500
/// - JSON limit: 2 MiB
async fn run_telemetry_server() {
    println!("starting telemetry server");

    // Configure Rocket server
    let figment = rocket::Config::figment()
        .merge(("address", "0.0.0.0"))
        .merge(("port", 2500))
        .merge(("limits", Limits::new().limit("json", 2.mebibytes())));

    let rocket = rocket::custom(figment).mount("/", routes![telemetry_metrics]);

    // Launch the server
    if let Err(e) = rocket.launch().await {
        println!("Failed to launch server: {}", e);
    }
}

/// Prometheus metrics endpoint handler.
///
/// Returns all registered Prometheus metrics in text format.
/// This endpoint is used by monitoring systems to collect metrics.
///
/// # Returns
/// String containing all Prometheus metrics in text format
#[get("/metrics")]
fn telemetry_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    let result = String::from_utf8(buffer).unwrap();
    return result;
}
