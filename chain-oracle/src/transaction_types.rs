//! Types and helpers for parsing and representing Cosmos transaction data.
//!
//! This module provides Rust structs for deserializing transaction data from
//! Cosmos-based blockchains, as well as helpers for extracting and working with this data.
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;

use crate::block_types::BlockError;
use crate::block_types::TransactionMessageRaW;
use crate::pubsub_chain::request_url;
use crate::NYKS_BLOCK_SUBSCRIBER_URL;
/// Represents a raw transaction as returned by the Cosmos RPC API.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxRaw {
    /// The transaction body, including messages and memo.
    pub tx: Tx,
    /// The transaction response metadata.
    #[serde(rename = "tx_response")]
    pub tx_response: TxResponse,
}
impl TxRaw {
    /// Decodes a JSON string into a `TxRaw` struct.
    ///
    /// # Arguments
    /// * `json` - The JSON string to decode.
    ///
    /// # Returns
    /// - `Ok(TxRaw)` if decoding is successful.
    /// - `Err(String)` if decoding fails or the response is an error.
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
    /// Retrieves the messages from the transaction body.
    ///
    /// # Returns
    /// A vector of `TransactionMessageRaW` structs representing the messages in the transaction.
    pub fn get_msg(&mut self) -> Vec<TransactionMessageRaW> {
        self.tx.body.messages.clone()
    }
    /// Retrieves a transaction by its transaction hash from the blockchain.
    ///
    /// # Arguments
    /// * `txhash` - The hexadecimal hash of the transaction.
    ///
    /// # Returns
    /// - `Ok(TxRaw)` if the transaction is found.
    /// - `Err(String)` if an error occurs during the request or decoding.
    pub fn get_transaction_from_chain_by_txhash(txhash: String) -> Result<TxRaw, String> {
        let url = format!(
            "{}/cosmos/tx/v1beta1/txs/{}",
            *NYKS_BLOCK_SUBSCRIBER_URL, txhash,
        );

        match request_url(&url) {
            Ok(block_data) => match TxRaw::decode(block_data) {
                Ok(tx) => Ok(tx),
                Err(arg) => Err(arg.to_string()),
            },
            Err(arg) => Err(arg.to_string()),
        }
    }
}
/// Represents the body of a transaction, including messages, memo, and timeout height.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tx {
    /// The transaction body, including messages and memo.
    pub body: Body,
    /// The authentication information for the transaction.
    #[serde(rename = "auth_info")]
    pub auth_info: AuthInfo,
    /// The signatures of the transaction.
    pub signatures: Vec<String>,
}

/// Represents the body of a transaction, including messages, memo, and timeout height.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Body {
    /// The messages included in the transaction.
    pub messages: Vec<TransactionMessageRaW>,
    /// A memo attached to the transaction.
    pub memo: String,
    /// The height at which the transaction will timeout.
    #[serde(rename = "timeout_height")]
    pub timeout_height: String,
    /// Extension options for the transaction.
    #[serde(rename = "extension_options")]
    pub extension_options: Vec<Value>,
    /// Non-critical extension options for the transaction.
    #[serde(rename = "non_critical_extension_options")]
    pub non_critical_extension_options: Vec<Value>,
}

/// Represents a single message within a transaction.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    /// The type of the message.
    #[serde(rename = "@type")]
    pub type_field: String,
    /// Indicates if the message is a mint or burn operation.
    pub mint_or_burn: Option<bool>,
    /// The Bitcoin value associated with the message.
    pub btc_value: Option<String>,
    /// The QQ account associated with the message.
    pub qq_account: Option<String>,
    /// The scalar used for encryption.
    pub encrypt_scalar: Option<String>,
    /// The twilight address associated with the message.
    pub twilight_address: Option<String>,
}

/// Represents the authentication information for a transaction, including signer infos and fee.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthInfo {
    /// The information about the signers.
    #[serde(rename = "signer_infos")]
    pub signer_infos: Vec<SignerInfo>,
    /// The fee for the transaction.
    pub fee: Fee,
}

/// Represents the information about a signer.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignerInfo {
    /// The public key of the signer.
    #[serde(rename = "public_key")]
    pub public_key: PublicKey,
    /// The mode information for the signer.
    #[serde(rename = "mode_info")]
    pub mode_info: ModeInfo,
    /// The sequence number of the signer.
    pub sequence: String,
}

/// Represents a public key.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicKey {
    /// The type of the public key.
    #[serde(rename = "@type")]
    pub type_field: String,
    /// The actual key data.
    pub key: String,
}

/// Represents the mode information for a signer.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModeInfo {
    /// The single mode.
    pub single: Single,
}

/// Represents the single mode.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Single {
    /// The mode.
    pub mode: String,
}

/// Represents the fee for a transaction, including amount, gas limit, payer, and granter.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fee {
    /// The amount of coins to be paid.
    pub amount: Vec<Amount>,
    /// The maximum gas limit for the transaction.
    #[serde(rename = "gas_limit")]
    pub gas_limit: String,
    /// The payer of the fee.
    pub payer: String,
    /// The granter of the fee.
    pub granter: String,
}

/// Represents an amount of coins.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Amount {
    /// The denomination of the coin.
    pub denom: String,
    /// The amount of the coin.
    pub amount: String,
}

