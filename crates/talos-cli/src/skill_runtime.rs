//! Runtime skill discovery and session prompt wiring.

use std::path::{Component, Path, PathBuf};

use anyhow::{Result, anyhow, bail};
use talos_agent::Agent;
use talos_conversation::SkillDiagnostic;
use talos_skill::{SkillIndex, SkillLoader, SkillManager};

const MAX_SKILL_BODY_BYTES: usize = 24 * 1024;
const MAX_SKILL_REFERENCE_BYTES: usize = 16 * 1024;

/// Skill metadata discovered for a runtime session.
pub(crate) struct RuntimeSkills {
    index: Vec<SkillIndex>,
    search_paths: Vec<PathBuf>,
    index_tokens: usize,
    manager: SkillManager,
    active_name: Option<String>,
    activated_content: Option<String>,
    loaded_references: Vec<String>,
}

impl RuntimeSkills {
    /// Returns the configured skill search paths that existed at startup.
    #[allow(dead_code)]
    pub(crate) fn search_paths(&self) -> &[PathBuf] {
        &self.search_paths
    }

    /// Returns the estimated token count of the Level 0 skill index.
    #[allow(dead_code)]
    pub(crate) fn index_tokens(&self) -> usize {
        self.index_tokens
    }

    /// Converts Level 0 metadata into conversation diagnostics.
    pub(crate) fn diagnostics(&self) -> Vec<SkillDiagnostic> {
        self.index
            .iter()
            .map(|skill| SkillDiagnostic {
                name: skill.name.clone(),
                description: skill.description.clone(),
                active: self.active_name.as_deref() == Some(skill.name.as_str()),
                source: skill.source.to_string(),
            })
            .collect()
    }

    pub(crate) fn active_name(&self) -> Option<&str> {
        self.active_name.as_deref()
    }

    /// Activates a Skill body and returns bounded model-visible content.
    pub(crate) fn activate(&mut self, name: &str) -> Result<String> {
        let skill = self
            .manager
            .load_skill(name)
            .map_err(|_| anyhow!("skill '{name}' was not found"))?;
        let body = bounded_text(&skill.body, MAX_SKILL_BODY_BYTES);
        let content = format!("## Skill Body\n{body}\n");
        self.active_name = Some(skill.name.clone());
        self.activated_content = Some(content.clone());
        self.loaded_references.clear();
        Ok(content)
    }

    /// Loads a bounded reference for the active Skill and returns combined context.
    pub(crate) fn load_reference(&mut self, relative_path: &str) -> Result<String> {
        let active = self
            .active_name
            .clone()
            .ok_or_else(|| anyhow!("activate a skill before loading references"))?;
        let skill = self
            .manager
            .get_active_skills()
            .into_iter()
            .find(|skill| skill.name == active)
            .ok_or_else(|| anyhow!("active skill '{active}' is not loaded"))?;
        let reference = read_confined_reference(skill.source_path.as_path(), relative_path)?;
        let reference = bounded_text(&reference, MAX_SKILL_REFERENCE_BYTES);
        let base = self.activated_content.clone().unwrap_or_default();
        let display_path = relative_path.trim();
        let combined = format!("{base}\n## Reference: {display_path}\n{reference}\n");
        self.activated_content = Some(combined.clone());
        self.loaded_references.push(display_path.to_string());
        Ok(combined)
    }
}

/// Discovers skills for a concrete workspace.
///
/// Invalid skill files are skipped by `SkillLoader`; duplicate names keep the
/// first match according to search-path priority.
pub(crate) fn discover_runtime_skills(
    workspace_root: &Path,
    discover_shared: bool,
) -> Result<RuntimeSkills> {
    let mut loader = SkillLoader::for_workspace_with_options(workspace_root, discover_shared);
    let search_paths = loader.search_paths.clone();
    loader.discover()?;

    let mut manager = SkillManager::new(loader);
    let index = manager.get_index().to_vec();
    let index_tokens = manager.get_index_tokens();

    Ok(RuntimeSkills {
        index,
        search_paths,
        index_tokens,
        manager,
        active_name: None,
        activated_content: None,
        loaded_references: Vec::new(),
    })
}

