//! Permission + hook gate for MCP `tools/call` requests.

use std::sync::Arc;

use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolRequestParams, ErrorCode};
use talos_core::message::ToolCall;
use talos_permission::{PermissionDecision, PermissionEngine};
use talos_plugin::{HookContext, HookEvent, HookRegistry};

/// Permission gate that enforces hook dispatch and permission decisions.
pub struct McpPermissionGate {
    permission_engine: Arc<PermissionEngine>,
    hook_registry: Arc<HookRegistry>,
}

impl McpPermissionGate {
    /// Creates a permission gate backed by Talos permission and hook systems.
    #[must_use]
    pub fn new(permission_engine: Arc<PermissionEngine>, hook_registry: Arc<HookRegistry>) -> Self {
        Self {
            permission_engine,
            hook_registry,
        }
    }

    /// Evaluates a `tools/call` request with required hook and permission checks.
    pub async fn evaluate_call(
        &self,
        hook_context: &HookContext,
        request: &CallToolRequestParams,
    ) -> Result<(), McpError> {
        let tool_call = to_tool_call(request);

        self.run_hook(
            hook_context,
            HookEvent::OnToolCallProposed { call: &tool_call },
        )
        .await?;

        self.run_hook(
            hook_context,
            HookEvent::BeforePermissionCheck { call: &tool_call },
        )
        .await?;

        let input = request
            .arguments
            .clone()
            .map(serde_json::Value::Object)
            .unwrap_or_else(|| serde_json::json!({}));
        let decision = self
            .permission_engine
            .evaluate(request.name.as_ref(), &input);

        self.run_hook(
            hook_context,
            HookEvent::AfterPermissionCheck {
                call: &tool_call,
                decision: decision.clone(),
            },
        )
        .await?;

        match decision {
            PermissionDecision::Allow => Ok(()),
            PermissionDecision::Deny(reason) => Err(McpError::new(
                ErrorCode::INTERNAL_ERROR,
                format!("permission denied: {reason}"),
                None,
            )),
            PermissionDecision::Ask => Err(McpError::new(
                ErrorCode::INTERNAL_ERROR,
                "permission denied: Ask policy unavailable in headless MCP server mode",
                None,
            )),
        }
    }

    async fn run_hook(&self, context: &HookContext, event: HookEvent<'_>) -> Result<(), McpError> {
        let outcome = self.hook_registry.dispatch(context, event).await;
        if let talos_plugin::HookOutcome::Deny { reason, .. } = outcome {
            return Err(McpError::new(
                ErrorCode::INTERNAL_ERROR,
                format!("hook denied operation: {reason}"),
                None,
            ));
        }
        Ok(())
    }
}

fn to_tool_call(request: &CallToolRequestParams) -> ToolCall {
    ToolCall {
        id: format!("mcp:{}", request.name),
        name: request.name.to_string(),
        input: request
            .arguments
            .clone()
            .map(serde_json::Value::Object)
            .unwrap_or_else(|| serde_json::json!({})),
    }
}
