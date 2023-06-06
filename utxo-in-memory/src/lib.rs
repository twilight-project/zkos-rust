pub mod db;
pub mod dbcurd;
mod threadpool;
mod types;
#[macro_use]
extern crate lazy_static;
pub use self::db::SnapShot;
// pub use self::db::{init_utxo, UTXO_STORAGE};
pub use self::threadpool::ThreadPool;
// pub use self::types::*;
use curve25519_dalek::scalar::Scalar;
use db::{LocalDB, TxInputOutputType, UTXOStorage};
use quisquislib::accounts::Account;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
lazy_static! {
    pub static ref UTXO_STORAGE: Arc<Mutex<UTXOStorage::<transaction::Output>>> =
        Arc::new(Mutex::new(UTXOStorage::<transaction::Output>::new()));
}

pub fn init_utxo() {
    let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
    let _ = utxo_storage.load_from_snapshot();
    //load data from intial block from chain
    if utxo_storage.block_height == 0 {
        println!("I'm here");
        let (acc, prv) = Account::generate_random_account_with_value(Scalar::from(20u64));
        let recordutxo = transaction::reference_tx::create_genesis_block(10000, 100, acc);
        // let recordutxo = crate::dbcurd::load_genesis_sets();
        // let add_utxo = UTXO::get_utxo_from_record_utxo_output(recordutxo);
        for utxo in recordutxo {
            let _ = utxo_storage.add(
                bincode::serialize(&utxo.utx).unwrap(),
                utxo.value.clone(),
                TxInputOutputType::convert_output_type(utxo.value.out_type),
            );
        }
        utxo_storage.block_height = 1;
    }
}
