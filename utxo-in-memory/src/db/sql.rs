use crate::db::KeyId;
use crate::ThreadPool;
use std::sync::{Arc, Mutex};
use zkvm::zkos_types::Output;
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

pub struct PGSQLData<T> {
    key: KeyId,
    data: T,
}
impl<T> PGSQLData<T> {
    pub fn new(key: KeyId, data: T) -> Self {
        PGSQLData::<T> {
            key: key,
            data: data,
        }
    }
}

pub trait PGSQLDBtrait<T> {
    fn add_into_sqldb(&mut self, input_type: usize);
    fn remove_from_sqldb(&mut self);
}

impl<Output> PGSQLDBtrait<Output> for PGSQLData<Output>
where
    Output: Clone + serde::Serialize + serde::de::DeserializeOwned + Send + 'static,
{
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

use r2d2_postgres::postgres::NoTls;
use r2d2_postgres::PostgresConnectionManager;

pub fn init_psql() {
    match create_utxo_logs_table() {
        Ok(_) => println!("utxo_logs table inserted successfully"),
        Err(arg) => println!("Some Error 11 Found, {:#?}", arg),
    }
}

fn create_utxo_logs_table() -> Result<(), r2d2_postgres::postgres::Error> {
    let query = format!(
        "CREATE TABLE IF NOT EXISTS public.utxo_logs (
            utxo BYTEA PRIMARY KEY,
            output BYTEA,
            address BYTEA,
            script_address VARCHAR(42),
            txid CHAR(64),
            vout BIGINT,
            block_height BIGINT,
            type VARCHAR(10) CHECK (type IN ('coin', 'memo', 'state'))        
          );"
    );
    let mut client = POSTGRESQL_POOL_CONNECTION.get().unwrap();

    match client.execute(&query, &[]) {
        Ok(_) => Ok(()),
        Err(arg) => Err(arg),
    }
}

// pub fn psql_utxo_logs(data: Output) {
//     //creating static connection
//     let mut client = POSTGRESQL_POOL_CONNECTION.get().unwrap();

//     let query = format!(
//         "INSERT INTO public.utxo_logs(utxo,output key, payload) VALUES ({},'{}',$1);",
//         data.offset, data.key
//     );
//     client.execute(&query, &[&data.value]).unwrap();
// }
