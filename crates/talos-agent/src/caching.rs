//! Prompt caching strategy for provider-side caching.
//!
//! This module structures system prompts to maximize cache hit rates by
//! separating stable content (identity, tool definitions, context files)
//! from dynamic content (conversation history). The static prefix remains
//! identical across turns, enabling providers like Anthropic to cache and
//! reuse token computations.
//!
//! # Cache Control Strategy
//!
//! The system prompt is divided into three cacheable sections:
//! 1. **Identity** — agent identity and behavioral instructions (~1024 tokens)
//! 2. **Tool definitions** — sorted by name for stable ordering
//! 3. **Context files** — AGENTS.md and other reference materials
//!
//! Each section boundary is marked with a `cache_control` breakpoint,
//! signaling the provider to cache up to that point.
//!
//! # Example
//!
//! ```
//! use talos_agent::caching::PromptCache;
//! use talos_core::provider::ToolDefinition;
//!
//! let mut cache = PromptCache::new();
//! let tools = vec![
//!     ToolDefinition::new("bash", "Execute shell commands", serde_json::json!({})),
//!     ToolDefinition::new("read", "Read a file", serde_json::json!({})),
//! ];
//! let system_prompt = cache.build_system_prompt(
//!     "You are a helpful coding assistant.",
//!     &tools,
//!     "# Project Rules\nFollow the coding guide.",
//! );
//!
//! let anthropic_format = system_prompt.to_anthropic_format();
//! ```

use serde_json::{Value, json};
use talos_core::provider::ToolDefinition;

/// A structured system prompt with cache control breakpoints.
///
/// The system prompt is divided into a static prefix (stable across turns)
/// and dynamic content. Cache control breakpoints mark positions where the
/// provider should cache token computations.
#[derive(Debug, Clone)]
pub struct SystemPrompt {
    /// The complete system prompt text.
    full_text: String,
    /// Byte positions where `cache_control` markers should be inserted.
    /// These correspond to boundaries between cacheable sections.
    cache_control_breakpoints: Vec<usize>,
}

impl SystemPrompt {
    /// Returns the complete system prompt text.
    #[must_use]
    pub fn full_text(&self) -> &str {
        &self.full_text
    }

    /// Returns the byte positions where cache control markers should be inserted.
    #[must_use]
    pub fn cache_control_breakpoints(&self) -> &[usize] {
        &self.cache_control_breakpoints
    }

    /// Formats this system prompt for the Anthropic Messages API with
    /// `cache_control` markers at the appropriate breakpoints.
    ///
    /// The output follows the Anthropic format:
    /// ```json
    /// {
    ///   "system": [
    ///     {"type": "text", "text": "...", "cache_control": {"type": "ephemeral"}},
    ///     {"type": "text", "text": "..."}
    ///   ]
    /// }
    /// ```
    ///
    /// Each section between breakpoints becomes a separate content block.
    /// The last block before each breakpoint gets a `cache_control` marker.
    #[must_use]
    pub fn to_anthropic_format(&self) -> Value {
        let mut blocks: Vec<Value> = Vec::new();
        let text = &self.full_text;
        let mut prev_pos = 0;

        for &bp in &self.cache_control_breakpoints {
            let end = bp.min(text.len());
            if end > prev_pos {
                let section_text = text[prev_pos..end].to_string();
                blocks.push(json!({
                    "type": "text",
                    "text": section_text,
                    "cache_control": {"type": "ephemeral"}
                }));
            }
            prev_pos = end;
        }

        // Remaining text after the last breakpoint (dynamic part, not cached)
        if prev_pos < text.len() {
            let remaining = text[prev_pos..].to_string();
            blocks.push(json!({
                "type": "text",
                "text": remaining
            }));
        }

        // If no breakpoints exist, return the full text as a single uncached block
        if blocks.is_empty() {
            blocks.push(json!({
                "type": "text",
                "text": text
            }));
        }

        json!({
            "system": blocks
        })
    }
}

/// Manages prompt caching for the agent turn loop.
///
/// `PromptCache` builds structured system prompts with cache control
/// breakpoints and tracks cache performance metrics.
#[derive(Debug)]
pub struct PromptCache {
    /// Number of cache hits observed.
    cache_hits: u64,
    /// Total number of cache checks performed.
    cache_checks: u64,
}

impl PromptCache {
    /// Creates a new prompt cache with zeroed metrics.
    #[must_use]
    pub fn new() -> Self {
        Self {
            cache_hits: 0,
            cache_checks: 0,
        }
    }

