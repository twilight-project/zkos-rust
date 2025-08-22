// Copyright © 2019 Interstellar & Stellar Development Foundation
// Copyright © 2025 Twilight Project Contributors
// SPDX-License-Identifier: Apache-2.0
//
// Portions of this file are derived from the `zkvm` crate in the
// `stellar/slingshot` project (Apache-2.0).

#![allow(missing_docs)]
//! ZkVM (_zero-knowledge virtual machine_): a transaction format for a shared, multi-asset, cryptographic ledger.
//!
//! ZkVM provides a secure, privacy-preserving transaction system that enables:
//! - **Zero-knowledge proofs** for transaction validation without revealing sensitive data
//! - **Multi-asset support** with confidential values and flexible predicates
//! - **Programmable contracts** with custom logic and access control
//! - **Merkle tree-based state** for efficient verification and storage
//!
//! ## Core Components
//!
//! - **Transaction System**: [`Tx`], [`TxEntry`], [`TxHeader`] for creating and validating transactions
//! - **Virtual Machine**: [`VMRun`], [`VMScript`] for executing ZkVM programs
//! - **Constraint System**: [`Constraint`], [`Expression`], [`Variable`] for zero-knowledge proofs
//! - **Contract System**: [`Contract`], [`ContractID`], [`PortableItem`] for programmable state
//! - **Predicate System**: [`Predicate`], [`PredicateTree`] for access control and authorization
//!
//! ## Key Features
//!
//! - **Prover/Verifier**: [`Prover`] for creating proofs, [`Verifier`] for validation
//! - **Encoding**: Serialization and deserialization with [`TranscriptProtocol`]
//! - **Types**: [`Value`], [`String`], [`Address`] for transaction data
//! - **UTXO Model**: [`Input`], [`Output`], [`Utxo`] for state management
//!
//! ## Documentation
//!
//! * [ZkVM whitepaper](https://github.com/stellar/slingshot/blob/main/zkvm/docs/zkvm-design.md) — technology overview
//! * [ZkVM specification](https://github.com/stellar/slingshot/blob/main/zkvm/docs/zkvm-spec.md) — transaction validation rules
//! * [Blockchain specification](https://github.com/stellar/slingshot/blob/main/zkvm/docs/zkvm-blockchain.md) — blockchain state machine specification
//! * [ZkVM API](https://github.com/stellar/slingshot/blob/main/zkvm/docs/zkvm-api.md) — how to create transactions with ZkVM
//!
//! ## Example
//! ```
//! use zkvm::{Prover, Verifier, Tx, Value, Predicate};
//!
//! // Create a transaction with confidential values
//! let mut prover = Prover::new();
//! let value = Value::confidential(100u64);
//! let predicate = Predicate::Opaque(Default::default());
//!
//! // Build and prove transaction
//! let tx = Tx::new().with_output(value, predicate);
//! let proof = prover.prove(tx)?;
//!
//! // Verify the proof
//! let mut verifier = Verifier::new();
//! verifier.verify(proof)?;
//! ```

pub extern crate bulletproofs;
pub extern crate merkle;
extern crate serde;

#[macro_use]
/// Serialization macros for ZkVM.
mod serialization;

/// Constraint system types and operations for ZkVM.
pub mod constraints;

/// Contract system types and operations for ZkVM.
mod contract;

/// Debug utilities for ZkVM.
mod debug;

/// Encoding utilities for ZkVM.
pub mod encoding;

/// Error handling for ZkVM.
pub mod errors;

/// Fee system types and operations for ZkVM.
mod fees;

/// Instruction and opcode definitions for ZkVM.
pub mod ops;

/// Predicate system types and operations for ZkVM.
pub mod predicate;

/// Program system types and operations for ZkVM.
pub mod program;

/// Prover system types and operations for ZkVM.
pub mod prover;

/// Scalar witness types and operations for ZkVM.
mod scalar_witness;

/// Transcript system types and operations for ZkVM.
mod transcript;

/// Transaction system types and operations for ZkVM.
pub mod tx;

/// Types for ZkVM.
mod types;

/// Verifier system types and operations for ZkVM.
pub mod verifier;

/// Virtual machine system types and operations for ZkVM.
pub mod vm;

/// ZkOS types for ZkVM.
pub mod zkos_types;

pub use self::constraints::{Commitment, CommitmentWitness, Constraint, Expression, Variable};
pub use self::contract::{Anchor, Contract, ContractID, PortableItem};
pub use self::errors::VMError;
pub use self::fees::{fee_flavor, CheckedFee, FeeRate, MAX_FEE};
pub use self::ops::{Instruction, Opcode};
pub use self::predicate::{Predicate, PredicateTree};
pub use self::program::{Program, ProgramItem};
pub use self::prover::Prover;
pub use self::scalar_witness::ScalarWitness;
pub use self::transcript::TranscriptProtocol;
pub use self::tx::{Tx, TxEntry, TxHeader, TxID, TxLog, UnsignedTx, VerifiedTx};
pub use self::types::{ClearValue, Item, String, Value, WideValue};
pub use self::verifier::Verifier;
pub use self::vm::{VMRun, VMScript};
pub use merkle::{Hash, Hasher, MerkleItem, MerkleTree};

pub use address::Address;
pub use mulmsgsig::{Signature, VerificationKey};

pub use self::zkos_types::{
    IOType, IOType::Coin, Input, InputData, Output, OutputData, Utxo, Witness,
};
