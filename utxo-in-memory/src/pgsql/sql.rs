use super::THREADPOOL_SQL_QUEUE;
use crate::db::KeyId;
use crate::ThreadPool;
use serde::{Deserialize, Serialize};
use zkvm::zkos_types::Output;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PGSQLTransaction {
    pub remove_utxo: Vec<KeyId>,
    pub insert_utxo: Vec<PGSQLDataInsert>,
    pub txid: String,
    pub block_height: u64,
    pub io_type: i64,
}

impl PGSQLTransaction {
    pub fn new(
        remove_utxo: Vec<KeyId>,
        insert_utxo: Vec<PGSQLDataInsert>,
        txid: String,
        block_height: u64,
        io_type: i64,
    ) -> Self {
        PGSQLTransaction {
            remove_utxo,
            insert_utxo,
            txid,
            block_height,
            io_type,
        }
    }
    pub fn default() -> Self {
        PGSQLTransaction {
            remove_utxo: Vec::new(),
            insert_utxo: Vec::new(),
            txid: " ".to_string(),
            block_height: 0,
            io_type: 0,
        }
    }

    pub fn update_utxo_log(&mut self) -> bool {
        let mut remove_utxo = self.remove_utxo.clone();
        let mut insert_utxo = self.insert_utxo.clone();
        let mut bulk_query_remove: String = "".to_string();
        for key_id in remove_utxo {
            bulk_query_remove = format!(
                "{}\n DELETE FROM public.utxo_coin_logs WHERE utxo ={:?};",
                bulk_query_remove, key_id
            );
        }
        let mut file1 = File::create("bulk_query_remove.txt").unwrap();
        file1
            .write_all(&serde_json::to_vec(&bulk_query_remove.clone()).unwrap())
            .unwrap();
        let mut bulk_query_insert: String = "".to_string();
        // for raw_utxo in insert_utxo.clone() {
        //     bulk_query_insert = format!(
        //         "{}\n INSERT INTO public.utxo_coin_logs( utxo, output, owner_address, script_address, txid, vout, block_height) VALUES ({:?}, {:?}, {:?}, {}, {}, {:?}, {:?});",
        //         bulk_query_insert,
        //         raw_utxo.key,
        //         raw_utxo.data,
        //         raw_utxo.owner_address,
        //         raw_utxo.script_address,
        //         self.txid,
        //         raw_utxo.vout,
        //         self.block_height
        //     );
        // }
        let mut index_count = 0;
        for raw_utxo in insert_utxo {
            let mut params_vec: Vec<&[u8]> = Vec::new();
            bulk_query_insert = "".to_string();
            bulk_query_insert = format!(
                "{} INSERT INTO public.utxo_coin_logs( utxo, output, owner_address, txid, vout, block_height) VALUES (${}, ${}, ${}, '{}', {}, {});",
                bulk_query_insert,
                index_count+1,
                index_count+2,
                index_count+3,
                self.txid.clone(),
                raw_utxo.vout,
                self.block_height
            );
            let mut file2 = File::create("bulk_query_insert.txt").unwrap();
            file2
                .write_all(&serde_json::to_vec(&bulk_query_insert.clone()).unwrap())
                .unwrap();
            // index_count += 4;
            crate::pgsql::psql_utxo_logs(
                bulk_query_insert.clone(),
                raw_utxo.key,
                raw_utxo.data,
                raw_utxo.owner_address,
            );
        }

        true
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PGSQLDataInsert {
    pub key: KeyId,
    pub data: Vec<u8>,
    pub owner_address: Vec<u8>,
    pub script_address: String,
    pub vout: usize,
}

impl PGSQLDataInsert {
    pub fn new(
        key: KeyId,
        data: Vec<u8>,
        owner_address: Vec<u8>,
        script_address: &String,
        vout: usize,
    ) -> Self {
        PGSQLDataInsert {
            key: key,
            data: data.clone(),
            owner_address: owner_address,
            script_address: script_address.clone(),
            vout: vout,
        }
    }
}

pub trait PGSQLDBtrait {
    fn add_into_sqldb(&mut self, input_type: usize);
    fn remove_from_sqldb(&mut self);
}

impl PGSQLDBtrait for PGSQLDataInsert {
    fn add_into_sqldb(&mut self, input_type: usize) {
        let sql_queue = THREADPOOL_SQL_QUEUE.lock().unwrap();
        sql_queue.execute(move || {});
        drop(sql_queue);
    }
    fn remove_from_sqldb(&mut self) {
        let sql_queue = THREADPOOL_SQL_QUEUE.lock().unwrap();
        sql_queue.execute(move || {});
        drop(sql_queue);
    }
}

// ------------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------------
#[cfg(test)]
mod test {
    use super::*;
    use crate::pgsql::{deserialize_tx_id, deserialize_tx_string, tx_id_string};
    use std::fs::File;
    use std::io::prelude::*;
    use zkvm::tx::TxID;

    use zkvm::zkos_types::{
        IOType, Input, Output, OutputCoin, OutputData, OutputMemo, OutputState, Utxo,
    };
    use zkvm::Hash;
    // cargo test -- --nocapture --test insert_utxo_in_psql_test --test-threads 1
    #[test]
    fn insert_utxo_in_psql_test() {}
    #[test]
    fn remove_utxo_in_psql_test() {}
    #[test]
    fn print_test_tx_test() {
        let test1 = deserialize_tx_string();
        println!("transaction - {:#?}", test1);

        let test2 = deserialize_tx_id();
        println!("transaction_id - {:#?}", test2);
    }
    #[test]
    fn tx_to_psqldata_test() {
        let mut pg_insert_data = PGSQLTransaction::default();
        pg_insert_data.txid = tx_id_string();
        // pg_insert_data.txid = hex::decode(tx_id_string().clone()).unwrap();
        pg_insert_data.block_height = 5;
        let test_transaction = deserialize_tx_string();
        let tx_input = test_transaction.get_tx_inputs();
        let tx_output = test_transaction.get_tx_outputs();

        for input in tx_input {
            let utxo_key = bincode::serialize(&input.as_utxo().unwrap()).unwrap();
            let utxo_input_type = input.in_type as usize;
            let utxo_test = Utxo::new(TxID(Hash([0; 32])), 0);
            let utxo = input.as_utxo().unwrap();
            if utxo.to_owned() != utxo_test {
                pg_insert_data.remove_utxo.push(utxo_key.clone());
            }
        }
        for (output_index, output_set) in tx_output.iter().enumerate() {
            let utxo_key = bincode::serialize(&Utxo::from_hash(
                Hash(deserialize_tx_id()),
                output_index as u8,
            ))
            .unwrap();
            let utxo_output_type = output_set.out_type as usize;

            match utxo_output_type {
                0 => {
                    pg_insert_data.insert_utxo.push(PGSQLDataInsert::new(
                        utxo_key,
                        bincode::serialize(&output_set).unwrap(),
                        bincode::serialize(output_set.output.get_owner_address().unwrap()).unwrap(),
                        &"".to_string(),
                        output_index,
                    ));
                }
                1 => {
                    pg_insert_data.insert_utxo.push(PGSQLDataInsert::new(
                        utxo_key,
                        bincode::serialize(&output_set).unwrap(),
                        bincode::serialize(output_set.output.get_owner_address().unwrap()).unwrap(),
                        output_set.output.get_script_address().unwrap(),
                        output_index,
                    ));
                }
                2 => {
                    pg_insert_data.insert_utxo.push(PGSQLDataInsert::new(
                        utxo_key,
                        bincode::serialize(&output_set).unwrap(),
                        bincode::serialize(output_set.output.get_owner_address().unwrap()).unwrap(),
                        output_set.output.get_script_address().unwrap(),
                        output_index,
                    ));
                }
                _ => {}
            }
        }

        pg_insert_data.update_utxo_log();

        let mut file1 = File::create("pg_insert_data.txt").unwrap();
        file1
            .write_all(&serde_json::to_vec(&pg_insert_data.clone()).unwrap())
            .unwrap();
    }
}

// utxo BYTEA PRIMARY KEY,
// output BYTEA,
// owner_address BYTEA,  owner address coin -only one, state memo 2 addess owner
// let addr = output_data.output.get_owner_address().unwrap(); 324
// script_address VARCHAR(42),empty in coin , state memo 2 addess owner
// get_script_address
// txid CHAR(64), //   #[serde(rename = "TxId")]   pub tx_id: String,

// vout BIGINT,
// block_height BIGINT,
// io_type VARCHAR(10) CHECK (io_type IN ('coin', 'memo', 'state'))

// pub fn psql_utxo_logs(data: Output) {
//     //creating static connection
//     let mut client = POSTGRESQL_POOL_CONNECTION.get().unwrap();

//     let query = format!(
//         "INSERT INTO public.utxo_logs(utxo,output key, payload) VALUES ({},'{}',$1);",
//         data.offset, data.key
//     );
// INSERT INTO public.utxo_coin_logs(
// 	utxo, output, owner_address, script_address, txid, vout, block_height)
// 	VALUES (?, ?, ?, ?, ?, ?, ?);

//     client.execute(&query, &[&data.value]).unwrap();
// }

// PGSQLData::new(id, value.clone()).add_into_sqldb(input_type);
// PGSQLData::new(id, value.clone()).remove_from_sqldb();
use std::fs::File;
use std::io::prelude::*;
