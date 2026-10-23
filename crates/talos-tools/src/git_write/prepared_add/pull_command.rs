//! One bounded configuration discovery followed by native command authority.
//! The stage dispatcher consumes the exact prepared proof before either entry.

use super::*;

impl AddPlan {
    pub(super) async fn discover_pull_command(
        mut self: Box<Self>,
    ) -> Result<Box<Self>, GitToolError> {
        self.revalidate()?;
        let output = self.inspect_config().await?;
        let config = super::super::config_state::ConfigState::capture_pull_command(
            &output,
            &self.repository,
            &self.environment,
        )?;
        config.verify_host(&self.inspect_config().await?)?;
        let configured_programs = config.pull_command_program_categories();
        let remote_hosts = config.pull_command_remote_hosts();
        self.config = Some(config);
        // These roots identify the invocation and provide review evidence.
        // They are not an exhaustive path confinement promise for helpers.
        self.facets.clear();
        for root in [
            &self.repository.workdir,
            &self.repository.git_dir,
            &self.repository.common_dir,
        ] {
            self.facets.push(path_facet(
                ToolNature::Read,
                root,
                "known native Pull repository root; command effects are not confined to this path",
            )?);
        }
        self.facets.push(ToolPermissionFacet::with_resource(
            ToolNature::Execute,
            self.program.to_str().ok_or_else(|| GitToolError::InvalidInput("Git program path is not UTF-8".into()))?,
            ToolResourceKind::Command,
        ).with_description("frozen argv: -c maintenance.autoDetach=false -c gc.autoDetach=false --no-pager pull; optional -- remote is retained from the invocation input; retained environment; Trace2 disabled"));
        self.facets.push(ToolPermissionFacet::with_resource(
            ToolNature::Execute, "git:pull-command", ToolResourceKind::Remote,
        ).with_description("one native Pull command and its transitive network, worktree, metadata, recursion, hooks, filters, credential/transport helpers and other configured programs; known roots are review evidence, not path or endpoint confinement; no retry, reset or automatic conflict repair"));
        if let Some(facet) = self.facets.last_mut() {
            facet.description = Some(format!(
                "{}; configured program categories: {} (values withheld; dynamically discovered programs remain included); configured remote hosts: {} (not exhaustive; local paths and helper payloads withheld)",
                facet.description.as_deref().unwrap_or_default(),
                configured_programs.join(", "),
                if remote_hosts.is_empty() {
                    "none displayed".into()
                } else {
                    remote_hosts.join(", ")
                }
            ));
        }
        self.phase = Phase::PullCommandMutation;
        self.identity = PreparedPlanIdentity::new();
        self.revalidate()?;
        Ok(self)
    }

    pub(super) async fn execute_pull_command(&self) -> Result<String, GitToolError> {
        self.revalidate()?;
        if self.phase != Phase::PullCommandMutation
            || !matches!(self.operation, Operation::Pull(_))
            || !self.facets.iter().any(|facet| {
                facet.nature == ToolNature::Execute
                    && facet.resource_kind == Some(ToolResourceKind::Remote)
                    && facet.resource.as_deref() == Some("git:pull-command")
            })
        {
            return Err(GitToolError::InvalidInput(
                "Native Pull command authority missing; no command started".into(),
            ));
        }
        self.config
            .as_ref()
            .ok_or_else(|| {
                GitToolError::InvalidInput("Native Pull configuration evidence missing".into())
            })?
            .verify_host(&self.inspect_config().await?)?;
        self.revalidate()?;
        self.run_native_pull_command().await
    }

    // Both routes use the same supervised command/result boundary. Callers
    // must consume their prepared proof and validate their authority first.
    pub(super) async fn run_native_pull_command(&self) -> Result<String, GitToolError> {
        let command = self
            .native_pull_command
            .as_ref()
            .ok_or_else(|| GitToolError::InvalidInput("Native Pull command missing".into()))?;
        let output = self.run_command_output(
            command.command(), std::time::Duration::from_secs(120), 1024 * 1024,
        ).await.map_err(|error| GitToolError::Git(format!(
            "Native Pull execution failed: {error}; partial effects may remain; inspect repositories before retrying",
        )))?;
        if !output.status.success() {
            // No retry, reset, abort or automatic conflict resolution. Avoid
            // reflecting diagnostic output which may contain remote credentials.
            return Err(GitToolError::Git(format!(
                "Native Pull failed ({}); fetched refs, conflicts or partial effects may remain; inspect repositories before retrying",
                output.status,
            )));
        }
        self.repository.revalidate_location().map_err(|error| GitToolError::Git(format!(
            "Native Pull completed but repository identity verification failed: {error}; effects are retained",
        )))?;
        Ok(
            "Native Pull completed; Git retained its configured integration and recursion behavior"
                .into(),
        )
    }
}
