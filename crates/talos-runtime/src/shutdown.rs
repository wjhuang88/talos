use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use talos_agent::session::{
    RuntimeActiveTurnOutcome as AgentActiveTurnOutcome, RuntimeAdmissionClose,
    RuntimeAdmissionControl,
    RuntimeDurableReconciliationOutcome as AgentDurableReconciliationOutcome,
    RuntimeShutdownTurnPolicy,
};
use talos_core::session::SessionOp;
use thiserror::Error;
use tokio::runtime::Handle;
use tokio::sync::{Notify, mpsc};
use tokio::task::{JoinError, JoinHandle};

use crate::RuntimeResult;

const LEGACY_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);
static NEXT_SHUTDOWN_PLAN_ID: AtomicU64 = AtomicU64::new(1);

/// Policy for the turn active when runtime admission closes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ActiveTurnPolicy {
    /// Let the active turn finish for at most `grace`, then interrupt it.
    FinishCurrent {
        /// Portion of the total timeout reserved for graceful turn completion.
        grace: Duration,
    },
    /// Interrupt the active turn immediately through its Session token.
    Interrupt,
}

/// Validation errors for [`ShutdownOptions`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum ShutdownOptionsError {
    /// The total shutdown timeout must be greater than zero.
    #[error("shutdown total timeout must be greater than zero")]
    ZeroTotalTimeout,
    /// Finish grace must leave time inside the total shutdown timeout.
    #[error("finish grace must be less than the total shutdown timeout")]
    FinishGraceNotLessThanTotal,
    /// The timeout cannot be represented by the monotonic clock.
    #[error("shutdown total timeout exceeds the monotonic clock range")]
    TotalTimeoutOutOfRange,
}

/// Validated options for one runtime shutdown request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShutdownOptions {
    total_timeout: Duration,
    active_turn_policy: ActiveTurnPolicy,
}

impl ShutdownOptions {
    /// Validates a total timeout and active-turn policy before runtime access.
    pub fn new(
        total_timeout: Duration,
        active_turn_policy: ActiveTurnPolicy,
    ) -> Result<Self, ShutdownOptionsError> {
        if total_timeout.is_zero() {
            return Err(ShutdownOptionsError::ZeroTotalTimeout);
        }
        if Instant::now().checked_add(total_timeout).is_none() {
            return Err(ShutdownOptionsError::TotalTimeoutOutOfRange);
        }
        if let ActiveTurnPolicy::FinishCurrent { grace } = active_turn_policy
            && grace >= total_timeout
        {
            return Err(ShutdownOptionsError::FinishGraceNotLessThanTotal);
        }
        Ok(Self {
            total_timeout,
            active_turn_policy,
        })
    }

    /// Creates an immediate-interrupt shutdown plan.
    pub fn interrupt(total_timeout: Duration) -> Result<Self, ShutdownOptionsError> {
        Self::new(total_timeout, ActiveTurnPolicy::Interrupt)
    }

    /// Creates a finish-current shutdown plan.
    pub fn finish_current(
        total_timeout: Duration,
        grace: Duration,
    ) -> Result<Self, ShutdownOptionsError> {
        Self::new(total_timeout, ActiveTurnPolicy::FinishCurrent { grace })
    }

    /// Returns the one total timeout used by all B-stage shutdown work.
    #[must_use]
    pub const fn total_timeout(&self) -> Duration {
        self.total_timeout
    }

    /// Returns the selected active-turn policy.
    #[must_use]
    pub const fn active_turn_policy(&self) -> ActiveTurnPolicy {
        self.active_turn_policy
    }

    pub(crate) fn legacy_default() -> Self {
        Self {
            total_timeout: LEGACY_SHUTDOWN_TIMEOUT,
            active_turn_policy: ActiveTurnPolicy::Interrupt,
        }
    }
}

/// Opaque identifier of the first accepted shutdown plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShutdownPlanId(u64);

/// Redacted outcome for the turn active at the shutdown fence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ShutdownActiveTurnOutcome {
    /// No turn was start-committed at the fence.
    Idle,
    /// The active turn completed before interruption won.
    Finished,
    /// The existing Session cancellation/finalization path completed.
    InterruptedAndFinalized,
    /// The active turn reached an error terminal result.
    Failed,
    /// The deadline contained the actor before terminal reconciliation was observed.
    Unreconciled,
}

