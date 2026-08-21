use std::sync::{Arc, Mutex, MutexGuard};

use talos_core::session::{SessionOp, TurnCompletionStatus};
use tokio::sync::{Notify, mpsc};
use tokio_util::sync::CancellationToken;

/// Runtime-selected behavior for the turn active at the shutdown fence.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeShutdownTurnPolicy {
    /// Give the active turn a bounded chance to finish before interruption.
    FinishCurrent,
    /// Interrupt the active turn immediately.
    Interrupt,
}

/// Result of atomically closing runtime admission.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeAdmissionClose {
    /// This request established the shutdown fence.
    Accepted {
        /// Whether a turn was already start-committed at the fence.
        active_at_fence: bool,
    },
    /// Another request already established the shutdown fence.
    Existing {
        /// Opaque identifier of the accepted plan.
        plan_id: u64,
    },
}

/// Redacted active-turn state observed by the runtime shutdown coordinator.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeActiveTurnOutcome {
    /// No turn was start-committed at the fence.
    Idle,
    /// The start-committed turn is still running.
    Running,
    /// The turn completed successfully.
    Finished,
    /// The turn was interrupted and finalized as cancelled.
    InterruptedAndFinalized,
    /// The turn reached an error terminal result.
    Failed,
}

/// Redacted durable-custody state observed during actor shutdown.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeDurableReconciliationOutcome {
    /// The actor has not completed shutdown custody reconciliation.
    Pending,
    /// Pending custody was reconciled without an observed error.
    Completed,
    /// At least one custody transition failed.
    Failed,
}

/// Redacted actor-owned lifecycle snapshot.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeLifecycleSnapshot {
    /// Active-turn outcome at the time of observation.
    pub active_turn: RuntimeActiveTurnOutcome,
    /// Durable reconciliation outcome at the time of observation.
    pub durable_reconciliation: RuntimeDurableReconciliationOutcome,
    /// Number of in-memory pending submissions rejected during shutdown.
    pub rejected_pending: u32,
}

#[derive(Debug)]
enum AdmissionPhase {
    Open,
    Closing {
        plan_id: u64,
        policy: RuntimeShutdownTurnPolicy,
    },
    Closed {
        plan_id: u64,
    },
}

#[derive(Debug)]
struct ActiveTurn {
    token: CancellationToken,
    interrupt_requested: bool,
}

#[derive(Debug)]
struct AdmissionState {
    phase: AdmissionPhase,
    active: Option<ActiveTurn>,
    active_outcome: RuntimeActiveTurnOutcome,
    durable_reconciliation: RuntimeDurableReconciliationOutcome,
    rejected_pending: u32,
    actor_stopped: bool,
}

/// Shared SDK-admission and actor-start linearization seam.
///
/// This type is public only so the `talos-runtime` facade can install it into
/// the Session actor. Direct Session users retain default-open behavior when
/// no control is installed.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct RuntimeAdmissionControl {
    state: Arc<Mutex<AdmissionState>>,
    active_changed: Arc<Notify>,
}

