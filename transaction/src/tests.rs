// Unit tests for transaction module
use crate::vm_run::{Prover, Verifier};

use address::{Address, Network};
use curve25519_dalek::scalar::Scalar;
use quisquislib::accounts::Account;
use quisquislib::elgamal::ElGamalCommitment;
use quisquislib::{
    // accounts::prover::Prover as QuisquisProver,
    keys::{PublicKey, SecretKey},
    ristretto::{RistrettoPublicKey, RistrettoSecretKey},
};
use readerwriter::Encodable;
use zkvm::merkle::{CallProof, Hasher, MerkleTree, Path};
use zkvm::zkos_types::{
    Input, InputData, Output, OutputCoin, OutputData, OutputMemo, OutputState, Utxo,
};
use zkvm::{Commitment, Program, String};

#[test]
fn call_proof_test() {
    // create a tree of programs
    let hasher = Hasher::new(b"ZkOS.MerkelTree");

    let prog1 = Program::build(|p| {
        p.drop();
    });
    let prog2 = Program::build(|p| {
        p.dup(1);
    });
    let prog3 = Program::build(|p| {
        p.roll(0);
    });
    let prog4 = Program::build(|p| {
        p.push(Commitment::blinded(10u64));
        p.push(Commitment::blinded(20u64));
        p.push(10);
        p.push(15);
        p.roll(1);
    });
    let prog5 = Program::build(|p| {
        p.push(Commitment::blinded(5u64));
        p.dup(1);
    });

    let prog6 = order_message_prog_with_stack_initialized();
    println!("prog4: {:?}", prog4);
    // Serialize the program
    let mut bytecode = Vec::new();
    //let prog7 = prog6.encode(&mut bytecode).unwrap();
    //println!("prog7: {:?}", bytecode);
    // recreate program from bytes
    let prog8 = Program::parse(&bytecode).unwrap();
    println!("prog8: {:?}", prog8);
    let progs = vec![
        prog1.clone(),
        prog2.clone(),
        prog3.clone(),
        prog4.clone(),
        prog5.clone(),
        //prog6.clone(),
    ];

    //create tree root
    let root = MerkleTree::root(b"ZkOS.MerkelTree", progs.iter());
    //convert root to address
    let address = Address::script_address(Network::default(), root.0);
    //script address as hex
    let address_hex = address.as_hex();
    //let address_default = Address::default().as_hex();
    // create path for program3

    // let _path = Path::new(&progs, 2 as usize, &hasher).unwrap();

    // create call proof for program3
    let call_proof =
        CallProof::create_call_proof(&progs, 2 as usize, &hasher, Network::default()).unwrap();

    // verify call proof
    let prog = prog3.clone();
    let verify = call_proof.verify_call_proof(address_hex, &prog, &hasher);
    println!("verify: {:?}", verify);
}
pub fn program_roll() -> Program {
    let prog4 = Program::build(|p| {
        p.push(5);
        p.push(7);
        p.push(10);
        p.push(15);
        p.dup(2);
    });
    prog4
}
#[test]
fn order_message_test() {
    let _program = order_message_prog_input_output(16u64, 9u64, 0, 0);
    let correct_program = self::order_message_prog(16u64, 9u64);
    //let correct_program = self::lend_order_initial_dup_test_stack_initialized();
    //useless predicates
    //let (preds, scalars) = generate_predicates(3);
    //create input and output array

    let mut rng = rand::thread_rng();
    let sk_in: RistrettoSecretKey = SecretKey::random(&mut rng);
    let pk_in = RistrettoPublicKey::from_secret_key(&sk_in, &mut rng);
    let commit_in = ElGamalCommitment::generate_commitment(
        &pk_in,
        Scalar::random(&mut rng),
        Scalar::from(10u64),
    );
    let add: Address = Address::standard_address(Network::default(), pk_in.clone());
    let out_coin = OutputCoin {
        encrypt: commit_in,
        owner: add.as_hex(),
    };
    let in_data: InputData = InputData::coin(
        Utxo::default(),
        /*  add.as_hex(), commit_in*/ out_coin,
        0,
    );
    let coin_in: Input = Input::coin(in_data);
    let input: Vec<Input> = vec![coin_in];
    let sk_out: RistrettoSecretKey = SecretKey::random(&mut rng);
    let pk_out = RistrettoPublicKey::from_secret_key(&sk_out, &mut rng);
    let add_out: Address = Address::standard_address(Network::default(), pk_out);
    let commit_out = ElGamalCommitment::generate_commitment(
        &pk_out,
        Scalar::random(&mut rng),
        Scalar::from(5u64),
    );
    let coin_out = OutputCoin {
        encrypt: commit_out,
        owner: add_out.as_base58(),
    };
    let out_data = OutputData::Coin(coin_out);
    let coin_out = Output::coin(out_data);
    let output: Vec<Output> = vec![coin_out];

    //cretae unsigned Tx with program proof
    let result = Prover::build_proof(correct_program, &input, &output, false, None);
    println!("{:?}", result);
    let (prog_bytes, proof) = result.unwrap();
    let verify = Verifier::verify_r1cs_proof(&proof, &prog_bytes, &input, &output, false, None);
    println!("{:?}", verify);
}
#[test]
fn test_contract_deploy_stack() {
    let correct_program = self::contract_initialize_program_with_stack_short();
    print!("\n Program \n{:?}", correct_program);
    let mut rng = rand::thread_rng();
    let sk_in: RistrettoSecretKey = SecretKey::random(&mut rng);
    let pk_in = RistrettoPublicKey::from_secret_key(&sk_in, &mut rng);
    let commit_in = ElGamalCommitment::generate_commitment(
        &pk_in,
        Scalar::random(&mut rng),
        Scalar::from(10u64),
    );
    let add: Address = Address::standard_address(Network::default(), pk_in.clone());
    let out_coin = OutputCoin {
        encrypt: commit_in,
        owner: add.as_hex(),
    };
    let in_data: InputData = InputData::coin(
        Utxo::default(),
        /*  add.as_hex(), commit_in*/ out_coin,
        0,
    );
    let coin_in: Input = Input::coin(in_data);
    //get a output memo
    //outputMemo
    let script_address = crate::verify_relayer::create_script_address(Network::default());
    let commit_memo: Commitment = Commitment::blinded(10u64);

    let memo_out = OutputMemo {
        script_address: script_address.clone(),
        owner: add.as_hex(),
        commitment: commit_memo.clone(),
        data: None,
        timebounds: 0,
    };
    let out_data = OutputData::Memo(memo_out);
    let memo = Output::memo(out_data);

    //create state variables
    //let s_var: String = String::Commitment(Box::new(Commitment::blinded(15)));
    let s_var: String = String::Commitment(Box::new(commit_memo.clone()));
    let s_var_vec: Vec<String> = vec![s_var];

    // create zero value commitment
    let zero_commitment = Commitment::blinded(0);
    // create Output state
    let out_state: OutputState = OutputState {
        nonce: 1,
        script_address: script_address.clone(),
        owner: add.as_hex(),
        commitment: commit_memo.clone(),
        state_variables: Some(s_var_vec),
        timebounds: 0,
    };

    // create Input State
    let temp_out_state = OutputState {
        nonce: 0,
        script_address: script_address.clone(),
        owner: add.as_hex(),
        commitment: zero_commitment.clone(),
        state_variables: None,
        timebounds: 0,
    };

    // convert to input
    let input_state: Input =
        Input::state(InputData::state(Utxo::default(), temp_out_state, None, 1));

    let input: Vec<Input> = vec![coin_in, input_state];

    let output: Vec<Output> = vec![memo, Output::state(OutputData::State(out_state))];

    //cretae unsigned Tx with program proof
    let result = Prover::build_proof(correct_program, &input, &output, true, None);
    println!("{:?}", result);
    let (prog_bytes, proof) = result.unwrap();

    let verify = Verifier::verify_r1cs_proof(&proof, &prog_bytes, &input, &output, true, None);
    println!("{:?}", verify);
    //let (prog_bytes, proof) = result.unwrap();
}

