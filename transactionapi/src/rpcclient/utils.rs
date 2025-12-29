//! Utility functions for JSON-RPC client implementation.
//!
//! This module provides helper functions for generating UUIDs and other
//! common utilities used in the JSON-RPC client implementation.
//!
//! ## Functions
//!
//! - `uuid_str`: Generate a random UUID string
//!
use getrandom::getrandom;

// use crate::prelude::*;

/// Produce a string containing a UUID.
///
/// Panics if random number generation fails.
pub fn uuid_str() -> String {
    let mut bytes = [0; 16];
    getrandom(&mut bytes).expect("RNG failure!");

    let uuid = uuid::Builder::from_bytes(bytes)
        .set_variant(uuid::Variant::RFC4122)
        .set_version(uuid::Version::Random)
        .build();

    uuid.to_string()
}
