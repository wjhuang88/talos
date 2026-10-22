//! Final native Pull execution. Discovery must supply a complete immutable scope.

use super::*;

impl AddPlan {
    // Resolve against retained, revalidated configuration and the current HEAD.
    // This is a local scalar probe, never a transport or fetch invocation.
    pub(super) async fn validate_native_pull_destination(&self) -> Result<(), GitToolError> {
        let Operation::Pull(input) = &self.operation else {
            return Err(GitToolError::InvalidInput(
                "Native Pull operation missing".into(),
            ));
        };
        let config = self.config.as_ref().ok_or_else(|| {
            GitToolError::InvalidInput("Native Pull configuration evidence missing".into())
        })?;
        let head = self
            .pull_probe(vec!["symbolic-ref".into(), "--quiet".into(), "HEAD".into()])
            .await?;
        let branch = match head.status.code() {
            Some(0) if head.stderr.is_empty() => Some(
                std::str::from_utf8(&head.stdout)
                    .ok()
                    .and_then(|text| text.strip_suffix('\n'))
                    .and_then(|text| text.strip_prefix("refs/heads/"))
                    .filter(|text| !text.is_empty() && !text.contains(['\n', '\r']))
                    .ok_or_else(|| {
                        GitToolError::InvalidInput("Native Pull branch evidence invalid".into())
                    })?,
            ),
            Some(1) if head.stdout.is_empty() && head.stderr.is_empty() => None,
            _ => {
                return Err(GitToolError::InvalidInput(
                    "Native Pull branch inspection failed".into(),
                ));
            }
        };
        let remote = super::super::pull_state::fetch_remote(
            &config.pull_records()?,
            branch,
            input.remote.as_deref(),
        )?;
        config.prepare_pull_transport(&remote)?;
        let destinations = super::super::remote_state::destinations(
            &config.pull_remote_records()?,
            &remote,
            false,
        )?;
        if self.pull_remote.as_deref() != Some(remote.as_str())
            || destinations != self.pull_destinations
        {
            return Err(GitToolError::InvalidInput(
                "Native Pull destination differs from admitted scope; no Pull started".into(),
            ));
        }
        self.revalidate()
    }

    // Only the stage dispatcher calls this after consuming this exact plan's
    // one-use proof. Path tokens alone cannot confer the named recursive scopes.
    pub(super) async fn execute_native_pull(
        &self,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<String, GitToolError> {
        self.revalidate()?;
        if self.phase != Phase::PullNativeMutation || !matches!(self.operation, Operation::Pull(_))
        {
            return Err(GitToolError::InvalidInput(
                "Native Pull stage missing".into(),
            ));
        }
        for nature in [ToolNature::Read, ToolNature::Write] {
            for capability in [
                "git:pull-integration-worktree",
                "git:pull-repository-metadata",
            ] {
                if !self.facets.iter().any(|facet| {
                    facet.nature == nature
                        && facet.resource_kind == Some(ToolResourceKind::Remote)
                        && facet.resource.as_deref() == Some(capability)
                }) {
                    return Err(GitToolError::InvalidInput(
                        "Native Pull scope capability missing".into(),
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
                        "Native Pull root lacks fresh admission".into(),
                    ));
                }
            }
        }
        if !self.pull_transport_supported
            || self.pull_destinations.is_empty()
            || self.pull_destinations.iter().any(|destination| {
                !self.facets.iter().any(|facet| {
                    facet.nature == ToolNature::Network
                        && facet.resource_kind == Some(ToolResourceKind::Remote)
                        && facet.resource.as_deref() == Some(destination)
                })
            })
        {
            return Err(GitToolError::InvalidInput(
                "Native Pull transport scope is incomplete".into(),
            ));
        }
        self.validate_native_pull_destination().await?;
        // The phase is deliberately unreachable from production discovery until
        // native recursive/external-effect scope discovery and activation tests
        // replace the old scheduler. Do not promote a fetch-only plan here.
        if !cfg!(test) {
            return Err(GitToolError::InvalidInput(
                "Native Pull awaits complete effect discovery and activation evidence; no command started".into(),
            ));
        }
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
