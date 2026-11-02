//! One Actor-owned manual/model operation; raw history stays with the Actor.

use super::*;
use crate::checkpoint_manual::ManualCheckpointResult;
use crate::compaction::{CompactAdmission, CompactOperations, CompactOrigin};
use talos_core::session::{CompactOutcome, CompactState};

pub(super) struct ManualCompaction {
    measurement: Arc<std::sync::Mutex<Option<u32>>>,
    ledger: CompactOperations,
    queued: Option<u64>,
    next_model_id: Option<u64>,
    model_authority: Option<crate::checkpoint_tools::PermissionLease>,
    queued_model: Option<(u64, crate::SubmittedCheckpoint)>,
    model_receipt: Option<tokio::sync::oneshot::Sender<CompactOutcome>>,
    boundary_resume: Option<(
        tokio::sync::oneshot::Sender<Option<crate::SteeringBoundaryBatch>>,
        Option<crate::SteeringBoundaryBatch>,
    )>,
    running: Option<(u64, CancellationToken, JoinHandle<ManualCheckpointResult>)>,
}

impl ManualCompaction {
    pub(super) fn new(generation: u64) -> Self {
        Self {
            measurement: Arc::new(std::sync::Mutex::new(None)),
            ledger: CompactOperations::new(generation),
            queued: None,
            next_model_id: Some(0),
            model_authority: None,
            queued_model: None,
            model_receipt: None,
            boundary_resume: None,
            running: None,
        }
    }

    fn emit(actor: &AppServerSession, id: u64, state: CompactState) {
        let _ = actor.eq_tx.send(SessionEvent::CompactStatus {
            session_id: actor.session_id.clone(),
            session_generation: actor.session_generation,
            operation_id: id,
            state,
        });
    }

    pub(super) fn admit(&mut self, actor: &AppServerSession, generation: u64, id: u64) {
        let state = match self.ledger.admit(generation, id) {
            CompactAdmission::Accepted => {
                self.measurement = Arc::new(std::sync::Mutex::new(None));
                self.queued = Some(id);
                CompactState::Queued
            }
            CompactAdmission::Pending => {
                if self.running.is_some() {
                    CompactState::Started
                } else {
                    CompactState::Queued
                }
            }
            CompactAdmission::Busy => CompactState::Busy,
            CompactAdmission::Completed(outcome) => CompactState::Completed {
                outcome,
                replayed: true,
            },
            CompactAdmission::Expired => CompactState::Expired,
            CompactAdmission::StaleGeneration => CompactState::StaleGeneration,
        };
        Self::emit(actor, id, state);
    }

    pub(super) fn is_running(&self) -> bool {
        self.running.is_some()
    }

    /// Forks require no accepted or finishing compaction operation.
    pub(super) fn is_idle_for_fork(&self) -> bool {
        self.running.is_none()
            && self.queued.is_none()
            && self.queued_model.is_none()
            && self.model_receipt.is_none()
            && self.boundary_resume.is_none()
    }

    pub(super) fn has_pending_work(&self) -> bool {
        self.running.is_some() || self.queued_model.is_some()
    }

    /// Reserve shared capacity before returning tool admission. Preparation and
    /// installation remain deferred until the complete paired batch boundary.
    pub(super) fn admit_model(
        &mut self,
        actor: &AppServerSession,
        submission: crate::ModelBoundarySubmission,
        turn_token: &CancellationToken,
    ) -> Result<u64, CompactOutcome> {
        let rejection = if turn_token.is_cancelled()
            || submission.receipt.is_closed()
            || submission
                .authority
                .as_ref()
                .is_some_and(|lease| !lease.valid())
        {
            Some(CompactOutcome::Cancelled)
        } else if self.next_model_id.is_none() {
            Some(CompactOutcome::Failed)
        } else {
            None
        };
        if let Some(outcome) = rejection {
            let _ = submission.receipt.send(outcome);
            return Err(outcome);
        }
        let Some(id) = self.next_model_id else {
            return Err(CompactOutcome::Failed);
        };
        if self
            .ledger
            .admit_from(actor.session_generation, CompactOrigin::Model, id)
            != CompactAdmission::Accepted
        {
            let _ = submission.receipt.send(CompactOutcome::Failed);
            return Err(CompactOutcome::Failed);
        }
        self.next_model_id = id.checked_add(1);
        self.measurement = Arc::new(std::sync::Mutex::new(None));
        self.model_authority = submission.authority;
        self.model_receipt = Some(submission.receipt);
        self.queued_model = Some((id, submission.checkpoint));
        Ok(id)
    }