/// Injects runtime-discovered Level 0 skills into the agent prompt.
pub(crate) fn apply_runtime_skills(agent: &mut Agent, runtime_skills: &RuntimeSkills) {
    agent.set_skill_index(runtime_skills.index.clone());
    if let (Some(name), Some(content)) = (
        runtime_skills.active_name.as_deref(),
        runtime_skills.activated_content.as_deref(),
    ) {
        agent.set_activated_skill_context(Some(talos_agent::ActivatedSkillContext {
            name: name.to_string(),
            content: content.to_string(),
        }));
    }
}

fn bounded_text(text: &str, max_bytes: usize) -> String {
    if text.len() <= max_bytes {
        return text.to_string();
    }
    let mut end = 0usize;
    for (idx, ch) in text.char_indices() {
        let next = idx + ch.len_utf8();
        if next > max_bytes {
            break;
        }
        end = next;
    }
    format!(
        "{}\n\n[truncated: original_bytes={}, kept_bytes={}]",
        &text[..end],
        text.len(),
        end
    )
}

fn read_confined_reference(skill_path: &Path, relative_path: &str) -> Result<String> {
    let rel = Path::new(relative_path.trim());
    if rel.as_os_str().is_empty() {
        bail!("reference path is required");
    }
    if rel.is_absolute()
        || rel
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::Prefix(_)))
    {
        bail!("reference path must stay inside the active skill directory");
    }

    let skill_dir = skill_path
        .parent()
        .ok_or_else(|| anyhow!("skill has no parent directory"))?;
    let canonical_dir = skill_dir.canonicalize()?;
    let candidate = canonical_dir.join(rel);
    let canonical_candidate = candidate.canonicalize()?;
    if !canonical_candidate.starts_with(&canonical_dir) {
        bail!("reference path escapes the active skill directory");
    }
    std::fs::read_to_string(canonical_candidate).map_err(Into::into)
}

