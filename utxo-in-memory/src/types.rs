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

// cargo test -- --nocapture --test-threads 1
// cargo test --test-threads 1
#[cfg(test)]
mod test {
    // use super::*;
    use crate::{db::UTXOStorage, *};
    use std::fs;
    pub fn init_utxo_for_test(test_path: &str) {
        let mut utxo_storage = temp_env::with_var(
            "SNAPSHOT_FILE_LOCATION",
            Some(format!("./snapshot_storage_test/{}/map", test_path)),
            || UTXO_STORAGE.lock().unwrap(),
        );
        let snapshot_load = SnapShot::load(&format!("./snapshot_storage_test/{}/map", test_path));
        // utxostore.snaps = snapshot_load;
        let utxostore = UTXOStorage {
            coin_storage: std::collections::HashMap::new(),
            memo_storage: std::collections::HashMap::new(),
            state_storage: std::collections::HashMap::new(),
            block_height: 0,
            aggrigate_log_sequence: 0,
            snaps: snapshot_load,
            pending_commands: Vec::new(),
        };

        *utxo_storage = utxostore;
        // utxo_storage.load_from_snapshot();
    }
    pub fn uninstall_delete_db_utxo_for_test(test_path: &str) {
        let static_files = format!(
            "{}{}",
            std::env::var("CARGO_MANIFEST_DIR")
                .expect("missing environment variable CARGO_MANIFEST_DIR"),
            "/snapshot_storage_test/"
        );
        // Removes a directory at this path, after removing all its contents. Use carefully!
        let _ = fs::remove_dir_all(static_files);
    }

    // cargo test -- --nocapture --test create_mkdir_snapshot_test --test-threads 5
    #[test]
    fn create_mkdir_snapshot_test() {
        let test_path = "test1";
        init_utxo_for_test(test_path);
        let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
        uninstall_delete_db_utxo_for_test(test_path);
        println!("db_create: {:#?}", utxo_storage);
    }

    // cargo test -- --nocapture --test add_utxoset_test --test-threads 5
    #[test]
    fn add_utxoset_test() {
        let test_path = "test2";
        init_utxo_for_test(test_path);
        let utxo_key: UtxoKey = String::from("utxo_key");
        let utxo_value: UtxoValue = String::from("utxo_value");
        let utxo_input_type = TxInputType::Coin;
        let utxo_set = UTXO::new(
            utxo_key.clone(),
            utxo_value.clone(),
            utxo_input_type.clone(),
        );

        let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
        let utxo = utxo_storage.add(utxo_key, utxo_value, utxo_input_type);
        assert_eq!(utxo.unwrap(), utxo_set);
        // println!("db: {:#?}", utxo_storage);
        uninstall_delete_db_utxo_for_test(test_path);
        println!("db_add: {:#?}", utxo_storage);
    }
    // cargo test -- --nocapture --test remove_utxoset_test --test-threads 5
    #[test]
    fn remove_utxoset_test() {
        let test_path = "test3";
        init_utxo_for_test(test_path);
        let utxo_key: UtxoKey = String::from("utxo_key");
        let utxo_value: UtxoValue = String::from("utxo_value");
        let utxo_input_type = TxInputType::Coin;
        let utxo_set = UTXO::new(
            utxo_key.clone(),
            utxo_value.clone(),
            utxo_input_type.clone(),
        );
        let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
        let utxo_added = utxo_storage.add(
            utxo_key.clone(),
            utxo_value.clone(),
            utxo_input_type.clone(),
        );
        assert_eq!(utxo_added.unwrap(), utxo_set);
        let utxo_removes = utxo_storage.remove(utxo_key, utxo_input_type);
        assert_eq!(utxo_removes.unwrap(), utxo_set);
        uninstall_delete_db_utxo_for_test(test_path);
        println!("db_remove: {:#?}", utxo_storage);
    }

    // cargo test -- --nocapture --test search_utxoset_test --test-threads 5
    #[test]
    fn search_utxoset_test() {
        let test_path = "test4";
        init_utxo_for_test(test_path);
        let utxo_key: UtxoKey = String::from("utxo_key");
        let utxo_value: UtxoValue = String::from("utxo_value");
        let utxo_input_type = TxInputType::Coin;
        let utxo_set = UTXO::new(
            utxo_key.clone(),
            utxo_value.clone(),
            utxo_input_type.clone(),
        );
        let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
        let utxo = utxo_storage.add(
            utxo_key.clone(),
            utxo_value.clone(),
            utxo_input_type.clone(),
        );
        assert_eq!(utxo.unwrap(), utxo_set);
        let search_bool = utxo_storage.search_key(utxo_key, utxo_input_type);
        assert_eq!(search_bool, true);
        uninstall_delete_db_utxo_for_test(test_path);
        println!("db_search: {:#?}", utxo_storage);
    }

