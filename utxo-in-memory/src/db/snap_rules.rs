use serde_derive::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SnapRules {
    pub path: String,
    pub block_size_threshold: usize,
    pub snap_time_threshold: usize,
}

impl SnapRules {
    pub fn env() -> SnapRules {
        dotenv::dotenv().expect("Failed loading dotenv");

        let snapshot_file_location: String = std::env::var("SNAPSHOT_FILE_LOCATION")
            .expect("missing environment variable SNAPSHOT_FILE_LOCATION");
        let snapshot_blockheight_threshold: usize = std::env::var("SNAPSHOT_BLOCKHEIGHT_THRESHOLD")
            .expect("missing environment variable SNAPSHOT_BLOCKHEIGHT_THRESHOLD")
            .parse::<usize>()
            .unwrap();
        let snapshot_duration_threshold: usize = std::env::var("SNAPSHOT_DURATION_THRESHOLD")
            .expect("missing environment variable SNAPSHOT_DURATION_THRESHOLD")
            .parse::<usize>()
            .unwrap();
        SnapRules {
            path: snapshot_file_location,
            block_size_threshold: snapshot_blockheight_threshold,
            snap_time_threshold: snapshot_duration_threshold,
        }
    }
}
