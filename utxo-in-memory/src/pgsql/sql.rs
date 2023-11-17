use crate::db::KeyId;
use crate::pgsql::{POSTGRESQL_POOL_CONNECTION, THREADPOOL_SQL_QUEUE};
use crate::ThreadPool;
use r2d2_postgres::postgres::types::ToSql;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PGSQLTransaction {
    pub remove_utxo: Vec<KeyId>,
    pub insert_coin_utxo: Vec<PGSQLDataInsert>,
    pub insert_memo_utxo: Vec<PGSQLDataInsert>,
    pub insert_state_utxo: Vec<PGSQLDataInsert>,
    pub txid: String,
    pub block_height: u64,
}

impl PGSQLTransaction {
    pub fn new(
        remove_utxo: Vec<KeyId>,
        insert_coin_utxo: Vec<PGSQLDataInsert>,
        insert_memo_utxo: Vec<PGSQLDataInsert>,
        insert_state_utxo: Vec<PGSQLDataInsert>,
        txid: String,
        block_height: u64,
    ) -> Self {
        PGSQLTransaction {
            remove_utxo,
            insert_coin_utxo,
            insert_memo_utxo,
            insert_state_utxo,
            txid,
            block_height,
        }
    }
    pub fn default() -> Self {
        PGSQLTransaction {
            remove_utxo: Vec::new(),
            insert_coin_utxo: Vec::new(),
            insert_memo_utxo: Vec::new(),
            insert_state_utxo: Vec::new(),
            txid: " ".to_string(),
            block_height: 0,
        }
    }

    pub fn update_utxo_log(&mut self) -> bool {
        let remove_utxo = self.remove_utxo.clone();
        let insert_coin_utxo = self.insert_coin_utxo.clone();
        let insert_memo_utxo = self.insert_memo_utxo.clone();
        let insert_state_utxo = self.insert_state_utxo.clone();

        let coin_table_name = "public.utxo_coin_logs";
        let memo_table_name = "public.utxo_memo_logs";
        let state_table_name = "public.utxo_state_logs";

        //remove utxo from psql
        if remove_utxo.len() > 0 {
            remove_bulk_utxo_in_psql(remove_utxo.clone(), coin_table_name);
            remove_bulk_utxo_in_psql(remove_utxo.clone(), memo_table_name);
            remove_bulk_utxo_in_psql(remove_utxo.clone(), state_table_name);
        }

        if insert_coin_utxo.len() > 0 {
            insert_bulk_utxo_in_psql_coin(
                insert_coin_utxo,
                self.txid.clone(),
                self.block_height,
                coin_table_name,
            );
        }
        if insert_memo_utxo.len() > 0 {
            insert_bulk_utxo_in_psql_memo_or_state(
                insert_memo_utxo,
                self.txid.clone(),
                self.block_height,
                memo_table_name,
            );
        }
        if insert_state_utxo.len() > 0 {
            insert_bulk_utxo_in_psql_memo_or_state(
                insert_state_utxo,
                self.txid.clone(),
                self.block_height,
                state_table_name,
            );
        }
        // match self.io_type {
        //     0 => {
        //         let table_name = "public.utxo_coin_logs";
        //         if remove_utxo.len() > 0 {
        //             remove_bulk_utxo_in_psql(remove_utxo, table_name);
        //         }
        //         if insert_utxo.len() > 0 {
        //             insert_bulk_utxo_in_psql_coin(
        //                 insert_utxo,
        //                 self.txid.clone(),
        //                 self.block_height,
        //                 table_name,
        //             );
        //         }
        //     }
        //     1 => {
        //         let table_name = "public.utxo_memo_logs";
        //         if remove_utxo.len() > 0 {
        //             remove_bulk_utxo_in_psql(remove_utxo, table_name);
        //         }
        //         if insert_utxo.len() > 0 {
        //             insert_bulk_utxo_in_psql_memo_or_state(
        //                 insert_utxo,
        //                 self.txid.clone(),
        //                 self.block_height,
        //                 table_name,
        //             );
        //         }
        //     }
        //     2 => {
        //         let table_name = "public.utxo_state_logs";
        //         if remove_utxo.len() > 0 {
        //             remove_bulk_utxo_in_psql(remove_utxo, table_name);
        //         }
        //         if insert_utxo.len() > 0 {
        //             insert_bulk_utxo_in_psql_memo_or_state(
        //                 insert_utxo,
        //                 self.txid.clone(),
        //                 self.block_height,
        //                 table_name,
        //             );
        //         }
        //     }
        //     _ => {}
        // }

        true
    }
}

pub fn insert_bulk_utxo_in_psql_coin(
    mut insert_utxo: Vec<PGSQLDataInsert>,
    tx_id: String,
    block_height: u64,
    table_name: &str,
) {
    let mut bulk_query_insert: String = format!(
        "INSERT INTO {}(utxo, output, owner_address, txid, vout, block_height) VALUES",
        table_name
    );
    let mut index_count = 0;
    let mut params_vec: Vec<&(dyn ToSql + Sync)> = Vec::new();

    for raw_utxo in insert_utxo.iter_mut() {
        if index_count != 0 {
            bulk_query_insert = format!("{},", bulk_query_insert);
        }
        bulk_query_insert = format!(
            "{} (${}, ${}, ${}, '{}', {}, {})",
            bulk_query_insert,
            index_count + 1,
            index_count + 2,
            index_count + 3,
            tx_id.clone(),
            raw_utxo.vout,
            block_height
        );

        index_count += 3;
        params_vec.push(&raw_utxo.key);
        params_vec.push(&raw_utxo.data);
        params_vec.push(&raw_utxo.owner_address);
    }
    let mut client = POSTGRESQL_POOL_CONNECTION.get().unwrap();
    client.execute(&bulk_query_insert, &params_vec).unwrap();
}

