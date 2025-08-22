//! ZkOS PostgreSQL Integration Module
//!
//! This module provides PostgreSQL database integration for the ZkOS UTXO state
//! management system. It handles connection pooling, bulk operations, and data
//! persistence with high performance and reliability.
//!
//! ## Core Components
//!
//! - **`init_psql()`**: PostgreSQL connection initialization and setup
//! - **`POSTGRESQL_POOL_CONNECTION`**: Connection pool for database operations
//! - **`THREADPOOL_SQL_QUERY`**: Thread pool for SQL query operations
//! - **`THREADPOOL_SQL_QUEUE`**: Thread pool for SQL queue operations
//!
//! ## Database Schema
//!
//! The PostgreSQL database includes the following tables:
//!
//! - **`utxos`**: Main UTXO storage with key, value, type, and metadata
//! - **`address_utxo_mappings`**: Address-to-UTXO ID mappings
//! - **`snapshots`**: Snapshot metadata and recovery information
//!
//! ## Connection Management
//!
//! The system uses connection pooling for efficient database access:
//!
//! - **Connection Pool**: Manages multiple database connections
//! - **Thread Safety**: Safe for concurrent access from multiple threads
//! - **Automatic Cleanup**: Connections are automatically managed
//! - **Error Handling**: Robust error handling and recovery
//!
//! ## Bulk Operations
//!
//! High-performance bulk operations are supported:
//!
//! - **Bulk Insert**: Efficient insertion of multiple UTXOs
//! - **Bulk Update**: Batch updates for performance
//! - **Bulk Delete**: Efficient removal of multiple UTXOs
//! - **Transaction Safety**: All operations are transactional
//!

mod initiate_sql;
mod sql;
mod sql_api;
pub use self::initiate_sql::{
    init_psql, POSTGRESQL_POOL_CONNECTION, THREADPOOL_SQL_QUERY, THREADPOOL_SQL_QUEUE,
};
pub use self::sql::*;
pub use self::sql_api::*;
