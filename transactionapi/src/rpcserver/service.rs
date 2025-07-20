//! RPC service implementation for transaction processing and blockchain interaction.
//!
//! This module provides core RPC service functions for transaction submission,
//! mint/burn operations, and blockchain state queries with thread pool management
//! and Prometheus metrics integration.

use super::threadpool::ThreadPool;
use crate::TransactionStatusId;
use address::{Address, Network};
use curve25519_dalek::scalar::Scalar;
use hex;
use prometheus::{register_gauge, Gauge};
use quisquislib::accounts::Account;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::error::Error;
use std::sync::Mutex;
use transaction::Transaction;

lazy_static! {
    /// Thread pool for handling RPC queue operations
    pub static ref THREADPOOL_RPC_QUEUE: Mutex<ThreadPool> =
        Mutex::new(ThreadPool::new(10, String::from("THREADPOOL_RPC_Queue")));
    /// Prometheus gauge for tracking total transaction count
    pub static ref TOTAL_TX_COUNTER: Gauge =
        register_gauge!("tx_counter", "A counter for tx").unwrap();
}

/// Queues a transaction for processing using the thread pool
pub fn tx_queue(transaction: Transaction, fee: u64) {
    {
        let queue = THREADPOOL_RPC_QUEUE.lock().unwrap();
        queue.execute(move || {
            let _ = tx_commit(transaction, fee);
        });
    }
}

/// Commits a transaction to the blockchain via HTTP POST
///
/// # Arguments
/// * `transaction` - Transaction to commit
/// * `fee` - Fee to pay for the transaction
///
/// # Returns
/// Result containing the transaction hash or error message
/// # Note: Assumes the tx is submitted to an oracle which is also running on the same machine.
pub async fn tx_commit(transaction: Transaction, fee: u64) -> Result<String, String> {
    let client = Client::new();
    let url = "http://0.0.0.0:7000/transaction";

    let serialized: Vec<u8> = bincode::serialize(&transaction).map_err(|e| e.to_string())?;
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

    let json_data = match serde_json::to_string(&payload) {
        Ok(json_data) => json_data,
        Err(e) => {
            return Err(format!(
                r#"{{"error": "error in transaction Payload (faulty data) {}"}}"#,
                e
            ))
        }
    };

    let response = match client
        .post(url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(json_data)
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            return Err(format!(
                r#"{{"error": "error in committing transaction: {}"}}"#,
                e
            ))
        }
    };

    let response_body: String = match response.text().await {
        Ok(response_body) => response_body,
        Err(e) => {
            return Err(format!(
                r#"{{"error": "error in committing transaction: {}"}}"#,
                e
            ))
        }
    };

    TOTAL_TX_COUNTER.inc();
    Ok(response_body)
}

/// Initiates mint/burn transaction with account and scalar data
///
/// # Arguments
/// * `value` - Value to mint/burn
/// * `qq_account` - Account to use for mint/burn
/// * `encrypt_scalar` - Scalar to use for mint/burn
/// * `twilight_address` - Address to send the mint/burn transaction to
///
/// # Returns
/// Result containing the transaction hash or error message
pub async fn mint_burn_tx_initiate(
    value: u64,
    qq_account: &Account,
    encrypt_scalar: &Scalar,
    twilight_address: String,
) -> Result<String, Box<dyn Error>> {
    let client = Client::new();
    let url = "http://0.0.0.0:7000/burnmessage";

    // convert qq_account into hex string
    let qq_account_hex = account_to_hex_str(qq_account, Network::default());
    // convert encrypt_scalar into hex string
    let encrypt_scalar_hex = hex::encode(encrypt_scalar.to_bytes());

    let payload = MintOrBurnPayload {
        btc_value: value,
        qq_account: qq_account_hex,
        encrypt_scalar: encrypt_scalar_hex,
        twilight_address,
    };

    let json_data = serde_json::to_string(&payload)?;

    let response = client
        .post(url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(json_data)
        .send()
        .await?;

    let response_body = response.text().await?;
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