impl RuntimeAdmissionControl {
    /// Creates an open runtime admission/start seam.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(AdmissionState {
                phase: AdmissionPhase::Open,
                active: None,
                active_outcome: RuntimeActiveTurnOutcome::Idle,
                durable_reconciliation: RuntimeDurableReconciliationOutcome::Pending,
                rejected_pending: 0,
                actor_stopped: false,
            })),
            active_changed: Arc::new(Notify::new()),
        }
    }

    fn lock(&self) -> MutexGuard<'_, AdmissionState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Returns whether SDK admission is still open.
    #[must_use]
    pub fn is_open(&self) -> bool {
        matches!(self.lock().phase, AdmissionPhase::Open)
    }

    /// Commits a previously reserved SDK command only while admission is open.
    pub fn commit_reserved(
        &self,
        permit: mpsc::Permit<'_, SessionOp>,
        op: SessionOp,
    ) -> Result<(), SessionOp> {
        let state = self.lock();
        if matches!(state.phase, AdmissionPhase::Open) {
            permit.send(op);
            Ok(())
        } else {
            Err(op)
        }
    }

    /// Establishes the shutdown fence or returns the existing winning plan.
    pub fn begin_shutdown(
        &self,
        plan_id: u64,
        policy: RuntimeShutdownTurnPolicy,
    ) -> RuntimeAdmissionClose {
        let mut state = self.lock();
        match state.phase {
            AdmissionPhase::Open => {
                let active_at_fence = state.active.is_some();
                state.phase = AdmissionPhase::Closing { plan_id, policy };
                RuntimeAdmissionClose::Accepted { active_at_fence }
            }
            AdmissionPhase::Closing { plan_id, .. } => RuntimeAdmissionClose::Existing { plan_id },
            AdmissionPhase::Closed { plan_id } => RuntimeAdmissionClose::Existing { plan_id },
        }
    }

    /// Installs the active turn token at the actor's start-commit point.
    pub fn commit_start(&self, token: CancellationToken) -> bool {
        let mut state = self.lock();
        if !matches!(state.phase, AdmissionPhase::Open) {
            return false;
        }
        state.active = Some(ActiveTurn {
            token,
            interrupt_requested: false,
        });
        state.active_outcome = RuntimeActiveTurnOutcome::Running;
        true
    }

    /// Records the actor-owned terminal result and releases the active slot.
    pub fn finish_active(&self, completion: Option<&TurnCompletionStatus>) {
        let mut state = self.lock();
        let interrupted = state
            .active
            .take()
            .is_some_and(|active| active.interrupt_requested);
        state.active_outcome = match completion {
            Some(TurnCompletionStatus::Success { .. }) => RuntimeActiveTurnOutcome::Finished,
            Some(TurnCompletionStatus::Cancelled) if interrupted => {
                RuntimeActiveTurnOutcome::InterruptedAndFinalized
            }
            Some(TurnCompletionStatus::Cancelled | TurnCompletionStatus::Error { .. }) | None => {
                RuntimeActiveTurnOutcome::Failed
            }
        };
        drop(state);
        self.active_changed.notify_waiters();
    }

    /// Cancels the start-committed turn, if any, without waiting.
    pub fn interrupt_active(&self) {
        let token = {
            let mut state = self.lock();
            state.active.as_mut().map(|active| {
                active.interrupt_requested = true;
                active.token.clone()
            })
        };
        if let Some(token) = token {
            token.cancel();
        }
    }

    /// Returns the accepted actor policy, if admission is closing.
    #[must_use]
    pub fn shutdown_policy(&self) -> Option<RuntimeShutdownTurnPolicy> {
        let state = self.lock();
        match state.phase {
            AdmissionPhase::Closing { policy, .. } => Some(policy),
            AdmissionPhase::Open | AdmissionPhase::Closed { .. } => None,
        }
    }

    /// Waits until no start-committed turn remains.
    pub async fn wait_until_idle(&self) {
        loop {
            let notified = self.active_changed.notified();
            if self.lock().active.is_none() {
                return;
            }
            notified.await;
        }
    }

    /// Records actor-owned pending-custody reconciliation without free text.
    pub fn record_reconciliation(&self, rejected_pending: usize, succeeded: bool) {
        let mut state = self.lock();
        state.rejected_pending = u32::try_from(rejected_pending).unwrap_or(u32::MAX);
        state.durable_reconciliation = if succeeded {
            RuntimeDurableReconciliationOutcome::Completed
        } else {
            RuntimeDurableReconciliationOutcome::Failed
        };
        drop(state);
        self.active_changed.notify_waiters();
    }

    /// Waits until active work and durable custody are terminal, or the actor stops.
    ///
    /// Returns `true` only for the normal active-plus-durable barrier. A `false`
    /// result lets the runtime report actor/durable failure and continue safe
    /// cleanup without joining the actor before registered finalizers run.
    pub async fn wait_until_shutdown_barrier(&self) -> bool {
        loop {
            let notified = self.active_changed.notified();
            {
                let state = self.lock();
                if state.active.is_none()
                    && !matches!(
                        state.durable_reconciliation,
                        RuntimeDurableReconciliationOutcome::Pending
                    )
                {
                    return true;
                }
                if state.actor_stopped {
                    return false;
                }
            }
            notified.await;
        }
    }

    /// Records that the Session actor future returned or unwound.
    pub fn record_actor_stopped(&self) {
        let mut state = self.lock();
        state.actor_stopped = true;
        drop(state);
        self.active_changed.notify_waiters();
    }

    /// Returns the current redacted actor-owned lifecycle projection.
    #[must_use]
    pub fn snapshot(&self) -> RuntimeLifecycleSnapshot {
        let state = self.lock();
        RuntimeLifecycleSnapshot {
            active_turn: state.active_outcome,
            durable_reconciliation: state.durable_reconciliation,
            rejected_pending: state.rejected_pending,
        }
    }

    /// Marks the lifecycle seam terminal after the runtime driver contains the actor.
    pub fn mark_closed(&self) {
        let mut state = self.lock();
        let plan_id = match state.phase {
            AdmissionPhase::Closing { plan_id, .. } | AdmissionPhase::Closed { plan_id } => plan_id,
            AdmissionPhase::Open => return,
        };
        state.phase = AdmissionPhase::Closed { plan_id };
    }
}

