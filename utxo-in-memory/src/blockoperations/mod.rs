//! ZkOS Block Processing Module
//!
//! This module provides block processing functionality for the ZkOS UTXO state
//! management system. It handles blockchain block parsing, UTXO extraction,
//! and state updates with high performance and reliability.
//!
//! ## Core Components
//!
//! - **`blockprocessing`**: Main block processing logic and UTXO state updates
//! - **`initialset`**: Initial UTXO set loading and genesis block processing
//! - **`genesis_sets.txt`**: Genesis block data for system initialization
//!
//! ## Block Processing Pipeline
//!
//! The block processing system follows this pipeline:
//!
//! 1. **Block Reception**: Receive blocks from chain oracle
//! 2. **Transaction Parsing**: Extract transactions from blocks
//! 3. **UTXO Extraction**: Identify UTXO additions and removals
//! 4. **State Updates**: Apply changes to in-memory storage
//! 5. **Persistence**: Sync changes to PostgreSQL
//! 6. **Snapshot Creation**: Create recovery points
//!
//! ## Transaction Types
//!
//! The system processes multiple transaction types:
//!
//! - **Transfer Transactions**: Confidential asset transfers
//! - **Script Transactions**: Smart contract execution
//! - **Message Transactions**: Data operations and messages
//!
//! ## UTXO Operations
//!
//! For each transaction, the system:
//!
//! - **Removes Input UTXOs**: Spent UTXOs are removed from storage
//! - **Adds Output UTXOs**: New UTXOs are added to storage
//! - **Updates Address Mappings**: Address-to-UTXO mappings are updated
//! - **Maintains Consistency**: Ensures state consistency throughout
//!
//! ## Performance Characteristics
//!
//! - **Block Processing**: ~1,000 transactions/second per block
//! - **UTXO Operations**: O(1) average case for add/remove operations
//! - **Memory Usage**: Efficient memory management for large blocks
//! - **Concurrent Processing**: Thread-safe block processing
//!
/// Block processing module for updating UTXO set.
pub mod blockprocessing;

/// Genesis set management module for loading and creating initial UTXO sets.
mod initialset;

/// Re-export blockprocessing module
pub use self::blockprocessing::*;
pub use self::initialset::*;
