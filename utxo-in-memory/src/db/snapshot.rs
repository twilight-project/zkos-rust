//! Snapshot management for UTXO storage using LevelDB.
//!
//! This module provides functionality for creating, storing, and retrieving
//! snapshots of the UTXO storage state using LevelDB as the backing store.
//! Snapshots include metadata about the storage state and can be used for
//! recovery and backup purposes.

use super::snap_rules::SnapRules;
use crate::db::SequenceNumber;
use rusty_leveldb::{CompressionType, Options, DB};
use serde_derive::{Deserialize, Serialize};

/// Snapshot metadata containing storage state information
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SnapShot {
    /// Current snapshot ID
    pub currentsnapid: SequenceNumber,
    /// Last snapshot ID
    pub lastsnapid: SequenceNumber,
    /// Timestamp of last snapshot creation
    pub lastsnaptimestamp: u128,
    /// Block height at snapshot creation
    pub block_height: SequenceNumber,
    /// Aggregate log sequence number
    pub aggrigate_log_sequence: SequenceNumber,
    /// Snapshot configuration rules
    pub snap_rules: SnapRules,
    /// Number of storage partitions
    pub partition_size: usize,
}

impl SnapShot {
    /// Creates new snapshot with default configuration from environment
    pub fn new(partition_size: usize) -> SnapShot {
        let snap_rules = SnapRules::env();
        let snapshot_backup = leveldb_get_snapshot_metadata(
            format!("{}-snapmap", snap_rules.path),
            &bincode::serialize(&String::from("utxosnapshot")).unwrap(),
        );

        match snapshot_backup {
            Ok(snap) => {
                let mut snapshot = snap;
                snapshot.snap_rules = snap_rules;
                snapshot
            }
            Err(_) => SnapShot {
                currentsnapid: 0,
                lastsnapid: 0,
                lastsnaptimestamp: 0,
                block_height: 0,
                aggrigate_log_sequence: 0,
                snap_rules: snap_rules,
                partition_size: partition_size,
            },
        }
    }

    /// Loads snapshot from specified path
    pub fn load(partition_size: usize, path: &str) -> SnapShot {
        let mut snap_rules = SnapRules::env();
        snap_rules.path = path.to_string();
        let snapshot_backup = leveldb_get_snapshot_metadata(
            format!("{}-snapmap", snap_rules.path),
            &bincode::serialize(&String::from("utxosnapshot")).unwrap(),
        );

        match snapshot_backup {
            Ok(snap) => {
                let mut snapshot = snap;
                snapshot.snap_rules = snap_rules;
                snapshot.partition_size = partition_size;
                snapshot
            }
            Err(_) => SnapShot {
                currentsnapid: 0,
                lastsnapid: 0,
                lastsnaptimestamp: 0,
                block_height: 0,
                aggrigate_log_sequence: 0,
                snap_rules: snap_rules,
                partition_size: partition_size,
            },
        }
    }
}

/// Writes key-value pair to LevelDB with Snappy compression
pub fn leveldb_custom_put(path: String, key: &[u8], value: &[u8]) -> Result<(), std::io::Error> {
    let mut opt = Options::default();
    opt.create_if_missing = true;
    opt.compression_type = CompressionType::CompressionSnappy;
    let mut db = DB::open(path, opt).unwrap();
    db.put(key, value).unwrap();
    db.flush().unwrap();
    Ok(())
}

/// Retrieves snapshot metadata from LevelDB
pub fn leveldb_get_snapshot_metadata(path: String, key: &[u8]) -> Result<SnapShot, std::io::Error> {
    //utxosnapshot
    let mut opt = Options::default();
    opt.create_if_missing = true;
    opt.compression_type = CompressionType::CompressionSnappy;
    let mut db = DB::open(path, opt).unwrap();
    match db.get(key) {
        Some(value) => return Ok(bincode::deserialize(&value).unwrap()),
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("snapshot does not exist"),
            ))
        }
    }
}

// pub fn leveldb_get_utxo_hashmap(
//     path: String,
//     key: &[u8],
// ) -> Result<HashMap<UtxoKey, UtxoValue>, std::io::Error> {
//     let mut opt = Options::default();
//     opt.create_if_missing = true;
//     opt.compression_type = CompressionType::CompressionSnappy;
//     let mut db = DB::open(path, opt).unwrap();
//     match db.get(key) {
//         Some(value) => return Ok(bincode::deserialize(&value).unwrap()),
//         None => {
//             return Err(std::io::Error::new(
//                 std::io::ErrorKind::NotFound,
//                 format!("snapshot does not exist"),
//             ))
//         }
//     }
// }

/// Retrieves raw UTXO data from LevelDB snapshot
pub fn leveldb_get_utxo_hashmap1(path: String, key: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut opt = Options::default();
    opt.create_if_missing = true;
    opt.compression_type = CompressionType::CompressionSnappy;
    let mut db = DB::open(path, opt).unwrap();
    match db.get(key) {
        Some(value) => return Ok(value),
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("snapshot does not exist"),
            ))
        }
    }
}
