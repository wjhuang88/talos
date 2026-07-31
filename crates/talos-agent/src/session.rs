//! AppServerSession actor — bridges SQ→Agent→EQ (ADR-005 L2 seam).
//!
//! The session actor owns an [`Agent`] and runs a message loop:
//! - Receives [`SessionOp`] on the bounded SQ (cap=512)
//! - Drives agent turns via [`Agent::run_streaming`]
//! - Emits [`SessionEvent`] on the unbounded EQ

use std::collections::{HashSet, VecDeque};
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
    MAX_STEERING_QUEUE_BYTES, MAX_STEERING_QUEUE_IMAGE_BYTES, MAX_STEERING_QUEUE_IMAGES,
    MAX_STEERING_QUEUE_ITEMS, MAX_SUBMISSION_BATCH_BYTES, MAX_SUBMISSION_BATCH_ITEMS,
    MAX_SUBMISSION_IMAGE_BYTES, MAX_SUBMISSION_IMAGE_COUNT, MAX_SUBMISSION_ITEM_BYTES,
    MAX_SUBMISSION_TOTAL_IMAGE_BYTES, SessionConfig, SessionEvent, SessionHandle, SessionOp,
    StructuredSubmission, SubmissionItem, SubmissionKind, SubmissionRejectionReason,
    SubmissionSource, TurnCompletionStatus, TurnEventPayload,
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
const MAX_RECENT_SUBMISSION_IDS: usize = 1024;
const MAX_RECENT_ITEM_IDS: usize = 4096;

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
        let mut pending_items = 0_usize;
        let mut pending_bytes = 0_usize;
        let mut pending_images = 0_usize;
        let mut pending_image_bytes = 0_u64;
        let mut active_submission_ids = HashSet::<String>::new();
        let mut active_item_ids = HashSet::<String>::new();
        let mut recent_submission_ids = VecDeque::<String>::new();
        let mut recent_item_ids = VecDeque::<String>::new();
        let mut current_turn: Option<JoinHandle<Option<TurnRecord>>> = None;
        let mut current_submission_size: Option<(usize, usize, usize, u64)> = None;
        let mut cancel_token: Option<CancellationToken> = None;
        let mut paused = false;
        let mut shutting_down = false;

        loop {
            if current_turn.is_none()
                && !paused
                && let Some(submission) = pending.pop_front()
            {
                let (images, image_bytes) = submission.image_totals();
                let submission_size = (
                    submission.items.len(),
                    submission.total_text_bytes(),
                    images,
                    image_bytes,
                );
                let submission_id = submission.id.clone();
                let item_ids = submission
                    .items
                    .iter()
                    .map(|item| item.id.clone())
                    .collect::<Vec<_>>();
                turn_counter = turn_counter.saturating_add(1);
                match self.start_submission(submission, turn_counter).await {
                    Some((handle, token)) => {
                        active_submission_ids.remove(&submission_id);
                        record_recent_identity(
                            &mut recent_submission_ids,
                            submission_id,
                            MAX_RECENT_SUBMISSION_IDS,
                        );
                        for item_id in item_ids {
                            active_item_ids.remove(&item_id);
                            record_recent_identity(
                                &mut recent_item_ids,
                                item_id,
                                MAX_RECENT_ITEM_IDS,
                            );
                        }
                        current_turn = Some(handle);
                        current_submission_size = Some(submission_size);
                        cancel_token = Some(token);
                    }
                    None => {
                        active_submission_ids.remove(&submission_id);
                        for item_id in item_ids {
                            active_item_ids.remove(&item_id);
                        }
                        pending_items = pending_items.saturating_sub(submission_size.0);
                        pending_bytes = pending_bytes.saturating_sub(submission_size.1);
                        pending_images = pending_images.saturating_sub(submission_size.2);
                        pending_image_bytes = pending_image_bytes.saturating_sub(submission_size.3);
                        paused = true;
                    }
                }
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
                    if let Some((items, bytes, images, image_bytes)) = current_submission_size.take() {
                        pending_items = pending_items.saturating_sub(items);
                        pending_bytes = pending_bytes.saturating_sub(bytes);
                        pending_images = pending_images.saturating_sub(images);
                        pending_image_bytes = pending_image_bytes.saturating_sub(image_bytes);
                    }
                    let succeeded = match completed.and_then(Result::ok).flatten() {
                        Some(record) => {
                            let status = record.status;
                            self.commit_turn_record(record);
                            status == TurnRecordStatus::Success
                        }
                        None => false,
                    };
                    paused = !succeeded;
                    if !succeeded {
                        let mut retained_scheduler = VecDeque::new();
                        while let Some(submission) = pending.pop_front() {
                            if submission.source == SubmissionSource::Scheduler {
                                retained_scheduler.push_back(submission);
                                continue;
                            }
                            pending_items = pending_items.saturating_sub(submission.items.len());
                            pending_bytes = pending_bytes.saturating_sub(submission.total_text_bytes());
                            let (images, image_bytes) = submission.image_totals();
                            pending_images = pending_images.saturating_sub(images);
                            pending_image_bytes = pending_image_bytes.saturating_sub(image_bytes);
                            release_active_submission(
                                &submission,
                                &mut active_submission_ids,
                                &mut active_item_ids,
                            );
                            self.reject_submission(
                                &submission.id,
                                submission.sender_generation,
                                SubmissionRejectionReason::Cancelled,
                            );
                        }
                        pending = retained_scheduler;
                    }
                }
                op = self.sq_rx.recv(), if !shutting_down => {
                    let Some(op) = op else {
                        shutting_down = true;
                        if let Some(token) = cancel_token.take() { token.cancel(); }
                        for submission in pending.drain(..) {
                            release_active_submission(
                                &submission,
                                &mut active_submission_ids,
                                &mut active_item_ids,
                            );
                            self.reject_submission(
                                &submission.id,
                                submission.sender_generation,
                                SubmissionRejectionReason::SessionClosed,
                            );
                        }
                        pending_items = current_submission_size.map_or(0, |size| size.0);
                        pending_bytes = current_submission_size.map_or(0, |size| size.1);
                        pending_images = current_submission_size.map_or(0, |size| size.2);
                        pending_image_bytes = current_submission_size.map_or(0, |size| size.3);
                        continue;
                    };
            match op {
                SessionOp::Submit { message } => {
                    submission_counter = submission_counter.saturating_add(1);
                    let submission = compatibility_submission(submission_counter, SubmissionKind::UserTurn, message, Vec::new());
                    if self.accept_submission(
                        submission,
                        &mut pending,
                        &mut pending_items,
                        &mut pending_bytes,
                        &mut pending_images,
                        &mut pending_image_bytes,
                        &mut active_submission_ids,
                        &mut active_item_ids,
                        &mut recent_submission_ids,
                        &mut recent_item_ids,
                        paused,
                    ) { paused = false; }
                }
                SessionOp::SubmitMultimodal { text, attachments } => {
                    submission_counter = submission_counter.saturating_add(1);
                    let submission = compatibility_submission(submission_counter, SubmissionKind::UserTurn, text, attachments);
                    if self.accept_submission(
                        submission,
                        &mut pending,
                        &mut pending_items,
                        &mut pending_bytes,
                        &mut pending_images,
                        &mut pending_image_bytes,
                        &mut active_submission_ids,
                        &mut active_item_ids,
                        &mut recent_submission_ids,
                        &mut recent_item_ids,
                        paused,
                    ) { paused = false; }
                }
                SessionOp::PreviewRequest { message } => {
                    submission_counter = submission_counter.saturating_add(1);
                    let submission = compatibility_submission(submission_counter, SubmissionKind::PreviewRequest, message, Vec::new());
                    if self.accept_submission(
                        submission,
                        &mut pending,
                        &mut pending_items,
                        &mut pending_bytes,
                        &mut pending_images,
                        &mut pending_image_bytes,
                        &mut active_submission_ids,
                        &mut active_item_ids,
                        &mut recent_submission_ids,
                        &mut recent_item_ids,
                        paused,
                    ) { paused = false; }
                }
                SessionOp::SubmitStructured { submission } => {
                    let resumes = submission.source != SubmissionSource::Scheduler;
                    if self.accept_submission(
                        submission,
                        &mut pending,
                        &mut pending_items,
                        &mut pending_bytes,
                        &mut pending_images,
                        &mut pending_image_bytes,
                        &mut active_submission_ids,
                        &mut active_item_ids,
                        &mut recent_submission_ids,
                        &mut recent_item_ids,
                        paused,
                    ) && resumes { paused = false; }
                }
                SessionOp::Interrupt => {
                    if let Some(token) = &cancel_token {
                        token.cancel();
                    } else if let Some(submission) = pending.pop_front() {
                        pending_items = pending_items.saturating_sub(submission.items.len());
                        pending_bytes = pending_bytes.saturating_sub(submission.total_text_bytes());
                        let (images, image_bytes) = submission.image_totals();
                        pending_images = pending_images.saturating_sub(images);
                        pending_image_bytes = pending_image_bytes.saturating_sub(image_bytes);
                        release_active_submission(
                            &submission,
                            &mut active_submission_ids,
                            &mut active_item_ids,
                        );
                        self.reject_submission(
                            &submission.id,
                            submission.sender_generation,
                            SubmissionRejectionReason::Cancelled,
                        );
                    }
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
                    if let Some(token) = &cancel_token { token.cancel(); }
                    for submission in pending.drain(..) {
                        release_active_submission(
                            &submission,
                            &mut active_submission_ids,
                            &mut active_item_ids,
                        );
                        self.reject_submission(
                            &submission.id,
                            submission.sender_generation,
                            SubmissionRejectionReason::SessionClosed,
                        );
                    }
                    pending_items = current_submission_size.map_or(0, |size| size.0);
                    pending_bytes = current_submission_size.map_or(0, |size| size.1);
                    pending_images = current_submission_size.map_or(0, |size| size.2);
                    pending_image_bytes = current_submission_size.map_or(0, |size| size.3);
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

    #[allow(clippy::too_many_arguments)]
    fn accept_submission(
        &self,
        submission: StructuredSubmission,
        pending: &mut VecDeque<StructuredSubmission>,
        pending_items: &mut usize,
        pending_bytes: &mut usize,
        pending_images: &mut usize,
        pending_image_bytes: &mut u64,
        active_submission_ids: &mut HashSet<String>,
        active_item_ids: &mut HashSet<String>,
        recent_submission_ids: &mut VecDeque<String>,
        recent_item_ids: &mut VecDeque<String>,
        paused: bool,
    ) -> bool {
        if let Err(reason) = validate_submission(&submission) {
            self.reject_submission(&submission.id, submission.sender_generation, reason);
            return false;
        }
        if active_submission_ids.contains(&submission.id)
            || recent_submission_ids.contains(&submission.id)
            || submission.items.iter().any(|item| {
                active_item_ids.contains(&item.id) || recent_item_ids.contains(&item.id)
            })
        {
            self.reject_submission(
                &submission.id,
                submission.sender_generation,
                SubmissionRejectionReason::Duplicate,
            );
            return false;
        }
        let Some(next_items) = pending_items.checked_add(submission.items.len()) else {
            self.reject_submission(
                &submission.id,
                submission.sender_generation,
                SubmissionRejectionReason::LimitExceeded,
            );
            return false;
        };
        let Some(next_bytes) = pending_bytes.checked_add(submission.total_text_bytes()) else {
            self.reject_submission(
                &submission.id,
                submission.sender_generation,
                SubmissionRejectionReason::LimitExceeded,
            );
            return false;
        };
        let (submission_images, submission_image_bytes) = submission.image_totals();
        let Some(next_images) = pending_images.checked_add(submission_images) else {
            self.reject_submission(
                &submission.id,
                submission.sender_generation,
                SubmissionRejectionReason::LimitExceeded,
            );
            return false;
        };
        let Some(next_image_bytes) = pending_image_bytes.checked_add(submission_image_bytes) else {
            self.reject_submission(
                &submission.id,
                submission.sender_generation,
                SubmissionRejectionReason::LimitExceeded,
            );
            return false;
        };
        if next_items > MAX_STEERING_QUEUE_ITEMS
            || next_bytes > MAX_STEERING_QUEUE_BYTES
            || next_images > MAX_STEERING_QUEUE_IMAGES
            || next_image_bytes > MAX_STEERING_QUEUE_IMAGE_BYTES
        {
            self.reject_submission(
                &submission.id,
                submission.sender_generation,
                SubmissionRejectionReason::LimitExceeded,
            );
            return false;
        }
        if self
            .eq_tx
            .send(SessionEvent::SubmissionQueued {
                session_id: self.session_id.clone(),
                submission_id: submission.id.clone(),
                sender_generation: submission.sender_generation,
                source: submission.source,
                item_count: submission.items.len(),
                total_text_bytes: submission.total_text_bytes(),
            })
            .is_err()
        {
            return false;
        }
        if submission.source != SubmissionSource::User
            && self
                .eq_tx
                .send(SessionEvent::ExternalSubmissionQueued {
                    session_id: self.session_id.clone(),
                    submission_id: submission.id.clone(),
                    sender_generation: submission.sender_generation,
                    source: submission.source,
                    item_texts: submission
                        .items
                        .iter()
                        .map(|item| item.text.clone())
                        .collect(),
                })
                .is_err()
        {
            return false;
        }

        active_submission_ids.insert(submission.id.clone());
        active_item_ids.extend(submission.items.iter().map(|item| item.id.clone()));
        *pending_items = next_items;
        *pending_bytes = next_bytes;
        *pending_images = next_images;
        *pending_image_bytes = next_image_bytes;
        if paused && submission.source != SubmissionSource::Scheduler {
            pending.push_front(submission);
        } else {
            pending.push_back(submission);
        }
        true
    }

    fn reject_submission(
        &self,
        submission_id: &str,
        sender_generation: u64,
        reason: SubmissionRejectionReason,
    ) {
        let _ = self.eq_tx.send(SessionEvent::SubmissionRejected {
            session_id: self.session_id.clone(),
            submission_id: submission_id.to_owned(),
            sender_generation,
            reason,
        });
    }

    async fn start_submission(
        &mut self,
        submission: StructuredSubmission,
        turn_counter: u64,
    ) -> Option<(JoinHandle<Option<TurnRecord>>, CancellationToken)> {
        let submission_kind = submission.common_kind();
        if submission_kind != Some(SubmissionKind::PreviewRequest)
            && let Some(agent_mut) = Arc::get_mut(&mut self.agent)
        {
            agent_mut.set_append_prompt_opt(None);
        }

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

        let mut prepared = match self
            .agent
            .prepare_session_turn(&submission.items, self.history.clone())
            .await
        {
            Ok(prepared) => prepared,
            Err(_) => {
                self.reject_submission(
                    &submission.id,
                    submission.sender_generation,
                    SubmissionRejectionReason::ContextBudgetExceeded,
                );
                return None;
            }
        };
        let mut request_tokens = self.agent.prepared_session_request_tokens(&prepared);
        if request_tokens > self.model_context_limit {
            let fixed_tokens = self.agent.prepared_session_fixed_tokens(&prepared);
            let history_budget = self.model_context_limit.saturating_sub(fixed_tokens);
            let mut projected_compactor = Compactor::new(TokenEstimator::new(), history_budget);
            self.history = match projected_compactor
                .compact(self.history.clone(), self.agent.provider())
                .await
            {
                Ok(history) => history,
                Err(_) => {
                    projected_compactor
                        .compact_deterministic(self.history.clone())
                        .0
                }
            };
            Agent::replace_prepared_session_history(&mut prepared, self.history.clone());
            request_tokens = self.agent.prepared_session_request_tokens(&prepared);
            if let (Some(file), Some(dir)) = (&self.session_file, &self.session_dir) {
                let _ = self.try_archive_session(file, dir, &self.history);
            }
        }
        if request_tokens > self.model_context_limit {
            self.reject_submission(
                &submission.id,
                submission.sender_generation,
                SubmissionRejectionReason::ContextBudgetExceeded,
            );
            return None;
        }

        let turn_id = format!("{}_{}", self.turn_prefix, turn_counter);
        if self
            .eq_tx
            .send(SessionEvent::TurnEvent {
                session_id: self.session_id.clone(),
                turn_id: turn_id.clone(),
                sequence: 0,
                payload: TurnEventPayload::Started,
            })
            .is_err()
        {
            return None;
        }
        if self
            .eq_tx
            .send(SessionEvent::SubmissionStarted {
                session_id: self.session_id.clone(),
                submission_id: submission.id.clone(),
                sender_generation: submission.sender_generation,
                turn_id: turn_id.clone(),
            })
            .is_err()
        {
            return None;
        }

        if submission_kind == Some(SubmissionKind::PreviewRequest) {
            let agent = self.agent.clone();
            let eq_tx = self.eq_tx.clone();
            let session_id = self.session_id.clone();
            let token = CancellationToken::new();
            let preview_token = token.clone();
            let handle = tokio::spawn(async move {
                let result = tokio::select! {
                    () = preview_token.cancelled() => {
                        let _ = eq_tx.send(SessionEvent::TurnEvent {
                            session_id,
                            turn_id,
                            sequence: 1,
                            payload: TurnEventPayload::Completed {
                                status: TurnCompletionStatus::Cancelled,
                            },
                        });
                        return Some(TurnRecord {
                            new_messages: Vec::new(),
                            status: TurnRecordStatus::Cancelled,
                        });
                    }
                    result = async { Ok::<Option<String>, crate::AgentError>(agent.preview_prepared_session_turn(&prepared)) } => result,
                };
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
            return Some((handle, token));
        }

        let sequence = Arc::new(AtomicU64::new(1));
        let token = CancellationToken::new();
        let token_clone = token.clone();
        let agent = self.agent.clone();
        let eq_tx = self.eq_tx.clone();
        let persistence = self.persistence.clone();
        let durable_persistence = self.durable_persistence.clone();
        let session_id = self.session_id.clone();
        let request_context_limit = self.model_context_limit;
        let handle = tokio::spawn(async move {
            let (event_tx, event_rx) = mpsc::unbounded_channel::<AgentEvent>();
            let (result_tx, result_rx) = tokio::sync::oneshot::channel::<TurnRecord>();
            let _ = AssertUnwindSafe(run_turn_with_forwarding(TurnForwarding {
                agent,
                prepared,
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
                request_context_limit,
            }))
            .catch_unwind()
            .await;
            result_rx.await.ok()
        });
        Some((handle, token))
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
        sender_generation: 0,
        items: vec![SubmissionItem {
            id: format!("compatibility_item_{sequence}"),
            enqueue_sequence: sequence,
            kind,
            text,
            attachments,
        }],
    }
}

fn release_active_submission(
    submission: &StructuredSubmission,
    active_submission_ids: &mut HashSet<String>,
    active_item_ids: &mut HashSet<String>,
) {
    active_submission_ids.remove(&submission.id);
    for item in &submission.items {
        active_item_ids.remove(&item.id);
    }
}

fn record_recent_identity(identities: &mut VecDeque<String>, id: String, capacity: usize) {
    while identities.len() >= capacity {
        identities.pop_front();
    }
    identities.push_back(id);
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
    let mut image_count = 0_usize;
    let mut total_image_bytes = 0_u64;
    for item in &submission.items {
        for attachment in &item.attachments {
            if let talos_core::message::ContentPart::Image { byte_count, .. } = attachment {
                image_count = image_count.saturating_add(1);
                total_image_bytes = total_image_bytes.saturating_add(*byte_count);
                if image_count > MAX_SUBMISSION_IMAGE_COUNT
                    || *byte_count > MAX_SUBMISSION_IMAGE_BYTES
                    || total_image_bytes > MAX_SUBMISSION_TOTAL_IMAGE_BYTES
                {
                    return Err(SubmissionRejectionReason::LimitExceeded);
                }
            }
        }
    }
    let mut item_ids = HashSet::with_capacity(submission.items.len());
    if submission
        .items
        .iter()
        .any(|item| !item_ids.insert(item.id.as_str()))
    {
        return Err(SubmissionRejectionReason::Duplicate);
    }
    Ok(())
}
