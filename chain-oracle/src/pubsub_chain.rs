use std::{
    sync::{mpsc, Arc, Mutex},
    thread, time,
};
// #[macro_use]
// extern crate lazy_static;
lazy_static! {
    pub static ref BLOCK_SERIALIZER_THREADPOOL: Arc<Mutex<ThreadPool>> = Arc::new(Mutex::new(
        ThreadPool::new(1, String::from("BLOCK_SERIALIZER_THREADPOOL Threadpool"))
    ));
    pub static ref NYKS_BLOCK_SUBSCRIBER_URL: String = std::env::var("NYKS_BLOCK_SUBSCRIBER_URL")
    .unwrap_or("http://localhost:1317/".to_string());
}
use crate::{block_types::Block, BlockRaw, ThreadPool};

pub fn subscribe_block(
    empty_block: bool,
) -> (Arc<Mutex<mpsc::Receiver<Block>>>, thread::JoinHandle<()>) {
    let (sender, receiver) = mpsc::channel();
    let receiver_mutex = Arc::new(Mutex::new(receiver));

    let subsciber_thread = thread::Builder
        ::new()
        .name("subsciber_thread".to_string())
        .spawn(move || {
            let  block_ser_threadpool = BLOCK_SERIALIZER_THREADPOOL.lock().unwrap();
            let mut latest_height = match BlockRaw::get_latest_block_height() {
                Ok(height) => height,
                Err(arg) => {
                    println!("Can not get latest height \nError: {:?}\nSetting height to 0", arg);
                    0
                }
            };
            let mut block_height = Block::get_local_block_height();

            loop {
                let mut attempt = 0;
                while block_height <= latest_height {
                    let block_raw_result = BlockRaw::get_block_data_from_height(block_height);
                    let sender1 = sender.clone();
                    match block_raw_result {
                        Ok(block_raw) => {
                            block_ser_threadpool.execute(move || {
                            let block = Block::new(block_raw);
                            if block.transactions.len() > 0 || empty_block {
                                sender1.clone().send(block).unwrap();
                            }
                        });
                            block_height += 1;
                            // Block::write_local_block_height(block_height);
                        }
                        Err(arg) => {
                            if arg == "3".to_string() {
                                println!("block fetching at block height :{}, return code=3, fetching next block", block_height);
                                block_height += 1;
                            } else {
                                attempt += 1;
                                println!(
                                    "block fetching error at block height : {:?} \nError:{:?}",
                                    block_height,
                                    arg
                                );
                                thread::sleep(time::Duration::from_millis(1000));
                                if attempt == 3 {
                                    println!("block fetching at block height :{} failed after 3 attempts, fethcing next block", block_height);
                                    block_height += 1;
                                    attempt = 0;
                                }
                            }
                        }
                    }
                
                }
                let mut height_attempt = 0;
                while latest_height < block_height {
                    thread::sleep(time::Duration::from_millis(1200));
                    latest_height = match BlockRaw::get_latest_block_height() {
                        Ok(height) => height,
                        Err(arg) => {
                            height_attempt += 1;
                            if height_attempt == 5 {
                                println!("Cannot get latest height \nError: {:?}\n", arg);
                            }
                            thread::sleep(time::Duration::from_millis(500));
                            0
                        }
                    };
                    if height_attempt == 10 {
                        height_attempt = 0;
                        println!("\nCannot get latest height in 10 attemps, sleeping for 30 sec");
                        thread::sleep(time::Duration::from_millis(30000));
                    }
                }
            }
        })
        .unwrap();

    return (Arc::clone(&receiver_mutex), subsciber_thread);
}

pub fn request_url(url: &str) -> Result<String, String> {
    let client = reqwest::blocking::Client::new();
    match client.get(url).send() {
        Ok(res) => match res.text() {
            Ok(text) => Ok(text),
            Err(arg) => {
                return Err(arg.to_string());
            }
        },
        Err(arg) => {
            return Err(arg.to_string());
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::{block_types::Block, BlockRaw};

    use super::subscribe_block;

    #[test]
    fn get_latest_block_test() {
        let latest_block_height = BlockRaw::get_latest_block_height();
        match latest_block_height {
            Ok(height) => println!("Latest Block Height : {}", height),
            Err(arg) => println!("Got Error finding Latest Height with error : {:?}", arg),
        }
    }

    #[test]
    fn get_block_raw_data_from_height_test() {
        let block_data = BlockRaw::get_block_data_from_height(415156);
        match block_data {
            Ok(block) => println!("Block: {:#?}", block),
            Err(arg) => println!(
                "Got Error finding block from Height: {} with error : {:?}",
                415156, arg
            ),
        }
    }
    #[test]
    fn get_block_raw_data_from_wrong_height_test() {
        let block_data = BlockRaw::get_block_data_from_height(0);
        match block_data {
            Ok(block) => println!("Block: {:#?}", block),
            Err(arg) => println!(
                "\nGot Error finding block from Height: {} with error code: {:?}",
                0, arg
            ),
        }
    }

    #[test]
    fn get_block_decoded_transfer_tx_test() {
        // "/twilightproject.nyks.zkos.MsgTransferTx"
        let block_data = BlockRaw::get_block_data_from_height(415156);

        match block_data {
            Ok(block) => {
                let block = Block::new(block);
                println!("Block: {:#?}", block)
            }
            Err(arg) => println!(
                "Got Error finding block from Height: {} with error : {:?}",
                415156, arg
            ),
        }
    }
    #[test]
    fn get_block_decoded_mint_or_burn_test() {
        // "@type": "/twilightproject.nyks.zkos.MsgMintBurnTradingBtc",
        let block_data = BlockRaw::get_block_data_from_height(380157);
        match block_data {
            Ok(block) => {
                let block = Block::new(block);
                println!("Block: {:#?}", block)
            }
            Err(arg) => println!(
                "Got Error finding block from Height: {} with error : {:?}",
                380157, arg
            ),
        }
    }
}
