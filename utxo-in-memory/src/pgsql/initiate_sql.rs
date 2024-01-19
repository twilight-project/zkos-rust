use crate::{db::KeyId, ThreadPool};
use r2d2_postgres::postgres::NoTls;
use r2d2_postgres::PostgresConnectionManager;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use zkvm::Output;
lazy_static! {
    pub static ref POSTGRESQL_POOL_CONNECTION: r2d2::Pool<PostgresConnectionManager<NoTls>> = {
        dotenv::from_filename("/Users/ahmadashraf/work/twilight/ZkOS/transactionapi/.env").expect("Failed loading dotenv");
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

// pub fn load_backup_from_psql()->Vec<(usize,Vec<(KeyId,zkvm::Output)>)> {
//     let mut utxo_storage = crate::UTXO_STORAGE.lock().unwrap();

//     let snapshot_path = utxo_storage.snaps.snap_rules.path.clone();
//     // let snap_path = format!("{}-snapmap", snapshot_path.clone());
//     let last_block = utxo_storage.block_height.clone();
//     // let new_snapshot_id = utxo_storage.snaps.lastsnapid + 1;
//     let mut snap_partition_clone: Vec<(usize, HashMap<KeyId, zkvm::zkos_types::Output>)> =
//         Vec::new();

//     let inner_snap_threadpool = ThreadPool::new(
//         if utxo_storage.partition_size >= 5 {
//             5
//         } else {
//             utxo_storage.partition_size + 1
//         },
//         String::from("inner_snap_threadpool"),
//     );

//     let (sender, receiver) = mpsc::channel();
//     inner_snap_threadpool.execute(move || {
//         let mut query:String="".to_string();
//        let io_type=IOType::Coin;
//         match io_type{
//             IOType::Coin=>{
//                 if end_block < 0 {
//                 query = format!("SELECT  output, block_height FROM public.utxo_coin_logs where block_height >= {} order by block_height asc OFFSET {} limit {};",0,10000*1,10000);

//                  println!("{}",query);
//            }

//             IOType::Memo=>{   if end_block < 0 {
//                 query = format!("SELECT  output, block_height FROM public.utxo_memo_logs where block_height >= {} order by block_height asc OFFSET {} limit {};",0,10000*1,10000);
//                println!("{}",query);
//            }

//             IOType::State=>{   if end_block < 0 {
//                 query = format!("SELECT  output, block_height FROM public.utxo_state_logs where block_height >= {} order by block_height asc OFFSET {} limit {};",0,10000*1,10000);
//                println!("{}",query);
//            }

//    }
//         }

//         let mut client = POSTGRESQL_POOL_CONNECTION.get().unwrap();
//         let mut result: Vec<(KeyId, zkvm::zkos_types::Output)> = Vec::new();
//         match client.query(&query, &[]) {
//             Ok(data) => {
//                 for row in data {
//                     result.push((row.get("utxo"),bincode::deserialize(row.get("output")).unwrap()));
//                 }
//                 sender.send(Ok(result)).unwrap();
//             }
//             Err(arg) => sender
//                 .send(Err(std::io::Error::new(std::io::ErrorKind::Other, arg)))
//                 .unwrap(),
//         }

//     });

//     drop(inner_snap_threadpool);

//     match receiver.recv().unwrap() {
//         Ok(value) => {
//             return Ok(UtxoHexDecodeResult { result: value });
//         }
//         Err(arg) => {
//             return Err(std::io::Error::new(std::io::ErrorKind::Other, arg));
//         }
//     };
// }
// // ------------------------------------------------------------------------
// // Tests
// // ------------------------------------------------------------------------
// #[cfg(test)]
// mod test {
//     use super::*;
//     // cargo test -- --nocapture --test create_psql_table_test --test-threads 1
//     #[test]
//     fn create_psql_table_test() {
//         init_psql();
//     }
// }
