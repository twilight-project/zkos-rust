use address::{Address, Network};
use criterion::{criterion_group, criterion_main, Criterion};
use curve25519_dalek::scalar::Scalar;
use quisquislib::accounts::Account;
use quisquislib::elgamal::ElGamalCommitment;
use quisquislib::{
    // accounts::prover::Prover as QuisquisProver,
    keys::PublicKey,
    ristretto::{RistrettoPublicKey, RistrettoSecretKey},
};
use transaction::{Sender, Transaction, TransferTransaction};
use zkvm::zkos_types::{Input, Output, Utxo};

fn bench_account_decrypt(c: &mut Criterion) {
    c.bench_function("Decrypt_Account", |b| {
        //let mut rng = rand::thread_rng();

        // create sender and reciever
        // lets say bob wants to Burn 500 tokens
        let (bob_account_1, bob_sk_account_1) =
            Account::generate_random_account_with_value(500u64.into());
        b.iter(|| {
            bob_account_1
                .decrypt_account_balance(&bob_sk_account_1, 500u64.into())
                .unwrap();
        });
    });
}
fn bench_account_decrypt_value(c: &mut Criterion) {
    c.bench_function("Decrypt Account Value", |b| {
        //let mut rng = rand::thread_rng();

        // create sender and reciever
        // lets say bob wants to Burn 500 tokens
        let (bob_account_1, bob_sk_account_1) =
            Account::generate_random_account_with_value(50000u64.into());
        b.iter(|| {
            bob_account_1
                .decrypt_account_balance_value(&bob_sk_account_1)
                .unwrap();
        });
    });
}

fn bench_verify_qq_tx(c: &mut Criterion) {
    c.bench_function("Verify QQ Tx", |b| {
        // create qq tx
        let (
            inputs,
            value_vector,
            account_vector,
            updated_balance_sender,
            reciever_value_balance,
            sk_sender,
            senders_count,
            receivers_count,
            diff,
        ) = create_quisquis_tx_single();
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
        )
        .unwrap();
        let tx = Transaction::transaction_transfer(
            transaction::TransactionData::TransactionTransfer(transfer.clone()),
        );
        b.iter(|| {
            assert!(tx.verify().is_ok());
        });
    });
}

fn bench_create_burn_tx(c: &mut Criterion) {
    c.bench_function("Burn Tx", |b| {
        // get data for burn t
        let (input, address, scalar, sk) = get_data_for_burn_tx();

        b.iter(|| {
            let burn_message = transaction::Message::create_burn_message(
                input.clone(),
                500u64,
                scalar.clone(),
                sk.clone(),
                address.clone(),
            );
            let burn_tx = crate::Transaction::from(burn_message);
        });
    });
}

fn bench_create_dark_tx(c: &mut Criterion) {
    c.bench_function("Dark Tx", |b| {
        // get data for burn t
        let (
            values,
            accounts,
            inputs,
            updated_sender_balance,
            updated_balance_reciever,
            sk_sender,
            sender_count,
            receiver_count,
        ) = create_dark_tx_data();

        b.iter(|| {
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
            )
            .unwrap();
        });
    });
}

fn bench_create_qq_tx(c: &mut Criterion) {
    c.bench_function("QQ Tx", |b| {
        // get data for burn t
        let (
            inputs,
            value_vector,
            account_vector,
            updated_balance_sender,
            reciever_value_balance,
            sk_sender,
            senders_count,
            receivers_count,
            diff,
        ) = create_quisquis_tx_single();

        b.iter(|| {
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
            )
            .unwrap();
        });
    });
}

