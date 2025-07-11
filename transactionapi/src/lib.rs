pub mod rpcclient;
pub mod rpcserver;
pub mod error;
#[macro_use]
extern crate lazy_static;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TransactionStatusId {
    pub txid: String,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TxResponse {
    pub response: String,
}
impl TxResponse {
    pub fn new(response: String) -> Self {
        TxResponse { response: response }
    }
}
