//! System RPC methods.

use crate::error::RpcError;
use crate::methods::MethodResult;

/// Handles `system.version`.
pub fn system_version() -> Result<MethodResult, RpcError> {
    Ok(MethodResult {
        result: serde_json::json!({
        "version": "0.1.0",
        "protocol": 1
        }),
        notifications: Vec::new(),
    })
}
