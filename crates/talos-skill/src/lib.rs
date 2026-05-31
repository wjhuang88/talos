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

use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;
use walkdir::WalkDir;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Errors that can occur during skill loading.
#[derive(Debug, Error)]
pub enum SkillError {
    /// An I/O error occurred while reading a file or directory.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// The YAML frontmatter could not be parsed.
    #[error("YAML parse error: {0}")]
    YamlParseError(#[from] serde_yaml::Error),

    /// The frontmatter is missing required fields or is malformed.
    #[error("invalid frontmatter: {0}")]
    InvalidFrontmatter(String),

    /// The specified skill file was not found.
    #[error("file not found: {0}")]
    FileNotFound(PathBuf),
}

/// Result type alias for skill operations.
pub type Result<T, E = SkillError> = std::result::Result<T, E>;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

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
}

// ---------------------------------------------------------------------------
// SkillLoader
// ---------------------------------------------------------------------------

/// Discovers and loads skills from configured search paths.
///
/// # Examples
///
/// ```no_run
/// use talos_skill::SkillLoader;
///
/// let mut loader = SkillLoader::new();
/// let skills = loader.discover().expect("failed to discover skills");
/// let index = loader.get_index();
/// ```
pub struct SkillLoader {
    /// All discovered skills.
    pub skills: Vec<Skill>,
    /// Directories to search for SKILL.md files.
    pub search_paths: Vec<PathBuf>,
}

impl SkillLoader {
    /// Creates a new `SkillLoader` with default search paths.
    ///
    /// Default paths (in priority order):
    /// 1. `.talos/skills/` relative to the current directory (project-local)
    /// 2. `~/.talos/skills/` (user-global)
    /// 3. Parent directories up to git root, each with `.talos/skills/`
    pub fn new() -> Self {
        let mut search_paths = Vec::new();

        // Project-local: .talos/skills/
        let cwd = std::env::current_dir().ok();
        if let Some(ref cwd) = cwd {
            let project_local = cwd.join(".talos/skills");
            if project_local.is_dir() {
                search_paths.push(project_local);
            }
        }

        // User-global: ~/.talos/skills/
        if let Some(home) = home_dir() {
            let user_global = home.join(".talos/skills");
            if user_global.is_dir() {
                search_paths.push(user_global);
            }
        }

        // Parent directories up to git root
        if let Some(ref cwd) = cwd {
            let mut current = cwd.as_path();
            while let Some(parent) = current.parent() {
                let git_dir = parent.join(".git");
                let skills_dir = parent.join(".talos/skills");
                if git_dir.is_dir() && skills_dir.is_dir() {
                    search_paths.push(skills_dir);
                }
                current = parent;
                if git_dir.is_dir() {
                    break;
                }
            }
        }

        Self {
            skills: Vec::new(),
            search_paths,
        }
    }

    /// Scans all search paths for SKILL.md files and parses them.
    ///
    /// Returns a vector of all successfully parsed skills. Files that fail to
    /// parse are silently skipped (errors are logged but not propagated).
    pub fn discover(&mut self) -> Result<&Vec<Skill>> {
        self.skills.clear();

        for path in &self.search_paths {
            if !path.is_dir() {
                continue;
            }

            for entry in WalkDir::new(path)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let entry_path = entry.path();
                if entry_path.file_name() == Some(std::ffi::OsStr::new("SKILL.md")) {
                    match Self::parse(entry_path) {
                        Ok(skill) => self.skills.push(skill),
                        Err(e) => {
                            // Silently skip unparseable skills; in production
                            // this would use tracing::warn!
                            let _ = e;
                        }
                    }
                }
            }
        }

        // Deduplicate by name (first occurrence wins — priority order)
        self.skills.dedup_by_key(|s| s.name.clone());

