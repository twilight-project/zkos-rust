use crate::db::*;
use crate::UTXO_STORAGE;
use hex;
use serde_derive::{Deserialize, Serialize};
use transaction::Transaction;

use transaction::reference_tx::{convert_output_to_input, RecordUtxo, create_dark_reference_tx_for_utxo_test};
use quisquislib::elgamal::elgamal::ElGamalCommitment;
use transaction::types::{InputData, InputType};
use transaction::types::{CData, Coin, Input, Memo, Output, OutputData, OutputType, State, TxId, Utxo};
use transaction::tx::{ScriptTransaction, TransactionData};
use transaction::util::{Address, Network};
use quisquislib::ristretto::RistrettoPublicKey;
use rand::Rng;
use std::collections::HashMap;
use serde::de::{self, Deserializer, Visitor};
use std::fmt;


use quisquislib::{
    accounts::Account,
    ristretto::RistrettoSecretKey,
};
use curve25519_dalek::ristretto::CompressedRistretto;
use curve25519_dalek::scalar::Scalar;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BlockResult {
    pub suceess_tx: Vec<transaction::TxId>,
    pub failed_tx: Vec<transaction::TxId>,
}
impl BlockResult {
    pub fn new() -> Self {
        BlockResult {
            suceess_tx: Vec::new(),
            failed_tx: Vec::new(),
        }
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionMessage {
    #[serde(rename = "@type")]
    pub tx_type: String,
    #[serde(rename = "TxId")]
    pub tx_id: String,
    #[serde(rename = "TxByteCode")]
    pub tx_byte_code: Option<String>,
    #[serde(rename = "ZkOracleAddress")]
    pub zk_oracle_address: Option<String>,
    #[serde(rename = "MintOrBurn")]
    pub mint_or_burn: Option<bool>, // Optional because it's not present in all types.
    #[serde(rename = "BtcValue")]
    pub btc_value: Option<u32>,
    #[serde(rename = "QqAccount")]
    pub qq_account: Option<String>,
    #[serde(rename = "EncryptScalar")]
    pub encrypt_scalar: Option<u64>,
    #[serde(rename = "TwilightAddress")]
    pub twilight_address: Option<String>,
}

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub enum MessageType {
//     #[serde(rename = "/twilightproject.nyks.zkos.MsgMintBurnTradingBtc")]
//     Trading(TransactionMessage),
//     #[serde(rename = "/twilightproject.nyks.zkos.MsgTransferTx")]
//     Transfer(TransactionMessage),
// }


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

pub fn process_transfer(transaction: TransactionMessage, height: u64, tx_result: &mut BlockResult){
    let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
    let tx_bytes = hex::decode(transaction.tx_byte_code.unwrap()).expect("Decoding failed");
    let transaction_info: Transaction = bincode::deserialize(&tx_bytes).unwrap();
    let tx_id = transaction_info.txid;
    let mut success: bool = true;
    let tx_input = transaction_info.get_input_values();
    let tx_output = transaction_info.get_output_values();

    for input in &tx_input {
        let utxo_key = bincode::serialize(input.input.as_utxo_id().unwrap()).unwrap();
        let utxo_input_type = input.in_type as usize;
        let bool = utxo_storage.search_key(&utxo_key, utxo_input_type);
        if bool {
        } else {
            success = false;
        }
    }
    for (output_index, output_set) in tx_output.iter().enumerate() {
        let utxo_key =
            bincode::serialize(&transaction::Utxo::new(tx_id, output_index as u8)).unwrap();
        let utxo_output_type = output_set.out_type as usize;
        let bool = utxo_storage.search_key(&utxo_key, utxo_output_type);
        if bool {
            success = false;
        } else {
        }
    }
    //proccess tx
    if success {
        //remove all input
        for input in tx_input {
            let utxo_key = bincode::serialize(&input.input.as_utxo_id().unwrap()).unwrap();
            let utxo_input_type = input.in_type as usize;
            let _result = utxo_storage.remove(utxo_key, utxo_input_type);
            println!("UTXO REMOVED TRANSFER")
        }
        //Add all output
        for (output_index, output_set) in tx_output.iter().enumerate() {
            let utxo_key =
                bincode::serialize(&transaction::Utxo::new(tx_id, output_index as u8)).unwrap();
            let utxo_output_type = output_set.out_type as usize;
            let _result = utxo_storage.add(utxo_key, output_set.clone(), utxo_output_type);
            println!("UTXO ADDED TRANSFER")
        }

        let _ = utxo_storage.data_meta_update(height as usize);
        tx_result.suceess_tx.push(tx_id);
    } else {
        tx_result.failed_tx.push(tx_id);
    }

}

pub fn process_trade(transaction: TransactionMessage, height: u64, tx_result: &mut BlockResult){
    println!("{:?}", transaction);
    let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
    let tx_id = hex::decode(transaction.tx_id).expect("error decoding tx id");
    let tx_id = TxId::from_vec(tx_id);
    let utxo_key =
    bincode::serialize(&transaction::Utxo::new(tx_id, 0 as u8)).unwrap();

    if transaction.mint_or_burn.unwrap() == true {
        let mut bytes = hex::decode(transaction.qq_account.unwrap()).expect("Decoding failed");
        let elgamal = bytes.split_off(bytes.len() - 64);
        let elgamal = ElGamalCommitment::from_bytes(&elgamal).unwrap();
        let address = Address::from_bytes(&bytes[0..69]).unwrap();
        let output = OutputData::Coin(Coin{encrypt: elgamal, address:address});
        let output = Output{out_type: OutputType::Coin, output: output};
        utxo_storage.add(utxo_key, output.clone(), output.out_type as usize);
        tx_result.suceess_tx.push(tx_id);
        println!("UTXO ADDED TRADE")
    }
    else { 
        utxo_storage.remove(utxo_key, InputType::Coin as usize);
        tx_result.suceess_tx.push(tx_id);
        println!("UTXO REMOVED TRADE")
    }

}

pub fn process_block_for_utxo_insert(block: Block) -> BlockResult {
    let mut tx_result: BlockResult = BlockResult::new();
    for transaction in block.transactions {

        match transaction.tx_type.as_str() {
            "/twilightproject.nyks.zkos.MsgTransferTx" => process_transfer(transaction, block.block_height, &mut tx_result),
            "/twilightproject.nyks.zkos.MsgMintBurnTradingBtc" => process_trade(transaction, block.block_height, &mut tx_result),
            _ => {}  // you might want to handle any other cases or just ignore them
        };
    }
    tx_result
}


pub fn search_coin_type_utxo_by_public_key(address: Address) -> Vec<String>  {
    let mut filtered_utxo: Vec<String> = Vec::new();
    let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
    let input_type = InputType::Coin as usize;
    let utxos = utxo_storage.data.get_mut(&input_type).unwrap();
    let mut utxo: Utxo;

    for (key, output_data) in utxos{
        let addr =  output_data.output.get_owner_address().unwrap();
        if addr.public_key == address.public_key{
            match bincode::deserialize(&key) {
                Ok(value) => utxo = value,
                Err(args) => {
                    let err = format!("Deserialization error, {:?}", args);
                    return vec!(err)
                }
            }

            let tx_id = utxo.tx_id_to_hex();
            filtered_utxo.push(format!("{}:{}", tx_id, utxo.output_index()));
        } 
    }

    return filtered_utxo
}


pub fn verify_utxo(transaction: transaction::Transaction) -> bool{
    let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
    let tx_data = TransactionData::to_transfer(transaction.tx).unwrap();
    for input in tx_data.clone().get_input_values(){
        let utxo_input_type = input.in_type as usize;
        if let InputData::Coin { utxo, owner, encryption, witness, account } = input.input {
            if utxo_storage.search_key(&utxo.tx_id().0.to_vec(), utxo_input_type) == false {
                return false;
            };
        }
        if let InputData::Memo { utxo, script_address, owner, commitment, data, witness } = input.input {
            continue;
        }
        if let InputData::State { utxo, nonce, script_address, owner, commitment, script_data, witness, program_index } = input.input {
            continue;
        }
    }
    return true;
}

pub fn create_utxo_test_block<>(
    set: &mut Vec<RecordUtxo>,
    prev_height: u64,
    sk_sender: &[RistrettoSecretKey],
) -> Block {
    // for the time being we will only build Script txs
    let mut rng = rand::thread_rng();
    //let mut set_size = set.len();
    let mut txs= Vec::<TransactionMessage>::new();
    let mut new_set: Vec<RecordUtxo> =  Vec::new();

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
                let out = Output::coin(OutputData::Coin(Coin {
                    encrypt: enc,
                    address: Address::standard(Network::default(), pk),
                }));
                outputs.push(out.clone());
                //add to new set
                let utx = Utxo::new(TxId(id), i.try_into().unwrap());
                new_set.push(RecordUtxo {
                    utx: utx,
                    value: out,
                });
            }
            if random_number == 1 {
                //memo output
                let (pk, _) = Account::generate_random_account_with_value(Scalar::from(0u64))
                    .0
                    .get_account();
                let add = Address::contract(Network::default(), pk);
                let data: CData = CData {
                    script_address: add,
                    owner: add,
                    commitment: CompressedRistretto::default(),
                };
                let out = Output::memo(OutputData::Memo(Memo {
                    contract: data,
                    data: None,
                }));
                outputs.push(out.clone());
                //add to new set
                let utx = Utxo::new(TxId(id), i.try_into().unwrap());
                new_set.push(RecordUtxo {
                    utx: utx,
                    value: out,
                });
            }
            if random_number == 2 {
                //state output
                let (pk, _) = Account::generate_random_account_with_value(Scalar::from(0u64))
                    .0
                    .get_account();
                let add = Address::contract(Network::default(), pk);
                let data: CData = CData {
                    script_address: add,
                    owner: add,
                    commitment: CompressedRistretto::default(),
                };
                let out = Output::state(OutputData::State(State {
                    nonce: 0u32,
                    contract: data,
                    script_data: None,
                }));
                outputs.push(out.clone());
                //add to new set
                let utx = Utxo::new(TxId(id), i.try_into().unwrap());
                new_set.push(RecordUtxo {
                    utx: utx,
                    value: out,
                });
            }
        }

        //create tx
        // let mut id: [u8; 32] = [0; 32];
        // // Generate random values and fill the array
        // rand::thread_rng().fill(&mut id);
        let script_tx: ScriptTransaction =
            ScriptTransaction::create_utxo_script_transaction(&inputs, &outputs);
        let tx: Transaction =
            Transaction::transaction_script(TxId(id), TransactionData::Script(script_tx));

        let serialized: Vec<u8> = bincode::serialize(&tx).unwrap();
        let hex = hex::encode(serialized);

        let txx: TransactionMessage = TransactionMessage{
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
                OutputType::Coin => {
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
        let outp = tx.clone().tx.to_transfer().unwrap().outputs;
        for ii in 0..outp.len() {
            let utx = Utxo::new(tx.txid, ii.try_into().unwrap());
            new_set.push(RecordUtxo {
                utx: utx,
                value: outp[ii].clone(),
            });
        }

        let serialized: Vec<u8> = bincode::serialize(&tx).unwrap();
        let hex = hex::encode(serialized);

        let txx: TransactionMessage = TransactionMessage{
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

    use crate::db::*;
    use crate::{init_utxo, UTXO_STORAGE};
    use crate::blockoperations::blockprocessing::create_utxo_test_block;
    use transaction::reference_tx::create_genesis_block;
    use crate::blockoperations::blockprocessing::process_block_for_utxo_insert;
    use curve25519_dalek::scalar::Scalar;
    use quisquislib::accounts::Account;

    // cargo test -- --nocapture --test check_block_test --test-threads 5
    #[test]
    fn check_block_test() {
        init_utxo();
        let utxo_storage = UTXO_STORAGE.lock().unwrap();
        let block_height = utxo_storage.block_height as u64;
        drop(utxo_storage);

        let (acc, prv) = Account::generate_random_account_with_value(Scalar::from(20u64));
        let mut recordutxo = crate::blockoperations::load_genesis_sets();

        let block1 = create_utxo_test_block(
            &mut recordutxo,
            block_height,
            &vec![prv],
        );
        let result = process_block_for_utxo_insert(block1);
        let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
        println!("result block update:{:?}", result);
        utxo_storage.take_snapshot();
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
