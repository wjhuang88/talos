use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
mod api_surface;
mod provider;
mod safety;
use provider::FixtureProvider as MockProvider;
use talos_runtime::{
    AgentTool, ApprovalChoice, ApprovalHandler, LanguageModel, PermissionDecision, PermissionRule,
    RuntimeBuilder, RuntimeError, RuntimeTurnCompletionStatus, SandboxFallbackPolicy,
    SessionManager, ShutdownOptions, ToolNature, ToolProtocol, ToolResult,
    collect_until_turn_completed, create_sandbox,
};

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
        .sandbox(create_sandbox())
        .permission_rule(PermissionRule::new_nature(
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
    assert!(matches!(
        status,
        RuntimeTurnCompletionStatus::Success { .. }
    ));
    let report = runtime
        .shutdown_controller()
        .shutdown(ShutdownOptions::interrupt(Duration::from_secs(1))?)
        .await?;
    assert!(report.is_complete());
    assert_eq!(report.finalizers().len(), 2);
    let finalizer_ids: Vec<_> = report
        .finalizers()
        .iter()
        .map(|finalizer| finalizer.identifier().as_str())
        .collect();
    assert!(finalizer_ids.contains(&"sandbox_commands"));
    assert!(finalizer_ids.contains(&"background_jobs"));
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
    assert!(matches!(
        status,
        RuntimeTurnCompletionStatus::Success { .. }
    ));
    runtime.shutdown().await?;
    let reopened = manager.create_or_open_session("fixture:durable")?;
    let messages = reopened.read_messages()?;
    assert!(serde_json::to_string(&messages)?.contains("durable fixture turn"));
    Ok(())
}

#[cfg(feature = "coding")]
async fn run_coding_preset() -> Result<()> {
    let directory = tempfile::tempdir()?;
    std::fs::write(directory.path().join("note.txt"), "sdk-coding-sentinel")?;
    let mut runtime = RuntimeBuilder::new()
        .workspace_root(directory.path())
        .coding_preset()
        .provider(Arc::new(
            MockProvider::new()
                .with_tool_call("read", serde_json::json!({"path":"note.txt"}))
                .with_response("coding done"),
        ))
        .build()?;
    runtime.submit("read the fixture file").await?;
    let status = collect_until_turn_completed(&mut runtime)
        .await
        .ok_or_else(|| anyhow::anyhow!("coding fixture ended early"))?;
    match status {
        RuntimeTurnCompletionStatus::Success { new_messages, .. } => {
            assert!(serde_json::to_string(&new_messages)?.contains("sdk-coding-sentinel"));
        }
        other => anyhow::bail!("coding fixture failed: {other:?}"),
    }
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
    tokio::time::timeout(Duration::from_secs(30), run()).await??;
    Ok(())
}

async fn run() -> Result<()> {
    api_surface::verify();
    verify_legacy_content_surface();
    verify_ephemeral_default_refusal().await?;
    verify_browser_is_opt_in().await?;
    run_minimal_runtime().await?;
    run_durable_session().await?;
    #[cfg(feature = "coding")]
    run_coding_preset().await?;
    validate_fallback_policies();
    safety::run().await?;
    println!("talos-runtime external fixture passed");
    Ok(())
}

struct InventoryProvider(MockProvider);

#[async_trait]
impl LanguageModel for InventoryProvider {
    async fn stream(
        &self,
        messages: &[talos_runtime::Message],
    ) -> talos_runtime::ProviderResult<talos_runtime::Receiver<talos_runtime::AgentEvent>> {
        self.0.stream(messages).await
    }

    async fn stream_with_tools(
        &self,
        messages: &[talos_runtime::Message],
        tools: &[talos_runtime::ToolDefinition],
    ) -> talos_runtime::ProviderResult<talos_runtime::Receiver<talos_runtime::AgentEvent>> {
        assert!(!tools.iter().any(|tool| tool.name == "browser"));
        self.0.stream(messages).await
    }
}

async fn verify_browser_is_opt_in() -> Result<()> {
    let builder = RuntimeBuilder::new().provider(Arc::new(InventoryProvider(
        MockProvider::new().with_response("default inventory excludes browser"),
    )));
    #[cfg(feature = "coding")]
    let builder = builder.coding_preset();
    let mut runtime = builder.build()?;
    runtime.submit("inventory probe").await?;
    assert!(matches!(
        collect_until_turn_completed(&mut runtime).await,
        Some(RuntimeTurnCompletionStatus::Success { .. })
    ));
    runtime.shutdown().await?;
    Ok(())
}

fn verify_legacy_content_surface() {
    // Downstream exhaustive matches and struct literals must remain source-compatible.
    let part = talos_runtime::ContentPart::Text {
        text: "legacy".into(),
    };
    let text = match part {
        talos_runtime::ContentPart::Text { text } => text,
        talos_runtime::ContentPart::Image {
            path,
            mime,
            byte_count,
            content_digest,
        } => {
            format!("{path:?}{mime}{byte_count}{content_digest:?}")
        }
    };
    let output = talos_runtime::ToolExecutionOutput {
        result: ToolResult::success(text),
        next_provider_parts: vec![],
    };
    let prepared: talos_runtime::PreparedExecutionOutput = output.into();
    assert!(prepared.images.is_empty());
}

async fn verify_ephemeral_default_refusal() -> Result<()> {
    let provider = MockProvider::new().with_response("not consumed by refusal");
    let image = talos_runtime::EphemeralImage::png(
        vec![137, 80, 78, 71, 13, 10, 26, 10],
        std::time::Instant::now() + Duration::from_secs(10),
    )
    .expect("valid carrier header");
    let (progress, _) = tokio::sync::mpsc::unbounded_channel();
    assert!(
        provider
            .stream_with_ephemeral_images(
                &[],
                &[],
                ToolProtocol::Native,
                progress,
                &[],
                vec![image],
            )
            .await
            .is_err()
    );
    // The legacy provider queue remains untouched: refusal happened before dispatch.
    assert!(provider.stream(&[]).await.is_ok());
    Ok(())
}
