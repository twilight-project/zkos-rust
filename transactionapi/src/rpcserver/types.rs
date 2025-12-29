//! Shared types for RPC operations in ZkOS Transaction API.
//!
//! This module contains the shared types used across the RPC server implementation,
//! including request and response structures for various methods.
//!
//! ## Types
//!
//! - `MintOrBurnTx`: Request structure for minting or burning transactions
//! - `UtxoRequest`: Request structure for UTXO queries
//! - `UtxoDetailResponse`: Response structure for UTXO details

use serde::{Deserialize, Serialize};
use zkvm::IOType;
use zkvm::Utxo;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintOrBurnTx {
    // value in satoshi
    pub btc_value: u64,
    // hex string
    pub qq_account: String,
    // hex string
    pub encrypt_scalar: String,
    // hex string
    pub twilight_address: String,
}
/// Request structure for UTXO queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoRequest {
    /// Address or ID
    pub address_or_id: String,
    /// Input type
    pub input_type: IOType,
}

/// Response structure for UTXO details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoDetailResponse {
    /// UTXO ID
    pub id: Utxo,
    /// UTXO output
    pub output: zkvm::Output,
}
/// Response structure for UTXO details
impl UtxoDetailResponse {
    pub fn new(id: Utxo, output: zkvm::Output) -> Self {
        UtxoDetailResponse { id, output }
    }
}
