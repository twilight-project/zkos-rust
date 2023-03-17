mod rpcserver;
// use crate::trasaction;
// mod transaction::tx;
use rpcserver::*;
#[macro_use]
extern crate lazy_static;
fn main() {
    rpcserver();
}
