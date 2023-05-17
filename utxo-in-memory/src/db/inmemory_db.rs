#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(non_camel_case_types)]
use super::{LogSequence, TxInputType, UTXO_OP};
use serde_derive::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{mpsc, Arc, Mutex, RwLock};
use std::thread;
use std::time::SystemTime;

lazy_static! {
    pub static ref UTXO_STORAGE: Arc<Mutex<UTXOStorage>> = Arc::new(Mutex::new(UTXOStorage::new()));
    // pub static ref UTXO_STORAGE1: UTXOStorage = UTXOStorage::new();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTXOStorage {
    pub coin_storage: HashMap<String, String>,
    pub memo_storage: HashMap<String, String>,
    pub state_storage: HashMap<String, String>,
    pub sequence: LogSequence,
    pub aggrigate_log_sequence: LogSequence,
    pub last_snapshot_id: LogSequence,
    pub pending_commands: Vec<String>,
}
impl UTXOStorage {
    pub fn new() -> UTXOStorage {
        UTXOStorage {
            coin_storage: HashMap::new(),
            memo_storage: HashMap::new(),
            state_storage: HashMap::new(),
            sequence: 0,
            aggrigate_log_sequence: 0,
            last_snapshot_id: 0,
            pending_commands: Vec::new(),
        }
    }

    // pub fn execute_command_single(cmd: UTXO_OP) {
    //     let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
    //     match cmd {
    //         UTXO_OP::Add(id, value) => {}
    //         UTXO_OP::Remove(id) => {}
    //         UTXO_OP::Search(id) => {}
    //         UTXO_OP::Snapshot(log_id, time) => {}
    //         UTXO_OP::StopTime(time) => {}
    //         UTXO_OP::ReloadDB(log_id, time) => {}
    //         UTXO_OP::AddBulk(id_array) => {}
    //         UTXO_OP::RemoveBUlk(id_array) => {}
    //         UTXO_OP::SearchBulk(id_array) => {}
    //     }
    // }
    // pub fn execute_command_bulk(cmd: UTXO_OP) {
    //     let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
    //     match cmd {
    //         UTXO_OP::AddBulk(id_array) => {}
    //         UTXO_OP::RemoveBUlk(id_array) => {}
    //         UTXO_OP::SearchBulk(id_array) => {}
    //         UTXO_OP::Add(id, value) => {}
    //         UTXO_OP::Remove(id) => {}
    //         UTXO_OP::Search(id) => {}
    //         UTXO_OP::Snapshot(log_id, time) => {}
    //         UTXO_OP::StopTime(time) => {}
    //         UTXO_OP::ReloadDB(log_id, time) => {}
    //     }
    // }

    pub fn add(
        &mut self,
        id: String,
        value: String,
        input_type: TxInputType,
    ) -> Result<(String, String), std::io::Error> {
        match input_type {
            TxInputType::Coin => {
                if self.coin_storage.contains_key(&id) {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "utxo address already exist",
                    ));
                } else {
                    self.coin_storage.insert(id.clone(), value.clone());
                    Ok((id, value))
                }
            }
            TxInputType::Memo => {
                if self.memo_storage.contains_key(&id) {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "utxo address already exist",
                    ));
                } else {
                    self.memo_storage.insert(id.clone(), value.clone());
                    Ok((id, value))
                }
            }
            TxInputType::State => {
                if self.state_storage.contains_key(&id) {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "utxo address already exist",
                    ));
                } else {
                    self.state_storage.insert(id.clone(), value.clone());
                    Ok((id, value))
                }
            }
        }
    }
    pub fn remove(
        &mut self,
        id: String,
        input_type: TxInputType,
    ) -> Result<(String, String), std::io::Error> {
        match input_type {
            TxInputType::Coin => match self.coin_storage.remove(&id) {
                Some(value) => {
                    return Ok((id, value.clone()));
                }
                None => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "utxo not found",
                    ))
                }
            },
            TxInputType::Memo => match self.memo_storage.remove(&id) {
                Some(value) => {
                    return Ok((id, value.clone()));
                }
                None => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "utxo not found",
                    ))
                }
            },
            TxInputType::State => match self.state_storage.remove(&id) {
                Some(value) => {
                    return Ok((id, value.clone()));
                }
                None => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "utxo not found",
                    ))
                }
            },
        }
    }
    pub fn search_key(&mut self, id: String, input_type: TxInputType) -> bool {
        match input_type {
            TxInputType::Coin => self.coin_storage.contains_key(&id),
            TxInputType::Memo => self.memo_storage.contains_key(&id),
            TxInputType::State => self.state_storage.contains_key(&id),
        }
    }
    pub fn get_value(
        &mut self,
        id: String,
        input_type: TxInputType,
    ) -> Result<String, std::io::Error> {
        match input_type {
            TxInputType::Coin => match self.coin_storage.get(&id) {
                Some(value) => {
                    return Ok(value.clone());
                }
                None => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "utxo not found",
                    ))
                }
            },
            TxInputType::Memo => match self.memo_storage.get(&id) {
                Some(value) => {
                    return Ok(value.clone());
                }
                None => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "utxo not found",
                    ))
                }
            },
            TxInputType::State => match self.state_storage.get(&id) {
                Some(value) => {
                    return Ok(value.clone());
                }
                None => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "utxo not found",
                    ))
                }
            },
        }
    }
}

pub fn init_utxo() {
    let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
}
