//! AppServerSession actor — bridges SQ→Agent→EQ (ADR-005 L2 seam).
//!
//! The session actor owns an [`Agent`] and runs a message loop:
//! - Receives [`SessionOp`] on the bounded SQ (cap=512)
//! - Drives agent turns via [`Agent::run_streaming`]
//! - Emits [`SessionEvent`] on the unbounded EQ

use std::collections::VecDeque;
use std::panic::AssertUnwindSafe;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use futures_util::FutureExt;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use talos_core::message::{AgentEvent, Message};
use talos_core::session::{
    MAX_STEERING_QUEUE_BYTES, MAX_STEERING_QUEUE_IMAGE_BYTES, MAX_STEERING_QUEUE_IMAGES,
    MAX_STEERING_QUEUE_ITEMS, SessionConfig, SessionEvent, SessionHandle, SessionOp,
    StructuredSubmission, SubmissionItem, SubmissionKind, SubmissionRejectionReason,
    SubmissionSource, TurnCompletionStatus, TurnEventPayload,
};
#[cfg(test)]
use talos_core::session::{
    MAX_SUBMISSION_BATCH_ITEMS, MAX_SUBMISSION_IMAGE_COUNT, SubmissionReceiptDisposition,
};
use talos_session::PendingSubmissionStore;

#[doc(hidden)]
pub use crate::background_jobs::BackgroundJobFinalizerHandle;
use crate::{ActivatedSkillContext, Agent, SteeringBoundaryAcknowledgement, SteeringBoundaryBatch};

mod custody;
mod lifecycle;
mod manual_compaction;
mod turn;

#[doc(hidden)]
pub use lifecycle::{
    RuntimeActiveTurnOutcome, RuntimeAdmissionClose, RuntimeAdmissionControl,
    RuntimeDurableReconciliationOutcome, RuntimeLifecycleSnapshot, RuntimeShutdownTurnPolicy,
};

#[cfg(test)]
#[allow(warnings)]
mod tests;

use turn::{
    DurableTurnPersistence, TurnForwarding, TurnPersistence, TurnRecord, TurnRecordStatus,
    run_turn_with_forwarding,
};

struct BackgroundPreparation {
    future: futures_util::future::BoxFuture<'static, Result<crate::PreparedSessionTurn, String>>,
    id: talos_core::background_job::BackgroundJobId,
    turn_id: String,
    token: CancellationToken,
}
impl Drop for BackgroundPreparation {
    fn drop(&mut self) {
        self.token.cancel();
    }
}

static NEXT_RUNTIME_SESSION_ID: AtomicU64 = AtomicU64::new(1);
const MAX_RECENT_SUBMISSION_IDS: usize = 1024;
const MAX_RECENT_ITEM_IDS: usize = 4096;

#[derive(Debug, Clone)]
struct ActiveStructuredTurn {
    submission_id: String,
    receipt_id: String,
    session_generation: u64,
    source: SubmissionSource,
    turn_id: String,
}

#[derive(Debug)]
struct BoundaryStructuredTurn {
    active: ActiveStructuredTurn,
    submission: StructuredSubmission,
}

struct StartedTurn {
    handle: JoinHandle<Option<TurnRecord>>,
    token: CancellationToken,
    structured: Option<ActiveStructuredTurn>,
    boundary_rx: mpsc::UnboundedReceiver<crate::AgentActorRequest>,
    boundary_ack_rx: mpsc::UnboundedReceiver<SteeringBoundaryAcknowledgement>,
}

enum UnstartedOutcome {
    Retained,
    Terminal,
}

fn immediate_started_turn(
    eq_tx: mpsc::UnboundedSender<SessionEvent>,
    session_id: String,
    turn_id: String,
    token: CancellationToken,
    structured: Option<ActiveStructuredTurn>,
    completion: TurnCompletionStatus,
) -> StartedTurn {
    let (_boundary_tx, boundary_rx) = mpsc::unbounded_channel();
    let (_boundary_ack_tx, boundary_ack_rx) = mpsc::unbounded_channel();
    let status = match &completion {
        TurnCompletionStatus::Success { .. } => TurnRecordStatus::Success,
        TurnCompletionStatus::Cancelled => TurnRecordStatus::Cancelled,
        TurnCompletionStatus::Error { .. } => TurnRecordStatus::Error,
    };
    let handle = tokio::spawn(async move {
        let _ = eq_tx.send(SessionEvent::TurnEvent {
            session_id,
            turn_id,
            sequence: 1,
            payload: TurnEventPayload::Completed {
                status: completion.clone(),
            },
        });
        Some(TurnRecord {
            committed_source: None,
            consumed_background_jobs: Vec::new(),
            completion_events: Vec::new(),
            persisted_entry_ids: Vec::new(),
            new_messages: Vec::new(),
            status,
            completion,
        })
    });
    StartedTurn {
        handle,
        token,
        structured,
        boundary_rx,
        boundary_ack_rx,
    }
}

/// Session actor that owns an [`Agent`] and processes commands from the SQ.
///
/// Created via [`AppServerSession::new`], which returns a [`SessionHandle`]
/// for the UI layer and the actor itself for spawning on a tokio task.
pub struct AppServerSession {
    agent: Arc<Agent>,
    background_jobs: crate::background_jobs::BackgroundJobSupervisor,
    sq_rx: tokio::sync::mpsc::Receiver<SessionOp>,
    eq_tx: mpsc::UnboundedSender<SessionEvent>,
    history: Vec<Message>,
    private_replays: Arc<crate::private_replay::PrivateReplayStore>,
    session_file: Option<PathBuf>,
    recovered_source: Option<crate::compaction::HistorySource>,
    post_turn_pressure: crate::checkpoint_progress::PostTurnPressure,
    persistence: Option<TurnPersistence>,
    durable_persistence: Option<DurableTurnPersistence>,
    pending_store: PendingSubmissionStore,
    session_id: String,
    session_generation: u64,
    turn_prefix: String,
    model_context_limit: u32,
    runtime_admission: Option<RuntimeAdmissionControl>,
    runtime_lease: Option<Arc<talos_session::SessionRuntimeLease>>,
    #[cfg(test)]
    start_commit_gate: Option<(Arc<tokio::sync::Notify>, Arc<tokio::sync::Notify>)>,
    #[cfg(test)]
    fork_worker_gate: Option<(Arc<tokio::sync::Notify>, Arc<std::sync::atomic::AtomicBool>)>,
    #[cfg(test)]
    boundary_ack_cancel_gate: Option<Arc<tokio::sync::Notify>>,
}

impl AppServerSession {
    /// Creates a new generation-zero session actor with the given agent and configuration.
    ///
    /// Product composition roots that replace an Actor must call [`Self::set_generation`]
    /// before spawning it. Returns a [`SessionHandle`] and the actor itself.
    pub fn new(mut agent: Agent, config: SessionConfig) -> (SessionHandle, Self) {
        let (sq_tx, sq_rx) = tokio::sync::mpsc::channel(512);
        let (eq_tx, eq_rx) = mpsc::unbounded_channel();

        let handle = SessionHandle { sq_tx, eq_rx };
        let instance_id = NEXT_RUNTIME_SESSION_ID.fetch_add(1, Ordering::Relaxed);
        let session_id = format!("runtime_{}_{}", std::process::id(), instance_id);
        let background_jobs = crate::background_jobs::BackgroundJobSupervisor::new(
            eq_tx.clone(),
            session_id.clone(),
            0,
        );
        agent.set_background_job_host(Arc::new(background_jobs.clone()));
        agent.register_process_tool(background_jobs.clone());
        let pending_session_file = config
            .workspace_root
            .join(".talos")
            .join("runtime")
            .join(format!("{session_id}.tlog"));
        let pending_store =
            PendingSubmissionStore::for_session_file(&pending_session_file, &session_id);

        let actor = Self {
            agent: Arc::new(agent),
            background_jobs,
            sq_rx,
            eq_tx,
            history: config.initial_history,
            private_replays: Default::default(),
            session_file: None,
            persistence: None,
            durable_persistence: None,
            recovered_source: None,
            post_turn_pressure: Default::default(),
            pending_store,
            session_id,
            session_generation: 0,
            turn_prefix: format!("turn_{}_{}", std::process::id(), instance_id),
            model_context_limit: config.model_context_limit,
            runtime_admission: None,
            runtime_lease: None,
            #[cfg(test)]
            start_commit_gate: None,
            #[cfg(test)]
            fork_worker_gate: None,
            #[cfg(test)]
            boundary_ack_cancel_gate: None,
        };

        (handle, actor)
    }

