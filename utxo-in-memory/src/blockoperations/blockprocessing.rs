#![allow(non_snake_case)]
#![allow(missing_docs)]
#![allow(warnings)]
//! Block processing to update Utxo set.

use crate::db::*;
/***************** POstgreSQL Insert Code *********/
use crate::pgsql::{PGSQLDataInsert, PGSQLTransaction, THREADPOOL_SQL_QUEUE};
/**************** POstgreSQL Insert Code End **********/

use crate::ADDRESS_TO_UTXO;
use crate::UTXO_STORAGE;
use address::{Address, Network};
use chain_oracle::Block;
use chain_oracle::TransactionMessage;
use hex;
use quisquislib::elgamal::elgamal::ElGamalCommitment;
use rand::Rng;
use serde::de::{self, Deserializer, Visitor};
use serde_derive::{Deserialize, Serialize};
use serde_ini;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io::Write;
use transaction::reference_tx::{
    convert_output_to_input, create_dark_reference_tx_for_utxo_test, RecordUtxo,
};

use transaction::{ScriptTransaction, Transaction, TransactionData, TransactionType};
use zkvm::constraints::Commitment;
use zkvm::tx::TxID;
use zkvm::zkos_types::{
    IOType, Input, Output, OutputCoin, OutputData, OutputMemo, OutputState, Utxo,
};
use zkvm::Hash;

use curve25519_dalek::ristretto::CompressedRistretto;
use curve25519_dalek::scalar::Scalar;
use prometheus::{register_counter, register_gauge, Counter, Encoder, Gauge, TextEncoder};
use quisquislib::{accounts::Account, ristretto::RistrettoSecretKey};

lazy_static! {
    pub static ref TOTAL_DARK_SATS_MINTED: Gauge =
        register_gauge!("dark_sats_minted", "A counter for dark Sats minted").unwrap();
    pub static ref TOTAL_TRANSFER_TX: Gauge =
        register_gauge!("transfer_tx_count", "A counter for transfer tx").unwrap();
    pub static ref TOTAL_SCRIPT_TX: Gauge =
        register_gauge!("script_tx_count", "A counter for script tx").unwrap();
}