fn order_message_prog(balance: u64, order_qty: u64) -> Program {
    let order_prog = Program::build(|p| {
        p.push(Commitment::blinded(balance))
            .commit()
            .expr()
            .push(order_qty)
            .scalar()
            .neg()
            .add()
            .range()
            .drop();
    });
    return order_prog;
}

fn contract_initialize_program() -> Program {
    let order_prog = Program::build(|p| {
        p.push(Commitment::blinded(100u64))
            .commit()
            .expr()
            .push(Commitment::blinded(100u64))
            .commit()
            .expr()
            .neg()
            .add()
            .push(Commitment::blinded(100u64))
            .commit()
            .expr()
            .push(Commitment::blinded(100u64))
            .commit()
            .expr()
            .neg()
            .add()
            .eq()
            .verify();
    });
    return order_prog;
}

fn contract_initialize_program_with_stack() -> Program {
    let order_prog = Program::build(|p| {
        p.dup(2)
            .commit()
            .expr()
            .roll(1)
            .commit()
            .expr()
            .neg()
            .add()
            .roll(2)
            .commit()
            .expr()
            .roll(1)
            .commit()
            .expr()
            .neg()
            .add()
            .eq()
            .verify();
    });
    return order_prog;
}
fn contract_initialize_program_with_stack_short() -> Program {
    let order_prog = Program::build(|p| {
        p.drop()
            .commit()
            .expr()
            .roll(1)
            .commit()
            .expr()
            .neg()
            .add()
            .range()
            .drop();
    });
    return order_prog;
}
fn order_message_prog_with_stack_initialized() -> Program {
    let order_prog = Program::build(|p| {
        p.commit()
            .expr()
            .neg()
            .roll(1)
            .commit()
            .expr()
            .add()
            .range()
            .drop();
    });
    return order_prog;
}