#[cfg(test)]
#[allow(warnings)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Arc;
    use talos_agent::Agent;
    use talos_core::tool::ToolRegistry;
    use talos_provider::mock::MockProvider;

    fn write_skill(path: &Path, name: &str, description: &str) {
        fs::create_dir_all(path).unwrap();
        fs::write(
            path.join("SKILL.md"),
            format!(
                "---\nname: {name}\ndescription: {description}\ntriggers:\n  - {name}\n---\n\n# {name}\n"
            ),
        )
        .unwrap();
    }

    fn write_skill_with_body(path: &Path, name: &str, description: &str, body: &str) {
        fs::create_dir_all(path).unwrap();
        fs::write(
            path.join("SKILL.md"),
            format!(
                "---\nname: {name}\ndescription: {description}\ntriggers:\n  - {name}\n---\n\n{body}\n"
            ),
        )
        .unwrap();
    }

    #[test]
    fn discovers_workspace_skills_for_level0_index() {
        let dir = tempfile::tempdir().unwrap();
        write_skill(
            &dir.path().join(".talos/skills/review"),
            "review",
            "Review code",
        );

        let runtime = discover_runtime_skills(dir.path(), false).unwrap();

        let skill = runtime
            .index
            .iter()
            .find(|skill| skill.name == "review")
            .expect("workspace skill should be discovered");
        assert!(skill.estimated_tokens > 0);
        assert!(runtime.index_tokens() > 0);
        assert!(
            runtime
                .search_paths()
                .iter()
                .any(|path| path.ends_with(".talos/skills"))
        );
    }

    #[test]
    fn bad_skills_are_skipped_without_crashing_startup() {
        let dir = tempfile::tempdir().unwrap();
        write_skill(&dir.path().join(".talos/skills/ok"), "ok", "Valid skill");
        fs::create_dir_all(dir.path().join(".talos/skills/bad")).unwrap();
        fs::write(
            dir.path().join(".talos/skills/bad/SKILL.md"),
            "not frontmatter",
        )
        .unwrap();

        let runtime = discover_runtime_skills(dir.path(), false).unwrap();

        assert!(runtime.index.iter().any(|skill| skill.name == "ok"));
        assert!(!runtime.index.iter().any(|skill| skill.name == "bad"));
    }

    #[test]
    fn apply_runtime_skills_reaches_agent_prompt() {
        let dir = tempfile::tempdir().unwrap();
        write_skill(
            &dir.path().join(".talos/skills/planning"),
            "planning",
            "Plan work",
        );
        let runtime = discover_runtime_skills(dir.path(), false).unwrap();

        let mut agent = Agent::new(
            Arc::new(MockProvider::new().with_response("ok")),
            ToolRegistry::new(),
        );
        apply_runtime_skills(&mut agent, &runtime);

        let prompt = agent.build_system_prompt();
        assert!(prompt.contains("# Skills"));
        assert!(prompt.contains("planning"));
        assert!(prompt.contains("Plan work"));
    }

    #[test]
    fn diagnostic_index_contains_level0_metadata() {
        let dir = tempfile::tempdir().unwrap();
        write_skill(&dir.path().join(".talos/skills/doc"), "doc", "Write docs");

        let runtime = discover_runtime_skills(dir.path(), false).unwrap();
        let index = runtime.diagnostics();

        assert!(
            index
                .iter()
                .any(|skill| skill.name == "doc" && skill.description == "Write docs")
        );
    }

    #[test]
    fn activate_skill_marks_diagnostic_active_and_returns_body_only_context() {
        let dir = tempfile::tempdir().unwrap();
        write_skill_with_body(
            &dir.path().join(".talos/skills/review"),
            "review",
            "Review code",
            "# Instructions\nLook for security issues.",
        );

        let mut runtime = discover_runtime_skills(dir.path(), false).unwrap();
        let content = runtime.activate("review").unwrap();
        let diagnostics = runtime.diagnostics();

        assert!(content.contains("## Skill Body"));
        assert!(content.contains("Look for security issues."));
        assert!(
            diagnostics
                .iter()
                .any(|skill| skill.name == "review" && skill.active)
        );
    }

    #[test]
    fn unknown_skill_activation_is_deterministic_error() {
        let dir = tempfile::tempdir().unwrap();
        write_skill(
            &dir.path().join(".talos/skills/review"),
            "review",
            "Review code",
        );

        let mut runtime = discover_runtime_skills(dir.path(), false).unwrap();
        let error = runtime.activate("missing").unwrap_err().to_string();

        assert!(error.contains("skill 'missing' was not found"));
        assert!(runtime.active_name().is_none());
    }

    #[test]
    fn active_skill_reference_is_confined_and_bounded() {
        let dir = tempfile::tempdir().unwrap();
        let skill_dir = dir.path().join(".talos/skills/review");
        write_skill(&skill_dir, "review", "Review code");
        fs::write(skill_dir.join("guide.md"), "reference details").unwrap();

        let mut runtime = discover_runtime_skills(dir.path(), false).unwrap();
        runtime.activate("review").unwrap();
        let content = runtime.load_reference("guide.md").unwrap();

        assert!(content.contains("## Reference: guide.md"));
        assert!(content.contains("reference details"));

        let parent_error = runtime
            .load_reference("../guide.md")
            .unwrap_err()
            .to_string();
        assert!(parent_error.contains("reference path must stay inside"));
        let absolute_error = runtime
            .load_reference(skill_dir.join("guide.md").to_str().unwrap())
            .unwrap_err()
            .to_string();
        assert!(absolute_error.contains("reference path must stay inside"));
    }

    #[test]
    fn bounded_text_truncates_on_utf8_boundary() {
        let text = "你好世界abcdef";
        let bounded = bounded_text(text, 7);

        assert!(bounded.starts_with("你好"));
        assert!(bounded.contains("[truncated:"));
    }
}
