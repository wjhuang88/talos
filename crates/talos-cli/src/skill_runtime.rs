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

/// Discovers skills for a concrete workspace using the system home directory.
///
/// Production entry point. Resolves the system home directory via `dirs::home_dir()`
/// and delegates to [`discover_runtime_skills_with_home`].
pub(crate) fn discover_runtime_skills(
    workspace_root: &Path,
    discover_shared: bool,
) -> Result<RuntimeSkills> {
    let home = dirs::home_dir();
    discover_runtime_skills_with_home(workspace_root, discover_shared, home.as_deref())
}

/// Discovers skills with an explicit home directory for test injection.
///
/// Does not read the `HOME` environment variable. When `home` is `None`, neither
/// user-global nor shared roots are added.
pub(crate) fn discover_runtime_skills_with_home(
    workspace_root: &Path,
    discover_shared: bool,
    home: Option<&Path>,
) -> Result<RuntimeSkills> {
    let mut loader = SkillLoader::for_workspace_with_home_and_options(
        workspace_root,
        home.map(|p| p.to_path_buf()),
        discover_shared,
    );
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
    use talos_config::Config;
    use talos_core::tool::ToolRegistry;
    use talos_provider::mock::MockProvider;

    fn write_skill(path: &Path, name: &str, description: &str) {
        fs::create_dir_all(path).expect("operation should succeed");
        fs::write(
            path.join("SKILL.md"),
            format!(
                "---\nname: {name}\ndescription: {description}\ntriggers:\n  - {name}\n---\n\n# {name}\n"
            ),
        )
        .expect("operation should succeed");
    }

    fn write_skill_with_body(path: &Path, name: &str, description: &str, body: &str) {
        fs::create_dir_all(path).expect("operation should succeed");
        fs::write(
            path.join("SKILL.md"),
            format!(
                "---\nname: {name}\ndescription: {description}\ntriggers:\n  - {name}\n---\n\n{body}\n"
            ),
        )
        .expect("operation should succeed");
    }

    #[test]
    fn discovers_workspace_skills_for_level0_index() {
        let dir = tempfile::tempdir().expect("operation should succeed");
        write_skill(
            &dir.path().join(".talos/skills/review"),
            "review",
            "Review code",
        );

        let runtime = discover_runtime_skills(dir.path(), false).expect("operation should succeed");

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
        let dir = tempfile::tempdir().expect("operation should succeed");
        write_skill(&dir.path().join(".talos/skills/ok"), "ok", "Valid skill");
        fs::create_dir_all(dir.path().join(".talos/skills/bad")).expect("operation should succeed");
        fs::write(
            dir.path().join(".talos/skills/bad/SKILL.md"),
            "not frontmatter",
        )
        .expect("operation should succeed");

        let runtime = discover_runtime_skills(dir.path(), false).expect("operation should succeed");

        assert!(runtime.index.iter().any(|skill| skill.name == "ok"));
        assert!(!runtime.index.iter().any(|skill| skill.name == "bad"));
    }

    #[test]
    fn apply_runtime_skills_reaches_agent_prompt() {
        let dir = tempfile::tempdir().expect("operation should succeed");
        write_skill(
            &dir.path().join(".talos/skills/planning"),
            "planning",
            "Plan work",
        );
        let runtime = discover_runtime_skills(dir.path(), false).expect("operation should succeed");

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
        let dir = tempfile::tempdir().expect("operation should succeed");
        write_skill(&dir.path().join(".talos/skills/doc"), "doc", "Write docs");

        let runtime = discover_runtime_skills(dir.path(), false).expect("operation should succeed");
        let index = runtime.diagnostics();

        assert!(
            index
                .iter()
                .any(|skill| skill.name == "doc" && skill.description == "Write docs")
        );
    }

    #[test]
    fn activate_skill_marks_diagnostic_active_and_returns_body_only_context() {
        let dir = tempfile::tempdir().expect("operation should succeed");
        write_skill_with_body(
            &dir.path().join(".talos/skills/review"),
            "review",
            "Review code",
            "# Instructions\nLook for security issues.",
        );

        let mut runtime =
            discover_runtime_skills(dir.path(), false).expect("operation should succeed");
        let content = runtime
            .activate("review")
            .expect("operation should succeed");
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
        let dir = tempfile::tempdir().expect("operation should succeed");
        write_skill(
            &dir.path().join(".talos/skills/review"),
            "review",
            "Review code",
        );

        let mut runtime =
            discover_runtime_skills(dir.path(), false).expect("operation should succeed");
        let error = runtime
            .activate("missing")
            .expect_err("operation should fail")
            .to_string();

        assert!(error.contains("skill 'missing' was not found"));
        assert!(runtime.active_name().is_none());
    }

    #[test]
    fn active_skill_reference_is_confined_and_bounded() {
        let dir = tempfile::tempdir().expect("operation should succeed");
        let skill_dir = dir.path().join(".talos/skills/review");
        write_skill(&skill_dir, "review", "Review code");
        fs::write(skill_dir.join("guide.md"), "reference details")
            .expect("operation should succeed");

        let mut runtime =
            discover_runtime_skills(dir.path(), false).expect("operation should succeed");
        runtime
            .activate("review")
            .expect("operation should succeed");
        let content = runtime
            .load_reference("guide.md")
            .expect("operation should succeed");

        assert!(content.contains("## Reference: guide.md"));
        assert!(content.contains("reference details"));

        let parent_error = runtime
            .load_reference("../guide.md")
            .expect_err("operation should fail")
            .to_string();
        assert!(parent_error.contains("reference path must stay inside"));
        let absolute_error = runtime
            .load_reference(
                skill_dir
                    .join("guide.md")
                    .to_str()
                    .expect("operation should succeed"),
            )
            .expect_err("operation should fail")
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

    #[test]
    fn application_default_adds_shared_skill_root() {
        let temp_home = tempfile::tempdir().expect("operation should succeed");
        let shared_skills = temp_home.path().join(".agents/skills");
        fs::create_dir_all(&shared_skills).expect("operation should succeed");

        let workspace = tempfile::tempdir().expect("operation should succeed");
        let config = Config::default();
        assert!(config.skills.discover_shared);

        let runtime = discover_runtime_skills_with_home(
            workspace.path(),
            config.skills.discover_shared,
            Some(temp_home.path()),
        )
        .expect("operation should succeed");
        assert!(
            runtime.search_paths().iter().any(|p| p == &shared_skills),
            "shared skill root must be in search paths when application default is used"
        );

        let last = runtime
            .search_paths()
            .last()
            .expect("operation should succeed");
        assert_eq!(
            last, &shared_skills,
            "shared root must be lowest priority (last)"
        );
    }

    #[test]
    fn application_explicit_false_excludes_shared_skill_root() {
        let temp_home = tempfile::tempdir().expect("operation should succeed");
        let shared_skills = temp_home.path().join(".agents/skills");
        fs::create_dir_all(&shared_skills).expect("operation should succeed");

        let workspace = tempfile::tempdir().expect("operation should succeed");
        let config = Config {
            skills: talos_config::SkillConfig {
                discover_shared: false,
            },
            ..Default::default()
        };
        assert!(!config.skills.discover_shared);

        let runtime = discover_runtime_skills_with_home(
            workspace.path(),
            config.skills.discover_shared,
            Some(temp_home.path()),
        )
        .expect("operation should succeed");
        assert!(
            !runtime.search_paths().iter().any(|p| p == &shared_skills),
            "shared skill root must NOT be in search paths when explicitly disabled"
        );
    }

    #[test]
    fn shared_skill_is_lowest_priority_end_to_end() {
        let temp_home = tempfile::tempdir().expect("operation should succeed");
        let shared_dir = temp_home.path().join(".agents/skills/dup-skill");
        write_skill(&shared_dir, "dup-skill", "Shared version");

        let workspace = tempfile::tempdir().expect("operation should succeed");
        let proj_dir = workspace.path().join(".talos/skills/dup-skill");
        write_skill(&proj_dir, "dup-skill", "Project version");

        let config = Config::default();
        let runtime = discover_runtime_skills_with_home(
            workspace.path(),
            config.skills.discover_shared,
            Some(temp_home.path()),
        )
        .expect("operation should succeed");

        let dup: Vec<_> = runtime
            .index
            .iter()
            .filter(|s| s.name == "dup-skill")
            .collect();
        assert_eq!(dup.len(), 1, "dup-skill should appear exactly once");
        assert_eq!(
            dup[0].description, "Project version",
            "workspace skill must shadow shared skill"
        );
        assert_eq!(dup[0].source.to_string(), "project");
    }

    #[test]
    fn application_without_home_does_not_add_shared_root() {
        let workspace = tempfile::tempdir().expect("operation should succeed");
        let proj_dir = workspace.path().join(".talos/skills/proj-skill");
        write_skill(&proj_dir, "proj-skill", "Project only");

        let runtime = discover_runtime_skills_with_home(workspace.path(), true, None)
            .expect("operation should succeed");
        assert!(
            !runtime
                .search_paths()
                .iter()
                .any(|p| p.ends_with(".agents/skills")),
            "no shared root when home is None"
        );
        assert!(
            runtime.index.iter().any(|s| s.name == "proj-skill"),
            "workspace skill still discovered without home"
        );
    }

    #[test]
    fn explicit_home_is_used_instead_of_process_environment() {
        let home_a = tempfile::tempdir().expect("operation should succeed");
        let shared_a = home_a.path().join(".agents/skills/skill-a");
        write_skill(&shared_a, "skill-a", "From home A");

        let home_b = tempfile::tempdir().expect("operation should succeed");
        let shared_b = home_b.path().join(".agents/skills/skill-b");
        write_skill(&shared_b, "skill-b", "From home B");

        let workspace = tempfile::tempdir().expect("operation should succeed");
        let runtime =
            discover_runtime_skills_with_home(workspace.path(), true, Some(home_a.path()))
                .expect("operation should succeed");

        assert!(
            runtime.index.iter().any(|s| s.name == "skill-a"),
            "skill from injected home A must be discovered"
        );
        assert!(
            !runtime.index.iter().any(|s| s.name == "skill-b"),
            "skill from home B must NOT be discovered"
        );
    }

    #[test]
    fn discover_runtime_skills_delegates_to_with_home() {
        let workspace = tempfile::tempdir().expect("operation should succeed");
        let proj_dir = workspace.path().join(".talos/skills/proj-skill");
        write_skill(&proj_dir, "proj-skill", "Project skill");

        let runtime =
            discover_runtime_skills(workspace.path(), false).expect("operation should succeed");
        assert!(runtime.index.iter().any(|s| s.name == "proj-skill"));
    }
}
