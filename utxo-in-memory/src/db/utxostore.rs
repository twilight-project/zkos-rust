// #![allow(dead_code)]
// #![allow(unused_imports)]
// #![allow(non_camel_case_types)]
use crate::db::*;
use crate::ADDRESS_TO_UTXO;
pub type KeyId = Vec<u8>;
pub type InputType = usize;
use crate::error::UtxosetError;
use crate::ThreadPool;
use serde_derive::{Deserialize, Serialize};
use zkvm::IOType;
use std::collections::HashMap;
use std::time::SystemTime;
pub type SequenceNumber = usize;
use std::sync::mpsc;
use crate::{UTXO_COIN_TELEMETRY_COUNTER, UTXO_MEMO_TELEMETRY_COUNTER, UTXO_STATE_TELEMETRY_COUNTER};
use crate::pgsql::{insert_bulk_utxo_in_psql_coin,insert_bulk_utxo_in_psql_memo_or_state};

use crate::pgsql::{POSTGRESQL_POOL_CONNECTION, THREADPOOL_SQL_QUERY, THREADPOOL_SQL_QUEUE};

pub trait LocalDBtrait<T> {
    fn new(partition: usize) -> Self;

    fn add(&mut self, id: KeyId, value: T, input_type: usize) -> Result<T, std::io::Error>;
    fn remove(&mut self, id: KeyId, input_type: usize) -> Result<T, std::io::Error>;
    fn search_key(&mut self, id: &KeyId, input_type: usize) -> bool;
    fn get_utxo_by_id(&self, id: KeyId, input_type: usize) -> Result<T, std::io::Error>;
    fn take_snapshot(&mut self) -> Result<(), std::io::Error>;
    fn load_from_snapshot(&mut self) -> Result<(), std::io::Error>;
    fn load_from_snapshot_from_psql(&mut self) -> Result<(), std::io::Error>
    fn data_meta_update(&mut self, blockheight: usize) -> bool;
    fn get_count_by_type(&self, input_type: usize) -> u64;
    fn get_utxo_from_db_by_block_height_range1(start_block: i128,limit: i64,pagination: i64,io_type: usize,
    ) -> Result<Vec<UtxokeyidOutput<T>>, UtxosetError> ;
    // bulk add and bulk remove functions needed
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UtxokeyidOutput<T> {
    pub keyid: Vec<u8>,
    pub output: T,
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
            partition_size,
        }
    }

    fn add(&mut self, id: KeyId, value: T, input_type: usize) -> Result<T, UtxosetError> {
        match self.data
            .get_mut(&input_type){
                Some(inner_map) => inner_map.insert(id.clone(), value.clone()),
                None => return Err(UtxosetError::UtxoNotFound),
            };

        match input_type {
            1 => UTXO_COIN_TELEMETRY_COUNTER.inc(),
            2 => UTXO_MEMO_TELEMETRY_COUNTER.inc(),
            3 => UTXO_STATE_TELEMETRY_COUNTER.inc(),
            _ => {}
        }

        Ok(value)
    }

    fn remove(&mut self, id: KeyId, input_type: usize) -> Result<T, UtxosetError> {
        let stoarage = self.data.get_mut(&input_type);
        let value = match stoarage{
            Some(inner_map) => inner_map.remove(&id),
            None => return Err(UtxosetError::UtxoNotFound),
        };
        match value {
            Some(value) => {
                match input_type {
                    1 => UTXO_COIN_TELEMETRY_COUNTER.dec(),
                    2 => UTXO_MEMO_TELEMETRY_COUNTER.dec(),
                    3 => UTXO_STATE_TELEMETRY_COUNTER.dec(),
                    _ => {}
                }
                Ok(value.clone())
            },
            None => Err(UtxosetError::UtxoNotFound),
        }
    }


    fn search_key(&mut self, id: &KeyId, input_type: usize) -> bool {
        self.data.get_mut(&input_type).unwrap().contains_key(id)
    }
    fn get_utxo_by_id(&self, id: KeyId, input_type: usize) -> Result<T, std::io::Error> {
        match self.data.get(&input_type).unwrap().get(&id) {
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
    fn get_utxo_by_id(&mut self, id: KeyId, input_type: usize) -> Result<T, UtxosetError> {
         Ok(self.data.get_mut(&input_type).and_then(|f|f.get(&id)).ok_or(UtxosetError::UtxoNotFound)?.clone())
    }

    fn get_count_by_type(&self, input_type: usize) -> u64 {
        let result: u64 = match self.data.get(&input_type) {
            Some(inner_map) => inner_map.len() as u64,
            None => 0,
        };

        result
    }

    fn take_snapshot(&mut self) -> Result<(), UtxosetError> {
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
                ).expect("error in leveldb_custom_put");
            });
        }

        self.snaps.block_height = last_block;
        self.snaps.lastsnapid = self.snaps.currentsnapid;
        self.snaps.currentsnapid = new_snapshot_id;
        self.snaps.aggrigate_log_sequence = self.aggrigate_log_sequence;
        self.snaps.lastsnaptimestamp = std::time::SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
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

    fn load_from_snapshot(&mut self) -> Result<(), UtxosetError> {
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


    fn load_from_snapshot_from_psql(&mut self) -> Result<(), std::io::Error> {
        let mut address_to_utxo_storage = ADDRESS_TO_UTXO.lock().unwrap();
        for inputtype in 0..self.partition_size {
            let mut pagination_bool = true;
            let mut pagination_counter = 0;
            while pagination_bool {
                let data = LocalStorage::get_utxo_from_db_by_block_height_range1(
                    0,
                    50000,
                    pagination_counter,
                    inputtype,
                );

                match data {
                    Ok(utxo_data) => {
                        if utxo_data.len() > 0 {
                            println!("utxo_data.len():{}",utxo_data.len());
                            for value in utxo_data {
                                self.data
                                    .get_mut(&inputtype)
                                    .unwrap()
                                    .insert(value.keyid, value.output);

                            }
                            pagination_counter += 1;
                        } else {
                            pagination_bool = false;
                            println!("done for iotype:{}",inputtype);
                        }
                    }
                    Err(arg) => {
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, arg));
                    }
                }
            }
        }

        Ok(())
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

     fn get_utxo_from_db_by_block_height_range1(
        start_block: i128,
        limit: i64,
        pagination: i64,
        io_type: usize,
    ) -> Result<Vec<UtxokeyidOutput<T>>, UtxosetError> {
        let public_threadpool = THREADPOOL_SQL_QUERY.lock().unwrap();
        let (sender, receiver) = mpsc::channel();
        public_threadpool.execute(move || {
            let mut query:String="".to_string();
    
            match io_type{
                0=>{   
                    query = format!("SELECT utxo, output FROM public.utxo_coin_logs where block_height>= {} order by block_height asc OFFSET {} limit {};",start_block,pagination*limit,limit);
                     println!("{}",query);
                   },
                1=>{ 
                    query = format!("SELECT utxo, output FROM public.utxo_memo_logs where block_height>= {} order by block_height asc OFFSET {} limit {};",start_block,pagination*limit,limit);
                   println!("{}",query);
              
                   },
                2=>{   
                    query = format!("SELECT utxo, output FROM public.utxo_state_logs where block_height>= {} order by block_height asc OFFSET {} limit {};",start_block,pagination*limit,limit);
                   println!("{}",query);
               
                   },
                _=>{}
            }
    
            let mut client = POSTGRESQL_POOL_CONNECTION.get().unwrap();
            let mut result: Vec<UtxokeyidOutput<T>> = Vec::new();
            match client.query(&query, &[]) {
                Ok(data) => {
                    for row in data {
                        result.push(UtxokeyidOutput {
                            keyid: row.get("utxo"),
                            output:bincode::deserialize(row.get("output")).unwrap() ,
                        });
                    }
                    sender.send(Ok(result)).unwrap();
                }
                Err(arg) => sender
                    .send(Err(std::io::Error::new(std::io::ErrorKind::Other, arg)))
                    .unwrap(),
            }
    
        });
    
        drop(public_threadpool);
    
        match receiver.recv().unwrap() {
            Ok(value) => {
                return Ok(value);
            }
            Err(arg) => {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, arg));
            }
        };
    }
    
}


