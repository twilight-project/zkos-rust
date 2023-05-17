use rusty_leveldb::{CompressionType, DBIterator, LdbIterator, Options, DB};
use std::collections::{HashMap, HashSet};
use std::println;
use utxolib::db;
// use utxolib::db::init_utxo;
use utxolib::db::SNAPSHOT_DB;
#[macro_use]
extern crate lazy_static;
fn main() {
    // init utxo
    // init_utxo();
    // db::leveldbrep::leveldbtest();
    // let mut db: HashMap<i64, String> = HashMap::new();
    // let x = db.insert(1, String::from("value 1"));
    // let x = db.try_insert(1, String::from("value 2"));
    // println!("db: {:#?}", db);
    let mut snapshot_db = SNAPSHOT_DB.lock().unwrap();
    // let mut snap_db = db::sn_db();
    // snap_db.put(b"Hello", b"World").unwrap();
    snapshot_db
        .put(
            String::from("key3").as_bytes(),
            &String::from("value 3").as_bytes(),
        )
        .unwrap();
    snapshot_db.flush().unwrap();
    // let mut iter = snapshot_db.new_iter().unwrap();
    // println!("data check : {:#?}", iter.next().unwrap());
    // println!("data check : {:#?}", iter.next().unwrap());
    // thread::sleep(Duration::from_secs(5));
}
use std::thread;
use std::time::Duration;
