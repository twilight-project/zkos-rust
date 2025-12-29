//! Genesis set management module for loading and creating initial UTXO sets.
//!
//! This module handles the loading of genesis UTXO sets from files and provides
//! functionality to create new genesis blocks for testing and initialization.

use curve25519_dalek::scalar::Scalar;
use quisquislib::accounts::Account;
use std::fs;
use std::io::prelude::*;
use transaction::reference_tx::RecordUtxo;

/// Loads genesis UTXO set from the main genesis file
///
/// # Returns
/// Vector of RecordUtxo containing the genesis UTXO set
pub fn load_genesis_sets() -> Vec<RecordUtxo> {
    let read_data = fs::read("../utxo-in-memory\\src\\blockoperations\\genesis_sets.txt");
    let mut record_utxo: Vec<RecordUtxo> = Vec::new();
    match read_data {
        Ok(data) => {
            record_utxo = bincode::deserialize(&data).unwrap();
        }
        Err(arg) => {
            println!("File not found:{:#?}", arg);
        }
    }
    record_utxo
}

/// Loads test genesis UTXO set from the test genesis file
///
/// # Returns
/// Vector of RecordUtxo containing the test genesis UTXO set
pub fn load_genesis_sets_test() -> Vec<RecordUtxo> {
    let read_data =
        fs::read("../utxo-in-memory\\src\\blockoperations\\test\\genesis_sets_test.txt");
    let mut record_utxo: Vec<RecordUtxo> = Vec::new();
    match read_data {
        Ok(data) => {
            record_utxo = bincode::deserialize(&data).unwrap();
        }
        Err(arg) => {
            println!("File not found:{:#?}", arg);
        }
    }
    record_utxo
}

/// Creates and saves a new genesis block with random account and UTXOs
///
/// This function generates a random account with 20 units of value and creates
/// a genesis block with 10000 UTXOs, 100 of which are script UTXOs. The result
/// is serialized and saved to the genesis_sets.txt file.
pub fn set_genesis_sets() {
    let (acc, prv) = Account::generate_random_account_with_value(Scalar::from(20u64));
    let recordutxo = transaction::reference_tx::create_genesis_block(10000, 100, acc);
    let mut file =
        std::fs::File::create("../utxo-in-memory\\src\\blockoperations\\genesis_sets.txt").unwrap();
    file.write_all(&bincode::serialize(&recordutxo.clone()).unwrap())
        .unwrap();
}
