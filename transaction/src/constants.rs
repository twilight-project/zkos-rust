// -*- mode: rust; -*-
//
// This file is part of ZkOS.
//
// See LICENSE for licensing information.
//
// Authors:
//
//

//! Various constants, such as the gas per byte and maximum number of inputs/outputs.
//!
//! Most of the constants are given with
//! `LONG_DESCRIPTIVE_UPPER_CASE_NAMES`, but they can be brought into
//! scope using a `let` binding:
//!
//! ```
//! ```

#![allow(non_snake_case)]

// Gas charged per byte of the transaction.
pub const GAS_PER_BYTE: u64 = 10;

// Maximum contract size, in bytes.
pub const CONTRACT_MAX_SIZE: u64 = 16 * 1024 * 1024;

/// Maximum number of inputs.
pub const MAX_INPUTS: u8 = 255;

/// Maximum number of outputs.
pub const MAX_OUTPUTS: u8 = 255;

/// Maximum number of witnesses.
pub const MAX_WITNESSES: u8 = 255;

/// Maximum gas per transaction.
pub const MAX_GAS_PER_TX: u64 = 1000000;

// TODO set max script length const
/// Maximum length of script, in instructions.
pub const MAX_SCRIPT_LENGTH: u64 = 1024 * 1024;

// TODO set max script length const
/// Maximum length of script data, in bytes.
pub const MAX_SCRIPT_DATA_LENGTH: u64 = 1024 * 1024;

/// Maximum number of static contracts.
pub const MAX_STATIC_CONTRACTS: u64 = 255;

/// Maximum length of predicate, in instructions.
pub const MAX_PREDICATE_LENGTH: u64 = 1024 * 1024;

// TODO set max predicate data length value
/// Maximum length of predicate data, in bytes.
pub const MAX_PREDICATE_DATA_LENGTH: u64 = 1024 * 1024;
