use std::fs;
use transaction::reference_tx::RecordUtxo;

pub fn load_genesis_sets() -> Vec<RecordUtxo> {
    let read_data = fs::read("../utxo-in-memory\\src\\utxo_set\\genesis_sets.txt");
    let mut record_utxo: Vec<RecordUtxo> = Vec::new();
    match read_data {
        Ok(data) => {
            record_utxo = serde_json::from_slice(&data).unwrap();
        }
        Err(arg) => {
            println!("File not found:{:#?}", arg);
        }
    }
    record_utxo
}
