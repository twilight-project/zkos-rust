#![allow(non_snake_case)]
//#![deny(missing_docs)]

use crate::transfer_tx::{Transaction, TransactionData, TransactionType, TransferTransaction};
use address::{Address, Standard};
use curve25519_dalek::constants::RISTRETTO_BASEPOINT_COMPRESSED;
use curve25519_dalek::ristretto::{CompressedRistretto, RistrettoPoint};
use curve25519_dalek::scalar::Scalar;
use quisquislib::elgamal::ElGamalCommitment;
use quisquislib::{
    accounts::Account,
    keys::PublicKey,
    ristretto::{RistrettoPublicKey, RistrettoSecretKey},
};
use rand::rngs::OsRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha3::Sha3_512;
use zkvm::merkle::Hash;
use zkvm::zkos_types::{
    IOType, Input, InputData, Output, OutputCoin, OutputData, OutputMemo, OutputState, Utxo,
};

///Needed for Creating Reference transaction for Testing RPC
///

//Create Sender and receiver

#[derive(Debug, Clone)]
pub struct Receiver {
    amount: i64,
    acc: Account,
}
impl Receiver {
    pub fn set_receiver(amount: i64, acc: Account) -> Receiver {
        Receiver { amount, acc }
    }
}

#[derive(Debug, Clone)]
pub struct Sender {
    total_amount: i64,
    account: Account,
    receivers: Vec<Receiver>,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RecordUtxo {
    pub utx: Utxo,
    pub value: Output,
}
impl Sender {
    pub fn generate_value_and_account_vector(
        tx_vector: Vec<Sender>,
    ) -> Result<(Vec<i64>, Vec<Account>, usize, usize), &'static str> {
        if tx_vector.len() < 9 {
            let mut value_vector: Vec<i64> = tx_vector.iter().map(|s| s.total_amount).collect();
            let mut account_vector: Vec<Account> = tx_vector.iter().map(|s| s.account).collect();

            let senders_count: usize = tx_vector.iter().count();
            let mut receivers_count = 0;

            let mut receiver_amount_vector: Vec<i64> = Vec::new();
            let mut receiver_account_vector: Vec<Account> = Vec::new();

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

                Ok((value_vector, account_vector, senders_count, receivers_count))
            } else {
                Err("senders and receivers count should be less than 9")
            }
        } else {
            Err("account count is more than 9")
        }
    }

    //create anonymous account for anonymity set
    pub fn create_anonymity_set(
        senders_count: usize,
        receivers_count: usize,
    ) -> (Vec<Account>, Vec<Scalar>) {
        let mut value_vector: Vec<i64> = Vec::new();
        let mut account_vector: Vec<Account> = Vec::new();

        //keep track of all the r used for commitment of value zero
        let mut annonymity_account_commmitment_scalars_vector: Vec<Scalar> = Vec::new();

        // lets create anonymity set - these are randomly generated on the fly
        // this anonymity set may need to come from the blockchain state itself in the future

        let diff = 9 - (senders_count + receivers_count);
        //use random key as base pk for annonymity accounts
        let pk_g = RISTRETTO_BASEPOINT_COMPRESSED;
        let pk_h =
            RistrettoPoint::hash_from_bytes::<Sha3_512>(RISTRETTO_BASEPOINT_COMPRESSED.as_bytes())
                .compress();
        let pk_ref = RistrettoPublicKey::new_from_pk(pk_g, pk_h);
        let pk_annonymity = PublicKey::update_public_key(&pk_ref, Scalar::random(&mut OsRng));

        if diff >= 1 {
            for _ in 0..diff {
                value_vector.push(0);
                let (acc, comm_scalar) = Account::generate_account(PublicKey::update_public_key(
                    &pk_annonymity,
                    Scalar::random(&mut OsRng),
                ));
                account_vector.push(acc);
                annonymity_account_commmitment_scalars_vector.push(comm_scalar);
            }
        }
        (
            account_vector,
            annonymity_account_commmitment_scalars_vector,
        )
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
        //let (bob_account_2, bob_sk_account_2) =
        // Account::generate_random_account_with_value(20u64.into());

        // lets create receiver accounts
        let alice_account = Account::generate_random_account_with_value(0u64.into()).0;
        //let fay_account = Account::generate_random_account_with_value(0u64.into()).0;
        //let jay_account = Account::generate_random_account_with_value(0u64.into()).0;

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
            // //Sender {
            //     total_amount: -3,
            //     account: bob_account_2,
            //     receivers: vec![
            //         Receiver {
            //             amount: 2,
            //             acc: fay_account,
            //         },
            //         Receiver {
            //             amount: 1,
            //             acc: jay_account,
            //         },
            //     ],
            // },
        ];

        let (mut value_vector, mut account_vector, sender_count, receiver_count) =
            Sender::generate_value_and_account_vector(tx_vector)?;
        let (anonymity_account_vec, annonymity_com_scalar_vector) =
            Sender::create_anonymity_set(sender_count, receiver_count);

        let diff: usize = 9 - (sender_count + receiver_count);
        if diff >= 1 {
            for i in 0..diff {
                value_vector.push(0);
                account_vector.push(anonymity_account_vec[i].clone());
            }
        }

        //Create sender updated account vector for the verification of sk and bl-v
        let bl_first_sender = 10 - 5; //bl-v
                                      //let bl_second_sender = 20 - 3; //bl-v
        let updated_balance_sender: Vec<u64> = vec![bl_first_sender]; //, bl_second_sender];
                                                                      //Create vector of sender secret keys
        let sk_sender: Vec<RistrettoSecretKey> = vec![bob_sk_account_1]; //, bob_sk_account_2];

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
    pub fn set_sender(total_amount: i64, account: Account, receivers: Vec<Receiver>) -> Sender {
        Sender {
            total_amount,
            account,
            receivers,
        }
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

    let hash: Hash = Hash::default();
    //SHOULD BE TAKEN FROM UTXO SET
    let utxo = Utxo::from_hash(hash, 0);
    let utxo_vector: Vec<Utxo> = vec![
        utxo.clone(),
        utxo.clone(),
        utxo.clone(),
        utxo.clone(),
        utxo.clone(),
        utxo.clone(),
        utxo.clone(),
        utxo.clone(),
        utxo.clone(),
    ];

    let updated_balance_reciever: Vec<u64> = vec![5]; //, 2, 1];
                                                      //println!("Data : {:?}", sender_count);
                                                      //create quisquis transfertransaction
    let transfer = TransferTransaction::create_quisquis_transaction(
        &utxo_vector,
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
    println!("value {:#?}", value_vector);
    //create vector of inputs to be used in tx
    //random utxo IDS to be used in Inputs
    // let id: [u8; 32] = [0; 32];
    let accounts: &[Account] = &account_vector[..(sender_count + receiver_count)];
    let values: &[i64] = &value_vector[..(sender_count + receiver_count)];

    let hash: Hash = Hash::default();
    //SHOULD BE TAKEN FROM UTXO SET
    let utxo = Utxo::from_hash(hash, 0);
    //create vec of Inouts
    let mut inputs: Vec<Input> = Vec::new();
    for input in accounts.iter() {
        //create address
        let (pk, enc) = input.get_account();
        let out_coin = OutputCoin {
            encrypt: enc,
            owner: address::Address::standard_address(address::Network::Mainnet, pk).as_hex(),
        };
        let inp = Input::coin(InputData::coin(
            utxo,
            // address::Address::coin_address(address::Network::Mainnet, pk).as_hex(),
            //  enc,
            out_coin, 0,
        ));
        inputs.push(inp.clone());
    }

    let updated_balance_reciever: Vec<u64> = vec![5];
    //println!("Data : {:?}", sender_count);
    //create quisquis transfertransaction
    let transfer = TransferTransaction::create_dark_transaction(
        &values,
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

///Random Initialization of UTXO set for testing purposes
///
//Should be called first. Will only create a random set of outputs
//with random txIDs to kickstart the system
pub fn create_genesis_block(
    total_outputs: u32,

    num_tx: u32,

    base_account: Account,
) -> Vec<RecordUtxo> {
    //100000 outputs divided among 10000 txs

    let mut outputs: Vec<RecordUtxo> = Vec::with_capacity(total_outputs as usize);

    let mut rng = rand::thread_rng();

    let tot_outs_per_tx = total_outputs / num_tx;

    for i in 0..num_tx {
        // let id: [u8; 32] = [i.try_into().unwrap(); 32];

        let mut id: [u8; 32] = [0; 32];

        // Generate random values and fill the array

        rand::thread_rng().fill(&mut id);

        for j in 0..tot_outs_per_tx {
            let random_number: u32 = rng.gen_range(0u32, 3u32);

            if random_number == 0 {
                //coin output

                let utx = Utxo::from_hash(Hash(id), j.try_into().unwrap());

                //create new account with same commitment

                let (pk, enc) = Account::update_account(
                    base_account,
                    Scalar::from(0u64),
                    Scalar::random(&mut rng),
                    Scalar::random(&mut rng),
                )
                .get_account();

                // let (pk, enc) = Account::generate_random_account_with_value(Scalar::from(20u64)).0.get_account();

                let out = OutputData::Coin(OutputCoin {
                    encrypt: enc,

                    owner: address::Address::standard_address(address::Network::Mainnet, pk)
                        .as_hex(),
                });

                let output = Output::coin(out);

                outputs.push(RecordUtxo { utx, value: output });
            }

            if random_number == 1 {
                //memo output

                let utx = Utxo::from_hash(Hash(id), j.try_into().unwrap());

                //create dummy MemoOutput
                let out = Output::memo(OutputData::Memo(OutputMemo::default()));

                outputs.push(RecordUtxo { utx, value: out });
            }

            if random_number == 2 {
                //state output

                let utx = Utxo::from_hash(Hash(id), j.try_into().unwrap());
                //create dummy StateOutput
                let out = Output::state(OutputData::State(OutputState::default()));

                outputs.push(RecordUtxo {
                    utx: utx,

                    value: out,
                });
            }
        }
    }

    outputs
}
///utility function for converting output to input to help with testing
///
pub fn convert_output_to_input(rec: RecordUtxo) -> Option<Input> {
    let utx = rec.utx;

    let out = rec.value;

    let mut inp: Input;

    match out.out_type {
        IOType::Coin => {
            let out_coin = out.output.get_output_coin().unwrap().to_owned();
            inp = Input::coin(InputData::coin(utx, out_coin, 0));
            Some(inp)
        }

        IOType::Memo => {
            let out_memo = out.output.get_output_memo().unwrap().to_owned();
            inp = Input::memo(InputData::memo(
                utx,
                out_memo.clone(),
                0,
                zkvm::Commitment::Closed(CompressedRistretto::default()),
            ));
            Some(inp)
        }

        IOType::State => {
            let out_state = out.output.get_output_state().unwrap().to_owned();

            inp = Input::state(InputData::state(utx, out_state.clone(), None, 0));

            Some(inp)
        }

        _ => None,
    }
}
///Build for testing UTXo Set for Quisquis dummy transactions
pub fn create_dark_reference_tx_for_utxo_test(
    input: Input,
    sk_sender: &[RistrettoSecretKey],
) -> Transaction {
    let mut rng = rand::thread_rng();

    // so we have 1 senders and 2 receivers, rest will be the anonymity set
    let add_input: String = input.input.owner().unwrap().to_owned();

    let input_enc = input.input.as_encryption().unwrap().to_owned();
    let add = address::Address::from_hex(&add_input, address::AddressType::Standard).unwrap();

    let pk = add.get_standard_address().unwrap().public_key;

    let input_account = Account::set_account(pk, input_enc);

    //zero balance account

    let (acc, _) = Account::generate_account(pk);

    //coin output

    let out_acc_1 = Account::update_account(
        acc,
        Scalar::from(20u64),
        Scalar::random(&mut rng),
        Scalar::random(&mut rng),
    );

    let out_acc_2 = Account::update_account(
        acc,
        Scalar::from(0u64),
        Scalar::random(&mut rng),
        Scalar::random(&mut rng),
    );

    //let mut tx_vector: Vec<Sender> = Vec::new();

    let tx_vector: Vec<Sender> = vec![Sender {
        total_amount: 0,

        account: input_account,

        receivers: vec![
            Receiver {
                amount: 0,

                acc: out_acc_1,
            },
            Receiver {
                amount: 0,

                acc: out_acc_2,
            },
        ],
    }];

    let (value_vector, account_vector, sender_count, receiver_count) =
        Sender::generate_value_and_account_vector(tx_vector).unwrap();

    // println!("S = {:?}, R = {:?}", sender_count, receiver_count);

    //Create sender updated account vector for the verification of sk and bl-v

    //let bl_first_sender = 10 - 5; //bl-v

    //let bl_second_sender = 20 - 3; //bl-v

    let updated_balance_sender: Vec<u64> = vec![20];

    //Create vector of sender secret keys

    //let sk_sender: Vec<RistrettoSecretKey> = vec![priv_key.to_owned()];

    let updated_balance_reciever: Vec<u64> = vec![0, 0];

    //create quisquis transfertransaction

    let transfer = TransferTransaction::create_dark_transaction(
        &value_vector,
        &account_vector,
        &updated_balance_sender,
        &updated_balance_reciever,
        &vec![input],
        &sk_sender,
        sender_count,
        receiver_count,
    );

    Transaction::transaction_transfer(TransactionData::TransactionTransfer(transfer.unwrap()))
}

pub fn verify_transaction(tx: Transaction) -> Result<(), &'static str> {
    match tx.tx_type {
        TransactionType::Transfer => {
            let tx_data = TransactionData::to_transfer(tx.tx).unwrap();
            //convert Inputs and Outputs to Just Accounts
            let out_data_pk: Vec<RistrettoPublicKey> = tx_data
                .outputs
                .iter()
                .map(|i| Standard::from_hex(i.output.get_owner_address().unwrap()).public_key)
                .collect();
            let out_data_enc: Vec<ElGamalCommitment> = tx_data
                .outputs
                .iter()
                .map(|i| i.output.get_encryption().unwrap())
                .collect();

            let in_data_pk: Vec<RistrettoPublicKey> = tx_data
                .inputs
                .iter()
                .map(|i| Standard::from_hex(i.input.owner().unwrap()).public_key)
                .collect();

            let in_data_enc: Vec<ElGamalCommitment> = tx_data
                .inputs
                .iter()
                .map(|i| i.input.as_encryption().unwrap())
                .collect();
            let mut outputs: Vec<Account> = Vec::new();
            for i in 0..out_data_pk.len() {
                let acc: Account = Account::set_account(out_data_pk[i], out_data_enc[i]);
                outputs.push(acc);
            }

            let mut inputs: Vec<Account> = Vec::new();
            for i in 0..in_data_pk.len() {
                let acc: Account = Account::set_account(in_data_pk[i], in_data_enc[i]);
                inputs.push(acc);
            }

            if tx_data.shuffle_proof.is_none() {
                //verify Dark transaction
                tx_data.verify_dark_tx(&inputs, &outputs)
            } else {
                //verify QQ Transaction
                tx_data.verify_quisquis_tx(&inputs, &outputs)
            }
        }
        TransactionType::Script => Ok(()),
        _ => Err("Tx Verification failed. Transaction Type is not valid."),
    }
}
// ------------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------------
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn create_dark_transaction_test() {
        let dark = create_dark_reference_transaction();
        let verify_dark = verify_transaction(dark);
        println!("{:?}", verify_dark);

        assert!(verify_dark.is_ok());
    }
    #[test]
    fn create_qq_transaction_test() {
        println!("IN TEST");

        let tx: Transaction = create_qq_reference_transaction();
        let verify = verify_transaction(tx);
        println!("{:?}", verify)
    }
    #[test]
    fn create_genesis_block_test() {
        //create base test account
        let (acc, prv) = Account::generate_random_account_with_value(Scalar::from(20u64));
        let utxo_set = create_genesis_block(1000, 100, acc);
        println!("{:?}", utxo_set);
    }
}
