//#![deny(missing_docs)]
#![allow(non_snake_case)]
//! ZkOS Transaction implementation.

pub extern crate quisquislib;

#[macro_use]

mod constants;
mod errors;
mod proof;
mod reference_tx;
mod script_tx;
mod serialization;
mod transfer_tx;
mod vm_run;

//mod encode;
#[cfg(test)]
mod tests;
pub use self::errors::TxError;
pub use self::proof::{DarkTxProof, ShuffleTxProof};
pub use self::transfer_tx::{Transaction, TransferTransaction};
//pub use self::encode::{ReaderExt, WriterExt};
