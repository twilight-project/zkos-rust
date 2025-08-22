//! ZkOS UTXO Types and Data Structures
//!
//! This module defines the core data structures used throughout the UTXO state management system.
//! It provides types for representing UTXOs, blocks, and transaction processing results.
//!
//! ## Core Types
//!
//! - **`UTXO`**: Individual unspent transaction output with key, value, and type
//! - **`ZkosBlock`**: Block representation with UTXO additions and removals
//! - **`ZkosBlockResult`**: Result of block processing operations
//! - **`TxInputOutputType`**: Enumeration of UTXO types (Coin, Memo, State)
//!
//! ## Type Aliases
//!
//! - **`UtxoKey`**: Serialized UTXO identifier (Vec<u8>)
//! - **`UtxoValue`**: Serialized UTXO value data (Vec<u8>)
//! - **`SequenceNumber`**: Block height and sequence tracking (usize)
//!
//! ## Usage
//!
//! ```rust
//! use utxo_in_memory::types::{UTXO, TxInputOutputType, ZkosBlock};
//!
//! // Create a new UTXO
//! let utxo = UTXO::new(
//!     bincode::serialize(&utxo_id).unwrap(),
//!     bincode::serialize(&output_data).unwrap(),
//!     TxInputOutputType::Coin
//! );
//!
//! // Create a block with UTXO operations
//! let block = ZkosBlock::new(
//!     vec![utxo.clone()],  // Add UTXOs
//!     vec![],              // Remove UTXOs
//!     123                  // Block height
//! );
//! ```

#![allow(non_snake_case)]
#![allow(missing_docs)]
use serde_derive::{Deserialize, Serialize};
pub type SequenceNumber = usize;

// bincode::serialize(&value).unwrap()
/// Serialized UTXO identifier used as key in storage
pub type UtxoKey = Vec<u8>; //pub struct Utxo {txid: TxId,output_index: u8,}
/// Serialized UTXO value data containing output information
pub type UtxoValue = Vec<u8>; // pub struct Output {pub out_type: OutputType, pub output: OutputData,}
use transaction::reference_tx::RecordUtxo;
use transaction::TransactionData;
use zkvm::zkos_types::{Utxo, IOType, Input, Output, OutputData};
use zkvm::tx::TxID;

use crate::blockoperations::blockprocessing::Block;

// impl UtxoKey {
//     pub fn serialize(utxo_key: transaction::Utxo) -> Vec<u8> {
//         bincode::serialize(&utxo_key).unwrap()
//     }
//     pub fn deserialize(json: &String) -> Self {
//         let deserialized: TraderOrder = serde_json::from_str(json).unwrap();
//         deserialized
//     }
// }

/// Enumeration of UTXO types supported by the ZkOS system
/// 
/// Each type represents a different category of state in the system:
/// - **Coin**: Confidential digital assets with ElGamal encryption
/// - **Memo**: Programmable data containers with time-bound access
/// - **State**: Smart contract state with nonce-based versioning
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TxInputOutputType {
    /// Confidential digital assets (Type 0)
    Coin = 0, //uint8
    /// Programmable data containers (Type 1)
    Memo = 1, //uint8
    /// Smart contract state (Type 2)
    State = 2, //uint8
              // Genesis = 3, //uint8
}

impl TxInputOutputType {
    /// Converts an IOType to TxInputOutputType for input processing
    /// 
    /// # Arguments
    /// * `input_type` - The IOType to convert
    /// 
    /// # Returns
    /// * Corresponding TxInputOutputType
    pub fn convert_input_type(input_type: IOType) -> Self {
        match input_type {
            IOType::Coin => TxInputOutputType::Coin,
            IOType::State => TxInputOutputType::State,
            IOType::Memo => TxInputOutputType::Memo,
        }
    }
    
    /// Converts an IOType to TxInputOutputType for output processing
    /// 
    /// # Arguments
    /// * `output_type` - The IOType to convert
    /// 
    /// # Returns
    /// * Corresponding TxInputOutputType
    pub fn convert_output_type(output_type: IOType) -> Self {
        match output_type {
            IOType::Coin => TxInputOutputType::Coin,
            IOType::State => TxInputOutputType::State,
            IOType::Memo => TxInputOutputType::Memo,
        }
    }
    
    /// Converts the enum to its underlying u8 representation
    /// 
    /// # Returns
    /// * u8 value representing the type
    pub fn convert_uint8(&self) -> u8 {
        self.clone() as u8
    }
}

