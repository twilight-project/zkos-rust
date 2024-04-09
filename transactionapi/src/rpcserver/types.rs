
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputArgs {
    pub tx_id: String,
    pub vout: u32,
    pub io_type: IOType,
}
