// Copyright © 2025 Twilight Project Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(non_snake_case)]
//! ZkOS Transaction implementation for confidential blockchain transactions.
//!
//! This crate provides a comprehensive transaction system for the ZkOS blockchain,
//! supporting multiple transaction types with zero-knowledge proofs and confidential
//! transfer capabilities.
//!
//! # Features
//!
//! - **Confidential Transfers**: Dark and QuisQuis transactions with zero-knowledge proofs
//! - **Script Transactions**: Smart contract execution with R1CS proofs
//! - **Message Transactions**: Burn and other message-based operations
//! - **Multi-Asset Support**: Handle different asset types and flavors
//! - **Range Proofs**: Bulletproofs-based range verification
//! - **Shuffle Proofs**: Input/output shuffling for privacy
//!
//! # Transaction Types
//!
//! ## Transfer Transactions
//! - **Dark Transactions**: Fully confidential transfers with delta/epsilon proofs
//! - **QuisQuis Transactions**: Shuffled transfers with enhanced privacy
//! - **Lit to Lit**: Transparent transfers between public addresses
//!
//! ## Script Transactions
//! - **Smart Contract Execution**: Run programs with R1CS proofs
//! - **Contract Deployment**: Deploy new contracts to the blockchain
//! - **State Management**: Handle contract state and witnesses
//!
//! ## Message Transactions
//! - **Burn Messages**: Destroy assets with reveal proofs
//! - **Custom Messages**: Extensible message system
//!
//! # Example
//!
//! ```rust
//! use transaction::{Transaction, TransactionType, TransactionData};
//! use transaction::{TransferTransaction, ScriptTransaction, Message};
//! use zkvm::zkos_types::{Input, Output};
//!
//! // Create a transfer transaction
//! let transfer_tx = TransferTransaction::create_quisquis_transaction(
//!     &inputs,
//!     &value_vector,
//!     &account_vector,
//!     &sender_updated_balance,
//!     &receiver_value_balance,
//!     &sender_sk,
//!     senders_count,
//!     receivers_count,
//!     anonymity_account_diff,
//!     witness_comm_scalar,
//!     fee,
//! ).unwrap();
//!
//! let transaction = Transaction::new(
//!     TransactionType::Transfer,
//!     TransactionData::TransactionTransfer(transfer_tx),
//! );
//!
//! // Verify the transaction
//! assert!(transaction.verify().is_ok());
//! ```
//!
//! # Architecture
//!
//! The transaction system is built on several key components:
//!
//! - **Proof System**: Bulletproofs and QuisQuis for zero-knowledge proofs
//! - **Cryptographic Primitives**: Ristretto255 curve, ElGamal encryption
//! - **VM Integration**: ZkVM for script execution and state management
//! - **Address System**: Multi-format address support
//!
//! # Security
//!
//! - Zero-knowledge proofs ensure transaction privacy
//! - Cryptographic commitments prevent double-spending
//! - Range proofs verify value bounds
//! - Shuffle proofs provide input/output privacy
//! - Signature verification ensures authorization

pub extern crate quisquislib;

#[macro_use]

/// Transaction constants and limits.
mod constants;
/// Error types for transaction operations.
mod errors;
/// Message transaction implementation.
mod message;
/// Zero-knowledge proof implementations.
mod proof;
// Reference transaction types.
pub mod reference_tx;
/// Script transaction implementation.
mod script_tx;
/// Transaction serialization utilities.
mod serialization;
/// Core transaction types and structures.
mod transaction;
/// Transfer transaction implementation.
mod transfer_tx;
/// Virtual machine execution utilities.
pub mod vm_run;

#[cfg(test)]
mod tests;

// Re-exports for public API
pub use self::errors::TxError;
pub use self::message::Message;
pub use self::proof::{DarkTxProof, ShuffleTxProof};
pub use self::reference_tx::{Receiver, Sender};
pub use self::script_tx::ScriptTransaction;
pub use self::transaction::{Transaction, TransactionData, TransactionType};
pub use self::transfer_tx::TransferTransaction;
