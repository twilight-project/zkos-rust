//! Chain Oracle
//!
//! A Rust library for connecting to a Cosmos-based blockchain, retrieving, parsing, and analyzing blocks and transactions—designed for integration with zkVM and related systems.
//!
//! # License
//! Licensed under the Apache License, Version 2.0.

#[macro_use]
extern crate lazy_static;
mod block_types;
mod transaction_types;
pub use self::block_types::*;
pub use self::transaction_types::*;
pub mod pubsub_chain;
mod threadpool;
pub use self::pubsub_chain::NYKS_BLOCK_SUBSCRIBER_URL;
pub use threadpool::ThreadPool;
