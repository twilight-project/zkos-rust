//#![deny(missing_docs)]
#![allow(non_snake_case)]
//! ZkOS Transaction implementation.

pub extern crate quisquislib;

#[macro_use]

mod constants;
mod errors;
mod message;
mod proof;
pub mod reference_tx;
mod script_tx;
mod serialization;
mod transaction;
mod transfer_tx;
pub mod vm_run;
//mod encode;
#[cfg(test)]
mod tests;

// re-exports
pub use self::errors::TxError;
pub use self::message::Message;
pub use self::proof::{DarkTxProof, ShuffleTxProof};
pub use self::reference_tx::{Receiver, Sender};
pub use self::script_tx::ScriptTransaction;
pub use self::transaction::{Transaction, TransactionData, TransactionType};
pub use self::transfer_tx::TransferTransaction;

//pub use self::encode::{ReaderExt, WriterExt};
