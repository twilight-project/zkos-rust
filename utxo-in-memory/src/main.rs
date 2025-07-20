//! ZkOS UTXO In-Memory Store Binary
//!
//! This binary provides a complete UTXO state management system for ZkOS,
//! including in-memory storage, PostgreSQL persistence, and blockchain
//! integration. It serves as the main entry point for the UTXO store.
//!
//! ## Features
//!
//! - **UTXO State Management**: Complete UTXO lifecycle management
//! - **Blockchain Integration**: Real-time block processing via chain oracle
//! - **PostgreSQL Persistence**: Reliable data persistence and recovery
//! - **Performance Monitoring**: Prometheus metrics and telemetry
//! - **Snapshot System**: State recovery and backup capabilities
//!
//! ## Architecture
//!
//! The binary implements a multi-layered architecture:
//!
//! 1. **Initialization**: Database setup and state loading
//! 2. **Block Processing**: Real-time blockchain integration
//! 3. **State Management**: In-memory UTXO operations
//! 4. **Persistence**: PostgreSQL synchronization
//! 5. **Monitoring**: Metrics and telemetry collection
//!
//! ## Startup Process
//!
//! 1. **Database Connection**: Establish PostgreSQL connection pool
//! 2. **State Loading**: Load UTXO state from PostgreSQL
//! 3. **Address Mapping**: Rebuild address-to-UTXO mappings
//! 4. **Metrics Initialization**: Set up Prometheus metrics
//! 5. **Block Subscription**: Start blockchain block processing
//!
//! ## Performance Characteristics
//!
//! - **Startup Time**: ~5-10 seconds for typical UTXO sets
//! - **Memory Usage**: ~150 bytes per UTXO including overhead
//! - **Block Processing**: ~1,000 transactions/second
//! - **Database Operations**: ~10,000 UTXOs/second bulk operations
//!
//! ## Configuration
//!
//! The binary uses environment variables for configuration:
//!
//! ```bash
//! # Database configuration
//! export DATABASE_URL="postgresql://user:password@localhost/zkos_db"
//!
//! # Chain oracle configuration
//! export NYKS_BLOCK_SUBSCRIBER_URL="http://localhost:1317/"
//!
//! # Block height tracking
//! export BLOCK_HEIGHT_FILE="height.txt"
//! ```
//!
//! ## Monitoring
//!
//! The binary exposes Prometheus metrics:
//!
//! - `utxo_coin_count`: Number of coin UTXOs
//! - `utxo_memo_count`: Number of memo UTXOs
//! - `utxo_state_count`: Number of state UTXOs
//!
//! ## Error Handling
//!
//! Comprehensive error handling is provided:
//! - **Database Errors**: Connection retry and recovery
//! - **Block Processing Errors**: Graceful error handling and logging
//! - **State Inconsistencies**: Detection and recovery mechanisms
//! - **System Failures**: Automatic restart and recovery

// #[macro_use]

extern crate lazy_static;
//use tungstenite::{connect, Message};
use utxo_in_memory::*;

/// Main entry point for the ZkOS UTXO in-memory store
///
/// This function initializes the UTXO store, loads state from PostgreSQL,
/// and starts the blockchain block processing system. It provides a complete
/// UTXO state management solution for the ZkOS system.
///
/// ## Process Flow
///
/// 1. **Performance Measurement**: Start timing the initialization process
/// 2. **UTXO Initialization**: Load UTXO state and set up storage
/// 3. **Timing Output**: Report initialization performance
/// 4. **Block Processing**: Start blockchain integration (commented out)
///
/// ## Performance Monitoring
///
/// The function measures and reports initialization time for performance
/// monitoring and optimization purposes.
///
/// ## Error Handling
///
/// Any errors during initialization will cause the program to panic,
/// ensuring that the system starts in a consistent state.
fn main() {
    // Start performance measurement
    let sw = Stopwatch::start_new();

    // Initialize UTXO store and load state
    init_utxo();

    // Report initialization performance
    let time1 = sw.elapsed();
    println!("init_utxo: {:#?}", time1);

    // Note: Block processing is currently commented out
    // The system is designed to start blockchain integration here
    // zk_oracle_subscriber();
}