    // cargo test -- --nocapture --test get_utxoset_test --test-threads 5
    #[test]
    fn get_utxoset_test() {
        let test_path = "test5";
        init_utxo_for_test(test_path);
        let utxo_key: UtxoKey = String::from("utxo_key");
        let utxo_value: UtxoValue = String::from("utxo_value");
        let utxo_input_type = TxInputType::Coin;
        let utxo_set = UTXO::new(
            utxo_key.clone(),
            utxo_value.clone(),
            utxo_input_type.clone(),
        );

        let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
        let utxo = utxo_storage.add(utxo_key.clone(), utxo_value, utxo_input_type.clone());
        assert_eq!(utxo.unwrap(), utxo_set);
        let get_utxo = utxo_storage.get_utxo_by_id(utxo_key, utxo_input_type);
        assert_eq!(get_utxo.unwrap(), utxo_set);
        uninstall_delete_db_utxo_for_test(test_path);
        println!("db_get: {:#?}", utxo_storage);
    }

    // cargo test -- --nocapture --test take_snapshot_utxoset_test --test-threads 5
    #[test]
    fn take_snapshot_utxoset_test() {
        let test_path = "test6";
        init_utxo_for_test(test_path);
        let utxo_key: UtxoKey = String::from("utxo_key");
        let utxo_value: UtxoValue = String::from("utxo_value");
        let utxo_input_type = TxInputType::Coin;
        let utxo_set = UTXO::new(
            utxo_key.clone(),
            utxo_value.clone(),
            utxo_input_type.clone(),
        );

        let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
        let utxo = utxo_storage.add(utxo_key, utxo_value, utxo_input_type);
        assert_eq!(utxo.unwrap(), utxo_set);

        let _ = utxo_storage.take_snapshot();
        uninstall_delete_db_utxo_for_test(test_path);
        println!("db_take_snap: {:#?}", utxo_storage);
    }

    // cargo test -- --nocapture --test load_snapshot_utxoset_test --test-threads 5
    #[test]
    fn load_snapshot_utxoset_test() {
        let test_path = "test7";
        init_utxo_for_test(test_path);
        let utxo_key: UtxoKey = String::from("utxo_key");
        let utxo_value: UtxoValue = String::from("utxo_value");
        let utxo_input_type = TxInputType::Coin;
        let utxo_set = UTXO::new(
            utxo_key.clone(),
            utxo_value.clone(),
            utxo_input_type.clone(),
        );

        let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
        let utxo = utxo_storage.add(utxo_key, utxo_value, utxo_input_type);
        assert_eq!(utxo.unwrap(), utxo_set);

        let _ = utxo_storage.take_snapshot();

        let snapshot_load = SnapShot::load(&format!("./snapshot_storage_test/{}/map", test_path));
        // utxostore.snaps = snapshot_load;
        let mut load_utxostore = UTXOStorage {
            coin_storage: std::collections::HashMap::new(),
            memo_storage: std::collections::HashMap::new(),
            state_storage: std::collections::HashMap::new(),
            block_height: 0,
            aggrigate_log_sequence: 0,
            snaps: snapshot_load,
            pending_commands: Vec::new(),
        };
        load_utxostore.load_from_snapshot();

        uninstall_delete_db_utxo_for_test(test_path);
        println!("db_load_snapshot: {:#?}", load_utxostore);
    }

    // cargo test -- --nocapture --test process_block_in_utxostore_test --test-threads 5
    // first I will add 4 utxo set in utxostore
    // then I will add 2 more utxo set in utxostore and remove 2 utxo sets from intialling added sets
    #[test]
    fn process_block_in_utxostore_test() {
        let test_path = "test7";
        init_utxo_for_test(test_path);
        let utxo_set1 = UTXO::new(
            String::from("utxo_key1"),
            String::from("utxo_value1"),
            TxInputType::Coin,
        );
        let utxo_set2 = UTXO::new(
            String::from("utxo_key2"),
            String::from("utxo_value2"),
            TxInputType::Memo,
        );
        let utxo_set3 = UTXO::new(
            String::from("utxo_key3"),
            String::from("utxo_value3"),
            TxInputType::Coin,
        );
        let utxo_set4 = UTXO::new(
            String::from("utxo_key4"),
            String::from("utxo_value4"),
            TxInputType::State,
        );
        let utxo_set5 = UTXO::new(
            String::from("utxo_key5"),
            String::from("utxo_value5"),
            TxInputType::State,
        );
        let utxo_set6 = UTXO::new(
            String::from("utxo_key6"),
            String::from("utxo_value6"),
            TxInputType::Memo,
        );

        let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
        // adding first/intial utxo sets in the utxostore
        let first_add_utxo = vec![utxo_set1.clone(), utxo_set2.clone(), utxo_set3, utxo_set4];
        let block1 = ZkBlock::new(first_add_utxo, Vec::new(), 1);
        utxo_storage.process_block(block1);

        // adding and removing utxo sets in second step with block height 2
        let second_add_utxo = vec![utxo_set5, utxo_set6];
        let second_remove_utxo = vec![utxo_set1, utxo_set2];
        let block2 = ZkBlock::new(second_add_utxo, second_remove_utxo, 2);
        utxo_storage.process_block(block2);
        uninstall_delete_db_utxo_for_test(test_path);
        println!("db_load_snapshot: {:#?}", utxo_storage);
    }
}
