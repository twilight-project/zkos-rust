#![allow(missing_docs)]
//! ZkVM (_zero-knowledge virtual machine_): a transaction format for a shared, multi-asset, cryptographic ledger.
//!
//! * [ZkVM whitepaper](https://github.com/stellar/slingshot/blob/main/zkvm/docs/zkvm-design.md) — technology overview.
//! * [ZkVM specification](https://github.com/stellar/slingshot/blob/main/zkvm/docs/zkvm-spec.md) — transaction validation rules.
//! * [Blockchain specification](https://github.com/stellar/slingshot/blob/main/zkvm/docs/zkvm-blockchain.md) — blockchain state machine specification.
//! * [ZkVM API](https://github.com/stellar/slingshot/blob/main/zkvm/docs/zkvm-api.md) — how to create transactions with ZkVM.

pub extern crate bulletproofs;
pub extern crate merkle;
extern crate serde;

#[macro_use]
mod serialization;
pub mod constraints;
mod contract;
mod debug;
pub mod encoding;
pub mod errors;
mod fees;
pub mod ops;
pub mod predicate;
pub mod program;
///ZkVM Prover
pub mod prover;
mod scalar_witness;
mod transcript;
pub mod tx;
mod types;
///ZKVM Verifier
pub mod verifier;
pub mod vm;
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
pub use mulmsgsig::{Signature as MultiSignature, VerificationKey};

pub use self::zkos_types::{
    IOType, IOType::Coin, Input, InputData, Output, OutputData, Utxo, Witness,
};
