use crate::db::*;
use crate::UTXO_STORAGE;
use serde_derive::{Deserialize, Serialize};
use transaction::reference_tx::Block;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BlockResult {
    pub suceess_tx: Vec<transaction::TxId>,
    pub failed_tx: Vec<transaction::TxId>,
}
impl BlockResult {
    pub fn new() -> Self {
        BlockResult {
            suceess_tx: Vec::new(),
            failed_tx: Vec::new(),
        }
    }
}

pub fn process_block_for_utxo_insert(block: Block) -> BlockResult {
    let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
    let mut tx_result: BlockResult = BlockResult::new();
    for transaction in block.txs {
        let tx_id = transaction.txid;
        let mut success: bool = true;
        let tx_input = transaction.get_input_values();
        let tx_output = transaction.get_output_values();

        for input in &tx_input {
            let utxo_key = bincode::serialize(input.input.as_utxo_id().unwrap()).unwrap();
            let utxo_input_type = input.in_type as usize;
            let bool = utxo_storage.search_key(&utxo_key, utxo_input_type);
            if bool {
            } else {
                success = false;
            }
        }
        for (output_index, output_set) in tx_output.iter().enumerate() {
            let utxo_key =
                bincode::serialize(&transaction::Utxo::new(tx_id, output_index as u8)).unwrap();
            let utxo_output_type = output_set.out_type as usize;
            let bool = utxo_storage.search_key(&utxo_key, utxo_output_type);
            if bool {
                success = false;
            } else {
            }
        }
        //proccess tx
        if success {
            //remove all input
            for input in tx_input {
                let utxo_key = bincode::serialize(&input.input.as_utxo_id().unwrap()).unwrap();
                let utxo_input_type = input.in_type as usize;
                let _result = utxo_storage.remove(utxo_key, utxo_input_type);
            }
            //Add all output
            for (output_index, output_set) in tx_output.iter().enumerate() {
                let utxo_key =
                    bincode::serialize(&transaction::Utxo::new(tx_id, output_index as u8)).unwrap();
                let utxo_output_type = output_set.out_type as usize;
                let _result = utxo_storage.add(utxo_key, output_set.clone(), utxo_output_type);
            }
            tx_result.suceess_tx.push(tx_id);
        } else {
            tx_result.failed_tx.push(tx_id);
        }
    }
    tx_result
}
