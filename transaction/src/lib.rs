//#![deny(missing_docs)]
#![allow(non_snake_case)]

//! ZkOS Transaction implementation.

pub extern crate quisquislib;

#[macro_use]

mod constants;
mod errors;
mod proof;
mod serialization;
mod tx;
pub mod types;
pub mod util;
//mod encode;

pub use self::errors::TxError;
pub use self::proof::{DarkTxProof, ShuffleTxProof};
pub use self::tx::{Transaction, TransferTransaction};
//pub use self::encode::{ReaderExt, WriterExt};

pub use self::types::{
    Input, InputType, Output, OutputType, TransactionType, TxId, TxPointer, Utxo, Witness, InputData, OutputData, Coin
};

pub use self::util::{Address, Network};