fn lend_order_initial_dup_test_stack_initialized() -> Program {
    let order_prog = Program::build(|p| {
        p.roll(7)
            .commit()
            .expr()
            .dup(7)
            .commit()
            .expr()
            .neg()
            .add()
            .range()
            .drop() // drop the rangeproof expression
            // TPS1 - TPS0 = PS or TPS1 = PS + TPS0
            .roll(1) //TPS1
            .commit()
            .expr()
            .dup(2) // TPS0
            .commit()
            .expr()
            .dup(6) // Poolshare
            .commit()
            .expr()
            .add() //
            .eq() //  TPS0 + PoolShare = TPS1
            //.and() // range && TPS0 + PoolShare = TPS1
            .roll(3) //TLV1
            .commit()
            .expr()
            .dup(4) //TLV0
            .commit()
            .expr()
            .dup(7) // Deposit
            .commit()
            .expr()
            .add() //Deposit + tlv
            .eq() // TLV1 = Deposit + TLV0
            .and() // TPS== &&  TLV== &&
            .roll(1) // error
            .commit()
            .expr()
            .roll(2) // TPS0
            .commit()
            .expr()
            .roll(5) //Deposit
            .commit()
            .expr()
            .mul() //Deposit * TPS0
            .add() // Deposit * TPS0 + error
            .roll(2) // TVL0
            .commit()
            .expr()
            .roll(3) // Poolshare
            .commit()
            .expr()
            .mul() // TVL0 * Poolshare
            .eq()
            .and()
            .verify();
        //.drop()
        //.drop();
    });

    return order_prog;
}

fn settle_order_test_stack_initialized() -> Program {
    let order_prog = Program::build(|p| {
        p.roll(7)
            .drop() //drop amount. Not needed
            .dup(0) // duplicate the price
            .roll(7) // get IM
            .commit()
            .expr()
            .roll(8) // get CM
            .commit()
            .expr()
            .neg() // -CM
            .add() // IM - CM
            .roll(1) //get price
            .scalar()
            .mul() // price * (IM - CM)
            .dup(2) //duplicate the payment
            .roll(2) //get the price
            .scalar()
            .mul() // price * payment
            .eq() // price * payment == price * (IM - CM)
            .roll(3) //get the TVL 1
            .commit()
            .expr()
            .roll(4) //get the TVL 0
            .commit()
            .expr()
            .neg()
            .add() // TVL 1 - TVL 0
            .dup(2) // duplicate the payment
            .scalar()
            .expr()
            .eq() //payment == TVL 1 - TVL 0
            .add() // price * (IM - CM) + payment == price * (IM - CM) + TVL 1 - TVL 0
            .roll(2)
            .commit()
            .expr()
            .roll(3)
            .commit()
            .expr()
            .neg()
            .add()
            .roll(2)
            .scalar()
            .expr()
            .mul() // TVL0 * Poolshare
            .eq()
            .and()
            .verify();
    });

    return order_prog;
}

fn order_message_prog_input_output(
    balance: u64,
    order_qty: u64,
    in_index: usize,
    out_index: usize,
) -> Program {
    let order_prog = Program::build(|p| {
        p.push(Commitment::blinded(balance))
            .commit()
            .expr()
            .push(order_qty)
            .scalar()
            .neg()
            .add()
            .range()
            .drop()
            .inputcoin(in_index)
            .outputcoin(out_index)
            .drop()
            .drop();
    });
    return order_prog;
}

#[test]
fn trade_order_tx_input_output_test() {
    let _program = order_message_prog_input_output(16u64, 9u64, 0, 0);
    // let correct_program = self::order_message_prog_with_stack_initialized();
    let correct_program = self::lend_order_initial_dup_test_stack_initialized();
    println!("\n Program \n{:?}", correct_program);

    //create InputCoin and OutputMemo

    let mut rng = rand::thread_rng();
    let sk_in: RistrettoSecretKey = SecretKey::random(&mut rng);
    let pk_in = RistrettoPublicKey::from_secret_key(&sk_in, &mut rng);
    let commit_in = ElGamalCommitment::generate_commitment(
        &pk_in,
        Scalar::random(&mut rng),
        Scalar::from(10u64),
    );
    let add: Address = Address::standard_address(Network::default(), pk_in.clone());
    let out_coin = OutputCoin {
        encrypt: commit_in,
        owner: add.as_hex(),
    };
    let in_data: InputData = InputData::coin(Utxo::default(), out_coin, 0);
    let coin_in: Input = Input::coin(in_data);
    let input: Vec<Input> = vec![coin_in];
    //outputMemo
    let script_address =
        Address::script_address(Network::Mainnet, *Scalar::random(&mut rng).as_bytes());
    let commit_memo = Commitment::blinded(10u64);
    //order size
    let order_size = Commitment::blinded(4u64);
    let data: Vec<String> = vec![String::from(order_size)];
    let memo_out = OutputMemo {
        script_address: script_address.as_hex(),
        owner: add.as_hex(),
        commitment: commit_memo,
        data: Some(data),
        timebounds: 0,
    };
    let out_data = OutputData::Memo(memo_out);
    let memo = Output::memo(out_data);
    let output: Vec<Output> = vec![memo];

    //cretae unsigned Tx with program proof
    let result = Prover::build_proof(correct_program, &input, &output, false, None);
    println!("{:?}", result);
    let (prog_bytes, proof) = result.unwrap();
    let verify = Verifier::verify_r1cs_proof(&proof, &prog_bytes, &input, &output, false, None);
    println!("{:?}", verify);
}

