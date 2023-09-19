use rpcclient::method::Method;
use rpcclient::txrequest::{Resp, RpcBody, RpcRequest};
use rpcserver::*;
use std::thread;
use std::time::Duration;
use transaction::Transaction;
use transactionapi::TransactionStatusId;
use transactionapi::{rpcclient, rpcserver};
#[macro_use]
extern crate lazy_static;
use transaction::reference_tx::{
    create_dark_reference_transaction, create_qq_reference_transaction,
};
use utxo_in_memory::{init_utxo, zk_oracle_subscriber};
fn main() {
    let handle = std::thread::Builder::new()
        .name(String::from("rpc request"))
        .spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(5000));
            let tx = create_dark_reference_transaction();

            let tx_send: RpcBody<Transaction> = RpcRequest::new(tx, Method::TxCommit);
            let res = tx_send.send("http://127.0.0.1:3030".to_string());

            // println!("res:{:#?}", res.unwrap().bytes());

            // let resp: Resp = serde_json::from_slice(&res.unwrap().bytes().unwrap()).unwrap();
            // println!("res:{:#?}", resp);
            match res {
                Ok(x) => {
                    println!("res1:{:#?}", x);
                }
                Err(arg) => {
                    println!("errr1:{:#?}", arg);
                }
            }
            let tx_send: RpcBody<TransactionStatusId> = RpcRequest::new(
                TransactionStatusId {
                    txid: "5f516a8a-fc68-4a34-b299-373dcaae6b4c".to_string(),
                },
                Method::TxStatus,
            );
            let res = tx_send.send("http://127.0.0.1:3030".to_string());
            match res {
                Ok(x) => {
                    println!("res:{:#?}", x);
                }
                Err(arg) => {
                    println!("errr:{:#?}", arg);
                }
            }
        })
        .unwrap();
    init_utxo();

    let handle = thread::spawn(|| {
        zk_oracle_subscriber();
    });
    rpcserver();
    handle.join().unwrap();
    //  handle.join().unwrap();
}