/// Redacted durable pending-custody outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ShutdownDurableOutcome {
    /// Pending custody was reconciled successfully.
    Completed {
        /// In-memory pending submissions rejected as Session-closed.
        rejected_pending: u32,
    },
    /// At least one pending-custody transition failed.
    Failed {
        /// In-memory pending submissions rejected before the failure was observed.
        rejected_pending: u32,
    },
    /// The global deadline expired before actor reconciliation completed.
    NotRunDeadline,
}

/// Redacted actor containment outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ShutdownActorOutcome {
    /// The Session actor joined normally.
    Joined,
    /// The Session actor task failed while joining.
    Failed,
    /// The global deadline forced task abortion.
    Contained,
}

/// Immutable redacted result shared by every shutdown caller.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ShutdownReport {
    plan_id: ShutdownPlanId,
    active_turn_policy: ActiveTurnPolicy,
    elapsed: Duration,
    deadline_exhausted: bool,
    active_turn: ShutdownActiveTurnOutcome,
    durable_reconciliation: ShutdownDurableOutcome,
    actor: ShutdownActorOutcome,
}

impl ShutdownReport {
    /// Returns the accepted plan identifier.
    #[must_use]
    pub const fn plan_id(&self) -> ShutdownPlanId {
        self.plan_id
    }

    /// Returns the first accepted active-turn policy.
    #[must_use]
    pub const fn active_turn_policy(&self) -> ActiveTurnPolicy {
        self.active_turn_policy
    }

    /// Returns monotonic elapsed shutdown time.
    #[must_use]
    pub const fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// Reports whether the one total deadline was exhausted.
    #[must_use]
    pub const fn deadline_exhausted(&self) -> bool {
        self.deadline_exhausted
    }

    /// Returns the redacted active-turn outcome.
    #[must_use]
    pub const fn active_turn(&self) -> ShutdownActiveTurnOutcome {
        self.active_turn
    }

    /// Returns the redacted durable reconciliation outcome.
    #[must_use]
    pub const fn durable_reconciliation(&self) -> ShutdownDurableOutcome {
        self.durable_reconciliation
    }

    /// Returns the actor join/containment outcome.
    #[must_use]
    pub const fn actor(&self) -> ShutdownActorOutcome {
        self.actor
    }

    /// Returns true only when all B-stage shutdown work completed cleanly.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        !self.deadline_exhausted
            && !matches!(self.active_turn, ShutdownActiveTurnOutcome::Unreconciled)
            && matches!(
                self.durable_reconciliation,
                ShutdownDurableOutcome::Completed { .. }
            )
            && matches!(self.actor, ShutdownActorOutcome::Joined)
    }
}

/// Cloneable shutdown-only controller for one runtime.
#[derive(Clone)]
pub struct RuntimeShutdownHandle {
    pub(crate) coordinator: Arc<ShutdownCoordinator>,
}

impl RuntimeShutdownHandle {
    /// Starts or joins shutdown and returns the one cached redacted report.
    pub async fn shutdown(&self, options: ShutdownOptions) -> RuntimeResult<ShutdownReport> {
        self.coordinator.shutdown(options).await
    }
}

#[derive(Debug, Clone, Copy)]
struct AcceptedPlan {
    id: ShutdownPlanId,
    options: ShutdownOptions,
    accepted_at: Instant,
    deadline: Instant,
    active_at_fence: bool,
}

enum CoordinatorState {
    Open,
    Closing(AcceptedPlan),
    Closed {
        report: ShutdownReport,
        actor_join_error: Option<JoinError>,
    },
}

pub(crate) struct ShutdownCoordinator {
    admission: RuntimeAdmissionControl,
    command_tx: mpsc::Sender<SessionOp>,
    actor_task: Mutex<Option<JoinHandle<()>>>,
    state: Mutex<CoordinatorState>,
    changed: Notify,
    runtime: Handle,
}