#[derive(Debug, Deserialize)]
struct TelemetryStats {
    total_dark_sats_minted: u64,
    total_transfer_tx: u64,
    total_script_tx: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BlockResult {
    pub suceess_tx: Vec<TxID>,
    pub failed_tx: Vec<TxID>,
}
impl BlockResult {
    pub fn new() -> Self {
        BlockResult {
            suceess_tx: Vec::new(),
            failed_tx: Vec::new(),
        }
    }
}

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct Block {
//     #[serde(rename = "Blockhash")]
//     pub block_hash: String,
//     #[serde(rename = "Blockheight", deserialize_with = "string_to_u64")]
//     pub block_height: u64,
//     #[serde(rename = "Transactions")]
//     pub transactions: Vec<TransactionMessage>,
// }

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct TransactionMessageTransfer {
//     #[serde(rename = "@type")]
//     pub tx_type: String,
//     #[serde(rename = "TxId")]
//     pub tx_id: String,
//     #[serde(rename = "TxByteCode")]
//     pub tx_byte_code: String,
//     #[serde(rename = "ZkOracleAddress")]
//     pub zk_oracle_address: String,
// }

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct TransactionMessageTrading {
//     #[serde(rename = "@type")]
//     pub tx_type: String,
//     #[serde(rename = "MintOrBurn")]
//     pub mint_or_burn: bool,
//     #[serde(rename = "BtcValue")]
//     pub btc_value: u32,
//     #[serde(rename = "QqAccount")]
//     pub qq_account: String,
//     #[serde(rename = "EncryptScalar")]
//     pub encrypt_scalar: u64,
//     #[serde(rename = "TwilightAddress")]
//     pub twilight_address: String,
//     #[serde(rename = "TxId")]
//     pub tx_id: String,
// }

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct TransactionMessage {
//     #[serde(rename = "@type")]
//     pub tx_type: String,
//     #[serde(rename = "TxId")]
//     pub tx_id: String,
//     #[serde(rename = "TxByteCode")]
//     pub tx_byte_code: Option<String>,
//     #[serde(rename = "ZkOracleAddress")]
//     pub zk_oracle_address: Option<String>,
//     #[serde(rename = "MintOrBurn")]
//     pub mint_or_burn: Option<bool>, // Optional because it's not present in all types.
//     #[serde(rename = "BtcValue")]
//     pub btc_value: Option<String>,
//     #[serde(rename = "QqAccount")]
//     pub qq_account: Option<String>,
//     #[serde(rename = "EncryptScalar")]
//     pub encrypt_scalar: Option<String>,
//     #[serde(rename = "TwilightAddress")]
//     pub twilight_address: Option<String>,
// }

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub enum MessageType {
//     #[serde(rename = "/twilightproject.nyks.zkos.MsgMintBurnTradingBtc")]
//     Trading(TransactionMessage),
//     #[serde(rename = "/twilightproject.nyks.zkos.MsgTransferTx")]
//     Transfer(TransactionMessage),
// }

pub fn read_telemetry_stats_from_file() -> Result<(), Box<dyn std::error::Error>> {
    let contents = fs::read_to_string("telemetry.ini")?;
    let config: TelemetryStats = serde_ini::from_str(&contents)?;

    TOTAL_DARK_SATS_MINTED.set(config.total_dark_sats_minted as f64);
    TOTAL_TRANSFER_TX.set(config.total_transfer_tx as f64);
    TOTAL_SCRIPT_TX.set(config.total_script_tx as f64);

    Ok(())
}

fn write_telemetry_stats_to_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create("telemetry.ini")?;
    write!(
        file,
        "total_dark_sats_minted={}\n",
        TOTAL_DARK_SATS_MINTED.get()
    )?;
    write!(file, "total_transfer_tx={}\n", TOTAL_TRANSFER_TX.get())?;
    write!(file, "total_script_tx={}\n", TOTAL_SCRIPT_TX.get())?;

    Ok(())
}

fn string_to_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
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

pub fn process_transfer(transaction: TransactionMessage, height: u64, tx_result: &mut BlockResult) {
    let tx_bytes = hex::decode(transaction.tx_byte_code.unwrap()).expect("Decoding failed");
    let transaction_info: Transaction = bincode::deserialize(&tx_bytes).unwrap();
    let tx_id: [u8; 32] = hex::decode(transaction.tx_id.clone())
        .unwrap()
        .try_into()
        .unwrap();
    let tx_input = transaction_info.get_tx_inputs();
    let tx_output = transaction_info.get_tx_outputs();

    let transaction_type = transaction_info.tx_type;

    let utxo_verified = verify_utxo(transaction_info);

    let mut utxo_storage = UTXO_STORAGE.write().unwrap();

    if utxo_verified {
        /***************** POstgreSQL Insert Code *********/
        /************************************************ */
        let mut pg_insert_data = PGSQLTransaction::default();
        pg_insert_data.txid = transaction.tx_id.clone();
        pg_insert_data.block_height = height;

        /**************** POstgreSQL Insert Code End **********/
        /**************************************************** */
        for input in tx_input {
            let utxo_key = bincode::serialize(&input.as_utxo().unwrap()).unwrap();
            let utxo_input_type = input.in_type as usize;
            let utxo_test = Utxo::new(TxID(Hash([0; 32])), 0);
            let utxo = input.as_utxo().unwrap();
            if utxo.to_owned() != utxo_test {
                let _result = utxo_storage.remove(utxo_key.clone(), utxo_input_type);

                //addres to utxo id link
                let mut address_to_utxo_storage = ADDRESS_TO_UTXO.lock().unwrap();
                address_to_utxo_storage.remove(
                    input.in_type.clone(),
                    input.as_owner_address().unwrap().clone(),
                );
                drop(address_to_utxo_storage);
                //
                match _result {
                    Ok(_) => {
                        /***************** POstgreSQL Insert Code *********/
                        /************************************************ */
                        pg_insert_data.remove_utxo.push(utxo_key.clone());
                        /**************** POstgreSQL Insert Code End **********/
                        /**************************************************** */
                        println!("UTXO REMOVED TRANSFER")
                    }
                    Err(err) => {
                        println!("ERROR IN REMOVING UTXO TRANSFER : {}", err)
                    }
                }
            }
        }
        //Add all output
        for (output_index, output_set) in tx_output.iter().enumerate() {
            let utxo_key =
                bincode::serialize(&Utxo::from_hash(Hash(tx_id), output_index as u8)).unwrap();
            let utxo_output_type = output_set.out_type as usize;
            let _result = utxo_storage.add(utxo_key.clone(), output_set.clone(), utxo_output_type);

            // address to utxo id linking ****
            let mut address_to_utxo_storage = ADDRESS_TO_UTXO.lock().unwrap();
            address_to_utxo_storage.add(
                output_set.out_type.clone(),
                output_set.output.get_owner_address().unwrap().clone(),
                hex::encode(utxo_key.clone()),
            );
            drop(address_to_utxo_storage);
            //******* */
            match _result {
                Ok(_) => {
                    /***************** POstgreSQL Insert Code *********/
                    /************************************************ */
                    match utxo_output_type {
                        0 => {
                            pg_insert_data.insert_coin_utxo.push(PGSQLDataInsert::new(
                                utxo_key,
                                bincode::serialize(&output_set).unwrap(),
                                bincode::serialize(output_set.output.get_owner_address().unwrap())
                                    .unwrap(),
                                &"".to_string(),
                                output_index,
                            ));
                            println!("UTXO COIN ADDED DB");
                        }
                        1 => {
                            pg_insert_data.insert_memo_utxo.push(PGSQLDataInsert::new(
                                utxo_key,
                                bincode::serialize(&output_set).unwrap(),
                                bincode::serialize(output_set.output.get_owner_address().unwrap())
                                    .unwrap(),
                                output_set.output.get_script_address().unwrap(),
                                output_index,
                            ));
                            println!("UTXO MEMO ADDED DB");
                        }
                        2 => {
                            pg_insert_data.insert_state_utxo.push(PGSQLDataInsert::new(
                                utxo_key,
                                bincode::serialize(&output_set).unwrap(),
                                bincode::serialize(output_set.output.get_owner_address().unwrap())
                                    .unwrap(),
                                output_set.output.get_script_address().unwrap(),
                                output_index,
                            ));
                            println!("UTXO STATE ADDED DB");
                        }
                        _ => {}
                    }
                    /**************** POstgreSQL Insert Code End **********/
                    /**************************************************** */
                    println!("UTXO ADDED TRANSFER")
                    //add vout here
                }
                Err(err) => {
                    println!("ERROR IN ADDING UTXO TRANSFER : {}", err)
                }
            }
        }

        // let _ = utxo_storage.data_meta_update(height as usize);

        /***************** POstgreSQL Insert Code *********/
        /************************************************ */
        let treadpool_sql_queue = THREADPOOL_SQL_QUEUE.lock().unwrap();
        treadpool_sql_queue.execute(move || {
            let _ = pg_insert_data.update_utxo_log();
        });
        drop(treadpool_sql_queue);
        /**************** POstgreSQL Insert Code End **********/
        /**************************************************** */

        if transaction_type == TransactionType::Script {
            TOTAL_SCRIPT_TX.inc();
            write_telemetry_stats_to_file();
        } else if transaction_type == TransactionType::Transfer {
            TOTAL_TRANSFER_TX.inc();
            write_telemetry_stats_to_file();
        }

        tx_result.suceess_tx.push(TxID(Hash(tx_id)));
    } else {
        tx_result.failed_tx.push(TxID(Hash(tx_id)));
    }
}

pub fn process_trade_mint(
    transaction: TransactionMessage,
    height: u64,
    tx_result: &mut BlockResult,
) {
    println!("In Process trade mint  tx :=:  {:?}", transaction);

    let mut utxo_storage = UTXO_STORAGE.write().unwrap();
    let tx_id = hex::decode(transaction.tx_id.clone()).expect("error decoding tx id");
    let tx_id = TxID(Hash(tx_id.try_into().unwrap()));
    let utxo_key = bincode::serialize(&Utxo::new(tx_id, 0 as u8)).unwrap();
    let mut qq_account_bytes =
        hex::decode(transaction.qq_account.unwrap()).expect("Decoding failed");
    let elgamal = qq_account_bytes.split_off(qq_account_bytes.len() - 64);
    let elgamal = ElGamalCommitment::from_bytes(&elgamal).unwrap();
    let address = address::Standard::from_bytes(&qq_account_bytes[0..69]).unwrap();

    if transaction.mint_or_burn.unwrap() == true {
        //Mint UTXOS
        //let output = OutputData::Coin(OutputCoin{encrypt: elgamal, address:address.as_hex()});
        let output = Output::coin(OutputData::Coin(OutputCoin {
            encrypt: elgamal,
            owner: address.as_hex(),
        }));
        utxo_storage.add(utxo_key.clone(), output.clone(), output.out_type as usize);
        // address to utxo id linking ****
        let mut address_to_utxo_storage = ADDRESS_TO_UTXO.lock().unwrap();
        address_to_utxo_storage.add(
            output.out_type.clone(),
            address.as_hex(),
            hex::encode(utxo_key.clone()),
        );
        drop(address_to_utxo_storage);
        //******* */
        let pk = address.as_hex();
        tx_result.suceess_tx.push(tx_id);

        /***************** POstgreSQL Insert Code *********/
        //*********************************************** */
        let mut pg_insert_data = PGSQLTransaction::default();
        pg_insert_data.txid = transaction.tx_id.clone();
        pg_insert_data.block_height = height;
        //pg_insert_data.io_type = output.out_type as usize;
        pg_insert_data.insert_coin_utxo.push(PGSQLDataInsert::new(
            utxo_key,
            bincode::serialize(&output.clone()).unwrap(),
            bincode::serialize(output.output.get_owner_address().unwrap()).unwrap(),
            &"".to_string(),
            0,
        ));
        let treadpool_sql_queue = THREADPOOL_SQL_QUEUE.lock().unwrap();
        treadpool_sql_queue.execute(move || {
            let _ = pg_insert_data.update_utxo_log();
        });
        drop(treadpool_sql_queue);
        /**************** POstgreSQL Insert Code End **********/
        /**************************************************** */

        let float_value: f64 = match transaction.btc_value.unwrap().parse() {
            Ok(value) => value, // If parsing is successful, use the parsed value
            Err(e) => {
                println!("Failed to convert string to f64: {:?}", e);
                0.0 // Use a default value (like 0.0) in case of an error
            }
        };

        TOTAL_DARK_SATS_MINTED.add(float_value);
        write_telemetry_stats_to_file();
        println!("UTXO ADDED MINT")
    } else if transaction.mint_or_burn.unwrap() == false {
        let float_value: f64 = match transaction.btc_value.unwrap().parse() {
            Ok(value) => value, // If parsing is successful, use the parsed value
            Err(e) => {
                println!("Failed to convert string to f64: {:?}", e);
                0.0 // Use a default value (like 0.0) in case of an error
            }
        };
        TOTAL_DARK_SATS_MINTED.sub(float_value);
        write_telemetry_stats_to_file();
    }
    // UTXO IS ALREADY REMOVED THROUGH THE ZKOS Burn Message TX that appears as Transfer Tx now
    // Therefore no need to do anything for Tendermint Burn Tx.
    // The tx is only needed for the chain to update the twilight balance
    /*else {
        let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
        let input_type = IOType::Coin as usize;
        let utxos = utxo_storage.data.get_mut(&input_type).unwrap();

        //TODO need to fix this and ask a utxo in mint burn message

        let result = utxo_storage.remove(utxo_key.clone(), IOType::Coin as usize);
        if result.is_ok() {
            tx_result.suceess_tx.push(tx_id);

            /***************** POstgreSQL Insert Code *********/
            /************************************************ */
            let mut pg_insert_data = PGSQLTransaction::default();
            pg_insert_data.txid = transaction.tx_id.clone();
            pg_insert_data.block_height = height;
            pg_insert_data.io_type = IOType::Coin as usize;
            pg_insert_data.remove_utxo.push(utxo_key.clone());
            let treadpool_sql_queue = THREADPOOL_SQL_QUEUE.lock().unwrap();
            treadpool_sql_queue.execute(move || {
                let _ = pg_insert_data.update_utxo_log();
            });
            drop(treadpool_sql_queue);
            /**************** POstgreSQL Insert Code End **********/
            /**************************************************** */
            println!("UTXO REMOVED TRADE")
        }
    }*/
}

pub fn process_block_for_utxo_insert(block: Block) -> BlockResult {
    let mut tx_result: BlockResult = BlockResult::new();
    for transaction in block.transactions {
        match transaction.tx_type.as_str() {
            "/twilightproject.nyks.zkos.MsgTransferTx" => {
                process_transfer(transaction, block.block_height, &mut tx_result)
            }
            "/twilightproject.nyks.zkos.MsgMintBurnTradingBtc" => {
                process_trade_mint(transaction, block.block_height, &mut tx_result)
            }
            _ => {} // you might want to handle any other cases or just ignore them
        };
    }
    tx_result
}

pub fn all_coin_type_utxo() -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let utxo_storage = UTXO_STORAGE.read().unwrap();
    let input_type = IOType::Coin as usize;
    let utxos = utxo_storage.data.get(&input_type).unwrap();
    for (key, output_data) in utxos {
        match bincode::deserialize(&key) {
            Ok(value) => {
                let utxo: Utxo = value;
                let hex_str: String = utxo.to_hex();
                result.push(hex_str)
            }
            Err(args) => {
                let err = format!("Deserialization error, {:?}", args);
                println!("{}", err)
            }
        }
    }
    return result;
}
pub fn all_memo_type_utxo() -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let utxo_storage = UTXO_STORAGE.read().unwrap();
    let input_type = IOType::Memo as usize;
    let utxos = utxo_storage.data.get(&input_type).unwrap();
    for (key, output_data) in utxos {
        match bincode::deserialize(&key) {
            Ok(value) => {
                let utxo: Utxo = value;
                let hex_str: String = utxo.to_hex();
                result.push(hex_str)
            }
            Err(args) => {
                let err = format!("Deserialization error, {:?}", args);
                println!("{}", err)
            }
        }
    }
    return result;
}
pub fn all_state_type_utxo() -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let utxo_storage = UTXO_STORAGE.read().unwrap();
    let input_type = IOType::State as usize;
    let utxos = utxo_storage.data.get(&input_type).unwrap();
    for (key, output_data) in utxos {
        match bincode::deserialize(&key) {
            Ok(value) => {
                let utxo: Utxo = value;
                let hex_str: String = utxo.to_hex();
                result.push(hex_str)
            }
            Err(args) => {
                let err = format!("Deserialization error, {:?}", args);
                println!("{}", err)
            }
        }
    }
    return result;
}

