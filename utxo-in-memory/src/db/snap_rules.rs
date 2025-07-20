//! Snapshot configuration rules loaded from environment variables.
//!
//! This module defines the configuration parameters for snapshot creation
//! including file paths, block height thresholds, and timing thresholds.

use crate::db::SequenceNumber;
use serde_derive::{Deserialize, Serialize};
use std::path::Path;

/// Configuration rules for snapshot creation and management
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SnapRules {
    /// File path for snapshot storage
    pub path: String,
    /// Block height threshold for triggering snapshots
    pub block_size_threshold: SequenceNumber,
    /// Time threshold for triggering snapshots
    pub snap_time_threshold: SequenceNumber,
}

impl SnapRules {
    /// Creates SnapRules from environment variables
    pub fn env() -> SnapRules {
        dotenv::from_path(Path::new("/testnet/zkos-rust/utxo-in-memory/.env")).ok();
        // dotenv::dotenv().expect("Failed loading dotenv");

        let snapshot_file_location: String = std::env::var("SNAPSHOT_FILE_LOCATION")
            .expect("missing environment variable SNAPSHOT_FILE_LOCATION");
        let snapshot_blockheight_threshold: SequenceNumber =
            std::env::var("SNAPSHOT_BLOCKHEIGHT_THRESHOLD")
                .expect("missing environment variable SNAPSHOT_BLOCKHEIGHT_THRESHOLD")
                .parse::<SequenceNumber>()
                .unwrap();
        let snapshot_duration_threshold: SequenceNumber =
            std::env::var("SNAPSHOT_DURATION_THRESHOLD")
                .expect("missing environment variable SNAPSHOT_DURATION_THRESHOLD")
                .parse::<SequenceNumber>()
                .unwrap();
        SnapRules {
            path: snapshot_file_location,
            block_size_threshold: snapshot_blockheight_threshold,
            snap_time_threshold: snapshot_duration_threshold,
        }
    }
}
