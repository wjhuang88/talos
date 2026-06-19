//! Runtime skill discovery and session prompt wiring.

use std::path::{Path, PathBuf};

use anyhow::Result;
use talos_agent::Agent;
use talos_conversation::SkillDiagnostic;
use talos_skill::{SkillIndex, SkillLoader, SkillManager};

/// Skill metadata discovered for a runtime session.
pub(crate) struct RuntimeSkills {
    index: Vec<SkillIndex>,
    search_paths: Vec<PathBuf>,
    index_tokens: usize,
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
                active: false,
            })
            .collect()
    }
}

/// Discovers skills for a concrete workspace.
///
/// Invalid skill files are skipped by `SkillLoader`; duplicate names keep the
/// first match according to search-path priority.
pub(crate) fn discover_runtime_skills(workspace_root: &Path) -> Result<RuntimeSkills> {
    let mut loader = SkillLoader::for_workspace(workspace_root);
    let search_paths = loader.search_paths.clone();
    loader.discover()?;

    let mut manager = SkillManager::new(loader);
    let index = manager.get_index().to_vec();
    let index_tokens = manager.get_index_tokens();

    Ok(RuntimeSkills {
        index,
        search_paths,
        index_tokens,
    })
}

/// Injects runtime-discovered Level 0 skills into the agent prompt.
pub(crate) fn apply_runtime_skills(agent: &mut Agent, runtime_skills: &RuntimeSkills) {
    agent.set_skill_index(runtime_skills.index.clone());
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

    #[test]
    fn discovers_workspace_skills_for_level0_index() {
        let dir = tempfile::tempdir().unwrap();
        write_skill(
            &dir.path().join(".talos/skills/review"),
            "review",
            "Review code",
        );

        let runtime = discover_runtime_skills(dir.path()).unwrap();

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

        let runtime = discover_runtime_skills(dir.path()).unwrap();

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
        let runtime = discover_runtime_skills(dir.path()).unwrap();

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

        let runtime = discover_runtime_skills(dir.path()).unwrap();
        let index = runtime.diagnostics();

        assert!(
            index
                .iter()
                .any(|skill| skill.name == "doc" && skill.description == "Write docs")
        );
    }
}