    pub(super) fn start_boundary(
        &mut self,
        actor: &AppServerSession,
        mut request: crate::SteeringBoundaryRequest,
        injected: Option<crate::SteeringBoundaryBatch>,
        turn_token: &CancellationToken,
    ) {
        if let Some(submission) = request.submitted.take()
            && self.admit_model(actor, submission, turn_token).is_err()
        {
            let _ = request.compact.send(None);
            let _ = request.response.send(injected);
            return;
        }
        let (model_id, submitted_checkpoint) = self
            .queued_model
            .take()
            .map_or((None, None), |(id, checkpoint)| {
                (Some(id), Some(checkpoint))
            });
        let Some(id) = model_id.or_else(|| self.queued.take()) else {
            let _ = request.compact.send(None);
            let _ = request.response.send(injected);
            return;
        };
        let (result, received) = tokio::sync::oneshot::channel();
        self.measurement = Arc::new(std::sync::Mutex::new(None));
        if request
            .compact
            .send(Some(crate::BoundaryCompactJob {
                submitted_checkpoint,
                result,
                measurement: self.measurement.clone(),
            }))
            .is_err()
        {
            self.finish(actor, id, CompactOutcome::Cancelled);
            let _ = request.response.send(injected);
            return;
        }
        let token = turn_token.child_token();
        let waiting_token = token.clone();
        let task = tokio::spawn(async move {
            tokio::select! { biased;
                () = waiting_token.cancelled() => ManualCheckpointResult::Outcome(CompactOutcome::Cancelled),
                result = received => result.unwrap_or(ManualCheckpointResult::Outcome(CompactOutcome::Failed)),
            }
        });
        self.boundary_resume = Some((request.response, injected));
        self.running = Some((id, token, task));
        if self.model_receipt.is_none() {
            Self::emit(actor, id, CompactState::Started);
        }
    }

    pub(super) fn start(&mut self, actor: &AppServerSession) {
        if self.running.is_some() {
            return;
        }
        let Some(id) = self.queued.take() else {
            return;
        };
        let token = CancellationToken::new();
        if actor
            .runtime_admission
            .as_ref()
            .is_some_and(|control| !control.commit_start(token.clone()))
        {
            self.finish(actor, id, CompactOutcome::Cancelled);
            return;
        }
        Self::emit(actor, id, CompactState::Started);
        let agent = actor.agent.clone();
        let history = actor.history.clone();
        let generation = actor.session_generation;
        let recovered = actor.recovered_source.clone();
        let limit = actor.model_context_limit;
        let task_token = token.clone();
        self.measurement = Arc::new(std::sync::Mutex::new(None));
        let measurement = self.measurement.clone();
        let task = tokio::spawn(async move {
            tokio::select! { biased;
                () = task_token.cancelled() => ManualCheckpointResult::Outcome(CompactOutcome::Cancelled),
                outcome = agent.compact_session_history_measured(history, generation, limit, recovered.as_ref(), Some(&measurement)) => outcome,
            }
        });
        self.running = Some((id, token, task));
    }

    pub(super) async fn wait(&mut self) -> ManualCheckpointResult {
        match self.running.as_mut() {
            Some((_, token, task)) => {
                if let Some(receipt) = self.model_receipt.as_mut() {
                    tokio::select! { biased;
                        () = receipt.closed() => {
                            token.cancel();
                            ManualCheckpointResult::Outcome(CompactOutcome::Cancelled)
                        },
                        result = task => result.unwrap_or(ManualCheckpointResult::Outcome(CompactOutcome::Failed)),
                    }
                } else {
                    task.await
                        .unwrap_or(ManualCheckpointResult::Outcome(CompactOutcome::Failed))
                }
            }
            None => {
                if self.queued_model.is_some()
                    && let Some(receipt) = self.model_receipt.as_mut()
                {
                    receipt.closed().await;
                    ManualCheckpointResult::Outcome(CompactOutcome::Cancelled)
                } else {
                    std::future::pending().await
                }
            }
        }
    }

    pub(super) fn complete(&mut self, actor: &AppServerSession, result: ManualCheckpointResult) {
        let Some((id, token, task)) = self.running.take() else {
            // wait() may be woken by an abandoned admission before any batch.
            if self
                .model_receipt
                .as_ref()
                .is_some_and(|receipt| receipt.is_closed())
            {
                self.cancel_queued_model(actor);
            }
            return;
        };
        task.abort();
        let commit = || {
            if token.is_cancelled()
                || self
                    .model_receipt
                    .as_ref()
                    .is_some_and(|receipt| receipt.is_closed())
            {
                return CompactOutcome::Cancelled;
            }
            match result {
                ManualCheckpointResult::Outcome(outcome) => outcome,
                ManualCheckpointResult::Candidate {
                    projection,
                    before,
                    after,
                    deadline,
                } => {
                    if projection.source.generation() != actor.session_generation
                        || projection.source.revision() != actor.history.len() as u64
                    {
                        return CompactOutcome::Cancelled;
                    }
                    match actor
                        .agent
                        .install_checkpoint_projection(*projection, deadline)
                    {
                        Ok(()) => CompactOutcome::Applied { before, after },
                        Err(_) => CompactOutcome::FailedWithEstimate {
                            before,
                            reason: talos_core::session::CompactFailureReason::CommitRejected,
                        },
                    }
                }
            }
        };
        // Runtime admission is acquired first. The inner permission acquisition
        // is nonblocking, so an inverse owner cannot deadlock this commit.
        let authorized_commit = || {
            if let Some(lease) = &self.model_authority {
                lease.commit(commit).unwrap_or(CompactOutcome::Cancelled)
            } else {
                commit()
            }
        };
        let outcome = if let Some(control) = &actor.runtime_admission {
            control
                .commit_checkpoint(authorized_commit)
                .unwrap_or(CompactOutcome::Cancelled)
        } else {
            authorized_commit()
        };
        if self.boundary_resume.is_none()
            && let Some(control) = &actor.runtime_admission
        {
            let status = match outcome {
                CompactOutcome::Applied { .. } | CompactOutcome::NotEligible { .. } => {
                    TurnCompletionStatus::Success {
                        final_text: String::new(),
                        new_messages: vec![],
                    }
                }
                CompactOutcome::Cancelled | CompactOutcome::CancelledWithEstimate { .. } => {
                    TurnCompletionStatus::Cancelled
                }
                CompactOutcome::Failed | CompactOutcome::FailedWithEstimate { .. } => {
                    TurnCompletionStatus::Error {
                        message: "checkpoint failed".into(),
                    }
                }
            };
            control.finish_active(Some(&status));
        }
        self.finish(actor, id, outcome);
        if let Some((response, injected)) = self.boundary_resume.take() {
            let _ = response.send(injected);
        }
    }