pub fn all_coin_type_output() -> String {
    let mut result: Vec<Output> = Vec::new();
    let utxo_storage = UTXO_STORAGE.read().unwrap();
    let input_type = IOType::Coin as usize;
    let utxos = utxo_storage.data.get(&input_type).unwrap();
    for (key, output_data) in utxos {
        result.push(output_data.clone());
    }
    let bytes = bincode::serialize(&result).unwrap();
    return hex::encode(bytes);
}

pub fn search_coin_type_utxo_by_address(address: address::Standard) -> Vec<Utxo> {
    let mut filtered_utxo: Vec<Utxo> = Vec::new();
    let mut utxo_storage = UTXO_STORAGE.read().unwrap();
    let input_type = IOType::Coin as usize;
    let utxos: &std::collections::HashMap<Vec<u8>, Output> =
        utxo_storage.data.get(&input_type).unwrap();

    for (key, output_data) in utxos {
        let addr = output_data.output.get_owner_address().unwrap();
        if address::Standard::from_hex(addr).public_key == address.public_key {
            match bincode::deserialize(&key) {
                Ok(value) => {
                    filtered_utxo.push(value);
                }
                Err(args) => {
                    let err = format!("Deserialization error, {:?}", args);
                    println!("{}", err)
                }
            }
        }
    }

    return filtered_utxo;
}
pub fn search_memo_type_utxo_by_address(address: address::Standard) -> Vec<Utxo> {
    let mut filtered_utxo: Vec<Utxo> = Vec::new();
    let utxo_storage = UTXO_STORAGE.read().unwrap();
    let input_type = IOType::Memo as usize;
    let utxos = utxo_storage.data.get(&input_type).unwrap();

    for (key, output_data) in utxos {
        let addr = output_data.output.get_owner_address().unwrap();
        if address::Standard::from_hex(addr).public_key == address.public_key {
            match bincode::deserialize(&key) {
                Ok(value) => {
                    filtered_utxo.push(value);
                }
                Err(args) => {
                    let err = format!("Deserialization error, {:?}", args);
                    println!("{}", err)
                }
            }
        }
    }

    return filtered_utxo;
}
pub fn search_state_type_utxo_by_address(address: address::Standard) -> Vec<Utxo> {
    let mut filtered_utxo: Vec<Utxo> = Vec::new();
    let utxo_storage = UTXO_STORAGE.read().unwrap();
    let input_type = IOType::State as usize;
    let utxos = utxo_storage.data.get(&input_type).unwrap();

    for (key, output_data) in utxos {
        let addr = output_data.output.get_owner_address().unwrap();
        if address::Standard::from_hex(addr).public_key == address.public_key {
            match bincode::deserialize(&key) {
                Ok(value) => {
                    filtered_utxo.push(value);
                }
                Err(args) => {
                    let err = format!("Deserialization error, {:?}", args);
                    println!("{}", err)
                }
            }
        }
    }

    return filtered_utxo;
}

