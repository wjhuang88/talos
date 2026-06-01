//! Agent RPC methods.

use serde::Deserialize;
use serde_json::{json, Value};
use talos_core::message::AgentEvent;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use crate::error::RpcError;
use crate::methods::{MethodContext, MethodResult};
use crate::protocol::{JsonRpcNotification, JSON_RPC_VERSION};

#[derive(Debug, Deserialize)]
struct AgentRunParams {
    prompt: String,
    #[serde(default)]
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct AgentCancelParams {
    turn_id: String,
}

/// Handles `agent.list_tools`.
pub fn list_tools(ctx: &MethodContext) -> Result<MethodResult, RpcError> {
    let _ = ctx;
    Ok(MethodResult {
        result: json!([
        {
            "name": "bash",
            "description": "Execute shell commands in sandboxed environment",
            "input_schema": {
                "type": "object",
                "properties": {
                    "command": { "type": "string" }
                },
                "required": ["command"]
            }
        },
        {
            "name": "read",
            "description": "Read files and directories from filesystem",
            "input_schema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string" }
                },
                "required": ["filePath"]
            }
        },
        {
            "name": "write",
            "description": "Write file contents",
            "input_schema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string" },
                    "content": { "type": "string" }
                },
                "required": ["filePath", "content"]
            }
        },
        {
            "name": "edit",
            "description": "Apply textual edits to files",
            "input_schema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string" },
                    "old_text": { "type": "string" },
                    "new_text": { "type": "string" }
                },
                "required": ["filePath", "old_text", "new_text"]
            }
        }
    ]),
        notifications: Vec::new(),
    })
}

/// Handles `agent.run`.
pub async fn run(
    ctx: &MethodContext,
    params: Option<Value>,
) -> Result<MethodResult, RpcError> {
    let params = params.ok_or_else(|| RpcError::InvalidParams("missing params".to_string()))?;
    let params: AgentRunParams =
        serde_json::from_value(params).map_err(|e| RpcError::InvalidParams(e.to_string()))?;

    let turn_id = format!("turn-{}", uuid_like_nonce());
    let token = CancellationToken::new();
    ctx.cancel_registry.insert(turn_id.clone(), token.clone());

    let mut notifications = Vec::new();

    let output = if params.stream {
        let (event_tx, mut event_rx) = broadcast::channel(32);
        let prompt = params.prompt;
        let agent = ctx.agent.clone();
        let mut run_task = tokio::spawn(async move { agent.run_streaming(prompt, event_tx).await });

        loop {
            tokio::select! {
                _ = token.cancelled() => {
                    run_task.abort();
                    ctx.cancel_registry.remove(&turn_id);
                    return Err(RpcError::Internal("turn cancelled".to_string()));
                }
                recv_result = event_rx.recv() => {
                    match recv_result {
                        Ok(AgentEvent::TextDelta { delta }) => {
                            let notification = JsonRpcNotification {
                                jsonrpc: JSON_RPC_VERSION.to_string(),
                                method: "agent.run.delta".to_string(),
                                params: Some(json!({
                                    "turn_id": turn_id,
                                    "delta": delta
                                })),
                            };
                            notifications.push(notification);
                        }
                        Ok(AgentEvent::TurnEnd { .. }) => break,
                        Ok(AgentEvent::Error { message }) => return Err(RpcError::Internal(message)),
                        Ok(AgentEvent::TurnStart | AgentEvent::ToolCall { .. } | AgentEvent::ToolResult { .. }) => {}
                        Err(_) => break,
                    }
                }
            }
        }

        tokio::select! {
            _ = token.cancelled() => {
                run_task.abort();
                ctx.cancel_registry.remove(&turn_id);
                return Err(RpcError::Internal("turn cancelled".to_string()));
            }
            result = &mut run_task => {
                result
                    .map_err(|e| RpcError::Internal(e.to_string()))?
                    .map_err(|e| RpcError::Internal(e.to_string()))?
            }
        }
    } else {
        let prompt = params.prompt;
        let agent = ctx.agent.clone();
        let mut run_task = tokio::spawn(async move { agent.run(prompt).await });
        tokio::select! {
            _ = token.cancelled() => {
                run_task.abort();
                ctx.cancel_registry.remove(&turn_id);
                return Err(RpcError::Internal("turn cancelled".to_string()));
            }
            result = &mut run_task => {
                result
                    .map_err(|e| RpcError::Internal(e.to_string()))?
                    .map_err(|e| RpcError::Internal(e.to_string()))?
            }
        }
    };

    ctx.cancel_registry.remove(&turn_id);

    Ok(MethodResult {
        result: json!({
            "turn_id": turn_id,
            "result": output
        }),
        notifications,
    })
}

/// Handles `agent.cancel`.
pub fn cancel(ctx: &MethodContext, params: Option<Value>) -> Result<MethodResult, RpcError> {
    let params = params.ok_or_else(|| RpcError::InvalidParams("missing params".to_string()))?;
    let params: AgentCancelParams =
        serde_json::from_value(params).map_err(|e| RpcError::InvalidParams(e.to_string()))?;
    let cancelled = ctx.cancel_registry.cancel(&params.turn_id);
    Ok(MethodResult {
        result: json!({ "cancelled": cancelled }),
        notifications: Vec::new(),
    })
}

fn uuid_like_nonce() -> String {
    format!("{}", talos_plugin::TurnId::new().0)
}
