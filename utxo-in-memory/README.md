# UTXO In-Memory Store

[![Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue)](/LICENSE)

**Status:** experimental ⛏️ – APIs may break before v1.0.

> ZkOS UTXO state management system with in-memory storage and PostgreSQL persistence.

## 🎯 Overview

The `utxo-in-memory` crate provides a comprehensive UTXO (Unspent Transaction Output) management system for ZkOS. It maintains the state of three types of outputs:

- **🪙 Coins**: Confidential digital assets with ElGamal encryption
- **📝 Memos**: Programmable data containers with time-bound access  
- **🏗️ State**: Smart contract state with nonce-based versioning

The system combines high-performance in-memory storage with PostgreSQL persistence for reliability and scalability.

## 🏗️ Architecture

### Core Components

- **`LocalStorage<T>`**: In-memory UTXO store with partitioned storage
- **`AddressUtxoIDStorage`**: Address-to-UTXO mapping for efficient queries
- **`SnapShot`**: Snapshot system for state recovery and persistence
- **`BlockProcessing`**: Block processing pipeline for state updates
- **`ChainOracle`**: Integration with Cosmos-based blockchains

### Storage Model

```
UTXO Storage (Partitioned by Type)
├── Coin UTXOs (Type 0)
├── Memo UTXOs (Type 1)  
└── State UTXOs (Type 2)

Address Mapping
├── Address → UTXO IDs (Coin)
├── Address → UTXO IDs (Memo)
└── Address → UTXO IDs (State)
```

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+
- PostgreSQL database
- Access to a Cosmos-based blockchain

### Installation

```bash
# Add to Cargo.toml
[dependencies]
utxo-in-memory = { path = "../utxo-in-memory" }
```

### Basic Usage

```rust
use utxo_in_memory::{init_utxo, UTXO_STORAGE, ADDRESS_TO_UTXO};
use zkvm::zkos_types::{Output, IOType};

// Initialize the UTXO store
init_utxo();

// Access UTXO storage
let utxo_storage = UTXO_STORAGE.read().unwrap();

// Add a new UTXO
let utxo_key = bincode::serialize(&utxo).unwrap();
let output_data = bincode::serialize(&output).unwrap();
utxo_storage.add(utxo_key, output_data, IOType::Coin as usize);

// Remove a spent UTXO
utxo_storage.remove(&utxo_key);

// Query UTXOs by address
let address_utxo_storage = ADDRESS_TO_UTXO.lock().unwrap();
let coin_utxos = address_utxo_storage.get_utxos_by_address(
    IOType::Coin,
    &address
);
```

## 📊 State Management

### UTXO Types

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

### Block Processing

The system processes blocks to update UTXO state:

```rust
use utxo_in_memory::blockoperations::blockprocessing::process_block_for_utxo_insert;

// Process a block to update UTXO state
let result = process_block_for_utxo_insert(block);

// Handle processing results
if result.success_tx.len() > 0 {
    // Take snapshot after successful processing
    save_snapshot();
}
```

## 🔧 Configuration

### Environment Variables

```bash
# Database configuration
export DATABASE_URL="postgresql://user:password@localhost/zkos_db"

# Chain oracle configuration  
export NYKS_BLOCK_SUBSCRIBER_URL="http://localhost:1317/"

# Block height file
export BLOCK_HEIGHT_FILE="height.txt"
```

### PostgreSQL Setup

```sql
-- Create database
CREATE DATABASE zkos_db;

-- Tables are created automatically by the crate
```

## 📈 Monitoring

### Prometheus Metrics

The system exposes Prometheus metrics for monitoring:

- `utxo_coin_count`: Number of coin UTXOs
- `utxo_memo_count`: Number of memo UTXOs  
- `utxo_state_count`: Number of state UTXOs

### Telemetry

```rust
use utxo_in_memory::{
    UTXO_COIN_TELEMETRY_COUNTER,
    UTXO_MEMO_TELEMETRY_COUNTER,
    UTXO_STATE_TELEMETRY_COUNTER
};

// Update metrics
UTXO_COIN_TELEMETRY_COUNTER.set(total_coin_type_utxos() as f64);
UTXO_MEMO_TELEMETRY_COUNTER.set(total_memo_type_utxos() as f64);
UTXO_STATE_TELEMETRY_COUNTER.set(total_state_type_utxos() as f64);
```

## 🔄 State Persistence

### Snapshots

The system supports state snapshots for recovery:

```rust
use utxo_in_memory::{UTXO_STORAGE, save_snapshot};

// Take a snapshot
save_snapshot();

// Load from snapshot
let mut utxo_storage = UTXO_STORAGE.write().unwrap();
utxo_storage.load_from_snapshot();
```

### PostgreSQL Persistence

```rust
use utxo_in_memory::{init_psql, UTXO_STORAGE};

// Initialize PostgreSQL connection
init_psql();

// Load from PostgreSQL
let mut utxo_storage = UTXO_STORAGE.write().unwrap();
utxo_storage.load_from_snapshot_from_psql();
```

## 🧪 Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test module
cargo test test::utxostore_tests

# Run with verbose output
cargo test -- --nocapture
```

### Test Utilities

```rust
use utxo_in_memory::types::test;

// Initialize test environment
test::init_utxo_for_test("test_path");

// Clean up after tests
test::uninstall_delete_db_utxo_for_test("test_path");
```

## 📚 API Reference

### Core Types

- **`LocalStorage<T>`**: Main UTXO storage with partitioning
- **`AddressUtxoIDStorage`**: Address-to-UTXO mapping
- **`SnapShot`**: Snapshot management
- **`ZkosBlock`**: Block representation for processing
- **`UTXO`**: Individual UTXO representation

### Key Functions

- **`init_utxo()`**: Initialize the UTXO store
- **`zk_oracle_subscriber()`**: Start blockchain subscription
- **`save_snapshot()`**: Create state snapshot
- **`process_block_for_utxo_insert()`**: Process blockchain blocks

## 🔍 Performance

### Optimizations

- **Partitioned Storage**: UTXOs stored by type for efficient access
- **In-Memory Caching**: Fast access to frequently used data
- **Batch Operations**: Efficient bulk updates
- **Thread Pool**: Concurrent block processing

### Benchmarks

```bash
# Performance testing
cargo bench

# Memory usage monitoring
cargo test --release -- --nocapture
```

## 🛠️ Development

### Building

```bash
# Build in release mode
cargo build --release

# Build with optimizations
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

### Code Quality

```bash
# Format code
cargo fmt

# Run lints
cargo clippy

# Check documentation
cargo doc --no-deps
```

## 🔗 Integration

### With ZkVM

```rust
use zkvm::zkos_types::{Input, Output, Utxo};
use utxo_in_memory::UTXO_STORAGE;

// Convert ZkVM types to UTXO storage
let utxo = Utxo::new(txid, output_index);
let utxo_key = bincode::serialize(&utxo).unwrap();
let output_data = bincode::serialize(&output).unwrap();
```

### With Transaction API

```rust
use transactionapi::api_server;

// Start API server
api_server::start_server();
```

## 📄 License

Licensed under [`Apache-2.0`](../../LICENSE).

## 🤝 Contributing

Contributions welcome! Please ensure:

1. Code is formatted with `cargo fmt`
2. All tests pass
3. Documentation is updated
4. New features include tests

---

**Part of the ZkOS Rust Workspace** 🚀 