# ZkOS Quick Start Guide

[![Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue)](/LICENSE)

**Status:** experimental ⛏️ – APIs may break before v1.0.

> Get started with ZkOS state management in minutes.

## 🚀 Prerequisites

### System Requirements

- **Rust**: 1.70 or newer
- **PostgreSQL**: 12 or newer
- **Memory**: 4GB+ RAM recommended
- **Storage**: 10GB+ free space

### Install Dependencies

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install PostgreSQL
# Ubuntu/Debian
sudo apt update
sudo apt install postgresql postgresql-contrib

# macOS
brew install postgresql

# Start PostgreSQL
sudo systemctl start postgresql  # Linux
brew services start postgresql   # macOS
```

## 📦 Installation

### Clone the Repository

```bash
git clone https://github.com/twilight-project/zkos-rust.git
cd zkos-rust
```

### Build the Workspace

```bash
# Build all crates
cargo build --release

# Verify installation
cargo test --workspace
```

## 🗄️ Database Setup

### Create Database

```bash
# Connect to PostgreSQL
sudo -u postgres psql

# Create database and user
CREATE DATABASE zkos_db;
CREATE USER zkos_user WITH PASSWORD 'your_password';
GRANT ALL PRIVILEGES ON DATABASE zkos_db TO zkos_user;
\q
```

### Configure Environment

```bash
# Set environment variables
export DATABASE_URL="postgresql://zkos_user:your_password@localhost/zkos_db"
export NYKS_BLOCK_SUBSCRIBER_URL="http://localhost:1317/"
export RPC_SERVER_PORT=8000
export TELEMETRY_PORT=2500
```

## 🏃‍♂️ Running ZkOS

### Start UTXO Store

```bash
# Start the UTXO store and chain oracle
cargo run -p utxo-in-memory

# You should see output like:
# starting utxo init
# finished loading from psql
# UTXO Memo Telemetry Counter Value: 0
# UTXO coin Telemetry Counter Value: 0
# UTXO state Telemetry Counter Value: 0
# finishing utxo init
# started zk subsciber
```

### Start Transaction API (Optional)

```bash
# In another terminal
cargo run -p transactionapi

# API will be available at:
# - RPC Server: http://localhost:8000
# - Telemetry: http://localhost:2500/metrics
```

## 💻 Basic Usage

### Working with Coins

```rust
use zkvm::zkos_types::{OutputCoin, Output, OutputData, IOType};
use quisquislib::elgamal::ElGamalCommitment;
use utxo_in_memory::{init_utxo, UTXO_STORAGE};

// Initialize UTXO store
init_utxo();

// Create a confidential coin
let coin = OutputCoin::new(
    ElGamalCommitment::default(), // Encrypted value
    "owner_address_here".to_string()
);

// Create output
let output = Output::coin(OutputData::Coin(coin));

// Add to UTXO store
let utxo_storage = UTXO_STORAGE.write().unwrap();
let utxo_key = bincode::serialize(&utxo).unwrap();
let output_data = bincode::serialize(&output).unwrap();
utxo_storage.add(utxo_key, output_data, IOType::Coin as usize);
```

### Working with Memos

```rust
use zkvm::zkos_types::{OutputMemo, Output, OutputData, IOType};
use zkvm::constraints::Commitment;

// Create a programmable memo
let memo = OutputMemo::new(
    "script_address".to_string(),     // Script that can access
    "owner_address".to_string(),      // Owner's address
    Commitment::unblinded(100u64),    // Pedersen commitment
    Some(vec!["memo_data".into()]),   // Optional data
    0,                                // Time restrictions
);

// Create output
let output = Output::memo(OutputData::Memo(memo));

// Add to UTXO store
let utxo_storage = UTXO_STORAGE.write().unwrap();
let utxo_key = bincode::serialize(&utxo).unwrap();
let output_data = bincode::serialize(&output).unwrap();
utxo_storage.add(utxo_key, output_data, IOType::Memo as usize);
```

### Working with State

```rust
use zkvm::zkos_types::{OutputState, Output, OutputData, IOType};
use zkvm::constraints::Commitment;

// Create smart contract state
let state = OutputState::new(
    0,                                // Nonce (version)
    "contract_script".to_string(),    // Contract script address
    "owner_address".to_string(),      // Owner's address
    Commitment::unblinded(0u64),      // Pedersen commitment
    Some(vec!["state_var".into()]),   // State variables
    0,                                // Time restrictions
);

// Create output
let output = Output::state(OutputData::State(state));

// Add to UTXO store
let utxo_storage = UTXO_STORAGE.write().unwrap();
let utxo_key = bincode::serialize(&utxo).unwrap();
let output_data = bincode::serialize(&output).unwrap();
utxo_storage.add(utxo_key, output_data, IOType::State as usize);
```

## 🔍 Querying State

### Query by Address

```rust
use utxo_in_memory::{ADDRESS_TO_UTXO, IOType};

// Query UTXOs by address
let address_storage = ADDRESS_TO_UTXO.lock().unwrap();

// Get coin UTXOs
let coin_utxos = address_storage.get_utxos_by_address(
    IOType::Coin,
    &"owner_address".to_string()
);

