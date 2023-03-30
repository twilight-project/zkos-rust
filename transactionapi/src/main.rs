// mod quisquislib;
mod rpcclient;
mod rpcserver;
use rpcclient::method::Method;
use rpcclient::txrequest::{RpcBody, RpcRequest};

use rpcserver::*;
// use transaction::Transaction;
#[macro_use]
extern crate lazy_static;
use transaction::reference_tx::{
    create_dark_reference_transaction, create_qq_reference_transaction,
};
fn main() {
    let handle = std::thread::Builder::new()
        .name(String::from("rpc request"))
        .spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(5000));
            let tx = create_qq_reference_transaction();

            let tx_send: RpcBody<transaction::Transaction> =
                RpcRequest::<transaction::Transaction>::new(tx, Method::TxQueue);
            let res = tx_send.send("http://127.0.0.1:3030".to_string());
            println!("res:{:#?}", res.unwrap().bytes());
        })
        .unwrap();
    rpcserver();
    handle.join().unwrap();
}