pub fn search_coin_type_utxo_by_utxo_key(utxo: Utxo) -> Result<Output, &'static str> {
    let mut utxo_storage = UTXO_STORAGE.read().unwrap();
    let input_type = IOType::Coin as usize;
    let result = match utxo_storage.get_utxo_by_id(utxo.to_bytes(), input_type) {
        Ok(output) => output,
        Err(_err) => return Err("Utxo not found "),
    };
    return Ok(result);
}

pub fn search_utxo_by_utxo_key(utxo: Utxo, input_type: IOType) -> Result<Output, &'static str> {
    let mut utxo_storage = UTXO_STORAGE.read().unwrap();

    let result = match utxo_storage.get_utxo_by_id(utxo.to_bytes(), input_type.to_usize()) {
        Ok(output) => output,
        Err(_err) => return Err("Utxo not found "),
    };
    return Ok(result);
}
pub fn search_utxo_by_utxo_key_bytes(
    utxo: Vec<u8>,
    input_type: IOType,
) -> Result<Output, &'static str> {
    let utxo_storage = UTXO_STORAGE.read().unwrap();

    let result = match utxo_storage.get_utxo_by_id(utxo, input_type.to_usize()) {
        Ok(output) => output,
        Err(_err) => return Err("Utxo not found "),
    };
    return Ok(result);
}
pub fn search_memo_type_utxo_by_utxo_key(utxo: Utxo) -> Result<Output, &'static str> {
    let mut utxo_storage = UTXO_STORAGE.read().unwrap();
    let input_type = IOType::Memo as usize;
    let result = match utxo_storage.get_utxo_by_id(utxo.to_bytes(), input_type) {
        Ok(output) => output,
        Err(_err) => return Err("Utxo not found "),
    };
    return Ok(result);
}
pub fn search_state_type_utxo_by_utxo_key(utxo: Utxo) -> Result<Output, &'static str> {
    let mut utxo_storage = UTXO_STORAGE.read().unwrap();
    let input_type = IOType::State as usize;
    let result = match utxo_storage.get_utxo_by_id(utxo.to_bytes(), input_type) {
        Ok(output) => output,
        Err(_err) => return Err("Utxo not found "),
    };
    return Ok(result);
}
pub fn total_memo_type_utxos() -> u64 {
    println!("inside total memo");
    let utxo_storage = UTXO_STORAGE.read().unwrap();
    let input_type = IOType::Memo as usize;
    let result = utxo_storage.get_count_by_type(input_type);
    println!("{}", result);
    return result;
}

