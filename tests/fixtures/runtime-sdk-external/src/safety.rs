//! Execution assertions through the public SDK, with no host commands or live provider.
use crate::provider::FixtureProvider;
use anyhow::{Result, ensure};
use async_trait::async_trait;
use serde_json::{Value, json};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use talos_runtime::{
    AgentEvent, AgentTool, ApprovalChoice, ApprovalHandler, GrantPreview, Message,
    PermissionDecision, PermissionRule, RuntimeBuilder, RuntimeHandle, SandboxConfig, SandboxError,
    SandboxFallbackPolicy, SandboxProvider, SandboxResult, SessionEvent, ToolNature,
    ToolPermissionFacet, ToolResourceKind, ToolResult, TurnCompletionStatus, TurnEventPayload,
};

struct ScopedApproval(Arc<AtomicUsize>);

#[async_trait]
impl ApprovalHandler for ScopedApproval {
    async fn request_approval(&self, _: &str, _: &Value, _: &[String]) -> ApprovalChoice {
        panic!("bounded scoped approval must be preferred");
    }
    async fn request_scoped_approval(
        &self,
        name: &str,
        _: &Value,
        _: &[String],
        preview: &GrantPreview,
    ) -> ApprovalChoice {
        assert_eq!(name, "bash");
        assert!(!preview.facets().is_empty());
        assert_eq!(preview.facets()[0].nature, ToolNature::Execute);
        self.0.fetch_add(1, Ordering::SeqCst);
        ApprovalChoice::ApproveOnce
    }
}

