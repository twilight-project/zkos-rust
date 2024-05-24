use crate::db::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AddressUtxoIDStorage {
    pub data: HashMap<IOType, HashMap<String, String>>,
}
impl AddressUtxoIDStorage {
    pub fn new() -> Self {
        struct AddressUtxoIDStorage {
            data: HashMap::new(),
        }
    }
    pub fn get_utxo_id_by_address(address: String, input_type: IOType) -> Option<String> {
        self.data
            .get_mut(&input_type)
            .unwrap()
            .get(&address)
            .clone()
    }
    pub fn add(
        &mut self,
        input_type: IOType,
        address: String,
        utxo_id: String,
    ) -> Result<T, std::io::Error> {
        self.data
            .get_mut(&input_type)
            .unwrap()
            .insert(address.clone(), utxo_id.clone())
    }

    fn remove(&mut self, input_type: IOType, address: String) -> Result<T, std::io::Error> {
        match self.data.get_mut(&input_type).unwrap().remove(&address) {
            Some(value) => {
                return Ok(value.clone());
            }
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("utxo id:{:?} not found", id),
                ))
            }
        }
    }
}
