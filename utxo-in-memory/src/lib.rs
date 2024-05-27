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
    pub static ref UTXO_STORAGE: Arc<RwLock<LocalStorage::<Output>>> =
        Arc::new(RwLock::new(LocalStorage::<Output>::new(3)));
    pub static ref ADDRESS_TO_UTXO: Arc<Mutex<AddressUtxoIDStorage>> =
        Arc::new(Mutex::new(AddressUtxoIDStorage::new()));
    pub static ref UTXO_MEMO_TELEMETRY_COUNTER: Gauge =
        register_gauge!("utxo_memo_count", "A counter for memo utxo").unwrap();
    pub static ref UTXO_STATE_TELEMETRY_COUNTER: Gauge =
        register_gauge!("utxo_state_count", "A counter for state utxo").unwrap();
    pub static ref UTXO_COIN_TELEMETRY_COUNTER: Gauge =
        register_gauge!("utxo_coin_count", "A counter for coin utxo").unwrap();
}
use blockoperations::blockprocessing::{
    total_coin_type_utxos, total_memo_type_utxos, total_state_type_utxos,
};

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
pub fn zk_oracle_subscriber() {
    println!("started zk subsciber");
    let block_height = match fs::read_to_string("height.txt") {
        Ok(block_height_str) => match block_height_str.trim().parse::<i64>() {
            Ok(block_height) => block_height,
            Err(_) => {
                eprintln!("Failed to parse block height");
                0
            }
        },
        Err(e) => {
            eprintln!("Failed to read block height: {}", e);
            0
        }
    };
    let url_str = format!("ws://0.0.0.0:7001/latestblock?blockHeight={}", block_height);
    let url = Url::parse(&url_str);
    let url: Url = match url {
        Ok(url) => url,
        Err(e) => {
            println!("Invalid URL: {}", e);
            return;
        }
    };

    let (mut socket, response) =
        connect(url).expect("Can't establish a web socket connection to ZKOracle");

    //match establish_websocket_connection() {
    //  Ok((mut socket, response)) =>
    loop {
        let msg = socket.read_message().expect("Error reading message");
        match msg {
            Message::Text(text) => {
                let block: blockoperations::blockprocessing::Block =
                    serde_json::from_str(&text).unwrap();
                let height = block.block_height;
                let result = blockoperations::blockprocessing::process_block_for_utxo_insert(block);
                if result.suceess_tx.len() > 0 {
                    save_snapshot();
                }
                write_block_height(height);
            }
            Message::Close(_) => {
                println!("Server disconnected");
                break;
            }
            _ => (),
        }
    }
    //Err(error) => {
    // Handle the error in a more controlled manner
    //   eprintln!("Error: {}", error);
    // }
    //}
}

fn save_snapshot() {
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

fn write_block_height(block_height: u64) {
    match fs::write("height.txt", block_height.to_string()) {
        Ok(_) => println!("Successfully wrote block height to file"),
        Err(e) => eprintln!("Failed to write block height: {}", e),
    }
}
