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
        AddressUtxoIDStorage {
            data: HashMap::new(),
        }
    }
    pub fn get_utxo_id_by_address(
        &mut self,
        address: String,
        input_type: IOType,
    ) -> Option<&String> {
        self.data
            .get_mut(&input_type.to_usize())
            .unwrap()
            .get(&address)
            .clone()
    }
    pub fn add(
        &mut self,
        input_type: IOType,
        address: String,
        utxo_id: String,
    ) -> Option<String> {
        self.data
            .get_mut(&input_type.to_usize())
            .unwrap()
            .insert(address.clone(), utxo_id.clone())
    }

    fn remove(&mut self, input_type: IOType, address: String) -> Result<String, std::io::Error> {
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
