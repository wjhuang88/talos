//! Regression tests for memory prompt injection (S2).
//!
//! Proves that memory section appears in provider requests when enabled
//! and is absent when disabled, using the MockProvider request-preview path.

use std::sync::Arc;

use talos_agent::Agent;
use talos_core::provider::LanguageModel;
use talos_core::tool::ToolRegistry;
use talos_permission::PermissionEngine;
use talos_provider::mock::MockProvider;

fn make_mock_provider() -> Arc<dyn LanguageModel> {
    Arc::new(MockProvider::new().with_request_debug_builder(|messages| {
        let mut snapshot = serde_json::Map::new();
        snapshot.insert(
            "messages".to_string(),
            serde_json::Value::Array(
                messages
                    .iter()
                    .map(|m| match m {
                        talos_core::message::Message::System { content, .. } => {
                            serde_json::json!({"role": "system", "content": content})
                        }
                        talos_core::message::Message::User { content } => {
                            serde_json::json!({"role": "user", "content": content})
                        }
                        talos_core::message::Message::Assistant { content, .. } => {
                            serde_json::json!({"role": "assistant", "content": content})
                        }
                        _ => serde_json::json!({"role": "other"}),
                    })
                    .collect(),
            ),
        );
        serde_json::to_string(&snapshot).unwrap_or_default()
    }))
}

fn make_agent_with_memory(
    provider: Arc<dyn LanguageModel>,
    memory_response: Option<String>,
) -> Agent {
    let mut agent = Agent::with_security(
        provider,
        ToolRegistry::new(),
        Some(Arc::new(PermissionEngine::new())),
        None,
        std::path::PathBuf::from("."),
    );

    if let Some(response) = memory_response {
        agent.set_memory_provider(Arc::new(move |_query: &str| Some(response.clone())));
    }

    agent
}

const MEMORY_CONTENT: &str = "## Relevant Memory\n- [confidence=0.9] Talos uses Rust for safety (source: session-abc, reinforced: 2025-01-01)\n";

#[tokio::test]
async fn memory_prompt_enabled_shows_in_request_preview() {
    let provider = make_mock_provider();
    let agent = make_agent_with_memory(provider, Some(MEMORY_CONTENT.to_string()));

    let result = agent
        .run("/mock-request summarize this repository".to_string())
        .await
        .expect("run should succeed");

    assert!(
        result.contains("## Relevant Memory"),
        "request preview should contain memory section header when enabled; got: {result}"
    );
    assert!(
        result.contains("Talos uses Rust for safety"),
        "request preview should contain memory content when enabled; got: {result}"
    );
    assert!(
        result.contains("confidence=0.9"),
        "request preview should contain confidence marker when enabled; got: {result}"
    );
    assert!(
        result.contains("source: session-abc"),
        "request preview should contain provenance marker when enabled; got: {result}"
    );
}

#[tokio::test]
async fn memory_prompt_disabled_absent_from_request_preview() {
    let provider = make_mock_provider();
    let agent = make_agent_with_memory(provider, None);

    let result = agent
        .run("/mock-request summarize this repository".to_string())
        .await
        .expect("run should succeed");

    assert!(
        !result.contains("## Relevant Memory"),
        "request preview should NOT contain memory section header when disabled; got: {result}"
    );
    assert!(
        !result.contains("Talos uses Rust for safety"),
        "request preview should NOT contain memory content when disabled; got: {result}"
    );
}