/// Individual Unspent Transaction Output (UTXO) representation
/// 
/// A UTXO represents an unspent transaction output in the ZkOS system.
/// Each UTXO contains a unique key, serialized value data, and type classification.
/// 
/// # Fields
/// * `key` - Serialized UTXO identifier (transaction ID + output index)
/// * `value` - Serialized output data containing the actual UTXO information
/// * `input_type` - Type classification (Coin, Memo, or State)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UTXO {
    /// Serialized UTXO identifier
    pub key: UtxoKey,
    /// Serialized UTXO value data
    pub value: UtxoValue,
    /// Type classification of the UTXO
    pub input_type: TxInputOutputType,
}

impl UTXO {
    /// Creates a default UTXO with empty key/value and Coin type
    /// 
    /// # Returns
    /// * Default UTXO instance
    pub fn default() -> Self {
        UTXO {
            key: bincode::serialize(&"".to_string()).unwrap(),
            value: bincode::serialize(&"".to_string()).unwrap(),
            input_type: TxInputOutputType::Coin,
        }
    }
    
    /// Creates a new UTXO with specified key, value, and type
    /// 
    /// # Arguments
    /// * `key` - Serialized UTXO identifier
    /// * `value` - Serialized UTXO value data
    /// * `input_type` - Type classification
    /// 
    /// # Returns
    /// * New UTXO instance
    pub fn new(key: UtxoKey, value: UtxoValue, input_type: TxInputOutputType) -> Self {
        UTXO {
            key,
            value,
            input_type,
        }
    }

    /// Creates a UTXO from a transaction input block
    /// 
    /// This method extracts the UTXO key from a transaction input and creates
    /// a corresponding UTXO representation for removal from storage.
    /// 
    /// # Arguments
    /// * `input` - Transaction input containing UTXO reference
    /// 
    /// # Returns
    /// * UTXO instance representing the input
    pub fn get_utxokey_from_input_block(input: Input) -> Self {
        UTXO::new(
            bincode::serialize(input.as_utxo().unwrap()).unwrap(),
            bincode::serialize(&"".to_string()).unwrap(),
            TxInputOutputType::convert_input_type(input.in_type),
        )
        // UTXO::default()
    }

    /// Creates a UTXO from a transaction output block
    /// 
    /// This method creates a UTXO representation from a transaction output,
    /// including the transaction ID and output index for unique identification.
    /// 
    /// # Arguments
    /// * `output` - Transaction output data
    /// * `txid` - Transaction identifier
    /// * `output_index` - Index of the output within the transaction
    /// 
    /// # Returns
    /// * UTXO instance representing the output
    pub fn get_utxo_from_output_block(
        output: &Output,
        txid: TxID,
        output_index: usize,
    ) -> Self {
        UTXO::new(
            bincode::serialize(&Utxo::new(txid, output_index as u8)).unwrap(),
            bincode::serialize(&output).unwrap(),
            TxInputOutputType::convert_output_type(output.out_type),
        )
    }

    /// Creates UTXOs from a vector of record UTXO outputs
    /// 
    /// This method processes a collection of record UTXOs and converts them
    /// to the internal UTXO representation for storage.
    /// 
    /// # Arguments
    /// * `record_utxo_vec` - Vector of record UTXO outputs
    /// 
    /// # Returns
    /// * Vector of UTXO instances
    pub fn get_utxo_from_record_utxo_output(record_utxo_vec: Vec<RecordUtxo>) -> Vec<UTXO> {
        let mut utxo_out: Vec<UTXO> = Vec::new();
        for record_utxo in record_utxo_vec {
            utxo_out.push(UTXO::new(
                bincode::serialize(&record_utxo.utx).unwrap(),
                bincode::serialize(&record_utxo.value).unwrap(),
                TxInputOutputType::convert_output_type(record_utxo.value.out_type),
            ));
        }
        return utxo_out;
    }
}

/// ZkOS block representation for UTXO processing
/// 
/// A ZkosBlock contains the UTXO operations (additions and removals) that
/// occur within a single blockchain block. This structure is used for
/// processing blocks and updating the UTXO state.
/// 
/// # Fields
/// * `add_utxo` - Vector of UTXOs to add to the state
/// * `remove_block` - Vector of UTXOs to remove from the state
/// * `block_height` - Height of the block in the blockchain
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ZkosBlock {
    /// UTXOs to add to the state
    pub add_utxo: Vec<UTXO>,
    /// UTXOs to remove from the state
    pub remove_block: Vec<UTXO>,
    /// Block height in the blockchain
    pub block_height: SequenceNumber,
}

impl ZkosBlock {
    /// Creates a default ZkosBlock with empty UTXO lists and height 0
    /// 
    /// # Returns
    /// * Default ZkosBlock instance
    pub fn default() -> Self {
        ZkosBlock {
            add_utxo: Vec::new(),
            remove_block: Vec::new(),
            block_height: 0,
        }
    }
    
