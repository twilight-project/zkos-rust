use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;

use crate::block_types::BlockError;
use crate::block_types::TransactionMessageRaW;
use crate::pubsub_chain::request_url;
use crate::NYKS_BLOCK_SUBSCRIBER_URL;
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxRaw {
    pub tx: Tx,
    #[serde(rename = "tx_response")]
    pub tx_response: TxResponse,
}
impl TxRaw {
    pub fn decode(json: String) -> Result<Self, String> {
        match serde_json::from_str(&json) {
            Ok(block) => Ok(block),
            Err(arg) => {
                let block_error: BlockError = match serde_json::from_str(&json) {
                    Ok(block_error_result) => block_error_result,
                    Err(arg) => return Err(arg.to_string()),
                };
                if block_error.code == 3 {
                    return Err("3".to_string());
                }

                Err(arg.to_string())
            }
        }
    }
    pub fn get_msg(&mut self) -> Vec<TransactionMessageRaW> {
        self.tx.body.messages.clone()
    }
    pub fn get_transaction_from_chain_by_txhash(txhash: String) -> Result<TxRaw, String> {
        let url = format!(
            "{}/cosmos/tx/v1beta1/txs/{}",
            *NYKS_BLOCK_SUBSCRIBER_URL, txhash,
        );

        match request_url(&url) {
            Ok(block_data) => match TxRaw::decode(block_data) {
                Ok(tx) => Ok(tx),
                Err(arg) => return Err(arg.to_string()),
            },
            Err(arg) => return Err(arg.to_string()),
        }
    }
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tx {
    pub body: Body,
    #[serde(rename = "auth_info")]
    pub auth_info: AuthInfo,
    pub signatures: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Body {
    pub messages: Vec<TransactionMessageRaW>,
    pub memo: String,
    #[serde(rename = "timeout_height")]
    pub timeout_height: String,
    #[serde(rename = "extension_options")]
    pub extension_options: Vec<Value>,
    #[serde(rename = "non_critical_extension_options")]
    pub non_critical_extension_options: Vec<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    #[serde(rename = "@type")]
    pub type_field: String,
    pub mint_or_burn: Option<bool>,
    pub btc_value: Option<String>,
    pub qq_account: Option<String>,
    pub encrypt_scalar: Option<String>,
    pub twilight_address: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthInfo {
    #[serde(rename = "signer_infos")]
    pub signer_infos: Vec<SignerInfo>,
    pub fee: Fee,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignerInfo {
    #[serde(rename = "public_key")]
    pub public_key: PublicKey,
    #[serde(rename = "mode_info")]
    pub mode_info: ModeInfo,
    pub sequence: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicKey {
    #[serde(rename = "@type")]
    pub type_field: String,
    pub key: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModeInfo {
    pub single: Single,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Single {
    pub mode: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fee {
    pub amount: Vec<Amount>,
    #[serde(rename = "gas_limit")]
    pub gas_limit: String,
    pub payer: String,
    pub granter: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Amount {
    pub denom: String,
    pub amount: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxResponse {
    pub height: String,
    pub txhash: String,
    pub codespace: String,
    pub code: i64,
    pub data: String,
    #[serde(rename = "raw_log")]
    pub raw_log: String,
    pub logs: Vec<Log>,
    pub info: String,
    #[serde(rename = "gas_wanted")]
    pub gas_wanted: String,
    #[serde(rename = "gas_used")]
    pub gas_used: String,
    pub tx: Tx2,
    pub timestamp: String,
    pub events: Vec<Event2>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Log {
    #[serde(rename = "msg_index")]
    pub msg_index: i64,
    pub log: String,
    pub events: Vec<Event>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    #[serde(rename = "type")]
    pub type_field: String,
    pub attributes: Vec<Attribute>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attribute {
    pub key: String,
    pub value: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tx2 {
    #[serde(rename = "@type")]
    pub type_field: String,
    pub body: Body2,
    #[serde(rename = "auth_info")]
    pub auth_info: AuthInfo2,
    pub signatures: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Body2 {
    pub messages: Vec<Message2>,
    pub memo: String,
    #[serde(rename = "timeout_height")]
    pub timeout_height: String,
    #[serde(rename = "extension_options")]
    pub extension_options: Vec<Value>,
    #[serde(rename = "non_critical_extension_options")]
    pub non_critical_extension_options: Vec<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message2 {
    #[serde(rename = "@type")]
    pub type_field: String,
    pub mint_or_burn: Option<bool>,
    pub btc_value: Option<String>,
    pub qq_account: Option<String>,
    pub encrypt_scalar: Option<String>,
    pub twilight_address: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthInfo2 {
    #[serde(rename = "signer_infos")]
    pub signer_infos: Vec<SignerInfo2>,
    pub fee: Fee2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignerInfo2 {
    #[serde(rename = "public_key")]
    pub public_key: PublicKey2,
    #[serde(rename = "mode_info")]
    pub mode_info: ModeInfo2,
    pub sequence: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicKey2 {
    #[serde(rename = "@type")]
    pub type_field: String,
    pub key: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModeInfo2 {
    pub single: Single2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Single2 {
    pub mode: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fee2 {
    pub amount: Vec<Amount2>,
    #[serde(rename = "gas_limit")]
    pub gas_limit: String,
    pub payer: String,
    pub granter: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Amount2 {
    pub denom: String,
    pub amount: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event2 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub attributes: Vec<Attribute2>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attribute2 {
    pub key: String,
    pub value: String,
    pub index: bool,
}

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Message {
//     pub key: String,
//     pub value: String,
//     pub index: bool,
// }

// type Message struct {
// 	Type            string `json:"@type"`
// 	TxId            string
// 	TxByteCode      string
// 	ZkOracleAddress string
// 	MintOrBurn      bool
// 	BtcValue        string
// 	QqAccount       string
// 	EncryptScalar   string
// 	TwilightAddress string
// }