pub fn total_state_type_utxos() -> u64 {
    let utxo_storage = UTXO_STORAGE.read().unwrap();
    let input_type = IOType::State as usize;
    let result = utxo_storage.get_count_by_type(input_type);
    println!("{}", result);
    return result;
}

pub fn total_coin_type_utxos() -> u64 {
    let utxo_storage = UTXO_STORAGE.read().unwrap();
    let input_type = IOType::Coin as usize;
    let result = utxo_storage.get_count_by_type(input_type);
    println!("{}", result);
    return result;
}
pub fn verify_utxo(transaction: transaction::Transaction) -> bool {
    let utxo_storage = UTXO_STORAGE.read().unwrap();

    let tx_inputs = transaction.get_tx_inputs();
    if transaction.tx_type == TransactionType::Script {
        for input in tx_inputs {
            let utxo = input.as_utxo().unwrap();
            let client_output: OutputData = match input.in_type {
                IOType::Coin => OutputData::Coin(input.as_out_coin().unwrap().clone()),
                IOType::Memo => OutputData::Memo(input.as_out_memo().unwrap().clone()),
                IOType::State => OutputData::State(input.as_out_state().unwrap().clone()),
            };
            let utxo_test = Utxo::new(TxID(Hash([0; 32])), 0);
            if utxo.to_owned() != utxo_test {
                let utxo_input_type = input.in_type as usize;
                let utxo_key = bincode::serialize(input.as_utxo().unwrap()).unwrap();

                let utxo_output_from_chain_result =
                    utxo_storage.get_utxo_by_id(utxo_key.clone(), utxo_input_type);

                match utxo_output_from_chain_result {
                    Ok(utxo_output_from_chain) => match input.in_type {
                        IOType::Coin => {
                            if utxo_output_from_chain
                                .as_out_coin()
                                .unwrap()
                                .clone()
                                .eq(client_output.get_output_coin().unwrap())
                            {
                                continue;
                            } else {
                                return false;
                            }
                        }
                        IOType::Memo => {
                            if utxo_output_from_chain
                                .as_out_memo()
                                .unwrap()
                                .clone()
                                .eq(client_output.get_output_memo().unwrap())
                            {
                                continue;
                            } else {
                                return false;
                            }
                        }
                        IOType::State => {
                            if utxo_output_from_chain
                                .as_out_state()
                                .unwrap()
                                .clone()
                                .eq(client_output.get_output_state().unwrap())
                            {
                                continue;
                            } else {
                                return false;
                            }
                        }
                    },

                    Err(arg) => {
                        return false;
                    }
                }

                // return true;
            }
        }
    } else if transaction.tx_type == TransactionType::Transfer {
        for input in tx_inputs {
            let utxo = input.as_utxo().unwrap();
            let client_output: OutputData = match input.in_type {
                IOType::Coin => OutputData::Coin(input.as_out_coin().unwrap().clone()),
                _ => return false,
            };
            let utxo_test = Utxo::new(TxID(Hash([0; 32])), 0);
            if utxo.to_owned() != utxo_test {
                let utxo_key = bincode::serialize(utxo).unwrap();

                let utxo_output_from_chain_result =
                    utxo_storage.get_utxo_by_id(utxo_key.clone(), 0);

                match utxo_output_from_chain_result {
                    Ok(utxo_output_from_chain) => match input.in_type {
                        IOType::Coin => {
                            if utxo_output_from_chain
                                .as_out_coin()
                                .unwrap()
                                .clone()
                                .eq(client_output.get_output_coin().unwrap())
                            {
                                continue;
                            } else {
                                return false;
                            }
                        }
                        _ => return false,
                    },

                    Err(arg) => {
                        return false;
                    }
                }
            }
        }
    } else if transaction.tx_type == TransactionType::Message {
        // check if message is burn
        let message = transaction.tx.to_message().unwrap();
        let input = message.input.clone();
        if message.msg_type == zkvm::zkos_types::MessageType::Burn {
            let utxo = message.input.as_utxo().unwrap();
            let client_output: OutputData = match input.in_type {
                IOType::Coin => OutputData::Coin(input.as_out_coin().unwrap().clone()),
                _ => return false,
            };
            let utxo_key = bincode::serialize(utxo).unwrap();
            let utxo_output_from_chain_result = utxo_storage.get_utxo_by_id(utxo_key.clone(), 0);
            match utxo_output_from_chain_result {
                Ok(utxo_output_from_chain) => match input.in_type {
                    IOType::Coin => {
                        if utxo_output_from_chain
                            .as_out_coin()
                            .unwrap()
                            .clone()
                            .eq(client_output.get_output_coin().unwrap())
                        {
                        } else {
                            return false;
                        }
                    }
                    _ => return false,
                },

                Err(arg) => {
                    return false;
                }
            }
        }
    }

    return true;
}
/// This function will create a block with a set of transactions
/// to test UTXO Set functionality
///
pub fn create_utxo_test_block(
    set: &mut Vec<RecordUtxo>,
    prev_height: u64,
    sk_sender: &[RistrettoSecretKey],
) -> Block {
    // for the time being we will only build Script txs
    let mut rng = rand::thread_rng();
    //let mut set_size = set.len();
    let mut txs = Vec::<TransactionMessage>::new();
    let mut new_set: Vec<RecordUtxo> = Vec::new();

    //select # of txs to be created. The numbers should be adjusted based on the size of the existing set
    let num_txs = rng.gen_range(0u32, 100u32);
    let num_inputs_per_tx = rng.gen_range(0u32, 9u32);
    let num_outputs_per_tx = rng.gen_range(5u32, 15u32);

    //let 10 % of these tx are transfer Txs

    //create Script Transactions
    let trans_tx = num_txs / 10;
    for _ in 0..(num_txs - trans_tx) {
        let mut id: [u8; 32] = [0; 32];
        // Generate random values and fill the array
        rand::thread_rng().fill(&mut id);
        let tx_id = TxID(Hash(id));
        //select random inputs
        let mut inputs: Vec<Input> = Vec::new();
        for _ in 0..num_inputs_per_tx {
            // println!("set len:{:#?}", set.len());
            if set.len() == 0 {
                break;
            }
            let random_number: u32 = rng.gen_range(0u32, set.len() as u32);
            let record: RecordUtxo = set[random_number as usize].clone();

            let inp = convert_output_to_input(record.clone()).unwrap();
            inputs.push(inp.clone());
            //remove input from set
            set.remove(random_number as usize);
        }
        //select random outputs
        let mut outputs: Vec<Output> = Vec::new();
        for i in 0..num_outputs_per_tx {
            let random_number: u32 = rng.gen_range(0u32, 3u32);
            if random_number == 0 {
                //coin output
                let (pk, enc) = Account::generate_random_account_with_value(Scalar::from(20u64))
                    .0
                    .get_account();
                let out = Output::coin(OutputData::Coin(OutputCoin {
                    encrypt: enc,
                    owner: Address::standard_address(Network::default(), pk).as_hex(),
                }));
                outputs.push(out.clone());
                //add to new set
                let utx = Utxo::new(tx_id, i.try_into().unwrap());
                new_set.push(RecordUtxo {
                    utx: utx,
                    value: out,
                });
            }
            if random_number == 1 {
                //memo output
                let (pk, _) = Account::generate_random_account_with_value(Scalar::from(10u64))
                    .0
                    .get_account();
                let add = Address::standard_address(Network::default(), pk);
                let out = Output::memo(OutputData::Memo(OutputMemo {
                    script_address: add.as_hex(),
                    owner: add.as_hex(),
                    commitment: Commitment::Closed(CompressedRistretto::default()),
                    data: None,
                    timebounds: 0u32,
                }));

                outputs.push(out.clone());
                //add to new set
                let utx = Utxo::new(tx_id, i.try_into().unwrap());
                new_set.push(RecordUtxo { utx, value: out });
            }
            if random_number == 2 {
                //state output
                let (pk, _) = Account::generate_random_account_with_value(Scalar::from(0u64))
                    .0
                    .get_account();
                let add = Address::standard_address(Network::default(), pk);
                let out = Output::state(OutputData::State(OutputState {
                    nonce: 0u32,
                    script_address: add.as_hex(),
                    owner: add.as_hex(),
                    commitment: Commitment::Closed(CompressedRistretto::default()),
                    state_variables: None,
                    timebounds: 0,
                }));
                outputs.push(out.clone());
                //add to new set
                let utx = Utxo::new(tx_id, i.try_into().unwrap());
                new_set.push(RecordUtxo { utx, value: out });
            }
        }

        //create tx
        // let mut id: [u8; 32] = [0; 32];
        // // Generate random values and fill the array
        // rand::thread_rng().fill(&mut id);
        let script_tx: ScriptTransaction =
            ScriptTransaction::create_utxo_dummy_script_transaction(&inputs, &outputs);
        let tx: Transaction =
            Transaction::transaction_script(TransactionData::TransactionScript(script_tx));

        let serialized: Vec<u8> = bincode::serialize(&tx).unwrap();
        let hex = hex::encode(serialized);

        let txx: TransactionMessage = TransactionMessage {
            tx_type: "testtype".to_string(),
            tx_id: "testid".to_string(),
            tx_byte_code: Some(hex),
            zk_oracle_address: Some("test address".to_string()),
            mint_or_burn: None,
            btc_value: None,
            qq_account: None,
            encrypt_scalar: None,
            twilight_address: None,
        };

        txs.push(txx);
    }
    //create Transfer Txs
    for _ in 0..trans_tx {
        //let there be 1 input and 2 output combos
        //select random inputs
        let input: Input;
        loop {
            // if set.len() == 0 {
            //     break;
            // }
            let random_number: u32 = rng.gen_range(0u32, set.len() as u32);
            let record: RecordUtxo = set[random_number as usize].clone();
            match record.value.out_type {
                IOType::Coin => {
                    input = convert_output_to_input(record.clone()).unwrap();

                    //remove input from set
                    set.remove(random_number as usize);
                    break;
                }
                _ => continue,
            }
        }
        // println!("creating dark tx");
        let tx = create_dark_reference_tx_for_utxo_test(input, &sk_sender);
        //extract outputs from tx for dummy set
        let outp = tx.clone().get_tx_outputs();
        //create random txid
        let mut id: [u8; 32] = [0; 32];
        rand::thread_rng().fill(&mut id);
        let tx_id = TxID(Hash(id));
        for ii in 0..outp.len() {
            let utx = Utxo::new(tx_id, ii.try_into().unwrap());
            new_set.push(RecordUtxo {
                utx,
                value: outp[ii].clone(),
            });
        }

        let serialized: Vec<u8> = bincode::serialize(&tx).unwrap();
        let hex = hex::encode(serialized);

        let txx: TransactionMessage = TransactionMessage {
            tx_type: "testtype".to_string(),
            tx_id: "testid".to_string(),
            tx_byte_code: Some(hex),
            zk_oracle_address: Some("test address".to_string()),
            mint_or_burn: None,
            btc_value: None,
            qq_account: None,
            encrypt_scalar: None,
            twilight_address: None,
        };

        txs.push(txx);
    }

    let block = Block {
        block_hash: "abc123".to_string(),
        block_height: prev_height + 1,
        transactions: txs,
    };
    //append new utxo set with old one to update the recent outputs
    set.append(&mut new_set);
    block
}

