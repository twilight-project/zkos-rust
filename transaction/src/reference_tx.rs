#![allow(non_snake_case)]
//#![deny(missing_docs)]

use crate::tx::{Transaction, TransactionData, TransferTransaction, ScriptTransaction};
use crate::types::{
    Input, InputData, Output, OutputData, TransactionType, TxEntry, TxId, TxLog, Utxo, Witness, CData, Coin, Memo, State
};
use crate::util::{Address, Network};
// use serde_derive::{Deserialize, Serialize};
use serde::{Deserialize, Serialize};

use curve25519_dalek::constants::RISTRETTO_BASEPOINT_COMPRESSED;
use curve25519_dalek::ristretto::{RistrettoPoint, CompressedRistretto};
use curve25519_dalek::scalar::Scalar;
use rand::rngs::OsRng;
use sha3::Sha3_512;
use rand::Rng;
use quisquislib::{
    accounts::Account,
    keys::PublicKey,
    ristretto::{RistrettoPublicKey, RistrettoSecretKey},
};

///Needed for Creating Reference transaction for Testing RPC
///

//Create Sender and receiver

#[derive(Debug, Clone)]
pub struct Receiver {
    amount: i64,
    acc: Account,
}

#[derive(Debug, Clone)]
pub struct Sender {
    total_amount: i64,
    account: Account,
    receivers: Vec<Receiver>,
}

#[derive(Debug, Clone)]
pub struct RecordUtxo {
    pub utx: Utxo,
    pub value: Output,
}

#[derive(Debug, Clone)]
pub struct UtxoSet {
    pub set: Vec<RecordUtxo>,
}

impl Sender {
    pub fn generate_value_and_account_vector(
        tx_vector: Vec<Sender>,
    ) -> Result<(Vec<i64>, Vec<Account>, Vec<Scalar>, usize, usize, usize), &'static str> {
        if tx_vector.len() < 9 {
            let mut value_vector: Vec<i64> = tx_vector.iter().map(|s| s.total_amount).collect();
            let mut account_vector: Vec<Account> = tx_vector.iter().map(|s| s.account).collect();
            let senders_count: usize = tx_vector.iter().count();
            let mut receivers_count = 0;
            let mut receiver_amount_vector: Vec<i64> = Vec::new();
            let mut receiver_account_vector: Vec<Account> = Vec::new();
            //keep track of all the r used for commitment of value zero
            let mut annonymity_account_commmitment_scalars_vector: Vec<Scalar> = Vec::new();

            for sender in tx_vector.iter() {
                receivers_count += &sender.receivers.iter().count();

                for rec in sender.receivers.iter() {
                    receiver_amount_vector.push(rec.amount);
                    receiver_account_vector.push(rec.acc.clone());
                }
            }

            if senders_count < 9 && receivers_count < 9 && senders_count + receivers_count <= 9 {
                value_vector.append(&mut receiver_amount_vector);
                account_vector.append(&mut receiver_account_vector);

                // lets create anonymity set - these are randomly generated on the fly
                // this anonymity set may need to come from the blockchain state itself in the future

                let diff = 9 - (senders_count + receivers_count);
                //use random key as base pk for annonymity accounts
                let pk_g = RISTRETTO_BASEPOINT_COMPRESSED;
                let pk_h = RistrettoPoint::hash_from_bytes::<Sha3_512>(
                    RISTRETTO_BASEPOINT_COMPRESSED.as_bytes(),
                )
                .compress();
                let pk_ref = RistrettoPublicKey::new_from_pk(pk_g, pk_h);
                let pk_annonymity =
                    PublicKey::update_public_key(&pk_ref, Scalar::random(&mut OsRng));

                if diff >= 1 {
                    for _ in 0..diff {
                        value_vector.push(0);
                        let (acc, comm_scalar) =
                            Account::generate_account(PublicKey::update_public_key(
                                &pk_annonymity,
                                Scalar::random(&mut OsRng),
                            ));
                        account_vector.push(acc);
                        annonymity_account_commmitment_scalars_vector.push(comm_scalar);
                    }
                }

                Ok((
                    value_vector,
                    account_vector,
                    annonymity_account_commmitment_scalars_vector,
                    diff,
                    senders_count,
                    receivers_count,
                ))
            } else {
                Err("senders and receivers count should be less than 9")
            }
        } else {
            Err("account count is more than 9")
        }
    }

    pub fn create_reference_tx_data_for_zkos_test() -> Result<
        (
            Vec<i64>,
            Vec<Account>,
            Vec<Scalar>,
            usize,
            usize,
            usize,
            Vec<RistrettoSecretKey>,
            Vec<u64>,
        ),
        &'static str,
    > {
        // lets say bob wants to sent 5 tokens to alice from his one account and 2 from his other account to fay
        // and 1 token to jay

        // lets create sender accounts to send these amounts from
        let (bob_account_1, bob_sk_account_1) =
            Account::generate_random_account_with_value(10u64.into());
        let (bob_account_2, bob_sk_account_2) =
            Account::generate_random_account_with_value(20u64.into());

        // lets create receiver accounts
        let alice_account = Account::generate_random_account_with_value(10u64.into()).0;
        let fay_account = Account::generate_random_account_with_value(20u64.into()).0;
        let jay_account = Account::generate_random_account_with_value(20u64.into()).0;

        // so we have 2 senders and 3 receivers, rest will be the anonymity set

        //let mut tx_vector: Vec<Sender> = Vec::new();

        let tx_vector: Vec<Sender> = vec![
            Sender {
                total_amount: -5,
                account: bob_account_1,
                receivers: vec![Receiver {
                    amount: 5,
                    acc: alice_account,
                }],
            },
            Sender {
                total_amount: -3,
                account: bob_account_2,
                receivers: vec![
                    Receiver {
                        amount: 2,
                        acc: fay_account,
                    },
                    Receiver {
                        amount: 1,
                        acc: jay_account,
                    },
                ],
            },
        ];

        let (
            value_vector,
            account_vector,
            annonymity_com_scalar_vector,
            diff,
            sender_count,
            receiver_count,
        ) = Sender::generate_value_and_account_vector(tx_vector)?;
        //Create sender updated account vector for the verification of sk and bl-v
        let bl_first_sender = 10 - 5; //bl-v
        let bl_second_sender = 20 - 3; //bl-v
        let updated_balance_sender: Vec<u64> = vec![bl_first_sender, bl_second_sender];
        //Create vector of sender secret keys
        let sk_sender: Vec<RistrettoSecretKey> = vec![bob_sk_account_1, bob_sk_account_2];

        Ok((
            value_vector,
            account_vector,
            annonymity_com_scalar_vector,
            diff,
            sender_count,
            receiver_count,
            sk_sender,
            updated_balance_sender,
        ))
    }
}

