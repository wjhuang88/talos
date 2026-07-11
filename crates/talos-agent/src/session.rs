//! AppServerSession actor — bridges SQ→Agent→EQ (ADR-005 L2 seam).
//!
//! The session actor owns an [`Agent`] and runs a message loop:
//! - Receives [`SessionOp`] on the bounded SQ (cap=512)
//! - Drives agent turns via [`Agent::run_streaming`]
//! - Emits [`SessionEvent`] on the unbounded EQ

use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use futures_util::FutureExt;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use talos_core::message::{AgentEvent, Message};
use talos_core::session::{
    SessionConfig, SessionEvent, SessionHandle, SessionOp, TurnEventPayload,
};

use crate::compaction::Compactor;
use crate::token::TokenEstimator;
use crate::{ActivatedSkillContext, Agent};

mod turn;

#[cfg(test)]
#[allow(warnings)]
mod tests;

use turn::{TurnForwarding, TurnPersistence, TurnRecord, run_turn_with_forwarding};

static NEXT_RUNTIME_SESSION_ID: AtomicU64 = AtomicU64::new(1);

/// Session actor that owns an [`Agent`] and processes commands from the SQ.
///
/// Created via [`AppServerSession::new`], which returns a [`SessionHandle`]
/// for the UI layer and the actor itself for spawning on a tokio task.
pub struct AppServerSession {
    agent: Arc<Agent>,
    sq_rx: tokio::sync::mpsc::Receiver<SessionOp>,
    eq_tx: mpsc::UnboundedSender<SessionEvent>,
    history: Vec<Message>,
    compactor: Compactor,
    session_file: Option<PathBuf>,
    session_dir: Option<PathBuf>,
    persistence: Option<TurnPersistence>,
    session_id: String,
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
        let (eq_tx, eq_rx) = mpsc::unbounded_channel();

        let handle = SessionHandle { sq_tx, eq_rx };

        let compactor = Compactor::new(TokenEstimator::new(), config.model_context_limit);

        let actor = Self {
            agent: Arc::new(agent),
            sq_rx,
            eq_tx,
            history: config.initial_history,
            compactor,
            session_file: None,
            session_dir: None,
            persistence: None,
            session_id: format!(
                "runtime_{}_{}",
                std::process::id(),
                NEXT_RUNTIME_SESSION_ID.fetch_add(1, Ordering::Relaxed)
            ),
        };

