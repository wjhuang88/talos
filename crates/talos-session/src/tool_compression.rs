//! Threshold-based tool output compression (ADR-037, Mechanism A).
//!
//! When a tool's output exceeds a configured byte threshold, the content is
//! summarized and the original is preserved in `raw_content` for later retrieval.
//! Compression is applied BEFORE the entry is written to the session log.

/// Result of applying threshold-based compression to a tool's output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolOutputCompression {
    /// The content visible to the model (summarized or original).
    pub model_content: String,
    /// The original unmodified content, present only when compression occurred.
    pub raw_content: Option<String>,
    /// `0` = no compression applied, `1` = compressed.
    pub raw_flag: u8,
}

/// Compress tool output based on a byte-length threshold.
///
/// When `content.len()` is at or below `threshold`, returns the content
/// unchanged with `raw_flag = 0`.
///
/// When `content.len()` exceeds `threshold`, returns a summary consisting of
/// the first `threshold` characters followed by a truncation notice, and stores
/// the original content in `raw_content` with `raw_flag = 1`.
pub fn compress_tool_output(content: &str, threshold: usize) -> ToolOutputCompression {
    if content.is_empty() || content.len() <= threshold {
        return ToolOutputCompression {
            model_content: content.to_string(),
            raw_content: None,
            raw_flag: 0,
        };
    }

    let total = content.len();
    let summary = format!(
        "{}\n... [truncated, {total} bytes total]",
        &content[..threshold]
    );

    ToolOutputCompression {
        model_content: summary,
        raw_content: Some(content.to_string()),
        raw_flag: 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn under_threshold_no_compression() {
        let result = compress_tool_output("hello world", 100);
        assert_eq!(result.model_content, "hello world");
        assert_eq!(result.raw_content, None);
        assert_eq!(result.raw_flag, 0);
    }

    #[test]
    fn over_threshold_summary_and_raw() {
        let content = "A".repeat(200);
        let result = compress_tool_output(&content, 50);

        assert_eq!(result.raw_flag, 1);
        assert_eq!(result.raw_content, Some(content.clone()));
        assert_eq!(
            result.model_content,
            format!("{}\n... [truncated, 200 bytes total]", "A".repeat(50))
        );
    }

    #[test]
    fn empty_content_no_compression() {
        let result = compress_tool_output("", 100);
        assert_eq!(result.model_content, "");
        assert_eq!(result.raw_content, None);
        assert_eq!(result.raw_flag, 0);
    }

    #[test]
    fn exactly_at_threshold_no_compression() {
        let content = "B".repeat(50);
        let result = compress_tool_output(&content, 50);
        assert_eq!(result.model_content, content);
        assert_eq!(result.raw_content, None);
        assert_eq!(result.raw_flag, 0);
    }
}
