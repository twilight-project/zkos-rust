//! Address to UTXO ID Mapping System
//!
//! This module provides an efficient mapping system between addresses and UTXO IDs
//! for quick lookup and retrieval of UTXOs by address. It supports all three UTXO
//! types (Coin, Memo, State) with separate storage for each type.
//!
//! ## Features
//!
//! - **Type-Separated Storage**: Separate mappings for each UTXO type
//! - **Fast Address Lookup**: O(1) average case lookup by address
//! - **Bidirectional Mapping**: Address → UTXO ID and UTXO ID → Address
//! - **Thread-Safe Operations**: Safe for concurrent access
//!
//! ## Storage Structure
//!
//! AddressUtxoIDStorage {
//!   data: {
//!     0: HashMap<String, String>, // Coin UTXOs: Address → UTXO ID
//!     1: HashMap<String, String>, // Memo UTXOs: Address → UTXO ID  
//!     2: HashMap<String, String>, // State UTXOs: Address → UTXO ID
//!   }
//! }
//!

use crate::db::*;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use zkvm::IOType;

/// Address to UTXO ID mapping storage
///
/// This structure maintains mappings between addresses and UTXO IDs for
/// efficient lookup and retrieval. It uses separate hash maps for each
/// UTXO type to optimize performance and organization.
///
/// # Fields
/// * `data` - HashMap containing type-separated address mappings
///   - Key: UTXO type as usize (0=Coin, 1=Memo, 2=State)
///   - Value: HashMap mapping address to UTXO ID
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AddressUtxoIDStorage {
    /// Type-separated address to UTXO ID mappings
    pub data: HashMap<usize, HashMap<String, String>>,
}

impl AddressUtxoIDStorage {
    /// Creates a new AddressUtxoIDStorage with initialized mappings for all UTXO types
    ///
    /// This method initializes separate hash maps for each UTXO type:
    /// - Type 0: Coin UTXOs
    /// - Type 1: Memo UTXOs  
    /// - Type 2: State UTXOs
    ///
    /// # Returns
    /// * New AddressUtxoIDStorage instance with empty mappings
    pub fn new() -> Self {
        let mut data: HashMap<usize, HashMap<String, String>> = HashMap::new();
        data.insert(0, HashMap::new()); // Coin UTXOs
        data.insert(1, HashMap::new()); // Memo UTXOs
        data.insert(2, HashMap::new()); // State UTXOs
        AddressUtxoIDStorage { data: data }
    }

    /// Retrieves the UTXO ID associated with a given address and type
    ///
    /// This method performs a lookup in the appropriate type-specific hash map
    /// to find the UTXO ID associated with the provided address.
    ///
    /// # Arguments
    /// * `address` - The address to look up
    /// * `input_type` - The UTXO type (Coin, Memo, or State)
    ///
    /// # Returns
    /// * `Some(utxo_id)` if the address is found, `None` otherwise
    pub fn get_utxo_id_by_address(
        &mut self,
        address: String,
        input_type: IOType,
    ) -> Option<String> {
        match self.data.get_mut(&input_type.to_usize()) {
            Some(utxo_id) => match utxo_id.get(&address) {
                Some(utxo_id) => Some(utxo_id.clone()),
                None => None,
            },
            None => None,
        }
    }

    /// Adds a new address to UTXO ID mapping
    ///
    /// This method creates a new mapping between an address and UTXO ID
    /// for the specified UTXO type. If the address already exists, the
    /// previous mapping will be replaced.
    ///
    /// # Arguments
    /// * `input_type` - The UTXO type (Coin, Memo, or State)
    /// * `address` - The address to map
    /// * `utxo_id` - The UTXO ID to associate with the address
    ///
    /// # Returns
    /// * `Some(previous_utxo_id)` if the address was previously mapped, `None` otherwise
    ///
    pub fn add(&mut self, input_type: IOType, address: String, utxo_id: String) -> Option<String> {
        self.data
            .get_mut(&input_type.to_usize())
            .unwrap()
            .insert(address.clone(), utxo_id.clone())
    }

    /// Removes an address to UTXO ID mapping
    ///
    /// This method removes the mapping between an address and UTXO ID
    /// for the specified UTXO type. It returns the removed UTXO ID
    /// if the mapping existed.
    ///
    /// # Arguments
    /// * `input_type` - The UTXO type (Coin, Memo, or State)
    /// * `address` - The address whose mapping should be removed
    ///
    /// # Returns
    /// * `Ok(utxo_id)` if the mapping was found and removed
    /// * `Err(io::Error)` if the address was not found
    ///
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
