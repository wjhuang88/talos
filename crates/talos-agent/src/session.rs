//! AppServerSession actor — bridges SQ→Agent→EQ (ADR-005 L2 seam).
//!
//! The session actor owns an [`Agent`] and runs a message loop:
//! - Receives [`SessionOp`] on the bounded SQ (cap=512)
//! - Drives agent turns via [`Agent::run_streaming`]
//! - Emits [`SessionEvent`] on the unbounded EQ

use std::collections::VecDeque;
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
    MAX_SUBMISSION_BATCH_BYTES, MAX_SUBMISSION_BATCH_ITEMS, MAX_SUBMISSION_ITEM_BYTES,
    SessionConfig, SessionEvent, SessionHandle, SessionOp, StructuredSubmission, SubmissionItem,
    SubmissionKind, SubmissionRejectionReason, SubmissionSource, TurnCompletionStatus,
    TurnEventPayload,
};

use crate::compaction::Compactor;
use crate::token::TokenEstimator;
use crate::{ActivatedSkillContext, Agent};

mod turn;

#[cfg(test)]
#[allow(warnings)]
mod tests;

use turn::{
    DurableTurnPersistence, TurnForwarding, TurnPersistence, TurnRecord, TurnRecordStatus,
    run_turn_with_forwarding,
};

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
    durable_persistence: Option<DurableTurnPersistence>,
    session_id: String,
    turn_prefix: String,
    model_context_limit: u32,
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

        let instance_id = NEXT_RUNTIME_SESSION_ID.fetch_add(1, Ordering::Relaxed);
        let actor = Self {
            agent: Arc::new(agent),
            sq_rx,
            eq_tx,
            history: config.initial_history,
            compactor,
            session_file: None,
            session_dir: None,
            persistence: None,
            durable_persistence: None,
            session_id: format!("runtime_{}_{}", std::process::id(), instance_id),
            // Keep durable turn IDs unique across Runtime reconstruction. A host that
            // needs retry idempotency supplies the stable ID to `DurableSession`.
            turn_prefix: format!("turn_{}_{}", std::process::id(), instance_id),
            model_context_limit: config.model_context_limit,
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

    /// Assigns an atomic durable session used only by the embedded runtime.
    pub fn set_durable_persistence(
        &mut self,
        session: talos_session::DurableSession,
        policy: talos_session::PersistencePolicy,
    ) {
        self.session_id = session.id().to_string();
        self.durable_persistence = Some(DurableTurnPersistence { session, policy });
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
        let mut submission_counter: u64 = 0;
        let mut pending = VecDeque::<StructuredSubmission>::new();
        let mut current_turn: Option<JoinHandle<Option<TurnRecord>>> = None;
        let mut cancel_token: Option<CancellationToken> = None;
        let mut paused = false;
        let mut shutting_down = false;

        loop {
            if current_turn.is_none()
                && !paused
                && let Some(submission) = pending.pop_front()
            {
                turn_counter = turn_counter.saturating_add(1);
                let (handle, token) = self.start_submission(submission, turn_counter).await;
                current_turn = Some(handle);
                cancel_token = Some(token);
            }

            if shutting_down && current_turn.is_none() && pending.is_empty() {
                break;
            }

            tokio::select! {
                completed = async {
                    match current_turn.as_mut() {
                        Some(turn) => Some(turn.await),
                        None => None,
                    }
                }, if current_turn.is_some() => {
                    current_turn = None;
                    cancel_token = None;
                    match completed.and_then(Result::ok).flatten() {
                        Some(record) => {
                            let status = record.status;
                            self.commit_turn_record(record);
                            paused = status != TurnRecordStatus::Success;
                        }
                        None => paused = true,
                    }
                }
                op = self.sq_rx.recv(), if !shutting_down => {
                    let Some(op) = op else {
                        shutting_down = true;
                        if let Some(token) = cancel_token.take() { token.cancel(); }
                        continue;
                    };
            match op {
                SessionOp::Submit { message } => {
                    submission_counter = submission_counter.saturating_add(1);
                    pending.push_back(compatibility_submission(submission_counter, SubmissionKind::UserTurn, message, Vec::new()));
                    paused = false;
                }
                SessionOp::SubmitMultimodal { text, attachments } => {
                    submission_counter = submission_counter.saturating_add(1);
                    pending.push_back(compatibility_submission(submission_counter, SubmissionKind::UserTurn, text, attachments));
                    paused = false;
                }
                SessionOp::PreviewRequest { message } => {
                    submission_counter = submission_counter.saturating_add(1);
                    pending.push_back(compatibility_submission(submission_counter, SubmissionKind::PreviewRequest, message, Vec::new()));
                    paused = false;
                }
                SessionOp::SubmitStructured { submission } => {
                    if let Err(reason) = validate_submission(&submission) {
                        self.reject_submission(&submission.id, reason);
                        continue;
                    }
                    let _ = self.eq_tx.send(SessionEvent::SubmissionQueued {
                        session_id: self.session_id.clone(),
                        submission_id: submission.id.clone(),
                        source: submission.source,
                        item_count: submission.items.len(),
                        total_text_bytes: submission.total_text_bytes(),
                    });
                    if submission.source != SubmissionSource::Scheduler {
                        paused = false;
                    }
                    pending.push_back(submission);
                }
                SessionOp::Interrupt => {
                    if let Some(token) = &cancel_token { token.cancel(); }
                    paused = true;
                }
                SessionOp::SetSkillContext { name, content } => {
                    if current_turn.is_some() || !pending.is_empty() {
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
                    shutting_down = true;
                    paused = false;
                }
            }
                }
            }
        }
    }

    fn commit_turn_record(&mut self, record: TurnRecord) {
        for msg in record.new_messages {
            self.history.push(msg);
        }
    }

    fn reject_submission(&self, submission_id: &str, reason: SubmissionRejectionReason) {
        let _ = self.eq_tx.send(SessionEvent::SubmissionRejected {
            session_id: self.session_id.clone(),
            submission_id: submission_id.to_owned(),
            reason,
        });
    }

    async fn start_submission(
        &mut self,
        submission: StructuredSubmission,
        turn_counter: u64,
    ) -> (JoinHandle<Option<TurnRecord>>, CancellationToken) {
        if self.compactor.should_compact(&self.history) {
            let compacted = self.compactor.apply_budget(self.history.clone());
            let compacted = self.compactor.apply_trim(compacted);
            let compacted = self.compactor.apply_microcompact(compacted);
            self.history = match self
                .compactor
                .compact(compacted, self.agent.provider())
                .await
            {
                Ok(history) => history,
                Err(_) => self.compactor.compact_deterministic(self.history.clone()).0,
            };
            if let (Some(file), Some(dir)) = (&self.session_file, &self.session_dir) {
                let _ = self.try_archive_session(file, dir, &self.history);
            }
        }

        let input_messages: Vec<Message> =
            submission.items.iter().map(submission_message).collect();
        let mut projected = self.history.clone();
        projected.extend(input_messages);
        if TokenEstimator::new().estimate(&projected) > self.model_context_limit {
            self.reject_submission(
                &submission.id,
                SubmissionRejectionReason::ContextBudgetExceeded,
            );
            return (
                tokio::spawn(async {
                    Some(TurnRecord {
                        new_messages: Vec::new(),
                        status: TurnRecordStatus::Error,
                    })
                }),
                CancellationToken::new(),
            );
        }

        let turn_id = format!("{}_{}", self.turn_prefix, turn_counter);
        let _ = self.eq_tx.send(SessionEvent::SubmissionStarted {
            session_id: self.session_id.clone(),
            submission_id: submission.id.clone(),
            turn_id: turn_id.clone(),
        });
        let _ = self.eq_tx.send(SessionEvent::TurnEvent {
            session_id: self.session_id.clone(),
            turn_id: turn_id.clone(),
            sequence: 0,
            payload: TurnEventPayload::Started,
        });

        if submission.common_kind() == Some(SubmissionKind::PreviewRequest) {
            let agent = self.agent.clone();
            let eq_tx = self.eq_tx.clone();
            let history = self.history.clone();
            let session_id = self.session_id.clone();
            let message = submission.items[0].text.clone();
            let handle = tokio::spawn(async move {
                let result = agent.preview_request(message, history).await;
                let (status, record_status) = match result {
                    Ok(Some(preview)) => {
                        let _ = eq_tx.send(SessionEvent::TurnEvent {
                            session_id: session_id.clone(),
                            turn_id: turn_id.clone(),
                            sequence: 1,
                            payload: TurnEventPayload::Progress {
                                event: AgentEvent::TurnStart,
                            },
                        });
                        let _ = eq_tx.send(SessionEvent::TurnEvent {
                            session_id: session_id.clone(),
                            turn_id: turn_id.clone(),
                            sequence: 2,
                            payload: TurnEventPayload::Progress {
                                event: AgentEvent::TextDelta {
                                    delta: preview.clone(),
                                },
                            },
                        });
                        let _ = eq_tx.send(SessionEvent::TurnEvent {
                            session_id: session_id.clone(),
                            turn_id: turn_id.clone(),
                            sequence: 3,
                            payload: TurnEventPayload::Progress {
                                event: AgentEvent::TurnEnd {
                                    stop_reason: talos_core::message::StopReason::EndTurn,
                                    usage: talos_core::message::Usage::default(),
                                },
                            },
                        });
                        (
                            TurnCompletionStatus::Success {
                                final_text: preview,
                                new_messages: Vec::new(),
                            },
                            TurnRecordStatus::Success,
                        )
                    }
                    Ok(None) => (
                        TurnCompletionStatus::Error {
                            message: "request preview is unavailable for this provider".into(),
                        },
                        TurnRecordStatus::Error,
                    ),
                    Err(error) => (
                        TurnCompletionStatus::Error {
                            message: error.to_string(),
                        },
                        TurnRecordStatus::Error,
                    ),
                };
                let _ = eq_tx.send(SessionEvent::TurnEvent {
                    session_id,
                    turn_id,
                    sequence: if record_status == TurnRecordStatus::Success {
                        4
                    } else {
                        1
                    },
                    payload: TurnEventPayload::Completed { status },
                });
                Some(TurnRecord {
                    new_messages: Vec::new(),
                    status: record_status,
                })
            });
            return (handle, CancellationToken::new());
        }

        if let Some(agent_mut) = Arc::get_mut(&mut self.agent) {
            agent_mut.set_append_prompt_opt(None);
        }

        let sequence = Arc::new(AtomicU64::new(1));
        let token = CancellationToken::new();
        let token_clone = token.clone();
        let agent = self.agent.clone();
        let eq_tx = self.eq_tx.clone();
        let history = self.history.clone();
        let persistence = self.persistence.clone();
        let durable_persistence = self.durable_persistence.clone();
        let session_id = self.session_id.clone();
        let items = submission.items;
        let handle = tokio::spawn(async move {
            let (event_tx, event_rx) = mpsc::unbounded_channel::<AgentEvent>();
            let (result_tx, result_rx) = tokio::sync::oneshot::channel::<TurnRecord>();
            let _ = AssertUnwindSafe(run_turn_with_forwarding(TurnForwarding {
                agent,
                items,
                history,
                event_tx,
                event_rx,
                eq_tx,
                cancel_token: token_clone,
                turn_id,
                session_id,
                sequence,
                persistence,
                durable_persistence,
                result_tx,
            }))
            .catch_unwind()
            .await;
            result_rx.await.ok()
        });
        (handle, token)
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

fn compatibility_submission(
    sequence: u64,
    kind: SubmissionKind,
    text: String,
    attachments: Vec<talos_core::message::ContentPart>,
) -> StructuredSubmission {
    StructuredSubmission {
        id: format!("compatibility_{sequence}"),
        source: SubmissionSource::Compatibility,
        items: vec![SubmissionItem {
            id: format!("compatibility_item_{sequence}"),
            enqueue_sequence: sequence,
            kind,
            text,
            attachments,
        }],
    }
}

fn validate_submission(submission: &StructuredSubmission) -> Result<(), SubmissionRejectionReason> {
    if submission.id.is_empty()
        || submission.items.is_empty()
        || submission.items.len() > MAX_SUBMISSION_BATCH_ITEMS
        || submission.common_kind().is_none()
        || (submission.common_kind() == Some(SubmissionKind::PreviewRequest)
            && (submission.items.len() != 1 || !submission.items[0].attachments.is_empty()))
    {
        return Err(SubmissionRejectionReason::InvalidStructure);
    }
    if submission.total_text_bytes() > MAX_SUBMISSION_BATCH_BYTES
        || submission
            .items
            .iter()
            .any(|item| item.id.is_empty() || item.text.len() > MAX_SUBMISSION_ITEM_BYTES)
    {
        return Err(SubmissionRejectionReason::LimitExceeded);
    }
    Ok(())
}

fn submission_message(item: &SubmissionItem) -> Message {
    if item.attachments.is_empty() {
        Message::User {
            content: item.text.clone(),
        }
    } else {
        let mut parts = Vec::with_capacity(item.attachments.len() + 1);
        if !item.text.is_empty() {
            parts.push(talos_core::message::ContentPart::Text {
                text: item.text.clone(),
            });
        }
        parts.extend(item.attachments.clone());
        Message::Multimodal { parts }
    }
}
