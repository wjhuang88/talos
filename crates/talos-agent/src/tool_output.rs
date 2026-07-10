#![allow(dead_code)]
//! Tool output compression (ADR-037 Mechanism A).
//!
//! When a tool produces output exceeding a threshold, the content is split into
//! a model-facing summary and a raw full version. The model gets the summary;
//! the raw version is stored in SessionMetadata.raw_content for UI display.

pub struct ToolOutputCompression {
    pub model_content: String,
    pub raw_content: Option<String>,
    pub raw_flag: u8,
}

pub fn compress_tool_output(content: &str, threshold: usize) -> ToolOutputCompression {
    if content.len() <= threshold {
        return ToolOutputCompression {
            model_content: content.to_string(),
            raw_content: None,
            raw_flag: 0,
        };
    }

    let prefix_end = content
        .char_indices()
        .map(|(index, _)| index)
        .take_while(|index| *index <= threshold)
        .last()
        .unwrap_or(0);
    let summary = format!(
        "{}\n... [truncated, {} bytes total]",
        &content[..prefix_end],
        content.len()
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
        let result = compress_tool_output("short", 100);
        assert_eq!(result.model_content, "short");
        assert!(result.raw_content.is_none());
        assert_eq!(result.raw_flag, 0);
    }

    #[test]
    fn over_threshold_compressed() {
        let content = "x".repeat(200);
        let result = compress_tool_output(&content, 50);
        assert!(result.model_content.contains("[truncated"));
        assert_eq!(result.raw_content, Some(content));
        assert_eq!(result.raw_flag, 1);
    }

    #[test]
    fn empty_content() {
        let result = compress_tool_output("", 100);
        assert_eq!(result.model_content, "");
        assert!(result.raw_content.is_none());
    }

    #[test]
    fn exactly_at_threshold() {
        let content = "x".repeat(100);
        let result = compress_tool_output(&content, 100);
        assert_eq!(result.model_content, content);
        assert!(result.raw_content.is_none());
        assert_eq!(result.raw_flag, 0);
    }

    #[test]
    fn truncation_preserves_utf8_boundaries() {
        let result = compress_tool_output("你好世界", 5);
        assert_eq!(result.model_content, "你\n... [truncated, 12 bytes total]");
        assert_eq!(result.raw_content.as_deref(), Some("你好世界"));
    }
}
