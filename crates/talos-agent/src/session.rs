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
    MAX_STEERING_QUEUE_BYTES, MAX_STEERING_QUEUE_IMAGE_BYTES, MAX_STEERING_QUEUE_IMAGES,
    MAX_STEERING_QUEUE_ITEMS, PendingSubmissionState, SessionConfig, SessionEvent, SessionHandle,
    SessionOp, StructuredSubmission, SubmissionItem, SubmissionKind, SubmissionReceipt,
    SubmissionReceiptDisposition, SubmissionRejectionReason, SubmissionSource,
    TurnCompletionStatus, TurnEventPayload,
};
#[cfg(test)]
use talos_core::session::{MAX_SUBMISSION_BATCH_ITEMS, MAX_SUBMISSION_IMAGE_COUNT};
use talos_session::PendingSubmissionStore;

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

#[derive(Debug, Clone)]
struct ActiveStructuredTurn {
    submission_id: String,
    receipt_id: String,
    session_generation: u64,
    turn_id: String,
}

struct StartedTurn {
    handle: JoinHandle<Option<TurnRecord>>,
    token: CancellationToken,
    structured: Option<ActiveStructuredTurn>,
}

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
    pending_store: PendingSubmissionStore,
    session_id: String,
    turn_prefix: String,
    model_context_limit: u32,
}

impl AppServerSession {
    /// Creates a new session actor with the given agent and configuration.
    ///
    /// Returns a [`SessionHandle`] and the actor itself for spawning.
    pub fn new(agent: Agent, config: SessionConfig) -> (SessionHandle, Self) {
        let (sq_tx, sq_rx) = tokio::sync::mpsc::channel(512);
        let (eq_tx, eq_rx) = mpsc::unbounded_channel();

        let handle = SessionHandle { sq_tx, eq_rx };
        let compactor = Compactor::new(TokenEstimator::new(), config.model_context_limit);
        let instance_id = NEXT_RUNTIME_SESSION_ID.fetch_add(1, Ordering::Relaxed);
        let session_id = format!("runtime_{}_{}", std::process::id(), instance_id);
        let pending_session_file = config
            .workspace_root
            .join(".talos")
            .join("runtime")
            .join(format!("{session_id}.tlog"));
        let pending_store =
            PendingSubmissionStore::for_session_file(&pending_session_file, &session_id);

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
            pending_store,
            session_id,
            turn_prefix: format!("turn_{}_{}", std::process::id(), instance_id),
            model_context_limit: config.model_context_limit,
        };