pub fn create_qq_reference_transaction() -> Transaction {
    let (
        value_vector,
        account_vector,
        annonymity_com_scalar_vector,
        diff,
        sender_count,
        receiver_count,
        sk_sender,
        updated_sender_balance,
    ) = Sender::create_reference_tx_data_for_zkos_test().unwrap();
    //create vector of inputs to be used in tx
    //random utxo IDS to be used in Inputs
    let id: [u8; 32] = [0; 32];

    let utxo = Utxo::new(TxId(id), 0);
    //create vec of Inouts
    let mut inputs: Vec<Input> = Vec::new();
    for input in account_vector.iter() {
        //get account
        let (pk, encrypt) = input.get_account();
        //create address
        let add:Address = Address::standard(Network::default(), pk);
        let inp = Input::coin(InputData::coin(utxo.clone(), add, encrypt, 0));
        inputs.push(inp.clone());
    }

    let updated_balance_reciever: Vec<u64> = vec![5, 2, 1];
    //println!("Data : {:?}", sender_count);
    //create quisquis transfertransaction
    let transfer = TransferTransaction::create_quisquis_transaction(
        &inputs,
        &value_vector,
        &account_vector,
        &updated_sender_balance,
        &updated_balance_reciever,
        &sk_sender,
        sender_count,
        receiver_count,
        &annonymity_com_scalar_vector,
        diff,
    );
    let id: [u8; 32] = [0; 32];
    Transaction::transaction_transfer(TxId(id), TransactionData::TransactionTransfer(transfer.unwrap()))
}

