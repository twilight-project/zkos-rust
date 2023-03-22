#![allow(non_snake_case)]
//#![deny(missing_docs)]

use crate::proof::{DarkTxProof, ShuffleTxProof};
use crate::types::{Input, Output, OutputData, TransactionType, TxEntry, TxLog, Witness};
use crate::util::{Address, Network};
use merlin::Transcript;
// use serde_derive::{Deserialize, Serialize};
use serde::{Deserialize, Serialize};

use curve25519_dalek::scalar::Scalar;
use quisquislib::{
    accounts::prover::Prover,
    accounts::verifier::Verifier,
    accounts::Account,
    keys::PublicKey,
    ristretto::{RistrettoPublicKey, RistrettoSecretKey},
    shuffle::Shuffle,
};




///Needed for Creating Reference transaction for Testing RPC
/// 

//Create Sender and receiver

#[derive(Debug, Clone)]
pub struct Receiver {
    amount: i64,
    public_key: RistrettoPublicKey,
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
                    let (receiver_account, _) = Account::generate_account(rec.public_key);
                    receiver_account_vector.push(receiver_account);
                }
            }

            if senders_count < 9 && receivers_count < 9 && senders_count + receivers_count <= 9 {
                value_vector.append(&mut receiver_amount_vector);
                account_vector.append(&mut receiver_account_vector);

                // lets create anonymity set - these are randomly generated on the fly
                // this anonymity set may need to come from the blockchain state itself in the future

                let diff = 9 - (senders_count + receivers_count);
                //use sender key as base pk for annonymity accounts
                let pk_annonymity =
                    PublicKey::update_public_key(&account_vector[0].pk, Scalar::random(&mut OsRng));

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


    pub fn create_reference_tx_data_for_zkos_test() -> Result<(Vec<i64>, Vec<Account>, Vec<Scalar>, usize, usize, usize, Vec<RistrettoSecretKey>, Vec<i64>), &'static str>{
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
                       public_key: alice_account.,
                   }],
               },
               Sender {
                   total_amount: -3,
                   account: bob_account_2,
                   receivers: vec![
                       Receiver {
                           amount: 2,
                           public_key: fay_account.pk,
                       },
                       Receiver {
                           amount: 1,
                           public_key: jay_account.pk,
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
           let updated_balance_sender: Vec<i64> = vec![bl_first_sender, bl_second_sender];
           //Create vector of sender secret keys
           let sk_sender: Vec<RistrettoSecretKey> = vec![bob_sk_account_1, bob_sk_account_2];
           
           Ok((
                       value_vector,
                       account_vector,
                       annonymity_account_commmitment_scalars_vector,
                       diff,
                       senders_count,
                       receivers_count,
                       sk_sender,
                       updated_balance_sender,    
                   ))
                }
}


pub fn create_transaction(){
                let (
                    value_vector,
                    account_vector,
                    annonymity_com_scalar_vector,
                    diff,
                    sender_count,
                    receiver_count,
                    sk_sender,
                    updated_sender_balance,
                ) = Sender::generacreate_reference_tx_data_for_zkos_test();

                println("Data : {:?}", sender_count)
            }


            // ------------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------------
#[cfg(test)]
mod test {
     use super::*;
    #[test]
    fn create_transaction_test() {
        println("IN TEST");
        create_transaction();
    }
}