#![allow(non_camel_case_types)]
use super::LogSequence;
use serde_derive::{Deserialize, Serialize};
use std::time::SystemTime;
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum UTXO_OP {
    Add(String, String),
    AddBulk(Vec<(String, String)>),
    Remove(String),
    RemoveBUlk(Vec<String>),
    Search(String),
    SearchBulk(Vec<String>),
    Snapshot(LogSequence, SystemTime), //consider converting it into utc iso time later
    StopTime(SystemTime),              //might use in future
    ReloadDB(LogSequence, SystemTime), //might use in future
}
