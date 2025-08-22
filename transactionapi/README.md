# TransactionAPI

[![Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue)](/LICENSE)

**Status:** experimental ⛏️ – APIs may break before v1.0.

> JSON-RPC API for ZkOS blockchain interaction and UTXO management.

## Overview

TransactionAPI provides a comprehensive JSON-RPC 2.0 interface for the ZkOS blockchain, enabling transaction submission, UTXO queries, and blockchain state management.

## Quick Start

```bash
# Build and run
cargo build --release
cargo run --bin api_server

# Server starts on http://127.0.0.1:3030
```

# Documentation
API documentation is available at: https://docs.twilight.rest/#zkos-rpc-api

## Core Features

### Transaction Operations
- **txCommit** - Submit and verify transactions
- **txStatus** - Query transaction status (placeholder)

### UTXO Queries
- **getUtxos** - Coin UTXOs by address
- **getMemoUtxos** - Memo UTXOs by address  
- **getStateUtxos** - State UTXOs by address
- **get_utxos_id** - UTXO ID by address and type
- **get_utxos_detail** - Detailed UTXO information

### Output Operations
- **getOutput** - Coin output by UTXO key
- **getMemoOutput** - Memo output by UTXO key
- **getStateOutput** - State output by UTXO key
- **allOutputs** - All coin outputs

### Database Queries
- **getUtxosFromDB** - UTXOs from PostgreSQL with pagination
- **allCoinUtxos** - All coin UTXOs
- **allMemoUtxos** - All memo UTXOs
- **allStateUtxos** - All state UTXOs

### Administrative
- **TestCommand** - Snapshot management and storage operations

## Example Usage

```json
{
  "jsonrpc": "2.0",
  "id": "1",
  "method": "txCommit",
  "params": ["0x1234...", "twilight_address"]
}
```

## Architecture

- **RPC Server**: JSON-RPC 2.0 over HTTP
- **Thread Pool**: Concurrent request processing
- **Storage**: In-memory UTXO storage with PostgreSQL persistence
- **Metrics**: Prometheus integration for monitoring

## License

Apache 2.0 - see [LICENSE](../../LICENSE) for details.

# zkos-transactionapi

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue)](../../LICENSE)
<!-- CI badge (uncomment once configured) -->
<!-- [![CI](https://github.com/twilight-project/zkos-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/twilight-project/zkos-rust/actions/workflows/ci.yml) -->

**Status:** experimental ⛏️ – APIs and endpoints may evolve before v1.0.

> A JSON-RPC 2.0 server implementation for the ZkOS blockchain, providing transaction submission, UTXO queries, and state management.

---

## Installation

Add the crate to your workspace (path dependency):

```toml
[dependencies]
transactionapi = { path = "../transactionapi" }
```

When published on crates.io:

```toml
transactionapi = "0.1"
```

---

## Quick Start

Build and run the RPC server:

```bash
cargo build --release
cargo run --bin api_server
```

By default, the server listens on `http://127.0.0.1:3030`.

---

## API Documentation

Full API reference is available at:  
https://docs.twilight.rest/#zkos-rpc-api

---

## Core Features

- **JSON-RPC 2.0** server with request validation and error handling  
- **Transaction Operations**:  
  - `txCommit` – submit and verify transactions  
  - `txStatus` – query transaction confirmation status  
- **UTXO Queries**:  
  - `getUtxos` – list coin UTXOs by address  
  - `getMemoUtxos` – list memo UTXOs by address  
  - `getStateUtxos` – list state UTXOs by address  
- **Output Retrieval**:  
  - `getOutput`, `getMemoOutput`, `getStateOutput` – fetch individual outputs by key  
  - `allOutputs` – list all coin outputs  
- **Database Integration**:  
  - PostgreSQL persistence with pagination support  
  - In-memory caching for low-latency queries  
- **Concurrency & Monitoring**:  
  - Thread pool for parallel request handling  
  - Prometheus metrics endpoint for performance insights  

---

## Example Request

```json
{
  "jsonrpc": "2.0",
  "id": "1",
  "method": "txCommit",
  "params": ["0x1234...", "twilight_address"]
}
```

---

## Architecture Overview

- **RPC Server**: JSON-RPC 2.0 over HTTP  
- **Thread Pool**: handles concurrent client connections  
- **Storage**: hybrid in-memory and PostgreSQL-backed UTXO store  
- **Metrics**: Prometheus integration for request and performance metrics  

---

## License

Licensed under the **Apache License, Version 2.0**.  
See [LICENSE](../../LICENSE) for details.

---