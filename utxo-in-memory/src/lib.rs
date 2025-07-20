//! ZkOS UTXO State Management System
//!
//! This crate provides a comprehensive UTXO (Unspent Transaction Output) management system
//! for ZkOS, supporting three types of state:
//!
//! - **🪙 Coins**: Confidential digital assets with ElGamal encryption
//! - **📝 Memos**: Programmable data containers with time-bound access
//! - **🏗️ State**: Smart contract state with nonce-based versioning
//!
//! ## Core Features
//!
//! - **In-Memory Storage**: High-performance partitioned storage by UTXO type
//! - **PostgreSQL Persistence**: Reliable state persistence and recovery
//! - **Block Processing**: Real-time blockchain integration via chain oracle
//! - **Address Mapping**: Efficient address-to-UTXO queries
//! - **Snapshot System**: State recovery and backup capabilities
//! - **Telemetry**: Prometheus metrics for monitoring
//!
//! ## Architecture
//!
//! The system maintains state through:
//!
//! - **`LocalStorage<T>`**: Partitioned in-memory storage
//! - **`AddressUtxoIDStorage`**: Address-to-UTXO mapping
//! - **`SnapShot`**: State persistence and recovery
//! - **Block Processing**: Real-time state updates from blockchain
//!
//! ## State Types
//!
//! ### Coins
//! Confidential digital assets with ElGamal encryption for privacy-preserving transfers.
//!
//! ### Memos
//! Programmable data containers with script-based access control and time restrictions.
//!
//! ### State
//! Smart contract state with nonce-based versioning for deterministic state transitions.
//!
//! ## Integration
//!
//! This crate integrates with:
//! - **ZkVM**: For transaction verification and state validation
//! - **Chain Oracle**: For real-time blockchain data
//! - **Transaction API**: For external state queries
//! - **PostgreSQL**: For persistent storage

#![allow(warnings)]
pub mod blockoperations;
pub mod db;
pub mod pgsql;
mod threadpool;
//pub mod types;
#[macro_use]
extern crate lazy_static;
pub use self::db::SnapShot;
pub use self::threadpool::ThreadPool;
use chain_oracle::pubsub_chain;
use chain_oracle::Block;
use chain_oracle::TransactionMessage;
use db::{AddressUtxoIDStorage, LocalDBtrait, LocalStorage};
pub use pgsql::init_psql;
use prometheus::{register_counter, register_gauge, Counter, Encoder, Gauge, TextEncoder};
use std::{
    collections::HashMap,
    fs,
    sync::{Arc, Mutex, RwLock},
};
use tungstenite::{connect, handshake::server::Response, Message, WebSocket};
use url::Url;
use zkvm::{zkos_types::Output, IOType};
lazy_static! {
    /// Global UTXO storage with partitioned storage by type (Coin, Memo, State)
    pub static ref UTXO_STORAGE: Arc<RwLock<LocalStorage::<Output>>> =
        Arc::new(RwLock::new(LocalStorage::<Output>::new(3)));

    /// Address-to-UTXO mapping for efficient queries by address and type
    pub static ref ADDRESS_TO_UTXO: Arc<Mutex<AddressUtxoIDStorage>> =
        Arc::new(Mutex::new(AddressUtxoIDStorage::new()));

    /// Prometheus metric for memo UTXO count
    pub static ref UTXO_MEMO_TELEMETRY_COUNTER: Gauge =
        register_gauge!("utxo_memo_count", "A counter for memo utxo").unwrap();

    /// Prometheus metric for state UTXO count
    pub static ref UTXO_STATE_TELEMETRY_COUNTER: Gauge =
        register_gauge!("utxo_state_count", "A counter for state utxo").unwrap();

    /// Prometheus metric for coin UTXO count
    pub static ref UTXO_COIN_TELEMETRY_COUNTER: Gauge =
        register_gauge!("utxo_coin_count", "A counter for coin utxo").unwrap();

    /// Thread pool for chain oracle subscription processing
    pub static ref ZK_ORACLE_SUBSCRIBER_THREADPOOL: Arc<Mutex<ThreadPool>> =
        Arc::new(Mutex::new(ThreadPool::new(
            1,
            String::from("ZK_ORACLE_SUBSCRIBER_THREADPOOL Threadpool")
        )));

    /// Thread pool for block height writing operations
    pub static ref ZK_ORACLE_HEIGHT_WRITE_THREADPOOL: Arc<Mutex<ThreadPool>> =
        Arc::new(Mutex::new(ThreadPool::new(
            1,
            String::from("ZK_ORACLE_SUBSCRIBER_THREADPOOL Threadpool")
        )));
}
use blockoperations::blockprocessing::{
    total_coin_type_utxos, total_memo_type_utxos, total_state_type_utxos,
};