    /// Creates a new ZkosBlock with specified UTXO operations and height
    /// 
    /// # Arguments
    /// * `add_utxo_sets` - UTXOs to add to the state
    /// * `remove_utxo_sets` - UTXOs to remove from the state
    /// * `block_height` - Height of the block
    /// 
    /// # Returns
    /// * New ZkosBlock instance
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

    /// Extracts UTXO operations from a blockchain block
    /// 
    /// This method processes a blockchain block and extracts all UTXO
    /// additions and removals from the transactions within the block.
    /// It handles both transfer and script transactions.
    /// 
    /// # Arguments
    /// * `block` - Blockchain block containing transactions
    /// 
    /// # Returns
    /// * ZkosBlock with extracted UTXO operations
    pub fn get_block_details(block: Block) -> Self {
        let block_height: usize = block.block_height as usize;
        let mut input_utxo_set: Vec<UTXO> = Vec::new();
        let mut output_utxo_set: Vec<UTXO> = Vec::new();
        for tx in block.transactions.iter() {
            let tx_id = tx.tx_id;
            match tx.tx_type {
                TransactionData::TransactionTransfer(transfer_transaction) => {
                    for input_set in transfer_transaction.get_input_values() {
                        input_utxo_set.push(UTXO::get_utxokey_from_input_block(input_set));
                    }
                    for (output_index, output_set) in
                        transfer_transaction.get_output_values().iter().enumerate()
                    {
                        output_utxo_set.push(UTXO::get_utxo_from_output_block(
                            output_set,
                            tx_id,
                            output_index,
                        ));
                    }
                }
                TransactionData::TransactionScript(script_transaction) => {
                    for input_set in script_transaction.get_input_values() {
                        input_utxo_set.push(UTXO::get_utxokey_from_input_block(input_set));
                    }
                    for (output_index, output_set) in
                        script_transaction.get_output_values().iter().enumerate()
                    {
                        output_utxo_set.push(UTXO::get_utxo_from_output_block(
                            output_set,
                            tx_id,
                            output_index,
                        ));
                    }
                }
            }
        }
        ZkosBlock {
            add_utxo: output_utxo_set,
            remove_block: input_utxo_set,
            block_height: block_height,
        }
    }
}

/// Result of ZkOS block processing operations
/// 
/// This structure contains the results of processing a ZkosBlock,
/// including successfully added/removed UTXOs and any errors that occurred.
/// 
/// # Fields
/// * `utxo_added` - UTXOs successfully added to the state
/// * `utxo_removed` - UTXOs successfully removed from the state
/// * `error_vec` - Errors encountered during processing
#[derive(Debug)]
pub struct ZkosBlockResult {
    /// UTXOs successfully added to the state
    pub utxo_added: Vec<UTXO>,
    /// UTXOs successfully removed from the state
    pub utxo_removed: Vec<UTXO>,
    /// Errors encountered during block processing
    pub error_vec: Vec<std::io::Error>,
}

impl ZkosBlockResult {
    /// Creates a new ZkosBlockResult with empty vectors
    /// 
    /// # Returns
    /// * New ZkosBlockResult instance
    pub fn new() -> Self {
        ZkosBlockResult {
            utxo_added: Vec::new(),
            utxo_removed: Vec::new(),
            error_vec: Vec::new(),
        }
    }
}

/// Type alias for ZkosBlock
pub type ZkBlock = ZkosBlock;
/// Type alias for ZkosBlockResult
pub type ZkBlockResult = ZkosBlockResult;

// cargo test -- --nocapture --test-threads 1
// cargo test --test-threads 1
#[cfg(test)]