#[test]
fn lend_order_tx_program_stack_initialized_test() {
    let _program = order_message_prog_input_output(16u64, 9u64, 0, 0);
    // let correct_program = self::order_message_prog_with_stack_initialized();
    let correct_program = self::lend_order_initial_dup_test_stack_initialized();
    println!("\n Program \n{:?}", correct_program);

    //create InputCoin and OutputMemo

    let mut rng = rand::thread_rng();
    let sk_in: RistrettoSecretKey = SecretKey::random(&mut rng);
    let pk_in = RistrettoPublicKey::from_secret_key(&sk_in, &mut rng);
    let commit_in = ElGamalCommitment::generate_commitment(
        &pk_in,
        Scalar::random(&mut rng),
        Scalar::from(10u64),
    );
    let add: Address = Address::standard_address(Network::default(), pk_in.clone());
    let out_coin = OutputCoin {
        encrypt: commit_in,
        owner: add.as_hex(),
    };
    let in_data: InputData = InputData::coin(Utxo::default(), out_coin, 0);
    let coin_in: Input = Input::coin(in_data);

    //outputMemo
    let script_address =
        Address::script_address(Network::Mainnet, *Scalar::random(&mut rng).as_bytes());
    let commit_memo = Commitment::blinded(10u64);
    //order size
    let deposit = Commitment::blinded(4u64);
    let pool_share = Commitment::blinded(4u64);
    let data: Vec<String> = vec![String::from(deposit), String::from(pool_share)];
    let memo_out = OutputMemo {
        script_address: script_address.as_hex(),
        owner: add.as_hex(),
        commitment: commit_memo,
        data: Some(data),
        timebounds: 0,
    };
    let out_data = OutputData::Memo(memo_out);
    let memo = Output::memo(out_data);

    //create output state
    let tvl_1: Commitment = Commitment::blinded(14u64);
    let tps_1: Commitment = Commitment::blinded(14u64);
    let s_var: String = String::from(tps_1.clone());
    let s_var_vec: Vec<String> = vec![s_var];
    // create Output state
    let out_state: OutputState = OutputState {
        nonce: 2,
        script_address: script_address.as_hex(),
        owner: add.as_hex(),
        commitment: tvl_1,
        state_variables: Some(s_var_vec),
        timebounds: 0,
    };

    let output: Vec<Output> = vec![memo, Output::state(OutputData::State(out_state))];
    // create Input State
    let tvl_0: Commitment = Commitment::blinded(10u64);
    let tps_0: Commitment = Commitment::blinded(10u64);
    let s_var: String = String::from(tps_0.clone());
    let in_state_var_vec: Vec<String> = vec![s_var];
    let temp_out_state = OutputState {
        nonce: 1,
        script_address: script_address.as_hex(),
        owner: add.as_hex(),
        commitment: tvl_0.clone(),
        state_variables: Some(in_state_var_vec),
        timebounds: 0,
    };
    let error = Commitment::blinded(0u64);
    let err_string = String::from(error);
    // convert to input
    let input_state: Input = Input::state(InputData::state(
        Utxo::default(),
        temp_out_state.clone(),
        Some(err_string),
        1,
    ));
    let input: Vec<Input> = vec![coin_in, input_state];

    //cretae unsigned Tx with program proof
    let result = Prover::build_proof(correct_program, &input, &output, false, None);
    println!("{:?}", result);
    let (prog_bytes, proof) = result.unwrap();
    let verify = Verifier::verify_r1cs_proof(&proof, &prog_bytes, &input, &output, false, None);
    println!("{:?}", verify);
}

#[test]
fn trade_order_settle_tx_program_stack_initialized_test() {
    let correct_program = self::settle_order_test_stack_initialized();
    println!("\n Program \n{:?}", correct_program);
    let mut rng = rand::thread_rng();
    let sk_in: RistrettoSecretKey = SecretKey::random(&mut rng);
    let pk_in = RistrettoPublicKey::from_secret_key(&sk_in, &mut rng);
    let add: Address = Address::standard_address(Network::default(), pk_in.clone());
    //create InputMemo and OutputCoin
    //Input memo
    let script_address =
        Address::script_address(Network::Mainnet, *Scalar::random(&mut rng).as_bytes());
    let commit_memo = Commitment::blinded(10u64);
    //order size
    let initial_margin = Commitment::blinded(8u64);
    let data: Vec<String> = vec![String::from(initial_margin)];
    let memo_out = OutputMemo {
        script_address: script_address.as_hex(),
        owner: add.as_hex(),
        commitment: commit_memo,
        data: Some(data),
        timebounds: 0,
    };
    let coin_value: Commitment = Commitment::blinded(6u64); // CM to be pushed back to the user
    let input_memo = Input::memo(InputData::memo(
        Utxo::default(),
        memo_out,
        0,
        Some(coin_value),
    ));

    let enc_out = ElGamalCommitment::generate_commitment(
        &pk_in,
        Scalar::random(&mut rng),
        Scalar::from(6u64), // CM . lost 2 sats
    );

    let out_coin = OutputCoin {
        encrypt: enc_out,
        owner: add.as_hex(),
    };
    let coin_out = Output::coin(OutputData::Coin(out_coin));

    //create output state
    let tvl_1: Commitment = Commitment::blinded(14u64);
    let tps_1: Commitment = Commitment::blinded(14u64);
    let s_var: String = String::from(tps_1.clone());
    let s_var_vec: Vec<String> = vec![s_var];
    // create Output state
    let out_state: OutputState = OutputState {
        nonce: 2,
        script_address: script_address.as_hex(),
        owner: add.as_hex(),
        commitment: tvl_1,
        state_variables: Some(s_var_vec),
        timebounds: 0,
    };

    let output: Vec<Output> = vec![coin_out, Output::state(OutputData::State(out_state))];
    // create Input State
    let tvl_0: Commitment = Commitment::blinded(10u64);
    let tps_0: Commitment = Commitment::blinded(10u64);
    let s_var: String = String::from(tps_0.clone());
    let in_state_var_vec: Vec<String> = vec![s_var];
    let temp_out_state = OutputState {
        nonce: 1,
        script_address: script_address.as_hex(),
        owner: add.as_hex(),
        commitment: tvl_0.clone(),
        state_variables: Some(in_state_var_vec),
        timebounds: 0,
    };
    let payment = Commitment::blinded(2u64);
    let pay_string = String::from(payment);
    // convert to input
    let input_state: Input = Input::state(InputData::state(
        Utxo::default(),
        temp_out_state.clone(),
        Some(pay_string),
        1,
    ));
    let input: Vec<Input> = vec![input_memo, input_state];
    //tx_date i.e., price
    let tx_data: zkvm::String = zkvm::String::U64(2u64);
    //cretae unsigned Tx with program proof
    let result = Prover::build_proof(
        correct_program,
        &input,
        &output,
        false,
        Some(tx_data.clone()),
    );
    println!("{:?}", result);
    let (prog_bytes, proof) = result.unwrap();
    let verify =
        Verifier::verify_r1cs_proof(&proof, &prog_bytes, &input, &output, false, Some(tx_data));
    println!("{:?}", verify);
}

