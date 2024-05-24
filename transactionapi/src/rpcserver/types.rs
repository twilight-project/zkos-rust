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

// #[derive(Serialize, Deserialize, Clone, Debug)]
// pub struct ZkOsAccount {
//     pub address: String,
//     pub encrypt: ElGamalCommitment,
// }

// impl ZkOsAccount {
//     pub fn new(address: String, encrypt: ElGamalCommitment) -> Self {
//         Self { address, encrypt }
//     }

//}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoRequest {
    pub address_or_id: String,
    pub input_type: IOType,
}