    /// Assigns the authoritative generation before this Actor is spawned.
    pub fn set_generation(&mut self, generation: u64) {
        self.session_generation = generation;
        self.private_replays = Default::default();
        self.post_turn_pressure = Default::default();
        self.background_jobs
            .set_identity(self.session_id.clone(), generation);
    }

    /// Installs the runtime facade's admission/start linearization seam.
    #[doc(hidden)]
    pub fn set_runtime_admission(&mut self, admission: RuntimeAdmissionControl) {
        self.runtime_admission = Some(admission);
    }

    #[cfg(test)]
    fn set_start_commit_gate(
        &mut self,
        reached: Arc<tokio::sync::Notify>,
        release: Arc<tokio::sync::Notify>,
    ) {
        self.start_commit_gate = Some((reached, release));
    }

    /// Returns the authoritative generation assigned by the composition root.
    #[must_use]
    pub fn generation(&self) -> u64 {
        self.session_generation
    }

    /// Returns the built-in background-job finalizer owned by this Session.
    #[doc(hidden)]
    #[must_use]
    pub fn background_job_finalizer(&self) -> BackgroundJobFinalizerHandle {
        self.background_jobs.finalizer_handle()
    }

    /// Records the legacy session path and invalidates private replay on change.
    ///
    /// Automatic head trimming is disabled to preserve restart history. This
    /// does not configure turn persistence; use `set_persistence` instead.
    pub fn set_session_paths(&mut self, file: PathBuf, _dir: PathBuf) {
        if self.session_file.as_ref() != Some(&file) {
            self.private_replays = Default::default();
        }
        self.session_file = Some(file);
    }

    /// Assigns the durable session that owns successful turn-message writes.
    pub fn set_persistence(
        &mut self,
        session: talos_session::Session,
        metadata: talos_session::SessionMetadata,
    ) {
        self.pending_store = PendingSubmissionStore::for_session(&session);
        self.recovered_source = None;
        if self.session_id != session.id.to_string() {
            self.private_replays = Default::default();
        }
        self.session_id = session.id.to_string();
        self.background_jobs
            .set_identity(self.session_id.clone(), self.session_generation);
        self.persistence = Some(TurnPersistence {
            session,
            metadata,
            history_owner: Arc::new(tokio::sync::Mutex::new(None)),
            recovery_preparation: Arc::new(tokio::sync::Mutex::new(None)),
        });
    }

    /// Assigns an atomic durable session used by the embedded runtime.
    pub fn set_durable_persistence(
        &mut self,
        session: talos_session::DurableSession,
        policy: talos_session::PersistencePolicy,
    ) {
        self.recovered_source = None;
        let session_id = session.id().to_string();
        self.pending_store =
            PendingSubmissionStore::for_session_file(session.file_path(), &session_id);
        if self.session_id != session_id {
            self.private_replays = Default::default();
        }
        self.session_id = session_id;
        self.background_jobs
            .set_identity(self.session_id.clone(), self.session_generation);
        self.durable_persistence = Some(DurableTurnPersistence {
            session,
            policy,
            recovery_preparation: Arc::new(tokio::sync::Mutex::new(None)),
        });
    }

    /// Bind recovered messages to entries in the configured durable session.
    /// Exact message equality and the current session namespace are required;
    /// oversized or inconsistent snapshots disable durable source reuse.
    pub fn set_recovered_history_sources(
        &mut self,
        snapshot: talos_session::DurableHistorySnapshot,
    ) {
        self.recovered_source = None;
        let Some(durable) = &self.durable_persistence else {
            return;
        };
        let entries = snapshot.entries();
        if !durable.session.owns_history_snapshot(&snapshot)
            || entries.len() > 8192
            || entries.len() != self.history.len()
            || entries
                .iter()
                .zip(&self.history)
                .any(|((_, message), raw)| message != raw)
        {
            return;
        }
        let namespace = durable.session.id();
        let ids = entries
            .iter()
            .map(|(id, _)| format!("tlog:{namespace}:{id}"))
            .collect();
        self.recovered_source = crate::compaction::HistorySource::bind(
            self.session_generation,
            self.history.len() as u64,
            &self.history,
            ids,
        );
    }

    /// Bind a storage-issued recovered snapshot cooperatively before running the Actor.
    ///
    /// Unlike the bounded synchronous compatibility setter, this accepts long
    /// histories under a caller-owned deadline. A dropped or expired operation
    /// leaves recovery identity reuse disabled. Metadata grows with history;
    /// individual comparisons and allocations are not hard-real-time operations.
    pub async fn set_recovered_history_sources_until(
        &mut self,
        snapshot: talos_session::DurableHistorySnapshot,
        deadline: tokio::time::Instant,
    ) -> bool {
        self.recovered_source = None;
        if tokio::time::Instant::now() >= deadline {
            return false;
        }
        let Some(durable) = &self.durable_persistence else {
            return false;
        };
        if !durable.session.owns_history_snapshot(&snapshot)
            || snapshot.entries().len() != self.history.len()
        {
            return false;
        }
        let namespace = durable.session.id();
        let mut ids = Vec::new();
        for ((id, message), raw) in snapshot.entries().iter().zip(&self.history) {
            tokio::task::yield_now().await;
            if tokio::time::Instant::now() >= deadline || message != raw {
                return false;
            }
            ids.push(format!("tlog:{namespace}:{id}"));
        }
        self.recovered_source = crate::compaction::HistorySource::bind_until(
            self.session_generation,
            self.history.len() as u64,
            &self.history,
            ids,
            deadline,
        )
        .await;
        self.recovered_source.is_some()
    }

    /// Retains host startup ownership until execution or Actor disposal.
    /// Requires matching configured persistence and refuses replacing an existing lease.
    /// `run` validates again, including after any persistence configuration change.
    pub fn adopt_runtime_lease(
        &mut self,
        lease: talos_session::SessionRuntimeLease,
    ) -> Result<(), talos_session::SessionError> {
        if self.runtime_lease.is_some() {
            return Err(talos_session::SessionError::DurableTurn(
                "Actor already owns a runtime lease".into(),
            ));
        }
        let transcript = self
            .persistence
            .as_ref()
            .map(|p| p.session.file_path.as_path())
            .or_else(|| {
                self.durable_persistence
                    .as_ref()
                    .map(|p| p.session.file_path())
            })
            .ok_or_else(|| {
                talos_session::SessionError::DurableTurn(
                    "runtime lease requires persistence".into(),
                )
            })?;
        lease.check_transcript(transcript)?;
        self.runtime_lease = Some(Arc::new(lease));
        Ok(())
    }

    /// Runs with ownership acquired before host identity/history preparation.
    /// The lease is retained without reacquisition and checked against persistence
    /// before any Actor recovery or execution. A mismatch refuses startup.
    pub async fn run_with_runtime_lease(&mut self, lease: talos_session::SessionRuntimeLease) {
        self.runtime_lease = Some(Arc::new(lease));
        self.run().await;
    }

    /// Runs the session actor until shutdown or SQ disconnect.
    pub async fn run(&mut self) {
        let transcript = self
            .persistence
            .as_ref()
            .map(|p| p.session.file_path.as_path())
            .or_else(|| {
                self.durable_persistence
                    .as_ref()
                    .map(|p| p.session.file_path())
            });
        if let Some(transcript) = transcript {
            let ownership = match self.runtime_lease.as_ref() {
                Some(lease) => lease.check_transcript(transcript),
                None => talos_session::SessionRuntimeLease::acquire(transcript).map(|lease| {
                    self.runtime_lease = Some(Arc::new(lease));
                }),
            };
            match ownership {
                Ok(()) => {}
                Err(error) => {
                    self.emit_custody_error("cannot start Session execution or recovery", &error);
                    self.runtime_lease = None;
                    return;
                }
            }
        }
        if transcript.is_none() && self.runtime_lease.is_some() {
            self.emit_custody_error(
                "cannot start Session execution or recovery",
                &"runtime lease requires persistence",
            );
            self.runtime_lease = None;
            return;
        }
        self.run_owned().await;
        self.runtime_lease = None;
    }

