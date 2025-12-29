//! PostgreSQL initialization module for setting up database tables.
//!
//! This module initializes the PostgreSQL database connection pool and
//! creates the necessary tables for UTXO storage. It uses lazy_static to
//! ensure thread-safe initialization of the connection pool.

use crate::{db::KeyId, ThreadPool};
use lazy_static::lazy_static;
use r2d2_postgres::postgres::NoTls;
use r2d2_postgres::PostgresConnectionManager;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use zkvm::Output;
lazy_static! {
    pub static ref POSTGRESQL_POOL_CONNECTION: r2d2::Pool<PostgresConnectionManager<NoTls>> = {
        dotenv::dotenv().expect("Failed loading dotenv");
        let postgresql_url =
            std::env::var("POSTGRESQL_URL").expect("missing environment variable POSTGRESQL_URL");
        let manager = PostgresConnectionManager::new(postgresql_url.parse().unwrap(), NoTls);
        r2d2::Pool::new(manager).unwrap()
    };
    pub static ref THREADPOOL_SQL_QUEUE: Mutex<ThreadPool> =
        Mutex::new(ThreadPool::new(1, String::from("THREADPOOL_SQL_QUEUE")));
    pub static ref THREADPOOL_SQL_QUERY: Mutex<ThreadPool> =
        Mutex::new(ThreadPool::new(4, String::from("THREADPOOL_SQL_QUEUE")));
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
