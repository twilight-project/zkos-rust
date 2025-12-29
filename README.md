# ZkOS Rust Workspace

[![Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue)](/LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)

**Status:** experimental ⛏️ – APIs may break before v1.0.

ZkOS is a collection of Rust crates implementing a zero-knowledge enabled transaction system for state management. It is based on the original ZKVM project from Slingshot but replaces the privacy layer with [QuisQuis](https://github.com/twilight-project/quisquis-rust). The project provides a comprehensive UTXO-based state management system supporting confidential coins, memos, and state variables.

## 🎯 Overview

ZkOS provides a privacy-preserving blockchain infrastructure with three core state types:

- **🪙 Coins**: Confidential digital assets with ElGamal encryption
- **📝 Memos**: Programmable data containers with time-bound access
- **🏗️ State**: Smart contract state with nonce-based versioning

The system maintains state through a UTXO (Unspent Transaction Output) model, ensuring immutability and enabling efficient verification of state transitions.

## 🏗️ Architecture

### Core Components

- **`zkvm`** – Zero-knowledge virtual machine and transaction verification logic
- **`transaction`** – Data structures and proofs for constructing confidential transactions
- **`utxo-in-memory`** – In-memory UTXO store with PostgreSQL persistence
- **`transactionapi`** – JSON-RPC API for blockchain interaction
- **`chain-oracle`** – Cosmos blockchain integration for block processing
- **`address`** – Address format utilities used across the network

### Supporting Crates

- **`merkle`**, **`mulmsgsig`**, **`rangeproof`**, **`readerwriter`**, **`starsig`** – Cryptographic primitives

## 🚀 Quick Start

### Prerequisites

- Rust 1.70 or newer
- PostgreSQL (for UTXO persistence)
- Access to a Cosmos-based blockchain (for chain oracle)

### Installation

```bash
# Clone the repository
git clone https://github.com/twilight-project/zkos-rust.git
cd zkos-rust

# Build the workspace
cargo build --release

# Run tests
cargo test --workspace
```

### Running the Local Testnet

The `utxo-in-memory` crate provides a complete testnet environment:

```bash
# Start the UTXO store and chain oracle
cargo run -p utxo-in-memory

# In another terminal, start the transaction API
cargo run -p transactionapi
```

## 📊 State Management

### UTXO Model

ZkOS uses a UTXO (Unspent Transaction Output) model for state management:

```rust
use zkvm::zkos_types::{Utxo, Output, IOType};

// UTXO represents an unspent transaction output
let utxo = Utxo::new(txid, output_index);

// Outputs can be of three types
let coin_output = Output::coin(OutputData::Coin(coin_data));
let memo_output = Output::memo(OutputData::Memo(memo_data));
let state_output = Output::state(OutputData::State(state_data));
```

### State Types

#### 🪙 Coins
Confidential digital assets with ElGamal encryption:

```rust
use zkvm::zkos_types::OutputCoin;

let coin = OutputCoin::new(
    elgamal_commitment,  // Encrypted value
    owner_address        // Owner's address
);
```

#### 📝 Memos
Programmable data containers with time-bound access:

```rust
use zkvm::zkos_types::OutputMemo;

let memo = OutputMemo::new(
    script_address,      // Script that can access this memo
    owner_address,       // Owner's address
    commitment,          // Pedersen commitment to value
    data,               // Optional memo data
    timebounds          // Time restrictions
);
```

#### 🏗️ State
Smart contract state with nonce-based versioning:

```rust
use zkvm::zkos_types::OutputState;

let state = OutputState::new(
    nonce,              // State version number
    script_address,     // Contract script address
    owner_address,      // Owner's address
    commitment,         // Pedersen commitment to value
    state_variables,    // Contract state variables
    timebounds          // Time restrictions
);
```

## 🔐 Privacy Features

### Zero-Knowledge Proofs
- **Range Proofs**: Verify value bounds without revealing amounts
- **Same-Value Proofs**: Prove equality between different commitments
- **Shuffle Proofs**: Hide input/output relationships

### Confidential Transactions
- **ElGamal Encryption**: Confidential value commitments
- **Pedersen Commitments**: Binding value representations
- **Ristretto255**: Secure elliptic curve operations

## 🛠️ Development

### Building Individual Crates

```bash
# Build specific crate
cargo build -p zkvm
cargo build -p transaction
cargo build -p utxo-in-memory

# Run tests for specific crate
cargo test -p zkvm
cargo test -p transaction
```

### API Documentation

Generate comprehensive API documentation:

```bash
# Generate docs for all crates
cargo doc --workspace --no-deps

# Open documentation in browser
cargo doc --open
```

### Code Quality

```bash
# Format code
cargo fmt

# Run clippy lints
cargo clippy --workspace

# Run tests with coverage
cargo test --workspace --verbose
```

## 📚 Examples

### Creating a Confidential Transfer

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
```

### Managing UTXO State

```rust
use utxo_in_memory::{init_utxo, UTXO_STORAGE};

// Initialize UTXO store
init_utxo();

// Access UTXO storage
let utxo_storage = UTXO_STORAGE.read().unwrap();

// Add new UTXO
utxo_storage.add(
    utxo_key,
    output_data,
    output_type as usize,
);

// Remove spent UTXO
utxo_storage.remove(utxo_key);
```

## 🔧 Configuration

### Environment Variables

```bash
# Database configuration
export DATABASE_URL="postgresql://user:password@localhost/zkos_db"

# Chain oracle configuration
export NYKS_BLOCK_SUBSCRIBER_URL="http://localhost:1317/"

# API server configuration
export RPC_SERVER_PORT=8000
export TELEMETRY_PORT=2500
```

### PostgreSQL Setup

```sql
-- Create database
CREATE DATABASE zkos_db;

-- Initialize tables (handled by utxo-in-memory crate)
```

## 📖 Documentation

- [API Documentation](https://docs.rs/transaction)
- [ZkVM Specification](zkvm/docs/zkvm-spec.md)
- [Transaction System](transaction/docs/spec.md)
- [UTXO Management](utxo-in-memory/README.md)

## 🤝 Contributing

We welcome contributions! Please ensure:

1. Code is formatted with `cargo fmt`
2. All tests pass with `cargo test`
3. Documentation is updated
4. New features include tests

### Development Workflow

```bash
# Fork and clone
git clone https://github.com/your-username/zkos-rust.git
cd zkos-rust

# Create feature branch
git checkout -b feature/your-feature

# Make changes and test
cargo test --workspace
cargo fmt
cargo clippy

# Commit and push
git commit -m "Add your feature"
git push origin feature/your-feature
```

## 📄 License & Attribution

This project is released under the **Apache License, Version 2.0**.  
See the full text in [`LICENSE`](./LICENSE).

Originally developed in the **Slingshot** project by the Stellar Development Foundation  
(*archived June 6 2024*) and subsequently adapted and extended by **Twilight Project Contributors** (© 2025).


By submitting a pull request you certify that you have the right to contribute the code and agree to 
license your work under Apache-2.0.

**Contributions welcome!** 🎉

## 🔗 References

- [ZkVM Whitepaper](https://github.com/stellar/slingshot/blob/main/zkvm/docs/zkvm-design.md)
- [QuisQuis Protocol](https://github.com/twilight-project/quisquis-rust)
- [Bulletproofs Paper](https://crypto.stanford.edu/bulletproofs/)
- [Ristretto255](https://ristretto.group/)

## 🚧 Status

This project is in **experimental** status. APIs may change before v1.0. The system is designed for:

- Research and experimentation
- Testnet deployment
- Privacy-preserving blockchain applications
- Zero-knowledge proof development

---