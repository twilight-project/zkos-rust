use thiserror::Error;

#[derive(Error, Debug)]
pub enum RpcError {
    #[error("Invalid request")]
    InvalidRequest,

    #[error("Method not found")]
    MethodNotFound,

    #[error("Invalid params")]
    InvalidParams,

    #[error("Internal error")]
    InternalError,

    #[error("Parse error")]
    ParseError,

    #[error("Json serde error {0:?}")]
    JsonError(#[from] serde_json::Error),

    #[error("Commit retry count exceeded!")]
    CommitRetryCountExceeded,

}

