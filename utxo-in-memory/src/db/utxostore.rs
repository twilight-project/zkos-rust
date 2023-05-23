// #![allow(dead_code)]
// #![allow(unused_imports)]
// #![allow(non_camel_case_types)]
use crate::db::*;
use crate::types::*;
use crate::ThreadPool;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::SystemTime;
lazy_static! {
    pub static ref UTXO_STORAGE: Arc<Mutex<UTXOStorage>> = Arc::new(Mutex::new(UTXOStorage::new()));
    pub static ref SNAPSHOT_THREADPOOL: Mutex<ThreadPool> =
        Mutex::new(ThreadPool::new(1, String::from("SnapShot_THREADPOOL")));
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UTXOStorage {
    pub coin_storage: HashMap<UtxoKey, UtxoValue>,
    pub memo_storage: HashMap<UtxoKey, UtxoValue>,
    pub state_storage: HashMap<UtxoKey, UtxoValue>,
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
        id: UtxoKey,
        value: UtxoValue,
        input_type: TxInputType,
    ) -> Result<UTXO, std::io::Error> {
        match input_type {
            TxInputType::Coin => {
                if self.coin_storage.contains_key(&id) {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("utxo:{:?} address already exist", id),
                    ));
                } else {
                    self.coin_storage.insert(id.clone(), value.clone());
                    Ok(UTXO::new(id, value, input_type))
                }
            }
            TxInputType::Memo => {
                if self.memo_storage.contains_key(&id) {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("utxo:{:?} address already exist", id),
                    ));
                } else {
                    self.memo_storage.insert(id.clone(), value.clone());
                    Ok(UTXO::new(id, value, input_type))
                }
            }
            TxInputType::State => {
                if self.state_storage.contains_key(&id) {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("utxo:{:?} address already exist", id),
                    ));
                } else {
                    self.state_storage.insert(id.clone(), value.clone());
                    Ok(UTXO::new(id, value, input_type))
                }
            }
        }
    }

    pub fn remove(&mut self, id: UtxoKey, input_type: TxInputType) -> Result<UTXO, std::io::Error> {
        match input_type {
            TxInputType::Coin => match self.coin_storage.remove(&id) {
                Some(value) => {
                    return Ok(UTXO::new(id, value.clone(), input_type));
                }
                None => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("utxo:{:?} not found", id),
                    ))
                }
            },
            TxInputType::Memo => match self.memo_storage.remove(&id) {
                Some(value) => {
                    return Ok(UTXO::new(id, value.clone(), input_type));
                }
                None => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("utxo:{:?} not found", id),
                    ))
                }
            },
            TxInputType::State => match self.state_storage.remove(&id) {
                Some(value) => {
                    return Ok(UTXO::new(id, value.clone(), input_type));
                }
                None => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("utxo:{:?} not found", id),
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
        id: UtxoKey,
        input_type: TxInputType,
    ) -> Result<UTXO, std::io::Error> {
        match input_type {
            TxInputType::Coin => match self.coin_storage.get(&id) {
                Some(value) => {
                    return Ok(UTXO::new(id, value.clone(), input_type));
                }
                None => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("utxo:{:?} not found", id),
                    ))
                }
            },
            TxInputType::Memo => match self.memo_storage.get(&id) {
                Some(value) => {
                    return Ok(UTXO::new(id, value.clone(), input_type));
                }
                None => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("utxo:{:?} not found", id),
                    ))
                }
            },
            TxInputType::State => match self.state_storage.get(&id) {
                Some(value) => {
                    return Ok(UTXO::new(id, value.clone(), input_type));
                }
                None => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("utxo:{:?} not found", id),
                    ))
                }
            },
        }
    }

    pub fn process_block(&mut self, block: ZkBlock) -> Result<ZkBlockResult, std::io::Error> {
        let mut block_result: ZkBlockResult = ZkBlockResult::new();

        //add utxo sets from block
        for utxo in block.add_utxo.clone() {
            match self.add(utxo.key, utxo.value, utxo.input_type) {
                Ok(utxo) => block_result.utxo_added.push(utxo),
                Err(arg) => block_result.error_vec.push(arg),
            }
        }

        //remove utxo sets from block
        for utxo in block.remove_block.clone() {
            match self.remove(utxo.key, utxo.input_type) {
                Ok(utxo) => block_result.utxo_added.push(utxo),
                Err(arg) => block_result.error_vec.push(arg),
            }
        }
        self.block_height = block.block_height;
        //should i update remaining utxo if some utxo not found or adready existed
        self.aggrigate_log_sequence += 1;

        if self.block_height
            >= self.snaps.snap_rules.block_size_threshold * (self.snaps.currentsnapid + 1)
        {
            let _ = self.take_snapshot();
        }

        Ok(block_result)
    }

    pub fn take_snapshot(&mut self) -> Result<(), std::io::Error> {
        let snapshot_path = self.snaps.snap_rules.path.clone();
        let coin_path = format!("{}-coin", snapshot_path.clone());
        let memo_path = format!("{}-memo", snapshot_path.clone());
        let state_path = format!("{}-state", snapshot_path.clone());
        let snap_path = format!("{}-snapmap", snapshot_path.clone());
        let snap_path1 = format!("{}-snapmap", snapshot_path.clone());
        let last_block = self.block_height.clone();
        let new_snapshot_id = self.snaps.lastsnapid + 1;

        let coin_storage = self.coin_storage.clone();
        let memo_storage = self.memo_storage.clone();
        let state_storage = self.state_storage.clone();

        let inner_snap_threadpool = ThreadPool::new(3, String::from("inner_snap_threadpool"));

        inner_snap_threadpool.execute(move || {
            // take snapshot of coin type utxo
            let coin_db_upload_status = leveldb_custom_put(
                coin_path,
                &bincode::serialize(&new_snapshot_id).unwrap(),
                &bincode::serialize(&coin_storage).unwrap(),
            )
            .unwrap();
        });
        inner_snap_threadpool.execute(move || {
            // take snapshot of memo type utxo
            let memo_db_upload_status = leveldb_custom_put(
                memo_path,
                &bincode::serialize(&new_snapshot_id).unwrap(),
                &bincode::serialize(&memo_storage).unwrap(),
            )
            .unwrap();
        });
        inner_snap_threadpool.execute(move || {
            // take snapshot of state type utxo
            let state_db_upload_status = leveldb_custom_put(
                state_path,
                &bincode::serialize(&new_snapshot_id).unwrap(),
                &bincode::serialize(&state_storage).unwrap(),
            )
            .unwrap();
        });
        inner_snap_threadpool.execute(move || {
            let snapmap_update_status = leveldb_custom_put(
                snap_path,
                &bincode::serialize(&new_snapshot_id).unwrap(),
                &bincode::serialize(&last_block).unwrap(),
            );
        });

        self.snaps.block_height = last_block;
        self.snaps.lastsnapid = self.snaps.currentsnapid;
        self.snaps.currentsnapid = new_snapshot_id;
        self.snaps.aggrigate_log_sequence = self.aggrigate_log_sequence;
        self.snaps.lastsnaptimestamp = std::time::SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros();
        let snap_storage = self.snaps.clone();
        inner_snap_threadpool.execute(move || {
            let snapmap_update_status = leveldb_custom_put(
                snap_path1,
                &bincode::serialize(&String::from("utxosnapshot")).unwrap(),
                &bincode::serialize(&snap_storage).unwrap(),
            );
        });
        Ok(())
    }

    pub fn load_from_snapshot(&mut self) {
        let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
        let last_updated_block = utxo_storage.snaps.block_height;
        let snapshot_id = utxo_storage.snaps.currentsnapid;
        let snapshot_path = utxo_storage.snaps.snap_rules.path.clone();
        let coin_map = leveldb_get_utxo_hashmap(
            format!("{}-coin", snapshot_path),
            &bincode::serialize(&snapshot_id).unwrap(),
        );
        let memo_map = leveldb_get_utxo_hashmap(
            format!("{}-memo", snapshot_path),
            &bincode::serialize(&snapshot_id).unwrap(),
        );
        let state_map = leveldb_get_utxo_hashmap(
            format!("{}-state", snapshot_path),
            &bincode::serialize(&snapshot_id).unwrap(),
        );
        match coin_map {
            Ok(coin) => {
                utxo_storage.coin_storage = coin;
            }
            Err(_) => {}
        }
        match memo_map {
            Ok(coin) => {
                utxo_storage.memo_storage = coin;
            }
            Err(_) => {}
        }
        match state_map {
            Ok(coin) => {
                utxo_storage.state_storage = coin;
            }
            Err(_) => {}
        }
        utxo_storage.block_height = utxo_storage.snaps.block_height;
        utxo_storage.aggrigate_log_sequence = utxo_storage.snaps.aggrigate_log_sequence;

        // check remaining blocks from chain and update the utxo set properly
    }
}

pub fn init_utxo() {
    let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
    utxo_storage.load_from_snapshot();
}
