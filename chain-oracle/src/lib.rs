//! # Chain Oracle
//!
//! A Rust library for connecting to a Cosmos-based blockchain, retrieving, parsing,
//! and analyzing blocks and transactions.
//!
//! ## Features
//!
//! - Block subscription and processing
//! - Transaction parsing and analysis
//! - Threaded processing pool
//! - Configurable endpoints
//!
//! # License
//! Licensed under the Apache License, Version 2.0.

/// Block types and processing
mod block_types;
/// Chain subscription and monitoring
pub mod pubsub_chain;
/// Thread pool implementation
mod threadpool;
/// Transaction types and processing
mod transaction_types;

pub use self::block_types::*;
/// Public use
/// The default NYKS block subscriber URL, configurable via environment variable.
pub use self::pubsub_chain::NYKS_BLOCK_SUBSCRIBER_URL;
pub use self::transaction_types::*;
pub use threadpool::ThreadPool;
#[macro_use]
extern crate lazy_static;