#[cfg(test)]
mod test {
    //write test to fail a tx

    use crate::blockoperations::blockprocessing::create_utxo_test_block;
    use crate::blockoperations::blockprocessing::process_block_for_utxo_insert;
    use crate::db::*;
    use crate::{init_utxo, UTXO_STORAGE};
    use curve25519_dalek::scalar::Scalar;
    use quisquislib::accounts::Account;
    use transaction::reference_tx::create_genesis_block;

    // cargo test -- --nocapture --test check_block_test --test-threads 5
    #[test]
    fn check_block_test() {
        init_utxo();
        let utxo_storage = UTXO_STORAGE.read().unwrap();
        let block_height = utxo_storage.block_height as u64;
        drop(utxo_storage);

        let (acc, prv) = Account::generate_random_account_with_value(Scalar::from(20u64));
        let mut recordutxo = crate::blockoperations::load_genesis_sets();

        let block1 = create_utxo_test_block(&mut recordutxo, block_height, &vec![prv]);
        let result = process_block_for_utxo_insert(block1);
        let utxo_storage = UTXO_STORAGE.read().unwrap();
        println!("result block update:{:?}", result);
        //utxo_storage.take_snapshot();
    }

    #[test]
    fn create_utxo_block_test() {
        //keep the private key safe

        let (acc, prv) = Account::generate_random_account_with_value(Scalar::from(20u64));
        let sk_sender = vec![prv];
        //let (pk, _) = acc.get_account();
        //let (inp_acc ,_)= Account::generate_account(pk);
        //let accc: Account = Account::set_account();
        let mut utxo_set = create_genesis_block(1000, 100, acc);
        let block = create_utxo_test_block(&mut utxo_set, 1, &sk_sender);
        println!("Block Height{:?} ", block.block_height);
        println!("Block Txs{:?} ", block.transactions);
    }
}
