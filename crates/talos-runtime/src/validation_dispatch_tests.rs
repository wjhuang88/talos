//! Provider-visible governance evidence through the shared runtime dispatcher.
use super::*;
use async_trait::async_trait;
use std::{sync::Mutex, time::Duration};
use talos_core::{
    message::{AgentEvent, Message},
    provider::{LanguageModel, ProviderResult},
};
use talos_provider::mock::MockProvider;

struct ObservedProvider {
    inner: MockProvider,
    requests: Arc<Mutex<Vec<Vec<Message>>>>,
}

#[async_trait]
impl LanguageModel for ObservedProvider {
    async fn stream(
        &self,
        messages: &[Message],
    ) -> ProviderResult<tokio::sync::mpsc::Receiver<AgentEvent>> {
        self.requests
            .lock()
            .expect("requests")
            .push(messages.to_vec());
        self.inner.stream(messages).await
    }
}

#[tokio::test]
async fn governance_dispatch_returns_provider_evidence() {
    let root = tempfile::tempdir().expect("workspace");
    std::fs::create_dir(root.path().join(".agent-governance")).expect("directory");
    std::fs::write(
        root.path().join(".agent-governance/manifest.yaml"),
        "profile: minimal\nstatus: adopting\n",
    )
    .expect("manifest");
    let requests = Arc::new(Mutex::new(Vec::new()));
    let provider = Arc::new(ObservedProvider {
        inner: MockProvider::new()
            .with_tool_call(
                "validate_project",
                serde_json::json!({"mode":"run","profile":"governance"}),
            )
            .with_response("validation observed"),
        requests: requests.clone(),
    });
    let mut runtime = RuntimeBuilder::new()
        .provider(provider)
        .workspace_root(root.path())
        .shared_tools()
        .permission_rule(PermissionRule::new_nature(
            talos_core::tool::ToolNature::Execute,
            None,
            None,
            PermissionDecision::Deny("host execution disabled".into()),
        ))
        .sandbox_fallback_policy(SandboxFallbackPolicy::AllowUnsandboxed)
        .build()
        .expect("runtime");
    runtime
        .submit("run fixed workspace validation")
        .await
        .expect("submit");
    let result = tokio::time::timeout(Duration::from_secs(20), async {
        let mut result = None;
        while let Some(event) = runtime.next_event().await {
            match event {
                SessionEvent::TurnEvent {
                    payload:
                        talos_core::session::TurnEventPayload::Progress {
                            event:
                                AgentEvent::ToolResult {
                                    result: observed, ..
                                },
                        },
                    ..
                } => result = Some(observed),
                SessionEvent::TurnEvent {
                    payload: talos_core::session::TurnEventPayload::Completed { .. },
                    ..
                } => break,
                _ => {}
            }
        }
        result.expect("tool result")
    })
    .await
    .expect("bounded turn");
    let evidence: serde_json::Value =
        serde_json::from_str(&result.content).expect("structured evidence");
    let records = evidence["records"].as_array().expect("records");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["execution_mode"], "internal");
    assert!(!root.path().join("target").exists());
    let serialized =
        serde_json::to_string(&*requests.lock().expect("requests")).expect("requests JSON");
    assert!(
        serialized.contains("internal:governance_validation"),
        "provider continuation must receive evidence"
    );
    runtime.shutdown().await.expect("owned cleanup");
}
