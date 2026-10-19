//! Prepared Pull integration stages.

use super::super as git_write;
use super::*;

impl AddPlan {
    pub(super) fn prepare_pull_integration_scope(
        mut self: Box<Self>,
    ) -> Result<Box<Self>, GitToolError> {
        self.revalidate()?;
        let integration = self.pull_integration.as_ref().ok_or_else(|| {
            GitToolError::InvalidInput("Missing retained Pull integration operation".into())
        })?;
        let fetched = self
            .pull_evidence
            .as_ref()
            .ok_or_else(|| {
                GitToolError::InvalidInput("Missing retained Pull integration evidence".into())
            })?
            .fetch_head()?;
        // Never display FETCH_HEAD contents: remote descriptions can contain
        // credentials. The digest binds the displayed plan to retained bytes.
        let digest = std::panic::catch_unwind(|| {
            let mut hasher = gix::hash::hasher(gix::hash::Kind::Sha256);
            hasher.update(fetched);
            hasher.try_finalize()
        })
        .map_err(|_| GitToolError::Git("Pull integration evidence digest failed".into()))?
        .map_err(|_| GitToolError::Git("Pull integration evidence digest failed".into()))?;
        let commands = integration.commands()?;
        let description = format!(
            "one Pull integration {integration:?}; commands={commands:?}; retained FETCH_HEAD sha256={digest}; no retry or rollback"
        );
        self.facets
            .retain(|facet| !matches!(facet.nature, ToolNature::Write | ToolNature::Network));
        for nature in [ToolNature::Read, ToolNature::Write] {
            self.facets
                .push(path_facet(nature, &self.repository.workdir, &description)?);
            self.facets.push(
                ToolPermissionFacet::with_resource(
                    nature,
                    "git:pull-integration-worktree",
                    ToolResourceKind::Remote,
                )
                .with_description(&description),
            );
            for root in [&self.repository.git_dir, &self.repository.common_dir] {
                self.facets.push(path_facet(
                    nature,
                    root,
                    "this Pull integration Git metadata root",
                )?);
            }
            self.facets.push(
                ToolPermissionFacet::with_resource(
                    nature,
                    "git:pull-repository-metadata",
                    ToolResourceKind::Remote,
                )
                .with_description("metadata for this integration and foreground maintenance only"),
            );
        }
        self.identity = PreparedPlanIdentity::new();
        self.phase = Phase::PullIntegrationMutation;
        Ok(self)
    }

    pub(super) fn validate_pull_integration_scope(
        &self,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<(), GitToolError> {
        self.revalidate()?;
        if self.phase != Phase::PullIntegrationMutation || self.pull_integration.is_none() {
            return Err(GitToolError::InvalidInput(
                "Pull integration stage missing".into(),
            ));
        }
        self.pull_evidence
            .as_ref()
            .ok_or_else(|| GitToolError::InvalidInput("Pull integration evidence missing".into()))?
            .fetch_head()?;
        // Called only after execute_stage consumes this immutable plan's proof.
        // Remote capabilities are retained facets, not general recursive Path
        // tokens. Every actual root still needs its own exact authorization.
        for nature in [ToolNature::Read, ToolNature::Write] {
            for scope in [
                "git:pull-integration-worktree",
                "git:pull-repository-metadata",
            ] {
                if !self.facets.iter().any(|facet| {
                    facet.nature == nature
                        && facet.resource_kind == Some(ToolResourceKind::Remote)
                        && facet.resource.as_deref() == Some(scope)
                }) {
                    return Err(GitToolError::InvalidInput(
                        "Pull integration capability missing".into(),
                    ));
                }
            }
            for root in [
                &self.repository.workdir,
                &self.repository.git_dir,
                &self.repository.common_dir,
            ] {
                if !authorizations.iter().any(|token| {
                    token.authorizes_path(
                        "git_pull",
                        nature,
                        &self.workspace,
                        root.to_str().unwrap_or_default(),
                    )
                }) {
                    return Err(GitToolError::InvalidInput(
                        "Pull integration root lacks fresh admission".into(),
                    ));
                }
            }
        }
        Ok(())
    }

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
            .pull_integration_hook_paths(
                &self.repository,
                &self.environment,
                integration,
                self.pull_before.as_ref().and_then(|before| before.branch()),
            )?;
        // Hook paths are known without opening them. Co-publish their exact
        // Read facets with the retained operation's integration scope rather
        // than requiring another approval just to assemble that same plan.
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
        self.prepare_pull_integration_scope()
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
        self.config
            .as_ref()
            .ok_or_else(|| {
                GitToolError::Git("Missing retained Pull integration configuration".into())
            })?
            .validate_pull_integration_sources(&self.repository, &self.pull_hook_paths)?;
        let evidence =
            git_write::config_state::ConfigState::capture_pull_hooks(&self.pull_hook_paths)?;
        self.pull_hook_evidence = evidence;
        self.revalidate()
    }
}
