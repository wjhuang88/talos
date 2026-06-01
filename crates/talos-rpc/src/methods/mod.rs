//! RPC method dispatch.

pub mod agent;
pub mod system;

use std::sync::Arc;

use serde_json::Value;
use talos_agent::Agent;

use crate::cancel::CancelRegistry;
use crate::error::RpcError;
use crate::protocol::{JsonRpcId, JsonRpcNotification};

/// Method invocation result including optional notifications.
pub struct MethodResult {
    /// Final method result payload.
    pub result: Value,
    /// Outbound notifications that must be emitted before the response.
    pub notifications: Vec<JsonRpcNotification>,
}

/// Shared context for RPC method handlers.
#[derive(Clone)]
pub struct MethodContext {
    /// Agent instance backing method execution.
    pub agent: Arc<Agent>,
    /// In-flight cancellation token registry.
    pub cancel_registry: Arc<CancelRegistry>,
}

/// Dispatches a method call to the corresponding handler.
pub async fn dispatch_method(
    ctx: &MethodContext,
    method: &str,
    params: Option<Value>,
) -> Result<MethodResult, RpcError> {
    match method {
        "system.version" => system::system_version(),
        "agent.list_tools" => agent::list_tools(ctx),
        "agent.run" => agent::run(ctx, params).await,
        "agent.cancel" => agent::cancel(ctx, params),
        _ => Err(RpcError::MethodNotFound(method.to_string())),
    }
}

/// Converts optional request id into a response id.
#[must_use]
pub fn response_id_or_null(id: Option<JsonRpcId>) -> JsonRpcId {
    id.unwrap_or(JsonRpcId::Null)
}