        (handle, actor)
    }

    pub fn set_session_paths(&mut self, file: PathBuf, dir: PathBuf) {
        self.session_file = Some(file);
        self.session_dir = Some(dir);
    }

    /// Assigns the durable session that owns successful turn-message writes.
    pub fn set_persistence(
        &mut self,
        session: talos_session::Session,
        metadata: talos_session::SessionMetadata,
    ) {
        self.pending_store = PendingSubmissionStore::for_session(&session);
        self.session_id = session.id.to_string();
        self.persistence = Some(TurnPersistence { session, metadata });
    }

    /// Assigns an atomic durable session used by the embedded runtime.
    pub fn set_durable_persistence(
        &mut self,
        session: talos_session::DurableSession,
        policy: talos_session::PersistencePolicy,
    ) {
        let session_id = session.id().to_string();
        self.pending_store =
            PendingSubmissionStore::for_session_file(session.file_path(), &session_id);
        self.session_id = session_id;
        self.durable_persistence = Some(DurableTurnPersistence { session, policy });
    }

    /// Runs the session actor until shutdown or SQ disconnect.
    pub async fn run(&mut self) {
        let mut turn_counter: u64 = 0;
        let mut submission_counter: u64 = 0;
        let mut pending = VecDeque::<StructuredSubmission>::new();
        let mut pending_items = 0_usize;
        let mut pending_bytes = 0_usize;
        let mut pending_images = 0_usize;
        let mut pending_image_bytes = 0_u64;
        let mut recent_submission_ids = VecDeque::<String>::new();
        let mut recent_item_ids = VecDeque::<String>::new();
        let mut current_turn: Option<JoinHandle<Option<TurnRecord>>> = None;
        let mut current_submission_size: Option<(usize, usize, usize, u64)> = None;
        let mut current_structured: Option<ActiveStructuredTurn> = None;
        let mut cancel_token: Option<CancellationToken> = None;
        let mut paused = self.restore_pending_submissions(
            &mut pending,
            &mut pending_items,
            &mut pending_bytes,
            &mut pending_images,
            &mut pending_image_bytes,
            &mut recent_submission_ids,
            &mut recent_item_ids,
        );
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
                turn_counter = turn_counter.saturating_add(1);
                match self.start_submission(submission, turn_counter).await {
                    Some(started) => {
                        current_turn = Some(started.handle);
                        current_submission_size = Some(submission_size);
                        current_structured = started.structured;
                        cancel_token = Some(started.token);
                    }
                    None => {
                        pending_items = pending_items.saturating_sub(submission_size.0);
                        pending_bytes = pending_bytes.saturating_sub(submission_size.1);
                        pending_images = pending_images.saturating_sub(submission_size.2);
                        pending_image_bytes = pending_image_bytes.saturating_sub(submission_size.3);
                        if let Err(error) = self.pending_store.pause_unstarted() {
                            self.emit_custody_error(
                                "failed to pause rejected pending work",
                                &error,
                            );
                        }
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
                    let structured = current_structured.take();
                    match completed.and_then(Result::ok).flatten() {
                        Some(record) => {
                            let status = record.status;
                            let completion = record.completion.clone();
                            self.commit_turn_record(record);
                            let custody_ok = structured.as_ref().is_none_or(|active| {
                                self.finish_structured_turn(active, &completion)
                            });
                            paused = status != TurnRecordStatus::Success || !custody_ok;
                        }
                        None => {
                            if let Some(active) = structured.as_ref() {
                                let completion = TurnCompletionStatus::Error {
                                    message: "turn task ended without a completion record".into(),
                                };
                                let _ = self.finish_structured_turn(active, &completion);
                            }
                            paused = true;
                        }
                    }
                }
                op = self.sq_rx.recv(), if !shutting_down => {
                    let Some(op) = op else {
                        shutting_down = true;
                        if let Some(token) = cancel_token.take() {
                            token.cancel();
                        }
                        self.release_in_memory_pending_on_shutdown(&mut pending);
                        let _ = self.pending_store.pause_unstarted();
                        pending_items = current_submission_size.map_or(0, |size| size.0);
                        pending_bytes = current_submission_size.map_or(0, |size| size.1);
                        pending_images = current_submission_size.map_or(0, |size| size.2);
                        pending_image_bytes = current_submission_size.map_or(0, |size| size.3);
                        continue;
                    };
                    match op {
                        SessionOp::Submit { message } => {
                            submission_counter = submission_counter.saturating_add(1);
                            let submission = compatibility_submission(
                                submission_counter,
                                SubmissionKind::UserTurn,
                                message,
                                Vec::new(),
                            );
                            if self.accept_submission(
                                submission,
                                &mut pending,
                                &mut pending_items,
                                &mut pending_bytes,
                                &mut pending_images,
                                &mut pending_image_bytes,
                                &mut recent_submission_ids,
                                &mut recent_item_ids,
                                paused,
                            ) {
                                paused = false;
                            }
                        }
                        SessionOp::SubmitMultimodal { text, attachments } => {
                            submission_counter = submission_counter.saturating_add(1);
                            let submission = compatibility_submission(
                                submission_counter,
                                SubmissionKind::UserTurn,
                                text,
                                attachments,
                            );
                            if self.accept_submission(
                                submission,
                                &mut pending,
                                &mut pending_items,
                                &mut pending_bytes,
                                &mut pending_images,
                                &mut pending_image_bytes,
                                &mut recent_submission_ids,
                                &mut recent_item_ids,
                                paused,
                            ) {
                                paused = false;
                            }
                        }
                        SessionOp::PreviewRequest { message } => {
                            submission_counter = submission_counter.saturating_add(1);
                            let submission = compatibility_submission(
                                submission_counter,
                                SubmissionKind::PreviewRequest,
                                message,
                                Vec::new(),
                            );
                            if self.accept_submission(
                                submission,
                                &mut pending,
                                &mut pending_items,
                                &mut pending_bytes,
                                &mut pending_images,
                                &mut pending_image_bytes,
                                &mut recent_submission_ids,
                                &mut recent_item_ids,
                                paused,
                            ) {
                                paused = false;
                            }
                        }
                        SessionOp::SubmitStructured { submission } => {
                            let resumes = submission.source != SubmissionSource::Scheduler;
                            if self.accept_durable_submission(
                                submission,
                                &mut pending,
                                &mut pending_items,
                                &mut pending_bytes,
                                &mut pending_images,
                                &mut pending_image_bytes,
                                &mut recent_submission_ids,
                                &mut recent_item_ids,
                                paused,
                                None,
                            ) && resumes
                            {
                                paused = false;
                            }
                        }
                        SessionOp::SubmitStructuredTracked {
                            submission,
                            receipt_tx,
                        } => {
                            let resumes = submission.source != SubmissionSource::Scheduler;
                            if self.accept_durable_submission(
                                submission,
                                &mut pending,
                                &mut pending_items,
                                &mut pending_bytes,
                                &mut pending_images,
                                &mut pending_image_bytes,
                                &mut recent_submission_ids,
                                &mut recent_item_ids,
                                paused,
                                receipt_tx.as_ref(),
                            ) && resumes
                            {
                                paused = false;
                            }
                        }
                        SessionOp::ReconcileStructured { submission } => {
                            let resumes = submission.source != SubmissionSource::Scheduler;
                            if self.reconcile_submission(&submission, &pending, None) && resumes {
                                paused = false;
                            }
                        }
                        SessionOp::ReconcileStructuredTracked {
                            submission,
                            receipt_tx,
                        } => {
                            let resumes = submission.source != SubmissionSource::Scheduler;
                            if self.reconcile_submission(
                                &submission,
                                &pending,
                                receipt_tx.as_ref(),
                            ) && resumes
                            {
                                paused = false;
                            }
                        }
                        SessionOp::Interrupt => {
                            if let Some(token) = &cancel_token {
                                token.cancel();
                            } else if !pending.is_empty() {
                                if let Err(error) = self.pending_store.pause_unstarted() {
                                    self.emit_custody_error("failed to pause pending work", &error);
                                }
                                paused = true;
                            }
                        }
                        SessionOp::SetSkillContext { name, content } => {
                            if current_turn.is_some() || !pending.is_empty() {
                                let _ = self.eq_tx.send(SessionEvent::Error {
                                    message: "cannot change active skill while a turn is active"
                                        .into(),
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
                                    message: "cannot change active skill while agent is busy"
                                        .into(),
                                });
                            }
                        }
                        SessionOp::Shutdown => {
                            shutting_down = true;
                            if let Some(token) = &cancel_token {
                                token.cancel();
                            }
                            self.release_in_memory_pending_on_shutdown(&mut pending);
                            if let Err(error) = self.pending_store.pause_unstarted() {
                                self.emit_custody_error("failed to persist shutdown pause", &error);
                            }
                            pending_items = current_submission_size.map_or(0, |size| size.0);
                            pending_bytes = current_submission_size.map_or(0, |size| size.1);
                            pending_images = current_submission_size.map_or(0, |size| size.2);
                            pending_image_bytes = current_submission_size.map_or(0, |size| size.3);
                        }
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn restore_pending_submissions(
        &self,
        pending: &mut VecDeque<StructuredSubmission>,
        pending_items: &mut usize,
        pending_bytes: &mut usize,
        pending_images: &mut usize,
        pending_image_bytes: &mut u64,
        recent_submission_ids: &mut VecDeque<String>,
        recent_item_ids: &mut VecDeque<String>,
    ) -> bool {
        let records = match self.pending_store.recover_unstarted() {
            Ok(records) => records,
            Err(error) => {
                self.emit_custody_error("failed to recover pending submissions", &error);
                return true;
            }
        };
        let paused = !records.is_empty();
        for record in records {
            let submission = record.submission;
            if let Err(reason) = validate_submission(&submission) {
                let _ = self.eq_tx.send(SessionEvent::Error {
                    message: format!(
                        "pending submission {} failed validation during recovery: {reason:?}",
                        submission.id
                    ),
                });
                continue;
            }
            let (images, image_bytes) = submission.image_totals();
            *pending_items = pending_items.saturating_add(submission.items.len());
            *pending_bytes = pending_bytes.saturating_add(submission.total_text_bytes());
            *pending_images = pending_images.saturating_add(images);
            *pending_image_bytes = pending_image_bytes.saturating_add(image_bytes);
            record_recent_identity(
                recent_submission_ids,
                submission.id.clone(),
                MAX_RECENT_SUBMISSION_IDS,
            );
            for item in &submission.items {
                record_recent_identity(recent_item_ids, item.id.clone(), MAX_RECENT_ITEM_IDS);
            }
            pending.push_back(submission);
        }
        paused
    }

    fn release_in_memory_pending_on_shutdown(&self, pending: &mut VecDeque<StructuredSubmission>) {
        for submission in pending.drain(..) {
            // Legacy clients only understand that this Actor instance closed.
            // Durable structured custody is preserved separately in the journal
            // and is paused immediately after this compatibility projection.
            self.reject_submission(
                &submission.id,
                submission.sender_generation,
                SubmissionRejectionReason::SessionClosed,
            );
        }
    }

    fn commit_turn_record(&mut self, record: TurnRecord) {
        for msg in record.new_messages {
            self.history.push(msg);
        }
    }

    fn finish_structured_turn(
        &self,
        active: &ActiveStructuredTurn,
        completion: &TurnCompletionStatus,
    ) -> bool {
        let transition = match completion {
            TurnCompletionStatus::Success { .. } => self
                .pending_store
                .mark_committed(&active.submission_id, &active.turn_id),
            TurnCompletionStatus::Cancelled => self.pending_store.mark_terminal(
                &active.submission_id,
                PendingSubmissionState::TerminalCancelled,
                &active.turn_id,
            ),
            TurnCompletionStatus::Error { .. } => self.pending_store.mark_terminal(
                &active.submission_id,
                PendingSubmissionState::TerminalError,
                &active.turn_id,
            ),
        };
        if let Err(error) = transition {
            self.emit_custody_error("failed to finalize structured turn custody", &error);
            return false;
        }
        if !matches!(completion, TurnCompletionStatus::Success { .. })
            && let Err(error) = self.pending_store.pause_unstarted()
        {
            self.emit_custody_error("failed to pause unstarted work after terminal turn", &error);
            return false;
        }
        let _ = self.eq_tx.send(SessionEvent::StructuredTurnEvent {
            session_id: self.session_id.clone(),
            session_generation: active.session_generation,
            submission_id: active.submission_id.clone(),
            receipt_id: active.receipt_id.clone(),
            turn_id: active.turn_id.clone(),
            sequence: 1,
            payload: TurnEventPayload::Completed {
                status: completion.clone(),
            },
        });
        true
    }

    fn emit_custody_error(&self, context: &str, error: &impl std::fmt::Display) {
        let _ = self.eq_tx.send(SessionEvent::Error {
            message: format!("{context}: {error}"),
        });
    }

    #[allow(clippy::too_many_arguments)]
    fn accept_durable_submission(
        &self,
        submission: StructuredSubmission,
        pending: &mut VecDeque<StructuredSubmission>,
        pending_items: &mut usize,
        pending_bytes: &mut usize,
        pending_images: &mut usize,
        pending_image_bytes: &mut u64,
        recent_submission_ids: &mut VecDeque<String>,
        recent_item_ids: &mut VecDeque<String>,
        paused: bool,
        receipt_tx: Option<&mpsc::UnboundedSender<SubmissionReceipt>>,
    ) -> bool {
        let (existing_receipt, existing_disposition) =
            match self.pending_store.reconcile(&submission) {
                Ok(result) => result,
                Err(error) => {
                    self.emit_durability_rejection(&submission, &error, receipt_tx);
                    return false;
                }
            };
        match existing_disposition {
            SubmissionReceiptDisposition::AlreadyAccepted { state, turn_id } => {
                let disposition = SubmissionReceiptDisposition::AlreadyAccepted { state, turn_id };
                self.emit_submission_receipt(
                    &submission,
                    existing_receipt,
                    disposition,
                    receipt_tx,
                );
                // Compatibility-only projection for clients that predate durable
                // receipts. The authoritative result is AlreadyAccepted above.
                self.reject_submission(
                    &submission.id,
                    submission.sender_generation,
                    SubmissionRejectionReason::Duplicate,
                );
                return matches!(
                    state,
                    PendingSubmissionState::AcceptedPending | PendingSubmissionState::PausedPending
                ) && pending.iter().any(|queued| queued.id == submission.id);
            }
            SubmissionReceiptDisposition::Rejected { reason } => {
                self.emit_submission_receipt(
                    &submission,
                    existing_receipt,
                    SubmissionReceiptDisposition::Rejected { reason },
                    receipt_tx,
                );
                self.reject_submission(&submission.id, submission.sender_generation, reason);
                return false;
            }
            SubmissionReceiptDisposition::NotAccepted => {}
            SubmissionReceiptDisposition::AcceptedPending => {
                self.emit_custody_error(
                    "structured reconciliation returned an invalid first-accept state",
                    &submission.id,
                );
                return false;
            }
        }

        if let Err(reason) = validate_submission(&submission) {
            self.emit_admission_rejection(&submission, reason, receipt_tx);
            return false;
        }
        if submission
            .items
            .iter()
            .any(|item| recent_item_ids.contains(&item.id))
        {
            self.emit_admission_rejection(
                &submission,
                SubmissionRejectionReason::Duplicate,
                receipt_tx,
            );
            return false;
        }
        if !queue_has_capacity(
            &submission,
            *pending_items,
            *pending_bytes,
            *pending_images,
            *pending_image_bytes,
        ) {
            self.emit_admission_rejection(
                &submission,
                SubmissionRejectionReason::LimitExceeded,
                receipt_tx,
            );
            return false;
        }

        let (receipt_id, disposition) = match self.pending_store.accept(&submission) {
            Ok(result) => result,
            Err(error) => {
                self.emit_durability_rejection(&submission, &error, receipt_tx);
                return false;
            }
        };
        self.emit_submission_receipt(&submission, receipt_id, disposition.clone(), receipt_tx);
        match disposition {
            SubmissionReceiptDisposition::AcceptedPending => self.accept_submission(
                submission,
                pending,
                pending_items,
                pending_bytes,
                pending_images,
                pending_image_bytes,
                recent_submission_ids,
                recent_item_ids,
                paused,
            ),
            SubmissionReceiptDisposition::AlreadyAccepted { state, .. } => {
                matches!(
                    state,
                    PendingSubmissionState::AcceptedPending | PendingSubmissionState::PausedPending
                ) && pending.iter().any(|queued| queued.id == submission.id)
            }
            SubmissionReceiptDisposition::NotAccepted => false,
            SubmissionReceiptDisposition::Rejected { reason } => {
                self.reject_submission(&submission.id, submission.sender_generation, reason);
                false
            }
        }
    }

    fn reconcile_submission(
        &self,
        submission: &StructuredSubmission,
        pending: &VecDeque<StructuredSubmission>,
        receipt_tx: Option<&mpsc::UnboundedSender<SubmissionReceipt>>,
    ) -> bool {
        let (receipt_id, disposition) = match self.pending_store.reconcile(submission) {
            Ok(result) => result,
            Err(error) => {
                self.emit_durability_rejection(submission, &error, receipt_tx);
                return false;
            }
        };
        let resume = matches!(
            disposition,
            SubmissionReceiptDisposition::AlreadyAccepted {
                state: PendingSubmissionState::AcceptedPending
                    | PendingSubmissionState::PausedPending,
                ..
            }
        ) && pending.iter().any(|queued| queued.id == submission.id);
        self.emit_submission_receipt(submission, receipt_id, disposition, receipt_tx);
        resume
    }

    fn emit_admission_rejection(
        &self,
        submission: &StructuredSubmission,
        reason: SubmissionRejectionReason,
        receipt_tx: Option<&mpsc::UnboundedSender<SubmissionReceipt>>,
    ) {
        self.emit_submission_receipt(
            submission,
            String::new(),
            SubmissionReceiptDisposition::Rejected { reason },
            receipt_tx,
        );
        self.reject_submission(&submission.id, submission.sender_generation, reason);
    }

    fn emit_durability_rejection(
        &self,
        submission: &StructuredSubmission,
        error: &impl std::fmt::Display,
        receipt_tx: Option<&mpsc::UnboundedSender<SubmissionReceipt>>,
    ) {
        self.emit_admission_rejection(
            submission,
            SubmissionRejectionReason::DurabilityUnavailable,
            receipt_tx,
        );
        self.emit_custody_error("structured submission durability failed", error);
    }

    fn emit_submission_receipt(
        &self,
        submission: &StructuredSubmission,
        receipt_id: String,
        disposition: SubmissionReceiptDisposition,
        receipt_tx: Option<&mpsc::UnboundedSender<SubmissionReceipt>>,
    ) {
        let receipt = SubmissionReceipt {
            session_id: self.session_id.clone(),
            session_generation: submission.sender_generation,
            submission_id: submission.id.clone(),
            reservation_id: submission.id.clone(),
            receipt_id,
            source: submission.source,
            item_count: submission.items.len(),
            total_text_bytes: submission.total_text_bytes(),
            disposition,
        };
        let _ = self.eq_tx.send(SessionEvent::SubmissionReceipt {
            session_id: receipt.session_id.clone(),
            session_generation: receipt.session_generation,
            submission_id: receipt.submission_id.clone(),
            reservation_id: receipt.reservation_id.clone(),
            receipt_id: receipt.receipt_id.clone(),
            source: receipt.source,
            item_count: receipt.item_count,
            total_text_bytes: receipt.total_text_bytes,
            disposition: receipt.disposition.clone(),
        });
        if let Some(receipt_tx) = receipt_tx {
            let _ = receipt_tx.send(receipt);
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
        recent_submission_ids: &mut VecDeque<String>,
        recent_item_ids: &mut VecDeque<String>,
        paused: bool,
    ) -> bool {
        if let Err(reason) = validate_submission(&submission) {
            self.reject_submission(&submission.id, submission.sender_generation, reason);
            return false;
        }
        if recent_submission_ids.contains(&submission.id)
            || submission
                .items
                .iter()
                .any(|item| recent_item_ids.contains(&item.id))
        {
            self.reject_submission(
                &submission.id,
                submission.sender_generation,
                SubmissionRejectionReason::Duplicate,
            );
            return false;
        }
        let Some((next_items, next_bytes, next_images, next_image_bytes)) = queue_totals_after(
            &submission,
            *pending_items,
            *pending_bytes,
            *pending_images,
            *pending_image_bytes,
        ) else {
            self.reject_submission(
                &submission.id,
                submission.sender_generation,
                SubmissionRejectionReason::LimitExceeded,
            );
            return false;
        };
        // EQ is a projection boundary. Once durable custody has transferred,
        // a disconnected observer must not prevent Actor arbitration.
        let _ = self.eq_tx.send(SessionEvent::SubmissionQueued {
            session_id: self.session_id.clone(),
            submission_id: submission.id.clone(),
            sender_generation: submission.sender_generation,
            source: submission.source,
            item_count: submission.items.len(),
            total_text_bytes: submission.total_text_bytes(),
        });

        record_recent_identity(
            recent_submission_ids,
            submission.id.clone(),
            MAX_RECENT_SUBMISSION_IDS,
        );
        for item in &submission.items {
            record_recent_identity(recent_item_ids, item.id.clone(), MAX_RECENT_ITEM_IDS);
        }
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
    ) -> Option<StartedTurn> {
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

        let prepared_turn = if submission.common_kind() == Some(SubmissionKind::PreviewRequest) {
            None
        } else {
            match self
                .agent
                .prepare_session_turn(
                    &submission.items,
                    self.history.clone(),
                    self.model_context_limit,
                )
                .await
            {
                Ok(prepared) => Some(prepared),
                Err(crate::AgentError::ContextBudgetExceeded { .. }) => {
                    self.reject_submission(
                        &submission.id,
                        submission.sender_generation,
                        SubmissionRejectionReason::ContextBudgetExceeded,
                    );
                    return None;
                }
                Err(error) => {
                    let _ = self.eq_tx.send(SessionEvent::Error {
                        message: format!(
                            "failed to seal Provider request plan for {}: {error}",
                            submission.id
                        ),
                    });
                    self.reject_submission(
                        &submission.id,
                        submission.sender_generation,
                        SubmissionRejectionReason::InvalidStructure,
                    );
                    return None;
                }
            }
        };

        let turn_id = format!("{}_{}", self.turn_prefix, turn_counter);
        let structured = if submission.source == SubmissionSource::Compatibility {
            None
        } else {
            let record = match self.pending_store.get(&submission.id) {
                Ok(Some(record)) => record,
                Ok(None) => {
                    self.reject_submission(
                        &submission.id,
                        submission.sender_generation,
                        SubmissionRejectionReason::DurabilityUnavailable,
                    );
                    return None;
                }
                Err(error) => {
                    self.emit_custody_error("failed to load accepted submission", &error);
                    return None;
                }
            };
            if let Err(error) = self.pending_store.mark_running(&submission.id, &turn_id) {
                self.emit_custody_error("failed to mark structured submission running", &error);
                return None;
            }
            let active = ActiveStructuredTurn {
                submission_id: submission.id.clone(),
                receipt_id: record.receipt_id,
                session_generation: submission.sender_generation,
                turn_id: turn_id.clone(),
            };
            let _ = self.eq_tx.send(SessionEvent::StructuredTurnEvent {
                session_id: self.session_id.clone(),
                session_generation: active.session_generation,
                submission_id: active.submission_id.clone(),
                receipt_id: active.receipt_id.clone(),
                turn_id: active.turn_id.clone(),
                sequence: 0,
                payload: TurnEventPayload::Started,
            });
            Some(active)
        };

        let _ = self.eq_tx.send(SessionEvent::SubmissionStarted {
            session_id: self.session_id.clone(),
            submission_id: submission.id.clone(),
            sender_generation: submission.sender_generation,
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
            let token = CancellationToken::new();
            let preview_token = token.clone();
            let handle = tokio::spawn(async move {
                let result = tokio::select! {
                    () = preview_token.cancelled() => {
                        let completion = TurnCompletionStatus::Cancelled;
                        let _ = eq_tx.send(SessionEvent::TurnEvent {
                            session_id,
                            turn_id,
                            sequence: 1,
                            payload: TurnEventPayload::Completed {
                                status: completion.clone(),
                            },
                        });
                        return Some(TurnRecord {
                            new_messages: Vec::new(),
                            status: TurnRecordStatus::Cancelled,
                            completion,
                        });
                    }
                    result = agent.preview_request(message, history) => result,
                };
                let (completion, record_status) = match result {
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
                    payload: TurnEventPayload::Completed {
                        status: completion.clone(),
                    },
                });
                Some(TurnRecord {
                    new_messages: Vec::new(),
                    status: record_status,
                    completion,
                })
            });
            return Some(StartedTurn {
                handle,
                token,
                structured,
            });
        }

        if let Some(agent_mut) = Arc::get_mut(&mut self.agent) {
            agent_mut.set_append_prompt_opt(None);
        }

        let sequence = Arc::new(AtomicU64::new(1));
        let token = CancellationToken::new();
        let token_clone = token.clone();
        let agent = self.agent.clone();
        let eq_tx = self.eq_tx.clone();
        let prepared = prepared_turn.expect("non-preview submission must be prepared");
        let persistence = self.persistence.clone();
        let durable_persistence = self.durable_persistence.clone();
        let session_id = self.session_id.clone();
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
            }))
            .catch_unwind()
            .await;
            result_rx.await.ok()
        });
        Some(StartedTurn {
            handle,
            token,
            structured,
        })
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

fn record_recent_identity(identities: &mut VecDeque<String>, id: String, capacity: usize) {
    while identities.len() >= capacity {
        identities.pop_front();
    }
    identities.push_back(id);
}

fn validate_submission(submission: &StructuredSubmission) -> Result<(), SubmissionRejectionReason> {
    submission.validate()
}

fn queue_has_capacity(
    submission: &StructuredSubmission,
    pending_items: usize,
    pending_bytes: usize,
    pending_images: usize,
    pending_image_bytes: u64,
) -> bool {
    queue_totals_after(
        submission,
        pending_items,
        pending_bytes,
        pending_images,
        pending_image_bytes,
    )
    .is_some()
}

fn queue_totals_after(
    submission: &StructuredSubmission,
    pending_items: usize,
    pending_bytes: usize,
    pending_images: usize,
    pending_image_bytes: u64,
) -> Option<(usize, usize, usize, u64)> {
    let next_items = pending_items.checked_add(submission.items.len())?;
    let next_bytes = pending_bytes.checked_add(submission.total_text_bytes())?;
    let (submission_images, submission_image_bytes) = submission.image_totals();
    let next_images = pending_images.checked_add(submission_images)?;
    let next_image_bytes = pending_image_bytes.checked_add(submission_image_bytes)?;
    (next_items <= MAX_STEERING_QUEUE_ITEMS
        && next_bytes <= MAX_STEERING_QUEUE_BYTES
        && next_images <= MAX_STEERING_QUEUE_IMAGES
        && next_image_bytes <= MAX_STEERING_QUEUE_IMAGE_BYTES)
        .then_some((next_items, next_bytes, next_images, next_image_bytes))
}