pub fn insert_bulk_utxo_in_psql_memo_or_state(
    mut insert_utxo: Vec<PGSQLDataInsert>,
    tx_id: String,
    block_height: u64,
    table_name: &str,
) {
    let mut bulk_query_insert: String = format!(
        "INSERT INTO {}(utxo, output, owner_address, script_address, txid, vout, block_height) VALUES",
        table_name
    );
    let mut index_count = 0;
    let mut params_vec: Vec<&(dyn ToSql + Sync)> = Vec::new();

    for raw_utxo in insert_utxo.iter_mut() {
        if index_count != 0 {
            bulk_query_insert = format!("{},", bulk_query_insert);
        }
        bulk_query_insert = format!(
            "{} (${}, ${}, ${},'{}', '{}', {}, {})",
            bulk_query_insert,
            index_count + 1,
            index_count + 2,
            index_count + 3,
            raw_utxo.script_address,
            tx_id.clone(),
            raw_utxo.vout,
            block_height
        );

        index_count += 3;
        params_vec.push(&raw_utxo.key);
        params_vec.push(&raw_utxo.data);
        params_vec.push(&raw_utxo.owner_address);
    }
    let mut client = POSTGRESQL_POOL_CONNECTION.get().unwrap();
    client.execute(&bulk_query_insert, &params_vec).unwrap();
}

pub fn remove_bulk_utxo_in_psql(remove_utxo: Vec<KeyId>, table_name: &str) {
    let mut bulk_query_remove: String = format!("DELETE FROM {} WHERE utxo = any($1);", table_name);
    let mut client = POSTGRESQL_POOL_CONNECTION.get().unwrap();
    client.execute(&bulk_query_remove, &[&remove_utxo]).unwrap();
}

// ------------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------------
#[cfg(test)]
mod test {
    use super::*;
    use crate::pgsql::{deserialize_tx_id, deserialize_tx_string, tx_id_string};
    // use std::fs::File;
    // use std::io::prelude::*;
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
        // pg_insert_data.io_type = 0;
        // for input in tx_input {
        //     let utxo_key = bincode::serialize(&input.as_utxo().unwrap()).unwrap();
        //     let utxo_input_type = input.in_type as usize;
        //     let utxo_test = Utxo::new(TxID(Hash([0; 32])), 0);
        //     let utxo = input.as_utxo().unwrap();
        //     if utxo.to_owned() != utxo_test {
        //         pg_insert_data.remove_utxo.push(utxo_key.clone());
        //     }
        // }
        // for (output_index, output_set) in tx_output.iter().enumerate() {
        //     let utxo_key = bincode::serialize(&Utxo::from_hash(
        //         Hash(deserialize_tx_id()),
        //         output_index as u8,
        //     ))
        //     .unwrap();
        //     let utxo_output_type = output_set.out_type as usize;

        //     match utxo_output_type {
        //         0 => {
        //             pg_insert_data.insert_utxo.push(PGSQLDataInsert::new(
        //                 utxo_key,
        //                 bincode::serialize(&output_set).unwrap(),
        //                 bincode::serialize(output_set.output.get_owner_address().unwrap()).unwrap(),
        //                 &"".to_string(),
        //                 output_index,
        //             ));
        //         }
        //         1 => {
        //             pg_insert_data.insert_utxo.push(PGSQLDataInsert::new(
        //                 utxo_key,
        //                 bincode::serialize(&output_set).unwrap(),
        //                 bincode::serialize(output_set.output.get_owner_address().unwrap()).unwrap(),
        //                 output_set.output.get_script_address().unwrap(),
        //                 output_index,
        //             ));
        //         }
        //         2 => {
        //             pg_insert_data.insert_utxo.push(PGSQLDataInsert::new(
        //                 utxo_key,
        //                 bincode::serialize(&output_set).unwrap(),
        //                 bincode::serialize(output_set.output.get_owner_address().unwrap()).unwrap(),
        //                 output_set.output.get_script_address().unwrap(),
        //                 output_index,
        //             ));
        //         }
        //         _ => {}
        //     }
        // }

        // pg_insert_data.update_utxo_log();
    }
}

// use std::fs::File;
// use std::io::prelude::*;
// let mut file1 = File::create("pg_insert_data.txt").unwrap();
// file1.write_all(&serde_json::to_vec(&pg_insert_data.clone()).unwrap()).unwrap();

// let mut remove_utxo_new: Vec<Vec<u8>> = Vec::new();
// for datautxo in insert_utxo.clone() {
//     remove_utxo_new.push(datautxo.key);
// }
// remove_bulk_utxo_in_psql(remove_utxo_new);
