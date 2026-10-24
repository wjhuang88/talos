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
    /// Prepared normal-tool stages share this admission deadline. Their execution
    /// uses a separate cumulative wall-clock budget.
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
        match self
            .authorize_call_inner(hook_context, request, profile, None, None)
            .await?
        {
            McpAdmission::Paths(paths) => Ok(paths),
            McpAdmission::Prepared(_) => Err(permission_error("admission route mismatch")),
        }
    }

    /// Legacy entrypoint: refuses preparation without a bound plan identity.
    pub async fn authorize_prepared_call(
        &self,
        hook_context: &HookContext,
        request: &McpCallRequest,
        profile: &[ToolPermissionFacet],
    ) -> Result<talos_permission::AdmittedInvocation, McpError> {
        let _ = (hook_context, request, profile);
        Err(permission_error("prepared plan identity required"))
    }

    /// Admits one exact prepared plan; legacy proofs cannot authorize this route.
    pub async fn authorize_prepared_plan(
        &self,
        hook_context: &HookContext,
        request: &McpCallRequest,
        profile: &[ToolPermissionFacet],
        plan: &talos_core::tool::PreparedPlanIdentity,
    ) -> Result<talos_permission::AdmittedInvocation, McpError> {
        self.authorize_prepared_plan_until(
            hook_context,
            request,
            profile,
            plan,
            self.prepared_deadline_at(),
        )
        .await
    }

    pub(super) fn prepared_deadline_at(&self) -> tokio::time::Instant {
        tokio::time::Instant::now() + self.deadline
    }

    pub(super) async fn authorize_prepared_plan_until(
        &self,
        hook_context: &HookContext,
        request: &McpCallRequest,
        profile: &[ToolPermissionFacet],
        plan: &talos_core::tool::PreparedPlanIdentity,
        deadline_at: tokio::time::Instant,
    ) -> Result<talos_permission::AdmittedInvocation, McpError> {
        self.authorize_prepared_command_until(
            hook_context,
            request,
            profile,
            (plan, None),
            deadline_at,
        )
        .await
    }

    pub(super) async fn authorize_prepared_command_until(
        &self,
        hook_context: &HookContext,
        request: &McpCallRequest,
        profile: &[ToolPermissionFacet],
        prepared: (
            &talos_core::tool::PreparedPlanIdentity,
            Option<&talos_core::tool::PreparedCommandEvidence>,
        ),
        deadline_at: tokio::time::Instant,
    ) -> Result<talos_permission::AdmittedInvocation, McpError> {
        match self
            .authorize_call_inner(
                hook_context,
                request,
                profile,
                Some(prepared),
                Some(deadline_at),
            )
            .await?
        {
            McpAdmission::Prepared(admission) => Ok(*admission),
            McpAdmission::Paths(_) => Err(permission_error("admission route mismatch")),
        }
    }

    async fn authorize_call_inner(
        &self,
        hook_context: &HookContext,
        request: &McpCallRequest,
        profile: &[ToolPermissionFacet],
        prepared: Option<(
            &talos_core::tool::PreparedPlanIdentity,
            Option<&talos_core::tool::PreparedCommandEvidence>,
        )>,
        prepared_deadline_at: Option<tokio::time::Instant>,
    ) -> Result<McpAdmission, McpError> {
        let deadline_at = prepared_deadline_at.unwrap_or_else(|| self.prepared_deadline_at());
        if tokio::time::Instant::now() >= deadline_at {
            return Err(permission_error("permission deadline exceeded"));
        }
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
        let authorization_request = PermissionAuthorizationRequest {
            tool_name: request.name.as_str(),
            provenance: talos_core::tool::ToolProvenance::McpRemote {
                server: "standalone-mcp".to_owned(),
            },
            profile,
            input: &input,
            presentation_input: tool_call.input.clone(),
            summary_fields: Vec::new(),
            deadline: deadline_at.saturating_duration_since(tokio::time::Instant::now()),
        };
        let authorization = if let Some((plan, evidence)) = prepared {
            self.pipeline
                .authorize_prepared_plan_with_evidence(authorization_request, None, plan, evidence)
                .await
                .map(|admission| McpAdmission::Prepared(Box::new(admission)))
        } else {
            self.pipeline
                .authorize(authorization_request)
                .await
                .map(McpAdmission::Paths)
        };
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

enum McpAdmission {
    Paths(Vec<ToolExecutionAuthorization>),
    Prepared(Box<talos_permission::AdmittedInvocation>),
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
