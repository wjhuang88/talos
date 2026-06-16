//! AppServerSession actor — bridges SQ→Agent→EQ (ADR-005 L2 seam).
//!
//! The session actor owns an [`Agent`] and runs a message loop:
//! - Receives [`SessionOp`] on the bounded SQ (cap=512)
//! - Drives agent turns via [`Agent::run_streaming`]
//! - Emits [`SessionEvent`] on the unbounded EQ

use std::panic::AssertUnwindSafe;
use std::sync::Arc;

use futures_util::FutureExt;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::error;

use talos_core::message::AgentEvent;
use talos_core::message::Message;
use talos_core::session::{
    SessionConfig, SessionEvent, SessionHandle, SessionOp, TurnCompletionStatus,
};

use crate::Agent;
use crate::compaction::Compactor;
use crate::token::TokenEstimator;

struct TurnRecord {
    new_messages: Vec<Message>,
}

/// Session actor that owns an [`Agent`] and processes commands from the SQ.
///
/// Created via [`AppServerSession::new`], which returns a [`SessionHandle`]
/// for the UI layer and the actor itself for spawning on a tokio task.
pub struct AppServerSession {
    agent: Arc<Agent>,
    sq_rx: tokio::sync::mpsc::Receiver<SessionOp>,
    eq_tx: tokio::sync::mpsc::UnboundedSender<SessionEvent>,
    history: Vec<Message>,
    compactor: Compactor,
}

impl AppServerSession {
    /// Creates a new session actor with the given agent and configuration.
    ///
    /// Returns a [`SessionHandle`] (for the UI to send commands and receive events)
    /// and the actor itself (to be spawned on a tokio task via [`AppServerSession::run`]).
    ///
    /// The SQ channel has a bounded capacity of 512; the EQ is unbounded.
    pub fn new(agent: Agent, config: SessionConfig) -> (SessionHandle, Self) {
        let (sq_tx, sq_rx) = tokio::sync::mpsc::channel(512);
        let (eq_tx, eq_rx) = tokio::sync::mpsc::unbounded_channel();

        let handle = SessionHandle { sq_tx, eq_rx };

        let compactor = Compactor::new(TokenEstimator::new(), config.model_context_limit);

        let actor = Self {
            agent: Arc::new(agent),
            sq_rx,
            eq_tx,
            history: config.initial_history,
            compactor,
        };

        (handle, actor)
    }

    /// Runs the session actor loop until shutdown or SQ disconnect.
    ///
    /// For each [`SessionOp::Submit`], spawns a turn task that:
    /// 1. Emits [`SessionEvent::TurnStarted`]
    /// 2. Calls `agent.run_streaming()` with an internal mpsc channel
    /// 3. Forwards `AgentEvent`s as `SessionEvent::AgentEvent` on the EQ
    /// 4. Emits [`SessionEvent::TurnCompleted`] on finish
    ///
    /// [`SessionOp::Interrupt`] cancels the current turn.
    /// [`SessionOp::Shutdown`] exits the loop.
    pub async fn run(&mut self) {
        let mut turn_counter: u64 = 0;
        let mut current_turn: Option<JoinHandle<Option<TurnRecord>>> = None;
        let mut cancel_token: Option<CancellationToken> = None;

        while let Some(op) = self.sq_rx.recv().await {
            match op {
                SessionOp::Submit { message } => {
                    if let Some(token) = cancel_token.take() {
                        token.cancel();
                    }
                    if let Some(handle) = current_turn.take() {
                        self.commit_finished_turn(handle).await;
                    }

                    turn_counter += 1;
                    let turn_id = format!("turn_{turn_counter}");

                    let _ = self.eq_tx.send(SessionEvent::TurnStarted {
                        turn_id: turn_id.clone(),
                    });

                    let token = CancellationToken::new();
                    cancel_token = Some(token.clone());

                    if let Some(agent_mut) = Arc::get_mut(&mut self.agent) {
                        agent_mut.set_append_prompt_opt(None);
                    }

                    // Apply compaction layers 1-3 before the turn if needed.
                    if self.compactor.should_compact(&self.history) {
                        let compacted = self.compactor.apply_budget(self.history.clone());
                        let compacted = self.compactor.apply_trim(compacted);
                        let compacted = self.compactor.apply_microcompact(compacted);
                        self.history = compacted;
                    }

                    let agent = self.agent.clone();
                    let eq_tx = self.eq_tx.clone();
                    let turn_id_clone = turn_id.clone();
                    let token_clone = token.clone();
                    let history = self.history.clone();

                    let handle = tokio::spawn(async move {
                        let (event_tx, event_rx) = mpsc::unbounded_channel::<AgentEvent>();
                        let (result_tx, result_rx) = tokio::sync::oneshot::channel::<TurnRecord>();

                        let _ = AssertUnwindSafe(run_turn_with_forwarding(TurnForwarding {
                            agent,
                            message,
                            history,
                            event_tx,
                            event_rx,
                            eq_tx,
                            cancel_token: token_clone,
                            turn_id: turn_id_clone,
                            result_tx,
                        }))
                        .catch_unwind()
                        .await;

                        result_rx.await.ok()
                    });

                    current_turn = Some(handle);
                }
                SessionOp::Interrupt => {
                    if let Some(token) = cancel_token.take() {
                        token.cancel();
                    }
                    if let Some(handle) = current_turn.take() {
                        self.commit_finished_turn(handle).await;
                    }
                }
                SessionOp::Shutdown => {
                    if let Some(handle) = current_turn.take() {
                        self.commit_finished_turn(handle).await;
                    }
                    break;
                }
            }
        }
    }