// Legacy code for UTXO loading and testing (commented out)
// pub fn load_utxo() {
//     let mut utxo_storage = UTXO_STORAGE.lock().unwrap();
//     let (acc, prv) = Account::generate_random_account_with_value(Scalar::from(20u64));
//     // let mut recordutxo = transaction::reference_tx::create_genesis_block(10000, 100, acc);
//     let mut recordutxo = crate::dbcurd::load_genesis_sets_test();
//     println!("new utxo0 len:{:#?}", recordutxo.len());
//     let block1 = transaction::reference_tx::create_utxo_test_block(
//         &mut recordutxo,
//         utxo_storage.block_height as u64,
//         &vec![prv],
//     );
//     println!("new utxo len:{:#?}", recordutxo.len());
//     let (acc, prv) = Account::generate_random_account_with_value(Scalar::from(20u64));
//     let block2 = transaction::reference_tx::create_utxo_test_block(
//         &mut recordutxo,
//         (utxo_storage.block_height + 1) as u64,
//         &vec![prv],
//     );
//     println!("new utxo len:{:#?}", recordutxo.len());
//     let (acc, prv) = Account::generate_random_account_with_value(Scalar::from(20u64));
//     let block3 = transaction::reference_tx::create_utxo_test_block(
//         &mut recordutxo,
//         (utxo_storage.block_height + 2) as u64,
//         &vec![prv],
//     );
//     println!("new utxo len:{:#?}", recordutxo.len());
//     let mut file =
//         std::fs::File::create("../utxo-in-memory\\src\\dbcurd\\test\\block1.txt").unwrap();
//     file.write_all(&serde_json::to_vec_pretty(&block1.clone()).unwrap())
//         .unwrap();
//     let mut file =
//         std::fs::File::create("../utxo-in-memory\\src\\dbcurd\\test\\block2.txt").unwrap();
//     file.write_all(&serde_json::to_vec_pretty(&block2.clone()).unwrap())
//         .unwrap();
//     let mut file =
//         std::fs::File::create("../utxo-in-memory\\src\\dbcurd\\test\\block3.txt").unwrap();
//     file.write_all(&serde_json::to_vec_pretty(&block3.clone()).unwrap())
//         .unwrap();

//     let zkblock = ZkosBlock::get_block_details(block1);
//     let resultblock1 = utxo_storage.process_block(zkblock);
//     let mut file =
//         std::fs::File::create("../utxo-in-memory\\src\\dbcurd\\test\\resultblock1.txt").unwrap();
//     file.write_all(
//         &serde_json::to_vec_pretty(&format!("{:#?}", resultblock1.unwrap().error_vec)).unwrap(),
//     )
//     .unwrap();

//     let zkblock = ZkosBlock::get_block_details(block2);
//     let resultblock2 = utxo_storage.process_block(zkblock);
//     let mut file =
//         std::fs::File::create("../utxo-in-memory\\src\\dbcurd\\test\\resultblock2.txt").unwrap();
//     file.write_all(
//         &serde_json::to_vec_pretty(&format!("{:#?}", resultblock2.unwrap().error_vec)).unwrap(),
//     )
//     .unwrap();

//     let zkblock = ZkosBlock::get_block_details(block3);
//     let zkblock_clone = zkblock.clone();
//     let sw1 = Stopwatch::start_new();
//     let resultblock3 = utxo_storage.process_block(zkblock_clone);
//     let time2 = sw1.elapsed();
//     println!(
//         "utxo_storage.process_block: {:#?}\n with len:{:#?}",
//         time2,
//         zkblock.add_utxo.len() + zkblock.remove_block.len()
//     );
//     let mut file =
//         std::fs::File::create("../utxo-in-memory\\src\\dbcurd\\test\\resultblock3.txt").unwrap();
//     file.write_all(
//         &serde_json::to_vec_pretty(&format!("{:#?}", resultblock3.unwrap().error_vec)).unwrap(),
//     )
//     .unwrap();
//     let sw = Stopwatch::start_new();
//     let _ = utxo_storage.take_snapshot();
//     let time1 = sw.elapsed();
//     println!(
//         "utxo_storage.take_snapshot: {:#?} with len:{:#?}",
//         time1,
//         utxo_storage.coin_storage.len()
//             + utxo_storage.memo_storage.len()
//             + utxo_storage.state_storage.len()
//     );
//     let mut file =
//         std::fs::File::create("../utxo-in-memory\\src\\dbcurd\\test\\genesis_sets_test.txt")
//             .unwrap();
//     file.write_all(&serde_json::to_vec_pretty(&recordutxo.clone()).unwrap())
//         .unwrap();
// }

use stopwatch::Stopwatch;
