//! Deterministic output compression for tool results entering model context.
//!
//! This module implements MEM-007 active context compression, starting with
//! `bash` tool output. Compression applies only to the model-facing
//! representation; raw output is preserved in the session JSONL log.
//!
//! # Design Constraints
//!
//! - **Deterministic**: same input always produces same compressed bytes.
//!   No timestamps, no random state, no session-dependent ordering.
//! - **Default OFF**: compression must be explicitly enabled via
//!   [`Agent::with_bash_compression`].
//! - **Stable prefix safe**: compression applies only to tool results entering
//!   the dynamic suffix, never to cached-prefix messages.
//! - **bash only**: other tools are unaffected by this module.

/// Default line threshold for bash output compression.
///
/// When bash output exceeds this many lines, it is compressed to the last N
/// lines plus a truncation marker.
pub const DEFAULT_BASH_LINE_THRESHOLD: usize = 30;

/// Truncation marker prepended to compressed bash output.
///
/// The `{omitted}` placeholder is replaced with the count of omitted lines.
const TRUNCATION_MARKER_TEMPLATE: &str =
    "\n... (first {omitted} lines omitted, see /export for full output)\n";

/// Result of compressing bash tool output.
#[derive(Debug, Clone)]
pub struct CompressedOutput {
    /// The model-facing content (compressed or original).
    pub content: String,
    /// Number of characters in the original input.
    pub original_size: usize,
    /// Number of characters in the output content.
    pub compressed_size: usize,
    /// The compression strategy applied.
    pub strategy: &'static str,
}

/// Deterministic compressor for bash tool output.
///
/// When enabled and bash output exceeds the configured line threshold, the
/// model-facing content is compressed to the last N lines plus a truncation
/// marker. Full output is preserved in the session JSONL log.
#[derive(Debug, Clone, Copy)]
pub struct BashOutputCompressor {
    /// Maximum number of lines to retain from the end of the output.
    line_threshold: usize,
}

impl BashOutputCompressor {
    /// Creates a new compressor with the default line threshold (30 lines).
    #[must_use]
    pub fn new() -> Self {
        Self {
            line_threshold: DEFAULT_BASH_LINE_THRESHOLD,
        }
    }

    /// Creates a new compressor with a custom line threshold.
    #[must_use]
    pub fn with_threshold(line_threshold: usize) -> Self {
        Self { line_threshold }
    }

    /// Compresses bash output if it exceeds the line threshold.
    ///
    /// When the number of lines is at or below the threshold, the content is
    /// returned unchanged with `strategy = "none"`.
    ///
    /// When the number of lines exceeds the threshold, the output is compressed
    /// to the last N lines with a truncation marker, and `strategy = "last_n_lines"`.
    ///
    /// # Determinism
    ///
    /// This method is fully deterministic: the same input string always produces
    /// the same output bytes. No timestamps, random state, or external context
    /// is used.
    #[must_use]
    pub fn compress(&self, content: &str) -> CompressedOutput {
        let original_size = content.len();
        let lines: Vec<&str> = content.lines().collect();
        let line_count = lines.len();

        if line_count <= self.line_threshold {
            return CompressedOutput {
                content: content.to_string(),
                original_size,
                compressed_size: original_size,
                strategy: "none",
            };
        }

        let omitted = line_count - self.line_threshold;
        let retained = &lines[omitted..];

        let marker = TRUNCATION_MARKER_TEMPLATE.replace("{omitted}", &omitted.to_string());
        let mut compressed = String::with_capacity(
            marker.len() + retained.iter().map(|l| l.len() + 1).sum::<usize>(),
        );
        compressed.push_str(&marker);
        for (i, line) in retained.iter().enumerate() {
            if i > 0 {
                compressed.push('\n');
            }
            compressed.push_str(line);
        }
        // Preserve trailing newline if the original had one
        if content.ends_with('\n') && !compressed.ends_with('\n') {
            compressed.push('\n');
        }

        let compressed_size = compressed.len();
        CompressedOutput {
            content: compressed,
            original_size,
            compressed_size,
            strategy: "last_n_lines",
        }
    }
}

impl Default for BashOutputCompressor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct CompressionMetrics {
    pub compression_events: u64,
    pub bytes_before: u64,
    pub bytes_after: u64,
}

impl CompressionMetrics {
    pub fn record(&mut self, output: &CompressedOutput) {
        if output.strategy == "none" {
            return;
        }
        self.compression_events += 1;
        self.bytes_before += output.original_size as u64;
        self.bytes_after += output.compressed_size as u64;
    }

    pub fn bytes_saved(&self) -> u64 {
        self.bytes_before.saturating_sub(self.bytes_after)
    }

    pub fn estimated_tokens_saved(&self) -> u64 {
        self.bytes_saved() / 4
    }
}

#[derive(Debug, Clone, Default)]
pub struct RetrievalMetrics {
    pub recall_calls: u64,
    pub results_returned: u64,
}

impl RetrievalMetrics {
    pub fn record_recall(&mut self, result_count: usize) {
        self.recall_calls += 1;
        self.results_returned += result_count as u64;
    }