    async fn run_owned(&mut self) {
        let reconciliation_ok = self.reconcile_running_submissions();
        let recovery_fenced = !reconciliation_ok
            || match self.pending_store.recover_running() {
                Ok(records) if records.is_empty() => false,
                Ok(_) => {
                    self.emit_custody_error(
                        "session stopped because Running custody has no terminal transcript proof",
                        &"reconcile the original Turn before resuming",
                    );
                    true
                }
                Err(error) => {
                    self.emit_custody_error(
                        "session stopped because Running custody could not be inspected",
                        &error,
                    );
                    true
                }
            };
        if recovery_fenced {
            // Preserve observational reconciliation while refusing execution
            // whose quota cannot be accounted for from orphan Running rows.
            while let Some(op) = self.sq_rx.recv().await {
                match op {
                    SessionOp::SubmitStructured { submission } => {
                        self.emit_admission_rejection(
                            &submission,
                            SubmissionRejectionReason::SessionClosed,
                            None,
                        );
                    }
                    SessionOp::SubmitStructuredTracked {
                        submission,
                        receipt_tx,
                    } => {
                        self.emit_admission_rejection(
                            &submission,
                            SubmissionRejectionReason::SessionClosed,
                            receipt_tx.as_ref(),
                        );
                    }
                    SessionOp::ReconcileStructured { submission }
                    | SessionOp::SubmitStructuredReconcile { submission } => {
                        self.reconcile_submission(&submission, None);
                    }
                    SessionOp::ReconcileStructuredTracked {
                        submission,
                        receipt_tx,
                    }
                    | SessionOp::SubmitStructuredReconcileTracked {
                        submission,
                        receipt_tx,
                    } => {
                        self.reconcile_submission(&submission, receipt_tx.as_ref());
                    }
                    SessionOp::Shutdown => break,
                    _ => self.emit_custody_error(
                        "execution is fenced by unresolved Running custody",
                        &"reconcile the original Turn before resuming",
                    ),
                }
            }
            return;
        }

        enum ForkOutput {
            Published(String),
            Captured(talos_core::session::OrdinaryForkCapture),
        }
        enum ForkReceipt {
            Published(mpsc::UnboundedSender<Result<String, String>>),
            Captured(
                mpsc::UnboundedSender<Result<talos_core::session::OrdinaryForkCapture, String>>,
            ),
        }
        impl ForkReceipt {
            fn send(&self, result: Result<ForkOutput, String>) {
                match self {
                    Self::Published(tx) => {
                        let _ = tx.send(result.and_then(|value| match value {
                            ForkOutput::Published(id) => Ok(id),
                            _ => Err("fork worker result mismatch".into()),
                        }));
                    }
                    Self::Captured(tx) => {
                        let _ = tx.send(result.and_then(|value| match value {
                            ForkOutput::Captured(snapshot) => Ok(snapshot),
                            _ => Err("fork worker result mismatch".into()),
                        }));
                    }
                }
            }
        }
        struct ForkWork {
            task: JoinHandle<Result<ForkOutput, String>>,
            cancel: CancellationToken,
            receipt: ForkReceipt,
        }
        impl Drop for ForkWork {
            fn drop(&mut self) {
                self.cancel.cancel();
            }
        }
        let mut fork_work: Option<ForkWork> = None;
        let mut turn_counter: u64 = 0;
        let mut submission_counter: u64 = 0;
        let mut pending = VecDeque::<StructuredSubmission>::new();
        let mut pending_items = 0_usize;
        let mut pending_bytes = 0_usize;
        let mut pending_images = 0_usize;
        let mut pending_image_bytes = 0_u64;
        let mut recent_submission_ids = VecDeque::<String>::new();
        let mut recent_item_ids = VecDeque::<String>::new();
        let completion_notify = self.background_jobs.completion_notify();
        let mut current_completion: Option<(talos_core::background_job::BackgroundJobId, String)> =
            None;
        let mut background_preparation: Option<BackgroundPreparation> = None;
        let mut current_turn: Option<JoinHandle<Option<TurnRecord>>> = None;
        let mut current_submission_size: Option<(usize, usize, usize, u64)> = None;
        let mut current_structured: Option<ActiveStructuredTurn> = None;
        let mut injected_structured = Vec::<BoundaryStructuredTurn>::new();
        let mut delivered_boundary: Option<BoundaryStructuredTurn> = None;
        let mut current_boundary_rx: Option<mpsc::UnboundedReceiver<crate::AgentActorRequest>> =
            None;
        let mut current_boundary_ack_rx: Option<
            mpsc::UnboundedReceiver<SteeringBoundaryAcknowledgement>,
        > = None;
        let mut cancel_token: Option<CancellationToken> = None;
        let mut resume_user_after_cancel = false;
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
        let mut compact = manual_compaction::ManualCompaction::new(self.session_generation);

        loop {
            if shutting_down {
                compact.cancel(self);
                if let Some(work) = &fork_work {
                    work.cancel.cancel();
                }
            }
            if current_turn.is_none()
                && background_preparation.is_none()
                && fork_work.is_none()
                && !shutting_down
            {
                compact.start(self);
            }
            if current_turn.is_none()
                && background_preparation.is_none()
                && fork_work.is_none()
                && !compact.is_running()
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
                let start_token = CancellationToken::new();
                #[cfg(test)]
                if let Some((reached, release)) = &self.start_commit_gate {
                    reached.notify_one();
                    release.notified().await;
                }
                let start_committed = self
                    .runtime_admission
                    .as_ref()
                    .is_none_or(|admission| admission.commit_start(start_token.clone()));
                if !start_committed {
                    pending.push_front(submission);
                    paused = true;
                    continue;
                }
                turn_counter = turn_counter.saturating_add(1);
                match self
                    .start_submission(submission.clone(), turn_counter, start_token)
                    .await
                {
                    Ok(started) => {
                        current_turn = Some(started.handle);
                        current_submission_size = Some(submission_size);
                        current_structured = started.structured;
                        current_boundary_rx = Some(started.boundary_rx);
                        current_boundary_ack_rx = Some(started.boundary_ack_rx);
                        cancel_token = Some(started.token);
                    }
                    Err(outcome) => {
                        if let Some(admission) = &self.runtime_admission {
                            admission.finish_active(None);
                        }
                        if matches!(outcome, UnstartedOutcome::Terminal) {
                            pending_items = pending_items.saturating_sub(submission_size.0);
                            pending_bytes = pending_bytes.saturating_sub(submission_size.1);
                            pending_images = pending_images.saturating_sub(submission_size.2);
                            pending_image_bytes =
                                pending_image_bytes.saturating_sub(submission_size.3);
                            continue;
                        } else {
                            pending.push_front(submission);
                            paused = true;
                        }
                    }
                }
            }

            if !shutting_down
                && !paused
                && pending.is_empty()
                && current_turn.is_none()
                && fork_work.is_none()
                && !compact.is_running()
                && background_preparation.is_none()
                && self.sq_rx.is_empty()
                && let Some((id, terminal)) = self.background_jobs.pending_completion()
            {
                let token = CancellationToken::new();
                if !self
                    .runtime_admission
                    .as_ref()
                    .is_none_or(|admission| admission.commit_start(token.clone()))
                {
                    self.background_jobs.retire_completion(&id);
                    paused = true;
                    continue;
                }
                turn_counter = turn_counter.saturating_add(1);
                let turn_id = format!("background-{}-{turn_counter}", self.session_generation);
                background_preparation =
                    Some(self.prepare_background_completion(id, terminal, turn_id, token));
            }

            if shutting_down
                && background_preparation.is_none()
                && fork_work.is_none()
                && current_turn.is_none()
                && !compact.is_running()
                && pending.is_empty()
            {
                break;
            }

            tokio::select! {
                result = async {
                    match background_preparation.as_mut() {
                        Some(work) => work.future.as_mut().await,
                        None => Err("background preparation unavailable".into()),
                    }
                }, if background_preparation.is_some() => {
                    let Some(mut work) = background_preparation.take() else { continue; };
                    // Ready input wins even if the preparation future was selected first.
                    if result.is_ok() && !self.sq_rx.is_empty() {
                        if let Some(admission) = &self.runtime_admission { admission.finish_active(None); }
                        continue;
                    }
                    match result {
                        Ok(prepared) if !work.token.is_cancelled() && self.background_jobs.start_completion(&work.id) => {
                            let token = std::mem::replace(&mut work.token, CancellationToken::new());
                            let _ = self.eq_tx.send(SessionEvent::TurnEvent {
                                session_id: self.session_id.clone(), turn_id: work.turn_id.clone(), sequence: 0,
                                payload: TurnEventPayload::Started,
                            });
                            let started = self.spawn_prepared_turn(prepared, work.turn_id.clone(), token, None);
                            current_completion = Some((work.id.clone(), work.turn_id.clone()));
                            current_turn = Some(started.handle);
                            current_boundary_rx = Some(started.boundary_rx);
                            current_boundary_ack_rx = Some(started.boundary_ack_rx);
                            cancel_token = Some(started.token);
                        }
                        result => {
                            // A preparation error has not consumed this pending completion.
                            // Keep custody until explicit stop or a later admitted retry.
                            if result.is_ok() || work.token.is_cancelled() {
                                self.background_jobs.retire_completion(&work.id);
                            }
                            paused = true;
                            let error = result.err().unwrap_or_else(|| "background completion was revoked before dispatch".into());
                            if let Some(admission) = &self.runtime_admission {
                                admission.finish_active(Some(&TurnCompletionStatus::Error { message: error.clone() }));
                            }
                            let _ = self.eq_tx.send(SessionEvent::Error { message: format!("background continuation preparation paused: {error}") });
                        }
                    }
                }
                () = completion_notify.notified() => {}
                result = async { match fork_work.as_mut() {
                    Some(work) => Some((&mut work.task).await),
                    None => None,
                } }, if fork_work.is_some() => {
                    if let Some(work) = fork_work.take() {
                        let result = result.and_then(Result::ok).unwrap_or_else(|| Err("durable fork worker failed".into()));
                        work.receipt.send(result);
                    }
                }
                result = compact.wait(), if compact.has_pending_work() => {
                    compact.complete(self, result);
                }
                completed = async {
                    match current_turn.as_mut() {
                        Some(turn) => Some(turn.await),
                        None => None,
                    }
                }, if current_turn.is_some() => {
                    current_turn = None;
                    let completed_background = current_completion.take();
                    compact.cancel_queued_model(self);
                    cancel_token = None;
                    current_boundary_rx = None;
                    if let Some(rx) = current_boundary_ack_rx.as_mut() {
                        while let Ok(submission_id) = rx.try_recv() {
                            self.acknowledge_boundary_submission(
                                submission_id,
                                &mut delivered_boundary,
                                &mut injected_structured,
                            );
                        }
                    }
                    current_boundary_ack_rx = None;
                    if let Some((items, bytes, images, image_bytes)) = current_submission_size.take() {
                        pending_items = pending_items.saturating_sub(items);
                        pending_bytes = pending_bytes.saturating_sub(bytes);
                        pending_images = pending_images.saturating_sub(images);
                        pending_image_bytes = pending_image_bytes.saturating_sub(image_bytes);
                    }
                    let structured = current_structured.take();
                    match completed.and_then(Result::ok).flatten() {
                        Some(mut record) => {
                            let completion_events = std::mem::take(&mut record.completion_events);
                            let status = record.status;
                            let completion = record.completion.clone();
                            let consumed_background_jobs = std::mem::take(&mut record.consumed_background_jobs);
                            self.commit_turn_record(record);
                            self.background_jobs.consume_completions(&consumed_background_jobs);
                            let mut custody_ok = true;
                            if let Some(active) = structured.as_ref() {
                                custody_ok &= self.finish_structured_turn(active, &completion);
                            }
                            for injected in injected_structured.drain(..) {
                                subtract_submission_totals(
                                    &injected.submission,
                                    &mut pending_items,
                                    &mut pending_bytes,
                                    &mut pending_images,
                                    &mut pending_image_bytes,
                                );
                                custody_ok &= self.finish_injected_structured_turn(
                                    &injected.active,
                                    &completion,
                                );
                            }
                            if let Some(unacknowledged) = delivered_boundary.take() {
                                subtract_submission_totals(
                                    &unacknowledged.submission,
                                    &mut pending_items,
                                    &mut pending_bytes,
                                    &mut pending_images,
                                    &mut pending_image_bytes,
                                );
                                custody_ok = false;
                                custody_ok &= self.finish_unacknowledged_boundary(&unacknowledged.active);
                            }
                            if let Some(admission) = &self.runtime_admission {
                                admission.finish_active(Some(&completion));
                            }
                            if let Some((id, _)) = &completed_background {
                                self.background_jobs.retire_completion(id);
                            }
                            for event in completion_events {
                                let _ = self.eq_tx.send(event);
                            }
                            let resume_queued_user = resume_user_after_cancel
                                && status == TurnRecordStatus::Cancelled
                                && pending.iter().any(|submission| submission.source == SubmissionSource::User);
                            paused = (status != TurnRecordStatus::Success && !resume_queued_user)
                                || !custody_ok;
                            if !custody_ok {
                                // Do not accept new work after releasing in-memory
                                // accounting for a still-Running journal identity.
                                // Reconstruction reconciles the terminal transcript.
                                shutting_down = true;
                                self.release_in_memory_pending_on_shutdown(&mut pending);
                                if let Err(error) = self.pending_store.pause_unstarted() {
                                    self.emit_custody_error("failed to pause pending work after custody failure", &error);
                                }
                            }
                        }
                        None => {
                            let completion = TurnCompletionStatus::Error {
                                message: "turn task ended without a completion record".into(),
                            };
                            if let Some(active) = structured.as_ref() {
                                let _ = self.finish_structured_turn(active, &completion);
                            }
                            for injected in injected_structured.drain(..) {
                                subtract_submission_totals(
                                    &injected.submission,
                                    &mut pending_items,
                                    &mut pending_bytes,
                                    &mut pending_images,
                                    &mut pending_image_bytes,
                                );
                                let _ = self.finish_injected_structured_turn(
                                    &injected.active,
                                    &completion,
                                );
                            }
                            if let Some(unacknowledged) = delivered_boundary.take() {
                                subtract_submission_totals(
                                    &unacknowledged.submission,
                                    &mut pending_items,
                                    &mut pending_bytes,
                                    &mut pending_images,
                                    &mut pending_image_bytes,
                                );
                                let _ = self.finish_unacknowledged_boundary(&unacknowledged.active);
                            }
                            if let Some(admission) = &self.runtime_admission {
                                admission.finish_active(None);
                            }
                            if let Some((id, _)) = &completed_background {
                                self.background_jobs.retire_completion(id);
                            }
                            paused = true;
                            shutting_down = true;
                            self.release_in_memory_pending_on_shutdown(&mut pending);
                            if let Err(error) = self.pending_store.pause_unstarted() {
                                self.emit_custody_error("failed to pause pending work after missing completion record", &error);
                            }
                        }
                    }
                    resume_user_after_cancel = false;
                }
                op = self.sq_rx.recv(), if !shutting_down => {
                    let Some(op) = op else {
                        self.background_jobs.invalidate_completions();
                        if background_preparation.take().is_some()
                            && let Some(admission) = &self.runtime_admission
                        {
                            admission.finish_active(None);
                        }
                        shutting_down = true;
                        compact.cancel(self);
                        let should_interrupt = self.runtime_admission.as_ref().is_none_or(|control| {
                            !matches!(
                                control.shutdown_policy(),
                                Some(RuntimeShutdownTurnPolicy::FinishCurrent)
                            )
                        });
                        if should_interrupt && let Some(token) = cancel_token.take() {
                            token.cancel();
                        }
                        let rejected_pending = pending.len();
                        self.release_in_memory_pending_on_shutdown(&mut pending);
                        let reconciliation_succeeded = self.pending_store.pause_unstarted().is_ok();
                        if let Some(control) = &self.runtime_admission {
                            control.record_reconciliation(
                                rejected_pending,
                                reconciliation_succeeded,
                            );
                        }
                        pending_items = current_submission_size.map_or(0, |size| size.0);
                        pending_bytes = current_submission_size.map_or(0, |size| size.1);
                        pending_images = current_submission_size.map_or(0, |size| size.2);
                        pending_image_bytes = current_submission_size.map_or(0, |size| size.3);
                        continue;
                    };
                    let preempts_preparation = match &op {
                        SessionOp::InterruptTurn { session_generation, turn_id } => *session_generation == self.session_generation
                            && background_preparation.as_ref().is_some_and(|work| work.turn_id == *turn_id),
                        SessionOp::Submit { .. } | SessionOp::SubmitMultimodal { .. } | SessionOp::PreviewRequest { .. }
                        | SessionOp::SubmitStructured { .. } | SessionOp::SubmitStructuredTracked { .. }
                        | SessionOp::SubmitStructuredReconcile { .. } | SessionOp::SubmitStructuredReconcileTracked { .. }
                        | SessionOp::SetSkillContext { .. } | SessionOp::ForkDurable { .. }
                        | SessionOp::ForkHistory { .. } | SessionOp::CaptureOrdinaryFork { .. } | SessionOp::Compact { .. }
                        | SessionOp::Interrupt | SessionOp::Shutdown => true,
                        _ => false,
                    };
                    if preempts_preparation && let Some(work) = background_preparation.take() {
                        if matches!(&op, SessionOp::InterruptTurn { .. }) {
                            self.background_jobs.retire_completion(&work.id);
                        }
                        if let Some(admission) = &self.runtime_admission { admission.finish_active(None); }
                        drop(work);
                    }
                    if let Some(work) = &fork_work {
                        if matches!(&op, SessionOp::Interrupt | SessionOp::Shutdown) {
                            work.cancel.cancel();
                        } else {
                            match &op {
                                SessionOp::Compact { session_generation, operation_id } => {
                                    let _ = self.eq_tx.send(SessionEvent::CompactStatus {
                                        session_id: self.session_id.clone(), session_generation: self.session_generation,
                                        operation_id: *operation_id,
                                        state: if *session_generation == self.session_generation { talos_core::session::CompactState::Busy } else { talos_core::session::CompactState::StaleGeneration },
                                    });
                                    continue;
                                }
                                SessionOp::SubmitStructured { submission } => self.emit_admission_rejection(submission, SubmissionRejectionReason::SessionBusy, None),
                                SessionOp::SubmitStructuredTracked { submission, receipt_tx } => self.emit_admission_rejection(submission, SubmissionRejectionReason::SessionBusy, receipt_tx.as_ref()),
                                SessionOp::ReconcileStructured { submission } | SessionOp::SubmitStructuredReconcile { submission } => {
                                    self.reconcile_submission(submission, None);
                                    continue;
                                }
                                SessionOp::ReconcileStructuredTracked { submission, receipt_tx } | SessionOp::SubmitStructuredReconcileTracked { submission, receipt_tx } => {
                                    self.reconcile_submission(submission, receipt_tx.as_ref());
                                    continue;
                                }
                                _ => {}
                            }
                            if let SessionOp::ForkDurable { receipt_tx: Some(receipt), .. } | SessionOp::ForkHistory { receipt_tx: Some(receipt), .. } = &op {
                                let _ = receipt.send(Err("durable fork is already running".into()));
                            }
                            if let SessionOp::CaptureOrdinaryFork { receipt_tx: Some(receipt), .. } = &op {
                                let _ = receipt.send(Err("fork capture is already running".into()));
                            }
                            let _ = self.eq_tx.send(SessionEvent::Error { message: "session is busy creating a durable fork".into() });
                            continue;
                        }
                    }
                    match op {
                        SessionOp::CancelCompact { session_generation, operation_id } => {
                            compact.cancel_operation(self, session_generation, operation_id);
                        }
                        SessionOp::Compact { session_generation, operation_id } => {
                            compact.admit(self, session_generation, operation_id);
                        }
                        SessionOp::Submit { message } => {
                            submission_counter = submission_counter.saturating_add(1);
                            let submission = compatibility_submission(
                                submission_counter,
                                self.session_generation,
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
                            ) {
                                paused = false;
                            }
                        }
                        SessionOp::SubmitMultimodal { text, attachments } => {
                            submission_counter = submission_counter.saturating_add(1);
                            let submission = compatibility_submission(
                                submission_counter,
                                self.session_generation,
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
                            ) {
                                paused = false;
                            }
                        }
                        SessionOp::PreviewRequest { message } => {
                            submission_counter = submission_counter.saturating_add(1);
                            let submission = compatibility_submission(
                                submission_counter,
                                self.session_generation,
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
                                receipt_tx.as_ref(),
                            ) && resumes
                            {
                                paused = false;
                            }
                        }
                        SessionOp::ReconcileStructured { submission }
                        | SessionOp::SubmitStructuredReconcile { submission } => {
                            self.reconcile_submission(&submission, None);
                        }
                        SessionOp::ReconcileStructuredTracked {
                            submission,
                            receipt_tx,
                        }
                        | SessionOp::SubmitStructuredReconcileTracked {
                            submission,
                            receipt_tx,
                        } => {
                            self.reconcile_submission(&submission, receipt_tx.as_ref());
                        }
                        SessionOp::Interrupt => {
                            self.background_jobs.invalidate_completions();
                            compact.cancel(self);
                            if let Some(token) = &cancel_token {
                                token.cancel();
                            } else if !pending.is_empty() {
                                if let Err(error) = self.pending_store.pause_unstarted() {
                                    self.emit_custody_error("failed to pause pending work", &error);
                                }
                                paused = true;
                            }
                        }
                        SessionOp::InterruptTurn {
                            session_generation,
                            turn_id,
                        } => {
                            let matches = session_generation == self.session_generation
                                && (current_structured.as_ref().is_some_and(|active| active.turn_id == turn_id)
                                    || current_completion.as_ref().is_some_and(|(_, active)| *active == turn_id));
                            if matches && let Some(token) = &cancel_token {
                                resume_user_after_cancel = true;
                                token.cancel();
                            }
                        }
                        SessionOp::CancelPausedSubmission {
                            session_generation,
                            submission_id,
                        } => {
                            if current_turn.is_none()
                                && session_generation == self.session_generation
                                && self.cancel_paused_submission(
                                    &submission_id,
                                    &mut pending,
                                    &mut pending_items,
                                    &mut pending_bytes,
                                    &mut pending_images,
                                    &mut pending_image_bytes,
                                )
                            {
                                paused = false;
                            }
                        }
                        SessionOp::CaptureOrdinaryFork { through_entry, receipt_tx } => {
                            let Some(receipt_tx) = receipt_tx else { continue; };
                            if current_turn.is_some() || !compact.is_idle_for_fork() || !pending.is_empty() || shutting_down {
                                let _ = receipt_tx.send(Err("ordinary fork requires an idle session".into()));
                                continue;
                            }
                            if receipt_tx.is_closed() { continue; }
                            let (Some(persistence), Some(lease)) = (&self.persistence, self.runtime_lease.clone()) else {
                                let _ = receipt_tx.send(Err("ordinary fork requires transcript ownership".into()));
                                continue;
                            };
                            let source = persistence.session.clone();
                            let observer = receipt_tx.clone();
                            let admission = self.runtime_admission.clone();
                            let cancel = CancellationToken::new();
                            let stopped = cancel.clone();
                            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
                            #[cfg(test)]
                            let gate = self.fork_worker_gate.clone();
                            let task = tokio::task::spawn_blocking(move || {
                                #[cfg(test)]
                                if let Some((reached, release)) = gate {
                                    reached.notify_one();
                                    while !release.load(std::sync::atomic::Ordering::SeqCst) {
                                        if stopped.is_cancelled() || observer.is_closed() || std::time::Instant::now() >= deadline {
                                            return Err("ordinary fork cancelled at test gate".into());
                                        }
                                        std::thread::park_timeout(std::time::Duration::from_millis(1));
                                    }
                                }
                                source.capture_ordinary_fork(&lease, through_entry.as_deref(), deadline,
                                    || stopped.is_cancelled() || observer.is_closed() || admission.as_ref().is_some_and(|control| !control.is_open()))
                                    .map(|snapshot| ForkOutput::Captured(talos_core::session::OrdinaryForkCapture {
                                        bytes: snapshot.bytes,
                                        tail_id: snapshot.tail.id,
                                        runtime_identity: snapshot.runtime_identity.map(|identity| (identity.provider, identity.model, identity.variant)),
                                    })).map_err(|_| "ordinary fork capture unavailable".to_owned())
                            });
                            fork_work = Some(ForkWork { task, cancel, receipt: ForkReceipt::Captured(receipt_tx) });
                        }
                        SessionOp::ForkHistory { child_id, child_branch, through_entry, receipt_tx } => {
                            let Some(receipt_tx) = receipt_tx else { continue; };
                            if current_turn.is_some() || !compact.is_idle_for_fork() || !pending.is_empty() || shutting_down {
                                let _ = receipt_tx.send(Err("history fork requires an idle session".into()));
                                continue;
                            }
                            if receipt_tx.is_closed() { continue; }
                            let (Some(persistence), Some(lease)) = (&self.persistence, self.runtime_lease.clone()) else {
                                let _ = receipt_tx.send(Err("history fork requires transcript runtime ownership".into()));
                                continue;
                            };
                            let source = persistence.session.clone();
                            let observer = receipt_tx.clone();
                            let admission = self.runtime_admission.clone();
                            let cancel = CancellationToken::new();
                            let stopped = cancel.clone();
                            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
                            #[cfg(test)]
                            let gate = self.fork_worker_gate.clone();
                            let task = tokio::task::spawn_blocking(move || {
                                #[cfg(test)]
                                if let Some((reached, release)) = gate {
                                    reached.notify_one();
                                    while !release.load(std::sync::atomic::Ordering::SeqCst) {
                                        if stopped.is_cancelled() || observer.is_closed() || std::time::Instant::now() >= deadline {
                                            return Err("history fork cancelled at test gate".into());
                                        }
                                        std::thread::park_timeout(std::time::Duration::from_millis(1));
                                    }
                                }
                                let id = uuid::Uuid::parse_str(&child_id).map_err(|_| "invalid child identity".to_owned())?;
                                let branch = uuid::Uuid::parse_str(&child_branch).map_err(|_| "invalid child branch".to_owned())?;
                                if id.is_nil() || branch.is_nil() { return Err("invalid child identity".into()); }
                                let directory = source.file_path.parent().ok_or_else(|| "source directory unavailable".to_owned())?;
                                let mut child = talos_session::Session::new(id, source.project.clone(), source.workspace_root.clone(), directory.join(format!("{id}.tlog")));
                                child.current_branch = branch.to_string();
                                source.publish_history_fork(&child, &lease, through_entry.as_deref(), deadline,
                                    || stopped.is_cancelled() || observer.is_closed() || admission.as_ref().is_some_and(|control| !control.is_open()))
                                    .map(|tail| ForkOutput::Published(tail.id)).map_err(|_| "history fork publication unavailable; retain child for reconciliation".to_owned())
                            });
                            fork_work = Some(ForkWork { task, cancel, receipt: ForkReceipt::Published(receipt_tx) });
                        }
                        SessionOp::ForkDurable { external_id, receipt_tx } => {
                            let Some(receipt_tx) = receipt_tx else {
                                let _ = self.eq_tx.send(SessionEvent::Error { message: "durable fork requires an in-process receipt channel".into() });
                                continue;
                            };
                            if current_turn.is_some() || !compact.is_idle_for_fork() || !pending.is_empty() || shutting_down {
                                let _ = receipt_tx.send(Err("durable fork requires an idle session".into()));
                                continue;
                            }
                            if receipt_tx.is_closed() { continue; }
                            #[cfg(any(unix, windows))]
                            if let Some(durable) = &self.durable_persistence {
                                let session = durable.session.clone();
                                let observer = receipt_tx.clone();
                                let admission = self.runtime_admission.clone();
                                let cancel = CancellationToken::new();
                                let stopped = cancel.clone();
                                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
                                #[cfg(test)]
                                let gate = self.fork_worker_gate.clone();
                                let task = tokio::task::spawn_blocking(move || {
                                    #[cfg(test)]
                                    if let Some((reached, release)) = gate {
                                        reached.notify_one();
                                        while !release.load(std::sync::atomic::Ordering::SeqCst) {
                                            if stopped.is_cancelled() || observer.is_closed() || std::time::Instant::now() >= deadline {
                                                return Err("durable fork cancelled at test gate".into());
                                            }
                                            std::thread::park_timeout(std::time::Duration::from_millis(1));
                                        }
                                    }
                                    session.fork_committed_tip(&external_id,
                                        deadline,
                                        move || stopped.is_cancelled() || observer.is_closed() || admission.as_ref().is_some_and(|control| !control.is_open())
                                    ).map(|child| ForkOutput::Published(child.id().to_string())).map_err(|_| "durable fork storage unavailable".to_owned())
                                });
                                fork_work = Some(ForkWork { task, cancel, receipt: ForkReceipt::Published(receipt_tx) });
                            } else {
                                let _ = receipt_tx.send(Err("durable fork requires configured durable storage".into()));
                            }
                            #[cfg(not(any(unix, windows)))]
                            { let _ = external_id; let _ = receipt_tx.send(Err("durable fork unavailable on this platform".into())); }
                        }
                        SessionOp::SetSkillContext { name, content } => {
                            if current_turn.is_some() || compact.is_running() || !pending.is_empty() {
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
                            self.background_jobs.invalidate_completions();
                            shutting_down = true;
                            compact.cancel(self);
                            let should_interrupt = self.runtime_admission.as_ref().is_none_or(|control| {
                                !matches!(
                                    control.shutdown_policy(),
                                    Some(RuntimeShutdownTurnPolicy::FinishCurrent)
                                )
                            });
                            if should_interrupt && let Some(token) = &cancel_token {
                                token.cancel();
                            }
                            let rejected_pending = pending.len();
                            self.release_in_memory_pending_on_shutdown(&mut pending);
                            let reconciliation_succeeded = match self.pending_store.pause_unstarted() {
                                Ok(_) => true,
                                Err(error) => {
                                    self.emit_custody_error("failed to persist shutdown pause", &error);
                                    false
                                }
                            };
                            if let Some(control) = &self.runtime_admission {
                                control.record_reconciliation(
                                    rejected_pending,
                                    reconciliation_succeeded,
                                );
                            }
                            pending_items = current_submission_size.map_or(0, |size| size.0);
                            pending_bytes = current_submission_size.map_or(0, |size| size.1);
                            pending_images = current_submission_size.map_or(0, |size| size.2);
                            pending_image_bytes = current_submission_size.map_or(0, |size| size.3);
                        }
                    }
                }
                request = async {
                    match current_boundary_rx.as_mut() {
                        Some(rx) => rx.recv().await,
                        None => None,
                    }
                }, if current_boundary_rx.is_some() => {
                    if let Some(request) = request {
                        let request = match request {
                            crate::AgentActorRequest::AdmitModel { submission, response } => {
                                if response.is_closed() || shutting_down {
                                    let _ = submission.receipt.send(talos_core::session::CompactOutcome::Cancelled);
                                } else if let Some(token) = cancel_token.as_ref() {
                                    let admitted = compact.admit_model(self, submission, token);
                                    if response.send(admitted).is_err() && admitted.is_ok() {
                                        compact.cancel_queued_model(self);
                                    }
                                } else {
                                    let _ = submission.receipt.send(talos_core::session::CompactOutcome::Cancelled);
                                }
                                continue;
                            }
                            crate::AgentActorRequest::Boundary(request) => request,
                        };
                        // The previous acknowledgement precedes this request in
                        // Agent execution, but the two channels can be selected
                        // in either order by the Actor.
                        if let Some(rx) = current_boundary_ack_rx.as_mut() {
                            while let Ok(submission_id) = rx.try_recv() {
                                self.acknowledge_boundary_submission(
                                    submission_id,
                                    &mut delivered_boundary,
                                    &mut injected_structured,
                                );
                            }
                        }
                        let injected = if shutting_down
                            || request.response.is_closed()
                            || cancel_token.as_ref().is_none_or(CancellationToken::is_cancelled)
                        {
                            None
                        } else {
                            self.take_boundary_submission(
                                &mut pending,
                                current_structured.as_ref(),
                                &mut delivered_boundary,
                            )
                        };
                        if !shutting_down && let Some(token) = cancel_token.as_ref().filter(|token| !token.is_cancelled()) {
                            compact.start_boundary(self, request, injected, token);
                        } else {
                            let _ = request.compact.send(None);
                            let _ = request.response.send(injected);
                        }
                    } else {
                        current_boundary_rx = None;
                    }
                }
                acknowledgement = async {
                    match current_boundary_ack_rx.as_mut() {
                        Some(rx) => rx.recv().await,
                        None => None,
                    }
                }, if current_boundary_ack_rx.is_some() => {
                    if let Some(submission_id) = acknowledgement {
                        self.acknowledge_boundary_submission(
                            submission_id,
                            &mut delivered_boundary,
                            &mut injected_structured,
                        );
                    } else {
                        current_boundary_ack_rx = None;
                    }
                }
            }
        }

        if self.runtime_admission.is_none()
            && let Err(error) = self.background_jobs.finalize().await
        {
            let _ = self.eq_tx.send(SessionEvent::Error {
                message: format!("background cleanup incomplete: {error}"),
            });
        }
    }

