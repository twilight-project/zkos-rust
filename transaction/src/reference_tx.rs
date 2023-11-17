#![allow(non_snake_case)]
//#![deny(missing_docs)]

use crate::vm_run::{Prover, Verifier};
use crate::{verify_relayer, ScriptTransaction, Transaction, TransactionData, TransferTransaction};
use address::{Address, Network};
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
use zkvm::merkle::{CallProof, Hash};
use zkvm::zkos_types::{
    IOType, Input, InputData, Output, OutputCoin, OutputData, OutputMemo, OutputState,
    StateWitness, Utxo, ValueWitness,
};
use zkvm::{Commitment, Witness};
use zkvm::{Program, String as ZkvmString};

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
        // let mut value_vector: Vec<i64> = Vec::new();
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
                //value_vector.push(0);
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
    println!("Data : {:?}", sender_count);

    //create vec of Inputs -- Convert input accounts into Inputs
    let mut inputs: Vec<Input> = Vec::new();
    for inp in account_vector.iter() {
        let input =
            Input::input_from_quisquis_account(inp, utxo_vector[0], 0, address::Network::default());
        inputs.push(input.clone());
    }
    // create quisquis transfer transaction
    let transfer = TransferTransaction::create_quisquis_transaction(
        &inputs,
        &value_vector,
        &account_vector,
        &updated_sender_balance,
        &updated_balance_reciever,
        &sk_sender,
        sender_count,
        receiver_count,
        // &annonymity_com_scalar_vector,
        diff,
        None,
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
        None,
    );
    let (transfer, _comm_scalar) = transfer.unwrap();

    Transaction::transaction_transfer(TransactionData::TransactionTransfer(transfer))
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

    for _i in 0..num_tx {
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

                outputs.push(RecordUtxo { utx, value: out });
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

    let inp: Input;

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
        None,
    );
    let (transfer, _comm_scalar) = transfer.unwrap();
    Transaction::transaction_transfer(TransactionData::TransactionTransfer(transfer))
}

