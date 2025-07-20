#![allow(non_snake_case)]
//! RPC service implementation for transaction processing and blockchain interaction.
//!
//! This module provides core RPC service functions for transaction submission,
//! mint/burn operations, and blockchain state queries with thread pool management
//! and Prometheus metrics integration.

use super::error::{ RpcError, RpcResult };
use super::threadpool::ThreadPool;
use crate::TransactionStatusId;
use address::{ Address, Network };
use curve25519_dalek::scalar::Scalar;
use hex;
use prometheus::{ register_gauge, Gauge };
use quisquislib::accounts::Account;
use reqwest::Client;
use serde::{ Deserialize, Serialize };
use sha3::{ Digest, Keccak256 };
use std::sync::Mutex;
use std::time::Duration;
use transaction::Transaction;
use tokio::runtime::Runtime;

lazy_static! {
    /// Thread pool for handling RPC queue operations
    pub static ref THREADPOOL_RPC_QUEUE: Mutex<ThreadPool> = Mutex::new(
        ThreadPool::new(10, String::from("THREADPOOL_RPC_Queue"))
    );
    /// Prometheus gauge for tracking total transaction count
    pub static ref TOTAL_TX_COUNTER: Gauge = register_gauge!(
        "tx_counter",
        "A counter for tx"
    ).unwrap();
    /// Runtime for async operations in thread pool
    pub static ref RUNTIME: Runtime = Runtime::new().unwrap();

    pub static ref ZKORACLE_TX_SUBMIT_URL: String = std::env
        ::var("ZKORACLE_TX_SUBMIT_URL")
        .unwrap_or("https://tx-submit.twilight.rest".to_string());
}

/// Configuration for the transaction service
#[derive(Clone, Debug)]
pub struct ServiceConfig {
    pub oracle_url: String,
    pub timeout_seconds: u64,
    pub max_retries: u32,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            oracle_url: ZKORACLE_TX_SUBMIT_URL.clone(),
            timeout_seconds: 30,
            max_retries: 3,
        }
    }
}

/// Queues a transaction for processing using the thread pool
pub fn tx_queue(transaction: Transaction, fee: u64) -> RpcResult<()> {
    let queue = THREADPOOL_RPC_QUEUE.lock().map_err(|_|
        RpcError::InternalError("Failed to acquire thread pool lock".to_string())
    )?;

    queue.execute(move || {
        if let Err(e) = RUNTIME.block_on(tx_commit(transaction, fee)) {
            eprintln!("Transaction commit failed: {}", e);
        }
    });

    Ok(())
}

/// Commits a transaction to the blockchain via HTTP POST
///
/// # Arguments
/// * `transaction` - Transaction to commit
/// * `fee` - Fee to pay for the transaction
///
/// # Returns
/// Result containing the transaction hash or error message
pub async fn tx_commit(transaction: Transaction, fee: u64) -> RpcResult<String> {
    let config = ServiceConfig::default();
    let client = Client::builder()
        .timeout(Duration::from_secs(config.timeout_seconds))
        .build()
        .map_err(|e| RpcError::InternalError(format!("Failed to create HTTP client: {}", e)))?;

    let url = format!("{}/transaction", config.oracle_url);
    let serialized: Vec<u8> = bincode
        ::serialize(&transaction)
        .map_err(|e| RpcError::SerializationError(e.to_string()))?;

    let tx_hex = hex::encode(serialized.clone());

    // Create transaction ID using Keccak256 hash
    let mut hasher = Keccak256::new();
    hasher.update(&serialized);
    let checksum = hasher.finalize();

    let payload = Payload {
        id: hex::encode(checksum.to_vec()),
        tx: tx_hex,
        fee,
    };

    let json_data = serde_json
        ::to_string(&payload)
        .map_err(|e| RpcError::SerializationError(format!("Failed to serialize payload: {}", e)))?;

    // Retry logic
    let mut last_error = None;
    for attempt in 1..=config.max_retries {
        match
            client
                .post(&url)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(json_data.clone())
                .send().await
        {
            Ok(response) => {
                if response.status().is_success() {
                    let response_body = response
                        .text().await
                        .map_err(|e|
                            RpcError::NetworkError(format!("Failed to read response: {}", e))
                        )?;

                    TOTAL_TX_COUNTER.inc();
                    return Ok(response_body);
                } else {
                    last_error = Some(
                        RpcError::NetworkError(format!("HTTP error: {}", response.status()))
                    );
                }
            }
            Err(e) => {
                last_error = Some(RpcError::NetworkError(format!("Request failed: {}", e)));
                if attempt < config.max_retries {
                    tokio::time::sleep(Duration::from_millis(100 * (attempt as u64))).await;
                    continue;
                }
            }
        }
    }

    Err(
        last_error.unwrap_or_else(|| {
            RpcError::InternalError("Transaction commit failed after all retries".to_string())
        })
    )
}

/// Initiates mint/burn transaction with account and scalar data
pub async fn mint_burn_tx_initiate(
    value: u64,
    qq_account: &Account,
    encrypt_scalar: &Scalar,
    twilight_address: String
) -> RpcResult<String> {
    let config = ServiceConfig::default();
    let client = Client::builder()
        .timeout(Duration::from_secs(config.timeout_seconds))
        .build()
        .map_err(|e| RpcError::InternalError(format!("Failed to create HTTP client: {}", e)))?;

    let url = format!("{}/burnmessage", config.oracle_url);

    let qq_account_hex = account_to_hex_str(qq_account, Network::default());
    let encrypt_scalar_hex = hex::encode(encrypt_scalar.to_bytes());

    let payload = MintOrBurnPayload {
        btc_value: value,
        qq_account: qq_account_hex,
        encrypt_scalar: encrypt_scalar_hex,
        twilight_address,
    };

    let json_data = serde_json
        ::to_string(&payload)
        .map_err(|e| RpcError::SerializationError(format!("Failed to serialize payload: {}", e)))?;

    let response = client
        .post(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(json_data)
        .send().await
        .map_err(|e| RpcError::NetworkError(format!("Request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(RpcError::NetworkError(format!("HTTP error: {}", response.status())));
    }

    let response_body = response
        .text().await
        .map_err(|e| RpcError::NetworkError(format!("Failed to read response: {}", e)))?;

    Ok(response_body)
}

/// Placeholder for transaction status query functionality
pub fn tx_status(transaction: TransactionStatusId) {}

/// Transaction payload structure for blockchain submission
#[derive(Serialize, Deserialize)]
struct Payload {
    id: String,
    tx: String,
    fee: u64,
}

/// Mint/burn operation payload structure
#[derive(Serialize, Deserialize)]
struct MintOrBurnPayload {
    btc_value: u64,
    qq_account: String,
    encrypt_scalar: String,
    twilight_address: String,
}

/// Response structure for transaction operations
#[derive(Serialize, Deserialize)]
struct Response {
    txHash: String,
}

/// Converts Account to hex string for network transmission
///
/// # Arguments
/// * `account` - Account to convert
/// * `net` - Network to use
///
/// # Returns
/// Hex string representation of the account
/// # Note: Convert account to bare bytes and then encode the complete sequence to hex
pub fn account_to_hex_str(account: &Account, net: Network) -> String {
    let (pk, enc) = account.get_account();
    let address = Address::standard_address(net, pk);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&address.as_bytes());
    bytes.extend_from_slice(&enc.to_bytes());
    hex::encode(bytes)
}
