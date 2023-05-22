use crate::db::{UTXOStorage, UTXO_STORAGE};
use crate::types::UTXO;
use crate::{SequenceNumber, UtxoKey, UtxoValue, ZkBlock, ZkBlockResult};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UTXOCMD {
    pub utxo_set: Vec<UTXO>,
    pub block_height: SequenceNumber,
}

impl UTXOCMD {
    pub fn new(utxo_set: Vec<UTXO>, block_height: SequenceNumber) -> Self {
        UTXOCMD {
            utxo_set,
            block_height,
        }
    }
    pub fn add(&mut self, mut utxostore: &UTXOStorage) -> Vec<Result<UTXO, std::io::Error>> {
        let mut result: Vec<Result<UTXO, std::io::Error>> = Vec::new();
        let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
        for utxo in self.utxo_set.clone() {
            result.push(utxo_storage.add(utxo.key, utxo.value, utxo.input_type));
        }
        result
    }
    pub fn remove(&mut self, mut utxostore: &UTXOStorage) -> Vec<Result<UTXO, std::io::Error>> {
        let mut result: Vec<Result<UTXO, std::io::Error>> = Vec::new();
        let mut utxo_storage = UTXO_STORAGE.lock().unwrap();

        for utxo in self.utxo_set.clone() {
            result.push(utxo_storage.remove(utxo.key, utxo.input_type));
        }
        result
    }

    pub fn process_block(block: ZkBlock) -> Result<ZkBlockResult, std::io::Error> {
        // new to update after tx block module update
        let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
        let mut block_result: ZkBlockResult = ZkBlockResult::new();
        let mut add_utxo_cmd = UTXOCMD::new(block.add_utxo, block.block_height);
        let mut remove_utxo_cmd = UTXOCMD::new(block.remove_block, block.block_height);
        let add_result = add_utxo_cmd.add(&utxo_storage);
        let remove_result = remove_utxo_cmd.remove(&utxo_storage);
        for single_result in add_result {
            match single_result {
                Ok(utxo) => block_result.key_added.push(utxo.key),
                Err(arg) => block_result.error_vec.push(arg),
            }
        }
        for single_result in remove_result {
            match single_result {
                Ok(utxo) => block_result.key_removed.push(utxo.key),
                Err(arg) => block_result.error_vec.push(arg),
            }
        }
        utxo_storage.block_height = block.block_height;
        //should i update remaining utxo if some utxo not found or adready existed
        utxo_storage.aggrigate_log_sequence += 1;
        Ok(block_result)
    }
}