    fn finish(&mut self, actor: &AppServerSession, id: u64, outcome: CompactOutcome) {
        let outcome = if outcome == CompactOutcome::Cancelled {
            self.measurement
                .lock()
                .ok()
                .and_then(|value| *value)
                .map(|before| CompactOutcome::CancelledWithEstimate { before })
                .unwrap_or(outcome)
        } else {
            outcome
        };
        let origin = if self.model_receipt.is_some() {
            CompactOrigin::Model
        } else {
            CompactOrigin::Manual
        };
        if self
            .ledger
            .finish_from(actor.session_generation, origin, id, outcome)
        {
            self.model_authority = None;
            if let Some(receipt) = self.model_receipt.take() {
                let _ = receipt.send(outcome);
                // The receiver may close, or consume the receipt without making
                // another provider request. Session observers still need a terminal.
                let _ = actor.eq_tx.send(SessionEvent::ModelCheckpointTerminal {
                    session_id: actor.session_id.clone(),
                    session_generation: actor.session_generation,
                    operation_id: id,
                    outcome,
                });
                return;
            }
            Self::emit(
                actor,
                id,
                CompactState::Completed {
                    outcome,
                    replayed: false,
                },
            );
        }
    }

    pub(super) fn cancel_operation(&mut self, actor: &AppServerSession, generation: u64, id: u64) {
        match self.ledger.observe(generation, id) {
            CompactAdmission::Pending => self.cancel(actor),
            CompactAdmission::Completed(outcome) => Self::emit(
                actor,
                id,
                CompactState::Completed {
                    outcome,
                    replayed: true,
                },
            ),
            CompactAdmission::StaleGeneration => {
                Self::emit(actor, id, CompactState::StaleGeneration)
            }
            _ => Self::emit(actor, id, CompactState::Expired),
        }
    }

    /// An admission belongs to its originating turn, never the next turn.
    pub(super) fn cancel_queued_model(&mut self, actor: &AppServerSession) {
        if let Some((id, _)) = self.queued_model.take() {
            self.finish(actor, id, CompactOutcome::Cancelled);
        }
    }

    pub(super) fn cancel(&mut self, actor: &AppServerSession) {
        if let Some((_, token, _)) = &self.running {
            token.cancel();
        }
        if let Some(id) = self.queued.take() {
            self.finish(actor, id, CompactOutcome::Cancelled);
        }
        self.cancel_queued_model(actor);
    }
}

