// -*- mode: rust; -*-
//
// This file is part of curve25519-dalek.
// Copyright (c) 2016-2019 Isis Lovecruft, Henry de Valence
// See LICENSE for licensing information.
//
// Authors:
// - Isis Agora Lovecruft <isis@patternsinthevoid.net>
// - Henry de Valence <hdevalence@hdevalence.ca>

//! Various constants, such as the Ristretto and Ed25519 basepoints.
//!
//! Most of the constants are given with
//! `LONG_DESCRIPTIVE_UPPER_CASE_NAMES`, but they can be brought into
//! scope using a `let` binding:
//!
//! ```
//! use curve25519_dalek_ng::constants;
//! use curve25519_dalek_ng::traits::IsIdentity;
//!
//! let B = &constants::RISTRETTO_BASEPOINT_TABLE;
//! let l = &constants::BASEPOINT_ORDER;
//!
//! let A = l * B;
//! assert!(A.is_identity());
//! ```

#![allow(non_snake_case)]

// Gas charged per byte of the transaction.      |
pub const GAS_PER_BYTE : u64 = 10       
//Maximum gas per transaction.
pub const MAX_GAS_PER_TX :u64  =           
//Maximum number of inputs. 
pub const MAX_INPUTS`                | `uint64` | `?`   |                     |
//Maximum number of outputs. 
pub const MAX_OUTPUTS`               | `uint64` | `?`   |                    |
pub const  MAX_PREDICATE_LENGTH`      | `uint64` |       | Maximum length of predicate, in instructions. |
pub const MAX_PREDICATE_DATA_LENGTH` | `uint64` |       | Maximum length of predicate data, in bytes.   |
pub const MAX_SCRIPT_LENGTH`         | `uint64` |       | Maximum length of script, in instructions.    |
pub const  `MAX_SCRIPT_DATA_LENGTH`    | `uint64` |       | Maximum length of script data, in bytes.      |
pub const  `MAX_STATIC_CONTRACTS`      | `uint64` | `255` | Maximum number of static contracts.           |
pub const  `MAX_WITNESSES`             | `uint64` | `16`  | Maximum number of witnesses.       