// Get memo UTXOs
let memo_utxos = address_storage.get_utxos_by_address(
    IOType::Memo,
    &"owner_address".to_string()
);

// Get state UTXOs
let state_utxos = address_storage.get_utxos_by_address(
    IOType::State,
    &"owner_address".to_string()
);
```

### Query by Type

```rust
use utxo_in_memory::{UTXO_STORAGE, IOType};

// Access UTXO storage
let utxo_storage = UTXO_STORAGE.read().unwrap();

// Get all coin UTXOs
let coin_utxos = utxo_storage.data.get(&(IOType::Coin as usize)).unwrap();

// Get all memo UTXOs
let memo_utxos = utxo_storage.data.get(&(IOType::Memo as usize)).unwrap();

// Get all state UTXOs
let state_utxos = utxo_storage.data.get(&(IOType::State as usize)).unwrap();
```

## 🔄 State Management

### Taking Snapshots

```rust
use utxo_in_memory::{UTXO_STORAGE, save_snapshot};

// Take a snapshot of current state
save_snapshot();

// Load from snapshot
let mut utxo_storage = UTXO_STORAGE.write().unwrap();
utxo_storage.load_from_snapshot();
```

### Block Processing

```rust
use utxo_in_memory::blockoperations::blockprocessing::process_block_for_utxo_insert;
use chain_oracle::Block;

// Process a block to update UTXO state
let block = Block::default(); // Your block data
let result = process_block_for_utxo_insert(block);

// Handle processing results
if result.success_tx.len() > 0 {
    println!("Successfully processed {} transactions", result.success_tx.len());
}
```

## 🧪 Testing

### Run Tests

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p utxo-in-memory
cargo test -p zkvm
cargo test -p transaction

# Run with verbose output
cargo test --workspace -- --nocapture
```

### Test Utilities

```rust
use utxo_in_memory::types::test;

// Initialize test environment
test::init_utxo_for_test("test_path");

// Your test code here...

// Clean up after tests
test::uninstall_delete_db_utxo_for_test("test_path");
```

## 📊 Monitoring

### Prometheus Metrics

```bash
# View metrics (if API server is running)
curl http://localhost:2500/metrics

# Expected metrics:
# utxo_coin_count 0
# utxo_memo_count 0
# utxo_state_count 0
```

### Logging

```rust
// Enable debug logging
env_logger::init();

// Log UTXO operations
println!("UTXO count: {}", utxo_storage.data.len());
```

## 🔧 Configuration

### Environment Variables

```bash
# Database
export DATABASE_URL="postgresql://user:password@localhost/zkos_db"

# Chain Oracle
export NYKS_BLOCK_SUBSCRIBER_URL="http://localhost:1317/"

# API Server
export RPC_SERVER_PORT=8000
export TELEMETRY_PORT=2500

# Block Height
export BLOCK_HEIGHT_FILE="height.txt"
```

### Configuration File

Create a `.env` file:

```env
DATABASE_URL=postgresql://zkos_user:your_password@localhost/zkos_db
NYKS_BLOCK_SUBSCRIBER_URL=http://localhost:1317/
RPC_SERVER_PORT=8000
TELEMETRY_PORT=2500
BLOCK_HEIGHT_FILE=height.txt
```

## 🚨 Troubleshooting

### Common Issues

#### Database Connection Failed
```bash
# Check PostgreSQL status
sudo systemctl status postgresql

# Check connection
psql -h localhost -U zkos_user -d zkos_db
```

#### Build Errors
```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

#### UTXO Store Not Starting
```bash
# Check environment variables
echo $DATABASE_URL

# Check PostgreSQL tables
psql -h localhost -U zkos_user -d zkos_db -c "\dt"
```

### Debug Mode

```bash
# Run with debug output
RUST_LOG=debug cargo run -p utxo-in-memory

# Run specific crate with debug
RUST_LOG=debug cargo run -p transactionapi
```

## 📚 Next Steps

### Learn More

- [Architecture Documentation](ARCHITECTURE.md)
- [API Documentation](https://docs.rs/transaction)
- [ZkVM Specification](zkvm/docs/zkvm-spec.md)

### Examples

- [Transaction Examples](transaction/examples/)
- [UTXO Management](utxo-in-memory/examples/)
- [API Usage](transactionapi/examples/)

### Development

- [Contributing Guidelines](CONTRIBUTING.md)
- [Code Style](STYLE.md)
- [Testing Guide](TESTING.md)

## 🤝 Getting Help

### Resources

- **Documentation**: [docs.rs/transaction](https://docs.rs/transaction)
- **Issues**: [GitHub Issues](https://github.com/twilight-project/zkos-rust/issues)
- **Discussions**: [GitHub Discussions](https://github.com/twilight-project/zkos-rust/discussions)

### Community

- **Discord**: [Twilight Project Discord](https://discord.gg/twilight)
- **Twitter**: [@TwilightProject](https://twitter.com/TwilightProject)
- **Blog**: [Twilight Project Blog](https://blog.twilightproject.com)

---

**Ready to build with ZkOS? Start with the examples and join our community!** 🚀 