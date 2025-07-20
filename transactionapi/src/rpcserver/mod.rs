//! RPC server implementation for ZkOS Transaction API.
//!
//! This module provides the server-side components for handling JSON-RPC requests
//! and responses. It includes the main server implementation, service definitions,
//! thread pool management, and shared types.
//!
//! ## Modules
//!
//! - `server`: Main server implementation
//! - `service`: Service definitions for JSON-RPC methods
//! - `threadpool`: Thread pool management
//! - `types`: Shared types for RPC operations
//!
//! ## Modules
//!
//! - `server`: Main server implementation
//!
#![allow(dead_code)]
#![allow(unused_variables)]
/// RPC server implementation for ZkOS Transaction API.
mod server;
/// Service definitions for JSON-RPC methods
mod service;
/// Thread pool management
mod threadpool;
/// Shared types for RPC operations
mod types;

/// Re-export server and types modules
pub use self::server::*;
pub use self::types::MintOrBurnTx;
