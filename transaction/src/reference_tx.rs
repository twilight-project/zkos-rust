#![allow(non_snake_case)]
//#![deny(missing_docs)]

use crate::tx::{ScriptTransaction, Transaction, TransactionData, TransferTransaction};
use crate::types::{
    CData, Coin, Input, InputData, Memo, Output, OutputData, OutputType, State, TransactionType,
    TxEntry, TxId, TxLog, Utxo, Witness,
};
use crate::util::{Address, Network};
use quisquislib::elgamal::ElGamalCommitment;
// use serde_derive::{Deserialize, Serialize};
use serde::{Deserialize, Serialize};

use curve25519_dalek::constants::RISTRETTO_BASEPOINT_COMPRESSED;
use curve25519_dalek::ristretto::{CompressedRistretto, RistrettoPoint};
use curve25519_dalek::scalar::Scalar;
use quisquislib::{
    accounts::Account,
    keys::PublicKey,
    ristretto::{RistrettoPublicKey, RistrettoSecretKey},
};
use rand::rngs::OsRng;

use rand::Rng;
use sha3::Sha3_512;
use bincode::serialize;

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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RecordUtxo {
    pub utx: Utxo,
    pub value: Output,
}

//#[derive(Debug, Clone)]
// pub struct UtxoSet {
//     pub set: Vec<RecordUtxo>,
// }

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
        let add: Address = Address::standard(Network::default(), pk);
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
    Transaction::transaction_transfer(
        TxId(id),
        TransactionData::TransactionTransfer(transfer.unwrap()),
    )
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
        let add: Address = Address::standard(Network::default(), pk);
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
    Transaction::transaction_transfer(
        TxId(id),
        TransactionData::TransactionTransfer(transfer.unwrap()),
    )
}

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
                let utx = Utxo::new(TxId(id), j.try_into().unwrap());
                //create new account with same commitment
                let (pk, enc) = Account::update_account(
                    base_account,
                    Scalar::from(0u64),
                    Scalar::random(&mut rng),
                    Scalar::random(&mut rng),
                )
                .get_account();
                // let (pk, enc) = Account::generate_random_account_with_value(Scalar::from(20u64)).0.get_account();
                let out = OutputData::Coin(Coin {
                    encrypt: enc,
                    address: Address::standard(Network::default(), pk),
                });
                let output = Output::coin(out);
                outputs.push(RecordUtxo {
                    utx: utx,
                    value: output,
                });
            }

            if random_number == 1 {
                //memo output
                let utx = Utxo::new(TxId(id), j.try_into().unwrap());
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

                outputs.push(RecordUtxo {
                    utx: utx,
                    value: out,
                });
            }
            if random_number == 2 {
                //state output
                let utx = Utxo::new(TxId(id), j.try_into().unwrap());
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
                outputs.push(RecordUtxo {
                    utx: utx,
                    value: out,
                });
            }
        }
    }
    outputs
}

pub fn convert_output_to_input(rec: RecordUtxo) -> Option<Input> {
    let utx = rec.utx;
    let out = rec.value;
    let mut inp: Input;
    match out.out_type {
        OutputType::Coin => {
            let add = out.output.get_owner_address().unwrap().to_owned();
            let enc: ElGamalCommitment = out.output.get_encryption().unwrap().to_owned();
            inp = Input::coin(InputData::coin(utx, add, enc, 0));
            Some(inp)
        }
        OutputType::Memo => {
            let add = out.output.get_owner_address().unwrap().to_owned();
            let com: CompressedRistretto = out.output.get_commitment().unwrap().to_owned();
            inp = Input::memo(InputData::memo(utx, add.clone(), add.clone(), com, None, 0));
            Some(inp)
        }
        OutputType::State => {
            let add = out.output.get_owner_address().unwrap().to_owned();
            let com: CompressedRistretto = out.output.get_commitment().unwrap().to_owned();
            inp = Input::state(InputData::state(
                utx,
                0,
                add.clone(),
                add.clone(),
                com,
                None,
                0,
                0,
            ));
            Some(inp)
        }
        _ => None,
    }

    // if out.out_type.is_coin(){
    //     let add = out.output.get_owner_address().unwrap().to_owned();
    //     let enc: ElGamalCommitment = out.output.get_encryption().unwrap().to_owned();
    //     inp = Input::coin(InputData::coin(utx, add, enc, 0));
    //     Some(inp)
    // }
    // if out.out_type.is_memo(){
    //     let add = out.output.get_owner_address().unwrap().to_owned();
    //     let com: CompressedRistretto = out.output.get_commitment().unwrap().to_owned();
    //         inp = Input::memo(InputData::memo(utx, add.clone(), add.clone(), com, None, 0));
    //         Some(inp)
    //     }
    //     if out.out_type.is_state(){
    //         let add = out.output.get_owner_address().unwrap().to_owned();
    //         let com: CompressedRistretto = out.output.get_commitment().unwrap().to_owned();
    //         inp = Input::state(InputData::state(utx, 0, add.clone(), add, com,None,0,0));
    //         Some(inp)
    //     }
    //     return inp;
}

pub fn create_dark_reference_tx_for_utxo_test(
    input: Input,
    sk_sender: &[RistrettoSecretKey],
) -> Transaction {
    let mut rng = rand::thread_rng();
    // so we have 1 senders and 2 receivers, rest will be the anonymity set
    let add_input: Address = input.input.as_owner().unwrap().to_owned();
    let input_enc = input.input.as_encryption().unwrap().to_owned();
    let pk = add_input.public_key;
    let input_account = Account::set_account(pk, input_enc);
    //zero balance account
    let (acc, _) = Account::generate_account(add_input.public_key);
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

    let (value_vector, account_vector, _, _, sender_count, receiver_count) =
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
    let mut id: [u8; 32] = [0; 32];
    rand::thread_rng().fill(&mut id);
    Transaction::transaction_transfer(
        TxId(id),
        TransactionData::TransactionTransfer(transfer.unwrap()),
    )
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
    fn create_genesis_block_test() {
        //create base test account
        let (acc, prv) = Account::generate_random_account_with_value(Scalar::from(20u64));
        let utxo_set = create_genesis_block(1000, 100, acc);
        println!("{:?}", utxo_set);
    }
}
