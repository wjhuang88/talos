/// Maximum characters allowed per tool result after budget compaction.
pub(crate) const MAX_TOOL_RESULT_CHARS: usize = 4000;

/// Truncation suffix appended when a tool result is truncated.
pub(crate) const TRUNCATION_SUFFIX: &str = "... [truncated]";

/// Number of recent turns to preserve verbatim (never compacted by layers 4/5).
pub(crate) const PRESERVED_TURNS: usize = 10;

/// Turn threshold for trim layer — tool results older than this are removed.
pub(crate) const TRIM_TURN_THRESHOLD: usize = 20;

/// Turn threshold for collapse layer — turns older than this are summarized.
pub(crate) const COLLAPSE_TURN_THRESHOLD: usize = 10;

/// Maximum consecutive compaction failures before the circuit breaker trips.
pub(crate) const CIRCUIT_BREAKER_THRESHOLD: usize = 3;
