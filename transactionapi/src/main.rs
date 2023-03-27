// mod quisquislib;
mod rpcclient;
mod rpcserver;
use rpcclient::txrequest;
// use crate::trasaction;
// mod transaction::tx;
use rpcserver::*;
use transaction::TransferTransaction;
#[macro_use]
extern crate lazy_static;
fn main() {
    let handle = std::thread::Builder::new()
        .name(String::from("rpc request"))
        .spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(5000));
            // txrequest::create_request();
        })
        .unwrap();
    handle.join().unwrap();
    rpcserver();
}
use bulletproofs::{PedersenGens, RangeProof};
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;
use quisquislib::{
    accounts::prover::{Prover, SigmaProof},
    accounts::verifier::Verifier,
    accounts::Account,
    keys::PublicKey,
    pedersen::vectorpedersen::VectorPedersenGens,
    ristretto::{RistrettoPublicKey, RistrettoSecretKey},
    shuffle::{shuffle::ROWS, Shuffle, ShuffleProof, ShuffleStatement},
};
use transaction::Input;
// pub fn create_tx() {
//     let base_pk = RistrettoPublicKey::generate_base_pk();

//     let value_vector: Vec<Scalar> = vec![
//         -Scalar::from(5u64),
//         -Scalar::from(3u64),
//         Scalar::from(5u64),
//         Scalar::from(3u64),
//     ];

//     let mut updated_accounts: Vec<Account> = Vec::new();
//     let mut sender_sk: Vec<RistrettoSecretKey> = Vec::new();

//     for i in 0..4 {
//         let (updated_account, sk) =
//             Account::generate_random_account_with_value(Scalar::from(10u64));

//         updated_accounts.push(updated_account);

//         // lets save the first and second sk as sender's sk as we discard the rest
//         if i == 0 || i == 1 {
//             sender_sk.push(sk);
//         }
//     }

//     let (delta_accounts, epsilon_accounts, delta_rscalar_vector) =
//         Account::create_delta_and_epsilon_accounts(&updated_accounts, &value_vector, base_pk);

//     let updated_delta_accounts =
//         Account::update_delta_accounts(&updated_accounts, &delta_accounts).unwrap();
//     let account_vector = updated_delta_accounts;
//     // balance that we want to prove should be sender balance - the balance user is trying to send
//     let sender_updated_balance: Vec<u64> = vec![5u64, 7u64];
//     let sender_updated_balance: Vec<u64> = Vec::from([5u64, 7u64]);

//     let reciever_updated_balance: Vec<u64> = vec![5u64, 3u64];

//     let base_pk = RistrettoPublicKey::generate_base_pk();

//     //create DarkTx Prover merlin transcript
//     let mut transcript = Transcript::new(b"TxProof");
//     let mut prover = Prover::new(b"DarkTx", &mut transcript);
//     let value_vector: Vec<i64> = vec![5, 3];

//     let tx = TransferTransaction::create_quisquis_transaction(
//         &Vec::<Input>::new(),
//         value_vector.as_slice(),
//         &account_vector,
//         &sender_updated_balance,
//         &reciever_updated_balance,
//         &sender_sk,
//         2,
//         2,
//         anonymity_comm_scalar,
//         anonymity_account_diff,
//         tx_log,
//     );
// }