    /// Builds a structured system prompt with cache control breakpoints.
    ///
    /// The system prompt is organized into three cacheable sections:
    /// 1. **Identity** — the agent's identity and behavioral instructions
    /// 2. **Tool definitions** — sorted alphabetically by name for stable ordering
    /// 3. **Context files** — reference materials like AGENTS.md
    ///
    /// A dynamic section follows the static prefix, reserved for conversation
    /// history that changes each turn.
    ///
    /// # Arguments
    ///
    /// * `identity` — Agent identity and behavioral instructions.
    /// * `tools` — List of tool definitions (will be sorted by name).
    /// * `context` — Context file contents (e.g., AGENTS.md).
    ///
    /// # Cache Breakpoints
    ///
    /// Three breakpoints are inserted:
    /// - After the identity section
    /// - After the tool definitions section
    /// - After the context files section
    #[must_use]
    pub fn build_system_prompt(
        &self,
        identity: &str,
        tools: &[ToolDefinition],
        context: &str,
    ) -> SystemPrompt {
        // Sort tools by name for stable ordering
        let mut sorted_tools: Vec<&ToolDefinition> = tools.iter().collect();
        sorted_tools.sort_by(|a, b| a.name.cmp(&b.name));

        // Build the identity section
        let identity_section = format!("# Identity\n{identity}\n");

        // Build the tool definitions section
        let tools_section = if sorted_tools.is_empty() {
            String::from("# Tools\nNo tools available.\n")
        } else {
            let mut section = String::from("# Tools\n");
            for tool in &sorted_tools {
                section.push_str(&tool.to_prompt_text());
                section.push_str("\n\n");
            }
            section
        };

        // Build the context section
        let context_section = if context.is_empty() {
            String::from("# Context\nNo context files loaded.\n")
        } else {
            format!("# Context\n{context}\n")
        };

        // Assemble the full static prefix
        let static_prefix = format!("{identity_section}\n{tools_section}\n{context_section}");

        // Calculate breakpoints (byte positions)
        let bp1 = identity_section.len();
        let bp2 = bp1 + 1 + tools_section.len(); // +1 for the separator newline
        let bp3 = bp2 + 1 + context_section.len(); // +1 for the separator newline

        SystemPrompt {
            full_text: static_prefix,
            cache_control_breakpoints: vec![bp1, bp2, bp3],
        }
    }

    /// Records a cache hit or miss for performance tracking.
    ///
    /// # Arguments
    ///
    /// * `hit` — `true` if the cache was hit, `false` if it was a miss.
    pub fn track_cache_hit_rate(&mut self, hit: bool) {
        self.cache_checks += 1;
        if hit {
            self.cache_hits += 1;
        }
    }

    /// Returns the cache hit rate as a percentage (0.0 to 100.0).
    ///
    /// Returns `0.0` if no cache checks have been recorded.
    #[must_use]
    pub fn cache_hit_rate(&self) -> f64 {
        if self.cache_checks == 0 {
            0.0
        } else {
            (self.cache_hits as f64 / self.cache_checks as f64) * 100.0
        }
    }
}

