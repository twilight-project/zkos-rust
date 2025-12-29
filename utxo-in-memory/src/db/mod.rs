//! ZkOS Database and Storage Module
//!
//! This module provides the core database and storage functionality for the ZkOS UTXO
//! state management system. It includes in-memory storage, address mapping, snapshot
//! management, and PostgreSQL persistence.
//!
//! ## Core Components
//!
//! - **`LocalStorage<T>`**: In-memory UTXO storage with partitioned access
//! - **`AddressUtxoIDStorage`**: Address-to-UTXO mapping for efficient queries
//! - **`SnapShot`**: Snapshot system for state recovery and persistence
//! - **`SnapRules`**: Configuration and rules for snapshot management
//!
//! ## Storage Architecture
//!
//! The storage system uses a multi-layered approach:
//!
//! 1. **In-Memory Storage**: High-performance partitioned hash maps
//! 2. **Address Mapping**: Fast address-to-UTXO lookups
//! 3. **Snapshot System**: State recovery and backup capabilities
//! 4. **PostgreSQL Persistence**: Reliable long-term storage
//!

#![allow(dead_code)]
#![allow(unused_variables)]
/// Address to UTXO mapping module
mod address_utxo_link;
/// Snap rules module for snapshot management
mod snap_rules;
/// Snapshot module for state recovery and persistence, and persistence
mod snapshot;
/// UTXO storage module for in-memory storage
pub mod utxostore;

pub use self::address_utxo_link::AddressUtxoIDStorage;
pub use self::snapshot::SnapShot;
/// Re-export key data types
pub use self::snapshot::*;
pub use self::utxostore::takesnapshotfrom_memory_to_postgresql_bulk;
pub use self::utxostore::KeyId;
pub use self::utxostore::LocalDBtrait;
pub use self::utxostore::LocalStorage;
pub use self::utxostore::SequenceNumber;
pub use self::utxostore::UtxokeyidOutput;