impl ShutdownCoordinator {
    pub(crate) fn new(
        admission: RuntimeAdmissionControl,
        command_tx: mpsc::Sender<SessionOp>,
        actor_task: JoinHandle<()>,
        runtime: Handle,
    ) -> Arc<Self> {
        Arc::new(Self {
            admission,
            command_tx,
            actor_task: Mutex::new(Some(actor_task)),
            state: Mutex::new(CoordinatorState::Open),
            changed: Notify::new(),
            runtime,
        })
    }

    fn state(&self) -> MutexGuard<'_, CoordinatorState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn actor_task(&self) -> MutexGuard<'_, Option<JoinHandle<()>>> {
        self.actor_task
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn initiate(self: &Arc<Self>, options: ShutdownOptions) -> ShutdownPlanId {
        let candidate = NEXT_SHUTDOWN_PLAN_ID.fetch_add(1, Ordering::Relaxed);
        let policy = match options.active_turn_policy {
            ActiveTurnPolicy::FinishCurrent { .. } => RuntimeShutdownTurnPolicy::FinishCurrent,
            ActiveTurnPolicy::Interrupt => RuntimeShutdownTurnPolicy::Interrupt,
        };
        match self.admission.begin_shutdown(candidate, policy) {
            RuntimeAdmissionClose::Existing { plan_id } => ShutdownPlanId(plan_id),
            RuntimeAdmissionClose::Accepted { active_at_fence } => {
                let accepted_at = Instant::now();
                let deadline = accepted_at
                    .checked_add(options.total_timeout)
                    .unwrap_or(accepted_at);
                let plan = AcceptedPlan {
                    id: ShutdownPlanId(candidate),
                    options,
                    accepted_at,
                    deadline,
                    active_at_fence,
                };
                *self.state() = CoordinatorState::Closing(plan);
                self.changed.notify_waiters();
                if matches!(options.active_turn_policy, ActiveTurnPolicy::Interrupt) {
                    self.admission.interrupt_active();
                }
                let coordinator = self.clone();
                let spawn = catch_unwind(AssertUnwindSafe(|| {
                    self.runtime.spawn(async move {
                        coordinator.drive(plan).await;
                    })
                }));
                if spawn.is_err() {
                    self.publish_driver_failure(plan);
                }
                plan.id
            }
        }
    }

    pub(crate) async fn shutdown(
        self: &Arc<Self>,
        options: ShutdownOptions,
    ) -> RuntimeResult<ShutdownReport> {
        let plan_id = self.initiate(options);
        self.wait_for_report(plan_id).await
    }

    pub(crate) fn initiate_default(self: &Arc<Self>) {
        let _ = self.initiate(ShutdownOptions::legacy_default());
    }

    pub(crate) fn commit_reserved(
        &self,
        permit: mpsc::Permit<'_, SessionOp>,
        op: SessionOp,
    ) -> Result<(), SessionOp> {
        self.admission.commit_reserved(permit, op)
    }

    pub(crate) fn is_admission_open(&self) -> bool {
        self.admission.is_open()
    }

    async fn wait_for_report(&self, plan_id: ShutdownPlanId) -> RuntimeResult<ShutdownReport> {
        loop {
            let changed = self.changed.notified();
            match &*self.state() {
                CoordinatorState::Closed { report, .. } => return Ok(report.clone()),
                CoordinatorState::Closing(plan) if plan.id == plan_id => {}
                CoordinatorState::Open | CoordinatorState::Closing(_) => {}
            }
            changed.await;
        }
    }

    pub(crate) fn take_actor_join_error(&self) -> Option<JoinError> {
        match &mut *self.state() {
            CoordinatorState::Closed {
                actor_join_error, ..
            } => actor_join_error.take(),
            CoordinatorState::Open | CoordinatorState::Closing(_) => None,
        }
    }

    async fn drive(self: Arc<Self>, plan: AcceptedPlan) {
        let deadline = tokio::time::Instant::from_std(plan.deadline);
        let send_result =
            tokio::time::timeout_at(deadline, self.command_tx.send(SessionOp::Shutdown)).await;

        if matches!(
            plan.options.active_turn_policy,
            ActiveTurnPolicy::FinishCurrent { .. }
        ) && plan.active_at_fence
        {
            let ActiveTurnPolicy::FinishCurrent { grace } = plan.options.active_turn_policy else {
                unreachable!("policy matched above")
            };
            let grace_end = plan.accepted_at.checked_add(grace).unwrap_or(plan.deadline);
            let grace_deadline = tokio::time::Instant::from_std(plan.deadline.min(grace_end));
            if tokio::time::timeout_at(grace_deadline, self.admission.wait_until_idle())
                .await
                .is_err()
            {
                self.admission.interrupt_active();
            }
        }

        let mut actor_join_error = None;
        let actor = self.actor_task().take();
        let actor_outcome = if let Some(mut actor) = actor {
            match tokio::time::timeout_at(deadline, &mut actor).await {
                Ok(Ok(())) => ShutdownActorOutcome::Joined,
                Ok(Err(error)) => {
                    actor_join_error = Some(error);
                    ShutdownActorOutcome::Failed
                }
                Err(_) => {
                    actor.abort();
                    ShutdownActorOutcome::Contained
                }
            }
        } else {
            ShutdownActorOutcome::Failed
        };

        let elapsed = plan.accepted_at.elapsed();
        let deadline_exhausted = Instant::now() >= plan.deadline
            || send_result.is_err()
            || matches!(actor_outcome, ShutdownActorOutcome::Contained);
        let snapshot = self.admission.snapshot();
        let active_turn = if !plan.active_at_fence {
            ShutdownActiveTurnOutcome::Idle
        } else {
            match snapshot.active_turn {
                AgentActiveTurnOutcome::Idle => ShutdownActiveTurnOutcome::Idle,
                AgentActiveTurnOutcome::Finished => ShutdownActiveTurnOutcome::Finished,
                AgentActiveTurnOutcome::InterruptedAndFinalized => {
                    ShutdownActiveTurnOutcome::InterruptedAndFinalized
                }
                AgentActiveTurnOutcome::Failed => ShutdownActiveTurnOutcome::Failed,
                AgentActiveTurnOutcome::Running => ShutdownActiveTurnOutcome::Unreconciled,
            }
        };
        let durable_reconciliation = match snapshot.durable_reconciliation {
            AgentDurableReconciliationOutcome::Completed => ShutdownDurableOutcome::Completed {
                rejected_pending: snapshot.rejected_pending,
            },
            AgentDurableReconciliationOutcome::Failed => ShutdownDurableOutcome::Failed {
                rejected_pending: snapshot.rejected_pending,
            },
            AgentDurableReconciliationOutcome::Pending => ShutdownDurableOutcome::NotRunDeadline,
        };
        let report = ShutdownReport {
            plan_id: plan.id,
            active_turn_policy: plan.options.active_turn_policy,
            elapsed,
            deadline_exhausted,
            active_turn,
            durable_reconciliation,
            actor: actor_outcome,
        };
        self.admission.mark_closed();
        *self.state() = CoordinatorState::Closed {
            report,
            actor_join_error,
        };
        self.changed.notify_waiters();
    }

    fn publish_driver_failure(&self, plan: AcceptedPlan) {
        if let Some(actor) = self.actor_task().take() {
            actor.abort();
        }
        let snapshot = self.admission.snapshot();
        self.admission.mark_closed();
        *self.state() = CoordinatorState::Closed {
            report: ShutdownReport {
                plan_id: plan.id,
                active_turn_policy: plan.options.active_turn_policy,
                elapsed: plan.accepted_at.elapsed(),
                deadline_exhausted: true,
                active_turn: if plan.active_at_fence {
                    ShutdownActiveTurnOutcome::Unreconciled
                } else {
                    ShutdownActiveTurnOutcome::Idle
                },
                durable_reconciliation: match snapshot.durable_reconciliation {
                    AgentDurableReconciliationOutcome::Completed => {
                        ShutdownDurableOutcome::Completed {
                            rejected_pending: snapshot.rejected_pending,
                        }
                    }
                    AgentDurableReconciliationOutcome::Failed => ShutdownDurableOutcome::Failed {
                        rejected_pending: snapshot.rejected_pending,
                    },
                    AgentDurableReconciliationOutcome::Pending => {
                        ShutdownDurableOutcome::NotRunDeadline
                    }
                },
                actor: ShutdownActorOutcome::Contained,
            },
            actor_join_error: None,
        };
        self.changed.notify_waiters();
    }
}
