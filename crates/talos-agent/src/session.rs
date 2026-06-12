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
use talos_core::session::{
    SessionConfig, SessionEvent, SessionHandle, SessionOp, TurnCompletionStatus,
};

use crate::Agent;

/// Session actor that owns an [`Agent`] and processes commands from the SQ.
///
/// Created via [`AppServerSession::new`], which returns a [`SessionHandle`]
/// for the UI layer and the actor itself for spawning on a tokio task.
pub struct AppServerSession {
    agent: Arc<Agent>,
    sq_rx: tokio::sync::mpsc::Receiver<SessionOp>,
    eq_tx: tokio::sync::mpsc::UnboundedSender<SessionEvent>,
    _config: SessionConfig,
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

        let actor = Self {
            agent: Arc::new(agent),
            sq_rx,
            eq_tx,
            _config: config,
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
        let mut current_turn: Option<JoinHandle<()>> = None;
        let mut cancel_token: Option<CancellationToken> = None;

        while let Some(op) = self.sq_rx.recv().await {
            match op {
                SessionOp::Submit { message } => {
                    if let Some(token) = cancel_token.take() {
                        token.cancel();
                    }
                    if let Some(handle) = current_turn.take() {
                        let _ = handle.await;
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

                    let agent = self.agent.clone();
                    let eq_tx = self.eq_tx.clone();
                    let turn_id_clone = turn_id.clone();
                    let token_clone = token.clone();

                    let handle = tokio::spawn(async move {
                        let (event_tx, event_rx) = mpsc::unbounded_channel::<AgentEvent>();

                        let _ = AssertUnwindSafe(run_turn_with_forwarding(
                            agent,
                            message,
                            event_tx,
                            event_rx,
                            eq_tx,
                            token_clone,
                            turn_id_clone,
                        ))
                        .catch_unwind()
                        .await;
                    });

                    current_turn = Some(handle);
                }
                SessionOp::Interrupt => {
                    if let Some(token) = cancel_token.take() {
                        token.cancel();
                    }
                    if let Some(handle) = current_turn.take() {
                        let _ = handle.await;
                    }
                }
                SessionOp::Shutdown => {
                    if let Some(handle) = current_turn.take() {
                        let _ = handle.await;
                    }
                    break;
                }
            }
        }
    }
}

async fn run_turn_with_forwarding(
    agent: Arc<Agent>,
    message: String,
    event_tx: mpsc::UnboundedSender<AgentEvent>,
    mut event_rx: mpsc::UnboundedReceiver<AgentEvent>,
    eq_tx: mpsc::UnboundedSender<SessionEvent>,
    cancel_token: CancellationToken,
    turn_id: String,
) {
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
                        None => break, // Channel closed (agent done)
                    }
                }
            }
        }
    });

    let mut agent_task = tokio::spawn(async move { agent.run_streaming(message, event_tx).await });

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
        Ok(Ok(_final_text)) => {
            let _ = eq_tx_clone.send(SessionEvent::TurnCompleted {
                turn_id,
                status: TurnCompletionStatus::Success,
            });
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
    use std::sync::Mutex;
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
                    status: TurnCompletionStatus::Success,
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
                        status: TurnCompletionStatus::Success,
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
}