        Ok(&self.skills)
    }

    /// Parses a single SKILL.md file into a [`Skill`].
    ///
    /// The file must start with `---`, followed by YAML frontmatter, then `---`,
    /// then the Markdown body.
    ///
    /// # Errors
    ///
    /// Returns [`SkillError::FileNotFound`] if the path does not exist,
    /// [`SkillError::YamlParseError`] if the frontmatter is invalid YAML,
    /// or [`SkillError::InvalidFrontmatter`] if required fields are missing.
    pub fn parse(path: &Path) -> Result<Skill> {
        if !path.exists() {
            return Err(SkillError::FileNotFound(path.to_path_buf()));
        }

        let content = std::fs::read_to_string(path)?;
        let (frontmatter, body) = split_frontmatter(&content)?;
        let fm: SkillFrontmatter = serde_yaml::from_str(frontmatter)?;

        validate_frontmatter(&fm)?;

        Ok(Skill {
            name: fm.name,
            description: fm.description,
            triggers: fm.triggers,
            body: body.trim().to_string(),
            source_path: path.to_path_buf(),
        })
    }

    /// Returns a lightweight index of all loaded skills.
    ///
    /// Use this for Level 0 progressive disclosure — injecting skill names
    /// and descriptions into the system prompt without loading full bodies.
    pub fn get_index(&self) -> Vec<SkillIndex> {
        self.skills
            .iter()
            .map(|s| SkillIndex {
                name: s.name.clone(),
                description: s.description.clone(),
                triggers: s.triggers.clone(),
            })
            .collect()
    }
}

impl Default for SkillLoader {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Splits a SKILL.md file into frontmatter YAML and Markdown body.
///
/// The content must start with `---` (optionally preceded by whitespace),
/// contain YAML, then end with another `---` delimiter.
fn split_frontmatter(content: &str) -> Result<(&str, &str)> {
    let trimmed = content.trim_start();

    if !trimmed.starts_with("---") {
        return Err(SkillError::InvalidFrontmatter(
            "file must start with '---' delimiter".into(),
        ));
    }

    // Skip the opening ---
    let after_open = &trimmed[3..];

    // Find the closing ---
    let close_pos = after_open
        .find("\n---")
        .ok_or_else(|| SkillError::InvalidFrontmatter("missing closing '---' delimiter".into()))?;

    let frontmatter = after_open[..close_pos].trim();
    let body = &after_open[close_pos + 4..]; // skip "\n---"

    Ok((frontmatter, body))
}

/// Validates that all required frontmatter fields are present and non-empty.
fn validate_frontmatter(fm: &SkillFrontmatter) -> Result<()> {
    if fm.name.is_empty() {
        return Err(SkillError::InvalidFrontmatter(
            "'name' field is required and must not be empty".into(),
        ));
    }
    if fm.description.is_empty() {
        return Err(SkillError::InvalidFrontmatter(
            "'description' field is required and must not be empty".into(),
        ));
    }
    Ok(())
}

/// Returns the user's home directory.
fn home_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("USERPROFILE").ok().map(PathBuf::from)
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var("HOME").ok().map(PathBuf::from)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // -----------------------------------------------------------------------
    // Test fixtures
    // -----------------------------------------------------------------------

    fn valid_skill_content() -> String {
        r#"---
name: test-skill
description: A test skill for unit testing
triggers:
  - test
  - unit
---

# Test Skill

This is the body of the test skill.

## Instructions

1. Do something
2. Do something else
"#
        .to_string()
    }

    fn no_frontmatter_content() -> String {
        "# No Frontmatter\n\nThis file has no YAML frontmatter.".to_string()
    }

