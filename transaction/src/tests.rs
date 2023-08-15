use crate::vm_run::{Prover, Verifier};

use address::{Address, Network};
use curve25519_dalek::scalar::Scalar;
use quisquislib::elgamal::ElGamalCommitment;
use quisquislib::{
    keys::{PublicKey, SecretKey},
    ristretto::{RistrettoPublicKey, RistrettoSecretKey},
};

use zkvm::merkle::{CallProof, Hasher, MerkleTree, Path};
use zkvm::zkos_types::{Input, InputData, Output, OutputCoin, OutputData, OutputMemo, Utxo};
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
    });
    let prog5 = Program::build(|p| {
        p.push(Commitment::blinded(5u64));
        p.dup(1);
    });

    let progs = vec![
        prog1.clone(),
        prog2.clone(),
        prog3.clone(),
        prog4.clone(),
        prog5.clone(),
    ];
    //create tree root
    let root = MerkleTree::root(b"ZkOS.MerkelTree", progs.iter());
    //convert root to address
    let address = Address::script_address(Network::Mainnet, root.0);
    //script address as hex
    let address_hex = address.as_hex();

    // create path for program3
    let _path = Path::new(&progs, 2 as usize, &hasher).unwrap();

    // create call proof for program3
    let call_proof =
        CallProof::create_call_proof(&progs, 2 as usize, &hasher, address_hex).unwrap();

    // verify call proof
    let prog = prog3.clone();
    let verify = call_proof.verify_call_proof(&address.as_script_address(), &prog, &hasher);
    println!("verify: {:?}", verify);
}

#[test]
fn order_message_test() {
    let _program = order_message_prog_input_output(16u64, 9u64, 0, 0);
    let correct_program = self::order_message_prog(16u64, 9u64);

    print!("\n Program \n{:?}", correct_program);

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
    let add: Address = Address::coin_address(Network::default(), pk_in.clone());
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
    let add_out: Address = Address::coin_address(Network::default(), pk_out);
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
    let result = Prover::build_proof(correct_program, &input, &output);
    println!("{:?}", result);
    let (prog_bytes, proof) = result.unwrap();
    let verify = Verifier::verify_r1cs_proof(proof, prog_bytes, &input, &output);
    println!("{:?}", verify);
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
    let correct_program = self::order_message_prog_with_stack_initialized();

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
    let add: Address = Address::coin_address(Network::default(), pk_in.clone());
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
    let commit_memo = Commitment::blinded(5u64);
    //order size
    let order_size = Commitment::blinded(4u64);
    let data: String = String::from(order_size);
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
    let result = Prover::build_proof(correct_program, &input, &output);
    println!("{:?}", result);
    let (prog_bytes, proof) = result.unwrap();
    let verify = Verifier::verify_r1cs_proof(proof, prog_bytes, &input, &output);
    println!("{:?}", verify);
}
