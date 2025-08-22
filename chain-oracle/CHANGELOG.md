# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-07-20

### Added

- **Initial public release of the `chain-oracle` crate.**
- Comprehensive crate-level documentation and module-level docs.
- Support for subscribing to new blocks from a Cosmos-based blockchain via a configurable endpoint.
- Thread pool implementation for concurrent block processing.
- Rich Rust structs for blocks, transactions, and related data, with Serde support for easy (de)serialization.
- Utility functions for making HTTP requests to Cosmos chain REST endpoints.
- Environment variable configuration for the chain endpoint (`NYKS_BLOCK_SUBSCRIBER_URL`).
- Example code for block subscription and block height retrieval.
- Public API re-exports for ergonomic use in downstream crates.
- Apache 2.0 license.

### Changed

- N/A (initial release)

### Fixed

- N/A (initial release)

---

<!-- No compare link: initial release -->

