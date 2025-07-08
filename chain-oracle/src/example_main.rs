use stopwatch::Stopwatch;
use zkoracle_rust::pubsub_chain;
fn main() {
    dotenv::dotenv().expect("Failed loading dotenv");
    let (receiver, handle) = pubsub_chain::subscribe_block(true);
    loop {
        let sw = Stopwatch::start_new();
        match receiver.lock().unwrap().recv() {
            Ok(block_data) => {
                println!("Block: {:#?}\n", block_data);
            }
            Err(arg) => {
                println!("subscriber crashed : {:?}", arg);
                break;
            }
        }
        println!("Thing took {:?}\n\n", sw.elapsed());
    }
    handle.join().unwrap()
}
