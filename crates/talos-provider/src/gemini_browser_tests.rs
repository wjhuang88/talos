//! Native HTTP → live Actor → prepared browser → signed replay → durable privacy.

use super::{gemini_provider::GeminiProvider, gemini_transport::GeminiTransport};
use serde_json::{Value, json};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;
use talos_core::message::Message;
use talos_core::session::{
    RuntimePolicy, SessionConfig, SessionEvent, SessionOp, TurnCompletionStatus, TurnEventPayload,
};
use talos_core::tool::{
    AgentTool, AuthorizedToolInvocation, BrowserRawArguments, PreparedInvocationError,
    PreparedToolInvocation, ToolExecutionOutput, ToolRegistry, ToolResult,
};

const PRIVATE: &str = "browser-private-sentinel";

struct BrowserFixture(Arc<AtomicUsize>);
#[async_trait::async_trait]
impl AgentTool for BrowserFixture {
    fn name(&self) -> &str {
        "browser"
    }
    fn description(&self) -> &str {
        "prepared browser fixture"
    }
    fn parameters(&self) -> Value {
        json!({"type":"object"})
    }
    fn requires_original_invocation(&self) -> bool {
        true
    }
    fn project_input(&self, _: &Value) -> Value {
        json!({"protocolVersion":2})
    }
    async fn execute(&self, _: Value) -> ToolResult {
        panic!("legacy execution is forbidden")
    }
    async fn prepare_original_invocation(
        &self,
        args: BrowserRawArguments,
    ) -> Result<Box<dyn PreparedToolInvocation>, PreparedInvocationError> {
        assert_eq!(args.fields()["text"], PRIVATE);
        Ok(Box::new(Self(self.0.clone())))
    }
}
#[async_trait::async_trait]
impl PreparedToolInvocation for BrowserFixture {
    async fn authorize(
        self: Box<Self>,
    ) -> Result<Box<dyn AuthorizedToolInvocation>, PreparedInvocationError> {
        Ok(self)
    }
}
#[async_trait::async_trait]
impl AuthorizedToolInvocation for BrowserFixture {
    async fn execute(self: Box<Self>) -> ToolExecutionOutput {
        self.0.fetch_add(1, Ordering::SeqCst);
        ToolExecutionOutput::success("browser completed")
    }
}

fn provider(endpoint: String) -> GeminiProvider {
    GeminiProvider::new(
        GeminiTransport::ai_studio("model", "fixture-key", true)
            .expect("transport")
            .with_test_endpoint(endpoint),
        "google".into(),
        "model".into(),
        Some(1024),
        Default::default(),
    )
    .expect("provider")
}
fn agent(
    provider: GeminiProvider,
    root: std::path::PathBuf,
    effects: Arc<AtomicUsize>,
) -> talos_agent::Agent {
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(BrowserFixture(effects)));
    talos_agent::Agent::with_security(Arc::new(provider), registry, None, None, root)
        .with_reasoning_identity(Some("google".into()), Some("model".into()), true)
}
fn wire(parts: Value) -> String {
    format!(
        "data: {}\n\n",
        json!({"candidates":[{"content":{"parts":parts},"finishReason":"STOP"}]})
    )
}
async fn complete(
    rx: &mut tokio::sync::mpsc::UnboundedReceiver<SessionEvent>,
) -> TurnCompletionStatus {
    tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            let event = rx.recv().await.expect("Actor event");
            let encoded = serde_json::to_string(&event).expect("event");
            assert!(
                !encoded.contains(PRIVATE),
                "raw arguments escaped into events"
            );
            if let SessionEvent::TurnEvent {
                payload: TurnEventPayload::Completed { status },
                ..
            } = event
            {
                return status;
            }
        }
    })
    .await
    .expect("bounded turn")
}