pub fn create_refenece_deploy_transaction(sk: RistrettoSecretKey, value_sats: u64) -> Transaction {
    let mut rng = rand::thread_rng();
    // commitment scalar for encryption and commitment
    let scalar_commitment = Scalar::random(&mut rng);
    // derive public key from secret key
    let pk = RistrettoPublicKey::from_secret_key(&sk, &mut rng);

    //create InputCoin and OutputMemo
    let enc_input =
        ElGamalCommitment::generate_commitment(&pk, scalar_commitment, Scalar::from(value_sats));
    let coin_address: Address = Address::standard_address(Network::default(), pk.clone());
    let out_coin = OutputCoin {
        encrypt: enc_input,
        owner: coin_address.as_hex(),
    };
    let in_data: InputData = InputData::coin(Utxo::default(), out_coin, 0);
    let coin_input: Input = Input::coin(in_data);
    let input_account: Account = Account::set_account(pk.clone(), enc_input.clone());
    //outputMemo
    let script_address = crate::verify_relayer::create_script_address(Network::default());
    let commit_memo = Commitment::blinded_with_factor(value_sats, scalar_commitment);

    let memo_out = OutputMemo {
        script_address: script_address.clone(),
        owner: coin_address.as_hex(),
        commitment: commit_memo.clone(),
        data: None,
        timebounds: 0,
    };
    let out_data = OutputData::Memo(memo_out);
    let memo = Output::memo(out_data);
    // create ValueWitness for input coin / output memo
    let value_witness = ValueWitness::create_value_witness(
        coin_input.clone(),
        sk,
        input_account,
        pk.clone(),
        commit_memo.to_point(),
        value_sats,
        scalar_commitment,
    );
    let s_var: ZkvmString = ZkvmString::Commitment(Box::new(commit_memo.clone()));
    let s_var_vec: Vec<ZkvmString> = vec![s_var];
    // create Output state
    let out_state: OutputState = OutputState {
        nonce: 1,
        script_address: script_address.clone(),
        owner: coin_address.as_hex(),
        commitment: commit_memo.clone(),
        state_variables: Some(s_var_vec),
        timebounds: 0,
    };
    // create zero value commitment
    let zero_commitment = Commitment::blinded_with_factor(0, scalar_commitment);
    let in_state_var = ZkvmString::Commitment(Box::new(zero_commitment.clone()));
    let in_state_var_vec: Vec<ZkvmString> = vec![in_state_var];
    // create Input State
    let temp_out_state = OutputState {
        nonce: 0,
        script_address: script_address.clone(),
        owner: coin_address.as_hex(),
        commitment: zero_commitment.clone(),
        state_variables: Some(in_state_var_vec),
        timebounds: 0,
    };
    let zero_proof = vec![scalar_commitment, scalar_commitment];
    // convert to input
    let input_state: Input = Input::state(InputData::state(
        Utxo::default(),
        temp_out_state.clone(),
        None,
        1,
    ));
    // create statewitness for input state / output state
    let state_witness: StateWitness =
        StateWitness::create_state_witness(input_state.clone(), sk, pk, Some(zero_proof));

    // create witness vector
    let witness: Vec<Witness> = vec![
        Witness::ValueWitness(value_witness),
        Witness::State(state_witness),
    ];
    let output: Vec<Output> = vec![memo, Output::state(OutputData::State(out_state))];
    let temp_out_state_verifier = temp_out_state.verifier_view();
    let iput_state_verifier = Input::state(InputData::state(
        Utxo::default(),
        temp_out_state_verifier.clone(),
        None,
        1,
    ));
    let input: Vec<Input> = vec![coin_input, iput_state_verifier];

    // create proof of program
    let correct_program = verify_relayer::contract_initialize_program_with_stack_short();
    //cretae unsigned Tx with program proof
    let result = Prover::build_proof(correct_program, &input, &output, true);
    let (prog_bytes, proof) = result.unwrap();

    // create callproof
    let call_proof = verify_relayer::create_call_proof(Network::default());
    //lets create a script tx
    let script_tx: ScriptTransaction = ScriptTransaction {
        version: 0,
        fee: 0,
        maturity: 0,
        input_count: 2,
        output_count: 2,
        witness_count: 2,
        inputs: input.to_vec(),
        outputs: output.to_vec(),
        program: prog_bytes.to_vec(),
        call_proof,
        proof,
        witness: Some(witness.to_vec()),
        data: vec![],
    };

    let tx = Transaction::transaction_script(TransactionData::TransactionScript(script_tx));
    tx
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
        // Given its a dark transaction
        let tx_dark = TransactionData::to_transfer(dark.tx).unwrap();
        let verify_dark = tx_dark.verify();
        println!("{:?}", verify_dark);

        assert!(verify_dark.is_ok());
    }
    #[test]
    fn create_qq_transaction_test() {
        println!("IN TEST");

        let tx: Transaction = create_qq_reference_transaction();
        // let verify = verify_transaction(tx);
        // println!("{:?}", verify)
        // Given its a qq transaction
        let tx_qq = TransactionData::to_transfer(tx.tx).unwrap();
        let verify_qq = tx_qq.verify();
        println!("{:?}", verify_qq);
    }
    #[test]
    fn create_genesis_block_test() {
        //create base test account
        let (acc, _prv) = Account::generate_random_account_with_value(Scalar::from(20u64));
        let utxo_set = create_genesis_block(1000, 100, acc);
        println!("{:?}", utxo_set);
    }
    #[test]
    fn test_deploy_contract() {
        let sk = <RistrettoSecretKey as quisquislib::keys::SecretKey>::random(&mut OsRng);
        let value_sats = 1000;
        let tx = create_refenece_deploy_transaction(sk, value_sats);
        println!("Tx  {:?}\n", tx);
        let verify: Result<(), &str> = tx.verify();
        println!("Verify  {:?}\n", verify);
    }

    #[test]
    fn create_chain_deploy_tx() {
        let seed = "r5Mbx5dlqyKTBYXbV5DAWkUQRh54q6YrwFdDJbItxlwLwmRBAoCC/UeEBtDxAvggemy57z4N/uxIzuQkxkLKdA==";
        let sk: RistrettoSecretKey = quisquislib::keys::SecretKey::from_bytes(seed.as_bytes());
        println!("sk {:?}", sk);
        let json_string = r#"{"out_type":"Coin","output":{"Coin":{"encrypt":{"c":[98,190,90,209,45,99,255,242,71,6,224,47,247,165,80,157,27,128,218,146,250,85,139,9,202,26,6,13,156,104,31,64],"d":[188,89,231,30,47,64,107,237,76,201,73,31,89,171,209,156,220,122,152,204,92,128,4,43,117,139,202,213,66,212,129,106]},"owner":"0cb6dccc85e2ca3e418e256843aa02ca89c94293ef4055c34b23ddbda1d12119244061ba6015191b780782a9355981e1de0fe89d5ba95a407d6425ee05272ab66fe4932ea4"}}}"#;
        let out: Output = serde_json::from_str(json_string).unwrap();
        let account: Account = out.as_out_coin().unwrap().to_quisquis_account();
        let (pk, _enc) = account.get_account();
        let verify_acc = account.verify_account(&sk, Scalar::from(100u64));
        println!("verify_acc {:?}", verify_acc);

        // create Utxo
        let utxo_str = "33f169a4151acdef806803bb9221c54364541cbd549064e269988cdac42dc5d800";
        let utxo_bytes = hex::decode(&utxo_str.to_string()).unwrap();
        let utxo: Utxo = bincode::deserialize(&utxo_bytes).unwrap();
        println!("utxo {:?}", utxo);
        println!("out {:?}", out);
        let out_coin = out.as_out_coin().unwrap().to_owned();
        //create input coin
        let inp_coin = Input::coin(InputData::coin(utxo, out_coin.clone(), 0));
        // recreate scalar used for coin encryption
        let scalar_str =
            "3b9f445a368c75336ae69bd39c2441473a3fab22f549d6f09b8baf9e0b790509".to_string();
        let scalar_bytes = hex::decode(&scalar_str).unwrap();
        let scalar_commitment = Scalar::from_bytes_mod_order(scalar_bytes.try_into().unwrap());
        println!("scalar {:?}", scalar_commitment);

        // create out memo
        let script_address = crate::verify_relayer::create_script_address(Network::default());
        let commit_memo = Commitment::blinded_with_factor(100u64, scalar_commitment);
        let coin_address = out_coin.owner.clone();
        let memo_out = OutputMemo {
            script_address: script_address.clone(),
            owner: coin_address.clone(),
            commitment: commit_memo.clone(),
            data: None,
            timebounds: 0,
        };
        let memo = Output::memo(OutputData::Memo(memo_out));

        // create ValueWitness for input coin / output memo
        let value_witness = ValueWitness::create_value_witness(
            inp_coin.clone(),
            sk,
            account,
            pk.clone(),
            commit_memo.to_point(),
            100u64,
            scalar_commitment,
        );
        let s_var: ZkvmString = ZkvmString::Commitment(Box::new(commit_memo.clone()));
        let s_var_vec: Vec<ZkvmString> = vec![s_var];
        // create Output state
        let out_state: OutputState = OutputState {
            nonce: 1,
            script_address: script_address.clone(),
            owner: coin_address.clone(),
            commitment: commit_memo.clone(),
            state_variables: Some(s_var_vec),
            timebounds: 0,
        };
        // create zero value commitment
        let zero_commitment = Commitment::blinded_with_factor(0, scalar_commitment);
        let in_state_var = ZkvmString::Commitment(Box::new(zero_commitment.clone()));
        let in_state_var_vec: Vec<ZkvmString> = vec![in_state_var];
        // create Input State
        let temp_out_state = OutputState {
            nonce: 0,
            script_address: script_address.clone(),
            owner: coin_address,
            commitment: zero_commitment.clone(),
            state_variables: Some(in_state_var_vec),
            timebounds: 0,
        };
        let zero_proof = vec![scalar_commitment, scalar_commitment];
        // convert to input
        let input_state: Input = Input::state(InputData::state(
            Utxo::default(),
            temp_out_state.clone(),
            None,
            1,
        ));

        // create statewitness for input state / output state
        let state_witness: StateWitness =
            StateWitness::create_state_witness(input_state.clone(), sk, pk, Some(zero_proof));

        // create witness vector
        let witness: Vec<Witness> = vec![
            Witness::ValueWitness(value_witness),
            Witness::State(state_witness),
        ];
        let output: Vec<Output> = vec![memo, Output::state(OutputData::State(out_state))];
        let temp_out_state_verifier = temp_out_state.verifier_view();
        let iput_state_verifier = Input::state(InputData::state(
            Utxo::default(),
            temp_out_state_verifier.clone(),
            None,
            1,
        ));
        let input: Vec<Input> = vec![inp_coin, iput_state_verifier];
        // create proof of program
        let correct_program = verify_relayer::contract_initialize_program_with_stack_short();
        //cretae unsigned Tx with program proof
        let result = Prover::build_proof(correct_program, &input, &output, true);
        let (prog_bytes, proof) = result.unwrap();

        // create callproof
        let call_proof = verify_relayer::create_call_proof(Network::default());
        //lets create a script tx
        let script_tx: ScriptTransaction = ScriptTransaction {
            version: 0,
            fee: 0,
            maturity: 0,
            input_count: 2,
            output_count: 2,
            witness_count: 2,
            inputs: input.to_vec(),
            outputs: output.to_vec(),
            program: prog_bytes.to_vec(),
            call_proof,
            proof,
            witness: Some(witness.to_vec()),
            data: vec![],
        };

        let tx = Transaction::transaction_script(TransactionData::TransactionScript(script_tx));
        //convert tx to hex
        let tx_bin = bincode::serialize(&tx).unwrap();
        let tx_hex = hex::encode(&tx_bin);
        println!("tx_hex {:?}", tx_hex);
        // let utx_json_string: &str = r#"{"output_index":0,"txid":[244,204,253,20,214,243,15,14,203,150,116,42,136,177,47,144,66,40,172,147,241,89,62,63,135,52,198,173,59,71,127,119]}"#;
        // let utxxx: Utxo = serde_json::from_str(utx_json_string).unwrap();
        // println!("utxxx {:?}", utxxx);
        // let utxx_bytes = bincode::serialize(&utxxx).unwrap();
        // let utxx_hex = hex::encode(&utxx_bytes);
        // println!("utxx_hex {:?}", utxx_hex);
    }
}
