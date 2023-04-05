use rpcclient::method::Method;
use rpcclient::txrequest::{Resp, RpcBody, RpcRequest};
use rpcserver::*;
use transaction::Transaction;
use transactionapi::{rpcclient, rpcserver};
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

            let tx_send: RpcBody<Transaction> = RpcRequest::new(tx, Method::TxQueue);
            let res = tx_send.send("http://127.0.0.1:3030".to_string());

            // println!("res:{:#?}", res.unwrap().bytes());

            // let resp: Resp = serde_json::from_slice(&res.unwrap().bytes().unwrap()).unwrap();
            // println!("res:{:#?}", resp);
            rpcclient::txrequest::rpc_response(res);
        })
        .unwrap();
    rpcserver();
    handle.join().unwrap();
}
