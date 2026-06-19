use crate::parser::{split_frontmatter, validate_frontmatter};
use crate::{Result, Skill, SkillError, SkillFrontmatter, SkillIndex, estimate_tokens};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

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
        let cwd = std::env::current_dir().ok();
        Self {
            skills: Vec::new(),
            search_paths: default_search_paths(cwd.as_deref()),
        }
    }

    /// Creates a new loader with search paths rooted at a specific workspace.
    ///
    /// Use this from runtime session startup instead of [`SkillLoader::new`]
    /// when the process current directory may differ from the active session
    /// workspace.
    pub fn for_workspace(workspace_root: impl AsRef<Path>) -> Self {
        Self {
            skills: Vec::new(),
            search_paths: default_search_paths(Some(workspace_root.as_ref())),
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
                            let _ = e;
                        }
                    }
                }
            }
        }

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
            .map(|s| {
                let level0_text = format!("{}: {}", s.name, s.description);
                SkillIndex {
                    name: s.name.clone(),
                    description: s.description.clone(),
                    triggers: s.triggers.clone(),
                    estimated_tokens: estimate_tokens(&level0_text),
                }
            })
            .collect()
    }
}

impl Default for SkillLoader {
    fn default() -> Self {
        Self::new()
    }
}

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

fn default_search_paths(workspace_root: Option<&Path>) -> Vec<PathBuf> {
    let mut search_paths = Vec::new();

    if let Some(root) = workspace_root {
        push_if_dir(&mut search_paths, root.join(".talos/skills"));
    }

    if let Some(home) = home_dir() {
        push_if_dir(&mut search_paths, home.join(".talos/skills"));
    }

    if let Some(root) = workspace_root {
        let mut current = root;
        while let Some(parent) = current.parent() {
            let git_dir = parent.join(".git");
            push_if_dir(&mut search_paths, parent.join(".talos/skills"));
            current = parent;
            if git_dir.is_dir() {
                break;
            }
        }
    }

    search_paths
}

fn push_if_dir(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if path.is_dir() && !paths.iter().any(|existing| existing == &path) {
        paths.push(path);
    }
}
