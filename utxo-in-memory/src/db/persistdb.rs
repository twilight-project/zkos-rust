use super::snap_rules::SnapRules;
use crate::db::{SequenceNumber, UTXOStorage};
use rusty_leveldb::{CompressionType, DBIterator, LdbIterator, Options, DB};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
// use std::sync::{mpsc, Arc, Mutex, RwLock};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapShot {
    // pub map: HashMap<u64, SequenceNumber>,
    pub currentsnapid: u64,
    pub lastsnapid: u64,
    pub lastsnaptimestamp: u128,
    pub block_height: SequenceNumber,
    pub aggrigate_log_sequence: SequenceNumber,
    pub snap_rules: SnapRules,
}
impl SnapShot {
    pub fn new() -> SnapShot {
        SnapShot {
            currentsnapid: 0,
            lastsnapid: 0,
            lastsnaptimestamp: 0,
            block_height: 0,
            aggrigate_log_sequence: 0,
            snap_rules: SnapRules::env(),
        }
    }
}

pub fn leveldb_custom_put(path: String, key: &[u8], value: &[u8]) -> Result<(), std::io::Error> {
    let mut opt = Options::default();
    opt.compression_type = CompressionType::CompressionSnappy;
    let mut db = DB::open("./snapshot_storage/timestamp_Data", opt).unwrap();
    db.put(key, value).unwrap();
    db.flush().unwrap();
    Ok(())
}
