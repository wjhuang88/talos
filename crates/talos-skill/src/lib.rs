//! SKILL.md parser and loader for Talos agent skills.
//!
//! This crate discovers and parses `SKILL.md` files from configured search paths.
//! Each skill consists of YAML frontmatter (between `---` delimiters) followed by
//! a Markdown body containing instructions.
//!
//! # Discovery Paths
//!
//! Skills are discovered from three locations (in priority order):
//! 1. `.talos/skills/` — project-local skills
//! 2. `~/.talos/skills/` — user-global skills
//! 3. Parent directories up to git root — inherited skills
//!
//! Directory-link following is controlled by `SkillDiscoveryPolicy`.
//! By default links are not followed; external targets are denied.
//! Discovery is bounded by the configured maximum depth and entry budget.
//!
//! # SKILL.md Format
//!
//! ```markdown
//! ---
//! name: my-skill
//! description: A skill that does something useful
//! triggers:
//!   - keyword1
//!   - keyword2
//! ---
//!
//! # Instructions
//!
//! Detailed markdown instructions go here...
//! ```

mod error;
mod loader;
mod manager;
mod parser;
mod token;
mod types;

pub use error::{Result, SkillError};
pub use loader::{
    ExternalTargetPolicy, SkillDiscoveryPolicy, SkillDiscoveryWarning, SkillDiscoveryWarningKind,
    SkillLoader,
};
pub use manager::SkillManager;
pub use token::estimate_tokens;
pub use types::{Skill, SkillDisclosure, SkillFrontmatter, SkillIndex, SkillSource};

#[cfg(test)]
#[allow(warnings)]
mod tests;
