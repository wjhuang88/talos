use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use talos_agent::Agent;
use talos_agent::session::AppServerSession;
use talos_conversation::{
    ContentOutput, ConversationEngine, MessageSource, ModelInfo, SteeringQueueSnapshot, UiOutput,
    UserInput,
};
use talos_core::message::{AgentEvent, Message};
use talos_core::provider::{LanguageModel, ProviderResult};
use talos_core::session::{
    PendingSubmissionState, RuntimePolicy, SessionConfig, SessionOp,
};
use talos_core::tool::ToolRegistry;
use talos_session::{PendingSubmissionStore, PersistencePolicy, SessionManager};
use tokio::sync::mpsc;

use crate::tui_bridge::{ConversationLoopIo, SessionLifecycleRequest, run_conversation_loop};

struct CountingModel {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl LanguageModel for CountingModel {
    async fn stream(&self, _messages: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let (_tx, rx) = mpsc::channel(1);
        Ok(rx)
    }
}

#[tokio::test]
async fn bridge_and_actor_retain_durable_custody_when_request_plan_exceeds_budget() {
    let temp = tempfile::tempdir().unwrap();
    let manager = SessionManager::with_dir(temp.path().join("sessions"));
    let durable = manager
        .create_or_open_session("i169-bridge-budget")
        .expect("durable session");
    let session_id = durable.id().to_string();
    let pending_store =
        PendingSubmissionStore::for_session_file(durable.file_path(), &session_id);

    let provider_calls = Arc::new(AtomicUsize::new(0));
    #[allow(deprecated)]
    let agent = Agent::new(
        Arc::new(CountingModel {
            calls: provider_calls.clone(),
        }),
        ToolRegistry::new(),
    );
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: temp.path().to_path_buf(),
        initial_history: Vec::new(),
        model_context_limit: 64,
    };
    let (session_handle, mut actor) = AppServerSession::new(agent, config);
    actor.set_durable_persistence(durable, PersistencePolicy::default());
    let actor_sq_tx = session_handle.sq_tx.clone();
    let actor_task = tokio::spawn(async move { actor.run().await });

    let engine = ConversationEngine::new("budget-model".into(), "test-provider".into());
    let (user_tx, user_rx) = mpsc::unbounded_channel();
    let (ui_tx, mut ui_rx) = mpsc::unbounded_channel();
    let (_sq_watch_tx, sq_watch_rx) = tokio::sync::watch::channel(session_handle.sq_tx);
    let (_model_tx, model_rx) = tokio::sync::watch::channel(ModelInfo {
        model_name: "budget-model".into(),
        provider: "test-provider".into(),
        context_limit: Some(64),
        ..Default::default()
    });
    let (session_tx, _session_rx) =
        mpsc::unbounded_channel::<SessionLifecycleRequest>();
    let skills_dir = tempfile::tempdir().unwrap();
    let runtime_skills = Arc::new(tokio::sync::Mutex::new(
        crate::skill_runtime::discover_runtime_skills(skills_dir.path(), false).unwrap(),
    ));

    let bridge_task = tokio::spawn(run_conversation_loop(
        engine,
        ConversationLoopIo {
            agent_rx: session_handle.eq_rx,
            user_rx,
            ui_tx,
            sq_tx_watch: sq_watch_rx,
            model_info_watch: model_rx,
            session_tx,
            runtime_skills,
            permission_engine: None,
        },
    ));

    user_tx
        .send(UserInput::Message("preserve this exact user input".into()))
        .unwrap();

    let mut last_snapshot = SteeringQueueSnapshot::default();
    let pause_message = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match ui_rx.recv().await {
                Some(UiOutput::SteeringQueueSnapshot(snapshot)) => last_snapshot = snapshot,
                Some(UiOutput::Content(ContentOutput::Block {
                    source: MessageSource::Error,
                    text,
                })) if text.contains("paused before Provider start") => break text,
                Some(_) => {}
                None => panic!("bridge UI channel closed before durable pause"),
            }
        }
    })
    .await
    .expect("Bridge must observe Actor pre-Provider pause");

    assert!(pause_message.contains("ContextBudgetExceeded"));
    assert!(pause_message.contains("durable custody was retained"));
    assert_eq!(
        last_snapshot.total_count, 0,
        "Engine reservation must be removed only after Actor durable custody"
    );
    assert_eq!(provider_calls.load(Ordering::SeqCst), 0);

    let recovered = pending_store
        .recover_unstarted()
        .expect("recover durable paused submission");
    assert_eq!(recovered.len(), 1);
    assert_eq!(recovered[0].state, PendingSubmissionState::PausedPending);
    assert_eq!(recovered[0].submission.items.len(), 1);
    assert_eq!(
        recovered[0].submission.items[0].text,
        "preserve this exact user input"
    );

    user_tx.send(UserInput::Exit).unwrap();
    tokio::time::timeout(Duration::from_secs(2), bridge_task)
        .await
        .expect("bridge exit timeout")
        .expect("bridge task");
    actor_sq_tx.send(SessionOp::Shutdown).await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), actor_task)
        .await
        .expect("Actor shutdown timeout")
        .expect("Actor task");
    assert_eq!(provider_calls.load(Ordering::SeqCst), 0);
}
