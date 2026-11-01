//! Per-request feedback sent directly to the session UI, never to persistence.

use talos_core::session::{AutomaticCompactFailure, AutomaticCompactState, SessionEvent};
use tokio::sync::mpsc;

/// Session-local advisory state. It never authorizes compaction or replaces the
/// next request's full admission check. A fresh slot isolates each turn.
#[derive(Clone, Default)]
pub(crate) struct PostTurnPressure(std::sync::Arc<std::sync::Mutex<Option<u32>>>);

impl PostTurnPressure {
    pub(crate) fn record(&self, estimated: u32, limit: u32) {
        use crate::compaction::RequestPressure;
        if let Ok(mut pending) = self.0.lock() {
            *pending = matches!(
                RequestPressure::assess(estimated, limit),
                RequestPressure::Critical | RequestPressure::Emergency
            )
            .then_some(estimated);
        }
    }

    pub(crate) fn take(&self) -> Option<u32> {
        self.0.lock().ok()?.take()
    }
}

#[derive(Clone)]
pub(crate) struct CheckpointObserver {
    pub(crate) sender: mpsc::UnboundedSender<SessionEvent>,
    pub(crate) session_id: String,
    pub(crate) generation: u64,
    pub(crate) admission: Option<crate::session::RuntimeAdmissionControl>,
    pub(crate) post_turn_pressure: PostTurnPressure,
}

impl CheckpointObserver {
    pub(crate) fn emit(&self, state: AutomaticCompactState) {
        let _ = self.sender.send(SessionEvent::AutomaticCompactStatus {
            session_id: self.session_id.clone(),
            session_generation: self.generation,
            state,
        });
    }
}

pub(crate) struct CheckpointProgress<'a> {
    observer: Option<&'a CheckpointObserver>,
    before: u32,
    finished: bool,
}

impl<'a> CheckpointProgress<'a> {
    pub(crate) fn start(observer: Option<&'a CheckpointObserver>, before: u32) -> Self {
        if let Some(observer) = observer {
            observer.emit(AutomaticCompactState::Started { before });
        }
        Self {
            observer,
            before,
            finished: false,
        }
    }

    pub(crate) fn cancel(mut self) {
        if let Some(observer) = self.observer {
            observer.emit(AutomaticCompactState::Cancelled {
                before: self.before,
            });
        }
        self.finished = true;
    }

    pub(crate) fn finish(mut self, result: Result<u32, AutomaticCompactFailure>) {
        if let Some(observer) = self.observer {
            observer.emit(match result {
                Ok(after) => AutomaticCompactState::Applied {
                    before: self.before,
                    after,
                },
                Err(reason) => AutomaticCompactState::Failed {
                    before: self.before,
                    reason,
                },
            });
        }
        self.finished = true;
    }
}

impl Drop for CheckpointProgress<'_> {
    fn drop(&mut self) {
        if !self.finished
            && let Some(observer) = self.observer
        {
            observer.emit(AutomaticCompactState::Cancelled {
                before: self.before,
            });
        }
    }
}

/// Translate only the typed category; provider/source/error strings stay private.
pub(crate) fn failure_reason(
    error: &crate::compaction::CompactionError,
) -> AutomaticCompactFailure {
    use crate::compaction::CompactionError;
    match error {
        CompactionError::ProviderError(_) => AutomaticCompactFailure::Provider,
        CompactionError::CompactionFailed(_) => AutomaticCompactFailure::Preparation,
        CompactionError::CircuitBreakerTripped => AutomaticCompactFailure::Preparation,
        CompactionError::TokenEstimationFailed => AutomaticCompactFailure::Preparation,
    }
}
