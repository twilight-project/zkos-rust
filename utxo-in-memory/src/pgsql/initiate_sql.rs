use crate::{error::UtxosetError, ThreadPool};
use r2d2_postgres::postgres::NoTls;
use r2d2_postgres::PostgresConnectionManager;
use std::sync::Mutex;

lazy_static! {
    pub static ref POSTGRESQL_POOL_CONNECTION: r2d2::Pool<PostgresConnectionManager<NoTls>> = {
        dotenv::dotenv().expect("Failed loading dotenv");
        let postgresql_url =
            std::env::var("POSTGRESQL_URL").expect("missing environment variable POSTGRESQL_URL");
        
        let manager = PostgresConnectionManager::new(postgresql_url.parse().expect("Can not parse the POSTGRES_URL credentials to create a database connection"), NoTls);
        match r2d2::Pool::new(manager){
            Ok(pool) => pool,
            Err(e) => panic!("Error creating r2d2 pool: {}", e)
        }
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

fn create_utxo_coin_table() -> Result<(), UtxosetError> {
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

    let mut client = POSTGRESQL_POOL_CONNECTION.get()?;

    client.execute(&query, &[])?;
    Ok(())
}
fn create_utxo_memo_table() -> Result<(), UtxosetError> {
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

    let mut client = POSTGRESQL_POOL_CONNECTION.get()?;

    client.execute(&query, &[])?;
    Ok(())
}
fn create_utxo_state_table() -> Result<(), UtxosetError> {
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

    let mut client = POSTGRESQL_POOL_CONNECTION.get()?;

    client.execute(&query, &[])?;
    Ok(())

}


// // ------------------------------------------------------------------------
// // Tests
// // ------------------------------------------------------------------------
#[cfg(test)]
mod test {
    use super::*;
    // cargo test -- --nocapture --test create_psql_table_test --test-threads 1
    #[test]
    fn create_psql_table_test() {
        init_psql();
    }
}
