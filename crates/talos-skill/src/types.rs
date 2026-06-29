use serde::Deserialize;
use std::path::PathBuf;

/// Identifies where a skill was discovered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum SkillSource {
    /// `.talos/skills/` in the active workspace.
    #[default]
    Project,
    /// Parent `.talos/skills/` directories (monorepo inheritance).
    Parent,
    /// `~/.talos/skills/` (user-global Talos-owned).
    UserGlobal,
    /// `~/.agents/skills/` (shared, opt-in).
    Shared,
}

impl std::fmt::Display for SkillSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillSource::Project => write!(f, "project"),
            SkillSource::Parent => write!(f, "parent"),
            SkillSource::UserGlobal => write!(f, "user"),
            SkillSource::Shared => write!(f, "shared"),
        }
    }
}

/// YAML frontmatter extracted from a SKILL.md file.
///
/// All fields are required. The frontmatter must appear between `---` delimiters
/// at the start of the file.
#[derive(Debug, Clone, Deserialize)]
pub struct SkillFrontmatter {
    /// Unique name identifier for the skill.
    pub name: String,
    /// Human-readable description of what the skill does.
    pub description: String,
    /// Keywords or patterns that activate this skill.
    pub triggers: Vec<String>,
}

/// A fully parsed skill with frontmatter metadata and Markdown body.
#[derive(Debug, Clone)]
pub struct Skill {
    /// Unique name identifier for the skill.
    pub name: String,
    /// Human-readable description of what the skill does.
    pub description: String,
    /// Keywords or patterns that activate this skill.
    pub triggers: Vec<String>,
    /// Markdown instructions (body content after frontmatter).
    pub body: String,
    /// Absolute path to the source SKILL.md file.
    pub source_path: PathBuf,
    /// Where this skill was discovered from.
    pub source: SkillSource,
}

/// Lightweight skill index entry for Level 0 progressive disclosure.
///
/// Contains only the metadata needed to inject into a system prompt,
/// without loading the full Markdown body.
#[derive(Debug, Clone)]
pub struct SkillIndex {
    /// Unique name identifier for the skill.
    pub name: String,
    /// Human-readable description of what the skill does.
    pub description: String,
    /// Keywords or patterns that activate this skill.
    pub triggers: Vec<String>,
    /// Estimated token count for this skill's Level 0 entry (name + description).
    pub estimated_tokens: usize,
    /// Where this skill was discovered from.
    pub source: SkillSource,
}

/// Disclosure level for progressive skill loading.
///
/// Skills are loaded in three levels to minimize system prompt size:
/// - **Level 0**: Name + description only — always present in the system prompt
///   so the agent knows which skills are available (~50 tokens each).
/// - **Level 1**: Full SKILL.md body — loaded on demand when the agent's task
///   matches a skill's triggers.
/// - **Level 2**: Specific reference files — loaded when the skill body
///   references external files (e.g., templates, schemas, scripts).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillDisclosure {
    /// Name + description only (always loaded).
    Level0,
    /// Full SKILL.md body (loaded on demand when task matches triggers).
    Level1,
    /// Specific reference files (loaded when skill body references them).
    Level2,
}
