use super::constants::{
    CIRCUIT_BREAKER_THRESHOLD, COLLAPSE_TURN_THRESHOLD, MAX_TOOL_RESULT_CHARS, PRESERVED_TURNS,
    TRIM_TURN_THRESHOLD,
};

/// Configurable compaction policy documenting threshold math and limit sources.
///
/// All thresholds have documented defaults and source precedence. This struct
/// makes the compaction decision boundary explicit and testable (MEM-005-A).
#[derive(Debug, Clone)]
pub struct CompactionPolicy {
    /// Fraction of model_limit that triggers compaction (default: 0.8).
    pub trigger_threshold: f32,
    /// Maximum characters per tool result before budget truncation (default: 4000).
    pub max_tool_result_chars: usize,
    /// Number of recent turns preserved verbatim, never compacted by layers 4/5 (default: 10).
    pub preserved_turns: usize,
    /// Turn threshold for trim layer — tool results older than this are emptied (default: 20).
    pub trim_turn_threshold: usize,
    /// Turn threshold for collapse layer — turns older than this are summarized (default: 10).
    pub collapse_turn_threshold: usize,
    /// Maximum consecutive failures before circuit breaker trips (default: 3).
    pub circuit_breaker_threshold: usize,
    /// Tokens reserved for model output, reducing effective context budget (placeholder, default: 0).
    pub output_reserve: u32,
}

impl Default for CompactionPolicy {
    fn default() -> Self {
        Self {
            trigger_threshold: 0.8,
            max_tool_result_chars: MAX_TOOL_RESULT_CHARS,
            preserved_turns: PRESERVED_TURNS,
            trim_turn_threshold: TRIM_TURN_THRESHOLD,
            collapse_turn_threshold: COLLAPSE_TURN_THRESHOLD,
            circuit_breaker_threshold: CIRCUIT_BREAKER_THRESHOLD,
            output_reserve: 0,
        }
    }
}

impl CompactionPolicy {
    /// Returns the token threshold at which compaction triggers.
    ///
    /// Source precedence: `model_limit * trigger_threshold - output_reserve`.
    #[must_use]
    pub fn trigger_tokens(&self, model_limit: u32) -> u32 {
        let raw = (model_limit as f32 * self.trigger_threshold) as u32;
        raw.saturating_sub(self.output_reserve)
    }
}
