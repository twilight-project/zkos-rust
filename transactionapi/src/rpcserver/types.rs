
use std::process::Output;

use quisquislib::elgamal::ElGamalCommitment;
use serde::{Deserialize, Serialize};
use zkvm::IOType;
use zkvm::{Input, Utxo};
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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum IOType {
    COIN,
    MEMO,
    STATE,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UtxoArgs {
    pub address: String,
    pub io_type: IOType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoRequest {
    pub address_or_id: String,
    pub input_type: IOType,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoDetailResponse {
    pub id: Utxo,
    pub output: zkvm::Output,
}
impl UtxoDetailResponse {
    pub fn new(id: Utxo, output: zkvm::Output) -> Self {
        UtxoDetailResponse { id, output }
    }

}
