#![allow(non_snake_case)]
//#![deny(missing_docs)]

use crate::tx::{Transaction, TransactionData, TransferTransaction};
use crate::types::{
    Input, InputData, Output, OutputData, TransactionType, TxEntry, TxId, TxLog, Utxo, Witness,
};

// use serde_derive::{Deserialize, Serialize};
use serde::{Deserialize, Serialize};

use curve25519_dalek::constants::RISTRETTO_BASEPOINT_COMPRESSED;
use curve25519_dalek::ristretto::RistrettoPoint;
use curve25519_dalek::scalar::Scalar;
use rand::rngs::OsRng;
use sha3::Sha3_512;

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
        //create address

        let inp = Input::coin(InputData::coin_dark(utxo, *input));
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

    Transaction::transaction_transfer(TransactionData::TransactionTransfer(transfer.unwrap()))
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
        //create address

        let inp = Input::coin(InputData::coin_dark(utxo, *input));
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

    Transaction::transaction_transfer(TransactionData::TransactionTransfer(transfer.unwrap()))
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
}