/// Represents the response metadata for a transaction.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxResponse {
    /// The height of the block containing the transaction.
    pub height: String,
    /// The hash of the transaction.
    pub txhash: String,
    /// The codespace of the transaction.
    pub codespace: String,
    /// The code of the transaction.
    pub code: i64,
    /// The data associated with the transaction.
    pub data: String,
    /// The raw log of the transaction.
    #[serde(rename = "raw_log")]
    pub raw_log: String,
    /// The logs associated with the transaction.
    pub logs: Vec<Log>,
    /// The info field of the transaction.
    pub info: String,
    /// The gas wanted for the transaction.
    #[serde(rename = "gas_wanted")]
    pub gas_wanted: String,
    /// The gas used for the transaction.
    #[serde(rename = "gas_used")]
    pub gas_used: String,
    /// The transaction details.
    pub tx: Tx2,
    /// The timestamp of the transaction.
    pub timestamp: String,
    /// The events associated with the transaction.
    pub events: Vec<Event2>,
}

/// Represents a log entry for a transaction.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Log {
    /// The index of the message in the transaction.
    #[serde(rename = "msg_index")]
    pub msg_index: i64,
    /// The log message.
    pub log: String,
    /// The events associated with the log.
    pub events: Vec<Event>,
}

/// Represents an event.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    /// The type of the event.
    #[serde(rename = "type")]
    pub type_field: String,
    /// The attributes of the event.
    pub attributes: Vec<Attribute>,
}

/// Represents an attribute.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attribute {
    /// The key of the attribute.
    pub key: String,
    /// The value of the attribute.
    pub value: String,
}

/// Represents the transaction details.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tx2 {
    /// The type of the transaction.
    #[serde(rename = "@type")]
    pub type_field: String,
    /// The transaction body.
    pub body: Body2,
    /// The authentication information for the transaction.
    #[serde(rename = "auth_info")]
    pub auth_info: AuthInfo2,
    /// The signatures of the transaction.
    pub signatures: Vec<String>,
}

/// Represents the transaction body.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Body2 {
    /// The messages included in the transaction.
    pub messages: Vec<Message2>,
    /// A memo attached to the transaction.
    pub memo: String,
    /// The height at which the transaction will timeout.
    #[serde(rename = "timeout_height")]
    pub timeout_height: String,
    /// Extension options for the transaction.
    #[serde(rename = "extension_options")]
    pub extension_options: Vec<Value>,
    /// Non-critical extension options for the transaction.
    #[serde(rename = "non_critical_extension_options")]
    pub non_critical_extension_options: Vec<Value>,
}

/// Represents a single message within a transaction.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message2 {
    /// The type of the message.
    #[serde(rename = "@type")]
    pub type_field: String,
    /// Indicates if the message is a mint or burn operation.
    pub mint_or_burn: Option<bool>,
    /// The Bitcoin value associated with the message.
    pub btc_value: Option<String>,
    /// The QQ account associated with the message.
    pub qq_account: Option<String>,
    /// The scalar used for encryption.
    pub encrypt_scalar: Option<String>,
    /// The twilight address associated with the message.
    pub twilight_address: Option<String>,
}

/// Represents the authentication information for a transaction, including signer infos and fee.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthInfo2 {
    /// The information about the signers.
    #[serde(rename = "signer_infos")]
    pub signer_infos: Vec<SignerInfo2>,
    /// The fee for the transaction.
    pub fee: Fee2,
}

/// Represents the information about a signer.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignerInfo2 {
    /// The public key of the signer.
    #[serde(rename = "public_key")]
    pub public_key: PublicKey2,
    /// The mode information for the signer.
    #[serde(rename = "mode_info")]
    pub mode_info: ModeInfo2,
    /// The sequence number of the signer.
    pub sequence: String,
}

/// Represents a public key.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicKey2 {
    /// The type of the public key.
    #[serde(rename = "@type")]
    pub type_field: String,
    /// The actual key data.
    pub key: String,
}

/// Represents the mode information for a signer.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModeInfo2 {
    /// The single mode.
    pub single: Single2,
}

/// Represents the single mode.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Single2 {
    /// The mode.
    pub mode: String,
}

/// Represents the fee for a transaction, including amount, gas limit, payer, and granter.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fee2 {
    /// The amount of coins to be paid.
    pub amount: Vec<Amount2>,
    /// The maximum gas limit for the transaction.
    #[serde(rename = "gas_limit")]
    pub gas_limit: String,
    /// The payer of the fee.
    pub payer: String,
    /// The granter of the fee.
    pub granter: String,
}

/// Represents an amount of coins.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Amount2 {
    /// The denomination of the coin.
    pub denom: String,
    /// The amount of the coin.
    pub amount: String,
}

/// Represents an event.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event2 {
    /// The type of the event.
    #[serde(rename = "type")]
    pub type_field: String,
    /// The attributes of the event.
    pub attributes: Vec<Attribute2>,
}

/// Represents an attribute.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attribute2 {
    /// The key of the attribute.
    pub key: String,
    /// The value of the attribute.
    pub value: String,
    /// Indicates if the attribute is an index.
    pub index: bool,
}
