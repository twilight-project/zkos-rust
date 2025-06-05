use crate::{db::KeyId, ThreadPool};
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
    match create_transfer_tx_event_table() {
        Ok(_) => println!("transfer event table created successfully"),
        Err(arg) => println!("Some Error 101 Found, {:#?}", arg),
    match create_mint_burn_event_table() {
        Ok(_) => println!("mint burn event table created successfully"),
        Err(arg) => println!("Some Error 101 Found, {:#?}", arg),
    match create_burn_req_event_table() {
        Ok(_) => println!("burn req event table created successfully"),
        Err(arg) => println!("Some Error 101 Found, {:#?}", arg),
}

fn create_transfer_tx_event_table() -> Result<(), r2d2_postgres::postgres::Error> {
    let query = format!(
        "CREATE TABLE transfer_tx_events (
            tx_id TEXT PRIMARY KEY,
            tx_bytecode TEXT NOT NULL,
            tx_fee BIGINT NOT NULL, 
            eth_address TEXT NOT NULL,
            block_number BIGINT NOT NULL, 
            zkos_tx_id TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT NOW()
        );"
    );

    let mut client = POSTGRESQL_POOL_CONNECTION.get().unwrap();

    match client.execute(&query, &[]) {
        Ok(_) => Ok(()),
        Err(arg) => Err(arg),
    }
}
fn create_mint_burn_event_table() -> Result<(), r2d2_postgres::postgres::Error> {
    let query = format!(
        "CREATE TABLE mint_or_burn_events (
            mint_or_burn BOOLEAN NOT NULL,
            usdc_value BIGINT NOT NULL,
            qq_account TEXT NOT NULL,
            encrypt_scalar TEXT NOT NULL,
            eth_address TEXT NOT NULL,
            block_number BIGINT NOT NULL,
            tx_id TEXT PRIMARY KEY,
            created_at TIMESTAMP DEFAULT NOW()
        );"
    );

    let mut client = POSTGRESQL_POOL_CONNECTION.get().unwrap();

    match client.execute(&query, &[]) {
        Ok(_) => Ok(()),
        Err(arg) => Err(arg),
    }
}

fn create_burn_req_event_table() -> Result<(), r2d2_postgres::postgres::Error> {
    let query = format!(
        "CREATE TABLE mint_or_burn_events (
            mint_or_burn BOOLEAN NOT NULL,
            usdc_value BIGINT NOT NULL,
            qq_account TEXT NOT NULL,
            encrypt_scalar TEXT NOT NULL,
            eth_address TEXT NOT NULL,
            block_number BIGINT NOT NULL,
            tx_id TEXT PRIMARY KEY,
            created_at TIMESTAMP DEFAULT NOW()
        );"
    );

    let mut client = POSTGRESQL_POOL_CONNECTION.get().unwrap();

    match client.execute(&query, &[]) {
        Ok(_) => Ok(()),
        Err(arg) => Err(arg),
    }
}
