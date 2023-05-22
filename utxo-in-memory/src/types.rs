use serde_derive::{Deserialize, Serialize};
pub type SequenceNumber = usize;
pub type UtxoKey = String;
pub type UtxoValue = String;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TxInputType {
    Coin = 0,  //uint8
    Memo = 1,  //uint8
    State = 2, //uint8
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UTXO {
    pub key: UtxoKey,
    pub value: UtxoValue,
    pub input_type: TxInputType,
}

impl UTXO {
    pub fn default() -> Self {
        UTXO {
            key: "".to_string(),
            value: "".to_string(),
            input_type: TxInputType::Coin,
        }
    }
    pub fn new(key: UtxoKey, value: UtxoValue, input_type: TxInputType) -> Self {
        UTXO {
            key,
            value,
            input_type,
        }
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
}
#[derive(Debug)]
pub struct ZkosBlockResult {
    pub key_added: Vec<UtxoKey>,
    pub key_removed: Vec<UtxoKey>,
    pub error_vec: Vec<std::io::Error>,
}
impl ZkosBlockResult {
    pub fn new() -> Self {
        ZkosBlockResult {
            key_added: Vec::new(),
            key_removed: Vec::new(),
            error_vec: Vec::new(),
        }
    }
}

pub type ZkBlock = ZkosBlock;
pub type ZkBlockResult = ZkosBlockResult;
