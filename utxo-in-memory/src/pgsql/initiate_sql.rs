#![feature(array_methods)]
use crate::ThreadPool;
use core::slice::SlicePattern;
use r2d2_postgres::postgres::NoTls;
use r2d2_postgres::PostgresConnectionManager;
use std::sync::{Arc, Mutex};

lazy_static! {
    pub static ref POSTGRESQL_POOL_CONNECTION: r2d2::Pool<PostgresConnectionManager<NoTls>> = {
        dotenv::dotenv().expect("Failed loading dotenv");
        let postgresql_url =
            std::env::var("POSTGRESQL_URL").expect("missing environment variable POSTGRESQL_URL");
        let manager = PostgresConnectionManager::new(postgresql_url.parse().unwrap(), NoTls);
        r2d2::Pool::new(manager).unwrap()
    };
    pub static ref THREADPOOL_SQL_QUEUE: Mutex<ThreadPool> =
        Mutex::new(ThreadPool::new(10, String::from("THREADPOOL_SQL_QUEUE")));
}
pub fn init_psql() {
    match create_utxo_coin_table() {
        Ok(_) => println!("utxo_coin_logs table inserted successfully"),
        Err(arg) => println!("Some Error 101 Found, {:#?}", arg),
    }
    match create_utxo_memo_table() {
        Ok(_) => println!("utxo_memo_logs table inserted successfully"),
        Err(arg) => println!("Some Error 105 Found, {:#?}", arg),
    }
    match create_utxo_state_table() {
        Ok(_) => println!("utxo_state_logs table inserted successfully"),
        Err(arg) => println!("Some Error 109 Found, {:#?}", arg),
    }
}

fn create_utxo_coin_table() -> Result<(), r2d2_postgres::postgres::Error> {
    let query = format!(
        "CREATE TABLE IF NOT EXISTS public.utxo_coin_logs (
            utxo BYTEA PRIMARY KEY,
            output BYTEA,
            owner_address BYTEA,
            txid CHAR(64),
            vout BIGINT,
            block_height BIGINT
          );"
    );

    let mut client = POSTGRESQL_POOL_CONNECTION.get().unwrap();

    match client.execute(&query, &[]) {
        Ok(_) => Ok(()),
        Err(arg) => Err(arg),
    }
}
fn create_utxo_memo_table() -> Result<(), r2d2_postgres::postgres::Error> {
    let query = format!(
        "CREATE TABLE IF NOT EXISTS public.utxo_memo_logs (
            utxo BYTEA PRIMARY KEY,
            output BYTEA,
            owner_address BYTEA,
            script_address VARCHAR(42),
            txid CHAR(64),
            vout BIGINT,
            block_height BIGINT       
          );"
    );

    let mut client = POSTGRESQL_POOL_CONNECTION.get().unwrap();

    match client.execute(&query, &[]) {
        Ok(_) => Ok(()),
        Err(arg) => Err(arg),
    }
}
fn create_utxo_state_table() -> Result<(), r2d2_postgres::postgres::Error> {
    let query = format!(
        "CREATE TABLE IF NOT EXISTS public.utxo_state_logs (
            utxo BYTEA PRIMARY KEY,
            output BYTEA,
            owner_address BYTEA,
            script_address VARCHAR(42),
            txid CHAR(64),
            vout BIGINT,
            block_height BIGINT       
          );"
    );

    let mut client = POSTGRESQL_POOL_CONNECTION.get().unwrap();

    match client.execute(&query, &[]) {
        Ok(_) => Ok(()),
        Err(arg) => Err(arg),
    }
}

// ------------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------------
#[cfg(test)]
mod test {
    use super::*;
    // cargo test -- --nocapture --test create_psql_table_test --test-threads 1
    #[test]
    fn create_psql_table_test() {
        init_psql();
    }
}

pub fn psql_utxo_logs(data: String, a: Vec<u8>, b: Vec<u8>, c: Vec<u8>) {
    //creating static connection
    let mut client = POSTGRESQL_POOL_CONNECTION.get().unwrap();

    // let query = format!(
    //     "INSERT INTO public.utxo_logs(utxo,output key, payload) VALUES ({},'{}',$1);",
    //     data.offset, data.key
    // );

    client.execute(&data, &[&a, &b, &c]).unwrap();
}

pub fn psql_utxo_logs1(data: String, params: Vec<Vec<u8>>) {
    //creating static connection
    let mut client = POSTGRESQL_POOL_CONNECTION.get().unwrap();

    // let query = format!(
    //     "INSERT INTO public.utxo_logs(utxo,output key, payload) VALUES ({},'{}',$1);",
    //     data.offset, data.key
    // );
    // let v = vec![1, 2, 3, 4, 5];
    // let boxed_slice: Box<[u8]> = params.into_boxed_slice();
    let arr: [Vec<u8>; 4] = params.try_into().unwrap();
    client
        .execute(&data, &arr.each_ref().map(|p| p as &(dyn ToSql + Sync)))
        .unwrap();
}
// use postgres_types::ToSql;
use r2d2_postgres::postgres::types::ToSql;