impl Default for RuntimeAdmissionControl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn shutdown_fence_rejects_a_reserved_but_uncommitted_submission() {
        let control = RuntimeAdmissionControl::new();
        let (tx, mut rx) = mpsc::channel(1);
        let permit = tx.reserve().await.expect("channel remains open");

        assert_eq!(
            control.begin_shutdown(7, RuntimeShutdownTurnPolicy::Interrupt),
            RuntimeAdmissionClose::Accepted {
                active_at_fence: false
            }
        );
        assert!(
            control
                .commit_reserved(
                    permit,
                    SessionOp::Submit {
                        message: "must-not-enqueue".into(),
                    },
                )
                .is_err()
        );
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn committed_send_is_pre_fence_even_before_actor_receive() {
        let control = RuntimeAdmissionControl::new();
        let (tx, mut rx) = mpsc::channel(1);
        let permit = tx.reserve().await.expect("channel remains open");
        control
            .commit_reserved(
                permit,
                SessionOp::Submit {
                    message: "pre-fence".into(),
                },
            )
            .expect("open admission commits reserved send");

        assert_eq!(
            control.begin_shutdown(11, RuntimeShutdownTurnPolicy::FinishCurrent),
            RuntimeAdmissionClose::Accepted {
                active_at_fence: false
            }
        );
        assert!(matches!(
            rx.recv().await,
            Some(SessionOp::Submit { message }) if message == "pre-fence"
        ));
    }

    #[tokio::test]
    async fn start_commit_and_shutdown_have_one_observable_order() {
        let control = RuntimeAdmissionControl::new();
        let token = CancellationToken::new();
        assert!(control.commit_start(token.clone()));
        assert_eq!(
            control.begin_shutdown(19, RuntimeShutdownTurnPolicy::Interrupt),
            RuntimeAdmissionClose::Accepted {
                active_at_fence: true
            }
        );

        control.interrupt_active();
        token.cancelled().await;
        control.finish_active(Some(&TurnCompletionStatus::Cancelled));
        assert_eq!(
            control.snapshot().active_turn,
            RuntimeActiveTurnOutcome::InterruptedAndFinalized
        );

        let losing_token = CancellationToken::new();
        assert!(!control.commit_start(losing_token));
        assert_eq!(
            control.begin_shutdown(23, RuntimeShutdownTurnPolicy::FinishCurrent),
            RuntimeAdmissionClose::Existing { plan_id: 19 }
        );
    }
}
