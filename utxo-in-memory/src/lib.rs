pub mod db;
pub mod dbcurd;
mod threadpool;
#[macro_use]
extern crate lazy_static;
pub use self::db::SnapShot;
pub use self::threadpool::ThreadPool;
use db::{LocalDBtrait, LocalStorage, TxInputOutputType};
use std::sync::{Arc, Mutex};

lazy_static! {
    pub static ref UTXO_STORAGE: Arc<Mutex<LocalStorage::<transaction::Output>>> =
        Arc::new(Mutex::new(LocalStorage::<transaction::Output>::new(3)));
}

pub fn init_utxo() {
    let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
    let _ = utxo_storage.load_from_snapshot();
    //load data from intial block from chain
    if utxo_storage.block_height == 0 {
        let recordutxo = crate::dbcurd::load_genesis_sets();
        for utxo in recordutxo {
            let _ = utxo_storage.add(
                bincode::serialize(&utxo.utx).unwrap(),
                utxo.value.clone(),
                utxo.value.out_type as usize,
            );
        }
        utxo_storage.block_height = 1;
    }
}
