use rpcclient::method::Method;
use rpcclient::txrequest::{Resp, RpcBody, RpcRequest};
use rpcserver::*;
use serde_json::to_string;
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
#[macro_use] extern crate rocket;
use rocket::data::{Limits, ToByteUnit};
use rocket::{State, response::content};
use prometheus::{Encoder, TextEncoder, Counter, Gauge, register_counter, register_gauge};


fn main() {
    init_utxo(); // Execute synchronously

    let zk_subscriber_thread = thread::spawn(|| {
        zk_oracle_subscriber();
    });

    let rpc_server_thread = thread::spawn(|| {
        rpcserver();
    });


    // Now start the async part
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async_main());

    zk_subscriber_thread.join().unwrap();
    rpc_server_thread.join().unwrap();
}

async fn async_main() {

    let telemetry_server_thread = std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            run_telemetry_server().await;
        })
    });

    telemetry_server_thread.join();

}

async fn run_telemetry_server(){
    println!("starting telemetry server");
        
    let figment = rocket::Config::figment()
    .merge(("address", "0.0.0.0"))
    .merge(("port", 2500))
    .merge(("limits", Limits::new().limit("json", 2.mebibytes())));

    let rocket = rocket::custom(figment).mount("/", routes![telemetry_metrics]);

    // Await the launch of the server
    if let Err(e) = rocket.launch().await {
        println!("Failed to launch server: {}", e);
    }
}

#[get("/metrics")]
fn telemetry_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    let result = String::from_utf8(buffer).unwrap();
    return result
}


// fn main() {
        // let handle = std::thread::Builder::new()
    //     .name(String::from("rpc request"))
    //     .spawn(move || {
    //         std::thread::sleep(std::time::Duration::from_millis(5000));
    //         let tx = create_dark_reference_transaction();

    //         let tx_send: RpcBody<Transaction> = RpcRequest::new(tx, Method::TxCommit);
    //         let res = tx_send.send("http://127.0.0.1:3030".to_string());

    //         // println!("res:{:#?}", res.unwrap().bytes());

    //         // let resp: Resp = serde_json::from_slice(&res.unwrap().bytes().unwrap()).unwrap();
    //         // println!("res:{:#?}", resp);
    //         match res {
    //             Ok(x) => {
    //                 println!("res1:{:#?}", x);
    //             }
    //             Err(arg) => {
    //                 println!("errr1:{:#?}", arg);
    //             }
    //         }
    //         let tx_send: RpcBody<TransactionStatusId> = RpcRequest::new(
    //             TransactionStatusId {
    //                 txid: "5f516a8a-fc68-4a34-b299-373dcaae6b4c".to_string(),
    //             },
    //             Method::TxStatus,
    //         );
    //         let res = tx_send.send("http://127.0.0.1:3030".to_string());
    //         match res {
    //             Ok(x) => {
    //                 println!("res:{:#?}", x);
    //             }
    //             Err(arg) => {
    //                 println!("errr:{:#?}", arg);
    //             }
    //         }
    //     })
    //     .unwrap();
// }