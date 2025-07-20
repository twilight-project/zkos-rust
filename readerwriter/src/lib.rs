// Copyright © 2019 Interstellar & Stellar Development Foundation
// Copyright © 2025 Twilight Project Contributors
// SPDX-License-Identifier: Apache-2.0
//
// Portions of this file are derived from the `readerwriter` crate in the
// `stellar/slingshot` project (Apache-2.0).

//! Binary reader/writer traits and utilities for serialization and cryptographic transcripts.
//!
//! This crate provides generic traits and implementations for reading and writing binary data,
//! with optional support for `bytes` and `merlin` transcripts.
//!
//! # Features
//! - Generic `Reader` and `Writer` traits
//! - Error types for robust error handling
//! - Optional support for `bytes` and `merlin`
//! - Codable/Encodable/Decodable traits for custom types
//!
//! # Example
//! ```
//! use readerwriter::{Writer, Reader};
//! let mut buf = Vec::new();
//! buf.write_u32(b"mylabel", 42).unwrap();
//! ```

#![allow(warnings)]
mod codable;
mod reader;
mod writer;

/// Traits and error types for encoding and decoding.
pub use codable::{Codable, Decodable, Encodable, ExactSizeEncodable};

/// Error type and trait for reading binary data.
pub use reader::{ReadError, Reader};

/// Error type and trait for writing binary data.
pub use writer::{WriteError, Writer};

/// Optional: Writer implementation for merlin::Transcript (if "merlin" feature enabled).
#[cfg(feature = "merlin")]
mod merlin_support;
#[cfg(feature = "merlin")]
pub use merlin_support::*;

/// Optional: Reader/Writer implementations for bytes crate (if "bytes" feature enabled).
#[cfg(feature = "bytes")]
mod bytes_support;
#[cfg(feature = "bytes")]
pub use bytes_support::*;
