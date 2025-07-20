[![Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue)](/LICENSE)
<!-- CI badge example; uncomment when you have CI -->
<!-- [![CI](https://github.com/twilight-project/zkos-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/twilight-project/zkos-rust/actions/workflows/ci.yml) -->

**Status:** experimental ⛏️ – APIs may break before v1.0.

# ZkOS Transaction: Confidential Blockchain Transactions

A comprehensive transaction system for the ZkOS based blockchain, providing confidential transfers, smart contract execution, and message-based operations with zero-knowledge proofs.

## Overview

The ZkOS transaction system enables privacy-preserving blockchain operations through advanced cryptographic techniques including Bulletproofs, QuisQuis protocols, and zero-knowledge proofs. It supports multiple transaction types designed for different use cases while maintaining strong privacy guarantees.

## Features

### �� Confidential Transfers
- **Dark Transactions**: Fully confidential transfers with delta/epsilon proofs
- **QuisQuis Transactions**: Shuffled transfers with enhanced privacy
- **Lit to Lit**: Transparent transfers between public addresses
- **Range Proofs**: Bulletproofs-based value verification

### ⚡ Smart Contract Execution
- **Script Transactions**: Execute programs with R1CS proofs
- **Contract Deployment**: Deploy new contracts to the blockchain
- **State Management**: Handle contract state and witnesses
- **VM Integration**: ZkVM for secure program execution

### �� Message Operations
- **Burn Messages**: Destroy assets with reveal proofs
- **Custom Messages**: Extensible message system
- **Signature Verification**: Cryptographic authorization

### 🛡️ Privacy & Security
- **Zero-Knowledge Proofs**: Transaction privacy without revealing amounts
- **Shuffle Proofs**: Input/output privacy through permutation
- **ElGamal Encryption**: Confidential value commitments
- **Ristretto255**: Secure elliptic curve operations

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
transaction = { git = "https://github.com/twilight-project/zkos-rust", package = "transaction" }
quisquis-rust = { git = "https://github.com/twilight-project/quisquis-rust.git", tag = "Testnet-v1.0.0" }
zkschnorr = { git = "https://github.com/twilight-project/zk-schnorr.git", tag = "Testnet-v1.0.0" }
```

### Basic Usage

#### Transfer Transaction

```rust
use transaction::{Transaction, TransactionType, TransactionData, TransferTransaction};
use zkvm::zkos_types::{Input, Output};
use quisquislib::ristretto::RistrettoSecretKey;

// Create inputs and outputs
let inputs = vec![/* input data */];
let outputs = vec![/* output data */];

// Create a QuisQuis transfer transaction
let transfer_tx = TransferTransaction::create_quisquis_transaction(
    &inputs,
    &value_vector,
    &account_vector,
    &sender_updated_balance,
    &receiver_value_balance,
    &sender_sk,
    senders_count,
    receivers_count,
    anonymity_account_diff,
    witness_comm_scalar,
    fee,
).unwrap();

let transaction = Transaction::new(
    TransactionType::Transfer,
    TransactionData::TransactionTransfer(transfer_tx),
);

// Verify the transaction
assert!(transaction.verify().is_ok());
```

#### Script Transaction

```rust
use transaction::{ScriptTransaction, Transaction, TransactionType, TransactionData};
use zkvm::{Program, zkos_types::{Input, Output}};
use quisquislib::ristretto::RistrettoSecretKey;

// Create a script transaction
let script_tx = ScriptTransaction::create_script_transaction(
    &secret_keys,
    program,
    call_proof,
    &inputs,
    &outputs,
    tx_data,
    contract_deploy_flag,
    fee,
).unwrap();

let transaction = Transaction::new(
    TransactionType::Script,
    TransactionData::TransactionScript(script_tx),
);
```

#### Message Transaction

```rust
use transaction::{Message, Transaction, TransactionType, TransactionData};
use zkvm::zkos_types::{Input, MessageType};
use curve25519_dalek::scalar::Scalar;
use quisquislib::ristretto::RistrettoSecretKey;

// Create a burn message
let burn_message = Message::create_burn_message(
    input,
    amount,
    encrypt_scalar,
    secret_key,
    initial_address,
);

let transaction = Transaction::new(
    TransactionType::Message,
    TransactionData::Message(burn_message),
);
```

## API Reference

### Core Types

- **`Transaction`**: Main transaction container
- **`TransactionType`**: Enumeration of transaction types
- **`TransactionData`**: Type-safe transaction payloads

### Transfer Transactions

- **`TransferTransaction`**: Confidential transfer implementation
- **`DarkTxProof`**: Zero-knowledge proofs for dark transactions
- **`ShuffleTxProof`**: Shuffle proofs for QuisQuis transactions

### Script Transactions

- **`ScriptTransaction`**: Smart contract execution
- **`vm_run`**: Virtual machine execution utilities

### Message Transactions

- **`Message`**: Message-based operations
- **`RevealProof`**: Proofs for revealing hidden values

## Architecture

### Transaction Flow

1. **Input Validation**: Verify transaction structure and constraints
2. **Proof Generation**: Create zero-knowledge proofs for privacy
3. **State Updates**: Apply transaction effects to blockchain state
4. **Verification**: Validate all cryptographic proofs and signatures

### Privacy Mechanisms

- **Delta/Epsilon Accounts**: Confidential balance representation
- **Shuffle Proofs**: Input/output permutation for privacy
- **Range Proofs**: Value bounds verification without disclosure
- **ElGamal Encryption**: Confidential value commitments

### Security Features

- **Zero-Knowledge Proofs**: Transaction privacy
- **Cryptographic Signatures**: Authorization verification
- **Range Verification**: Prevent value overflow
- **State Consistency**: Ensure valid state transitions

## Performance

- **Proof Generation**: Optimized for batch processing
- **Verification**: Constant-time verification algorithms
- **Memory Usage**: Efficient data structures for large transactions
- **Network Overhead**: Compact proof representations

## Minimum Supported Rust Version

Rust **1.70** or newer.

## Documentation

- [API Documentation](https://docs.rs/transaction)
- [Specification](docs/spec.md)
- [Examples](examples/)

## Contributing

Contributions are welcome! Please ensure all code is properly documented and tested.

## License & Attribution

Licensed under [`Apache-2.0`](../../LICENSE).

## References

- [Bulletproofs Paper](https://crypto.stanford.edu/bulletproofs/)
- [QuisQuis Protocol](https://github.com/twilight-project/quisquis-rust)
- [Ristretto255](https://ristretto.group/)
- [ZkVM](https://github.com/twilight-project/zkos-rust/tree/main/zkvm)
