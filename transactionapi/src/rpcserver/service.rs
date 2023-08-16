use super::threadpool::ThreadPool;
use reqwest::Client;
use hex;
use std::error::Error;
use sha3::{Digest, Keccak256};
use serde::{Serialize, Deserialize};
// use std::sync::mpsc;
// use std::sync::Arc;
use std::sync::Mutex;
// use std::thread;
use crate::TransactionStatusId;
use transaction::Transaction;
// #[macro_use]
// extern crate lazy_static;
lazy_static! {
    pub static ref THREADPOOL_RPC_QUEUE: Mutex<ThreadPool> =
        Mutex::new(ThreadPool::new(10, String::from("THREADPOOL_RPC_Queue")));
}
pub fn tx_queue(transaction: Transaction) {
    {
        let queue = THREADPOOL_RPC_QUEUE.lock().unwrap();
        queue.execute(move || {
            tx_commit(transaction);
        });
    } // Mutex lock is automatically dropped here
}

#[tokio::main]
pub async fn tx_commit(transaction: Transaction) -> String{

    let client = Client::new();
    let url = "http://165.232.134.41:7000/transaction";


    let serialized: Vec<u8> = bincode::serialize(&transaction).unwrap();
    let tx_hex = hex::encode(serialized.clone());
    //Creating dummy TxiD of ZKOS Transaction to be used as transaction id 
    let mut hasher = Keccak256::new();
        hasher.update(&serialized);
    let checksum = hasher.finalize();
    let payload = Payload{
        id: hex::encode(checksum.to_vec()),
        tx: tx_hex, 
    };

    let json_data = match serde_json::to_string(&payload) {
        Ok(json_data) => json_data,
        Err(e) => return format!(r#"{{"error": "error in transaction Payload (faulty data)"}}"#)
    };

    let response = match client.post(url)
    .header(reqwest::header::CONTENT_TYPE, "application/json")
    .body(json_data)
    .send().await{
        Ok(response) => response,
        Err(e) => return format!(r#"{{"error": "error in commiting transaction"}}"#)
    };

    let response_body: String = match response.text().await{
        Ok(response_body) => return response_body,
        Err(e) => return format!(r#"{{"error": "error in commiting transaction"}}"#)
    };   
}




pub fn tx_status(transaction: TransactionStatusId) {}

#[derive(Serialize, Deserialize)]
struct Payload {
    id: String,
    tx: String,
}

#[derive(Serialize, Deserialize)]
struct Response{
    txHash: String,
}