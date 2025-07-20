# Changelog

All notable changes to the `utxo-in-memory` crate are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [Unreleased]


## [0.1.0] – 2025-07-20

### Summary

Initial public release of the UTXO in-memory storage engine for ZkOS, supporting concurrent block processing, persistent snapshots, and cryptographic UTXO queries.

### Features

- In-memory UTXO storage partitioned by type: Coin, Memo, and State
- Address-to-UTXO ID mapping for efficient lookups
- LevelDB-based snapshot system for fast recovery
- PostgreSQL persistence with bulk operation support
- Custom thread pool with graceful shutdown
- Telemetry instrumentation and Prometheus metrics
- Block processing pipeline with chain oracle integration
- Zero-knowledge proof compatibility and cryptographic primitives
- Thread-safe architecture using `RwLock` and `Mutex`
- Full test coverage and documentation
- Internal specs: [`README.md`](./README.md), [`STORAGE_SPEC.md`](./STORAGE_SPEC.md)


<!-- Inital release -->