# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
- No pending changes.

## [0.1.2] – 2025-12-29
### Maintenance
- Updated internal dependency `quisquis-rust` to tag `Testnet-v1.0.1` in workspace crates (`address`, `transaction`, `transactionapi`, `zkvm`, `utxo-in-memory`).
- No public API or functional behavior changes are expected from this update.
- This release is a routine dependency maintenance update to keep the cryptographic stack in sync.

## [0.1.0] – 2025-07-15

### Security Warning ⚠️

- This is an **experimental testnet release** and is **not safe for production use**.
- It contains known timing-leak vulnerabilities inherited from upstream crates. Please see the [Security Policy](.github/SECURITY.md) for full details before using this software.

### Added


### Changed
- (none for initial release)

### Fixed

- **Linter Warnings**
  - All public items now have documentation, resolving previous linter warnings about missing doc comments.

---
<!-- No compare link: initial release -->