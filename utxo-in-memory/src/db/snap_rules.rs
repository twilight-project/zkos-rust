use crate::{db::SequenceNumber, error::UtxosetError};
use std::path::Path;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SnapRules {
    pub path: String,
    pub block_size_threshold: SequenceNumber,
    pub snap_time_threshold: SequenceNumber,
}

impl SnapRules {

    pub fn env() -> SnapRules {
        dotenv::from_path(Path::new("/testnet/zkos-rust/utxo-in-memory/.env")).ok();
        // dotenv::dotenv().expect("Failed loading dotenv");

        // let snapshot_file_location: String = std::env::var("SNAPSHOT_FILE_LOCATION")
        //     .expect("missing environment variable SNAPSHOT_FILE_LOCATION");
        // let snapshot_blockheight_threshold: SequenceNumber =
        //     std::env::var("SNAPSHOT_BLOCKHEIGHT_THRESHOLD")
        //         .expect("missing environment variable SNAPSHOT_BLOCKHEIGHT_THRESHOLD")
        //         .parse::<SequenceNumber>()
        //         .unwrap();
        // let snapshot_duration_threshold: SequenceNumber =
        //     std::env::var("SNAPSHOT_DURATION_THRESHOLD")
        //         .expect("missing environment variable SNAPSHOT_DURATION_THRESHOLD")
        //         .parse::<SequenceNumber>()
        //         .unwrap();
        let snapshot_file_location: String = std::env::var("SNAPSHOT_FILE_LOCATION")?;
        let snapshot_blockheight_threshold: SequenceNumber =
            std::env::var("SNAPSHOT_BLOCKHEIGHT_THRESHOLD")?.parse::<SequenceNumber>()?;
        let snapshot_duration_threshold: SequenceNumber = std::env::var("SNAPSHOT_DURATION_THRESHOLD")?
            .parse::<SequenceNumber>()?;    
        Ok(SnapRules {
            path: snapshot_file_location,
            block_size_threshold: snapshot_blockheight_threshold,
            snap_time_threshold: snapshot_duration_threshold,
        })
    }
}
