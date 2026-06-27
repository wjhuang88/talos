use crate::MemoryStore;

// ---------------------------------------------------------------------------
// Memory prompt injection — bounded, disable-able, safety-filtered.
// ---------------------------------------------------------------------------

/// Configuration for memory prompt injection.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default)]
pub struct MemoryPromptConfig {
    /// Whether memory injection is enabled.
    pub enabled: bool,
    /// Maximum number of memory items to include.
    pub max_items: usize,
    /// Maximum character budget for the formatted section.
    pub max_chars: usize,
}

impl Default for MemoryPromptConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_items: 5,
            max_chars: 2000,
        }
    }
}

/// Patterns that indicate content originated from hidden tool/system output.
/// If any of these appear in a memory item's content, the item is filtered
/// out as a defense-in-depth measure.
const HIDDEN_OUTPUT_PATTERNS: &[&str] = &[
    // Existing patterns (do not remove).
    "<tool_result>",
    "</tool_result>",
    "Tool output:",
    "is_error:",
    "tool_call",
    "tool_result",
    // JSON-style markers.
    "\"type\": \"tool_result\"",
    "\"type\":\"tool_result\"",
    "\"role\": \"tool\"",
    "\"role\":\"tool\"",
    // Anthropic-style markers.
    "tool_use",
    "tool_use_id",
    "function_call",
    // Whitespace-padded tag variants (caught after normalization, but listed
    // here for documentation and direct matching on un-normalized content).
    "< tool_result",
    "tool_result >",
    "<tool_result ",
    // System markers.
    "<system>",
    "</system>",
    "<system-reminder",
    "system-reminder",
];

/// Returns `true` if `content` appears to contain hidden tool or system output.
///
/// Applies a normalization pass before pattern matching: trims leading/trailing
/// whitespace and collapses runs of multiple spaces into a single space. This
/// catches bypass attempts like `<tool_result  >` or `<  tool_result  >`.
pub(crate) fn is_hidden_output(content: &str) -> bool {
    let normalized = collapse_whitespace(content.trim());
    let lower = normalized.to_lowercase();
    HIDDEN_OUTPUT_PATTERNS.iter().any(|pat| lower.contains(pat))
}

fn collapse_whitespace(s: &str) -> std::borrow::Cow<'_, str> {
    if !s.contains("  ") {
        return std::borrow::Cow::Borrowed(s);
    }
    std::borrow::Cow::Owned(s.replace("  ", " "))
}

/// Format retrieved memory into a bounded prompt section.
///
/// Returns `None` if: disabled, no query, no results, or all results are filtered.
/// The output includes provenance (source session/turn), confidence, freshness,
/// and contradiction markers. Hidden tool output is filtered out.
pub fn format_memory_prompt(
    store: &MemoryStore,
    query: &str,
    config: &MemoryPromptConfig,
) -> Option<String> {
    if !config.enabled || query.trim().is_empty() {
        return None;
    }

    let results = store.retrieve(query, config.max_items).ok()?;
    if results.is_empty() {
        return None;
    }

    let mut items = Vec::new();
    let header = "## Relevant Memory\n";
    let mut total_len = header.len();

    for result in &results {
        // Defense-in-depth: skip items that look like hidden tool output.
        if is_hidden_output(&result.item.content) {
            continue;
        }

        let source_ref = result
            .evidence
            .first()
            .map(|e| e.source_ref.as_str())
            .unwrap_or("unknown");

        let reinforced = result.item.last_reinforced.format("%Y-%m-%d");

        let line = if result.item.contradiction_ref.is_some() {
            format!(
                "- ⚠ CONTRADICTION: [confidence={:.1}] {} (source: {}, reinforced: {})\n",
                result.item.confidence, result.item.content, source_ref, reinforced,
            )
        } else {
            format!(
                "- [confidence={:.1}] {} (source: {}, reinforced: {})\n",
                result.item.confidence, result.item.content, source_ref, reinforced,
            )
        };

        // Check if adding this line would exceed the budget.
        let line_len = line.len();
        if total_len + line_len > config.max_chars {
            // Truncate: append the truncation notice and stop.
            let truncation_notice = "... (memory section truncated)";
            // Ensure we have room for the notice.
            if total_len + truncation_notice.len() <= config.max_chars {
                items.push(truncation_notice.to_string());
            }
            break;
        }

        items.push(line);
        total_len += line_len;
    }

    if items.is_empty() {
        // All results were filtered out.
        return None;
    }

    let mut output = String::with_capacity(config.max_chars);
    output.push_str(header);
    for item in &items {
        output.push_str(item);
    }

    Some(output)
}
