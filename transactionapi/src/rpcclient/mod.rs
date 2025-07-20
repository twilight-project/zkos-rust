//! JSON-RPC client implementation for ZkOS transaction API.
//!
//! This module provides the client-side components for interacting with the ZkOS
//! blockchain via JSON-RPC 2.0 protocol. It includes request/response handling,
//! method definitions, and utility functions for building and sending RPC requests.
//!
//! ## Modules
//!
//! - `id`: JSON-RPC request identifier management
//! - `method`: RPC method definitions and parameter structures
//! - `txrequest`: Transaction request building and handling
//! - `utils`: Utility functions for UUID generation and other helpers

pub mod id;
pub mod method;
pub mod txrequest;
pub mod utils;
