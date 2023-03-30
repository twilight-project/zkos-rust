use super::threadpool::ThreadPool;
// use std::sync::mpsc;
// use std::sync::Arc;
use std::sync::Mutex;
// use std::thread;
use transaction::Transaction;
// extern crate lazy_static;
// #[macro_use]
lazy_static! {
    pub static ref THREADPOOL_RPC_QUEUE: Mutex<ThreadPool> =
        Mutex::new(ThreadPool::new(10, String::from("THREADPOOL_RPC_Queue")));
}
pub fn tx_queue(transaction: Transaction) {
    let queue = THREADPOOL_RPC_QUEUE.lock().unwrap();
    queue.execute(move || {
        //put tx in queue
    });
    drop(queue);
}
pub fn tx_commit(transaction: Transaction) {

    // need to call transaction function
}
pub fn tx_status(transaction: Transaction) {}