    fn take_boundary_submission(
        &self,
        pending: &mut VecDeque<StructuredSubmission>,
        current: Option<&ActiveStructuredTurn>,
        delivered: &mut Option<BoundaryStructuredTurn>,
    ) -> Option<SteeringBoundaryBatch> {
        if delivered.is_some() {
            return None;
        }
        let current = current?;
        let index = pending
            .iter()
            .position(|submission| submission.source == SubmissionSource::User)?;
        let submission = pending.remove(index)?;
        if submission.common_kind() != Some(SubmissionKind::UserTurn) {
            pending.insert(index, submission);
            return None;
        }
        let record = match self.pending_store.get(&submission.id) {
            Ok(Some(record)) => record,
            Ok(None) => {
                self.emit_custody_error(
                    "boundary submission is missing from the durable journal",
                    &submission.id,
                );
                pending.insert(index, submission);
                return None;
            }
            Err(error) => {
                self.emit_custody_error("failed to load boundary submission", &error);
                pending.insert(index, submission);
                return None;
            }
        };
        if let Err(error) = self
            .pending_store
            .mark_running(&submission.id, &current.turn_id)
        {
            self.emit_custody_error("failed to bind boundary submission to active Turn", &error);
            pending.insert(index, submission);
            return None;
        }
        let active = ActiveStructuredTurn {
            submission_id: submission.id.clone(),
            receipt_id: record.receipt_id.clone(),
            session_generation: self.session_generation,
            source: submission.source,
            turn_id: current.turn_id.clone(),
        };
        let batch = SteeringBoundaryBatch {
            submission_id: submission.id.clone(),
            items: submission.items.clone(),
        };
        *delivered = Some(BoundaryStructuredTurn { active, submission });
        Some(batch)
    }

