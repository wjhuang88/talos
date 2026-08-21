use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use talos_core::approval::ApprovalChoice;
use talos_core::provider::LanguageModel;
use talos_core::tool::{AgentTool, ToolNature, ToolResult};
use talos_permission::PermissionDecision;
use talos_provider::mock::MockProvider;
use talos_runtime::{
    ApprovalHandler, RuntimeBuilder, RuntimeError, RuntimeTurnCompletionStatus,
    SandboxFallbackPolicy, ShutdownOptions, collect_until_turn_completed,
};
use talos_session::SessionManager;

struct ReadOnlyGreeting;

#[async_trait]
impl AgentTool for ReadOnlyGreeting {
    fn name(&self) -> &str {
        "fixture_greeting"
    }

    fn description(&self) -> &str {
        "Returns a deterministic greeting for fixture validation"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({"type": "object", "properties": {}})
    }

    async fn execute(&self, _input: Value) -> ToolResult {
        ToolResult::success("hello from the external fixture")
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn nature(&self) -> ToolNature {
        ToolNature::Read
    }
}

struct DenyApproval;

#[async_trait]
impl ApprovalHandler for DenyApproval {
    async fn request_approval(
        &self,
        _tool_name: &str,
        _arguments: &Value,
        _summary_fields: &[String],
    ) -> ApprovalChoice {
        ApprovalChoice::Deny
    }
}

fn provider(response: &str) -> Arc<dyn LanguageModel> {
    Arc::new(MockProvider::new().with_response(response))
}

fn classify_runtime_error(error: &RuntimeError) -> &'static str {
    match error {
        RuntimeError::RuntimeClosing => "closing",
        RuntimeError::ShutdownIncomplete { .. } => "shutdown_incomplete",
        _ => "other_runtime_error",
    }
}

async fn run_minimal_runtime() -> Result<()> {
    let mut runtime = RuntimeBuilder::new()
        .provider(Arc::new(
            MockProvider::new()
                .with_tool_call("fixture_greeting", serde_json::json!({}))
                .with_response("fixture turn completed"),
        ))
        .workspace_root(".")
        .tool(Arc::new(ReadOnlyGreeting))
        .approval_handler(Arc::new(DenyApproval))
        .sandbox(talos_sandbox::create_sandbox())
        .permission_rule(talos_permission::PermissionRule::new_nature(
            ToolNature::Read,
            None,
            None,
            PermissionDecision::Allow,
        ))
        .build()?;

    runtime.submit("run the fixture greeting").await?;
    let status = collect_until_turn_completed(&mut runtime)
        .await
        .ok_or_else(|| anyhow::anyhow!("fixture runtime ended before completion"))?;
    assert!(matches!(status, RuntimeTurnCompletionStatus::Success { .. }));
    let report = runtime
        .shutdown_controller()
        .shutdown(ShutdownOptions::interrupt(Duration::from_secs(1))?)
        .await?;
    assert!(report.is_complete());
    assert_eq!(
        classify_runtime_error(&RuntimeError::RuntimeClosing),
        "closing"
    );
    runtime.shutdown().await?;
    Ok(())
}

async fn run_durable_session() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let manager = SessionManager::with_dir(directory.path().to_path_buf());
    let session = manager.create_or_open_session("fixture:durable")?;
    let mut runtime = RuntimeBuilder::new()
        .provider(provider("durable fixture turn"))
        .durable_session(session)
        .build()?;
    runtime.submit("persist this fixture turn").await?;
    let status = collect_until_turn_completed(&mut runtime)
        .await
        .ok_or_else(|| anyhow::anyhow!("durable fixture ended before completion"))?;
    assert!(matches!(status, RuntimeTurnCompletionStatus::Success { .. }));
    runtime.shutdown().await?;
    Ok(())
}

fn validate_fallback_policies() {
    for policy in [
        SandboxFallbackPolicy::Deny,
        SandboxFallbackPolicy::Ask,
        SandboxFallbackPolicy::AllowUnsandboxed,
    ] {
        let builder = RuntimeBuilder::new()
            .provider(provider("fallback policy fixture"))
            .sandbox_fallback_policy(policy);
        #[cfg(feature = "coding")]
        let builder = builder.coding_preset();
        builder.build().expect("policy-only runtime should build");
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    run_minimal_runtime().await?;
    run_durable_session().await?;
    validate_fallback_policies();
    println!("talos-runtime external fixture passed");
    Ok(())
}
