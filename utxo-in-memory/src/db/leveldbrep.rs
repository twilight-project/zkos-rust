// extern crate tempdir;
// extern crate leveldb;

// use tempdir::TempDir;
// use leveldb::database::Database;
// use leveldb::kv::KV;
// use leveldb::options::{Options,WriteOptions,ReadOptions};

// let tempdir = TempDir::new("demo").unwrap();
// let path = tempdir.path();

// let mut options = Options::new();
// options.create_if_missing = true;
// let mut database = match Database::open(path, options) {
//     Ok(db) => { db },
//     Err(e) => { panic!("failed to open database: {:?}", e) }
// };

// let write_opts = WriteOptions::new();
// match database.put(write_opts, 1, &[1]) {
//     Ok(_) => { () },
//     Err(e) => { panic!("failed to write to database: {:?}", e) }
// };

// let read_opts = ReadOptions::new();
// let res = database.get(read_opts, 1);

// match res {
//   Ok(data) => {
//     assert!(data.is_some());
//     assert_eq!(data, Some(vec![1]));
//   }
//   Err(e) => { panic!("failed reading data: {:?}", e) }
// }

use rusty_leveldb::{CompressionType, DBIterator, LdbIterator, Options, DB};
use std::sync::{mpsc, Arc, Mutex, RwLock};
lazy_static! {
    // pub static ref UTXO_STORAGE: Arc<Mutex<UTXOStorage>> = Arc::new(Mutex::new(UTXOStorage::new()));


    pub static ref SNAPSHOT_DB:Arc<Mutex<DB>> =
    { let mut opt = Options::default();
        opt.compression_type = CompressionType::CompressionSnappy;
        Arc::new(Mutex::new( DB::open("./snapshot_storage/timestamp_Data", opt).unwrap()))
    };
}
pub fn leveldbtest() {
    let opt = rusty_leveldb::in_memory();
    let mut opt = Options::default();
    opt.compression_type = CompressionType::CompressionSnappy;
    let mut db = DB::open("./snapshot_storage/timestamp_Data", opt).unwrap();

    db.put(b"Hello", b"World").unwrap();
    assert_eq!(b"World", db.get(b"Hello").unwrap().as_slice());

    let mut iter = db.new_iter().unwrap();
    assert_eq!((b"Hello".to_vec(), b"World".to_vec()), iter.next().unwrap());

    // db.delete(b"Hello").unwrap();
    // db.flush().unwrap();
}

pub fn sn_db() -> DB {
    let mut opt = Options::default();
    opt.compression_type = CompressionType::CompressionSnappy;
    let mut db = DB::open("./snapshot_storage/timestamp_Data", opt).unwrap();
    db
}
