use super::{LogSequence, UTXO_OP};
use serde_derive::{Deserialize, Serialize};
use std::time::SystemTime;
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Event {
    UTXOInit(SystemTime),
    UTXOUpdate(UTXO_OP, SystemTime, LogSequence),
    UTXOReload(LogSequence, SystemTime),
    StartSnapShot(UTXO_OP),
    Stop(String), //time in miliseconds
}
