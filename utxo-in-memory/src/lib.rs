pub mod blockoperations;
pub mod db;
mod threadpool;
pub mod types;
#[macro_use]
extern crate lazy_static;
pub use self::db::SnapShot;
pub use self::threadpool::ThreadPool;
use db::{LocalDBtrait, LocalStorage};
use std::sync::{Arc, Mutex};
use tungstenite::{connect, Message};
use url::Url;
use zkvm::zkos_types::Output;

lazy_static! {
    pub static ref UTXO_STORAGE: Arc<Mutex<LocalStorage::<Output>>> =
        Arc::new(Mutex::new(LocalStorage::<Output>::new(3)));
}

pub fn init_utxo() {
    let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
    let _ = utxo_storage.load_from_snapshot();
    //load data from intial block from chain
    if utxo_storage.block_height == 0 {
        let recordutxo = crate::blockoperations::load_genesis_sets();
        for utxo in recordutxo {
            let _ = utxo_storage.add(
                bincode::serialize(&utxo.utx).unwrap(),
                utxo.value.clone(),
                utxo.value.out_type as usize,
            );
        }
        utxo_storage.block_height = 1;
    }
}

pub fn zk_oracle_subscriber() {
    let (mut socket, response) =
        connect(Url::parse("ws://165.232.134.41:7001/latestblock").unwrap()).expect("Can't connect");

    loop {
        let msg = socket.read_message().expect("Error reading message");
        match msg {
            Message::Text(text) => {
                println!("{}", text);
                let block: blockoperations::blockprocessing::Block = serde_json::from_str(&text).unwrap();
                let result = blockoperations::blockprocessing::process_block_for_utxo_insert(block);
                if result.suceess_tx.len() > 0{
                    save_snapshot();
                }
            }
            Message::Close(_) => {
                println!("Server disconnected");
                break;
            }
            _ => (),
        }
    }
}

fn save_snapshot(){
    let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
    println!("get block height:{:#?}", utxo_storage.block_height);
    println!("get snap:{:#?}", utxo_storage.snaps);
    for i in 0..utxo_storage.partition_size {
        println!("get snap:{:#?}", utxo_storage.data.get(&i).unwrap().len());
    }
    utxo_storage.take_snapshot();
}