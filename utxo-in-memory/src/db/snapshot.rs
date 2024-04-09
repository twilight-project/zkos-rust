use super::snap_rules::SnapRules;
use crate::db::SequenceNumber;
use rusty_leveldb::{CompressionType, Options, DB};
use serde_derive::{Deserialize, Serialize};
use crate::error::UtxosetError;
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SnapShot {
    // pub map: HashMap<u64, SequenceNumber>,
    pub currentsnapid: SequenceNumber,
    pub lastsnapid: SequenceNumber,
    pub lastsnaptimestamp: u128,
    pub block_height: SequenceNumber,
    pub aggrigate_log_sequence: SequenceNumber,
    pub snap_rules: SnapRules,
    pub partition_size: usize,
}
impl SnapShot {
    pub fn new(partition_size: usize) -> SnapShot {
        let snap_rules = SnapRules::env().expect("Failed to load snap rules");
        // Do we need to Panic here or what is the best next step?
        let backup_key = match bincode::serialize(&String::from("utxosnapshot")){
            Ok(key) => key,
            Err(e) => panic!("Failed to serialize key: {}", e),
        };
        let snapshot_backup = leveldb_get_snapshot_metadata(
            format!("{}-snapmap", snap_rules.path),
            &backup_key,
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
               snap_rules,
                 partition_size,
            },
        }
    }

    pub fn load(partition_size: usize, path: &str) -> SnapShot {
        let mut snap_rules = SnapRules::env().expect("Failed to load snap rules");
        snap_rules.path = path.to_string();
         let backup_key = match bincode::serialize(&String::from("utxosnapshot")){
            Ok(key) => key,
            Err(e) => panic!("Failed to serialize key: {}", e),
        };
        let snapshot_backup = leveldb_get_snapshot_metadata(
            format!("{}-snapmap", snap_rules.path),
            &backup_key,
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
                snap_rules,
                partition_size,
            },
        }
    }
}

pub fn leveldb_custom_put(path: String, key: &[u8], value: &[u8]) -> Result<(), UtxosetError> {
    let mut opt = Options::default();
    opt.create_if_missing = true;
    opt.compression_type = CompressionType::CompressionSnappy;
    let mut db = DB::open(path, opt)?;
    db.put(key, value)?;
    Ok(db.flush()?)
}
pub fn leveldb_get_snapshot_metadata(path: String, key: &[u8]) -> Result<SnapShot, UtxosetError> {
    //utxosnapshot
    let mut opt = Options::default();
    opt.create_if_missing = true;
    opt.compression_type = CompressionType::CompressionSnappy;
    let mut db = DB::open(path, opt)?;
    match db.get(key){
        Some(value) => Ok(bincode::deserialize(&value)?),
        None => Err(UtxosetError::SnapshotNotFound),
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

pub fn leveldb_get_utxo_hashmap1(path: String, key: &[u8]) -> Result<Vec<u8>, UtxosetError> {
    let mut opt = Options::default();
    opt.create_if_missing = true;
    opt.compression_type = CompressionType::CompressionSnappy;
    let mut db = DB::open(path, opt)?;
    db.get(key).ok_or(UtxosetError::SnapshotNotFound)
}
