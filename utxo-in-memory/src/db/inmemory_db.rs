#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(non_camel_case_types)]
use super::{SequenceNumber, TxInputType, UTXO_OP};
use crate::snapshot::SnapShot;
use serde_derive::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{mpsc, Arc, Mutex, RwLock};
use std::thread;
use std::time::SystemTime;

lazy_static! {
    pub static ref UTXO_STORAGE: Arc<Mutex<UTXOStorage>> = Arc::new(Mutex::new(UTXOStorage::new()));
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTXOStorage {
    pub coin_storage: HashMap<String, String>,
    pub memo_storage: HashMap<String, String>,
    pub state_storage: HashMap<String, String>,
    pub block_height: SequenceNumber,
    pub aggrigate_log_sequence: SequenceNumber,
    pub snaps: SnapShot,
    pub pending_commands: Vec<String>,
}
impl UTXOStorage {
    pub fn new() -> UTXOStorage {
        UTXOStorage {
            coin_storage: HashMap::new(),
            memo_storage: HashMap::new(),
            state_storage: HashMap::new(),
            block_height: 0,
            aggrigate_log_sequence: 0,
            snaps: SnapShot::new(),
            pending_commands: Vec::new(),
        }
    }

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
    pub fn get_utxo_by_id(
        &mut self,
        id: String,
        input_type: TxInputType,
    ) -> Result<(String, String), std::io::Error> {
        match input_type {
            TxInputType::Coin => match self.coin_storage.get(&id) {
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
            TxInputType::Memo => match self.memo_storage.get(&id) {
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
            TxInputType::State => match self.state_storage.get(&id) {
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
}

pub fn init_utxo() {
    let utxo_storage = UTXO_STORAGE.lock().unwrap();
}