#[test]
fn test_dark_transaction_single_sender_reciever() {
    let mut rng = rand::thread_rng();

    // create sender and reciever
    // lets say bob wants to sent 500 tokens to alice from his account
    let (bob_account_1, bob_sk_account_1) =
        Account::generate_random_account_with_value(1000u64.into());
    //create alice account with 0 balance
    let alice_pk = RistrettoPublicKey::generate_base_pk();
    let alice_comm_scalar = Scalar::random(&mut rng);
    let alice_commitment =
        ElGamalCommitment::generate_commitment(&alice_pk, alice_comm_scalar, Scalar::from(0u64));

    let alice_account = Account::set_account(alice_pk, alice_commitment);

    // create sender array
    let alice_reciever = crate::Receiver::set_receiver(500, alice_account);
    let bob_sender = crate::Sender::set_sender(-500, bob_account_1, vec![alice_reciever]);
    let tx_vector: Vec<crate::Sender> = vec![bob_sender];

    let (value_vector, account_vector, sender_count, receiver_count) =
        crate::Sender::generate_value_and_account_vector(tx_vector).unwrap();
    println!(
        "value_vector: {:?} \n sender_count {:?} \n receiver_count {:?}",
        value_vector, sender_count, receiver_count
    );
    // no need for anonymity as it is dark transaction
    //Create sender updated account vector for the verification of sk and bl-v
    let bl_first_sender = 1000 - 500; //bl-v
                                      //let bl_second_sender = 20 - 3; //bl-v
    let updated_balance_sender: Vec<u64> = vec![bl_first_sender]; //, bl_second_sender];
                                                                  //Create vector of sender secret keys
    let sk_sender: Vec<RistrettoSecretKey> = vec![bob_sk_account_1]; //, bob_sk_account_2];

    // create input from account vector
    let bob_utxo = Utxo::random(); //Simulating a valid UTXO input
    let bob_input =
        Input::input_from_quisquis_account(&bob_account_1, bob_utxo, 0, Network::default());

    //Simulating a non UTXO input. Provide a valid witness index and Zero balance proof
    let alice_input =
        Input::input_from_quisquis_account(&alice_account, Utxo::default(), 0, Network::default());
    let inputs: Vec<Input> = vec![bob_input, alice_input];

    // let utxo = Utxo::default();
    // let inputs: Vec<Input> = account_vector
    //     .iter()
    //     .map(|acc| Input::input_from_quisquis_account(acc, utxo, 0, Network::default()))
    //     .collect();

    let reciever_value_balance: Vec<u64> = vec![500];
    //println!("Data : {:?}", sender_count);
    //create quisquis transfertransactio
    let dark_transfer = crate::TransferTransaction::create_dark_transaction(
        &value_vector,
        &account_vector,
        &updated_balance_sender,
        &reciever_value_balance,
        &inputs,
        &sk_sender,
        sender_count,
        receiver_count,
        Some(&vec![alice_comm_scalar]),
    );
    let (transfer, comm_scalar) = dark_transfer.unwrap();
    let tx = crate::Transaction::transaction_transfer(crate::TransactionData::TransactionTransfer(
        transfer.clone(),
    ));
    println!("Transaction : {:?}", tx.clone());

    // Verify the transaction
    let verify = tx.verify();
    println!("Verify : {:?}", verify);
    assert!(verify.is_ok());
}

