use thiserror::Error;

/// Errors that can occur during context compaction.
#[derive(Debug, Error)]
pub enum CompactionError {
    /// Token estimation failed during compaction.
    #[error("token estimation failed")]
    TokenEstimationFailed,

    /// Compaction could not reduce context sufficiently.
    #[error("compaction failed: {0}")]
    CompactionFailed(String),

    /// The circuit breaker has tripped due to repeated failures.
    #[error("circuit breaker tripped after repeated compaction failures")]
    CircuitBreakerTripped,

    /// The LLM provider returned an error during summarization.
    #[error("provider error: {0}")]
    ProviderError(String),
}

/// Result alias for compaction operations.
pub type CompactionResult<T> = Result<T, CompactionError>;

/// Outcome of a compaction attempt, reported without exposing hidden tool output.
///
/// Status fields contain only counts and token estimates — never raw message
/// content or tool result text. This is the hidden-output guard (MEM-005-A).
#[derive(Debug, Clone, PartialEq)]
pub enum CompactionStatus {
    /// Compaction was applied successfully.
    Applied {
        /// Names of layers applied, in order (e.g., `["budget", "trim"]`).
        layers_applied: Vec<&'static str>,
        /// Estimated token count before compaction.
        tokens_before: u32,
        /// Estimated token count after compaction.
        tokens_after: u32,
    },
    /// Compaction was skipped (context already fits or below threshold).
    Skipped {
        /// Why compaction was skipped.
        reason: &'static str,
        /// Current estimated token count.
        tokens_current: u32,
    },
    /// Compaction failed.
    Failed {
        /// Error message (never includes tool result content).
        error: String,
    },
}
