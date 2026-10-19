//! Prepared Pull integration stages.

use super::super as git_write;
use super::*;

impl AddPlan {
    pub(super) fn discover_pull_integration_hooks(
        mut self: Box<Self>,
    ) -> Result<Box<Self>, GitToolError> {
        self.revalidate()?;
        let integration = self
            .pull_integration
            .as_ref()
            .ok_or_else(|| GitToolError::Git("Missing retained Pull integration plan".into()))?;
        let paths = self
            .config
            .as_ref()
            .ok_or_else(|| {
                GitToolError::Git("Missing retained Pull integration configuration".into())
            })?
            .pull_integration_hook_paths(&self.repository, &self.environment, integration)?;
        // Drop fetch/child Write and Network facets. This stage admits only
        // integration hook reads, not integration mutation or hook execution.
        self.facets
            .retain(|facet| !matches!(facet.nature, ToolNature::Write | ToolNature::Network));
        for path in &paths {
            self.facets.push(path_facet(
                ToolNature::Read,
                path,
                "Pull integration hook candidate; content inspection requires this new stage",
            )?);
        }
        self.pull_hook_paths = paths;
        self.pull_hook_evidence.clear();
        self.identity = PreparedPlanIdentity::new();
        self.phase = Phase::PullIntegrationHooks;
        Ok(self)
    }

    pub(super) fn capture_pull_integration_hooks(
        &mut self,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<(), GitToolError> {
        self.revalidate()?;
        // Validate the complete frontier before opening any candidate.
        for path in &self.pull_hook_paths {
            if !authorizations.iter().any(|token| {
                token.authorizes_path(
                    "git_pull",
                    ToolNature::Read,
                    &self.workspace,
                    path.to_str().unwrap_or_default(),
                )
            }) {
                return Err(GitToolError::InvalidInput(
                    "Pull integration hook frontier lacks fresh Read admission".into(),
                ));
            }
        }
        let evidence =
            git_write::config_state::ConfigState::capture_pull_hooks(&self.pull_hook_paths)?;
        self.pull_hook_evidence = evidence;
        self.revalidate()
    }
}