#[test]
fn test_dark_transaction_pow_2() {
    let mut rng = rand::thread_rng();

    // create mutiple sender and recievers
    // lets say bob wants to sent 500 tokens to alice from his account
    // and fay 300 from other account
    let (bob_account_1, bob_sk_account_1) =
        Account::generate_random_account_with_value(1000u64.into());
    // create bob account 2
    let (bob_account_2, bob_sk_account_2) =
        Account::generate_random_account_with_value(500u64.into());

    //create alice and fay account with 0 balance
    let base_pk = RistrettoPublicKey::generate_base_pk();
    let alice_key = PublicKey::update_public_key(&base_pk, Scalar::random(&mut rng));

    let (alice_account, alice_comm_rscalar) = Account::generate_account(alice_key.clone());
    let (fay_account, fay_comm_rscalar) = Account::generate_account(PublicKey::update_public_key(
        &alice_key,
        Scalar::random(&mut rng),
    ));

    // create sender array
    let alice_reciever = crate::Receiver::set_receiver(500, alice_account);
    let fay_reciever = crate::Receiver::set_receiver(300, fay_account);

    let bob_sender = crate::Sender::set_sender(-500, bob_account_1, vec![alice_reciever]);
    let bob_sender_2 = crate::Sender::set_sender(-300, bob_account_2, vec![fay_reciever]);
    let tx_vector: Vec<crate::Sender> = vec![bob_sender, bob_sender_2];

    let (value_vector, account_vector, sender_count, receiver_count) =
        crate::Sender::generate_value_and_account_vector(tx_vector).unwrap();
    println!(
        "value_vector: {:?} \n sender_count {:?} \n receiver_count {:?}",
        value_vector, sender_count, receiver_count
    );
    // no need for anonymity as it is dark transaction
    //Create sender updated account vector for the verification of sk and bl-v
    let bl_first_sender = 1000 - 500; //bl-v
    let bl_second_sender = 500 - 300; //bl-v
    let updated_balance_sender: Vec<u64> = vec![bl_first_sender, bl_second_sender];
    //Create vector of sender secret keys
    let sk_sender: Vec<RistrettoSecretKey> = vec![bob_sk_account_1, bob_sk_account_2];

    // create input from account vector
    let bob_utxo = Utxo::random(); //Simulating a valid UTXO input
    let bob_input_1 =
        Input::input_from_quisquis_account(&bob_account_1, bob_utxo, 0, Network::default());
    let bob_utxo = Utxo::random(); //Simulating a valid UTXO input
    let bob_input_2 =
        Input::input_from_quisquis_account(&bob_account_2, bob_utxo, 0, Network::default());

    //Simulating a non UTXO input. Provide a valid witness index and Zero balance proof
    let alice_input =
        Input::input_from_quisquis_account(&alice_account, Utxo::default(), 0, Network::default());
    let fay_input =
        Input::input_from_quisquis_account(&fay_account, Utxo::default(), 1, Network::default());

    let inputs: Vec<Input> = vec![bob_input_1, bob_input_2, alice_input, fay_input];

    let reciever_value_balance: Vec<u64> = vec![500, 300];
    //println!("Data : {:?}", sender_count);
    //create quisquis transfertransactio
    let dark_transfer = crate::TransferTransaction::create_dark_transaction(
        &value_vector,
        &account_vector,
        &updated_balance_sender,
        &reciever_value_balance,
        &inputs,
        &sk_sender,
        sender_count,
        receiver_count,
        Some(&vec![alice_comm_rscalar, fay_comm_rscalar]),
    );
    let (tranfer, _comm_scalar) = dark_transfer.unwrap();
    let tx = crate::Transaction::transaction_transfer(crate::TransactionData::TransactionTransfer(
        tranfer,
    ));
    //  println!("Transaction : {:?}", tx);

    // Verify the transaction
    let verify = tx.verify();
    println!("Verify : {:?}", verify);
    assert!(verify.is_ok());
}

