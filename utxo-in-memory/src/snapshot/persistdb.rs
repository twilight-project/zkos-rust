use super::snap_rules::SnapRules;
use crate::db::SequenceNumber;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapShot {
    map: HashMap<u64, SequenceNumber>,
    currentsnapid: u64,
    lastsnapid: u64,
    lastsnaptimestamp: u64,
    block_height: SequenceNumber,
    snap_rules: SnapRules,
}
impl SnapShot {
    pub fn new() -> SnapShot {
        SnapShot {
            map: HashMap::new(),
            currentsnapid: 0,
            lastsnapid: 0,
            lastsnaptimestamp: 0,
            block_height: 0,
            snap_rules: SnapRules::env(),
        }
    }
}

pub fn get_snapshot() {}
pub fn load_from_snapshot() {}