    fn acknowledge_boundary_submission(
        &self,
        acknowledgement: SteeringBoundaryAcknowledgement,
        delivered: &mut Option<BoundaryStructuredTurn>,
        injected: &mut Vec<BoundaryStructuredTurn>,
    ) {
        let SteeringBoundaryAcknowledgement {
            submission_id,
            projected,
        } = acknowledgement;
        let Some(boundary) = delivered.take() else {
            self.emit_custody_error(
                "ignored boundary acknowledgement without a delivered submission",
                &submission_id,
            );
            return;
        };
        if boundary.active.submission_id != submission_id {
            self.emit_custody_error(
                "ignored mismatched boundary acknowledgement",
                &submission_id,
            );
            *delivered = Some(boundary);
            return;
        }
        let _ = self.eq_tx.send(SessionEvent::StructuredSubmissionInjected {
            session_id: self.session_id.clone(),
            session_generation: self.session_generation,
            submission: boundary.submission.clone(),
            receipt_id: boundary.active.receipt_id.clone(),
            turn_id: boundary.active.turn_id.clone(),
        });
        injected.push(boundary);
        let _ = projected.send(());
    }

    fn commit_turn_record(&mut self, record: TurnRecord) {
        let start = self.history.len();
        self.history.extend(record.new_messages);
        self.recovered_source = record
            .committed_source
            .unwrap_or_else(|| {
                crate::compaction::HistorySource::bind_committed(
                    self.session_generation,
                    &self.history,
                    self.recovered_source.as_ref(),
                    start,
                    self.durable_persistence
                        .as_ref()
                        .map(|durable| durable.session.id().to_string())
                        .as_deref(),
                    &record.persisted_entry_ids,
                )
            })
            .filter(|source| {
                source.generation() == self.session_generation
                    && source.revision() == self.history.len() as u64
            });
    }