#[test]
fn test_dark_transaction_odd() {
    let mut rng = rand::thread_rng();

    // create mutiple sender and recievers
    // lets say bob wants to sent 300 tokens to alice and 200 to jay from his account
    // and fay 300 from other account
    let (bob_account_1, bob_sk_account_1) =
        Account::generate_random_account_with_value(1000u64.into());
    // create bob account 2
    let (bob_account_2, bob_sk_account_2) =
        Account::generate_random_account_with_value(500u64.into());

    //create alice and fay account with 0 balance
    let base_pk = RistrettoPublicKey::generate_base_pk();
    let alice_key = PublicKey::update_public_key(&base_pk, Scalar::random(&mut rng));

    let (alice_account, alice_comm_rscalar) = Account::generate_account(alice_key.clone());
    let (fay_account, fay_comm_rscalar) = Account::generate_account(PublicKey::update_public_key(
        &alice_key,
        Scalar::random(&mut rng),
    ));
    let (jay_account, jay_comm_rscalar) = Account::generate_account(PublicKey::update_public_key(
        &alice_key,
        Scalar::random(&mut rng),
    ));

    // create sender array
    let alice_reciever = crate::Receiver::set_receiver(300, alice_account);
    let fay_reciever = crate::Receiver::set_receiver(300, fay_account);
    let jay_reciever = crate::Receiver::set_receiver(200, jay_account);

    let bob_sender =
        crate::Sender::set_sender(-500, bob_account_1, vec![alice_reciever, jay_reciever]);
    let bob_sender_2 = crate::Sender::set_sender(-300, bob_account_2, vec![fay_reciever]);
    let tx_vector: Vec<crate::Sender> = vec![bob_sender, bob_sender_2];

    let (value_vector, account_vector, sender_count, receiver_count) =
        crate::Sender::generate_value_and_account_vector(tx_vector).unwrap();
    println!(
        "value_vector: {:?} \n sender_count {:?} \n receiver_count {:?}",
        value_vector, sender_count, receiver_count
    );
    // no need for anonymity as it is dark transaction
    //Create sender updated account vector for the verification of sk and bl-v
    let bl_first_sender = 1000 - 500; //bl-v
    let bl_second_sender = 500 - 300; //bl-v
    let updated_balance_sender: Vec<u64> = vec![bl_first_sender, bl_second_sender];
    //Create vector of sender secret keys
    let sk_sender: Vec<RistrettoSecretKey> = vec![bob_sk_account_1, bob_sk_account_2];

    // create input from account vector
    let bob_utxo = Utxo::random(); //Simulating a valid UTXO input
    let bob_input_1 =
        Input::input_from_quisquis_account(&bob_account_1, bob_utxo, 0, Network::default());
    let bob_utxo = Utxo::random(); //Simulating a valid UTXO input
    let bob_input_2 =
        Input::input_from_quisquis_account(&bob_account_2, bob_utxo, 0, Network::default());

    //Simulating a non UTXO input. Provide a valid witness index and Zero balance proof
    let alice_input =
        Input::input_from_quisquis_account(&alice_account, Utxo::default(), 0, Network::default());
    let fay_input =
        Input::input_from_quisquis_account(&fay_account, Utxo::default(), 2, Network::default());
    let jay_input =
        Input::input_from_quisquis_account(&jay_account, Utxo::default(), 1, Network::default());

    let inputs: Vec<Input> = vec![bob_input_1, bob_input_2, alice_input, jay_input, fay_input];

    let reciever_value_balance: Vec<u64> = vec![300, 200, 300];
    //println!("Data : {:?}", sender_count);
    //create quisquis transfertransactio
    let dark_transfer = crate::TransferTransaction::create_dark_transaction(
        &value_vector,
        &account_vector,
        &updated_balance_sender,
        &reciever_value_balance,
        &inputs,
        &sk_sender,
        sender_count,
        receiver_count,
        Some(&vec![
            alice_comm_rscalar,
            jay_comm_rscalar,
            fay_comm_rscalar,
        ]),
    );
    let (transfer, _comm_scalar) = dark_transfer.unwrap();
    let tx = crate::Transaction::transaction_transfer(crate::TransactionData::TransactionTransfer(
        transfer,
    ));
    // hex encode the tx
    let tx_hex = hex::encode(bincode::serialize(&tx).unwrap());
    println!("Transaction : {:?}", tx_hex);

    // Verify the transaction
    let verify = tx.verify();
    println!("Verify : {:?}", verify);
    assert!(verify.is_ok());
}

#[test]
fn test_quisquis_transaction_single_sender_reciever() {
    let mut rng = rand::thread_rng();

    // create sender and reciever

    // lets say bob wants to sent 500 tokens to alice from his account
    let (bob_account_1, bob_sk_account_1) =
        Account::generate_random_account_with_value(1000u64.into());

    //create alice account with 0 balance
    let alice_pk = RistrettoPublicKey::generate_base_pk();
    let alice_comm_scalar = Scalar::random(&mut rng);
    let alice_commitment =
        ElGamalCommitment::generate_commitment(&alice_pk, alice_comm_scalar, Scalar::from(0u64));

    let alice_account = Account::set_account(alice_pk, alice_commitment);

    // create sender array
    //let alice_reciever = crate::Receiver::set_receiver(500, alice_account);
    //let bob_sender = crate::Sender::set_sender(-500, bob_account_1, vec![alice_reciever]);
    //let tx_vector: Vec<crate::Sender> = vec![bob_sender];

    //let (mut value_vector, mut account_vector, sender_count, receiver_count) =
    //  crate::Sender::generate_value_and_account_vector(tx_vector).unwrap();
    // arrange value and account vector directly for testing
    let value_vector: Vec<i64> = vec![-500, 500, 0, 0, 0, 0, 0, 0, 0];
    let mut account_vector: Vec<Account> = vec![bob_account_1, alice_account];

    // get anonymity accounts. Creating them on the fly for testing purposes. Should be retrieved from utxo
    let (anonymity_account_vector, anonymity_scalar_vector) =
        crate::Sender::create_anonymity_set(1, 1);

    // add anonymity accounts to account vectors
    account_vector.extend(anonymity_account_vector);
    let senders_count = 1;
    let receivers_count = 1;
    println!(
        "value_vector: {:?} \n sender_count {:?} \n receiver_count {:?}",
        value_vector, senders_count, receivers_count
    );

    //Create sender updated account vector for the verification of sk and bl-v
    let updated_balance_sender: Vec<u64> = vec![1000 - 500];
    //Create vector of sender secret keys
    let sk_sender: Vec<RistrettoSecretKey> = vec![bob_sk_account_1];

    // create input from account vector
    // let bob_utxo = Utxo::random(); //Simulating a valid UTXO input
    // let bob_input =
    //   Input::input_from_quisquis_account(&bob_account_1, bob_utxo, 0, Network::default());

    //Simulating a non UTXO input. Provide a valid witness index and Zero balance proof
    //let alice_input =
    // Input::input_from_quisquis_account(&alice_account, Utxo::default(), 0, Network::default());
    //let inputs: Vec<Input> = vec![bob_input, alice_input];
    //Simulating rendom Utxo based Inputs from accounts
    let utxo = Utxo::random();
    let inputs: Vec<Input> = account_vector
        .iter()
        .map(|acc| Input::input_from_quisquis_account(acc, utxo, 0, Network::default()))
        .collect();

    let reciever_value_balance: Vec<u64> = vec![500];
    let diff: usize = 9 - (senders_count + receivers_count);
    // create quisquis transfer transaction
    let transfer = crate::TransferTransaction::create_quisquis_transaction(
        &inputs,
        &value_vector,
        &account_vector,
        &updated_balance_sender,
        &reciever_value_balance,
        &sk_sender,
        senders_count,
        receivers_count,
        // &anonymity_scalar_vector,
        diff,
        None,
    );

    let tx = crate::Transaction::transaction_transfer(crate::TransactionData::TransactionTransfer(
        transfer.unwrap(),
    ));
    // println!("Transaction : {:?}", tx);

    // Verify the transaction
    let verify = tx.verify();
    println!("Verify : {:?}", verify);
    assert!(verify.is_ok());
}