pub fn create_dark_reference_transaction() -> Transaction {
    let (
        value_vector,
        account_vector,
        _,
        _,
        sender_count,
        receiver_count,
        sk_sender,
        updated_sender_balance,
    ) = Sender::create_reference_tx_data_for_zkos_test().unwrap();
    //create vector of inputs to be used in tx
    //random utxo IDS to be used in Inputs
    let id: [u8; 32] = [0; 32];
    let accounts = &account_vector[..(sender_count + receiver_count)];
    let utxo = Utxo::new(TxId(id), 0);
    //create vec of Inouts
    let mut inputs: Vec<Input> = Vec::new();
    for input in accounts.iter() {
        //get account
        let (pk, encrypt) = input.get_account();
        //create address
        let add:Address = Address::standard(Network::default(), pk);
        let inp = Input::coin(InputData::coin(utxo.clone(), add, encrypt, 0));
        inputs.push(inp.clone());
    }

    let updated_balance_reciever: Vec<u64> = vec![5, 2, 1];
    //println!("Data : {:?}", sender_count);
    //create quisquis transfertransaction
    let transfer = TransferTransaction::create_dark_transaction(
        &value_vector,
        &accounts,
        &updated_sender_balance,
        &updated_balance_reciever,
        &inputs,
        &sk_sender,
        sender_count,
        receiver_count,
    );
    let id: [u8; 32] = [0; 32];
    Transaction::transaction_transfer(TxId(id), TransactionData::TransactionTransfer(transfer.unwrap()))
}


//Should be called first. Will only create a random set of outputs 
//with random txIDs to kickstart the system
pub fn create_genesis_block(total_outputs: u64, num_tx: u64)-> UtxoSet{
    //100000 outputs divided among 10000 txs
    let mut outputs:Vec<RecordUtxo> = Vec::with_capacity(total_outputs as usize);
    let mut rng = rand::thread_rng();
    let tot_outs_per_tx = total_outputs/num_tx; 

    for i in 0..num_tx{
        let id: [u8; 32] = [i.try_into().unwrap(); 32];
        for j in 0..tot_outs_per_tx{
            
            let random_number: u32 = rng.gen_range(0u32, 3u32); 
            if random_number == 0{ //coin output
                let utx = Utxo::new(TxId(id), j.try_into().unwrap());
                let (pk, enc) = Account::generate_random_account_with_value(Scalar::from(0u64)).0.get_account();
                let out = OutputData::Coin(Coin{encrypt: enc, address: Address::standard(Network::default(), pk)}); 
            let output = Output::coin(out);
            outputs.push(RecordUtxo { utx: utx, value: output });   
            }                
            
            if random_number == 1{ //memo output
                let utx = Utxo::new(TxId(id), j.try_into().unwrap());
                let (pk, _) = Account::generate_random_account_with_value(Scalar::from(0u64)).0.get_account();
                let add = Address::contract(Network::default(), pk);
                let data : CData = CData { script_address: add, owner: add, commitment: CompressedRistretto::default()};
                let out = Output::memo(OutputData::Memo(Memo{contract: data, data:None}));
            
                outputs.push(RecordUtxo { utx: utx, value: out }); 
            }
            if random_number == 2{ //state output
                let utx = Utxo::new(TxId(id), j.try_into().unwrap());
                let (pk, _) = Account::generate_random_account_with_value(Scalar::from(0u64)).0.get_account();
                let add = Address::contract(Network::default(), pk);
                let data : CData = CData { script_address: add, owner: add, commitment: CompressedRistretto::default()};
                let out = Output::state(OutputData::State(State{nonce: 0u32, contract: data, script_data:None}));
                outputs.push(RecordUtxo { utx: utx, value: out }); 

            }
        }
    }
    UtxoSet{set: outputs }
}
pub struct Block{
    pub height: u64,
    pub txs: Vec<Transaction>,
}
pub fn create_utxo_test_block(set: UtxoSet, prev_height: u64) -> (Block, UtxoSet) {
    // for the time being we will only build Script txs
    let mut rng = rand::thread_rng();
    let set_size = set.set.len();
    let mut txs: Vec<Transaction> = Vec::new();
    let mut new_set: Vec<RecordUtxo> = Vec::new();

    //select # of txs to be created. The numbers should be adjusted based on the size of the existing set 
    let num_txs = rng.gen_range(0u32, 100u32);
    let num_inputs_per_tx = rng.gen_range(0u32, 10u32);
    let num_outputs_per_tx = rng.gen_range(5u32, 15u32);

    let mut inputs: Vec<Input> = Vec::new();
    let mut outputs: Vec<Output> = Vec::new();

    for _ in 0..num_txs {
        //select random inputs
        let mut inputs: Vec<Input> = Vec::new();
        for _ in 0..num_inputs_per_tx {
            let random_number: u32 = rng.gen_range(0u32, set_size as u32);
            let record: RecordUtxo = set.set[random_number as usize].clone();

            let inp = convert_output_to_input(record.clone());
            inputs.push(inp);
            //remove input from set
            set.set.remove(random_number as usize);
        }
        //select random outputs
        let mut outputs: Vec<Output> = Vec::new();
        for i in 0..num_outputs_per_tx {
            let random_number: u32 = rng.gen_range(0u32, 3u32);
            if random_number == 0 {
                //coin output
                let (pk, enc) = Account::generate_random_account_with_value(Scalar::from(0u64)).0.get_account();
                let out = Output::coin(OutputData::Coin(Coin{encrypt: enc, address: Address::standard(Network::default(), pk)}));
                outputs.push(out.clone());
                //add to new set
                let utx = Utxo::new(TxId([0u8; 32]), i.try_into().unwrap());
                new_set.push(RecordUtxo { utx: utx, value: out });
            }
            if random_number == 1 {
                //memo output
                let (pk, _) = Account::generate_random_account_with_value(Scalar::from(0u64)).0.get_account();
                let add = Address::contract(Network::default(), pk);
                let data : CData = CData { script_address: add, owner: add, commitment: CompressedRistretto::default()};
                let out = Output::memo(OutputData::Memo(Memo{contract: data, data:None}));
                outputs.push(out.clone());
                //add to new set
                let utx = Utxo::new(TxId([0u8; 32]), i.try_into().unwrap());
                new_set.push(RecordUtxo { utx: utx, value: out });
            }
            if random_number == 2 {
                //state output
                let (pk, _) = Account::generate_random_account_with_value(Scalar::from(0u64)).0.get_account();
                let add = Address::contract(Network::default(), pk);
                let data : CData = CData { script_address: add, owner: add, commitment: CompressedRistretto::default()};
                let out = Output::state(OutputData::State(State{nonce: 0u32, contract: data, script_data:None}));
                outputs.push(out.clone());
                //add to new set
                let utx = Utxo::new(TxId([0u8; 32]), i.try_into().unwrap());
                new_set.push(RecordUtxo { utx: utx, value: out });
            }
        }
        //create tx
        let mut id: [u8; 32] = [0; 32];
        // Generate random values and fill the array
        rand::thread_rng().fill(&mut id);
        let script_tx:ScriptTransaction = ScriptTransaction::create_utxo_script_transaction(&inputs, &outputs);
        let tx: Transaction = Transaction::transaction_script(id, TransactionData::Script(script_tx));

        txs.push(tx);
    }
    let block = Block{height: prev_height + 1, txs: txs}; 
    //append new utxo set with old one to update the recent outputs
    set.set.append(&mut new_set);  
    (block, set)


}