mod test {
    use super::*;
    use crate::db::*;
    use curve25519_dalek::scalar::Scalar;
    use quisquislib::accounts::Account;
    use std::fs;
    use transaction::reference_tx::RecordUtxo;    
    pub fn init_utxo_for_test(test_path: &str) {
        let mut utxo_storage = temp_env::with_var(
            "SNAPSHOT_FILE_LOCATION",
            Some(format!("./snapshot_storage_test/{}/map", test_path)),
            || UTXO_STORAGE.lock().unwrap(),
        );
        let snapshot_load =
            SnapShot::load(3, &format!("./snapshot_storage_test/{}/map", test_path));
        // utxostore.snaps = snapshot_load;
        let utxostore = UTXOStorage {
            coin_storage: std::collections::HashMap::new(),
            memo_storage: std::collections::HashMap::new(),
            state_storage: std::collections::HashMap::new(),
            block_height: 0,
            aggrigate_log_sequence: 0,
            snaps: snapshot_load,
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
        let utxo_key: UtxoKey = bincode::serialize(&String::from("utxo_key")).unwrap();
        let utxo_value: UtxoValue = bincode::serialize(&String::from("utxo_value")).unwrap();
        let utxo_input_type = TxInputOutputType::Coin;
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
        let utxo_key: UtxoKey = bincode::serialize(&String::from("utxo_key")).unwrap();
        let utxo_value: UtxoValue = bincode::serialize(&String::from("utxo_value")).unwrap();
        let utxo_input_type = TxInputOutputType::Coin;
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
        let utxo_key: UtxoKey = bincode::serialize(&String::from("utxo_key")).unwrap();
        let utxo_value: UtxoValue = bincode::serialize(&String::from("utxo_value")).unwrap();
        let utxo_input_type = TxInputOutputType::Coin;
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
        let search_bool = utxo_storage.search_key(&utxo_key, &utxo_input_type);
        assert_eq!(search_bool, true);
        uninstall_delete_db_utxo_for_test(test_path);
        println!("db_search: {:#?}", utxo_storage);
    }

    // cargo test -- --nocapture --test get_utxoset_test --test-threads 5
    #[test]
    fn get_utxoset_test() {
        let test_path = "test5";
        init_utxo_for_test(test_path);
        let utxo_key: UtxoKey = bincode::serialize(&String::from("utxo_key")).unwrap();
        let utxo_value: UtxoValue = bincode::serialize(&String::from("utxo_value")).unwrap();
        let utxo_input_type = TxInputOutputType::Coin;
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
        let utxo_key: UtxoKey = bincode::serialize(&String::from("utxo_key")).unwrap();
        let utxo_value: UtxoValue = bincode::serialize(&String::from("utxo_value")).unwrap();
        let utxo_input_type = TxInputOutputType::Coin;
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
        let utxo_key: UtxoKey = bincode::serialize(&String::from("utxo_key")).unwrap();
        let utxo_value: UtxoValue = bincode::serialize(&String::from("utxo_value")).unwrap();
        let utxo_input_type = TxInputOutputType::Coin;
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
        let test_path = "test8";
        init_utxo_for_test(test_path);
        let utxo_set1 = UTXO::new(
            bincode::serialize(&String::from("utxo_key1")).unwrap(),
            bincode::serialize(&String::from("utxo_value1")).unwrap(),
            TxInputOutputType::Coin,
        );
        let utxo_set2 = UTXO::new(
            bincode::serialize(&String::from("utxo_key2")).unwrap(),
            bincode::serialize(&String::from("utxo_value2")).unwrap(),
            TxInputOutputType::Memo,
        );
        let utxo_set3 = UTXO::new(
            bincode::serialize(&String::from("utxo_key3")).unwrap(),
            bincode::serialize(&String::from("utxo_value3")).unwrap(),
            TxInputOutputType::Coin,
        );
        let utxo_set4 = UTXO::new(
            bincode::serialize(&String::from("utxo_key4")).unwrap(),
            bincode::serialize(&String::from("utxo_value4")).unwrap(),
            TxInputOutputType::State,
        );
        let utxo_set5 = UTXO::new(
            bincode::serialize(&String::from("utxo_key5")).unwrap(),
            bincode::serialize(&String::from("utxo_value5")).unwrap(),
            TxInputOutputType::State,
        );
        let utxo_set6 = UTXO::new(
            bincode::serialize(&String::from("utxo_key6")).unwrap(),
            bincode::serialize(&String::from("utxo_value6")).unwrap(),
            TxInputOutputType::Memo,
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
        utxo_storage.take_snapshot();
        uninstall_delete_db_utxo_for_test(test_path);
        println!("db_load_snapshot: {:#?}", utxo_storage);
    }

    // cargo test -- --nocapture --test process_real_block_in_utxostore_test --test-threads 5
    #[test]
    fn process_real_block_in_utxostore_test() {
        let test_path = "test9";
        init_utxo_for_test(test_path);
        let (acc, prv) = Account::generate_random_account_with_value(Scalar::from(20u64));
        let sk_sender = vec![prv];
        let mut utxo_array = transaction::reference_tx::create_genesis_block(1000, 100, acc);

        // let mut utxo_array = utxo_set::load_genesis_sets();
        let block =
            transaction::reference_tx::create_utxo_test_block(&mut utxo_array, 1, &sk_sender);
        let mut file = std::fs::File::create("foo.txt").unwrap();
        file.write_all(&serde_json::to_vec_pretty(&block.clone()).unwrap())
            .unwrap();

        let zkblock = ZkosBlock::get_block_details(block);
        let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
        //check for any invalid key
        // println!("block:{:#?}", zkblock);

        // match utxo_storage.before_process_block(&zkblock) {
        //     Ok(_) => {
        //         utxo_storage.process_block(zkblock);
        //     }
        //     Err(arg) => {
        //         println!("utxo key not found to remove");
        //         panic!()
        //     }
        // }
        uninstall_delete_db_utxo_for_test(test_path);
        // println!("db_load_snapshot: {:#?}", utxo_storage);
    }
    use std::io::prelude::*;
}
