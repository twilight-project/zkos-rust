use base64::prelude::*;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockError {
    pub code: i64,
    pub message: String,
    pub details: Vec<String>,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockRaw {
    #[serde(rename = "block_id")]
    pub block_id: BlockId,
    pub block: Block1,
}

impl BlockRaw {
    pub fn get_txid(&mut self) -> Vec<String> {
        let mut txid_vec: Vec<String> = Vec::new();
        let txs = self.block.data.txs.clone();
        for i in 0..txs.len() {
            let tx_decode = BASE64_STANDARD.decode(txs[i].clone()).unwrap();
            let mut sha256 = Sha256::new();
            sha256.update(tx_decode.clone());
            let result = sha256.finalize();
            // println!("tx data bytes : {:?}", hex::encode(result));
            txid_vec.push(hex::encode(result));
        }
        txid_vec
    }
    pub fn get_tx_byte_code(&mut self) -> Vec<String> {
        self.block.data.txs.clone()
    }
    pub fn get_tx_byte_code_with_txhash(&mut self) -> Vec<(String, String)> //byte_code, txhash
    {
        let mut txid_vec: Vec<(String, String)> = Vec::new();

        let txs = self.block.data.txs.clone();
        for tx in txs {
            let tx_decode = BASE64_STANDARD.decode(tx.clone()).unwrap();
            let mut sha256 = Sha256::new();
            sha256.update(tx_decode.clone());
            let result = sha256.finalize();
            // println!("tx data bytes : {:?}", hex::encode(result));
            txid_vec.push((tx, hex::encode(result)));
        }
        txid_vec
    }
    pub fn get_block_hash(&mut self) -> String {
        self.block_id.hash.clone()
    }
    pub fn get_block_height(&mut self) -> u64 {
        self.block.header.height.clone()
    }
    pub fn get_latest_block_height() -> Result<u64, String> {
        let url = format!(
            "{}/cosmos/base/tendermint/v1beta1/blocks/latest",
            *NYKS_BLOCK_SUBSCRIBER_URL
        );
        // println!("url :{:?}", url);
        match request_url(&url) {
            Ok(block_data) => {
                let mut block = BlockRaw::decode(block_data).unwrap();
                Ok(block.get_block_height())
            }
            Err(arg) => return Err(arg.to_string()),
        }
    }
    pub fn get_block_data_from_height(block_height: u64) -> Result<BlockRaw, String> {
        let url = format!(
            "{}/cosmos/base/tendermint/v1beta1/blocks/{}",
            *NYKS_BLOCK_SUBSCRIBER_URL, block_height,
        );
        match request_url(&url) {
            Ok(block_data) => match BlockRaw::decode(block_data) {
                Ok(block) => Ok(block),
                Err(arg) => return Err(arg.to_string()),
            },
            Err(arg) => return Err(arg.to_string()),
        }
    }
    pub fn convert_to_zkos_block(&mut self) {}

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
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockId {
    pub hash: String,
    #[serde(rename = "part_set_header")]
    pub part_set_header: PartSetHeader,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartSetHeader {
    pub total: i64,
    pub hash: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block1 {
    pub header: Header,
    pub data: Data,
    pub evidence: Evidence,
    #[serde(rename = "last_commit")]
    pub last_commit: LastCommit,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    pub version: Version,
    #[serde(rename = "chain_id")]
    pub chain_id: String,
    #[serde(rename = "height", deserialize_with = "string_to_u64")]
    pub height: u64,
    pub time: String,
    #[serde(rename = "last_block_id")]
    pub last_block_id: LastBlockId,
    #[serde(rename = "last_commit_hash")]
    pub last_commit_hash: String,
    #[serde(rename = "data_hash")]
    pub data_hash: String,
    #[serde(rename = "validators_hash")]
    pub validators_hash: String,
    #[serde(rename = "next_validators_hash")]
    pub next_validators_hash: String,
    #[serde(rename = "consensus_hash")]
    pub consensus_hash: String,
    #[serde(rename = "app_hash")]
    pub app_hash: String,
    #[serde(rename = "last_results_hash")]
    pub last_results_hash: String,
    #[serde(rename = "evidence_hash")]
    pub evidence_hash: String,
    #[serde(rename = "proposer_address")]
    pub proposer_address: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub block: String,
    pub app: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LastBlockId {
    pub hash: String,
    #[serde(rename = "part_set_header")]
    pub part_set_header: PartSetHeader2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartSetHeader2 {
    pub total: i64,
    pub hash: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub txs: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Evidence {
    pub evidence: Vec<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LastCommit {
    pub height: String,
    pub round: i64,
    #[serde(rename = "block_id")]
    pub block_id: BlockId2,
    pub signatures: Vec<Signature>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockId2 {
    pub hash: String,
    #[serde(rename = "part_set_header")]
    pub part_set_header: PartSetHeader3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartSetHeader3 {
    pub total: i64,
    pub hash: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Signature {
    #[serde(rename = "block_id_flag")]
    pub block_id_flag: String,
    #[serde(rename = "validator_address")]
    pub validator_address: String,
    pub timestamp: String,
    pub signature: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionMessage {
    pub tx_type: String,
    pub tx_id: String,
    pub tx_byte_code: Option<String>,
    pub zk_oracle_address: Option<String>,
    pub mint_or_burn: Option<bool>, // Optional because it's not present in all types.
    pub btc_value: Option<String>,
    pub qq_account: Option<String>,
    pub encrypt_scalar: Option<String>,
    pub twilight_address: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionMessageRaW {
    #[serde(rename = "@type")]
    pub tx_type: String,
    #[serde(rename = "txId")]
    pub tx_id: Option<String>,
    #[serde(rename = "txByteCode")]
    pub tx_byte_code: Option<String>,
    #[serde(rename = "zkOracleAddress")]
    pub zk_oracle_address: Option<String>,
    #[serde(rename = "mintOrBurn")]
    pub mint_or_burn: Option<bool>, // Optional because it's not present in all types.
    #[serde(rename = "btcValue")]
    pub btc_value: Option<String>,
    #[serde(rename = "qqAccount")]
    pub qq_account: Option<String>,
    #[serde(rename = "encryptScalar")]
    pub encrypt_scalar: Option<String>,
    #[serde(rename = "twilightAddress")]
    pub twilight_address: Option<String>,
}
impl TransactionMessageRaW {
    pub fn to_tx_msg(&self, tx_hash: String) -> TransactionMessage {
        let tx_msg_raw = self.clone();
        TransactionMessage {
            tx_type: tx_msg_raw.tx_type,
            tx_id: match tx_msg_raw.tx_id {
                Some(tx_id) => tx_id,
                None => tx_hash,
            },
            tx_byte_code: tx_msg_raw.tx_byte_code,
            zk_oracle_address: tx_msg_raw.zk_oracle_address,
            mint_or_burn: tx_msg_raw.mint_or_burn,
            btc_value: tx_msg_raw.btc_value,
            qq_account: tx_msg_raw.qq_account,
            encrypt_scalar: tx_msg_raw.encrypt_scalar,
            twilight_address: tx_msg_raw.twilight_address,
        }
    }
}
impl TransactionMessage {
    pub fn new(txid_hash: String, tx_byte_code: String) -> Vec<TransactionMessage> {
        let tx_data_result = TxRaw::get_transaction_from_chain_by_txhash(txid_hash.clone());
        let mut tx_msg_vec: Vec<TransactionMessage> = Vec::new();
        match tx_data_result {
            Ok(tx_data) => {
                if tx_data.tx_response.code == 0 {
                    for mut msg in tx_data.tx.body.messages {
                        //is it required?
                        msg.tx_byte_code = match msg.tx_byte_code {
                            Some(bytes_code) => Some(bytes_code),
                            None => Some(tx_byte_code.clone()),
                        };
                        tx_msg_vec.push(msg.to_tx_msg(txid_hash.clone()));
                    }
                }
            }
            Err(_arg) => return Vec::new(),
        }
        tx_msg_vec
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    #[serde(rename = "Blockhash")]
    pub block_hash: String,
    #[serde(rename = "Blockheight", deserialize_with = "string_to_u64")]
    pub block_height: u64,
    #[serde(rename = "Transactions")]
    pub transactions: Vec<TransactionMessage>,
}
impl Block {
    pub fn new(mut block_raw: BlockRaw) -> Self {
        let mut transactions: Vec<TransactionMessage> = Vec::new();
        let txid_hash_vec = block_raw.get_tx_byte_code_with_txhash();
        for (tx_byte_code, tx_id) in txid_hash_vec {
            let mut tx_msg_vec = TransactionMessage::new(tx_id, tx_byte_code);
            transactions.append(&mut tx_msg_vec);
        }
        Block {
            block_hash: block_raw.get_block_hash(),
            block_height: block_raw.get_block_height(),
            transactions,
        }
    }
    pub fn get_local_block_height() -> u64 {
        let block_height: u64 = match fs::read_to_string("height.txt") {
            Ok(block_height_str) => match block_height_str.trim().parse::<u64>() {
                Ok(block_height) => block_height,
                Err(_) => {
                    eprintln!("Failed to parse block height");
                    0
                }
            },
            Err(e) => {
                eprintln!("Failed to read block height: {}", e);
                0
            }
        };
        block_height
    }
    pub fn write_local_block_height(block_height: u64) {
        match fs::write("height.txt", block_height.to_string()) {
            Ok(_) => {}
            Err(e) => eprintln!("Failed to write block height: {}", e),
        }
    }
}
use serde::{
    de::{self, Visitor},
    Deserializer,
};
use std::fmt;
use std::fs;

use crate::pubsub_chain::request_url;
use crate::TxRaw;
use crate::NYKS_BLOCK_SUBSCRIBER_URL;
pub fn string_to_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringVisitor;

    impl<'de> Visitor<'de> for StringVisitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string representation for u64")
        }

        fn visit_str<E: de::Error>(self, value: &str) -> Result<u64, E> {
            value.parse::<u64>().map_err(E::custom)
        }
    }
    deserializer.deserialize_str(StringVisitor)
}
