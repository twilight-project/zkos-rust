// #![allow(dead_code)]
// #![allow(unused_imports)]
// #![allow(non_camel_case_types)]
use crate::db::*;
pub type KeyId = Vec<u8>;
pub type InputType = usize;
use crate::ThreadPool;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;
pub type SequenceNumber = usize;

pub trait LocalDBtrait<T> {
    fn new(partition: usize) -> Self;
    fn add(&mut self, id: KeyId, value: T, input_type: usize) -> Result<T, std::io::Error>;
    fn remove(&mut self, id: KeyId, input_type: usize) -> Result<T, std::io::Error>;
    fn search_key(&mut self, id: &KeyId, input_type: usize) -> bool;
    fn get_utxo_by_id(&mut self, id: KeyId, input_type: usize) -> Result<T, std::io::Error>;
    fn take_snapshot(&mut self) -> Result<(), std::io::Error>;
    fn load_from_snapshot(&mut self) -> Result<(), std::io::Error>;
    fn data_meta_update(&mut self, blockheight: usize) -> bool;
    // bulk add and bulk remove functions needed
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LocalStorage<T> {
    pub data: HashMap<InputType, HashMap<KeyId, T>>,
    pub block_height: SequenceNumber,
    pub aggrigate_log_sequence: SequenceNumber,
    pub snaps: SnapShot,
    pub partition_size: usize,
}

impl<T> LocalDBtrait<T> for LocalStorage<T>
where
    T: Clone + serde::Serialize + serde::de::DeserializeOwned + Send + 'static,
{
    fn new(mut partition_size: usize) -> LocalStorage<T> {
        LocalStorage {
            data: {
                let mut data = HashMap::new();
                if partition_size < 1 {
                    partition_size = 1;
                }
                for i in 0..partition_size {
                    data.insert(i, HashMap::new());
                }
                data
            },
            block_height: 0,
            aggrigate_log_sequence: 0,
            snaps: SnapShot::new(partition_size),
            partition_size: partition_size,
        }
    }

    fn add(&mut self, id: KeyId, value: T, input_type: usize) -> Result<T, std::io::Error> {
        self.data
            .get_mut(&input_type)
            .unwrap()
            .insert(id.clone(), value.clone());

        Ok(value)
    }

    fn remove(&mut self, id: KeyId, input_type: usize) -> Result<T, std::io::Error> {
        match self.data.get_mut(&input_type).unwrap().remove(&id) {
            Some(value) => {
                return Ok(value.clone());
            }
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("utxo:{:?} not found", id),
                ))
            }
        }
    }

    fn search_key(&mut self, id: &KeyId, input_type: usize) -> bool {
        self.data.get_mut(&input_type).unwrap().contains_key(id)
    }
    fn get_utxo_by_id(&mut self, id: KeyId, input_type: usize) -> Result<T, std::io::Error> {
        match self.data.get_mut(&input_type).unwrap().get(&id) {
            Some(value) => {
                return Ok(value.clone());
            }
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("utxo:{:?} not found", id),
                ))
            }
        }
    }

    fn take_snapshot(&mut self) -> Result<(), std::io::Error> {
        let snapshot_path = self.snaps.snap_rules.path.clone();
        let snap_path = format!("{}-snapmap", snapshot_path.clone());
        let last_block = self.block_height.clone();
        let new_snapshot_id = self.snaps.lastsnapid + 1;
        let mut snap_partition_clone: Vec<(String, HashMap<KeyId, T>)> = Vec::new();

        let inner_snap_threadpool = ThreadPool::new(
            if self.partition_size >= 5 {
                5
            } else {
                self.partition_size + 1
            },
            String::from("inner_snap_threadpool"),
        );

        for i in 0..self.partition_size {
            snap_partition_clone.push((
                format!("{}-{}", snapshot_path.clone(), i),
                self.data.get(&i).unwrap().clone(),
            ));
        }
        for (path, data) in snap_partition_clone {
            inner_snap_threadpool.execute(move || {
                // take snapshot of coin type utxo
                let coin_db_upload_status = leveldb_custom_put(
                    path,
                    &bincode::serialize(&new_snapshot_id).unwrap(),
                    &bincode::serialize(&data).unwrap(),
                )
                .unwrap();
            });
        }

        self.snaps.block_height = last_block;
        self.snaps.lastsnapid = self.snaps.currentsnapid;
        self.snaps.currentsnapid = new_snapshot_id;
        self.snaps.aggrigate_log_sequence = self.aggrigate_log_sequence;
        self.snaps.lastsnaptimestamp = std::time::SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros();
        let snap_storage = self.snaps.clone();
        //storing snapshot state with keyname "utxosnapshot"
        inner_snap_threadpool.execute(move || {
            let _snapmap_update_status = leveldb_custom_put(
                snap_path.clone(),
                &bincode::serialize(&new_snapshot_id).unwrap(),
                &bincode::serialize(&last_block).unwrap(),
            );

            let _snapmap_update_status = leveldb_custom_put(
                snap_path,
                &bincode::serialize(&String::from("utxosnapshot")).unwrap(),
                &bincode::serialize(&snap_storage).unwrap(),
            );
        });
        Ok(())
    }

    fn load_from_snapshot(&mut self) -> Result<(), std::io::Error> {
        let last_updated_block = self.snaps.block_height;
        let snapshot_id = self.snaps.currentsnapid;
        let snapshot_path = self.snaps.snap_rules.path.clone();
        let mut snap_partition_clone: Vec<Result<Vec<u8>, std::io::Error>> = Vec::new();

        for i in 0..self.partition_size {
            snap_partition_clone.push(leveldb_get_utxo_hashmap1(
                format!("{}-{}", snapshot_path, i),
                &bincode::serialize(&snapshot_id).unwrap(),
            ));
        }

        for (inputtype, result_data) in snap_partition_clone.iter().enumerate() {
            match result_data {
                Ok(data) => {
                    self.data
                        .insert(inputtype, bincode::deserialize(&data).unwrap());
                }
                Err(_) => {}
            }
        }

        self.block_height = self.snaps.block_height;
        self.aggrigate_log_sequence = self.snaps.aggrigate_log_sequence;
        Ok(())
        // check remaining blocks from chain and update the utxo set properly
        //get current block from the chain and update the remaining data from chain
    }

    fn data_meta_update(&mut self, blockheight: usize) -> bool {
        self.block_height = blockheight as usize;
        self.aggrigate_log_sequence += 1;
        if self.block_height
            >= self.snaps.snap_rules.block_size_threshold * (self.snaps.currentsnapid + 1)
        {
            let _ = self.take_snapshot();
        }
        true
    }
}
