use crate::{Result, Skill, SkillError, SkillIndex, SkillLoader, estimate_tokens};
use std::collections::HashMap;
use std::path::PathBuf;

/// Manages progressive disclosure of skills across three levels.
///
/// `SkillManager` wraps a [`SkillLoader`] and provides on-demand loading
/// of skill content. Level 0 (index) is always available; Level 1 (full body)
/// and Level 2 (reference files) are loaded as needed.
///
/// # Example
///
/// ```no_run
/// use talos_skill::{SkillLoader, SkillManager};
///
/// let loader = SkillLoader::new();
/// let mut manager = SkillManager::new(loader);
/// let index = manager.get_index();
/// ```
pub struct SkillManager {
    /// Underlying skill loader for discovery and parsing.
    loader: SkillLoader,
    /// Skills loaded at Level 1 or Level 2, keyed by name.
    active_skills: HashMap<String, Skill>,
    /// Level 0 index entries, computed on demand.
    skill_index: Vec<SkillIndex>,
}

impl SkillManager {
    /// Creates a new `SkillManager` wrapping the given [`SkillLoader`].
    ///
    /// The loader should already have discovered skills via [`SkillLoader::discover`]
    /// before being passed to this constructor.
    pub fn new(loader: SkillLoader) -> Self {
        Self {
            loader,
            active_skills: HashMap::new(),
            skill_index: Vec::new(),
        }
    }

    /// Returns the Level 0 index of all discovered skills.
    ///
    /// The index is computed lazily on first call and cached. Subsequent calls
    /// return the cached index unless the underlying loader's skills have changed.
    pub fn get_index(&mut self) -> &[SkillIndex] {
        if self.skill_index.is_empty() && !self.loader.skills.is_empty() {
            self.skill_index = self
                .loader
                .skills
                .iter()
                .map(|s| {
                    let level0_text = format!("{}: {}", s.name, s.description);
                    let estimated_tokens = estimate_tokens(&level0_text);
                    SkillIndex {
                        name: s.name.clone(),
                        description: s.description.clone(),
                        triggers: s.triggers.clone(),
                        estimated_tokens,
                        source: s.source,
                    }
                })
                .collect();
        }
        &self.skill_index
    }

    /// Returns the total estimated token count for the Level 0 index.
    ///
    /// Target: <3000 tokens for 20 skills (~150 tokens per skill).
    pub fn get_index_tokens(&mut self) -> usize {
        self.get_index().iter().map(|e| e.estimated_tokens).sum()
    }

    /// Loads a full skill into active memory (Level 1 disclosure).
    ///
    /// Searches the loader's discovered skills by name. If found, clones the
    /// skill into the active set. Returns a reference to the loaded skill.
    ///
    /// # Errors
    ///
    /// Returns [`SkillError::FileNotFound`] if no skill with the given name
    /// exists in the loader's discovered skills.
    pub fn load_skill(&mut self, name: &str) -> Result<&Skill> {
        if self.active_skills.contains_key(name) {
            return Ok(self.active_skills.get(name).expect("key exists"));
        }

        let skill = self
            .loader
            .skills
            .iter()
            .find(|s| s.name == name)
            .ok_or_else(|| {
                SkillError::FileNotFound(PathBuf::from(format!("skill '{name}' not found")))
            })?;

        let skill = skill.clone();
        self.active_skills.insert(name.to_string(), skill);
        Ok(self.active_skills.get(name).expect("key just inserted"))
    }

    /// Loads a specific reference file from a skill (Level 2 disclosure).
    ///
    /// Reads the file at `file_path` relative to the skill's source directory.
    /// The skill must already be loaded at Level 1 (via [`load_skill`]).
    ///
    /// # Errors
    ///
    /// Returns [`SkillError::FileNotFound`] if the skill is not loaded or
    /// the reference file does not exist.
    pub fn load_reference(&self, skill_name: &str, file_path: &str) -> Result<String> {
        let skill = self.active_skills.get(skill_name).ok_or_else(|| {
            SkillError::FileNotFound(PathBuf::from(format!(
                "skill '{skill_name}' not loaded (call load_skill first)"
            )))
        })?;

        let skill_dir = skill.source_path.parent().ok_or_else(|| {
            SkillError::InvalidFrontmatter("skill has no parent directory".into())
        })?;

        let ref_path = skill_dir.join(file_path);
        if !ref_path.exists() {
            return Err(SkillError::FileNotFound(ref_path));
        }

        std::fs::read_to_string(&ref_path).map_err(SkillError::IoError)
    }

    /// Matches a task description to a skill based on trigger keywords.
    ///
    /// Returns the name of the first skill whose triggers match the task
    /// description (case-insensitive substring match). If multiple skills
    /// match, the first one in discovery order is returned.
    pub fn match_skill(&self, task_description: &str) -> Option<String> {
        let task_lower = task_description.to_lowercase();

        self.loader
            .skills
            .iter()
            .find(|skill| {
                skill.triggers.iter().any(|trigger| {
                    let trigger_lower = trigger.to_lowercase();
                    task_lower.contains(&trigger_lower)
                })
            })
            .map(|s| s.name.clone())
    }

    /// Removes a skill from the active set.
    ///
    /// This does not affect the Level 0 index or the loader's discovered skills.
    /// The skill can be reloaded via [`load_skill`].
    pub fn unload_skill(&mut self, name: &str) {
        self.active_skills.remove(name);
    }

    /// Returns references to all currently active skills.
    pub fn get_active_skills(&self) -> Vec<&Skill> {
        self.active_skills.values().collect()
    }
}
