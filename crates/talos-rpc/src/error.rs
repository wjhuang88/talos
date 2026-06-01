//! RPC error types and JSON-RPC code mapping.

use serde_json::Value;
use thiserror::Error;

use crate::protocol::JsonRpcError;

/// JSON-RPC parse error code.
pub const CODE_PARSE_ERROR: i32 = -32700;
/// JSON-RPC invalid request code.
pub const CODE_INVALID_REQUEST: i32 = -32600;
/// JSON-RPC method not found code.
pub const CODE_METHOD_NOT_FOUND: i32 = -32601;
/// JSON-RPC invalid params code.
pub const CODE_INVALID_PARAMS: i32 = -32602;
/// JSON-RPC internal error code.
pub const CODE_INTERNAL_ERROR: i32 = -32603;

/// RPC server error.
#[derive(Debug, Error)]
pub enum RpcError {
    /// Input could not be parsed as JSON.
    #[error("parse error")]
    ParseError,
    /// Request shape is invalid for JSON-RPC 2.0.
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    /// Requested method is not registered.
    #[error("method not found: {0}")]
    MethodNotFound(String),
    /// Parameters are invalid for the selected method.
    #[error("invalid params: {0}")]
    InvalidParams(String),
    /// Internal server failure.
    #[error("internal error: {0}")]
    Internal(String),
}

impl RpcError {
    /// Converts this error to a JSON-RPC error object.
    #[must_use]
    pub fn to_json_rpc_error(&self) -> JsonRpcError {
        match self {
            Self::ParseError => JsonRpcError {
                code: CODE_PARSE_ERROR,
                message: "Parse error".to_string(),
                data: None,
            },
            Self::InvalidRequest(message) => JsonRpcError {
                code: CODE_INVALID_REQUEST,
                message: "Invalid Request".to_string(),
                data: Some(Value::String(message.clone())),
            },
            Self::MethodNotFound(message) => JsonRpcError {
                code: CODE_METHOD_NOT_FOUND,
                message: "Method not found".to_string(),
                data: Some(Value::String(message.clone())),
            },
            Self::InvalidParams(message) => JsonRpcError {
                code: CODE_INVALID_PARAMS,
                message: "Invalid params".to_string(),
                data: Some(Value::String(message.clone())),
            },
            Self::Internal(message) => JsonRpcError {
                code: CODE_INTERNAL_ERROR,
                message: "Internal error".to_string(),
                data: Some(Value::String(message.clone())),
            },
        }
    }
}