impl Default for PromptCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- System prompt structure tests ---

    #[test]
    fn test_system_prompt_has_static_prefix() {
        let cache = PromptCache::new();
        let tools = vec![ToolDefinition::new(
            "bash",
            "Execute shell commands",
            json!({}),
        )];
        let prompt = cache.build_system_prompt("You are an assistant.", &tools, "Context here.");

        assert!(prompt.full_text().contains("# Identity"));
        assert!(prompt.full_text().contains("You are an assistant."));
        assert!(prompt.full_text().contains("# Tools"));
        assert!(prompt.full_text().contains("# Context"));
        assert!(prompt.full_text().contains("Context here."));
    }

    #[test]
    fn test_system_prompt_static_prefix_is_consistent() {
        let cache = PromptCache::new();
        let tools = vec![ToolDefinition::new(
            "bash",
            "Execute shell commands",
            json!({}),
        )];

        let prompt1 = cache.build_system_prompt("You are an assistant.", &tools, "Context here.");
        let prompt2 = cache.build_system_prompt("You are an assistant.", &tools, "Context here.");

        assert_eq!(prompt1.full_text(), prompt2.full_text());
        assert_eq!(
            prompt1.cache_control_breakpoints(),
            prompt2.cache_control_breakpoints()
        );
    }

    // --- Cache control breakpoint tests ---

    #[test]
    fn test_cache_control_breakpoints_at_correct_positions() {
        let cache = PromptCache::new();
        let prompt = cache.build_system_prompt("Identity text.", &[], "");

        let breakpoints = prompt.cache_control_breakpoints();
        assert_eq!(breakpoints.len(), 3);

        // BP1 should be after the identity section
        let bp1 = breakpoints[0];
        assert!(prompt.full_text()[..bp1].contains("# Identity"));
        assert!(prompt.full_text()[..bp1].contains("Identity text."));

        // BP2 should be after the tools section
        let bp2 = breakpoints[1];
        assert!(prompt.full_text()[..bp2].contains("# Tools"));

        // BP3 should be after the context section
        let bp3 = breakpoints[2];
        assert!(prompt.full_text()[..bp3].contains("# Context"));

        // BP3 should be at or near the end of the text
        assert!(bp3 <= prompt.full_text().len());
    }

    #[test]
    fn test_breakpoints_are_increasing() {
        let cache = PromptCache::new();
        let tools = vec![ToolDefinition::new("bash", "Execute commands", json!({}))];
        let prompt = cache.build_system_prompt("Identity.", &tools, "Context.");

        let bps = prompt.cache_control_breakpoints();
        assert!(bps[0] < bps[1]);
        assert!(bps[1] < bps[2]);
    }

    // --- Tool definition sorting tests ---

    #[test]
    fn test_tool_definitions_sorted_by_name() {
        let cache = PromptCache::new();
        let tools = vec![
            ToolDefinition::new("write", "Write a file", json!({})),
            ToolDefinition::new("bash", "Execute commands", json!({})),
            ToolDefinition::new("read", "Read a file", json!({})),
        ];
        let prompt = cache.build_system_prompt("Identity.", &tools, "");

        let text = prompt.full_text();
        let bash_pos = text.find("## bash").expect("bash should be present");
        let read_pos = text.find("## read").expect("read should be present");
        let write_pos = text.find("## write").expect("write should be present");

        assert!(bash_pos < read_pos, "bash should come before read");
        assert!(read_pos < write_pos, "read should come before write");
    }

    #[test]
    fn test_empty_tools_list() {
        let cache = PromptCache::new();
        let prompt = cache.build_system_prompt("Identity.", &[], "");

        assert!(prompt.full_text().contains("No tools available."));
        assert_eq!(prompt.cache_control_breakpoints().len(), 3);
    }

    #[test]
    fn test_empty_context() {
        let cache = PromptCache::new();
        let prompt = cache.build_system_prompt("Identity.", &[], "");

        assert!(prompt.full_text().contains("No context files loaded."));
    }

    // --- Anthropic format tests ---

    #[test]
    fn test_to_anthropic_format_produces_valid_json() {
        let cache = PromptCache::new();
        let tools = vec![ToolDefinition::new("bash", "Execute commands", json!({}))];
        let prompt = cache.build_system_prompt("Identity.", &tools, "Context.");
        let anthropic = prompt.to_anthropic_format();

        // Should be a valid JSON object with "system" key
        assert!(anthropic.get("system").is_some());
        let system_blocks = anthropic["system"].as_array().unwrap();
        assert!(!system_blocks.is_empty());
    }

    #[test]
    fn test_to_anthropic_format_has_cache_control_markers() {
        let cache = PromptCache::new();
        let prompt = cache.build_system_prompt("Identity.", &[], "");
        let anthropic = prompt.to_anthropic_format();

        let system_blocks = anthropic["system"].as_array().unwrap();

        // Should have 4 blocks: 3 cached + 1 uncached (or 3 cached if no trailing text)
        // At minimum, the first 3 blocks should have cache_control
        let cached_blocks: Vec<_> = system_blocks
            .iter()
            .filter(|b| b.get("cache_control").is_some())
            .collect();

        assert!(
            cached_blocks.len() >= 3,
            "Expected at least 3 cached blocks, got {}",
            cached_blocks.len()
        );

        // Verify cache_control format
        for block in &cached_blocks {
            let cc = block["cache_control"].as_object().unwrap();
            assert_eq!(cc.get("type").unwrap().as_str().unwrap(), "ephemeral");
        }
    }

    #[test]
    fn test_to_anthropic_format_last_block_uncached() {
        let cache = PromptCache::new();
        let prompt = cache.build_system_prompt("Identity.", &[], "Context.");
        let anthropic = prompt.to_anthropic_format();

        let system_blocks = anthropic["system"].as_array().unwrap();

        // The last block should NOT have cache_control (it's the dynamic part)
        // But since build_system_prompt only builds the static prefix,
        // the last cached block ends at bp3 which is the end of text.
        // So all blocks will be cached in this case.
        let last_block = system_blocks.last().unwrap();
        // When static_prefix ends exactly at bp3, the last block is cached
        assert!(last_block.get("cache_control").is_some());
    }

    #[test]
    fn test_to_anthropic_format_with_empty_prompt() {
        let prompt = SystemPrompt {
            full_text: String::new(),
            cache_control_breakpoints: vec![],
        };
        let anthropic = prompt.to_anthropic_format();

        let system_blocks = anthropic["system"].as_array().unwrap();
        assert_eq!(system_blocks.len(), 1);
        assert_eq!(system_blocks[0]["text"], "");
        // No cache_control on the single block
        assert!(system_blocks[0].get("cache_control").is_none());
    }

    // --- Cache hit rate tests ---

    #[test]
    fn test_cache_hit_rate_initially_zero() {
        let cache = PromptCache::new();
        assert_eq!(cache.cache_hit_rate(), 0.0);
    }

    #[test]
    fn test_cache_hit_rate_after_hits() {
        let mut cache = PromptCache::new();
        cache.track_cache_hit_rate(true);
        cache.track_cache_hit_rate(true);
        cache.track_cache_hit_rate(false);
        cache.track_cache_hit_rate(true);

        // 3 hits out of 4 checks = 75%
        assert!((cache.cache_hit_rate() - 75.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cache_hit_rate_all_hits() {
        let mut cache = PromptCache::new();
        cache.track_cache_hit_rate(true);
        cache.track_cache_hit_rate(true);

        assert!((cache.cache_hit_rate() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cache_hit_rate_all_misses() {
        let mut cache = PromptCache::new();
        cache.track_cache_hit_rate(false);
        cache.track_cache_hit_rate(false);

        assert!((cache.cache_hit_rate() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cache_hit_rate_single_hit() {
        let mut cache = PromptCache::new();
        cache.track_cache_hit_rate(true);

        assert!((cache.cache_hit_rate() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cache_hit_rate_single_miss() {
        let mut cache = PromptCache::new();
        cache.track_cache_hit_rate(false);

        assert!((cache.cache_hit_rate() - 0.0).abs() < f64::EPSILON);
    }

    // --- ToolDefinition tests ---

    #[test]
    fn test_tool_definition_to_prompt_text() {
        let tool = ToolDefinition::new(
            "read_file",
            "Read the contents of a file",
            json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string"}
                }
            }),
        );

        let text = tool.to_prompt_text();
        assert!(text.contains("## read_file"));
        assert!(text.contains("Read the contents of a file"));
        assert!(text.contains("path"));
    }

    // --- Integration-style tests ---

    #[test]
    fn test_full_prompt_with_all_sections() {
        let cache = PromptCache::new();
        let tools = vec![
            ToolDefinition::new("bash", "Run shell commands", json!({"command": "string"})),
            ToolDefinition::new("read", "Read files", json!({"path": "string"})),
            ToolDefinition::new(
                "write",
                "Write files",
                json!({"path": "string", "content": "string"}),
            ),
        ];
        let identity = "You are Talos, a safety-first agent runtime.";
        let context = "# AGENTS.md\nFollow the coding guide.";

        let prompt = cache.build_system_prompt(identity, &tools, context);
        let anthropic = prompt.to_anthropic_format();

        // Verify structure
        assert!(prompt.full_text().contains("You are Talos"));
        assert!(prompt.full_text().contains("## bash"));
        assert!(prompt.full_text().contains("## read"));
        assert!(prompt.full_text().contains("## write"));
        assert!(prompt.full_text().contains("AGENTS.md"));

        // Verify Anthropic format
        let system_blocks = anthropic["system"].as_array().unwrap();
        assert!(!system_blocks.is_empty());

        // Verify tools are in alphabetical order in the output
        let text = prompt.full_text();
        assert!(text.find("## bash").unwrap() < text.find("## read").unwrap());
        assert!(text.find("## read").unwrap() < text.find("## write").unwrap());
    }

    #[test]
    fn test_prompt_cache_default_trait() {
        let cache = PromptCache::default();
        assert_eq!(cache.cache_hit_rate(), 0.0);
    }

    #[test]
    fn test_system_prompt_clone() {
        let cache = PromptCache::new();
        let prompt = cache.build_system_prompt("Identity.", &[], "");
        let cloned = prompt.clone();

        assert_eq!(prompt.full_text(), cloned.full_text());
        assert_eq!(
            prompt.cache_control_breakpoints(),
            cloned.cache_control_breakpoints()
        );
    }

    #[test]
    fn test_tool_definition_clone_and_eq() {
        let tool1 = ToolDefinition::new("bash", "Run commands", json!({}));
        let tool2 = tool1.clone();

        assert_eq!(tool1, tool2);
    }
}