        (handle, actor)
    }

    pub fn set_session_paths(&mut self, file: PathBuf, dir: PathBuf) {
        self.session_file = Some(file);
        self.session_dir = Some(dir);
    }

    /// Assigns the durable session that owns all successful turn-message writes.
    pub fn set_persistence(
        &mut self,
        session: talos_session::Session,
        metadata: talos_session::SessionMetadata,
    ) {
        self.session_id = session.id.to_string();
        self.persistence = Some(TurnPersistence { session, metadata });
    }

    /// Runs the session actor loop until shutdown or SQ disconnect.
    ///
    /// For each [`SessionOp::Submit`], spawns a turn task that:
    /// 1. Emits [`TurnEventPayload::Started`]
    /// 2. Calls `agent.run_streaming()` with an internal mpsc channel
    /// 3. Forwards `AgentEvent`s as ordered [`TurnEventPayload::Progress`] on the EQ
    /// 4. Emits [`TurnEventPayload::Completed`] on finish
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

                    let _ = self.eq_tx.send(SessionEvent::TurnEvent {
                        session_id: self.session_id.clone(),
                        turn_id: turn_id.clone(),
                        sequence: 0,
                        payload: TurnEventPayload::Started,
                    });
                    let sequence = Arc::new(AtomicU64::new(1));

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

                        let compacted = match self
                            .compactor
                            .compact(compacted, self.agent.provider())
                            .await
                        {
                            Ok(c) => c,
                            Err(_) => {
                                let (c, _) =
                                    self.compactor.compact_deterministic(self.history.clone());
                                c
                            }
                        };

                        if let (Some(file), Some(dir)) = (&self.session_file, &self.session_dir) {
                            let _ = self.try_archive_session(file, dir, &compacted);
                        }

                        self.history = compacted;
                    }

                    let agent = self.agent.clone();
                    let eq_tx = self.eq_tx.clone();
                    let turn_id_clone = turn_id.clone();
                    let token_clone = token.clone();
                    let history = self.history.clone();
                    let persistence = self.persistence.clone();
                    let session_id = self.session_id.clone();

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
                            session_id,
                            sequence,
                            persistence,
                            result_tx,
                        }))
                        .catch_unwind()
                        .await;

                        result_rx.await.ok()
                    });

                    current_turn = Some(handle);
                }
                SessionOp::PreviewRequest { message } => {
                    if let Some(token) = cancel_token.take() {
                        token.cancel();
                    }
                    if let Some(handle) = current_turn.take() {
                        self.commit_finished_turn(handle).await;
                    }

                    turn_counter += 1;
                    let turn_id = format!("turn_{turn_counter}");

                    let _ = self.eq_tx.send(SessionEvent::TurnEvent {
                        session_id: self.session_id.clone(),
                        turn_id: turn_id.clone(),
                        sequence: 0,
                        payload: TurnEventPayload::Started,
                    });
                    let mut sequence = 1;

                    match self
                        .agent
                        .preview_request(message, self.history.clone())
                        .await
                    {
                        Ok(Some(preview)) => {
                            let _ = self.eq_tx.send(SessionEvent::TurnEvent {
                                session_id: self.session_id.clone(),
                                turn_id: turn_id.clone(),
                                sequence,
                                payload: TurnEventPayload::Progress {
                                    event: AgentEvent::TurnStart,
                                },
                            });
                            sequence += 1;
                            let _ = self.eq_tx.send(SessionEvent::TurnEvent {
                                session_id: self.session_id.clone(),
                                turn_id: turn_id.clone(),
                                sequence,
                                payload: TurnEventPayload::Progress {
                                    event: AgentEvent::TextDelta {
                                        delta: preview.clone(),
                                    },
                                },
                            });
                            sequence += 1;
                            let _ = self.eq_tx.send(SessionEvent::TurnEvent {
                                session_id: self.session_id.clone(),
                                turn_id: turn_id.clone(),
                                sequence,
                                payload: TurnEventPayload::Progress {
                                    event: AgentEvent::TurnEnd {
                                        stop_reason: talos_core::message::StopReason::EndTurn,
                                        usage: talos_core::message::Usage::default(),
                                    },
                                },
                            });
                            sequence += 1;
                            let _ = self.eq_tx.send(SessionEvent::TurnEvent {
                                session_id: self.session_id.clone(),
                                turn_id,
                                sequence,
                                payload: TurnEventPayload::Completed {
                                    status: talos_core::session::TurnCompletionStatus::Success {
                                        final_text: preview,
                                        new_messages: vec![],
                                    },
                                },
                            });
                        }
                        Ok(None) => {
                            let _ = self.eq_tx.send(SessionEvent::TurnEvent {
                                session_id: self.session_id.clone(),
                                turn_id,
                                sequence,
                                payload: TurnEventPayload::Completed {
                                    status: talos_core::session::TurnCompletionStatus::Error {
                                        message: "request preview is unavailable for this provider"
                                            .into(),
                                    },
                                },
                            });
                        }
                        Err(error) => {
                            let _ = self.eq_tx.send(SessionEvent::TurnEvent {
                                session_id: self.session_id.clone(),
                                turn_id,
                                sequence,
                                payload: TurnEventPayload::Completed {
                                    status: talos_core::session::TurnCompletionStatus::Error {
                                        message: error.to_string(),
                                    },
                                },
                            });
                        }
                    }
                }
                SessionOp::Interrupt => {
                    if let Some(token) = cancel_token.take() {
                        token.cancel();
                    }
                    if let Some(handle) = current_turn.take() {
                        self.commit_finished_turn(handle).await;
                    }
                }
                SessionOp::SetSkillContext { name, content } => {
                    if current_turn.is_some() {
                        let _ = self.eq_tx.send(SessionEvent::Error {
                            message: "cannot change active skill while a turn is active".into(),
                        });
                        continue;
                    }
                    let context = match (name, content) {
                        (Some(name), Some(content)) => {
                            Some(ActivatedSkillContext { name, content })
                        }
                        _ => None,
                    };
                    if let Some(agent_mut) = Arc::get_mut(&mut self.agent) {
                        agent_mut.set_activated_skill_context(context);
                    } else {
                        let _ = self.eq_tx.send(SessionEvent::Error {
                            message: "cannot change active skill while agent is busy".into(),
                        });
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

    fn try_archive_session(
        &self,
        file: &Path,
        dir: &Path,
        _compacted: &[Message],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use talos_session::CompactTextSessionStore;
        use talos_session::compaction_engine::CompactionEngine;

        let store = std::sync::Arc::new(CompactTextSessionStore);
        let engine = CompactionEngine::new(store);

        if !engine.should_compact(file, 0) {
            return Ok(());
        }

        match engine.compact_segment(file, dir, 0)? {
            talos_session::compaction_engine::CompactionResult::Compacted {
                segment_id,
                original_count,
                ..
            } => {
                let _ = self.eq_tx.send(SessionEvent::Error {
                    message: format!(
                        "Session compacted: {original_count} entries archived to {segment_id}"
                    ),
                });
            }
            talos_session::compaction_engine::CompactionResult::Skipped => {}
        }

        Ok(())
    }
}