pub fn takesnapshotfrom_memory_to_postgresql_bulk() {
    let mut utxo_storage = crate::UTXO_STORAGE.write().unwrap();

    let snapshot_path = utxo_storage.snaps.snap_rules.path.clone();
    let snap_path = format!("{}-snapmap", snapshot_path.clone());
    let last_block = utxo_storage.block_height.clone();
    // let new_snapshot_id = utxo_storage.snaps.lastsnapid + 1;
    let mut snap_partition_clone: Vec<(usize, HashMap<KeyId, zkvm::zkos_types::Output>)> =
        Vec::new();

    let inner_snap_threadpool = ThreadPool::new(
        if utxo_storage.partition_size >= 5 {
            5
        } else {
            utxo_storage.partition_size + 1
        },
        String::from("inner_snap_threadpool"),
    );

    for i in 0..utxo_storage.partition_size {
        snap_partition_clone.push((i, utxo_storage.data.get(&i).unwrap().clone()));
    }
    for (path, data) in snap_partition_clone {
        inner_snap_threadpool.execute(move || {
            for (key, output) in data.iter() {
                let mut script_address: &String = &"".to_string();
                if path != 0 {
                    script_address = output.output.get_script_address().ok_or(UtxosetError::ScriptAddressNotFound)?;
                }

                let utxo_key: zkvm::zkos_types::Utxo = bincode::deserialize(key).unwrap();
                // let mut insert_utxo = Vec::new();

                let utxo_out: crate::pgsql::PGSQLDataInsert = crate::pgsql::PGSQLDataInsert::new(
                    key.clone(),
                    bincode::serialize(output).unwrap(),
                    bincode::serialize(output.output.get_owner_address().unwrap()).unwrap(),
                    script_address,
                    utxo_key.output_index() as usize,
                );

                // insert_utxo.push(utxo_out);
            //     let mut pgql_data = crate::pgsql::PGSQLTransaction::new(
            //         Vec::new(),
            //         insert_utxo,
            //         hex::encode(utxo_key.tx_id()),
            //         last_block as u64,
            //         path,
            //     );
            //    pgql_data.update_utxo_log();
            match path{
                0 =>{
                    insert_bulk_utxo_in_psql_coin(
                        vec![utxo_out],
                        hex::encode(utxo_key.tx_id()),
                        0u64,
                        "public.utxo_coin_logs",
                    );
                    
                }
                1 =>{
                    insert_bulk_utxo_in_psql_memo_or_state(
                        vec![utxo_out],
                        hex::encode(utxo_key.tx_id()),
                        0u64,
                        "public.utxo_memo_logs",
                    )
                }
                2 =>{insert_bulk_utxo_in_psql_memo_or_state(
                    vec![utxo_out],
                    hex::encode(utxo_key.tx_id()),
                    0u64,
                    "public.utxo_state_logs",
                )}
                _ =>{}
            }

            }
        });
    }
    Ok(())
}
