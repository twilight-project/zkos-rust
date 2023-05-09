#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(non_camel_case_types)]
use super::{LogSequence, UTXO_OP};
use serde_derive::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{mpsc, Arc, Mutex, RwLock};
use std::thread;
use std::time::SystemTime;
// /// Identification of unspend transaction output.

lazy_static! {
    // pub static ref TRADER_LP_LONG: Arc<Mutex<SortedSet>> = Arc::new(Mutex::new(SortedSet::new()));
    // pub static ref LEND_ORDER_DB: Arc<Mutex<OrderDB<LendOrder>>> =
    //     Arc::new(Mutex::new(LocalDB::<LendOrder>::new()));
    pub static ref UTXO_Storage: Arc<Mutex<UTXOStorage>> =
        Arc::new(Mutex::new(UTXOStorage::new()));
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Debug, Clone)]
pub struct UTXOStorage {
    pub ordertable: HashMap<String, Arc<RwLock<String>>>,
    pub sequence: usize,
    pub aggrigate_log_sequence: usize,
    pub last_snapshot_id: LogSequence,
    pub pending_commands: Vec<String>,
}
impl UTXOStorage {
    pub fn new() -> UTXOStorage {
        UTXOStorage {
            ordertable: HashMap::new(),
            sequence: 0,
            aggrigate_log_sequence: 0,
            last_snapshot_id: 0,
            pending_commands: Vec::new(),
        }
    }
    pub fn execute_command(cmd: UTXO_OP) {}
}
