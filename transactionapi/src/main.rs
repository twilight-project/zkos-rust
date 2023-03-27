// mod quisquislib;
mod rpcclient;
mod rpcserver;
use rpcclient::method::Method;
use rpcclient::txrequest;
use rpcclient::txrequest::{RpcBody, RpcRequest};
// use crate::trasaction;
// mod transaction::tx;

use std::fs::File;
use std::io::prelude::*;

use rpcserver::*;
use transaction::TransactionData;
use transaction::TransferTransaction;
#[macro_use]
extern crate lazy_static;
use transaction::reference_tx::{
    create_dark_reference_transaction, create_qq_reference_transaction,
};
fn main() {
    // let handle = std::thread::Builder::new()
    //     .name(String::from("rpc request"))
    //     .spawn(move || {
    //         std::thread::sleep(std::time::Duration::from_millis(5000));
    let tx = create_dark_reference_transaction();
    // println!("tx: {:?}", tx);
    // let mut file = File::create("foo.txt").unwrap();
    // file.write_all(&serde_json::to_vec(&tx).unwrap()).unwrap();
    // let tx1 = serde_json::to_vec(&tx.clone()).unwrap();
    // let mut file2 = File::create("foo2.txt").unwrap();
    // file2.write_all(&serde_json::to_vec(&tx1).unwrap()).unwrap();
    // // let tx3: transaction::Transaction = serde_json::from_str(&tx1).unwrap();
    // let tx3: transaction::Transaction =
    //     serde_json::from_str(&String::from_utf8_lossy(&tx1)).unwrap();
    // let mut file2 = File::create("foo3.txt").unwrap();
    // file2.write_all(&serde_json::to_vec(&tx3).unwrap()).unwrap();
    // let tx_send: RpcBody<transaction::Transaction> =
    //     RpcRequest::<transaction::Transaction>::new(tx, Method::TxCommit);
    // let res = tx_send.send("http://127.0.0.1:3030".to_string());
    // println!("res:{:#?}", res.unwrap().bytes());

    //**************************** */
    // let proff = tx.tx.clone();
    // let prooof: TransferTransaction = match proff {
    //     TransactionData::TransactionTransfer(xx) => xx,
    // };
    // let prooff = prooof.get_proof();
    // let mut file = File::create("foo.txt").unwrap();
    // file.write_all(&serde_json::to_vec(&prooff).unwrap())
    //     .unwrap();
    // let tx1 = serde_json::to_value(&prooff.clone()).unwrap();
    // let mut file2 = File::create("foo2.txt").unwrap();
    // file2.write_all(&serde_json::to_vec(&tx1).unwrap()).unwrap();

    // let tx3: transaction::DarkTxProof = serde_json::from_value(tx1).unwrap();
    // let mut file2 = File::create("foo3.txt").unwrap();
    //     })
    //     .unwrap();
    // // rpcserver();
    // handle.join().unwrap();
}
