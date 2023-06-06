use serde_derive::{Deserialize, Serialize};
pub type UtxoKey = Vec<u8>;
pub type UtxoValue = Vec<u8>; // pub struct Output {pub out_type: OutputType, pub output: OutputData,}

pub type SequenceNumber = usize;
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

    pub fn get_utxo_from_record_utxo_output(
        record_utxo_vec: Vec<transaction::reference_tx::RecordUtxo>,
    ) -> Vec<UTXO> {
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