    async fn commit_finished_turn(&mut self, handle: JoinHandle<Option<TurnRecord>>) {
        let Some(record) = handle.await.ok().flatten() else {
            return;
        };

        for msg in record.new_messages {
            self.history.push(msg);
        }
    }
}

struct TurnForwarding {
    agent: Arc<Agent>,
    message: String,
    history: Vec<talos_core::message::Message>,
    event_tx: mpsc::UnboundedSender<AgentEvent>,
    event_rx: mpsc::UnboundedReceiver<AgentEvent>,
    eq_tx: mpsc::UnboundedSender<SessionEvent>,
    cancel_token: CancellationToken,
    turn_id: String,
    result_tx: tokio::sync::oneshot::Sender<TurnRecord>,
}

async fn run_turn_with_forwarding(turn: TurnForwarding) {
    let TurnForwarding {
        agent,
        message,
        history,
        event_tx,
        mut event_rx,
        eq_tx,
        cancel_token,
        turn_id,
        result_tx,
    } = turn;

    let eq_tx_clone = eq_tx.clone();
    let cancel_clone = cancel_token.clone();

    let forwarder = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = cancel_clone.cancelled() => break,
                event = event_rx.recv() => {
                    match event {
                        Some(event) => {
                            let _ = eq_tx.send(SessionEvent::AgentEvent(event));
                        }
                        None => break,
                    }
                }
            }
        }
    });

    let mut agent_task =
        tokio::spawn(async move { agent.run_streaming(message, history, event_tx).await });

    let agent_result = tokio::select! {
        result = &mut agent_task => result,
        _ = cancel_token.cancelled() => {
            agent_task.abort();
            let _ = forwarder.await;
            let _ = eq_tx_clone.send(SessionEvent::TurnCompleted {
                turn_id,
                status: TurnCompletionStatus::Cancelled,
            });
            return;
        }
    };

    let _ = forwarder.await;

    match agent_result {
        Ok(Ok((final_text, new_messages))) => {
            let _ = eq_tx_clone.send(SessionEvent::TurnCompleted {
                turn_id,
                status: TurnCompletionStatus::Success {
                    final_text: final_text.clone(),
                },
            });
            let _ = result_tx.send(TurnRecord { new_messages });
        }
        Ok(Err(e)) => {
            let _ = eq_tx_clone.send(SessionEvent::TurnCompleted {
                turn_id,
                status: TurnCompletionStatus::Error {
                    message: e.to_string(),
                },
            });
        }
        Err(_join_error) => {
            error!("agent panicked during turn");
            let _ = eq_tx_clone.send(SessionEvent::TurnCompleted {
                turn_id,
                status: TurnCompletionStatus::Error {
                    message: "agent panicked".into(),
                },
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use talos_core::message::{Message, StopReason};
    use talos_core::provider::{LanguageModel, ProviderResult};
    use talos_core::tool::ToolRegistry;
    use tokio::sync::mpsc;

    type Receiver<T> = mpsc::Receiver<T>;

    struct MockModel {
        responses: Arc<Mutex<VecDeque<Vec<AgentEvent>>>>,
    }

    impl MockModel {
        fn new(responses: Vec<Vec<AgentEvent>>) -> Self {
            Self {
                responses: Arc::new(Mutex::new(VecDeque::from(responses))),
            }
        }
    }

    #[async_trait]
    impl LanguageModel for MockModel {
        async fn stream(&self, _messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
            let (tx, rx) = mpsc::channel(64);
            let events = {
                let mut responses = self.responses.lock().unwrap();
                responses.pop_front().unwrap_or_default()
            };
            tokio::spawn(async move {
                for event in events {
                    let _ = tx.send(event).await;
                }
            });
            Ok(rx)
        }
    }

    struct SlowModel {
        delay: Duration,
        events: Vec<AgentEvent>,
    }

    #[async_trait]
    impl LanguageModel for SlowModel {
        async fn stream(&self, _messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
            let (tx, rx) = mpsc::channel(64);
            let events = self.events.clone();
            let delay = self.delay;
            tokio::spawn(async move {
                tokio::time::sleep(delay).await;
                for event in events {
                    let _ = tx.send(event).await;
                }
            });
            Ok(rx)
        }
    }

    struct PanicModel;

    #[async_trait]
    impl LanguageModel for PanicModel {
        async fn stream(&self, _messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
            panic!("intentional panic for testing");
        }
    }

    fn make_agent(model: impl LanguageModel + 'static) -> Agent {
        #[allow(deprecated)]
        Agent::new(Arc::new(model), ToolRegistry::new())
    }

    fn success_events(text: &str) -> Vec<AgentEvent> {
        vec![
            AgentEvent::TurnStart,
            AgentEvent::TextDelta { delta: text.into() },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: talos_core::message::Usage::default(),
            },
        ]
    }

    async fn collect_events(
        mut eq_rx: tokio::sync::mpsc::UnboundedReceiver<SessionEvent>,
        timeout: Duration,
    ) -> Vec<SessionEvent> {
        let mut events = Vec::new();
        loop {
            tokio::select! {
                event = eq_rx.recv() => {
                    match event {
                        Some(e) => events.push(e),
                        None => break,
                    }
                }
                _ = tokio::time::sleep(timeout) => break,
            }
        }
        events
    }

    #[tokio::test]
    async fn test_submit_and_receive() {
        let agent = make_agent(MockModel::new(vec![success_events("hello")]));
        let config = SessionConfig {
            print_mode: false,
            workspace_root: "/tmp".into(),
            initial_history: vec![],
            model_context_limit: 128_000,
        };
        let (handle, mut actor) = AppServerSession::new(agent, config);

        let eq_rx = handle.eq_rx;
        let sq_tx = handle.sq_tx;

        let actor_task = tokio::spawn(async move { actor.run().await });

        sq_tx
            .send(SessionOp::Submit {
                message: "hi".into(),
            })
            .await
            .unwrap();

        sq_tx.send(SessionOp::Shutdown).await.unwrap();
        let _ = actor_task.await;

        let events = collect_events(eq_rx, Duration::from_secs(2)).await;

        assert!(
            events
                .iter()
                .any(|e| matches!(e, SessionEvent::TurnStarted { .. })),
            "Should have TurnStarted"
        );
        assert!(
            events.iter().any(|e| matches!(e, SessionEvent::AgentEvent(AgentEvent::TextDelta { delta }) if delta == "hello")),
            "Should have TextDelta with 'hello'"
        );
        assert!(
            events.iter().any(|e| matches!(
                e,
                SessionEvent::TurnCompleted {
                    status: TurnCompletionStatus::Success { .. },
                    ..
                }
            )),
            "Should have TurnCompleted(Success)"
        );
    }

    #[tokio::test]
    async fn test_multi_turn() {
        let agent = make_agent(MockModel::new(vec![
            success_events("first"),
            success_events("second"),
        ]));
        let config = SessionConfig {
            print_mode: false,
            workspace_root: "/tmp".into(),
            initial_history: vec![],
            model_context_limit: 128_000,
        };
        let (handle, mut actor) = AppServerSession::new(agent, config);

        let eq_rx = handle.eq_rx;
        let sq_tx = handle.sq_tx;

        let actor_task = tokio::spawn(async move { actor.run().await });

        sq_tx
            .send(SessionOp::Submit {
                message: "hi".into(),
            })
            .await
            .unwrap();

        sq_tx
            .send(SessionOp::Submit {
                message: "again".into(),
            })
            .await
            .unwrap();

        sq_tx.send(SessionOp::Shutdown).await.unwrap();
        let _ = actor_task.await;

        let events = collect_events(eq_rx, Duration::from_secs(2)).await;

        let turn_started_count = events
            .iter()
            .filter(|e| matches!(e, SessionEvent::TurnStarted { .. }))
            .count();
        assert_eq!(turn_started_count, 2, "Should have 2 TurnStarted events");

        let success_count = events
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    SessionEvent::TurnCompleted {
                        status: TurnCompletionStatus::Success { .. },
                        ..
                    }
                )
            })
            .count();
        assert!(
            success_count >= 1,
            "Should have at least 1 TurnCompleted(Success)"
        );
    }

    #[tokio::test]
    async fn test_interrupt() {
        let slow_events = vec![
            AgentEvent::TurnStart,
            AgentEvent::TextDelta {
                delta: "slow response".into(),
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: talos_core::message::Usage::default(),
            },
        ];
        let agent = make_agent(SlowModel {
            delay: Duration::from_millis(500),
            events: slow_events,
        });
        let config = SessionConfig {
            print_mode: false,
            workspace_root: "/tmp".into(),
            initial_history: vec![],
            model_context_limit: 128_000,
        };
        let (handle, mut actor) = AppServerSession::new(agent, config);

        let eq_rx = handle.eq_rx;
        let sq_tx = handle.sq_tx;

        let actor_task = tokio::spawn(async move { actor.run().await });

        sq_tx
            .send(SessionOp::Submit {
                message: "hi".into(),
            })
            .await
            .unwrap();

        sq_tx.send(SessionOp::Interrupt).await.unwrap();

        sq_tx.send(SessionOp::Shutdown).await.unwrap();
        let _ = actor_task.await;

        let events = collect_events(eq_rx, Duration::from_secs(3)).await;

        assert!(
            events
                .iter()
                .any(|e| matches!(e, SessionEvent::TurnStarted { .. })),
            "Should have TurnStarted"
        );
        assert!(
            events.iter().any(|e| matches!(
                e,
                SessionEvent::TurnCompleted {
                    status: TurnCompletionStatus::Cancelled,
                    ..
                }
            )),
            "Should have TurnCompleted(Cancelled)"
        );
    }

    #[tokio::test]
    async fn test_shutdown() {
        let agent = make_agent(MockModel::new(vec![]));
        let config = SessionConfig {
            print_mode: false,
            workspace_root: "/tmp".into(),
            initial_history: vec![],
            model_context_limit: 128_000,
        };
        let (handle, mut actor) = AppServerSession::new(agent, config);

        let sq_tx = handle.sq_tx;

        let actor_task = tokio::spawn(async move { actor.run().await });

        sq_tx.send(SessionOp::Shutdown).await.unwrap();

        let result = tokio::time::timeout(Duration::from_secs(2), actor_task).await;
        assert!(result.is_ok(), "Actor should exit cleanly on Shutdown");
    }

    #[tokio::test]
    async fn test_eq_consumer_disconnect() {
        let agent = make_agent(MockModel::new(vec![success_events("hello")]));
        let config = SessionConfig {
            print_mode: false,
            workspace_root: "/tmp".into(),
            initial_history: vec![],
            model_context_limit: 128_000,
        };
        let (handle, mut actor) = AppServerSession::new(agent, config);

        let sq_tx = handle.sq_tx;
        drop(handle.eq_rx);

        let actor_task = tokio::spawn(async move { actor.run().await });

        sq_tx
            .send(SessionOp::Submit {
                message: "hi".into(),
            })
            .await
            .unwrap();

        sq_tx.send(SessionOp::Shutdown).await.unwrap();

        let result = tokio::time::timeout(Duration::from_secs(2), actor_task).await;
        assert!(
            result.is_ok(),
            "Actor should handle EQ disconnect gracefully"
        );
    }

    #[tokio::test]
    async fn test_sq_backpressure() {
        let agent = make_agent(MockModel::new(vec![success_events("hello")]));
        let config = SessionConfig {
            print_mode: false,
            workspace_root: "/tmp".into(),
            initial_history: vec![],
            model_context_limit: 128_000,
        };
        let (handle, _actor) = AppServerSession::new(agent, config);

        let sq_tx = handle.sq_tx;

        for _ in 0..512 {
            sq_tx
                .try_send(SessionOp::Submit {
                    message: "fill".into(),
                })
                .unwrap();
        }

        let result = sq_tx.try_send(SessionOp::Submit {
            message: "overflow".into(),
        });
        assert!(
            result.is_err(),
            "try_send should fail when SQ is at capacity"
        );
        assert!(
            matches!(
                result.unwrap_err(),
                tokio::sync::mpsc::error::TrySendError::Full(_)
            ),
            "Error should be Full, not Closed"
        );
    }

    #[tokio::test]
    async fn test_panic_recovery() {
        let agent = make_agent(PanicModel);
        let config = SessionConfig {
            print_mode: false,
            workspace_root: "/tmp".into(),
            initial_history: vec![],
            model_context_limit: 128_000,
        };
        let (handle, mut actor) = AppServerSession::new(agent, config);

        let eq_rx = handle.eq_rx;
        let sq_tx = handle.sq_tx;

        let actor_task = tokio::spawn(async move { actor.run().await });

        sq_tx
            .send(SessionOp::Submit {
                message: "panic me".into(),
            })
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(50)).await;

        sq_tx
            .send(SessionOp::Submit {
                message: "still here?".into(),
            })
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(50)).await;

        sq_tx.send(SessionOp::Shutdown).await.unwrap();
        let _ = actor_task.await;

        let events = collect_events(eq_rx, Duration::from_secs(3)).await;

        let turn_started_count = events
            .iter()
            .filter(|e| matches!(e, SessionEvent::TurnStarted { .. }))
            .count();
        assert_eq!(turn_started_count, 2, "Should have 2 TurnStarted events");

        let error_count = events
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    SessionEvent::TurnCompleted {
                        status: TurnCompletionStatus::Error { .. },
                        ..
                    }
                )
            })
            .count();
        assert_eq!(error_count, 2, "Should have 2 TurnCompleted(Error) events");
    }

    #[tokio::test]
    async fn test_concurrent_submit_and_interrupt() {
        let slow_events = vec![
            AgentEvent::TurnStart,
            AgentEvent::TextDelta {
                delta: "slow".into(),
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: talos_core::message::Usage::default(),
            },
        ];
        let agent = make_agent(SlowModel {
            delay: Duration::from_millis(500),
            events: slow_events,
        });
        let config = SessionConfig {
            print_mode: false,
            workspace_root: "/tmp".into(),
            initial_history: vec![],
            model_context_limit: 128_000,
        };
        let (handle, mut actor) = AppServerSession::new(agent, config);

        let eq_rx = handle.eq_rx;
        let sq_tx = handle.sq_tx;

        let actor_task = tokio::spawn(async move { actor.run().await });

        sq_tx
            .send(SessionOp::Submit {
                message: "slow turn".into(),
            })
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(50)).await;

        sq_tx.send(SessionOp::Interrupt).await.unwrap();

        sq_tx
            .send(SessionOp::Submit {
                message: "after interrupt".into(),
            })
            .await
            .unwrap();

        sq_tx.send(SessionOp::Shutdown).await.unwrap();
        let _ = actor_task.await;

        let events = collect_events(eq_rx, Duration::from_secs(3)).await;

        assert!(
            events
                .iter()
                .any(|e| matches!(e, SessionEvent::TurnStarted { .. })),
            "Should have TurnStarted"
        );

        assert!(
            events.iter().any(|e| matches!(
                e,
                SessionEvent::TurnCompleted {
                    status: TurnCompletionStatus::Cancelled,
                    ..
                }
            )),
            "First turn should be Cancelled"
        );
    }

    #[tokio::test]
    async fn test_multi_turn_with_history() {
        use talos_core::message::Message;

        let captured_messages = Arc::new(Mutex::new(Vec::<Vec<Message>>::new()));
        let responses = Arc::new(Mutex::new(VecDeque::from(vec![
            success_events("first response"),
            success_events("second response"),
            success_events("third response"),
        ])));
        let _captured = captured_messages.clone();

        struct CapturingModel {
            responses: Arc<Mutex<VecDeque<Vec<AgentEvent>>>>,
            captured: Arc<Mutex<Vec<Vec<Message>>>>,
        }

        #[async_trait]
        impl LanguageModel for CapturingModel {
            async fn stream(&self, messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
                self.captured.lock().unwrap().push(messages.to_vec());
                let (tx, rx) = mpsc::channel(64);
                let events = {
                    let mut responses = self.responses.lock().unwrap();
                    responses.pop_front().unwrap_or_default()
                };
                tokio::spawn(async move {
                    for event in events {
                        let _ = tx.send(event).await;
                    }
                });
                Ok(rx)
            }
        }

        let agent = make_agent(CapturingModel {
            responses,
            captured: captured_messages.clone(),
        });
        let config = SessionConfig {
            print_mode: false,
            workspace_root: "/tmp".into(),
            initial_history: vec![],
            model_context_limit: 128_000,
        };
        let (handle, mut actor) = AppServerSession::new(agent, config);

        let eq_rx = handle.eq_rx;
        let sq_tx = handle.sq_tx;

        let actor_task = tokio::spawn(async move { actor.run().await });

        // Submit 3 turns
        sq_tx
            .send(SessionOp::Submit {
                message: "turn 1".into(),
            })
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        sq_tx
            .send(SessionOp::Submit {
                message: "turn 2".into(),
            })
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        sq_tx
            .send(SessionOp::Submit {
                message: "turn 3".into(),
            })
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        sq_tx.send(SessionOp::Shutdown).await.unwrap();
        let _ = actor_task.await;

        let events = collect_events(eq_rx, Duration::from_secs(2)).await;
        let success_count = events
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    SessionEvent::TurnCompleted {
                        status: TurnCompletionStatus::Success { .. },
                        ..
                    }
                )
            })
            .count();
        assert!(success_count >= 1, "Should have at least 1 Success");

        // Verify the 3rd turn received history from turns 1 and 2
        let captured = captured_messages.lock().unwrap();
        assert!(captured.len() >= 3, "Should have captured at least 3 calls");

        // 3rd call should have messages from turns 1 and 2
        let third_call_messages = &captured[2];
        // Should have: User(turn 1), Assistant(first response), User(turn 2), Assistant(second response), User(turn 3 with system prompt)
        let user_messages: Vec<_> = third_call_messages
            .iter()
            .filter(|m| matches!(m, Message::User { .. }))
            .collect();
        assert!(
            user_messages.len() >= 3,
            "Third turn should have at least 3 user messages (turns 1, 2, 3), got {}",
            user_messages.len()
        );
    }

    #[tokio::test]
    async fn test_interrupt_after_success_preserves_history() {
        use talos_core::message::Message;

        let captured_messages = Arc::new(Mutex::new(Vec::<Vec<Message>>::new()));
        let responses = Arc::new(Mutex::new(VecDeque::from(vec![
            success_events("first response"),
            success_events("second response"),
        ])));

        struct CapturingModel {
            responses: Arc<Mutex<VecDeque<Vec<AgentEvent>>>>,
            captured: Arc<Mutex<Vec<Vec<Message>>>>,
        }

        #[async_trait]
        impl LanguageModel for CapturingModel {
            async fn stream(&self, messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
                self.captured.lock().unwrap().push(messages.to_vec());
                let (tx, rx) = mpsc::channel(64);
                let events = {
                    let mut responses = self.responses.lock().unwrap();
                    responses.pop_front().unwrap_or_default()
                };
                tokio::spawn(async move {
                    for event in events {
                        let _ = tx.send(event).await;
                    }
                });
                Ok(rx)
            }
        }

        let agent = make_agent(CapturingModel {
            responses,
            captured: captured_messages.clone(),
        });
        let config = SessionConfig {
            print_mode: false,
            workspace_root: "/tmp".into(),
            initial_history: vec![],
            model_context_limit: 128_000,
        };
        let (handle, mut actor) = AppServerSession::new(agent, config);

        let sq_tx = handle.sq_tx;
        let actor_task = tokio::spawn(async move { actor.run().await });

        sq_tx
            .send(SessionOp::Submit {
                message: "turn 1".into(),
            })
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        sq_tx.send(SessionOp::Interrupt).await.unwrap();

        sq_tx
            .send(SessionOp::Submit {
                message: "turn 2".into(),
            })
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        sq_tx.send(SessionOp::Shutdown).await.unwrap();
        let _ = actor_task.await;

        let captured = captured_messages.lock().unwrap();
        assert!(captured.len() >= 2, "Should have captured 2 calls");

        let second_call_messages = &captured[1];
        assert!(
            second_call_messages
                .iter()
                .any(|m| matches!(m, Message::User { content } if content == "turn 1")),
            "Second turn should retain first user message after interrupt"
        );
        assert!(
            second_call_messages.iter().any(
                |m| matches!(m, Message::Assistant { content, .. } if content == "first response")
            ),
            "Second turn should retain first assistant response after interrupt"
        );
    }

    #[tokio::test]
    async fn test_initial_history_from_jsonl_resume() {
        use talos_core::message::Message;
        use talos_session::SessionManager;

        let temp_dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::with_dir(temp_dir.path().to_path_buf());
        let session = manager.create_session("resume-test", "").unwrap();
        let session_id = session.id.to_string();
        session
            .append(&Message::User {
                content: "prior question".into(),
            })
            .unwrap();
        session
            .append(&Message::Assistant {
                content: "prior answer".into(),
                tool_calls: vec![],
            })
            .unwrap();
        let resumed = manager.resume_session(&session_id).unwrap();
        let prior_history = resumed.read_messages().unwrap();

        let captured_messages = Arc::new(Mutex::new(Vec::<Vec<Message>>::new()));
        let responses = Arc::new(Mutex::new(VecDeque::from(vec![success_events(
            "new response",
        )])));

        struct CapturingModel {
            responses: Arc<Mutex<VecDeque<Vec<AgentEvent>>>>,
            captured: Arc<Mutex<Vec<Vec<Message>>>>,
        }

        #[async_trait]
        impl LanguageModel for CapturingModel {
            async fn stream(&self, messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
                self.captured.lock().unwrap().push(messages.to_vec());
                let (tx, rx) = mpsc::channel(64);
                let events = {
                    let mut responses = self.responses.lock().unwrap();
                    responses.pop_front().unwrap_or_default()
                };
                tokio::spawn(async move {
                    for event in events {
                        let _ = tx.send(event).await;
                    }
                });
                Ok(rx)
            }
        }

        let agent = make_agent(CapturingModel {
            responses,
            captured: captured_messages.clone(),
        });
        let config = SessionConfig {
            print_mode: false,
            workspace_root: "/tmp".into(),
            initial_history: prior_history,
            model_context_limit: 128_000,
        };
        let (handle, mut actor) = AppServerSession::new(agent, config);
        let sq_tx = handle.sq_tx;
        let actor_task = tokio::spawn(async move { actor.run().await });

        sq_tx
            .send(SessionOp::Submit {
                message: "new question".into(),
            })
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        sq_tx.send(SessionOp::Shutdown).await.unwrap();
        let _ = actor_task.await;

        let captured = captured_messages.lock().unwrap();
        assert_eq!(captured.len(), 1, "Should have captured exactly 1 call");

        let messages = &captured[0];
        assert!(
            messages
                .iter()
                .any(|m| matches!(m, Message::User { content } if content == "prior question")),
            "Resumed session should include prior user message"
        );
        assert!(
            messages.iter().any(
                |m| matches!(m, Message::Assistant { content, .. } if content == "prior answer")
            ),
            "Resumed session should include prior assistant response"
        );
        assert!(
            messages.iter().any(
                |m| matches!(m, Message::User { content } if content.contains("new question"))
            ),
            "Resumed session should include new user message"
        );
    }
}