    pub fn avg_results_per_call(&self) -> f64 {
        if self.recall_calls == 0 {
            0.0
        } else {
            self.results_returned as f64 / self.recall_calls as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_lines(n: usize) -> String {
        (0..n)
            .map(|i| format!("line {}", i))
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn short_output_no_compression() {
        let compressor = BashOutputCompressor::new();
        let content = make_lines(10);
        let result = compressor.compress(&content);

        assert_eq!(result.strategy, "none");
        assert_eq!(result.content, content);
        assert_eq!(result.original_size, result.compressed_size);
    }

    #[test]
    fn exactly_threshold_no_compression() {
        let compressor = BashOutputCompressor::new();
        let content = make_lines(DEFAULT_BASH_LINE_THRESHOLD);
        let result = compressor.compress(&content);

        assert_eq!(result.strategy, "none");
        assert_eq!(result.content, content);
    }

    #[test]
    fn one_over_threshold_compressed() {
        let compressor = BashOutputCompressor::new();
        let content = make_lines(DEFAULT_BASH_LINE_THRESHOLD + 1);
        let result = compressor.compress(&content);

        assert_eq!(result.strategy, "last_n_lines");
        assert!(result.content.contains("first 1 lines omitted"));
        // Should contain last 30 lines
        assert!(result.content.contains("line 1")); // line index 1 = second line (first retained)
    }

    #[test]
    fn long_output_compressed_to_last_n() {
        let compressor = BashOutputCompressor::new();
        let content = make_lines(100);
        let result = compressor.compress(&content);

        assert_eq!(result.strategy, "last_n_lines");
        assert!(result.content.contains("first 70 lines omitted"));
        // Last retained line should be "line 99"
        assert!(result.content.contains("line 99"));
        // First retained line should be "line 70"
        assert!(result.content.contains("line 70"));
        // First omitted line "line 0" should NOT appear
        assert!(!result.content.contains("line 0"));
        assert!(!result.content.contains("line 69"));
    }

    #[test]
    fn determinism() {
        let compressor = BashOutputCompressor::new();
        let content = make_lines(100);

        let result1 = compressor.compress(&content);
        let result2 = compressor.compress(&content);

        assert_eq!(result1.content, result2.content);
        assert_eq!(result1.original_size, result2.original_size);
        assert_eq!(result1.compressed_size, result2.compressed_size);
        assert_eq!(result1.strategy, result2.strategy);
    }

    #[test]
    fn trailing_newline_preserved() {
        let compressor = BashOutputCompressor::new();
        let content = format!("{}\n", make_lines(100));
        let result = compressor.compress(&content);

        assert!(result.content.ends_with('\n'));
    }

    #[test]
    fn no_trailing_newline_preserved() {
        let compressor = BashOutputCompressor::new();
        let content = make_lines(100);
        let result = compressor.compress(&content);

        assert!(!result.content.ends_with('\n'));
    }

    #[test]
    fn empty_input_no_compression() {
        let compressor = BashOutputCompressor::new();
        let result = compressor.compress("");

        assert_eq!(result.strategy, "none");
        assert_eq!(result.content, "");
    }

    #[test]
    fn custom_threshold() {
        let compressor = BashOutputCompressor::with_threshold(5);
        let content = make_lines(10);
        let result = compressor.compress(&content);

        assert_eq!(result.strategy, "last_n_lines");
        assert!(result.content.contains("first 5 lines omitted"));
        // Should retain last 5 lines: line 5 through line 9
        assert!(result.content.contains("line 5"));
        assert!(result.content.contains("line 9"));
        assert!(!result.content.contains("line 4"));
    }

    #[test]
    fn metadata_accuracy() {
        let compressor = BashOutputCompressor::new();
        let content = make_lines(100);
        let result = compressor.compress(&content);

        assert_eq!(result.original_size, content.len());
        assert_eq!(result.compressed_size, result.content.len());
    }

    #[test]
    fn metrics_accumulate_across_events() {
        let compressor = BashOutputCompressor::new();
        let mut metrics = CompressionMetrics::default();

        let r1 = compressor.compress(&make_lines(50));
        let r2 = compressor.compress(&make_lines(100));
        let r3 = compressor.compress(&make_lines(10));

        metrics.record(&r1);
        metrics.record(&r2);
        metrics.record(&r3);

        assert_eq!(metrics.compression_events, 2);
        assert!(metrics.bytes_saved() > 0);
        assert!(metrics.bytes_before > metrics.bytes_after);
    }

    #[test]
    fn metrics_skip_none_strategy() {
        let compressor = BashOutputCompressor::new();
        let mut metrics = CompressionMetrics::default();

        let r = compressor.compress(&make_lines(10));
        metrics.record(&r);

        assert_eq!(metrics.compression_events, 0);
        assert_eq!(metrics.bytes_saved(), 0);
    }

    #[test]
    fn metrics_estimated_tokens_saved() {
        let compressor = BashOutputCompressor::new();
        let mut metrics = CompressionMetrics::default();

        metrics.record(&compressor.compress(&make_lines(100)));

        assert!(metrics.estimated_tokens_saved() > 0);
    }

    #[test]
    fn retrieval_metrics_track_calls_and_results() {
        let mut metrics = RetrievalMetrics::default();

        metrics.record_recall(5);
        metrics.record_recall(3);
        metrics.record_recall(0);

        assert_eq!(metrics.recall_calls, 3);
        assert_eq!(metrics.results_returned, 8);
        assert!((metrics.avg_results_per_call() - 2.666).abs() < 0.1);
    }

    #[test]
    fn retrieval_metrics_empty_has_zero_avg() {
        let metrics = RetrievalMetrics::default();
        assert_eq!(metrics.avg_results_per_call(), 0.0);
    }
}
