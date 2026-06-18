//! System prompt assembly for the Talos agent.
//!
//! This module provides [`SystemPromptBuilder`] for constructing system prompts
//! from multiple components: identity, tool descriptions, skill index, context
//! files, and user preferences. The prompt is structured for optimal caching
//! by LLM providers, with stable sections placed first.
//!
//! # Prompt Ordering
//!
//! Components are assembled in this order for optimal cache hits:
//! 1. **Identity** — stable, cacheable
//! 2. **Tools** — stable, cacheable
//! 3. **Skill index** — stable, cacheable
//! 4. **Context files** — semi-stable
//! 5. **User preferences** — semi-stable
//! 6. **Custom prompt** — replaces identity if provided
//! 7. **Append prompt** — added at end if provided
//!
//! # Cache Markers
//!
//! [`SystemPromptBuilder::build_with_cache_markers`] returns cache control
//! markers indicating which byte ranges are stable and suitable for provider
//! caching (e.g., Anthropic's `cache_control: { type: "ephemeral" }`).
//!
//! # Example
//!
//! ```
//! use talos_agent::prompt::{SystemPromptBuilder, ToolDescription, ContextFile};
//! use talos_skill::SkillIndex;
//!
//! let prompt = SystemPromptBuilder::new()
//!     .with_tools(vec![ToolDescription {
//!         name: "read".into(),
//!         description: "Read a file".into(),
//!         ..Default::default()
//!     }])
//!     .with_skill_index(vec![SkillIndex {
//!         name: "git-skill".into(),
//!         description: "Git operations".into(),
//!         triggers: vec!["git".into()],
//!         estimated_tokens: 0,
//!     }])
//!     .with_context_files(vec![ContextFile {
//!         path: "AGENTS.md".into(),
//!         content: "# Project Rules\nBe helpful.".into(),
//!     }])
//!     .build();
//!
//! assert!(prompt.contains("# Identity"));
//! assert!(prompt.contains("# Tools"));
//! assert!(prompt.contains("# Skills"));
//! ```

use std::collections::HashMap;

use talos_core::message::{SystemCacheMarker, SystemCacheType};
use talos_plugin::{HookContext, HookEvent, HookOutcome, HookRegistry};
use talos_skill::SkillIndex;

/// Default identity text for the Talos agent.
pub const DEFAULT_IDENTITY: &str = include_str!("../../../prompts/identity.txt");

pub const TOOL_CALLING_FORMAT: &str = include_str!("../../../prompts/tool_calling_format.txt");
pub const TOOL_CALLING_STRICT: &str = include_str!("../../../prompts/tool_calling_strict.txt");

/// A description of a tool for inclusion in the system prompt.
///
/// Contains only the name and human-readable description, without the full
/// JSON Schema parameters. This is sufficient for the system prompt context.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ToolDescription {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// A context file for inclusion in the system prompt.
///
/// Typically loaded from `AGENTS.md` files in the workspace hierarchy.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextFile {
    /// Relative or absolute path to the source file.
    pub path: String,
    /// Full content of the file.
    pub content: String,
}

/// Type of cache control marker for prompt sections.
///
/// Used to indicate which sections of the system prompt are stable across
/// turns and suitable for provider-side caching.
#[derive(Debug, Clone, PartialEq)]
pub enum CacheType {
    /// The section is stable and suitable for ephemeral caching.
    /// Content in this range should be cached by the provider and reused
    /// across turns when the section remains unchanged.
    Ephemeral,
}

/// A cache marker indicating a byte range suitable for provider caching.
///
/// Each marker specifies the offset and length of a cacheable section within
/// the assembled prompt, along with the cache type.
#[derive(Debug, Clone, PartialEq)]
pub struct CacheMarker {
    /// Starting byte offset of the cacheable section.
    pub offset: usize,
    /// Length of the cacheable section in bytes.
    pub length: usize,
    /// Type of caching to apply to this section.
    pub cache_type: CacheType,
}