#[test]
fn test_create_burn_message() {
    // For Complete test
    // create Dark transfer to Burn Address first
    let mut rng = rand::thread_rng();

    // create sender and reciever
    // lets say bob wants to Burn 500 tokens
    let (bob_account_1, bob_sk_account_1) =
        Account::generate_random_account_with_value(500u64.into());
    let (bob_pk, _) = bob_account_1.get_account();

    //create Burn Address/Account witn zero balance
    let burn_pk = RistrettoPublicKey::update_public_key(&bob_pk, Scalar::random(&mut rng));

    let burn_comm_scalar = Scalar::random(&mut rng);
    let burn_commitment =
        ElGamalCommitment::generate_commitment(&burn_pk, burn_comm_scalar, Scalar::from(0u64));

    let burn_account = Account::set_account(burn_pk, burn_commitment);

    // create sender array
    let burn_reciever = crate::Receiver::set_receiver(500, burn_account);
    let bob_sender = crate::Sender::set_sender(-500, bob_account_1, vec![burn_reciever]);
    let tx_vector: Vec<crate::Sender> = vec![bob_sender];

    let (value_vector, account_vector, sender_count, receiver_count) =
        crate::Sender::generate_value_and_account_vector(tx_vector).unwrap();
    println!(
        "value_vector: {:?} \n sender_count {:?} \n receiver_count {:?}",
        value_vector, sender_count, receiver_count
    );

    //Create sender updated account vector for the verification of sk and bl-v
    let updated_balance_sender: Vec<u64> = vec![500];
    let sk_sender: Vec<RistrettoSecretKey> = vec![bob_sk_account_1];

    // create input from account vector
    let bob_utxo = Utxo::random(); //Simulating a valid UTXO input
    let bob_input =
        Input::input_from_quisquis_account(&bob_account_1, bob_utxo, 0, Network::default());

    //Simulating a non UTXO input. Provide a valid witness index and Zero balance proof
    let burn_input =
        Input::input_from_quisquis_account(&burn_account, Utxo::default(), 0, Network::default());
    let inputs: Vec<Input> = vec![bob_input, burn_input.clone()];

    let reciever_value_balance: Vec<u64> = vec![500];
    //println!("Data : {:?}", sender_count);
    //create Dark transfer transaction
    let dark_transfer = crate::TransferTransaction::create_dark_transaction(
        &value_vector,
        &account_vector,
        &updated_balance_sender,
        &reciever_value_balance,
        &inputs,
        &sk_sender,
        sender_count,
        receiver_count,
        Some(&vec![burn_comm_scalar]),
    );
    let (transfer, comm_scalar_final) = dark_transfer.unwrap();
    let tx = crate::Transaction::transaction_transfer(crate::TransactionData::TransactionTransfer(
        transfer.clone(),
    ));
    // Use data from the transfer transaction to create burn message
    // create input for burn message
    let outputs = tx.get_tx_outputs();
    // get reciever out
    let reciever_out = outputs[1].clone();
    let input_burn_message = reciever_out
        .as_out_coin()
        .unwrap()
        .to_input(Utxo::default(), 0);
    // get input reciever address
    let burn_inital_address = burn_input.as_owner_address().unwrap().to_owned();

    // create burn message
    let burn_message = crate::Message::create_burn_message(
        input_burn_message,
        500u64,
        comm_scalar_final.unwrap().clone(),
        bob_sk_account_1,
        burn_inital_address,
    );
    let burn_tx = crate::Transaction::from(burn_message);
    println!("Burn Transaction : {:?}", burn_tx);
    //verify burn transaction
    let verify = burn_tx.verify();
    println!("Verify : {:?}", verify);
    assert!(verify.is_ok());
    // testing Encrypt scalar addition
    // let outputs = tx.get_tx_outputs();
    // // get reciever out
    // let reciever_out = outputs[1].clone();
    // let recieever_account = reciever_out.to_quisquis_account().unwrap();
    // let (_pk, enc) = recieever_account.get_account();

    // // get pk od the receiver account in the Input vector
    // let input_test = tx.get_tx_inputs();
    // let input_account_reciever = input_test[1].to_quisquis_account().unwrap();
    // let (pk_input_reciever, _) = input_account_reciever.get_account();
    // // recreate el gamal encryption using input pk and updated with new scalar and 500

    // let new_enc = ElGamalCommitment::generate_commitment(
    //     &pk_input_reciever,
    //     comm_scalar_final.unwrap(),
    //     Scalar::from(500u64),
    // );
    // assert_eq!(new_enc, enc);
}
