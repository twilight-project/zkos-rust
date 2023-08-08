//#![deny(missing_docs)]
#![allow(non_snake_case)]

//! ZkOS Transaction implementation.

pub extern crate quisquislib;

#[macro_use]

mod constants;
mod errors;
mod proof;
pub mod reference_tx;
mod serialization;
// mod tx;
pub mod tx;
pub mod types;
pub mod util;
pub use self::errors::TxError;
pub use self::proof::{DarkTxProof, ShuffleTxProof};
pub use self::tx::{Transaction, TransferTransaction};

pub use self::types::{
    Input, InputType, Output, OutputType, TransactionType, TxId, TxPointer, Utxo, Witness,
};

pub use self::util::{Address, Network};

pub use self::reference_tx::{Sender, Receiver};
