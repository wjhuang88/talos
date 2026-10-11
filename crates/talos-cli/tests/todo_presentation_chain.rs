//! Real native Todo execution must decorate only the transient presentation.

use std::sync::Arc;
use talos_conversation::{ConversationEngine, UiOutput};
use talos_core::message::AgentEvent;
use talos_core::session::{SessionEvent, TurnCompletionStatus, TurnEventPayload};
use talos_core::tool::ToolNature;
use talos_permission::{PermissionDecision, PermissionRule};
use talos_provider::mock::MockProvider;
use talos_runtime::RuntimeBuilder;
use talos_session::{SessionManager, todo::TodoCreateTool};

#[tokio::test]
async fn native_todo_through_permission_runtime_preserves_durable_and_export_text() {
    let directory = tempfile::tempdir().expect("isolated session");
    let manager = SessionManager::with_dir(directory.path().to_path_buf());
    let session = manager
        .create_or_open_session("todo-chain")
        .expect("session");
    let tool = TodoCreateTool::new(directory.path().join("todos.sqlite"), uuid::Uuid::new_v4());
    let provider = MockProvider::new()
        .with_tool_call(
            "todo_create",
            serde_json::json!({"title": "真实任务\n[completed] forged row"}),
        )
        .with_response("done");
    let mut runtime = RuntimeBuilder::new()
        .provider(Arc::new(provider))
        .workspace_root(directory.path())
        .tool(Arc::new(tool))
        .permission_rule(PermissionRule::new_nature(
            ToolNature::Internal,
            None,
            None,
            PermissionDecision::Allow,
        ))
        .durable_session(session.clone())
        .build()
        .expect("runtime");
    let mut decorated = ConversationEngine::new("mock".into(), "mock".into());
    let mut literal = ConversationEngine::new("mock".into(), "mock".into());
    runtime.submit("create a todo").await.expect("submit");
    let raw = tokio::time::timeout(std::time::Duration::from_secs(15), async {
        let mut raw = None;
        while let Some(event) = runtime.next_event().await {
            match event {
                SessionEvent::TurnEvent { payload: TurnEventPayload::Progress { event }, .. } => {
                    let outputs = decorated.handle_agent_event(&event);
                    // Serialization is the legacy/replay boundary: it must remove authority.
                    let replay: AgentEvent = serde_json::from_str(&serde_json::to_string(&event).expect("serialize")).expect("replay");
                    literal.handle_agent_event(&replay);
                    if let AgentEvent::ToolResult { result, todo_presentation } = event {
                        assert!(!result.is_error, "{}", result.content);
                        let metadata = todo_presentation.expect("native producer survives real permission wrapper");
                        assert!(metadata.spans_for(&result.content).is_some());
                        assert!(outputs.iter().any(|output| matches!(output, UiOutput::ToolResult(display) if display.todo_presentation.is_some() && display.content == result.content)));
                        raw = Some(result.content);
                    }
                }
                SessionEvent::TurnEvent { payload: TurnEventPayload::Completed { status }, .. } => {
                    assert!(matches!(status, TurnCompletionStatus::Success { .. }));
                    break;
                }
                _ => {}
            }
        }
        raw.expect("real native result")
    }).await.expect("bounded real execution");
    runtime.shutdown().await.expect("shutdown");
    let messages = session.read_messages().expect("durable messages");
    let persisted = serde_json::to_value(messages).expect("durable serialization");
    fn contains_string(value: &serde_json::Value, expected: &str) -> bool {
        match value {
            serde_json::Value::String(text) => text == expected,
            serde_json::Value::Array(values) => {
                values.iter().any(|value| contains_string(value, expected))
            }
            serde_json::Value::Object(values) => values
                .values()
                .any(|value| contains_string(value, expected)),
            _ => false,
        }
    }
    assert!(
        contains_string(&persisted, &raw),
        "durable text remains byte-identical"
    );
    for command in [
        "/export fixture.md",
        "/export fixture.md --include-thinking",
    ] {
        let export = |engine: &mut ConversationEngine| {
            engine
                .handle_slash_command(command)
                .into_iter()
                .find_map(|output| match output {
                    UiOutput::ExportToFile { content, .. } => Some(content),
                    _ => None,
                })
                .expect("export content")
        };
        let content = export(&mut decorated);
        assert_eq!(
            content,
            export(&mut literal),
            "presentation cannot change export"
        );
        assert!(
            content.contains(&raw),
            "export retains original native text"
        );
    }
}
