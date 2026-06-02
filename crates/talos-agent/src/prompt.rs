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

use talos_plugin::{HookContext, HookEvent, HookOutcome, HookRegistry};
use talos_skill::SkillIndex;

/// Default identity text for the Talos agent.
pub const DEFAULT_IDENTITY: &str = "You are Talos, an AI coding assistant. You help users with programming tasks \
     by using tools to read, write, and execute code.";

/// A description of a tool for inclusion in the system prompt.
///
/// Contains only the name and human-readable description, without the full
/// JSON Schema parameters. This is sufficient for the system prompt context.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolDescription {
    /// Unique name of the tool.
    pub name: String,
    /// Human-readable description of what the tool does.
    pub description: String,
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
}

impl SystemPromptBuilder {
    /// Creates a new builder with the default identity and no other components.
    ///
    /// All optional components start empty. Use the builder methods to
    /// configure tools, skills, context files, and other components.
    #[must_use]
    pub fn new() -> Self {
        Self {
            identity: DEFAULT_IDENTITY.to_string(),
            tools: Vec::new(),
            skill_index: Vec::new(),
            context_files: Vec::new(),
            user_preferences: String::new(),
            custom_prompt: None,
            append_prompt: None,
        }
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
        let mut parts: Vec<String> = Vec::new();

        // 1. Identity or custom prompt
        if let Some(ref custom) = self.custom_prompt {
            parts.push(format!("# Identity\n{custom}\n"));
        } else {
            parts.push(format!("# Identity\n{}\n", self.identity));
        }

        // 2. Tools
        let mut sorted_tools: Vec<&ToolDescription> = self.tools.iter().collect();
        sorted_tools.sort_by(|a, b| a.name.cmp(&b.name));

        if sorted_tools.is_empty() {
            parts.push(String::from("# Tools\nNo tools available.\n"));
        } else {
            let mut tools_section = String::from("# Tools\n");
            for tool in &sorted_tools {
                tools_section.push_str(&format!("## {}\n{}\n\n", tool.name, tool.description));
            }
            parts.push(tools_section);
        }

        // 3. Skill index
        if self.skill_index.is_empty() {
            parts.push(String::from("# Skills\nNo skills available.\n"));
        } else {
            let mut skills_section = String::from("# Skills\n");
            for skill in &self.skill_index {
                skills_section.push_str(&format!("- **{}**: {}\n", skill.name, skill.description));
            }
            skills_section.push('\n');
            parts.push(skills_section);
        }

        // 4. Context files
        if self.context_files.is_empty() {
            parts.push(String::from("# Context\nNo context files loaded.\n"));
        } else {
            let mut context_section = String::from("# Context\n");
            for file in &self.context_files {
                context_section.push_str(&format!("--- {} ---\n{}\n\n", file.path, file.content));
            }
            parts.push(context_section);
        }

        // 5. User preferences
        if !self.user_preferences.is_empty() {
            parts.push(format!("# User Preferences\n{}\n", self.user_preferences));
        }

        // 6. Append prompt
        if let Some(ref append) = self.append_prompt {
            parts.push(format!("# Additional Instructions\n{append}\n"));
        }

        parts.join("\n")
    }

    /// Assembles the system prompt and emits the `OnSystemPromptBuilt` hook.
    ///
    /// Handlers may replace the final prompt by returning
    /// [`talos_plugin::HookResult::Modify`]. `Skip` leaves the prompt unchanged.
    pub(crate) async fn build_with_hooks(
        &self,
        hook_registry: &HookRegistry,
        ctx: &HookContext,
    ) -> Result<String, String> {
        let prompt = self.build();
        let outcome = hook_registry
            .dispatch(ctx, HookEvent::OnSystemPromptBuilt { prompt: &prompt })
            .await;

        match outcome {
            HookOutcome::Continue(HookEvent::OnSystemPromptBuilt { prompt })
            | HookOutcome::Skip(HookEvent::OnSystemPromptBuilt { prompt }) => {
                Ok(prompt.to_string())
            }
            HookOutcome::Deny { reason, .. } => Err(reason),
            HookOutcome::Continue(_) | HookOutcome::Skip(_) => Ok(prompt),
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
        let prompt = self.build();
        let mut markers: Vec<CacheMarker> = Vec::new();
        let mut offset: usize = 0;

        // Marker 1: Identity section
        let identity_header = "# Identity\n";
        if let Some(ref custom) = self.custom_prompt {
            let section_len = identity_header.len() + custom.len() + 1; // +1 for trailing \n
            markers.push(CacheMarker {
                offset,
                length: section_len,
                cache_type: CacheType::Ephemeral,
            });
            offset += section_len;
        } else {
            let section_len = identity_header.len() + self.identity.len() + 1;
            markers.push(CacheMarker {
                offset,
                length: section_len,
                cache_type: CacheType::Ephemeral,
            });
            offset += section_len;
        }

        // Skip the separator \n between sections
        offset += 1;

        // Marker 2: Tools section
        let tools_header = "# Tools\n";
        let tools_len = if self.tools.is_empty() {
            tools_header.len() + "No tools available.\n".len()
        } else {
            let mut len = tools_header.len();
            let mut sorted_tools: Vec<&ToolDescription> = self.tools.iter().collect();
            sorted_tools.sort_by(|a, b| a.name.cmp(&b.name));
            for tool in &sorted_tools {
                len += format!("## {}\n{}\n\n", tool.name, tool.description).len();
            }
            len
        };
        markers.push(CacheMarker {
            offset,
            length: tools_len,
            cache_type: CacheType::Ephemeral,
        });
        offset += tools_len;

        // Skip the separator \n between sections
        offset += 1;

        // Marker 3: Skill index section
        let skills_header = "# Skills\n";
        let skills_len = if self.skill_index.is_empty() {
            skills_header.len() + "No skills available.\n".len()
        } else {
            let mut len = skills_header.len();
            for skill in &self.skill_index {
                len += format!("- **{}**: {}\n", skill.name, skill.description).len();
            }
            len + 1 // +1 for trailing \n
        };
        markers.push(CacheMarker {
            offset,
            length: skills_len,
            cache_type: CacheType::Ephemeral,
        });

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

        let skills_section = &prompt[markers[2].offset..markers[2].offset + markers[2].length];
        assert!(skills_section.contains("# Skills"));
        assert!(skills_section.contains("test-skill"));
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
                },
                ToolDescription {
                    name: "write".into(),
                    description: "Write a file".into(),
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
            },
            ToolDescription {
                name: "bash".into(),
                description: "Run commands".into(),
            },
            ToolDescription {
                name: "read".into(),
                description: "Read a file".into(),
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
        }]);

        let cloned = builder.clone();
        let prompt1 = builder.build();
        let prompt2 = cloned.build();

        assert_eq!(prompt1, prompt2);
    }
}
