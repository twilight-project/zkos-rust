use super::threadpool::ThreadPool;
use super::MintOrBurnTx;
use address::{Address, Network};
use curve25519_dalek::scalar::Scalar;
use hex;
use quisquislib::accounts::Account;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::error::Error;
// use std::sync::mpsc;
// use std::sync::Arc;
use std::sync::Mutex;
// use std::thread;
use crate::TransactionStatusId;
use transaction::Transaction;
use prometheus::{Encoder, TextEncoder, Counter, Gauge, register_counter, register_gauge};
// #[macro_use]
// extern crate lazy_static;
lazy_static! {
    pub static ref THREADPOOL_RPC_QUEUE: Mutex<ThreadPool> =
        Mutex::new(ThreadPool::new(10, String::from("THREADPOOL_RPC_Queue")));
    pub static ref TOTAL_TX_COUNTER: Gauge = register_gauge!("tx counter", "A counter for tx").unwrap();
}
pub fn tx_queue(transaction: Transaction, fee: u64) {
    {
        let queue = THREADPOOL_RPC_QUEUE.lock().unwrap();
        queue.execute(move || {
            tx_commit(transaction, fee);
        });
    } // Mutex lock is automatically dropped here
}

pub async fn tx_commit(transaction: Transaction, fee: u64) -> Result<String, String> {
    let client = Client::new();
    let url = "http://165.232.134.41:7000/transaction";

    let serialized: Vec<u8> = bincode::serialize(&transaction).unwrap();
    let tx_hex = hex::encode(serialized.clone());
    //Creating dummy TxiD of ZKOS Transaction to be used as transaction id
    let mut hasher = Keccak256::new();
    hasher.update(&serialized);
    let checksum = hasher.finalize();
    let payload = Payload {
        id: hex::encode(checksum.to_vec()),
        tx: tx_hex,
        fee,
    };
    // let json_data = serde_json::to_string(&payload)?;
    let json_data = match serde_json::to_string(&payload) {
        Ok(json_data) => json_data,
        Err(e) => {
            return Err(format!(
                r#"{{"error": "error in transaction Payload (faulty data)"}}"#
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
        Err(e) => return Err(format!(r#"{{"error": "error in commiting transaction"}}"#)),
    };
    let response_body: String = match response.text().await {
        Ok(response_body) => response_body,
        Err(e) => return Err(format!(r#"{{"error": "error in commiting transaction"}}"#)),
    };
    TOTAL_TX_COUNTER.inc();
    Ok(response_body)
}

pub async fn mint_burn_tx_initiate(
    value: u64,
    qq_account: &Account,
    encrypt_scalar: &Scalar,
    twilight_address: String,
) -> Result<String, Box<dyn Error>> {
    let client = Client::new();
    let url = "http://165.232.134.41:7000/burnmessage";

    // convert qq_account into hex string
    let qq_account_hex = account_to_hex_str(qq_account, Network::default());
    // convert encrypt_scalar into hex string
    let encrypt_scalar_hex = hex::encode(encrypt_scalar.to_bytes());
    // create payload
    let payload = MintOrBurnPayload {
        btc_value: value,
        qq_account: qq_account_hex,
        encrypt_scalar: encrypt_scalar_hex,
        twilight_address,
    };
    let json_data = serde_json::to_string(&payload)?;
    // let json_data = match serde_json::to_string(&payload) {
    //     Ok(json_data) => json_data,
    //     Err(e) => return format!(r#"{{"error": "error in transaction Payload (faulty data)"}}"#),
    // };

    let response = client
        .post(url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(json_data)
        .send()
        .await?;
    //{
    //  Ok(response) => response,
    //  Err(e) => return format!(r#"{{"error": "error in commiting transaction"}}"#),
    //};
    let response_body = response.text().await?;
    Ok(response_body)
    //let response_body: String = match response.text().await?{
    //  Ok(response_body) => return response_body,
    //  Err(e) => return format!(r#"{{"error": "error in commiting transaction"}}"#)
    //};
}

pub fn tx_status(transaction: TransactionStatusId) {}

#[derive(Serialize, Deserialize)]
struct Payload {
    id: String,
    tx: String,
    fee: u64,
}

#[derive(Serialize, Deserialize)]
struct MintOrBurnPayload {
    btc_value: u64,
    qq_account: String,
    encrypt_scalar: String,
    twilight_address: String,
}

#[derive(Serialize, Deserialize)]
struct Response {
    txHash: String,
}

/// Utility function to convert Account into hex string for sending it over the network
///convert account to bare bytes and then encode the complete sequence to hex
pub fn account_to_hex_str(account: &Account, net: Network) -> String {
    let (pk, enc) = account.get_account();
    // convert pk to standard coin address
    let address = Address::standard_address(net, pk);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&address.as_bytes());
    bytes.extend_from_slice(&enc.to_bytes());
    let hex = hex::encode(bytes);
    hex
}
