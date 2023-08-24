//#![deny(missing_docs)]
#![allow(non_snake_case)]
//! ZkOS Transaction implementation.

pub extern crate quisquislib;

#[macro_use]

mod constants;
mod errors;
pub mod proof;
pub mod reference_tx;
pub mod script_tx;
mod serialization;
pub mod transfer_tx;
pub mod vm_run;
mod verify_relayer;
//mod encode;
#[cfg(test)]
mod tests;
pub use self::errors::TxError;
pub use self::proof::{DarkTxProof, ShuffleTxProof};
pub use self::reference_tx::{Receiver, Sender};
pub use self::script_tx::ScriptTransaction;
pub use self::transfer_tx::{Transaction, TransactionData, TransactionType, TransferTransaction};
//pub use self::encode::{ReaderExt, WriterExt};
