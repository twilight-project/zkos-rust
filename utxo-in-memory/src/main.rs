// #[macro_use]
use utxo_in_memory::blockoperations;
use utxo_in_memory::db::LocalDBtrait;
extern crate lazy_static;
use utxo_in_memory::*;
use serde::Deserialize;
use tungstenite::{connect, Message};
use url::Url;

fn main() {

    let sw = Stopwatch::start_new();
    init_utxo();
    let time1 = sw.elapsed();
    println!("init_utxo: {:#?}", time1);

    save_snapshot();
    socket_connection();

}

fn save_snapshot(){
    let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
    println!("get block height:{:#?}", utxo_storage.block_height);
    println!("get snap:{:#?}", utxo_storage.snaps);
    for i in 0..utxo_storage.partition_size {
        println!("get snap:{:#?}", utxo_storage.data.get(&i).unwrap().len());
    }
    utxo_storage.take_snapshot();
}


fn socket_connection() {
    let (mut socket, response) =
        connect(Url::parse("ws://165.232.134.41:7001/latestblock").unwrap()).expect("Can't connect");

    loop {
        let msg = socket.read_message().expect("Error reading message");
        match msg {
            Message::Text(text) => {
                println!("{}", text);
                let block: blockoperations::blockprocessing::Block = serde_json::from_str(&text).unwrap();
                blockoperations::blockprocessing::process_block_for_utxo_insert(block);
            }
            Message::Close(_) => {
                println!("Server disconnected");
                break;
            }
            _ => (),
        }
        save_snapshot();
    }
}
use curve25519_dalek::scalar::Scalar;
use quisquislib::accounts::Account;
use std::io::prelude::*;

// pub fn load_utxo() {
//     let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
//     let (acc, prv) = Account::generate_random_account_with_value(Scalar::from(20u64));
//     // let mut recordutxo = transaction::reference_tx::create_genesis_block(10000, 100, acc);
//     let mut recordutxo = crate::dbcurd::load_genesis_sets_test();
//     println!("new utxo0 len:{:#?}", recordutxo.len());
//     let block1 = transaction::reference_tx::create_utxo_test_block(
//         &mut recordutxo,
//         utxo_storage.block_height as u64,
//         &vec![prv],
//     );
//     println!("new utxo len:{:#?}", recordutxo.len());
//     let (acc, prv) = Account::generate_random_account_with_value(Scalar::from(20u64));
//     let block2 = transaction::reference_tx::create_utxo_test_block(
//         &mut recordutxo,
//         (utxo_storage.block_height + 1) as u64,
//         &vec![prv],
//     );
//     println!("new utxo len:{:#?}", recordutxo.len());
//     let (acc, prv) = Account::generate_random_account_with_value(Scalar::from(20u64));
//     let block3 = transaction::reference_tx::create_utxo_test_block(
//         &mut recordutxo,
//         (utxo_storage.block_height + 2) as u64,
//         &vec![prv],
//     );
//     println!("new utxo len:{:#?}", recordutxo.len());
//     let mut file =
//         std::fs::File::create("../utxo-in-memory\\src\\dbcurd\\test\\block1.txt").unwrap();
//     file.write_all(&serde_json::to_vec_pretty(&block1.clone()).unwrap())
//         .unwrap();
//     let mut file =
//         std::fs::File::create("../utxo-in-memory\\src\\dbcurd\\test\\block2.txt").unwrap();
//     file.write_all(&serde_json::to_vec_pretty(&block2.clone()).unwrap())
//         .unwrap();
//     let mut file =
//         std::fs::File::create("../utxo-in-memory\\src\\dbcurd\\test\\block3.txt").unwrap();
//     file.write_all(&serde_json::to_vec_pretty(&block3.clone()).unwrap())
//         .unwrap();

//     let zkblock = ZkosBlock::get_block_details(block1);
//     let resultblock1 = utxo_storage.process_block(zkblock);
//     let mut file =
//         std::fs::File::create("../utxo-in-memory\\src\\dbcurd\\test\\resultblock1.txt").unwrap();
//     file.write_all(
//         &serde_json::to_vec_pretty(&format!("{:#?}", resultblock1.unwrap().error_vec)).unwrap(),
//     )
//     .unwrap();

//     let zkblock = ZkosBlock::get_block_details(block2);
//     let resultblock2 = utxo_storage.process_block(zkblock);
//     let mut file =
//         std::fs::File::create("../utxo-in-memory\\src\\dbcurd\\test\\resultblock2.txt").unwrap();
//     file.write_all(
//         &serde_json::to_vec_pretty(&format!("{:#?}", resultblock2.unwrap().error_vec)).unwrap(),
//     )
//     .unwrap();

//     let zkblock = ZkosBlock::get_block_details(block3);
//     let zkblock_clone = zkblock.clone();
//     let sw1 = Stopwatch::start_new();
//     let resultblock3 = utxo_storage.process_block(zkblock_clone);
//     let time2 = sw1.elapsed();
//     println!(
//         "utxo_storage.process_block: {:#?}\n with len:{:#?}",
//         time2,
//         zkblock.add_utxo.len() + zkblock.remove_block.len()
//     );
//     let mut file =
//         std::fs::File::create("../utxo-in-memory\\src\\dbcurd\\test\\resultblock3.txt").unwrap();
//     file.write_all(
//         &serde_json::to_vec_pretty(&format!("{:#?}", resultblock3.unwrap().error_vec)).unwrap(),
//     )
//     .unwrap();
//     let sw = Stopwatch::start_new();
//     let _ = utxo_storage.take_snapshot();
//     let time1 = sw.elapsed();
//     println!(
//         "utxo_storage.take_snapshot: {:#?} with len:{:#?}",
//         time1,
//         utxo_storage.coin_storage.len()
//             + utxo_storage.memo_storage.len()
//             + utxo_storage.state_storage.len()
//     );
//     let mut file =
//         std::fs::File::create("../utxo-in-memory\\src\\dbcurd\\test\\genesis_sets_test.txt")
//             .unwrap();
//     file.write_all(&serde_json::to_vec_pretty(&recordutxo.clone()).unwrap())
//         .unwrap();
// }

use stopwatch::Stopwatch;