impl Drop for ManualCompaction {
    fn drop(&mut self) {
        if let Some((_, token, task)) = &self.running {
            token.cancel();
            task.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn queued_manual_operation_prevents_fork_idle() {
        let mut compact = super::ManualCompaction::new(1);
        assert!(compact.is_idle_for_fork());
        compact.queued = Some(1);
        assert!(!compact.is_idle_for_fork());
        compact.queued = None;
        assert!(compact.is_idle_for_fork());
    }

    use super::*;
    #[tokio::test]
    async fn model_boundary_shares_manual_slot_and_receipts_before_resume() {
        let agent = Agent::with_security(
            Arc::new(SummaryModel),
            ToolRegistry::new(),
            None,
            None,
            std::env::temp_dir(),
        );
        let (mut handle, actor) = AppServerSession::new(
            agent,
            SessionConfig {
                runtime_policy: RuntimePolicy::interactive(),
                workspace_root: std::env::temp_dir(),
                initial_history: vec![],
                model_context_limit: 30_000,
            },
        );
        let mut coordinator = ManualCompaction::new(0);
        let request = || {
            let (compact, job) = tokio::sync::oneshot::channel();
            let (response, resume) = tokio::sync::oneshot::channel();
            let (receipt, outcome) = tokio::sync::oneshot::channel();
            let source = crate::compaction::HistorySource::bind(0, 0, &[], vec![]).expect("source");
            (
                crate::SteeringBoundaryRequest {
                    compact,
                    response,
                    submitted: Some(crate::ModelBoundarySubmission {
                        checkpoint: crate::SubmittedCheckpoint {
                            payload: "{}".into(),
                            source,
                            covered_end: 0,
                        },
                        receipt,
                        authority: None,
                    }),
                },
                job,
                resume,
                outcome,
            )
        };
        let token = CancellationToken::new();
        coordinator.admit(&actor, 0, 1);
        let (submission, job, resume, outcome) = request();
        coordinator.start_boundary(&actor, submission, None, &token);
        assert!(job.await.expect("refusal").is_none());
        assert!(resume.await.expect("resume").is_none());
        assert_eq!(outcome.await.expect("receipt"), CompactOutcome::Failed);
        assert_eq!(coordinator.next_model_id, Some(0));
        coordinator.cancel(&actor);
        while handle.eq_rx.try_recv().is_ok() {}

        let (submission, job, mut resume, mut outcome) = request();
        coordinator.start_boundary(&actor, submission, None, &token);
        let job = job.await.expect("job").expect("model preparation");
        assert!(job.submitted_checkpoint.is_some());
        assert!(resume.try_recv().is_err());
        assert!(outcome.try_recv().is_err());
        coordinator.admit(&actor, 0, 2);
        assert!(matches!(
            handle.eq_rx.try_recv().expect("manual busy"),
            SessionEvent::CompactStatus {
                state: CompactState::Busy,
                ..
            }
        ));
        job.result
            .send(ManualCheckpointResult::Outcome(CompactOutcome::Failed))
            .ok()
            .expect("result");
        let result = coordinator.wait().await;
        coordinator.complete(&actor, result);
        assert_eq!(outcome.await.expect("terminal"), CompactOutcome::Failed);
        assert!(resume.await.expect("resume after terminal").is_none());
        assert!(matches!(
            handle.eq_rx.try_recv().expect("model terminal"),
            SessionEvent::ModelCheckpointTerminal {
                operation_id: 0,
                outcome: CompactOutcome::Failed,
                ..
            }
        ));
        assert!(handle.eq_rx.try_recv().is_err(), "exactly one terminal");
        coordinator.admit(&actor, 0, 2);
        assert!(matches!(
            handle.eq_rx.try_recv().expect("manual queued"),
            SessionEvent::CompactStatus {
                state: CompactState::Queued,
                ..
            }
        ));
    }

    #[tokio::test]
    async fn model_admission_cancel_before_batch_releases_shared_capacity() {
        let agent = Agent::with_security(
            Arc::new(SummaryModel),
            ToolRegistry::new(),
            None,
            None,
            std::env::temp_dir(),
        );
        let (mut handle, actor) = AppServerSession::new(
            agent,
            SessionConfig {
                runtime_policy: RuntimePolicy::interactive(),
                workspace_root: std::env::temp_dir(),
                initial_history: vec![],
                model_context_limit: 30_000,
            },
        );
        let mut coordinator = ManualCompaction::new(0);
        let (receipt, mut outcome) = tokio::sync::oneshot::channel();
        let token = CancellationToken::new();
        assert_eq!(
            coordinator.admit_model(
                &actor,
                crate::ModelBoundarySubmission {
                    checkpoint: crate::SubmittedCheckpoint {
                        payload: "{}".into(),
                        covered_end: 0,
                        source: crate::compaction::HistorySource::bind(0, 0, &[], vec![])
                            .expect("source"),
                    },
                    receipt,
                    authority: None,
                },
                &token
            ),
            Ok(0)
        );
        assert!(outcome.try_recv().is_err());
        coordinator.admit(&actor, 0, 1);
        assert!(matches!(
            handle.eq_rx.try_recv().expect("busy"),
            SessionEvent::CompactStatus {
                state: CompactState::Busy,
                ..
            }
        ));
        coordinator.cancel(&actor);
        assert_eq!(
            outcome.await.expect("cancel receipt"),
            CompactOutcome::Cancelled
        );
        assert!(coordinator.queued_model.is_none());
        assert!(matches!(
            handle.eq_rx.try_recv().expect("cancel terminal"),
            SessionEvent::ModelCheckpointTerminal {
                operation_id: 0,
                outcome: CompactOutcome::Cancelled,
                ..
            }
        ));
        coordinator.admit(&actor, 0, 1);
        assert!(matches!(
            handle.eq_rx.try_recv().expect("released"),
            SessionEvent::CompactStatus {
                state: CompactState::Queued,
                ..
            }
        ));
    }

    #[tokio::test]
    async fn abandoned_admission_wakes_actor_without_waiting_for_batch_or_turn_end() {
        let agent = Agent::with_security(
            Arc::new(SummaryModel),
            ToolRegistry::new(),
            None,
            None,
            std::env::temp_dir(),
        );
        let (mut handle, actor) = AppServerSession::new(
            agent,
            SessionConfig {
                runtime_policy: RuntimePolicy::interactive(),
                workspace_root: std::env::temp_dir(),
                initial_history: vec![],
                model_context_limit: 30_000,
            },
        );
        let mut coordinator = ManualCompaction::new(0);
        let (receipt, outcome) = tokio::sync::oneshot::channel();
        let token = CancellationToken::new();
        assert_eq!(
            coordinator.admit_model(
                &actor,
                crate::ModelBoundarySubmission {
                    checkpoint: crate::SubmittedCheckpoint {
                        payload: "{}".into(),
                        covered_end: 0,
                        source: crate::compaction::HistorySource::bind(0, 0, &[], vec![])
                            .expect("source"),
                    },
                    receipt,
                    authority: None,
                },
                &token
            ),
            Ok(0)
        );
        assert!(coordinator.has_pending_work());
        assert!(!coordinator.is_running());
        drop(outcome);
        let terminal = tokio::time::timeout(std::time::Duration::from_secs(1), coordinator.wait())
            .await
            .expect("receipt closure wakes Actor");
        coordinator.complete(&actor, terminal);
        assert!(!coordinator.has_pending_work());
        assert!(matches!(
            handle
                .eq_rx
                .try_recv()
                .expect("abandoned receiver terminal"),
            SessionEvent::ModelCheckpointTerminal {
                operation_id: 0,
                outcome: CompactOutcome::Cancelled,
                ..
            }
        ));
        coordinator.cancel(&actor);
        assert!(handle.eq_rx.try_recv().is_err(), "terminal must not repeat");
        coordinator.admit(&actor, 0, 1);
        assert!(matches!(
            handle.eq_rx.try_recv().expect("released slot"),
            SessionEvent::CompactStatus {
                state: CompactState::Queued,
                ..
            }
        ));
        assert!(
            !token.is_cancelled(),
            "abandonment does not cancel the turn"
        );
    }

    #[tokio::test]
    async fn cancelled_or_abandoned_model_submission_never_consumes_identity() {
        for abandoned in [false, true] {
            let agent = Agent::with_security(
                Arc::new(SummaryModel),
                ToolRegistry::new(),
                None,
                None,
                std::env::temp_dir(),
            );
            let (_, actor) = AppServerSession::new(
                agent,
                SessionConfig {
                    runtime_policy: RuntimePolicy::interactive(),
                    workspace_root: std::env::temp_dir(),
                    initial_history: vec![],
                    model_context_limit: 30_000,
                },
            );
            let mut coordinator = ManualCompaction::new(0);
            let (compact, job) = tokio::sync::oneshot::channel();
            let (response, resume) = tokio::sync::oneshot::channel();
            let (receipt, outcome) = tokio::sync::oneshot::channel();
            let token = CancellationToken::new();
            let mut outcome = Some(outcome);
            if abandoned {
                drop(outcome.take());
            } else {
                token.cancel();
            }
            coordinator.start_boundary(
                &actor,
                crate::SteeringBoundaryRequest {
                    compact,
                    response,
                    submitted: Some(crate::ModelBoundarySubmission {
                        checkpoint: crate::SubmittedCheckpoint {
                            payload: "{}".into(),
                            source: crate::compaction::HistorySource::bind(0, 0, &[], vec![])
                                .expect("source"),
                            covered_end: 0,
                        },
                        receipt,
                        authority: None,
                    }),
                },
                None,
                &token,
            );
            assert!(job.await.expect("refused job").is_none());
            assert!(resume.await.expect("resumed").is_none());
            if let Some(outcome) = outcome {
                assert_eq!(outcome.await.expect("receipt"), CompactOutcome::Cancelled);
            }
            assert_eq!(coordinator.next_model_id, Some(0));
            assert!(!coordinator.is_running());
            assert!(coordinator.model_receipt.is_none());
            assert_eq!(coordinator.ledger.admit(0, 1), CompactAdmission::Accepted);
        }
    }

    #[tokio::test]
    async fn model_boundary_cancellation_and_closed_preparation_release_capacity() {
        for closed_preparation in [false, true] {
            let agent = Agent::with_security(
                Arc::new(SummaryModel),
                ToolRegistry::new(),
                None,
                None,
                std::env::temp_dir(),
            );
            let (_, actor) = AppServerSession::new(
                agent,
                SessionConfig {
                    runtime_policy: RuntimePolicy::interactive(),
                    workspace_root: std::env::temp_dir(),
                    initial_history: vec![],
                    model_context_limit: 30_000,
                },
            );
            let mut coordinator = ManualCompaction::new(0);
            let (compact, job) = tokio::sync::oneshot::channel();
            let (response, resume) = tokio::sync::oneshot::channel();
            let (receipt, outcome) = tokio::sync::oneshot::channel();
            let request = crate::SteeringBoundaryRequest {
                compact,
                response,
                submitted: Some(crate::ModelBoundarySubmission {
                    checkpoint: crate::SubmittedCheckpoint {
                        payload: "{}".into(),
                        source: crate::compaction::HistorySource::bind(0, 0, &[], vec![])
                            .expect("source"),
                        covered_end: 0,
                    },
                    receipt,
                    authority: None,
                }),
            };
            let token = CancellationToken::new();
            if closed_preparation {
                drop(job);
                coordinator.start_boundary(&actor, request, None, &token);
            } else {
                coordinator.start_boundary(&actor, request, None, &token);
                let job = job.await.expect("dispatch").expect("job");
                *job.measurement.lock().expect("measurement") = Some(123);
                token.cancel();
                let result = coordinator.wait().await;
                coordinator.complete(&actor, result);
                assert!(job.result.is_closed());
            }
            let outcome = outcome.await.expect("terminal receipt");
            assert_eq!(
                outcome,
                if closed_preparation {
                    CompactOutcome::Cancelled
                } else {
                    CompactOutcome::CancelledWithEstimate { before: 123 }
                }
            );
            assert!(resume.await.expect("resume").is_none());
            assert!(!coordinator.is_running());
            assert!(coordinator.model_receipt.is_none());
            assert_eq!(coordinator.ledger.admit(0, 1), CompactAdmission::Accepted);
        }
    }

    #[tokio::test]
    async fn abandoned_receipt_after_dispatch_cancels_before_candidate_install() {
        for at_commit in [false, true] {
            let agent = Agent::with_security(
                Arc::new(SummaryModel),
                ToolRegistry::new(),
                None,
                None,
                std::env::temp_dir(),
            );
            let (_, actor) = AppServerSession::new(
                agent,
                SessionConfig {
                    runtime_policy: RuntimePolicy::interactive(),
                    workspace_root: std::env::temp_dir(),
                    initial_history: vec![],
                    model_context_limit: 30_000,
                },
            );
            let mut coordinator = ManualCompaction::new(0);
            let (compact, job) = tokio::sync::oneshot::channel();
            let (response, resume) = tokio::sync::oneshot::channel();
            let (receipt, outcome) = tokio::sync::oneshot::channel();
            let source = crate::compaction::HistorySource::bind(0, 0, &[], vec![]).expect("source");
            coordinator.start_boundary(
                &actor,
                crate::SteeringBoundaryRequest {
                    compact,
                    response,
                    submitted: Some(crate::ModelBoundarySubmission {
                        checkpoint: crate::SubmittedCheckpoint {
                            payload: "{}".into(),
                            source: source.clone(),
                            covered_end: 0,
                        },
                        receipt,
                        authority: None,
                    }),
                },
                None,
                &CancellationToken::new(),
            );
            let job = job.await.expect("dispatch").expect("job");
            let candidate = ManualCheckpointResult::Candidate {
                projection: Box::new(crate::checkpoint_projection::CheckpointProjection {
                    sequence: 0,
                    parent_sequence: None,
                    source,
                    covered: vec![],
                    content: "must not install".into(),
                    context_limit: 30_000,
                    private: crate::transient_text::PrivateTokens::default(),
                }),
                before: 100,
                after: 20,
                deadline: tokio::time::Instant::now() + std::time::Duration::from_secs(5),
            };
            let result = if at_commit {
                job.result.send(candidate).ok().expect("send");
                let result = coordinator.wait().await;
                drop(outcome);
                result
            } else {
                drop(outcome);
                let result =
                    tokio::time::timeout(std::time::Duration::from_secs(1), coordinator.wait())
                        .await
                        .expect("closed receipt wakes wait");
                drop(candidate);
                result
            };
            coordinator.complete(&actor, result);
            assert!(
                actor
                    .agent
                    .checkpoint_projection_state
                    .lock()
                    .expect("state")
                    .is_none()
            );
            assert!(resume.await.expect("resume").is_none());
            assert!(coordinator.model_receipt.is_none());
            assert_eq!(coordinator.ledger.admit(0, 1), CompactAdmission::Accepted);
        }
    }

    #[tokio::test]
    async fn cancelled_measurement_is_receipted_and_never_leaks_to_next_operation() {
        let agent = Agent::with_security(
            Arc::new(SummaryModel),
            ToolRegistry::new(),
            None,
            None,
            std::env::temp_dir(),
        );
        let (mut handle, actor) = AppServerSession::new(
            agent,
            SessionConfig {
                runtime_policy: RuntimePolicy::interactive(),
                workspace_root: std::env::temp_dir(),
                initial_history: vec![],
                model_context_limit: 30_000,
            },
        );
        let mut manual = ManualCompaction::new(0);
        manual.admit(&actor, 0, 1);
        *manual.measurement.lock().expect("measurement") = Some(12345);
        manual.cancel(&actor);
        manual.admit(&actor, 0, 1);
        manual.admit(&actor, 0, 2);
        manual.cancel(&actor);
        let mut outcomes = vec![];
        while let Ok(SessionEvent::CompactStatus { state, .. }) = handle.eq_rx.try_recv() {
            if let CompactState::Completed { outcome, replayed } = state {
                outcomes.push((outcome, replayed));
            }
        }
        assert_eq!(
            outcomes,
            vec![
                (
                    CompactOutcome::CancelledWithEstimate { before: 12345 },
                    false
                ),
                (
                    CompactOutcome::CancelledWithEstimate { before: 12345 },
                    true
                ),
                (CompactOutcome::Cancelled, false),
            ]
        );
    }
    use async_trait::async_trait;
    use talos_core::message::{AgentEvent, Message, StopReason, Usage};
    use talos_core::provider::{LanguageModel, ProviderResult};
    use talos_core::session::{RuntimePolicy, SessionConfig};
    use talos_core::tool::ToolRegistry;

    struct SummaryModel;
    #[async_trait]
    impl LanguageModel for SummaryModel {
        async fn stream(&self, messages: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
            let Message::User { content } = &messages[0] else {
                panic!("summary request")
            };
            let sources: Vec<serde_json::Value> =
                serde_json::from_str(content.split("\nSources: ").last().expect("sources"))
                    .expect("JSON");
            let item = serde_json::json!({"text":"retain original task", "sources":[sources[0]["source"]]});
            let payload =
                serde_json::json!({"schema_version":1,"summary":item,"goal":item,"next_step":item,
                "constraints":[],"completed":[],"in_progress":[],"pending":[],"decisions":[],
                "facts":[],"assumptions":[],"resources":[],"open_loops":[]})
                .to_string();
            let (tx, rx) = mpsc::channel(4);
            tx.send(AgentEvent::TextDelta { delta: payload })
                .await
                .expect("text");
            tx.send(AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: Usage::default(),
            })
            .await
            .expect("end");
            Ok(rx)
        }
    }

    #[tokio::test]
    async fn submitted_model_candidate_applies_before_boundary_resume_without_raw_rewrite() {
        let raw: Vec<_> = (0..30)
            .flat_map(|_| {
                [
                    Message::User {
                        content: "constraint ".repeat(400),
                    },
                    Message::Assistant {
                        content: "recorded".into(),
                        tool_calls: vec![],
                        reasoning: None,
                    },
                ]
            })
            .collect();
        let agent = Agent::with_security(
            Arc::new(SummaryModel),
            ToolRegistry::new(),
            None,
            None,
            std::env::temp_dir(),
        );
        let (mut handle, mut actor) = AppServerSession::new(
            agent,
            SessionConfig {
                runtime_policy: RuntimePolicy::interactive(),
                workspace_root: std::env::temp_dir(),
                initial_history: raw.clone(),
                model_context_limit: 30_000,
            },
        );
        let control = RuntimeAdmissionControl::new();
        actor.set_runtime_admission(control.clone());
        let token = CancellationToken::new();
        assert!(control.commit_start(token.clone()));
        let source = crate::compaction::HistorySource::bind(
            0,
            raw.len() as u64,
            &raw,
            (0..raw.len()).map(|i| format!("source-{i}")).collect(),
        )
        .expect("source");
        let item =
            serde_json::json!({"text":"Preserve constraint and continue", "sources":["source-0"]});
        let payload =
            serde_json::json!({"schema_version":1,"summary":item,"goal":item,"next_step":item,
            "constraints":[],"completed":[],"in_progress":[],"pending":[],"decisions":[],
            "facts":[],"assumptions":[],"resources":[],"open_loops":[]})
            .to_string();
        let (compact, job) = tokio::sync::oneshot::channel();
        let (response, mut resume) = tokio::sync::oneshot::channel();
        let (receipt, mut outcome) = tokio::sync::oneshot::channel();
        let mut coordinator = ManualCompaction::new(0);
        assert_eq!(
            coordinator.admit_model(
                &actor,
                crate::ModelBoundarySubmission {
                    checkpoint: crate::SubmittedCheckpoint {
                        payload,
                        source: source.clone(),
                        covered_end: 20
                    },
                    receipt,
                    authority: None,
                },
                &token
            ),
            Ok(0)
        );
        assert!(
            actor
                .agent
                .checkpoint_projection_state
                .lock()
                .expect("cache")
                .is_none()
        );
        assert!(outcome.try_recv().is_err(), "admission is not application");
        coordinator.start_boundary(
            &actor,
            crate::SteeringBoundaryRequest {
                compact,
                response,
                submitted: None,
            },
            None,
            &token,
        );
        let job = job.await.expect("dispatch").expect("job");
        let submitted = job.submitted_checkpoint.expect("model checkpoint");
        assert!(
            submitted
                .matches_source(
                    &source,
                    raw.len(),
                    tokio::time::Instant::now() + std::time::Duration::from_secs(2)
                )
                .await
        );
        let before = actor.agent.estimate_provider_request_tokens(&raw, &[]);
        let mut plan = crate::request_plan::ProviderRequestPlan {
            messages: raw.clone(),
            tool_definitions: vec![],
            estimated_tokens: before,
            ephemeral_image_tokens: 0,
            omitted_tool_exchanges: 0,
            tool_protocol: talos_core::tool::ToolProtocol::Native,
        };
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
        let mut projection = None;
        actor
            .agent
            .checkpoint_submitted_deferred(
                &mut plan,
                before * 2,
                &crate::transient_text::PrivateTokens::default(),
                &source,
                deadline,
                &submitted.payload,
                submitted.covered_end,
                &mut projection,
            )
            .await
            .expect("prepare");
        assert!(
            actor
                .agent
                .checkpoint_projection_state
                .lock()
                .expect("cache")
                .is_none()
        );
        assert!(resume.try_recv().is_err());
        assert!(outcome.try_recv().is_err());
        job.result
            .send(ManualCheckpointResult::Candidate {
                projection: Box::new(projection.expect("candidate")),
                before,
                after: plan.estimated_tokens,
                deadline,
            })
            .ok()
            .expect("deliver");
        let result = coordinator.wait().await;
        coordinator.complete(&actor, result);
        assert!(
            matches!(outcome.try_recv().expect("terminal before continuation"),
            CompactOutcome::Applied { before, after } if after < before)
        );
        assert!(resume.try_recv().expect("resume").is_none());
        assert!(
            actor
                .agent
                .checkpoint_projection_state
                .lock()
                .expect("cache")
                .is_some()
        );
        assert_eq!(actor.history, raw);
        assert!(
            matches!(handle.eq_rx.try_recv().expect("applied model terminal"),
                SessionEvent::ModelCheckpointTerminal { operation_id: 0, outcome: CompactOutcome::Applied { before, after }, .. } if after < before
            )
        );
        assert!(handle.eq_rx.try_recv().is_err(), "no aliased manual event");
        coordinator.admit(&actor, 0, 1);
        assert!(matches!(
            handle.eq_rx.try_recv().expect("manual slot reusable"),
            SessionEvent::CompactStatus {
                state: CompactState::Queued,
                ..
            }
        ));
    }

    #[tokio::test]
    async fn completed_candidate_waits_for_actor_cancel_or_apply_decision() {
        let raw: Vec<_> = (0..30)
            .flat_map(|_| {
                [
                    Message::User {
                        content: "constraint ".repeat(400),
                    },
                    Message::Assistant {
                        content: "recorded".into(),
                        tool_calls: vec![],
                        reasoning: None,
                    },
                ]
            })
            .collect();
        let agent = Agent::with_security(
            Arc::new(SummaryModel),
            ToolRegistry::new(),
            None,
            None,
            std::env::temp_dir(),
        );
        let (mut handle, mut actor) = AppServerSession::new(
            agent,
            SessionConfig {
                runtime_policy: RuntimePolicy::interactive(),
                workspace_root: std::env::temp_dir(),
                initial_history: raw.clone(),
                model_context_limit: 30_000,
            },
        );
        let control = RuntimeAdmissionControl::new();
        actor.set_runtime_admission(control.clone());
        let mut manual = ManualCompaction::new(0);
        for id in 1..=3 {
            let candidate = actor
                .agent
                .compact_session_history(raw.clone(), 0, 30_000, None)
                .await;
            assert!(matches!(
                &candidate,
                ManualCheckpointResult::Candidate { .. }
            ));
            assert!(
                actor
                    .agent
                    .checkpoint_projection_state
                    .lock()
                    .expect("cache")
                    .is_none()
            );
            assert_eq!(manual.ledger.admit(0, id), CompactAdmission::Accepted);
            let token = CancellationToken::new();
            assert!(control.commit_start(token.clone()));
            manual.running = Some((
                id,
                token,
                tokio::spawn(async { ManualCheckpointResult::Outcome(CompactOutcome::Failed) }),
            ));
            if id == 1 {
                control.interrupt_active();
            } else if id == 2 {
                manual.cancel_operation(&actor, 1, id);
                assert!(matches!(
                    handle.eq_rx.recv().await,
                    Some(SessionEvent::CompactStatus {
                        state: CompactState::StaleGeneration,
                        ..
                    })
                ));
                manual.cancel_operation(&actor, 0, id + 99);
                assert!(matches!(
                    handle.eq_rx.recv().await,
                    Some(SessionEvent::CompactStatus {
                        state: CompactState::Expired,
                        ..
                    })
                ));
                assert!(!manual.running.as_ref().expect("running").1.is_cancelled());
                manual.cancel_operation(&actor, 0, id);
            }
            manual.complete(&actor, candidate);
            let event = handle.eq_rx.recv().await.expect("terminal receipt");
            if id <= 2 {
                assert!(matches!(
                    event,
                    SessionEvent::CompactStatus {
                        state: CompactState::Completed {
                            outcome: CompactOutcome::Cancelled,
                            replayed: false
                        },
                        ..
                    }
                ));
                assert!(
                    actor
                        .agent
                        .checkpoint_projection_state
                        .lock()
                        .expect("cache")
                        .is_none()
                );
            } else {
                assert!(
                    matches!(event, SessionEvent::CompactStatus { state: CompactState::Completed { outcome: CompactOutcome::Applied { before, after }, replayed: false }, .. } if after < before)
                );
                assert!(
                    actor
                        .agent
                        .checkpoint_projection_state
                        .lock()
                        .expect("cache")
                        .is_some()
                );
                manual.cancel_operation(&actor, 0, id);
                assert!(
                    matches!(
                        handle.eq_rx.recv().await,
                        Some(SessionEvent::CompactStatus {
                            state: CompactState::Completed {
                                outcome: CompactOutcome::Applied { .. },
                                replayed: true
                            },
                            ..
                        })
                    ),
                    "completed cancellation must return prior receipt without undoing application"
                );
            }
            assert_eq!(actor.history, raw);
        }
    }
}