impl From<CacheMarker> for SystemCacheMarker {
    fn from(marker: CacheMarker) -> Self {
        let cache_type = match marker.cache_type {
            CacheType::Ephemeral => SystemCacheType::Ephemeral,
        };
        Self {
            offset: marker.offset,
            length: marker.length,
            cache_type,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PromptSectionKind {
    Cacheable,
    Dynamic,
}

#[derive(Debug, Clone, PartialEq)]
struct PromptSection {
    text: String,
    kind: PromptSectionKind,
}

/// Builder for assembling a system prompt from multiple components.
///
/// The builder uses a fluent API to configure each component of the system
/// prompt. Components are assembled in a fixed order optimized for LLM
/// provider caching: stable sections first, then semi-stable, then dynamic.
///
/// # Component Order
///
/// 1. Identity (or custom prompt if provided)
/// 2. Tool descriptions
/// 3. Skill index (Level 0)
/// 4. Context files (AGENTS.md)
/// 5. User preferences
/// 6. Append prompt (if provided)
///
/// # Example
///
/// ```
/// use talos_agent::prompt::SystemPromptBuilder;
///
/// let prompt = SystemPromptBuilder::new()
///     .with_user_preferences("Always use British English.".into())
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct SystemPromptBuilder {
    /// Agent identity and role instructions.
    identity: String,
    /// Tool names and descriptions.
    tools: Vec<ToolDescription>,
    /// Level 0 skill index (name + description).
    skill_index: Vec<SkillIndex>,
    /// AGENTS.md and other context file contents.
    context_files: Vec<ContextFile>,
    /// User-specific instructions.
    user_preferences: String,
    /// Overrides the default identity when provided.
    custom_prompt: Option<String>,
    /// Appended to the end of the prompt when provided.
    append_prompt: Option<String>,
    tool_call_format: &'static str,
    /// Runtime template values used for `{{slot}}` substitution.
    template_vars: HashMap<String, String>,
}

impl SystemPromptBuilder {
    /// Creates a new builder with the default identity and no other components.
    ///
    /// All optional components start empty. Use the builder methods to
    /// configure tools, skills, context files, and other components.
    #[must_use]
    pub fn new() -> Self {
        let mut template_vars = HashMap::new();
        template_vars.insert(
            "workspace_info".to_string(),
            "Workspace information unavailable.".to_string(),
        );
        template_vars.insert(
            "model_info".to_string(),
            "Provider model metadata unavailable.".to_string(),
        );

        Self {
            identity: DEFAULT_IDENTITY.to_string(),
            tools: Vec::new(),
            skill_index: Vec::new(),
            context_files: Vec::new(),
            user_preferences: String::new(),
            custom_prompt: None,
            append_prompt: None,
            tool_call_format: "",
            template_vars,
        }
    }

    pub fn with_strict_tool_format(mut self) -> Self {
        self.tool_call_format = TOOL_CALLING_STRICT;
        self
    }

    pub fn with_tool_format(mut self, format: &'static str) -> Self {
        self.tool_call_format = format;
        self
    }

    /// Sets a template slot value for `{{slot}}` substitution.
    #[must_use]
    pub fn with_template_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.template_vars.insert(key.into(), value.into());
        self
    }

    /// Sets workspace information for the default identity template.
    #[must_use]
    pub fn with_workspace_info(self, value: impl Into<String>) -> Self {
        self.with_template_var("workspace_info", value)
    }

    /// Sets model information for the default identity template.
    #[must_use]
    pub fn with_model_info(self, value: impl Into<String>) -> Self {
        self.with_template_var("model_info", value)
    }

    /// Sets the tool descriptions for inclusion in the system prompt.
    ///
    /// Tools are sorted alphabetically by name to ensure stable ordering
    /// across turns, maximizing cache hit rates.
    #[must_use]
    pub fn with_tools(mut self, tools: Vec<ToolDescription>) -> Self {
        self.tools = tools;
        self
    }

    /// Sets the skill index for inclusion in the system prompt.
    ///
    /// Only Level 0 metadata (name, description, triggers) is included.
    /// Full skill bodies are not loaded at this stage.
    #[must_use]
    pub fn with_skill_index(mut self, skills: Vec<SkillIndex>) -> Self {
        self.skill_index = skills;
        self
    }

    /// Sets the context files for inclusion in the system prompt.
    ///
    /// Typically loaded from `AGENTS.md` files via [`ContextLoader`].
    ///
    /// [`ContextLoader`]: crate::context::ContextLoader
    #[must_use]
    pub fn with_context_files(mut self, files: Vec<ContextFile>) -> Self {
        self.context_files = files;
        self
    }

    /// Sets user-specific instructions for inclusion in the system prompt.
    #[must_use]
    pub fn with_user_preferences(mut self, prefs: String) -> Self {
        self.user_preferences = prefs;
        self
    }

    /// Sets a custom prompt that replaces the default identity.
    ///
    /// When provided, the custom prompt is used instead of the default
    /// identity. The rest of the prompt (tools, skills, etc.) is still
    /// assembled normally.
    #[must_use]
    pub fn with_custom_prompt(mut self, prompt: String) -> Self {
        self.custom_prompt = Some(prompt);
        self
    }

    /// Sets an append prompt that is added at the end of the system prompt.
    ///
    /// The append prompt is always placed last, after all other components.
    #[must_use]
    pub fn with_append_prompt(mut self, prompt: String) -> Self {
        self.append_prompt = Some(prompt);
        self
    }

    /// Clears the append prompt, removing any previously set value.
    pub fn clear_append_prompt(&mut self) {
        self.append_prompt = None;
    }

    /// Sets the append prompt to an optional value.
    ///
    /// Use `None` to clear the append prompt, or `Some(prompt)` to set it.
    pub fn set_append_prompt_opt(&mut self, prompt: Option<String>) {
        self.append_prompt = prompt;
    }

    fn render_template(&self, template: &str, extra_vars: &[(&str, String)]) -> String {
        let mut rendered = template.to_string();
        let mut vars = self.template_vars.clone();
        for (key, value) in extra_vars {
            vars.insert((*key).to_string(), value.clone());
        }

        for (key, value) in vars {
            rendered = rendered.replace(&format!("{{{{{key}}}}}"), &value);
        }
        rendered
    }

    fn current_datetime() -> String {
        let seconds = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        format!("unix_seconds={seconds}")
    }

    fn tool_protocol_hint(&self) -> String {
        if self.tool_call_format.is_empty() {
            "Native tool calling is enabled. Use provider-native tool calls; do not emit textual tool-call JSON unless the provider requires a fallback.".to_string()
        } else {
            self.tool_call_format.trim().to_string()
        }
    }

    fn prompt_sections(&self) -> Vec<PromptSection> {
        let mut sections: Vec<PromptSection> = Vec::new();

        let stable_vars = [("tool_protocol_hint", self.tool_protocol_hint())];

        let identity = if let Some(ref custom) = self.custom_prompt {
            self.render_template(custom, &stable_vars)
        } else {
            self.render_template(&self.identity, &stable_vars)
        };
        sections.push(PromptSection {
            text: format!("# Identity\n{identity}\n"),
            kind: PromptSectionKind::Cacheable,
        });

        let mut sorted_tools: Vec<&ToolDescription> = self.tools.iter().collect();
        sorted_tools.sort_by(|a, b| a.name.cmp(&b.name));

        if sorted_tools.is_empty() {
            sections.push(PromptSection {
                text: String::from("# Tools\nNo tools available.\n"),
                kind: PromptSectionKind::Cacheable,
            });
        } else {
            let mut tools_section = String::from("# Tools\n");
            for tool in &sorted_tools {
                tools_section.push_str(&format!("## {}\n{}\n", tool.name, tool.description));
                if let Some(props) = tool.parameters.get("properties")
                    && let Some(required) = tool.parameters.get("required")
                {
                    let req_list: Vec<&str> = required
                        .as_array()
                        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
                        .unwrap_or_default();
                    let mut param_parts = Vec::new();
                    for (key, val) in props.as_object().unwrap_or(&serde_json::Map::new()) {
                        let desc = val
                            .get("description")
                            .and_then(|d| d.as_str())
                            .unwrap_or("");
                        let ptype = val.get("type").and_then(|t| t.as_str()).unwrap_or("any");
                        let req = if req_list.contains(&key.as_str()) {
                            "required"
                        } else {
                            "optional"
                        };
                        param_parts.push(format!("  - {} ({}): {} [{}]", key, ptype, desc, req));
                    }
                    if !param_parts.is_empty() {
                        tools_section.push_str("Parameters:\n");
                        tools_section.push_str(&param_parts.join("\n"));
                        tools_section.push_str("\n\n");
                    }
                }
                tools_section.push('\n');
            }
            sections.push(PromptSection {
                text: tools_section,
                kind: PromptSectionKind::Cacheable,
            });
        }

        if self.skill_index.is_empty() {
            sections.push(PromptSection {
                text: String::from("# Skills\nNo skills available.\n"),
                kind: PromptSectionKind::Cacheable,
            });
        } else {
            let mut skills_section = String::from("# Skills\n");
            for skill in &self.skill_index {
                skills_section.push_str(&format!("- **{}**: {}\n", skill.name, skill.description));
            }
            skills_section.push('\n');
            sections.push(PromptSection {
                text: skills_section,
                kind: PromptSectionKind::Cacheable,
            });
        }

        if self.context_files.is_empty() {
            sections.push(PromptSection {
                text: String::from("# Context\nNo context files loaded.\n"),
                kind: PromptSectionKind::Dynamic,
            });
        } else {
            let mut context_section = String::from("# Context\n");
            for file in &self.context_files {
                context_section.push_str(&format!("--- {} ---\n{}\n\n", file.path, file.content));
            }
            sections.push(PromptSection {
                text: context_section,
                kind: PromptSectionKind::Dynamic,
            });
        }

        if !self.user_preferences.is_empty() {
            sections.push(PromptSection {
                text: format!("# User Preferences\n{}\n", self.user_preferences),
                kind: PromptSectionKind::Dynamic,
            });
        }

        let runtime_section = self.render_template(
            "# Runtime Context\nCurrent datetime: {{datetime}}\n",
            &[("datetime", Self::current_datetime())],
        );
        sections.push(PromptSection {
            text: runtime_section,
            kind: PromptSectionKind::Dynamic,
        });

        if let Some(ref append) = self.append_prompt {
            sections.push(PromptSection {
                text: format!("# Additional Instructions\n{append}\n"),
                kind: PromptSectionKind::Dynamic,
            });
        }

        sections
    }

    /// Assembles and returns the final system prompt as a string.
    ///
    /// Components are assembled in the optimal order for caching:
    /// 1. Identity (or custom prompt if provided)
    /// 2. Tools (sorted by name)
    /// 3. Skill index
    /// 4. Context files
    /// 5. User preferences
    /// 6. Append prompt (if provided)
    ///
    /// Empty components are omitted from the output.
    #[must_use]
    pub fn build(&self) -> String {
        self.prompt_sections()
            .into_iter()
            .map(|section| section.text)
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Assembles the system prompt and emits the `OnSystemPromptBuilt` hook.
    ///
    /// Handlers may replace the final prompt by returning
    /// [`talos_plugin::HookResult::Modify`]. `Skip` leaves the prompt unchanged.
    pub(crate) async fn build_with_hooks(
        &self,
        hook_registry: &HookRegistry,
        ctx: &HookContext,
    ) -> Result<(String, Vec<SystemCacheMarker>), String> {
        let (prompt, markers) = self.build_with_cache_markers();
        let original_prompt = prompt.clone();
        let outcome = hook_registry
            .dispatch(ctx, HookEvent::OnSystemPromptBuilt { prompt: &prompt })
            .await;

        match outcome {
            HookOutcome::Continue(HookEvent::OnSystemPromptBuilt { prompt })
            | HookOutcome::Skip(HookEvent::OnSystemPromptBuilt { prompt }) => {
                let prompt = prompt.to_string();
                let markers = if prompt == original_prompt {
                    markers.into_iter().map(Into::into).collect()
                } else {
                    Vec::new()
                };
                Ok((prompt, markers))
            }
            HookOutcome::Deny { reason, .. } => Err(reason),
            HookOutcome::Continue(_) | HookOutcome::Skip(_) => {
                Ok((prompt, markers.into_iter().map(Into::into).collect()))
            }
        }
    }

    /// Assembles the system prompt with cache control markers.
    ///
    /// Returns the prompt string and a list of [`CacheMarker`]s indicating
    /// which byte ranges are stable and suitable for provider caching.
    ///
    /// Cacheable sections:
    /// - Identity (or custom prompt)
    /// - Tools
    /// - Skill index
    ///
    /// Semi-stable sections (context files, user preferences) and the
    /// append prompt are not marked for caching.
    #[must_use]
    pub fn build_with_cache_markers(&self) -> (String, Vec<CacheMarker>) {
        let mut markers: Vec<CacheMarker> = Vec::new();
        let sections = self.prompt_sections();
        let mut prompt = String::new();

        for (index, section) in sections.iter().enumerate() {
            if index > 0 {
                prompt.push('\n');
            }
            let offset = prompt.len();
            prompt.push_str(&section.text);
            if section.kind == PromptSectionKind::Cacheable {
                markers.push(CacheMarker {
                    offset,
                    length: section.text.len(),
                    cache_type: CacheType::Ephemeral,
                });
            }
        }

        (prompt, markers)
    }

    /// Estimates the total token count of the assembled prompt.
    ///
    /// Uses a heuristic of 1 token per 4 characters, which is a reasonable
    /// approximation for English text. This is not exact and should not be
    /// used for billing purposes.
    #[must_use]
    pub fn total_tokens(&self) -> usize {
        let prompt = self.build();
        // Heuristic: ~1 token per 4 characters for English text
        prompt.chars().count().div_ceil(4)
    }

    /// Logs the prompt size for debugging purposes.
    ///
    /// Prints the character count and estimated token count to stderr.
    /// This is useful for monitoring prompt size during development.
    pub fn log_size(&self) {
        let prompt = self.build();
        let char_count = prompt.chars().count();
        let token_estimate = self.total_tokens();
        eprintln!("System prompt: {char_count} characters, ~{token_estimate} tokens");
    }
}

impl Default for SystemPromptBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Basic assembly tests ---

    #[test]
    fn test_build_with_default_identity() {
        let builder = SystemPromptBuilder::new();
        let prompt = builder.build();

        assert!(prompt.contains("# Identity"));
        assert!(prompt.contains("You are Talos"));
    }

    #[test]
    fn test_build_with_all_components() {
        let builder = SystemPromptBuilder::new()
            .with_tools(vec![ToolDescription {
                name: "read".into(),
                description: "Read a file".into(),
                ..Default::default()
            }])
            .with_skill_index(vec![SkillIndex {
                name: "git-skill".into(),
                description: "Git operations".into(),
                triggers: vec!["git".into()],
                estimated_tokens: 0,
            }])
            .with_context_files(vec![ContextFile {
                path: "AGENTS.md".into(),
                content: "# Project Rules\nBe helpful.".into(),
            }])
            .with_user_preferences("Use British English.".into());

        let prompt = builder.build();

        assert!(prompt.contains("# Identity"));
        assert!(prompt.contains("# Tools"));
        assert!(prompt.contains("## read"));
        assert!(prompt.contains("# Skills"));
        assert!(prompt.contains("git-skill"));
        assert!(prompt.contains("# Context"));
        assert!(prompt.contains("AGENTS.md"));
        assert!(prompt.contains("# User Preferences"));
        assert!(prompt.contains("British English"));
    }

    #[test]
    fn test_build_with_missing_components() {
        let builder = SystemPromptBuilder::new();
        let prompt = builder.build();

        assert!(prompt.contains("# Identity"));
        assert!(prompt.contains("No tools available"));
        assert!(prompt.contains("No skills available"));
        assert!(prompt.contains("No context files loaded"));
        // User preferences should not appear when empty
        assert!(!prompt.contains("# User Preferences"));
        // Append prompt should not appear when not set
        assert!(!prompt.contains("# Additional Instructions"));
    }

    // --- Custom prompt tests ---

    #[test]
    fn test_custom_prompt_replaces_identity() {
        let builder =
            SystemPromptBuilder::new().with_custom_prompt("You are a custom assistant.".into());

        let prompt = builder.build();

        assert!(prompt.contains("You are a custom assistant."));
        assert!(!prompt.contains("You are Talos"));
    }

    #[test]
    fn test_custom_prompt_with_other_components() {
        let builder = SystemPromptBuilder::new()
            .with_custom_prompt("Custom identity.".into())
            .with_tools(vec![ToolDescription {
                name: "bash".into(),
                description: "Run commands".into(),
                ..Default::default()
            }]);

        let prompt = builder.build();

        assert!(prompt.contains("Custom identity."));
        assert!(prompt.contains("## bash"));
    }

    // --- Append prompt tests ---

    #[test]
    fn test_append_prompt_added_at_end() {
        let builder = SystemPromptBuilder::new().with_append_prompt("Always be concise.".into());

        let prompt = builder.build();

        assert!(prompt.contains("# Additional Instructions"));
        assert!(prompt.contains("Always be concise."));
        // Append should be the last section
        let append_pos = prompt
            .find("# Additional Instructions")
            .expect("append not found");
        let remaining = &prompt[append_pos..];
        assert!(!remaining[1..].contains("# Identity"));
        assert!(!remaining[1..].contains("# Tools"));
    }

    // --- Cache marker tests ---

    #[test]
    fn test_cache_marker_generation() {
        let builder = SystemPromptBuilder::new()
            .with_tools(vec![ToolDescription {
                name: "read".into(),
                description: "Read a file".into(),
                ..Default::default()
            }])
            .with_skill_index(vec![SkillIndex {
                name: "test-skill".into(),
                description: "A test skill".into(),
                triggers: vec!["test".into()],
                estimated_tokens: 0,
            }]);

        let (prompt, markers) = builder.build_with_cache_markers();

        assert_eq!(markers.len(), 3);

        // All markers should be Ephemeral type
        for marker in &markers {
            assert!(matches!(marker.cache_type, CacheType::Ephemeral));
        }

        // Markers should be in increasing order
        assert!(markers[0].offset < markers[1].offset);
        assert!(markers[1].offset < markers[2].offset);

        // Verify marker content matches prompt sections
        let identity_section = &prompt[markers[0].offset..markers[0].offset + markers[0].length];
        assert!(identity_section.contains("# Identity"));

        let tools_section = &prompt[markers[1].offset..markers[1].offset + markers[1].length];
        assert!(tools_section.contains("# Tools"));
        assert!(tools_section.contains("## read"));

        assert!(prompt.contains("# Skills"));
        assert!(prompt.contains("test-skill"));
    }

    #[test]
    fn test_identity_template_slots_are_rendered() {
        let builder = SystemPromptBuilder::new()
            .with_template_var("custom_slot", "custom value")
            .with_workspace_info("Workspace root: /repo")
            .with_model_info("Model: test-model")
            .with_custom_prompt(
                "{{workspace_info}}\n{{model_info}}\n{{tool_protocol_hint}}\n{{custom_slot}}"
                    .into(),
            );

        let prompt = builder.build();

        assert!(prompt.contains("Workspace root: /repo"));
        assert!(prompt.contains("Model: test-model"));
        assert!(prompt.contains("Native tool calling is enabled"));
        assert!(prompt.contains("custom value"));
        assert!(!prompt.contains("{{workspace_info}}"));
        assert!(!prompt.contains("{{model_info}}"));
        assert!(!prompt.contains("{{tool_protocol_hint}}"));
    }

    #[test]
    fn test_datetime_lives_after_cache_markers() {
        let builder = SystemPromptBuilder::new();
        let (prompt, markers) = builder.build_with_cache_markers();

        let runtime_pos = prompt
            .find("# Runtime Context")
            .expect("runtime context should be present");
        assert!(prompt.contains("Current datetime: unix_seconds="));
        for marker in markers {
            assert!(
                marker.offset + marker.length <= runtime_pos,
                "cache marker must not include dynamic datetime section"
            );
        }
    }

    #[test]
    fn test_cache_markers_with_empty_components() {
        let builder = SystemPromptBuilder::new();
        let (_prompt, markers) = builder.build_with_cache_markers();

        assert_eq!(markers.len(), 3);

        // Identity marker
        assert!(markers[0].length > 0);
        // Tools marker (empty)
        assert!(markers[1].length > 0);
        // Skills marker (empty)
        assert!(markers[2].length > 0);
    }

    // --- Token estimation tests ---

    #[test]
    fn test_token_estimation() {
        let builder = SystemPromptBuilder::new();
        let tokens = builder.total_tokens();

        // Default identity is ~100 chars, so ~25 tokens minimum
        assert!(tokens > 10);
    }

    #[test]
    fn test_token_estimation_with_content() {
        let builder = SystemPromptBuilder::new()
            .with_tools(vec![
                ToolDescription {
                    name: "read".into(),
                    description: "Read a file".into(),
                    ..Default::default()
                },
                ToolDescription {
                    name: "write".into(),
                    description: "Write a file".into(),
                    ..Default::default()
                },
            ])
            .with_context_files(vec![ContextFile {
                path: "AGENTS.md".into(),
                content: "A".repeat(1000),
            }]);

        let tokens = builder.total_tokens();
        // 1000 chars of context alone should be ~250 tokens
        assert!(tokens > 200);
    }

    // --- Prompt ordering tests ---

    #[test]
    fn test_prompt_ordering_identity_first() {
        let builder = SystemPromptBuilder::new().with_tools(vec![ToolDescription {
            name: "bash".into(),
            description: "Run commands".into(),
            ..Default::default()
        }]);

        let prompt = builder.build();

        let identity_pos = prompt.find("# Identity").expect("identity not found");
        let tools_pos = prompt.find("# Tools").expect("tools not found");

        assert!(
            identity_pos < tools_pos,
            "identity should come before tools"
        );
    }

    #[test]
    fn test_prompt_ordering_tools_before_skills() {
        let builder = SystemPromptBuilder::new()
            .with_tools(vec![ToolDescription {
                name: "bash".into(),
                description: "Run commands".into(),
                ..Default::default()
            }])
            .with_skill_index(vec![SkillIndex {
                name: "test".into(),
                description: "Test skill".into(),
                triggers: vec![],
                estimated_tokens: 0,
            }]);

        let prompt = builder.build();

        let tools_pos = prompt.find("# Tools").expect("tools not found");
        let skills_pos = prompt.find("# Skills").expect("skills not found");

        assert!(tools_pos < skills_pos, "tools should come before skills");
    }

    #[test]
    fn test_prompt_ordering_skills_before_context() {
        let builder = SystemPromptBuilder::new()
            .with_skill_index(vec![SkillIndex {
                name: "test".into(),
                description: "Test skill".into(),
                triggers: vec![],
                estimated_tokens: 0,
            }])
            .with_context_files(vec![ContextFile {
                path: "AGENTS.md".into(),
                content: "Rules".into(),
            }]);

        let prompt = builder.build();

        let skills_pos = prompt.find("# Skills").expect("skills not found");
        let context_pos = prompt.find("# Context").expect("context not found");

        assert!(
            skills_pos < context_pos,
            "skills should come before context"
        );
    }

    #[test]
    fn test_prompt_ordering_context_before_preferences() {
        let builder = SystemPromptBuilder::new()
            .with_context_files(vec![ContextFile {
                path: "AGENTS.md".into(),
                content: "Rules".into(),
            }])
            .with_user_preferences("Be concise.".into());

        let prompt = builder.build();

        let context_pos = prompt.find("# Context").expect("context not found");
        let prefs_pos = prompt.find("# User Preferences").expect("prefs not found");

        assert!(
            context_pos < prefs_pos,
            "context should come before preferences"
        );
    }

    #[test]
    fn test_prompt_ordering_append_last() {
        let builder = SystemPromptBuilder::new()
            .with_user_preferences("Be concise.".into())
            .with_append_prompt("Extra instructions.".into());

        let prompt = builder.build();

        let prefs_pos = prompt.find("# User Preferences").expect("prefs not found");
        let append_pos = prompt
            .find("# Additional Instructions")
            .expect("append not found");

        assert!(
            prefs_pos < append_pos,
            "preferences should come before append"
        );
    }

    // --- Tools sorting tests ---

    #[test]
    fn test_tools_sorted_alphabetically() {
        let builder = SystemPromptBuilder::new().with_tools(vec![
            ToolDescription {
                name: "write".into(),
                description: "Write a file".into(),
                ..Default::default()
            },
            ToolDescription {
                name: "bash".into(),
                description: "Run commands".into(),
                ..Default::default()
            },
            ToolDescription {
                name: "read".into(),
                description: "Read a file".into(),
                ..Default::default()
            },
        ]);

        let prompt = builder.build();

        let bash_pos = prompt.find("## bash").expect("bash not found");
        let read_pos = prompt.find("## read").expect("read not found");
        let write_pos = prompt.find("## write").expect("write not found");

        assert!(bash_pos < read_pos, "bash should come before read");
        assert!(read_pos < write_pos, "read should come before write");
    }

    // --- Log size test ---

    #[test]
    fn test_log_size_does_not_panic() {
        let builder = SystemPromptBuilder::new();
        // Should not panic, just prints to stderr
        builder.log_size();
    }

    // --- Default trait test ---

    #[test]
    fn test_default_builder() {
        let builder = SystemPromptBuilder::default();
        let prompt = builder.build();
        assert!(prompt.contains("# Identity"));
    }

    // --- Clone test ---

    #[test]
    fn test_builder_clone() {
        let builder = SystemPromptBuilder::new().with_tools(vec![ToolDescription {
            name: "read".into(),
            description: "Read a file".into(),
            ..Default::default()
        }]);

        let cloned = builder.clone();
        let prompt1 = builder.build();
        let prompt2 = cloned.build();

        assert_eq!(prompt1, prompt2);
    }
}
