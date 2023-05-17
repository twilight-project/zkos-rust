use std::collections::{HashMap, HashSet};
use utxolib::db;
use utxolib::db::init_utxo;
#[macro_use]
extern crate lazy_static;
fn main() {
    // init utxo
    init_utxo();
    // db::leveldbrep::leveldbtest();
    // let mut db: HashMap<i64, String> = HashMap::new();
    // let x = db.insert(1, String::from("value 1"));
    // let x = db.try_insert(1, String::from("value 2"));

    // println!("db: {:#?}", db);
}
