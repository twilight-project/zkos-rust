mod rpcserver;
use rpcserver::*;
#[macro_use]
extern crate lazy_static;
fn main() {
rpcserver();
}
