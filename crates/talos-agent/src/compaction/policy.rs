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

/// Pressure of an entire provider request, including all reserved tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestPressure {
    /// Below eighty percent; manual compaction is not eligible.
    Normal,
    /// At least eighty percent; fitting requests continue without forced rewrite.
    Advisory,
    /// At least ninety-five percent and within capacity; checkpoint before dispatch.
    Critical,
    /// Above capacity (or no usable capacity); checkpoint or refuse dispatch.
    Emergency,
}

impl RequestPressure {
    /// Classify the existing admission estimate against the resolved total context.
    ///
    /// The caller must include output/reasoning reserves in `estimated_tokens`
    /// exactly as admission does. Do not subtract them from `context_limit` again.
    #[must_use]
    pub fn assess(estimated_tokens: u32, context_limit: u32) -> Self {
        if context_limit == 0 || estimated_tokens > context_limit {
            return Self::Emergency;
        }
        let estimated = u64::from(estimated_tokens) * 100;
        let capacity = u64::from(context_limit);
        if estimated >= capacity * 95 {
            Self::Critical
        } else if estimated >= capacity * 80 {
            Self::Advisory
        } else {
            Self::Normal
        }
    }
}

#[cfg(test)]
mod pressure_tests {
    use super::RequestPressure;

    #[test]
    fn exact_whole_request_thresholds() {
        for (tokens, expected) in [
            (0, RequestPressure::Normal),
            (79, RequestPressure::Normal),
            (80, RequestPressure::Advisory),
            (94, RequestPressure::Advisory),
            (95, RequestPressure::Critical),
            (100, RequestPressure::Critical),
            (101, RequestPressure::Emergency),
        ] {
            assert_eq!(RequestPressure::assess(tokens, 100), expected);
        }
    }

    #[test]
    fn no_capacity_fractional_cutoffs_and_large_limits() {
        assert_eq!(RequestPressure::assess(0, 0), RequestPressure::Emergency);
        assert_eq!(RequestPressure::assess(80, 101), RequestPressure::Normal);
        assert_eq!(RequestPressure::assess(95, 101), RequestPressure::Advisory);
        assert_eq!(
            RequestPressure::assess(u32::MAX, u32::MAX),
            RequestPressure::Critical
        );
    }
}