struct CountingTool {
    name: &'static str,
    nature: ToolNature,
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl AgentTool for CountingTool {
    fn name(&self) -> &str {
        self.name
    }
    fn description(&self) -> &str {
        "SDK fixture invocation counter; no actual side effects"
    }
    fn parameters(&self) -> Value {
        json!({"type":"object", "properties":{"command":{"type":"string"}}})
    }
    fn nature(&self) -> ToolNature {
        self.nature
    }
    fn permission_profile(&self, input: &Value) -> Vec<ToolPermissionFacet> {
        if self.nature == ToolNature::Execute {
            vec![ToolPermissionFacet::with_resource(
                self.nature,
                input["command"].as_str().expect("fixture command"),
                ToolResourceKind::Command,
            )]
        } else {
            vec![ToolPermissionFacet::new(self.nature)]
        }
    }
    async fn execute(&self, _: Value) -> ToolResult {
        self.calls.fetch_add(1, Ordering::SeqCst);
        ToolResult::success("executed fixture tool")
    }
}

struct FixtureSandbox {
    available: bool,
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl SandboxProvider for FixtureSandbox {
    fn is_available(&self) -> bool {
        self.available
    }
    async fn execute(
        &self,
        command: &str,
        config: &SandboxConfig,
    ) -> Result<SandboxResult, SandboxError> {
        assert!(self.available);
        assert!(!config.allow_network);
        assert_eq!(command, "fixture-command");
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(SandboxResult {
            stdout: "isolated fixture".into(),
            stderr: String::new(),
            exit_code: 0,
        })
    }
}

async fn collect(runtime: &mut RuntimeHandle) -> Result<bool> {
    let mut result_error = None;
    let mut started = false;
    let mut tool_call = false;
    while let Some(event) = runtime.next_event().await {
        // The event and all nested protocol types are available without talos-core imports.
        let wire = serde_json::to_string(&event)?;
        let _: SessionEvent = serde_json::from_str(&wire)?;
        if let SessionEvent::TurnEvent { payload, .. } = event {
            match payload {
                TurnEventPayload::Started => started = true,
                TurnEventPayload::Progress {
                    event: AgentEvent::ToolCall { call, .. },
                } => {
                    ensure!(call.id == "fixture-call");
                    tool_call = true;
                }
                TurnEventPayload::Progress {
                    event: AgentEvent::ToolResult { result },
                } => {
                    ensure!(result.tool_use_id == "fixture-call");
                    ensure!(
                        result_error.replace(result.is_error).is_none(),
                        "duplicate result"
                    );
                }
                TurnEventPayload::Completed { status } => {
                    ensure!(started && tool_call, "missing typed lifecycle events");
                    ensure!(matches!(status, TurnCompletionStatus::Success { .. }));
                    return result_error.ok_or_else(|| anyhow::anyhow!("missing tool result"));
                }
                _ => {}
            }
        }
    }
    anyhow::bail!("event stream ended before terminal event")
}

pub async fn run() -> Result<()> {
    // Permission Deny must dominate fallback AllowUnsandboxed; Ask with no host fails closed.
    // An allowed write proves the counter can actually execute, so zero counters are meaningful.
    for (name, nature, decision, available, fallback, expected_tool, expected_sandbox) in [
        (
            "fixture_write",
            ToolNature::Write,
            PermissionDecision::Allow,
            false,
            None,
            1,
            0,
        ),
        (
            "fixture_write",
            ToolNature::Write,
            PermissionDecision::Deny("fixture deny".into()),
            false,
            None,
            0,
            0,
        ),
        (
            "fixture_write",
            ToolNature::Write,
            PermissionDecision::Ask,
            false,
            None,
            0,
            0,
        ),
        (
            "bash",
            ToolNature::Execute,
            PermissionDecision::Allow,
            false,
            None,
            0,
            0,
        ),
        (
            "bash",
            ToolNature::Execute,
            PermissionDecision::Allow,
            true,
            None,
            0,
            1,
        ),
        (
            "bash",
            ToolNature::Execute,
            PermissionDecision::Deny("fixture deny".into()),
            false,
            Some(SandboxFallbackPolicy::AllowUnsandboxed),
            0,
            0,
        ),
        (
            "bash",
            ToolNature::Execute,
            PermissionDecision::Allow,
            false,
            Some(SandboxFallbackPolicy::AllowUnsandboxed),
            1,
            0,
        ),
        (
            "bash",
            ToolNature::Execute,
            PermissionDecision::Allow,
            false,
            Some(SandboxFallbackPolicy::Ask),
            0,
            0,
        ),
    ] {
        let tools = Arc::new(AtomicUsize::new(0));
        let sandboxes = Arc::new(AtomicUsize::new(0));
        let directory = tempfile::tempdir()?;
        let mut builder = RuntimeBuilder::new()
            .workspace_root(directory.path())
            .initial_history(vec![Message::User {
                content: "fixture history".into(),
            }])
            .provider(Arc::new(
                FixtureProvider::new()
                    .with_tool_call(name, json!({"command":"fixture-command"}))
                    .with_response("fixture completed"),
            ))
            .tool(Arc::new(CountingTool {
                name,
                nature,
                calls: tools.clone(),
            }))
            .permission_rule(PermissionRule::new_nature(nature, None, None, decision))
            .sandbox(Box::new(FixtureSandbox {
                available,
                calls: sandboxes.clone(),
            }));
        if let Some(policy) = fallback {
            builder = builder.sandbox_fallback(policy);
        }
        let mut runtime = builder.build()?;
        runtime.submit("exercise fixture tool").await?;
        let error = collect(&mut runtime).await?;
        runtime.shutdown().await?;
        ensure!(
            tools.load(Ordering::SeqCst) == expected_tool,
            "unexpected tool execution: {name}"
        );
        ensure!(
            sandboxes.load(Ordering::SeqCst) == expected_sandbox,
            "unexpected sandbox execution"
        );
        ensure!(
            error == (expected_tool + expected_sandbox == 0),
            "wrong tool result status"
        );
    }
    // Ordinary approval is not sandbox-fallback approval: default host fallback stays Deny.
    let calls = Arc::new(AtomicUsize::new(0));
    let approvals = Arc::new(AtomicUsize::new(0));
    let sandbox_calls = Arc::new(AtomicUsize::new(0));
    let directory = tempfile::tempdir()?;
    let mut runtime = RuntimeBuilder::new()
        .workspace_root(directory.path())
        .provider(Arc::new(
            FixtureProvider::new()
                .with_tool_call("bash", json!({"command":"fixture-command"}))
                .with_response("approval scope checked"),
        ))
        .tool(Arc::new(CountingTool {
            name: "bash",
            nature: ToolNature::Execute,
            calls: calls.clone(),
        }))
        .permission_rule(PermissionRule::new_nature(
            ToolNature::Execute,
            None,
            None,
            PermissionDecision::Ask,
        ))
        .approval_handler(Arc::new(ScopedApproval(approvals.clone())))
        .sandbox(Box::new(FixtureSandbox {
            available: false,
            calls: sandbox_calls.clone(),
        }))
        .sandbox_fallback(SandboxFallbackPolicy::Ask)
        .build()?;
    runtime.submit("check separate fallback approval").await?;
    ensure!(
        collect(&mut runtime).await?,
        "fallback must still be denied"
    );
    runtime.shutdown().await?;
    ensure!(approvals.load(Ordering::SeqCst) == 1);
    ensure!(calls.load(Ordering::SeqCst) == 0);
    ensure!(sandbox_calls.load(Ordering::SeqCst) == 0);
    Ok(())
}