    async fn start_submission(
        &mut self,
        submission: StructuredSubmission,
        turn_counter: u64,
        token: CancellationToken,
    ) -> Result<StartedTurn, UnstartedOutcome> {
        if let Some(estimated_tokens) = self.post_turn_pressure.take() {
            tracing::debug!(
                estimated_tokens,
                "post-turn pressure requires fresh request assessment"
            );
        }
        // Prior work cannot mark a later turn's slot, including after cancellation.
        // Admission below still evaluates the newly assembled complete request.
        self.post_turn_pressure = Default::default();
        let checkpoint_observer = crate::checkpoint_progress::CheckpointObserver {
            sender: self.eq_tx.clone(),
            session_id: self.session_id.clone(),
            generation: self.session_generation,
            admission: self.runtime_admission.clone(),
            post_turn_pressure: self.post_turn_pressure.clone(),
        };
        let runtime_managed = self.runtime_admission.is_some();
        let prepared_before_commit = if runtime_managed {
            None
        } else {
            if submission.common_kind() == Some(SubmissionKind::PreviewRequest) {
                None
            } else {
                match self
                    .agent
                    .prepare_session_turn_at_revision(
                        &submission.items,
                        self.history.clone(),
                        self.model_context_limit,
                        self.session_generation,
                        self.history.len() as u64,
                        self.recovered_source.as_ref(),
                        Some(checkpoint_observer.clone()),
                        self.private_replays.clone(),
                    )
                    .await
                {
                    Ok(prepared) => Some(prepared),
                    Err(crate::AgentError::ContextBudgetExceeded { estimated, limit }) => {
                        let _ = self.eq_tx.send(SessionEvent::SubmissionContextBudget {
                            session_id: self.session_id.clone(),
                            session_generation: self.session_generation,
                            submission_id: submission.id.clone(),
                            budget: talos_core::message::ContextBudget {
                                estimated_tokens: estimated,
                                limit: Some(limit),
                                omitted_tool_exchanges: 0,
                            },
                        });
                        let terminal = self.pause_before_start(
                            &submission,
                            SubmissionRejectionReason::ContextBudgetExceeded,
                        );
                        return Err(if terminal {
                            UnstartedOutcome::Terminal
                        } else {
                            UnstartedOutcome::Retained
                        });
                    }
                    Err(error) => {
                        let _ = self.eq_tx.send(SessionEvent::Error {
                            message: format!(
                                "failed to seal Provider request plan for {}: {error}",
                                submission.id
                            ),
                        });
                        self.pause_before_start(
                            &submission,
                            SubmissionRejectionReason::InvalidStructure,
                        );
                        return Err(UnstartedOutcome::Retained);
                    }
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
                    self.emit_custody_error(
                        "accepted structured submission is missing from journal",
                        &submission.id,
                    );
                    return Err(UnstartedOutcome::Retained);
                }
                Err(error) => {
                    self.emit_custody_error("failed to load accepted submission", &error);
                    return Err(UnstartedOutcome::Retained);
                }
            };
            if let Err(error) = self.pending_store.mark_running(&submission.id, &turn_id) {
                self.emit_custody_error("failed to mark structured submission running", &error);
                return Err(UnstartedOutcome::Retained);
            }
            let active = ActiveStructuredTurn {
                submission_id: submission.id.clone(),
                receipt_id: record.receipt_id,
                session_generation: self.session_generation,
                source: submission.source,
                turn_id: turn_id.clone(),
            };
            let _ = self.eq_tx.send(SessionEvent::StructuredSubmissionStarted {
                session_id: self.session_id.clone(),
                session_generation: active.session_generation,
                submission: submission.clone(),
                receipt_id: active.receipt_id.clone(),
                turn_id: active.turn_id.clone(),
            });
            let _ = self.eq_tx.send(SessionEvent::StructuredTurnEvent {
                session_id: self.session_id.clone(),
                session_generation: active.session_generation,
                source: active.source,
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

        let prepared_turn = if !runtime_managed {
            prepared_before_commit
        } else if submission.common_kind() == Some(SubmissionKind::PreviewRequest) {
            None
        } else {
            let prepare_result = tokio::select! {
                () = token.cancelled() => {
                    return Ok(immediate_started_turn(
                        self.eq_tx.clone(),
                        self.session_id.clone(),
                        turn_id,
                        token,
                        structured,
                        TurnCompletionStatus::Cancelled,
                    ));
                }
                result = self.agent.prepare_session_turn_at_revision(
                    &submission.items,
                    self.history.clone(),
                    self.model_context_limit,
                    self.session_generation,
                    self.history.len() as u64,
                    self.recovered_source.as_ref(),
                    Some(checkpoint_observer.clone()),
                    self.private_replays.clone(),
                ) => result,
            };
            match prepare_result {
                Ok(prepared) => Some(prepared),
                Err(error) => {
                    if let crate::AgentError::ContextBudgetExceeded { estimated, limit } = &error {
                        let _ = self.eq_tx.send(SessionEvent::SubmissionContextBudget {
                            session_id: self.session_id.clone(),
                            session_generation: self.session_generation,
                            submission_id: submission.id.clone(),
                            budget: talos_core::message::ContextBudget {
                                estimated_tokens: *estimated,
                                limit: Some(*limit),
                                omitted_tool_exchanges: 0,
                            },
                        });
                    }
                    let _ = self.eq_tx.send(SessionEvent::Error {
                        message: format!(
                            "failed to seal Provider request plan for {}: {error}",
                            submission.id
                        ),
                    });
                    return Ok(immediate_started_turn(
                        self.eq_tx.clone(),
                        self.session_id.clone(),
                        turn_id,
                        token,
                        structured,
                        TurnCompletionStatus::Error {
                            message: error.to_string(),
                        },
                    ));
                }
            }
        };

        if submission.common_kind() == Some(SubmissionKind::PreviewRequest) {
            let runtime_lease = self.runtime_lease.clone();
            let agent = self.agent.clone();
            let eq_tx = self.eq_tx.clone();
            let history = self.history.clone();
            let session_id = self.session_id.clone();
            let message = submission.items[0].text.clone();
            let preview_token = token.clone();
            let handle = tokio::spawn(async move {
                let _runtime_lease = runtime_lease;
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
                            committed_source: None,
                            consumed_background_jobs: Vec::new(),
                            completion_events: Vec::new(),
                            persisted_entry_ids: Vec::new(),
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
                    committed_source: None,
                    consumed_background_jobs: Vec::new(),
                    completion_events: Vec::new(),
                    persisted_entry_ids: Vec::new(),
                    new_messages: Vec::new(),
                    status: record_status,
                    completion,
                })
            });
            return Ok(StartedTurn {
                handle,
                token,
                structured,
                boundary_rx: {
                    let (_tx, rx) = mpsc::unbounded_channel();
                    rx
                },
                boundary_ack_rx: {
                    let (_tx, rx) = mpsc::unbounded_channel();
                    rx
                },
            });
        }

        if let Some(agent_mut) = Arc::get_mut(&mut self.agent) {
            agent_mut.set_append_prompt_opt(None);
        }

        let prepared = prepared_turn.expect("non-preview submission must be prepared");
        Ok(self.spawn_prepared_turn(prepared, turn_id, token, structured))
    }

    /// Build owned preparation work so the Actor keeps polling control inputs.
    fn prepare_background_completion(
        &mut self,
        id: talos_core::background_job::BackgroundJobId,
        terminal: crate::background_completion::Terminal,
        turn_id: String,
        token: CancellationToken,
    ) -> BackgroundPreparation {
        let input = terminal.context(&id);
        self.post_turn_pressure = Default::default();
        let observer = crate::checkpoint_progress::CheckpointObserver {
            sender: self.eq_tx.clone(),
            session_id: self.session_id.clone(),
            generation: self.session_generation,
            admission: self.runtime_admission.clone(),
            post_turn_pressure: self.post_turn_pressure.clone(),
        };
        let agent = self.agent.clone();
        let history = self.history.clone();
        let limit = self.model_context_limit;
        let generation = self.session_generation;
        let source = self.recovered_source.clone();
        let replays = self.private_replays.clone();
        let revision = history.len() as u64;
        let cancel = token.clone();
        let future = Box::pin(async move {
            let input = input
                .ok_or_else(|| "background completion has invalid identity or state".to_owned())?;
            tokio::select! {
                biased;
                () = cancel.cancelled() => Err("background continuation cancelled before start".into()),
                result = AssertUnwindSafe(agent.prepare_session_messages_at_revision(
                    String::new(), vec![input], history, limit, generation,
                    revision, source.as_ref(), Some(observer), replays,
                )).catch_unwind() => result.map_err(|_| "background preparation panicked".to_owned())?
                    .map_err(|error| error.to_string()),
            }
        });
        BackgroundPreparation {
            future,
            id,
            turn_id,
            token,
        }
    }

    /// Shares worker supervision and persistence for admitted prepared turns.
    /// Callers must complete runtime admission and emit the start event first.
    fn spawn_prepared_turn(
        &self,
        prepared: crate::PreparedSessionTurn,
        turn_id: String,
        token: CancellationToken,
        structured: Option<ActiveStructuredTurn>,
    ) -> StartedTurn {
        let sequence = Arc::new(AtomicU64::new(1));
        let token_clone = token.clone();
        let agent = self.agent.clone();
        let eq_tx = self.eq_tx.clone();
        let persistence = self.persistence.clone();
        let durable_persistence = self.durable_persistence.clone();
        let session_id = self.session_id.clone();
        let (boundary_tx, boundary_rx) = mpsc::unbounded_channel();
        let (boundary_ack_tx, boundary_ack_rx) = mpsc::unbounded_channel();
        #[cfg(test)]
        let boundary_ack_cancel_gate = self.boundary_ack_cancel_gate.clone();
        let mut source_history = self.history.clone();
        let previous_source = self.recovered_source.clone();
        let source_generation = self.session_generation;
        let source_namespace = durable_persistence
            .as_ref()
            .map(|p| p.session.id().to_string());
        let runtime_lease = self.runtime_lease.clone();
        let handle = tokio::spawn(async move {
            let (event_tx, event_rx) = mpsc::unbounded_channel::<AgentEvent>();
            let (result_tx, result_rx) = tokio::sync::oneshot::channel::<TurnRecord>();
            let (worker_eq_tx, mut worker_eq_rx) = mpsc::unbounded_channel();
            let forwarding = AssertUnwindSafe(run_turn_with_forwarding(TurnForwarding {
                agent,
                prepared,
                event_tx,
                event_rx,
                eq_tx: worker_eq_tx,
                cancel_token: token_clone,
                turn_id,
                session_id,
                sequence,
                persistence,
                durable_persistence,
                runtime_lease,
                result_tx,
                boundary_tx,
                boundary_ack_tx,
                #[cfg(test)]
                boundary_ack_cancel_gate,
            }))
            .catch_unwind();
            tokio::pin!(forwarding);
            let mut completion_events = Vec::new();
            loop {
                tokio::select! {
                    _ = &mut forwarding => break,
                    Some(event) = worker_eq_rx.recv() => {
                        if matches!(&event, SessionEvent::TurnEvent { payload: TurnEventPayload::Completed { .. }, .. }) {
                            completion_events.push(event);
                        } else {
                            let _ = eq_tx.send(event);
                        }
                    }
                }
            }
            while let Ok(event) = worker_eq_rx.try_recv() {
                if matches!(
                    &event,
                    SessionEvent::TurnEvent {
                        payload: TurnEventPayload::Completed { .. },
                        ..
                    }
                ) {
                    completion_events.push(event);
                } else {
                    let _ = eq_tx.send(event);
                }
            }
            let mut record = result_rx.await.ok()?;
            record.completion_events = completion_events;
            let start = source_history.len();
            source_history.extend(record.new_messages.iter().cloned());
            let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(2);
            record.committed_source = Some(
                crate::compaction::HistorySource::bind_committed_until(
                    source_generation,
                    &source_history,
                    previous_source.as_ref(),
                    start,
                    source_namespace.as_deref(),
                    &record.persisted_entry_ids,
                    deadline,
                )
                .await,
            );
            Some(record)
        });
        StartedTurn {
            handle,
            token,
            structured,
            boundary_rx,
            boundary_ack_rx,
        }
    }
}

impl Drop for AppServerSession {
    fn drop(&mut self) {
        self.background_jobs.begin_shutdown();
    }
}

fn compatibility_submission(
    sequence: u64,
    sender_generation: u64,
    kind: SubmissionKind,
    text: String,
    attachments: Vec<talos_core::message::ContentPart>,
) -> StructuredSubmission {
    StructuredSubmission {
        id: format!("compatibility_{sequence}"),
        source: SubmissionSource::Compatibility,
        sender_generation,
        items: vec![SubmissionItem {
            id: format!("compatibility_item_{sequence}"),
            enqueue_sequence: sequence,
            kind,
            text,
            attachments,
        }],
    }
}

fn enqueue_by_source(
    pending: &mut VecDeque<StructuredSubmission>,
    submission: StructuredSubmission,
) {
    if submission.source == SubmissionSource::Scheduler {
        pending.push_back(submission);
        return;
    }
    let scheduler_boundary = pending
        .iter()
        .position(|queued| queued.source == SubmissionSource::Scheduler)
        .unwrap_or(pending.len());
    pending.insert(scheduler_boundary, submission);
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

fn subtract_submission_totals(
    submission: &StructuredSubmission,
    pending_items: &mut usize,
    pending_bytes: &mut usize,
    pending_images: &mut usize,
    pending_image_bytes: &mut u64,
) {
    let (images, image_bytes) = submission.image_totals();
    *pending_items = pending_items.saturating_sub(submission.items.len());
    *pending_bytes = pending_bytes.saturating_sub(submission.total_text_bytes());
    *pending_images = pending_images.saturating_sub(images);
    *pending_image_bytes = pending_image_bytes.saturating_sub(image_bytes);
}
