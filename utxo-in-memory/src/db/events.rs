use super::UTXO_OP;
use crate::SequenceNumber;
use serde_derive::{Deserialize, Serialize};
use std::time::SystemTime;
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Event {
    UTXOInit(SystemTime),
    UTXOUpdate(UTXO_OP, SystemTime, SequenceNumber),
    UTXOReload(SequenceNumber, SystemTime),
    StartSnapShot(UTXO_OP),
    Stop(String), //time in miliseconds
}
