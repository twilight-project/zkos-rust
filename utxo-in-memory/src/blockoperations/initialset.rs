use curve25519_dalek::scalar::Scalar;
use quisquislib::accounts::Account;
use std::fs;
use std::io::prelude::*;
use transaction::reference_tx::RecordUtxo;
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

// in future get the geensis block data from the chain
// let (acc, prv) = Account::generate_random_account_with_value(Scalar::from(20u64));
// let recordutxo = transaction::reference_tx::create_genesis_block(10000, 100, acc);
// let mut file = std::fs::File::create("../utxo-in-memory\\src\\blockoperations\\genesis_sets.txt").unwrap();
// file.write_all(&serde_json::to_vec_pretty(&recordutxo.clone()).unwrap())
//     .unwrap();
//written insite utxostore
// pub fn init_utxo() {
//     let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
//     utxo_storage.load_from_snapshot();
//     //load data from intial block from chain
//     if utxo_storage.block_height == 0 {
//         let recordutxo = crate::blockoperations::load_genesis_sets();
//         let add_utxo = UTXO::get_utxo_from_record_utxo_output(recordutxo);
//         for utxo in add_utxo {
//             let _ = utxo_storage.add(utxo.key, utxo.value, utxo.input_type);
//         }
//         utxo_storage.block_height = 1;
//     }
// }

pub fn set_genesis_sets() {
    let (acc, _prv) = Account::generate_random_account_with_value(Scalar::from(20u64));
    let recordutxo = transaction::reference_tx::create_genesis_block(10000, 100, acc);
    let mut file =
        std::fs::File::create("../utxo-in-memory\\src\\blockoperations\\genesis_sets.txt").unwrap();
    file.write_all(&bincode::serialize(&recordutxo.clone()).unwrap())
        .unwrap();
}
