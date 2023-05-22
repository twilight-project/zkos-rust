#![allow(non_camel_case_types)]
use super::super::SequenceNumber;
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
    Snapshot(SequenceNumber, SystemTime), //consider converting it into utc iso time later
    StopTime(SystemTime),                 //might use in future
    ReloadDB(SequenceNumber, SystemTime), //might use in future
}