criterion_group!(
    benches,
    bench_account_decrypt,
    bench_account_decrypt_value,
    bench_verify_qq_tx,
    bench_create_burn_tx,
    bench_create_dark_tx,
    bench_create_qq_tx,
    //bench_verify_dark_transaction_single,
    //bench_dark_transaction_single,
    //bench_qq_transaction_single,
    //bench_verify_qq_transaction_single,
    //bench_decrypt_trading_account_value_20000,
    // bench_decrypt_trading_account_value_10000,
    // bench_decrypt_trading_account_value_1000,
    // bench_decrypt_trading_account_value_100,
    // bench_verify_account,
);
criterion_main!(benches);
pub fn create_dark_tx_data() -> (
    Vec<i64>,
    Vec<Account>,
    Vec<Input>,
    Vec<u64>,
    Vec<u64>,
    Vec<RistrettoSecretKey>,
    usize,
    usize,
) {
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
    let accounts: &[Account] = &account_vector[..(sender_count + receiver_count)];
    let values: &[i64] = &value_vector[..(sender_count + receiver_count)];

    let hash = zkvm::Hash::default();
    //SHOULD BE TAKEN FROM UTXO SET
    let utxo = Utxo::from_hash(hash, 0);
    //create vec of Inouts
    let mut inputs: Vec<Input> = Vec::new();
    for input in accounts.iter() {
        //create address
        let (pk, enc) = input.get_account();
        let out_coin = zkvm::zkos_types::OutputCoin {
            encrypt: enc,
            owner: address::Address::standard_address(address::Network::Mainnet, pk).as_hex(),
        };
        let inp = Input::coin(zkvm::InputData::coin(
            utxo,
            // address::Address::coin_address(address::Network::Mainnet, pk).as_hex(),
            //  enc,
            out_coin, 0,
        ));
        inputs.push(inp.clone());
    }

    let updated_balance_reciever: Vec<u64> = vec![5];
    (
        value_vector,
        account_vector,
        inputs,
        updated_sender_balance,
        updated_balance_reciever,
        sk_sender,
        sender_count,
        receiver_count,
    )
}
pub fn create_quisquis_tx_single() -> (
    Vec<Input>,
    Vec<i64>,
    Vec<Account>,
    Vec<u64>,
    Vec<u64>,
    Vec<RistrettoSecretKey>,
    usize,
    usize,
    usize,
) {
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
    // println!(
    //     "value_vector: {:?} \n sender_count {:?} \n receiver_count {:?}",
    //     value_vector, senders_count, receivers_count
    // );

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
    // let transfer = crate::TransferTransaction::create_quisquis_transaction(
    //     &inputs,
    //     &value_vector,
    //     &account_vector,
    //     &updated_balance_sender,
    //     &reciever_value_balance,
    //     &sk_sender,
    //     senders_count,
    //     receivers_count,
    //     // &anonymity_scalar_vector,
    //     diff,
    //     None,
    // );
    // let tx = Transaction::transaction_transfer(transaction::TransactionData::TransactionTransfer(
    //     transfer.unwrap(),
    // ));
    // tx
    (
        inputs,
        value_vector.to_vec(),
        account_vector.to_vec(),
        updated_balance_sender.to_vec(),
        reciever_value_balance.to_vec(),
        sk_sender.to_vec(),
        senders_count,
        receivers_count,
        diff,
    )
}

fn get_data_for_burn_tx() -> (Input, String, Scalar, RistrettoSecretKey) {
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
    let burn_reciever = transaction::Receiver::set_receiver(500, burn_account);
    let bob_sender = Sender::set_sender(-500, bob_account_1, vec![burn_reciever]);
    let tx_vector: Vec<crate::Sender> = vec![bob_sender];

    let (value_vector, account_vector, sender_count, receiver_count) =
        crate::Sender::generate_value_and_account_vector(tx_vector).unwrap();
    // println!(
    //     "value_vector: {:?} \n sender_count {:?} \n receiver_count {:?}",
    //     value_vector, sender_count, receiver_count
    // );

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
    let dark_transfer = transaction::TransferTransaction::create_dark_transaction(
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
    let tx = Transaction::transaction_transfer(transaction::TransactionData::TransactionTransfer(
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
    (
        input_burn_message.clone(),
        burn_inital_address.clone(),
        comm_scalar_final.unwrap().clone(),
        bob_sk_account_1.clone(),
    )
}
