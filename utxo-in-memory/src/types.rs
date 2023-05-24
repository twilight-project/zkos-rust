use serde_derive::{Deserialize, Serialize};
pub type SequenceNumber = usize;
pub type UtxoKey = String;
pub type UtxoValue = String;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TxInputType {
    Coin = 0,  //uint8
    Memo = 1,  //uint8
    State = 2, //uint8
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UTXO {
    pub key: UtxoKey,
    pub value: UtxoValue,
    pub input_type: TxInputType,
}

impl UTXO {
    pub fn default() -> Self {
        UTXO {
            key: "".to_string(),
            value: "".to_string(),
            input_type: TxInputType::Coin,
        }
    }
    pub fn new(key: UtxoKey, value: UtxoValue, input_type: TxInputType) -> Self {
        UTXO {
            key,
            value,
            input_type,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ZkosBlock {
    pub add_utxo: Vec<UTXO>,
    pub remove_block: Vec<UTXO>,
    pub block_height: SequenceNumber,
}
impl ZkosBlock {
    pub fn default() -> Self {
        ZkosBlock {
            add_utxo: Vec::new(),
            remove_block: Vec::new(),
            block_height: 0,
        }
    }
    pub fn new(
        add_utxo_sets: Vec<UTXO>,
        remove_utxo_sets: Vec<UTXO>,
        block_height: SequenceNumber,
    ) -> Self {
        ZkosBlock {
            add_utxo: add_utxo_sets,
            remove_block: remove_utxo_sets,
            block_height: block_height,
        }
    }
}
#[derive(Debug)]
pub struct ZkosBlockResult {
    pub utxo_added: Vec<UTXO>,
    pub utxo_removed: Vec<UTXO>,
    pub error_vec: Vec<std::io::Error>,
}
impl ZkosBlockResult {
    pub fn new() -> Self {
        ZkosBlockResult {
            utxo_added: Vec::new(),
            utxo_removed: Vec::new(),
            error_vec: Vec::new(),
        }
    }
}

pub type ZkBlock = ZkosBlock;
pub type ZkBlockResult = ZkosBlockResult;

#[cfg(test)]
mod test {
    // use super::*;
    use crate::*;
    use std::fs;
    pub fn init_utxo_for_test() {
        let mut utxo_storage = temp_env::with_var(
            "SNAPSHOT_FILE_LOCATION",
            Some("./snapshot_storage/test/map"),
            || UTXO_STORAGE.lock().unwrap(),
        );
        utxo_storage.load_from_snapshot();
    }
    pub fn uninstall_delete_utxo_for_test() {
        temp_env::with_var(
            "SNAPSHOT_FILE_LOCATION",
            Some("./snapshot_storage/test/map"),
            || {
                let path = std::env::var("SNAPSHOT_FILE_LOCATION")
                    .expect("missing environment variable SNAPSHOT_FILE_LOCATION");
                let _ = fs::remove_dir_all(path);
                // println!("{}", path);
            },
        );
    }

    // cargo test -- --nocapture --test create_mk_snapshot_test --test-threads 5
    #[test]
    fn create_mk_snapshot_test() {
        init_utxo_for_test();
        let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
        println!("db: {:#?}", utxo_storage);
        uninstall_delete_utxo_for_test();
    }
}
