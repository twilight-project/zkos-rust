
use std::{error::Error, fmt::Debug};
use r2d2_postgres::postgres::Error as PostgresError;
use r2d2::Error as R2D2Error;
use rusty_leveldb::Status as RustyLevelDBStatus;
#[derive(thiserror::Error)]
pub enum UtxosetError {
    #[error("failed to retrieve a connection from the pool")]
    ConnectionError(#[from] R2D2Error),

    #[error("failed to execute the query over database")]
    StatementExecutionError(#[source] PostgresError),
    
    #[error("missing environment variable")]
    SnapshotFileError(#[from] std::env::VarError),

    #[error("sequence number parse error")]
    SequenceNumberParseError(#[from] std::num::ParseIntError),

    #[error("IO Error")]
    IOError(#[from] std::io::Error),

    #[error("rusty_leveldb error")]
    RustyLevelDBError(#[from] RustyLevelDBStatus),

    #[error("snap shot not found")]
    SnapshotNotFound,

    #[error("serializatrion/desearialization error")]
    SerializationError(#[from] bincode::Error),

    #[error("script address not found")]
    ScriptAddressNotFound,

    #[error("utxo not found")]
    UtxoNotFound,

    #[error("system time error")]
    SystemTimeError(#[from] std::time::SystemTimeError),
    // Add more error variants as needed
}

impl From<PostgresError> for UtxosetError {
    fn from(e: PostgresError) -> Self {
        UtxosetError::StatementExecutionError(e)
    }
}
impl Debug for UtxosetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self)?;
        if let Some(source) = self.source() {
            writeln!(f, "Caused by:\n\t{}", source)?;
        }
        Ok(())
    }
}