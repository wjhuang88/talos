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

mod assets;
mod builder;
mod sections;
mod types;

#[cfg(test)]
mod tests;

pub use assets::{DEFAULT_IDENTITY, MEMORY_PROMPT, TOOL_CALLING_FORMAT, TOOL_CALLING_STRICT};
pub use builder::SystemPromptBuilder;
pub use types::{ActivatedSkillContext, CacheMarker, CacheType, ContextFile, ToolDescription};