#[tokio::test]
async fn browser_signed_replay_survives_two_live_submissions_but_not_durable_reopen() {
    let temp = tempfile::tempdir().expect("workspace");
    let root = temp.path().canonicalize().expect("root");
    let mut server = mockito::Server::new_async().await;
    let parts = json!([
        {"text":format!("thought {PRIVATE}"),"thought":true,"thoughtSignature":"private-thought-signature"},
        {"functionCall":{"id":"b","name":"browser","args":{"protocolVersion":2,"operation":"fill","text":PRIVATE}},"thoughtSignature":"private-call-signature"},
        {"text":format!("opening {PRIVATE}")}
    ]);
    let first = server
        .mock("POST", "/")
        .match_request(|r| {
            !String::from_utf8_lossy(r.body().expect("body")).contains("functionResponse")
        })
        .with_body(wire(parts.clone()))
        .expect(1)
        .create_async()
        .await;
    let expected = parts.clone();
    let second=server.mock("POST","/").match_request(move|r| {
        let body:Value=serde_json::from_slice(r.body().expect("body")).expect("JSON");
        let contents=body["contents"].as_array().expect("contents");
        contents.iter().any(|item|item["role"]=="model" && item["parts"]==expected)
            && contents.iter().any(|item|item["parts"].as_array().is_some_and(|parts|parts.iter().any(|part|part.get("functionResponse").is_some())))
            && !contents.iter().any(|item|item["parts"].as_array().is_some_and(|parts|parts.iter().any(|part|part["text"]=="again")))
    }).with_body(wire(json!([{"text":format!("done {PRIVATE}"),"thoughtSignature":"private-final-signature"}]))).expect(1).create_async().await;
    let expected = parts;
    let third = server
        .mock("POST", "/")
        .match_request(move |r| {
            let body: Value = serde_json::from_slice(r.body().expect("body")).expect("JSON");
            body["contents"]
                .as_array()
                .expect("contents")
                .iter()
                .any(|item| item["parts"] == expected)
                && body.to_string().contains("again")
                && body.to_string().contains("private-final-signature")
        })
        .with_body(wire(json!([{"text":format!("later {PRIVATE}")}])))
        .expect(1)
        .create_async()
        .await;
    let manager = talos_session::SessionManager::with_dir(root.join("sessions"));
    let session = manager
        .create_session("private-browser", "")
        .expect("session");
    let effects = Arc::new(AtomicUsize::new(0));
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: root.clone(),
        initial_history: vec![],
        model_context_limit: 128_000,
    };
    let (mut handle, mut actor) = talos_agent::session::AppServerSession::new(
        agent(provider(server.url()), root.clone(), effects.clone()),
        config,
    );
    actor.set_persistence(session.clone(), talos_session::SessionMetadata::default());
    let actor = tokio::spawn(async move {
        actor.run().await;
    });
    handle
        .sq_tx
        .send(SessionOp::Submit {
            message: "use browser".into(),
        })
        .await
        .expect("first input");
    assert!(matches!(
        complete(&mut handle.eq_rx).await,
        TurnCompletionStatus::Success { .. }
    ));
    handle
        .sq_tx
        .send(SessionOp::Submit {
            message: "again".into(),
        })
        .await
        .expect("second input");
    assert!(matches!(
        complete(&mut handle.eq_rx).await,
        TurnCompletionStatus::Success { .. }
    ));
    handle
        .sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("shutdown");
    actor.await.expect("Actor joins");
    assert_eq!(effects.load(Ordering::SeqCst), 1);
    first.assert_async().await;
    second.assert_async().await;
    third.assert_async().await;
    let entries = session.read_entries().expect("TLOG entries");
    let history = session.read_messages().expect("durable messages");
    for output in [
        String::from_utf8_lossy(&session.snapshot_bytes().expect("snapshot")).into_owned(),
        talos_session::export_json(&entries).expect("JSON export"),
        talos_session::export_markdown(&entries),
        format!("{history:?}"),
    ] {
        assert!(!output.contains(PRIVATE));
        assert!(!output.contains("private-call-signature"));
        assert!(!output.contains("private-final-signature"));
    }
    assert!(history.iter().any(|m| {
        talos_core::provider::TransientReplay::marker(m)
            .expect("marker")
            .is_some()
    }));
    let unexpected = server.mock("POST", "/").expect(0).create_async().await;
    let (mut reopened, mut actor) = talos_agent::session::AppServerSession::new(
        agent(provider(server.url()), root.clone(), effects.clone()),
        SessionConfig {
            runtime_policy: RuntimePolicy::interactive(),
            workspace_root: root,
            initial_history: history,
            model_context_limit: 128_000,
        },
    );
    let actor = tokio::spawn(async move {
        actor.run().await;
    });
    reopened
        .sq_tx
        .send(SessionOp::Submit {
            message: "resume".into(),
        })
        .await
        .expect("resume input");
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if let SessionEvent::Error { message } =
                reopened.eq_rx.recv().await.expect("reopen rejection")
            {
                assert!(message.contains("private replay"));
                break;
            }
        }
    })
    .await
    .expect("explicit missing-context refusal");
    reopened
        .sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("shutdown");
    actor.await.expect("Actor joins");
    unexpected.assert_async().await;
    assert_eq!(effects.load(Ordering::SeqCst), 1);
}