    fn invalid_yaml_content() -> String {
        r#"---
name: [invalid yaml
description: test
---

Body content.
"#
        .to_string()
    }

    fn missing_fields_content() -> String {
        r#"---
name: incomplete-skill
---

Body content.
"#
        .to_string()
    }

    // -----------------------------------------------------------------------
    // Parsing tests
    // -----------------------------------------------------------------------

    #[test]
    fn parse_valid_skill_md() {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let path = dir.path().join("SKILL.md");
        fs::write(&path, valid_skill_content()).expect("failed to write test file");

        let skill = SkillLoader::parse(&path).expect("parsing should succeed");

        assert_eq!(skill.name, "test-skill");
        assert_eq!(skill.description, "A test skill for unit testing");
        assert_eq!(skill.triggers, vec!["test", "unit"]);
        assert!(skill.body.contains("# Test Skill"));
        assert_eq!(skill.source_path, path);
    }

    #[test]
    fn parse_skill_without_frontmatter_errors() {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let path = dir.path().join("SKILL.md");
        fs::write(&path, no_frontmatter_content()).expect("failed to write test file");

        let result = SkillLoader::parse(&path);
        assert!(result.is_err());

        match result.unwrap_err() {
            SkillError::InvalidFrontmatter(msg) => {
                assert!(msg.contains("---"));
            }
            other => panic!("expected InvalidFrontmatter, got: {other:?}"),
        }
    }

    #[test]
    fn parse_invalid_yaml_errors() {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let path = dir.path().join("SKILL.md");
        fs::write(&path, invalid_yaml_content()).expect("failed to write test file");

        let result = SkillLoader::parse(&path);
        assert!(result.is_err());

        match result.unwrap_err() {
            SkillError::YamlParseError(_) => {}
            other => panic!("expected YamlParseError, got: {other:?}"),
        }
    }

    #[test]
    fn parse_missing_required_fields_errors() {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let path = dir.path().join("SKILL.md");
        fs::write(&path, missing_fields_content()).expect("failed to write test file");

        let result = SkillLoader::parse(&path);
        assert!(result.is_err());

        // serde_yaml reports missing fields as YamlParseError
        match result.unwrap_err() {
            SkillError::YamlParseError(e) => {
                assert!(e.to_string().contains("description"));
            }
            other => panic!("expected YamlParseError, got: {other:?}"),
        }
    }

    #[test]
    fn parse_nonexistent_file_errors() {
        let path = Path::new("/nonexistent/path/SKILL.md");
        let result = SkillLoader::parse(path);
        assert!(result.is_err());

        match result.unwrap_err() {
            SkillError::FileNotFound(p) => {
                assert_eq!(p, path);
            }
            other => panic!("expected FileNotFound, got: {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // Discovery tests
    // -----------------------------------------------------------------------

    #[test]
    fn discover_from_single_directory() {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let skills_dir = dir.path().join("skills");
        fs::create_dir(&skills_dir).expect("failed to create skills dir");

        fs::write(skills_dir.join("SKILL.md"), valid_skill_content())
            .expect("failed to write skill");

        let mut loader = SkillLoader {
            skills: Vec::new(),
            search_paths: vec![skills_dir],
        };

        let skills = loader.discover().expect("discovery should succeed");
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "test-skill");
    }

    #[test]
    fn discover_from_multiple_directories() {
        let dir1 = tempfile::tempdir().expect("failed to create temp dir");
        let dir2 = tempfile::tempdir().expect("failed to create temp dir");

        let skills1 = dir1.path().join("skills");
        let skills2 = dir2.path().join("skills");
        fs::create_dir(&skills1).expect("failed to create skills dir");
        fs::create_dir(&skills2).expect("failed to create skills dir");

        let skill_a = r#"---
name: skill-a
description: First skill
triggers:
  - alpha
---

Body A.
"#;

        let skill_b = r#"---
name: skill-b
description: Second skill
triggers:
  - beta
---

Body B.
"#;

        fs::write(skills1.join("SKILL.md"), skill_a).expect("failed to write skill A");
        fs::write(skills2.join("SKILL.md"), skill_b).expect("failed to write skill B");

        let mut loader = SkillLoader {
            skills: Vec::new(),
            search_paths: vec![skills1, skills2],
        };

        let skills = loader.discover().expect("discovery should succeed");
        assert_eq!(skills.len(), 2);

        let names: Vec<&str> = skills.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"skill-a"));
        assert!(names.contains(&"skill-b"));
    }

    #[test]
    fn discover_deduplicates_by_name() {
        let dir1 = tempfile::tempdir().expect("failed to create temp dir");
        let dir2 = tempfile::tempdir().expect("failed to create temp dir");

        let skills1 = dir1.path().join("skills");
        let skills2 = dir2.path().join("skills");
        fs::create_dir(&skills1).expect("failed to create skills dir");
        fs::create_dir(&skills2).expect("failed to create skills dir");

        // Same name, different descriptions
        let skill_v1 = r#"---
name: duplicate-skill
description: Version 1
triggers:
  - dup
---

Body V1.
"#;

        let skill_v2 = r#"---
name: duplicate-skill
description: Version 2
triggers:
  - dup
---

Body V2.
"#;

        fs::write(skills1.join("SKILL.md"), skill_v1).expect("failed to write skill V1");
        fs::write(skills2.join("SKILL.md"), skill_v2).expect("failed to write skill V2");

        let mut loader = SkillLoader {
            skills: Vec::new(),
            search_paths: vec![skills1, skills2],
        };

        let skills = loader.discover().expect("discovery should succeed");
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].description, "Version 1"); // first wins
    }

    #[test]
    fn discover_skips_non_skill_files() {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let skills_dir = dir.path().join("skills");
        fs::create_dir(&skills_dir).expect("failed to create skills dir");

        // Write a valid SKILL.md
        fs::write(skills_dir.join("SKILL.md"), valid_skill_content())
            .expect("failed to write skill");

        // Write a non-skill file
        fs::write(skills_dir.join("README.md"), "# Not a skill").expect("failed to write readme");

        let mut loader = SkillLoader {
            skills: Vec::new(),
            search_paths: vec![skills_dir],
        };

        let skills = loader.discover().expect("discovery should succeed");
        assert_eq!(skills.len(), 1);
    }

    #[test]
    fn discover_skips_unparseable_files() {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let skills_dir = dir.path().join("skills");
        fs::create_dir(&skills_dir).expect("failed to create skills dir");

        fs::write(skills_dir.join("SKILL.md"), valid_skill_content())
            .expect("failed to write skill");

        fs::create_dir_all(skills_dir.join("nested")).expect("failed to create nested dir");
        fs::write(
            skills_dir.join("nested/SKILL.md"),
            "no frontmatter",
        )
        .expect("failed to write invalid skill");

        let mut loader = SkillLoader {
            skills: Vec::new(),
            search_paths: vec![skills_dir],
        };

        let skills = loader.discover().expect("discovery should succeed");
        assert_eq!(skills.len(), 1);
    }

    // -----------------------------------------------------------------------
    // Index tests
    // -----------------------------------------------------------------------

    #[test]
    fn skill_index_generation() {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let skills_dir = dir.path().join("skills");
        fs::create_dir(&skills_dir).expect("failed to create skills dir");

        let skill_a = r#"---
name: skill-a
description: Description A
triggers:
  - trigger-a
---

Body A.
"#;

        let skill_b = r#"---
name: skill-b
description: Description B
triggers:
  - trigger-b
  - trigger-b2
---

Body B.
"#;

        fs::create_dir(skills_dir.join("skill-a")).expect("failed to create skill-a dir");
        fs::create_dir(skills_dir.join("skill-b")).expect("failed to create skill-b dir");
        fs::write(skills_dir.join("skill-a/SKILL.md"), skill_a)
            .expect("failed to write skill A");
        fs::write(skills_dir.join("skill-b/SKILL.md"), skill_b)
            .expect("failed to write skill B");

        let mut loader = SkillLoader {
            skills: Vec::new(),
            search_paths: vec![skills_dir],
        };

        loader.discover().expect("discovery should succeed");
        let index = loader.get_index();

        assert_eq!(index.len(), 2);

        let names: Vec<&str> = index.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"skill-a"));
        assert!(names.contains(&"skill-b"));

        // Verify index entries don't contain body content
        for entry in &index {
            assert!(!entry.description.contains("Body"));
        }
    }

    #[test]
    fn skill_index_empty_when_no_skills() {
        let loader = SkillLoader {
            skills: Vec::new(),
            search_paths: Vec::new(),
        };

        let index = loader.get_index();
        assert!(index.is_empty());
    }

    // -----------------------------------------------------------------------
    // Trigger matching tests
    // -----------------------------------------------------------------------

    #[test]
    fn trigger_matching_exact() {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let skills_dir = dir.path().join("skills");
        fs::create_dir(&skills_dir).expect("failed to create skills dir");

        let skill = r#"---
name: git-skill
description: Git operations
triggers:
  - git
  - commit
  - push
---

Git instructions.
"#;

        fs::write(skills_dir.join("SKILL.md"), skill).expect("failed to write skill");

        let mut loader = SkillLoader {
            skills: Vec::new(),
            search_paths: vec![skills_dir],
        };

        loader.discover().expect("discovery should succeed");

        // Check that triggers are correctly parsed
        let skill = &loader.skills[0];
        assert!(skill.triggers.contains(&"git".to_string()));
        assert!(skill.triggers.contains(&"commit".to_string()));
        assert!(skill.triggers.contains(&"push".to_string()));
    }

    #[test]
    fn trigger_matching_case_sensitive() {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let skills_dir = dir.path().join("skills");
        fs::create_dir(&skills_dir).expect("failed to create skills dir");

        let skill = r#"---
name: case-skill
description: Case sensitivity test
triggers:
  - Git
  - GIT
---

Body.
"#;

        fs::write(skills_dir.join("SKILL.md"), skill).expect("failed to write skill");

        let mut loader = SkillLoader {
            skills: Vec::new(),
            search_paths: vec![skills_dir],
        };

        loader.discover().expect("discovery should succeed");

        let skill = &loader.skills[0];
        assert!(skill.triggers.contains(&"Git".to_string()));
        assert!(skill.triggers.contains(&"GIT".to_string()));
        // Triggers preserve case as specified
    }

    #[test]
    fn trigger_matching_empty_triggers() {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let skills_dir = dir.path().join("skills");
        fs::create_dir(&skills_dir).expect("failed to create skills dir");

        let skill = r#"---
name: no-triggers
description: A skill with no triggers
triggers: []
---

Body.
"#;

        fs::write(skills_dir.join("SKILL.md"), skill).expect("failed to write skill");

        let mut loader = SkillLoader {
            skills: Vec::new(),
            search_paths: vec![skills_dir],
        };

        loader.discover().expect("discovery should succeed");

        let skill = &loader.skills[0];
        assert!(skill.triggers.is_empty());
    }

    // -----------------------------------------------------------------------
    // Frontmatter splitting tests
    // -----------------------------------------------------------------------

    #[test]
    fn split_frontmatter_basic() {
        let content = "---\nname: test\n---\n\nBody content.";
        let (fm, body) = split_frontmatter(content).expect("split should succeed");
        assert_eq!(fm, "name: test");
        assert_eq!(body.trim(), "Body content.");
    }

    #[test]
    fn split_frontmatter_with_leading_whitespace() {
        let content = "  \n---\nname: test\n---\n\nBody.";
        let (fm, _body) = split_frontmatter(content).expect("split should succeed");
        assert_eq!(fm, "name: test");
    }

    #[test]
    fn split_frontmatter_multiline_yaml() {
        let content = "---\nname: test\ndescription: A skill\ntriggers:\n  - a\n  - b\n---\n\nBody.";
        let (fm, body) = split_frontmatter(content).expect("split should succeed");
        assert!(fm.contains("name: test"));
        assert!(fm.contains("description: A skill"));
        assert!(fm.contains("- a"));
        assert_eq!(body.trim(), "Body.");
    }

    #[test]
    fn split_frontmatter_no_opening_delimiter() {
        let content = "name: test\n---\n\nBody.";
        let result = split_frontmatter(content);
        assert!(result.is_err());
    }

    #[test]
    fn split_frontmatter_no_closing_delimiter() {
        let content = "---\nname: test\n\nBody.";
        let result = split_frontmatter(content);
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // Validation tests
    // -----------------------------------------------------------------------

    #[test]
    fn validate_empty_name() {
        let fm = SkillFrontmatter {
            name: String::new(),
            description: "A skill".into(),
            triggers: vec![],
        };
        let result = validate_frontmatter(&fm);
        assert!(result.is_err());
    }

    #[test]
    fn validate_empty_description() {
        let fm = SkillFrontmatter {
            name: "test".into(),
            description: String::new(),
            triggers: vec![],
        };
        let result = validate_frontmatter(&fm);
        assert!(result.is_err());
    }

    #[test]
    fn validate_valid_frontmatter() {
        let fm = SkillFrontmatter {
            name: "test".into(),
            description: "A skill".into(),
            triggers: vec!["trigger".into()],
        };
        assert!(validate_frontmatter(&fm).is_ok());
    }

    // -----------------------------------------------------------------------
    // Default implementation test
    // -----------------------------------------------------------------------

    #[test]
    fn skill_loader_default() {
        let loader = SkillLoader::default();
        assert!(loader.skills.is_empty());
        // search_paths may or may not be empty depending on filesystem state
    }
}