pub fn convert_output_to_input(rec: RecordUtxo)-> Input{
    let utx = rec.utx;
    let out = rec.value;
    if out.out_type.is_coin(){
        let add = out.output.get_owner_address().unwrap().to_owned();
        let inp = Input::coin(InputData::coin(utx, out.output.get_owner_address().to_owned(), o.encrypt, 0));
            inp
    }
    if out.out_type.is_memo(){
            let inp = Input::memo(InputData::memo(utx, o.contract.script_address, o.contract.owner, o.contract.commitment, o.data, 0));
            inp
        }
        if out.out_type.is_state(){
            let inp = Input::state(InputData::state(utx, o.nonce, o.contract, o.script_data, 0));
            inp
        }
    
}
// pub fn verify_transaction(tx: Transaction)-> Result<(), &'static str> {
    
    
//     match tx.tx_type {
//         TransactionType::Transfer => {
//             let tx_data = TransactionData::to_transfer(tx.tx).unwrap();
//             //convert Inputs and Outputs to Just Accounts
//             let inputs:Vec<Accounts> =  tx_data.inputs.iter().map(|i| )
//             match tx_data.shuffle_proof {
//                 Some(i) => {
//                     //verify QQ Transaction
//                     tx_data.verify_quisquis_tx(inputs, outputs)
//                 }
//                 None => {
//                     //verify Dark transaction
//                     tx_data.verify_dark_tx(inputs, outputs)
//                 }
//             }
//         } 
//             _  => Err("Not found"),
//     }
// }
// ------------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------------
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn create_transaction_test() {
        println!("IN TEST");
        //println!("{:?}",create_dark_reference_transaction());
        println!("{:?}", create_qq_reference_transaction())
    }
    #[test]
    fn create_genesis_block_test(){
        let utxo_set = create_genesis_block(1000, 100);
        println!("{:?}", utxo_set);
    }
}
