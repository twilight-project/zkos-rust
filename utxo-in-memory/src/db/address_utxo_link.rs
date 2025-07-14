use crate::db::*;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use zkvm::IOType;
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AddressUtxoIDStorage {
    pub data: HashMap<usize, HashMap<String, String>>,
}
impl AddressUtxoIDStorage {
    pub fn new() -> Self {
        let mut data: HashMap<usize, HashMap<String, String>> = HashMap::new();
        data.insert(0, HashMap::new());
        data.insert(1, HashMap::new());
        data.insert(2, HashMap::new());
        AddressUtxoIDStorage { data: data }
    }
    pub fn get_utxo_id_by_address(&self, address: String, input_type: IOType) -> Option<String> {
        match self.data.get(&input_type.to_usize()) {
            Some(utxo_id) => match utxo_id.get(&address) {
                Some(utxo_id) => Some(utxo_id.clone()),
                None => None,
            },
            None => None,
        }
    }
    pub fn add(&mut self, input_type: IOType, address: String, utxo_id: String) -> Option<String> {
        self.data
            .get_mut(&input_type.to_usize())
            .unwrap()
            .insert(address.clone(), utxo_id.clone())
    }

    pub fn remove(
        &mut self,
        input_type: IOType,
        address: String,
    ) -> Result<String, std::io::Error> {
        match self
            .data
            .get_mut(&input_type.to_usize())
            .unwrap()
            .remove(&address)
        {
            Some(value) => {
                return Ok(value.clone());
            }
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("utxo id:{:?} not found", address),
                ))
            }
        }
    }
}
