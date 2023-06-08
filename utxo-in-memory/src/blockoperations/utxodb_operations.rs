// use crate::db::{UTXOStorage, UTXO_STORAGE};
// use crate::types::UTXO;
// use crate::{SequenceNumber, ZkBlock, ZkBlockResult};
// use crate::*;
use crate::types::*;
use serde_derive::{Deserialize, Serialize};
pub type UtxoKey = Vec<u8>; //pub struct Utxo {txid: TxId,output_index: u8,}
pub type UtxoValue = Vec<u8>; // pub struct Output {pub out_type: OutputType, pub output: OutputData,}
use transaction::reference_tx::{Block, RecordUtxo};
use transaction::{InputType, OutputType};
use transaction::{TransactionData, TxId, Utxo};
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TxInputOutputType {
    Coin = 0, //uint8
    Memo = 1, //uint8
    State = 2, //uint8
              // Genesis = 3, //uint8
}
impl TxInputOutputType {
    pub fn convert_input_type(input_type: transaction::InputType) -> Self {
        match input_type {
            transaction::InputType::Coin => TxInputOutputType::Coin,
            transaction::InputType::State => TxInputOutputType::State,
            transaction::InputType::Memo => TxInputOutputType::Memo,
        }
    }
    pub fn convert_output_type(output_type: transaction::OutputType) -> Self {
        match output_type {
            transaction::OutputType::Coin => TxInputOutputType::Coin,
            transaction::OutputType::State => TxInputOutputType::State,
            transaction::OutputType::Memo => TxInputOutputType::Memo,
        }
    }
    pub fn convert_uint8(&self) -> u8 {
        self.clone() as u8
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UTXO {
    pub key: UtxoKey,
    pub value: UtxoValue,
    pub input_type: TxInputOutputType,
}

impl UTXO {
    pub fn default() -> Self {
        UTXO {
            key: bincode::serialize(&"".to_string()).unwrap(),
            value: bincode::serialize(&"".to_string()).unwrap(),
            input_type: TxInputOutputType::Coin,
        }
    }
    pub fn new(key: UtxoKey, value: UtxoValue, input_type: TxInputOutputType) -> Self {
        UTXO {
            key,
            value,
            input_type,
        }
    }

    pub fn get_utxokey_from_input_block(input: transaction::Input) -> Self {
        UTXO::new(
            bincode::serialize(input.input.as_utxo_id().unwrap()).unwrap(),
            bincode::serialize(&"".to_string()).unwrap(),
            TxInputOutputType::convert_input_type(input.in_type),
        )
        // UTXO::default()
    }

    pub fn get_utxo_from_output_block(
        output: &transaction::Output,
        txid: transaction::TxId,
        output_index: usize,
    ) -> Self {
        UTXO::new(
            bincode::serialize(&transaction::Utxo::new(txid, output_index as u8)).unwrap(),
            bincode::serialize(&output).unwrap(),
            TxInputOutputType::convert_output_type(output.out_type),
        )
    }

    pub fn get_utxo_from_record_utxo_output(record_utxo_vec: Vec<RecordUtxo>) -> Vec<UTXO> {
        let mut utxo_out: Vec<UTXO> = Vec::new();
        for record_utxo in record_utxo_vec {
            utxo_out.push(UTXO::new(
                bincode::serialize(&record_utxo.utx).unwrap(),
                bincode::serialize(&record_utxo.value).unwrap(),
                TxInputOutputType::convert_output_type(record_utxo.value.out_type),
            ));
        }
        return utxo_out;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ZkosBlock {
    pub add_utxo: Vec<UTXO>,
    pub remove_block: Vec<UTXO>,
    pub block_height: SequenceNumber,
}
impl ZkosBlock {
    pub fn default() -> Self {
        ZkosBlock {
            add_utxo: Vec::new(),
            remove_block: Vec::new(),
            block_height: 0,
        }
    }
    pub fn new(
        add_utxo_sets: Vec<UTXO>,
        remove_utxo_sets: Vec<UTXO>,
        block_height: SequenceNumber,
    ) -> Self {
        ZkosBlock {
            add_utxo: add_utxo_sets,
            remove_block: remove_utxo_sets,
            block_height: block_height,
        }
    }

    pub fn get_block_details(block: Block) -> Self {
        let block_height: usize = block.height as usize;
        let mut input_utxo_set: Vec<UTXO> = Vec::new();
        let mut output_utxo_set: Vec<UTXO> = Vec::new();
        for tx in block.txs {
            let tx_id = tx.txid;
            match tx.tx {
                TransactionData::TransactionTransfer(transfer_transaction) => {
                    for input_set in transfer_transaction.get_input_values() {
                        input_utxo_set.push(UTXO::get_utxokey_from_input_block(input_set));
                    }
                    for (output_index, output_set) in
                        transfer_transaction.get_output_values().iter().enumerate()
                    {
                        output_utxo_set.push(UTXO::get_utxo_from_output_block(
                            output_set,
                            tx_id,
                            output_index,
                        ));
                    }
                }
                TransactionData::Script(script_transaction) => {
                    for input_set in script_transaction.get_input_values() {
                        input_utxo_set.push(UTXO::get_utxokey_from_input_block(input_set));
                    }
                    for (output_index, output_set) in
                        script_transaction.get_output_values().iter().enumerate()
                    {
                        output_utxo_set.push(UTXO::get_utxo_from_output_block(
                            output_set,
                            tx_id,
                            output_index,
                        ));
                    }
                }
            }
        }
        ZkosBlock {
            add_utxo: output_utxo_set,
            remove_block: input_utxo_set,
            block_height: block_height,
        }
    }
}

#[derive(Debug)]
pub struct ZkosBlockResult {
    pub utxo_added: Vec<UTXO>,
    pub utxo_removed: Vec<UTXO>,
    pub error_vec: Vec<std::io::Error>,
}
impl ZkosBlockResult {
    pub fn new() -> Self {
        ZkosBlockResult {
            utxo_added: Vec::new(),
            utxo_removed: Vec::new(),
            error_vec: Vec::new(),
        }
    }
}

pub type ZkBlock = ZkosBlock;
pub type ZkBlockResult = ZkosBlockResult;
