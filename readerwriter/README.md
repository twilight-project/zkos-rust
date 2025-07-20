# zkos-readerwriter

Binary reader/writer traits and utilities for serialization and cryptographic transcripts.

[![Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue)](/LICENSE)
<!-- CI badge (uncomment when workflow exists) -->
<!-- [![CI](https://github.com/twilight-project/zkos-rust/actions/workflows/ci.yml/badge.svg)](...) -->

> **Origin:** Portions adapted from the  
> [`stellar/slingshot`](https://github.com/stellar/slingshot/tree/main/merkle) project (Apache-2.0).

**Status:** experimental ⛏️ – APIs may break before v1.0.

---

## Installation

```toml
[dependencies]
readerwriter = { path = "../readerwriter" }   # inside the zkos-rust workspace
```

When the crate is published on crates.io you’ll be able to use:

```toml
zkos-readerwriter = "0.1"
```

---

## About

This crate provides generic traits and implementations for reading and writing binary data, with optional support for `bytes` and `merlin` transcripts.  
It is based on code from the [Slingshot project by Stellar](https://github.com/stellar/slingshot/tree/main/readerwriter) and extended for **ZkOS**.

## Features

- Generic `Reader` and `Writer` traits
- Error types for robust error handling
- Optional adapters for `bytes` and `merlin`
- `Codable` / `Encodable` / `Decodable` helper traits for custom types


## Quick example

```rust
use readerwriter::{ReaderExt, WriterExt};

let mut buf = Vec::new();
buf.write_u32(b"mylabel", 42).unwrap();

let value = (&buf[..]).read_u32(b"mylabel").unwrap();
assert_eq!(value, 42);
```

## Minimum Supported Rust Version

Rust **1.70** or newer.

---

## License & Attribution

Licensed under [`Apache-2.0`](../../LICENSE).  
Portions derived from Stellar’s **Slingshot** project; adapted © 2025 by Twilight Project Contributors.


