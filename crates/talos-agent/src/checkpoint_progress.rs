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

/// Return only audited constant diagnostics, never provider or transcript text.
pub(crate) fn failure_diagnostic(error: &crate::compaction::CompactionError) -> &'static str {
    use crate::compaction::CompactionError;
    match error {
        CompactionError::CompactionFailed(reason) => match reason.as_str() {
            "checkpoint aggregate output limit exceeded" => {
                "checkpoint aggregate output limit exceeded"
            }
            "checkpoint batch serialization failed" => "checkpoint batch serialization failed",
            "checkpoint collection limit exceeded" => "checkpoint collection limit exceeded",
            "checkpoint contains private content" => "checkpoint contains private content",
            "checkpoint deadline exceeded" => "checkpoint deadline exceeded",
            "checkpoint exceeds rendered token budget" => {
                "checkpoint exceeds rendered token budget"
            }
            "checkpoint folding limit exceeded" => "checkpoint folding limit exceeded",
            "checkpoint identity unavailable" => "checkpoint identity unavailable",
            "checkpoint leaves request over capacity" => "checkpoint leaves request over capacity",
            "checkpoint payload exceeds byte limit" => "checkpoint payload exceeds byte limit",
            "checkpoint provider request failed" => "checkpoint provider request failed",
            "checkpoint reclaims insufficient context" => {
                "checkpoint reclaims insufficient context"
            }
            "checkpoint request exceeds capacity" => "checkpoint request exceeds capacity",
            "checkpoint requires complete tool exchange" => {
                "checkpoint requires complete tool exchange"
            }
            "checkpoint retry suppressed" => "checkpoint retry suppressed",
            "checkpoint schema serialization failed" => "checkpoint schema serialization failed",
            "checkpoint serialization failed" => "checkpoint serialization failed",
            "checkpoint source binding changed" => "checkpoint source binding changed",
            "checkpoint source byte limit exceeded" => "checkpoint source byte limit exceeded",
            "checkpoint source is outside coverage" => "checkpoint source is outside coverage",
            "checkpoint source serialization failed" => "checkpoint source serialization failed",
            "checkpoint text offsets unavailable" => "checkpoint text offsets unavailable",
            "fixed request overhead exceeds capacity" => "fixed request overhead exceeds capacity",
            "invalid checkpoint schema" => "invalid checkpoint schema",
            "invalid checkpoint text" => "invalid checkpoint text",
            "irreducible checkpoint exchange" => "irreducible checkpoint exchange",
            "irreducible checkpoint text" => "irreducible checkpoint text",
            "no eligible checkpoint prefix" => "no eligible checkpoint prefix",
            "unsupported checkpoint version" => "unsupported checkpoint version",
            "summary exceeds byte limit" => "summary exceeds byte limit",
            "summary stream contains tool activity" => "summary stream contains tool activity",
            "invalid summary terminal sequence" => "invalid summary terminal sequence",
            "summary missing successful terminal or content" => {
                "summary missing successful terminal or content"
            }
            _ => "unclassified preparation failure",
        },
        CompactionError::ProviderError(reason) => match reason.as_str() {
            "checkpoint deadline exceeded" => "checkpoint deadline exceeded",
            "checkpoint provider request failed" => "checkpoint provider request failed",
            "summary stream failed" => "summary stream failed",
            _ => "provider failure",
        },
        CompactionError::CircuitBreakerTripped => "circuit breaker tripped",
        CompactionError::TokenEstimationFailed => "token estimation failed",
    }
}

#[cfg(test)]
mod diagnostic_tests {
    use super::failure_diagnostic;
    use crate::compaction::CompactionError;

    #[test]
    fn diagnostics_distinguish_validation_without_echoing_untrusted_errors() {
        for reason in [
            "invalid checkpoint schema",
            "checkpoint source is outside coverage",
            "checkpoint exceeds rendered token budget",
            "checkpoint requires complete tool exchange",
        ] {
            assert_eq!(
                failure_diagnostic(&CompactionError::CompactionFailed(reason.into())),
                reason
            );
        }
        for reason in [
            "secret transcript",
            "invalid checkpoint schema: secret transcript",
        ] {
            assert_eq!(
                failure_diagnostic(&CompactionError::CompactionFailed(reason.into())),
                "unclassified preparation failure"
            );
            assert_eq!(
                failure_diagnostic(&CompactionError::ProviderError(reason.into())),
                "provider failure"
            );
        }
    }
}