/// Initializes the UTXO store by loading from PostgreSQL and setting up address mappings.
///
/// This function:
/// 1. Initializes PostgreSQL connection
/// 2. Loads UTXO data from database
/// 3. Builds address-to-UTXO mappings
/// 4. Updates Prometheus metrics
/// 5. Sets up telemetry counters
pub fn init_utxo() {
    println!("starting utxo init");
    init_psql();

    {
        let mut utxo_storage = UTXO_STORAGE.write().unwrap();
        // let _ = utxo_storage.load_from_snapshot();
        let _ = utxo_storage.load_from_snapshot_from_psql();
        let mut address_to_utxo_storage = ADDRESS_TO_UTXO.lock().unwrap();
        for input_type in 0..3 {
            let utxos: &mut std::collections::HashMap<Vec<u8>, Output> =
                utxo_storage.data.get_mut(&input_type).unwrap();

            for (key, output_data) in utxos {
                let addr = output_data.output.get_owner_address().unwrap().clone();

                address_to_utxo_storage.add(
                    IOType::from_usize(input_type).unwrap(),
                    addr,
                    hex::encode(key.clone()),
                );
            }
        }
        drop(address_to_utxo_storage);
        println!("finished loading from psql");
    }

    UTXO_MEMO_TELEMETRY_COUNTER.set(total_memo_type_utxos() as f64);
    UTXO_STATE_TELEMETRY_COUNTER.set(total_state_type_utxos() as f64);
    UTXO_COIN_TELEMETRY_COUNTER.set(total_coin_type_utxos() as f64);

    println!(
        "UTXO Memo Telemetry Counter Value: {}",
        UTXO_MEMO_TELEMETRY_COUNTER.get()
    );
    println!(
        "UTXO coin Telemetry Counter Value: {}",
        UTXO_COIN_TELEMETRY_COUNTER.get()
    );
    println!(
        "UTXO state Telemetry Counter Value: {}",
        UTXO_STATE_TELEMETRY_COUNTER.get()
    );

    //load data from intial block from chain
    // if utxo_storage.block_height == 0 {
    //     let recordutxo = crate::blockoperations::load_genesis_sets();
    //     for utxo in recordutxo {
    //         let _ = utxo_storage.add(
    //             bincode::serialize(&utxo.utx).unwrap(),
    //             utxo.value.clone(),
    //             utxo.value.out_type as usize,
    //         );
    //     }
    //     utxo_storage.block_height = 1;
    // }

    println!("finishing utxo init");
}
//To be done later
// fn establish_websocket_connection(
// ) -> Result<(WebSocket<dyn jsonrpc_core::futures_util::Stream>, Response), String> {
//     let url_str = "ws://0.0.0.0:7001/latestblock";
//     let url = Url::parse(url_str).map_err(|e| format!("Invalid URL: {}", e))?;

//     let (socket, response) =
//         connect(url).map_err(|e| format!("WebSocket connection error: {}", e))?;

//     Ok((socket, response))
// }

/// Starts the ZkOS chain oracle subscriber for real-time block processing.
///
/// This function:
/// 1. Reads the current block height from file
/// 2. Subscribes to blockchain updates via chain oracle
/// 3. Processes incoming blocks to update UTXO state
/// 4. Updates block height tracking
/// 5. Uses thread pools for concurrent processing
pub fn zk_oracle_subscriber() {
    println!("started zk subsciber");
    let block_height = match fs::read_to_string("height.txt") {
        Ok(block_height_str) => match block_height_str.trim().parse::<i64>() {
            Ok(block_height) => block_height,
            Err(_) => {
                eprintln!("Failed to parse block height");
                1
            }
        },
        Err(e) => {
            eprintln!("Failed to read block height: {}", e);
            1
        }
    };
    // let url_str = format!(
    //     "ws://147.182.235.183:7001/latestblock?blockHeight={}",
    //     block_height
    // );
    // println!("url : {:?}", url_str);
    // let url = Url::parse(&url_str);
    // let url: Url = match url {
    //     Ok(url) => url,
    //     Err(e) => {
    //         println!("Invalid URL: {}", e);
    //         return;
    //     }
    // };

    // let (mut socket, response) =
    //     connect(url).expect("Can't establish a web socket connection to ZKOracle");

    //match establish_websocket_connection() {
    //  Ok((mut socket, response)) =>
    let mut oracle_threadpool = ZK_ORACLE_SUBSCRIBER_THREADPOOL.lock().unwrap();
    let (receiver, handle) = pubsub_chain::subscribe_block(true);
    loop {
        match receiver.lock().unwrap().recv() {
            Ok(block) => {
                oracle_threadpool.execute(move || {
                    let height = block.block_height;
                    let result =
                        blockoperations::blockprocessing::process_block_for_utxo_insert(block);
                    // if result.suceess_tx.len() > 0 {
                    //     save_snapshot();
                    // }
                    let mut height_write_threadpool =
                        ZK_ORACLE_HEIGHT_WRITE_THREADPOOL.lock().unwrap();

                    height_write_threadpool.execute(move || {
                        write_block_height(height);
                    });
                    drop(height_write_threadpool);
                });
            }
            Err(arg) => {
                println!("subscriber crashed : {:?}", arg);
                println!("Server disconnected");
                break;
            }
        }
    }
    //Err(error) => {
    // Handle the error in a more controlled manner
    //   eprintln!("Error: {}", error);
    // }
    //}
}

/// Creates a snapshot of the current UTXO state for persistence and recovery.
///
/// This function:
/// 1. Takes a snapshot of the current UTXO storage
/// 2. Logs the snapshot details for debugging
/// 3. Returns the snapshot result
pub fn save_snapshot() {
    let mut utxo_storage = UTXO_STORAGE.write().unwrap();
    println!("get block height:{:#?}", utxo_storage.block_height);
    println!("get snap:{:#?}", utxo_storage.snaps);
    for i in 0..utxo_storage.partition_size {
        println!("get snap:{:#?}", utxo_storage.data.get(&i).unwrap().len());
    }
    let res = utxo_storage.take_snapshot();
    // log the result
    println!("get snap:{:#?}", res);
}

/// Writes the current block height to a file for persistence.
///
/// This function:
/// 1. Writes the block height to "height.txt"
/// 2. Logs success or failure
/// 3. Used for tracking blockchain progress
fn write_block_height(block_height: u64) {
    match fs::write("height.txt", block_height.to_string()) {
        Ok(_) => println!("Successfully wrote block height:{} to file", block_height),
        Err(e) => eprintln!("Failed to write block height: {}", e),
    }
}
