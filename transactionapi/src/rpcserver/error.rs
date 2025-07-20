use jsonrpc_core::types::error::Error as JsonRpcError;
use std::fmt;

/// A custom error type for our RPC server.
///
/// This enum consolidates all possible errors that can occur within the RPC server,
/// providing a single, consistent way to handle failures.
#[derive(Debug)]
pub enum RpcError {
    /// Error for invalid parameters in a request.
    InvalidParams(String),
    /// Error for internal server issues.
    InternalError(String),
    /// Error for failed transaction verification.
    TxVerificationError(String),
    /// Error when a UTXO is not found.
    UtxoNotFound,
    /// Error from hex decoding.
    Hex(hex::FromHexError),
    /// Error from bincode serialization/deserialization.
    Bincode(Box<bincode::ErrorKind>),
    /// Error from serde_json serialization/deserialization.
    SerdeJson(serde_json::Error),
    /// An error originating from the JSON-RPC library itself.
    JsonRpc(JsonRpcError),
    /// A generic error for failed responses.
    ResponseError(String),
    /// Error for serialization errors.
    SerializationError(String),
    /// Error for network errors.
    NetworkError(String),
}

impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RpcError::InvalidParams(s) => write!(f, "Invalid parameters: {}", s),
            RpcError::InternalError(s) => write!(f, "Internal server error: {}", s),
            RpcError::TxVerificationError(s) => write!(f, "Transaction verification failed: {}", s),
            RpcError::UtxoNotFound => write!(f, "UTXO not found"),
            RpcError::Hex(e) => write!(f, "Hex decoding error: {}", e),
            RpcError::Bincode(e) => write!(f, "Bincode error: {}", e),
            RpcError::SerdeJson(e) => write!(f, "Serde JSON error: {}", e),
            RpcError::JsonRpc(e) => write!(f, "JSON RPC error: {}", e.message),
            RpcError::ResponseError(s) => write!(f, "Response error: {}", s),
            RpcError::SerializationError(s) => write!(f, "Serialization error: {}", s),
            RpcError::NetworkError(s) => write!(f, "Network error: {}", s),
        }
    }
}

impl std::error::Error for RpcError {}

impl From<hex::FromHexError> for RpcError {
    fn from(err: hex::FromHexError) -> Self {
        RpcError::Hex(err)
    }
}

impl From<Box<bincode::ErrorKind>> for RpcError {
    fn from(err: Box<bincode::ErrorKind>) -> Self {
        RpcError::Bincode(err)
    }
}

impl From<serde_json::Error> for RpcError {
    fn from(err: serde_json::Error) -> Self {
        RpcError::SerdeJson(err)
    }
}

impl From<JsonRpcError> for RpcError {
    fn from(err: JsonRpcError) -> Self {
        RpcError::JsonRpc(err)
    }
}

/// Converts our custom `RpcError` into a `JsonRpcError` that the `jsonrpc-http-server` can use.
impl From<RpcError> for JsonRpcError {
    fn from(err: RpcError) -> Self {
        match err {
            RpcError::InvalidParams(msg) => JsonRpcError::invalid_params(msg),
            RpcError::InternalError(msg) => {
                let mut err = JsonRpcError::internal_error();
                err.message = format!("Internal Server Error: {}", msg);
                err
            }
            RpcError::TxVerificationError(msg) => JsonRpcError {
                code: jsonrpc_core::ErrorCode::ServerError(-32001),
                message: msg,
                data: None,
            },
            RpcError::UtxoNotFound => JsonRpcError {
                code: jsonrpc_core::ErrorCode::ServerError(-32002),
                message: "UTXO not found".to_string(),
                data: None,
            },
            RpcError::Hex(e) => JsonRpcError::invalid_params(format!("Invalid hex string: {}", e)),
            RpcError::Bincode(e) => {
                JsonRpcError::invalid_params(format!("Invalid transaction data: {}", e))
            }
            RpcError::SerdeJson(_) => JsonRpcError::internal_error(), // Serialization errors are internal.
            RpcError::JsonRpc(e) => e,
            RpcError::ResponseError(msg) => JsonRpcError {
                code: jsonrpc_core::ErrorCode::ServerError(-32003),
                message: msg,
                data: None,
            },
            RpcError::SerializationError(msg) => JsonRpcError {
                code: jsonrpc_core::ErrorCode::ServerError(-32004),
                message: msg,
                data: None,
            },
            RpcError::NetworkError(msg) => JsonRpcError {
                code: jsonrpc_core::ErrorCode::ServerError(-32005),
                message: msg,
                data: None,
            },
        }
    }
}

/// A type alias for a `Result` that uses our custom `RpcError`.
pub type RpcResult<T> = Result<T, RpcError>;
