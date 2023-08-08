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
pub mod tx;
pub mod types;
pub mod util;

pub use self::errors::TxError;
pub use self::proof::{DarkTxProof, ShuffleTxProof};
pub use self::tx::{ScriptTransaction, Transaction, TransactionData, TransferTransaction};

pub use self::types::{
    CData, Coin, Input, InputType, Memo, Output, OutputType, State, TransactionType, TxId, Utxo,
    Witness,
};

pub use self::util::{Address, Network};

pub use self::reference_tx::Sender;
