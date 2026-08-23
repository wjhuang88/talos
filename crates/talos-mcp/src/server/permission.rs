//! Permission + hook gate for MCP `tools/call` requests.

use std::sync::Arc;
use std::time::Duration;

use rmcp::ErrorData as McpError;
use rmcp::model::ErrorCode;
use talos_agent::permission_pipeline::{PermissionAuthorizationRequest, PermissionPipeline};
use talos_core::message::ToolCall;
use talos_core::tool::{ToolExecutionAuthorization, ToolPermissionFacet};
use talos_permission::{
    PermissionContext, PermissionEngine, PermissionMode, PermissionSessionState,
};
use talos_plugin::{HookContext, HookEvent, HookRegistry};

use crate::types::McpCallRequest;

/// Permission gate that enforces hook dispatch and permission decisions.
pub struct McpPermissionGate {
    pipeline: PermissionPipeline,
    hook_registry: Arc<HookRegistry>,
    deadline: Duration,
}

impl McpPermissionGate {
    /// Creates a permission gate backed by Talos permission and hook systems.
    #[must_use]
    pub fn new(permission_engine: Arc<PermissionEngine>, hook_registry: Arc<HookRegistry>) -> Self {
        Self {
            pipeline: PermissionPipeline::new(
                Arc::new(PermissionSessionState::new((*permission_engine).clone())),
                PermissionContext::new(
                    PermissionMode::Headless,
                    talos_permission::InteractionCapability::Unavailable,
                ),
                None,
            ),
            hook_registry,
            deadline: Duration::from_secs(30),
        }
    }

    /// Overrides the total budget shared by MCP permission hooks and admission.
    #[must_use]
    pub fn with_deadline(mut self, deadline: Duration) -> Self {
        self.deadline = deadline;
        self
    }

    /// Evaluates a `tools/call` request with the source-compatible legacy result.
    ///
    /// New execution paths should use [`Self::authorize_call`] so the admitted capability reaches
    /// `execute_authorized_with_output` without a second evaluation.
    pub async fn evaluate_call(
        &self,
        hook_context: &HookContext,
        request: &McpCallRequest,
        profile: &[ToolPermissionFacet],
    ) -> Result<(), McpError> {
        self.authorize_call(hook_context, request, profile)
            .await
            .map(|_| ())
    }

    /// Authorizes an MCP call and returns the exact capability consumed by execution.
    pub async fn authorize_call(
        &self,
        hook_context: &HookContext,
        request: &McpCallRequest,
        profile: &[ToolPermissionFacet],
    ) -> Result<Vec<ToolExecutionAuthorization>, McpError> {
        let deadline_at = tokio::time::Instant::now() + self.deadline;
        let tool_call = to_permission_tool_call(request);

        self.run_permission_hook_before_deadline(
            deadline_at,
            hook_context,
            HookEvent::OnToolCallProposed { call: &tool_call },
        )
        .await?;

        self.run_permission_hook_before_deadline(
            deadline_at,
            hook_context,
            HookEvent::BeforePermissionCheck { call: &tool_call },
        )
        .await?;

        let input = request
            .arguments
            .clone()
            .map(serde_json::Value::Object)
            .unwrap_or_else(|| serde_json::json!({}));
        let authorization = self
            .pipeline
            .authorize(PermissionAuthorizationRequest {
                tool_name: request.name.as_str(),
                provenance: talos_core::tool::ToolProvenance::McpRemote {
                    server: "standalone-mcp".to_owned(),
                },
                profile,
                input: &input,
                presentation_input: tool_call.input.clone(),
                summary_fields: Vec::new(),
                deadline: deadline_at.saturating_duration_since(tokio::time::Instant::now()),
            })
            .await;
        let decision = authorization
            .as_ref()
            .err()
            .map_or(talos_permission::PermissionDecision::Allow, |error| {
                error.final_decision()
            });
        self.run_permission_hook_before_deadline(
            deadline_at,
            hook_context,
            HookEvent::AfterPermissionCheck {
                call: &tool_call,
                decision,
            },
        )
        .await?;
        authorization.map_err(|error| {
            McpError::new(
                ErrorCode::INTERNAL_ERROR,
                format!("permission denied: {error}"),
                None,
            )
        })
    }

    async fn run_permission_hook_before_deadline(
        &self,
        deadline: tokio::time::Instant,
        context: &HookContext,
        event: HookEvent<'_>,
    ) -> Result<(), McpError> {
        let outcome = tokio::time::timeout_at(
            deadline,
            self.hook_registry.dispatch_permission_gate(context, event),
        )
        .await
        .map_err(|_| permission_error("permission deadline exceeded"))?;
        match outcome {
            talos_plugin::HookOutcome::Continue(_) => Ok(()),
            talos_plugin::HookOutcome::Skip(_) => {
                Err(permission_error("permission hook failed closed"))
            }
            talos_plugin::HookOutcome::Deny { reason, .. } => Err(permission_error(&reason)),
        }
    }
}

fn to_permission_tool_call(request: &McpCallRequest) -> ToolCall {
    let input = request
        .arguments
        .clone()
        .map(serde_json::Value::Object)
        .unwrap_or_else(|| serde_json::json!({}));
    ToolCall {
        id: format!("mcp:{}", request.name),
        name: request.name.clone(),
        input: talos_agent::permission_pipeline::project_permission_input(&input),
    }
}

fn permission_error(reason: &str) -> McpError {
    McpError::new(
        ErrorCode::INTERNAL_ERROR,
        format!("permission denied: {reason}"),
        None,
    )
}
