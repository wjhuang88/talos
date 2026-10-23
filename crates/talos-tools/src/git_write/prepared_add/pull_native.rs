//! Final native Pull execution. Discovery must supply a complete immutable scope.

use super::*;

impl AddPlan {
    pub(super) fn native_pull_source_facet(
        destination: &str,
        execution_dir: &Path,
    ) -> Result<ToolPermissionFacet, GitToolError> {
        if let Some(path) =
            super::super::remote_state::local_destination(destination, execution_dir)?
        {
            path_facet(
                ToolNature::Read,
                &path,
                "native Pull configured local source; source contents and effects deferred",
            )
        } else {
            Ok(ToolPermissionFacet::with_resource(
                ToolNature::Network,
                destination,
                ToolResourceKind::Remote,
            ))
        }
    }

    // Shared by scope publication and execution checks. This describes only
    // the parent scope; recursive/external evidence is a separate prerequisite.
    pub(super) fn native_pull_parent_facets(
        &self,
    ) -> Result<Vec<ToolPermissionFacet>, GitToolError> {
        let mut facets = Vec::new();
        for nature in [ToolNature::Read, ToolNature::Write] {
            for root in [
                &self.repository.workdir,
                &self.repository.git_dir,
                &self.repository.common_dir,
            ] {
                facets.push(path_facet(
                    nature,
                    root,
                    "native Pull parent operation scope",
                )?);
            }
            for capability in [
                "git:pull-integration-worktree",
                "git:pull-repository-metadata",
            ] {
                facets.push(ToolPermissionFacet::with_resource(
                    nature,
                    capability,
                    ToolResourceKind::Remote,
                ));
            }
        }
        for destination in &self.pull_destinations {
            facets.push(Self::native_pull_source_facet(
                destination,
                &self.repository.workdir,
            )?);
        }
        facets.push(ToolPermissionFacet::with_resource(
            ToolNature::Execute,
            self.program.to_str().ok_or_else(|| {
                GitToolError::InvalidInput("Git program path is not UTF-8".into())
            })?,
            ToolResourceKind::Command,
        ));
        Ok(facets)
    }

    // Publish all currently bound roots together. This is not a claim that
    // recursive execution effects are complete: activation retains its guard.
    pub(super) fn native_pull_known_root_facets(
        &self,
    ) -> Result<Vec<ToolPermissionFacet>, GitToolError> {
        let mut facets = self.native_pull_parent_facets()?;
        let Some(modules) = &self.native_pull_modules else {
            return Ok(facets);
        };
        modules.revalidate()?;
        if !modules.complete() || !modules.all_worktree_targets_resolved() {
            return Err(GitToolError::InvalidInput(
                "Native Pull child root inventory incomplete".into(),
            ));
        }
        let mut roots = std::collections::BTreeSet::new();
        for root in modules.stored_module_roots() {
            let metadata = self.native_pull_metadata.get(root).ok_or_else(|| {
                GitToolError::InvalidInput("Native Pull child metadata missing".into())
            })?;
            if !self.native_pull_configs.contains_key(root)
                || !self.pull_child_startup.contains_key(root)
            {
                return Err(GitToolError::InvalidInput(
                    "Native Pull child root configuration incomplete".into(),
                ));
            }
            let common = if metadata.common_path == root {
                root
            } else {
                modules
                    .resolved_common_target(&metadata.common_path)
                    .ok_or_else(|| {
                        GitToolError::InvalidInput(
                            "Native Pull child common target unresolved".into(),
                        )
                    })?
            };
            roots.insert(root);
            roots.insert(common);
        }
        roots.extend(modules.resolved_startup_worktrees());
        for root in roots {
            for nature in [ToolNature::Read, ToolNature::Write] {
                let facet = path_facet(nature, root, "native Pull bound child repository root")?;
                if !facets.iter().any(|existing| {
                    existing.nature == facet.nature
                        && existing.resource_kind == facet.resource_kind
                        && existing.resource == facet.resource
                }) {
                    facets.push(facet);
                }
            }
        }
        Ok(facets)
    }

    // Resolve against retained, revalidated configuration and the current HEAD.
    // This is a local scalar probe, never a transport or fetch invocation.
    pub(super) async fn resolve_native_pull_destination(
        &self,
    ) -> Result<(String, Vec<String>), GitToolError> {
        let Operation::Pull(input) = &self.operation else {
            return Err(GitToolError::InvalidInput(
                "Native Pull operation missing".into(),
            ));
        };
        let config = self.config.as_ref().ok_or_else(|| {
            GitToolError::InvalidInput("Native Pull configuration evidence missing".into())
        })?;
        let branch = self.native_pull_branch().await?;
        let remote = super::super::pull_state::fetch_remote(
            &config.pull_records()?,
            branch.as_deref(),
            input.remote.as_deref(),
        )?;
        config.prepare_pull_transport(&remote)?;
        let destinations = super::super::remote_state::destinations(
            &config.pull_remote_records()?,
            &remote,
            false,
        )?;
        self.revalidate()?;
        Ok((remote, destinations))
    }

    // Full symbolic branch used by both pre-fetch policy and destination discovery.
    pub(super) async fn native_pull_branch(&self) -> Result<Option<String>, GitToolError> {
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
                    })?
                    .to_owned(),
            ),
            Some(1) if head.stdout.is_empty() && head.stderr.is_empty() => None,
            _ => {
                return Err(GitToolError::InvalidInput(
                    "Native Pull branch inspection failed".into(),
                ));
            }
        };
        Ok(branch)
    }

    pub(super) async fn validate_native_pull_destination(&self) -> Result<(), GitToolError> {
        let (remote, destinations) = self.resolve_native_pull_destination().await?;
        if self.pull_remote.as_deref() != Some(remote.as_str())
            || destinations != self.pull_destinations
        {
            return Err(GitToolError::InvalidInput(
                "Native Pull destination differs from admitted scope; no Pull started".into(),
            ));
        }
        self.revalidate()
    }

    // Reuse bounded, unredirected auxiliary evidence where available; otherwise
    // require exact hook Read admission. Re-derive the frontier so omitted
    // candidates cannot hide executable effects from the final operation profile.
    pub(super) async fn capture_native_pull_hooks(
        &self,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<Vec<(PathBuf, snapshot::RegularEvidence)>, GitToolError> {
        self.revalidate()?;
        let config = self.config.as_ref().ok_or_else(|| {
            GitToolError::InvalidInput("Native Pull configuration evidence missing".into())
        })?;
        let branch = self.native_pull_branch().await?;
        let paths = config.native_pull_hook_paths(
            &self.repository,
            &self.environment,
            branch.as_deref(),
        )?;
        if paths != self.pull_hook_paths {
            return Err(GitToolError::InvalidInput(
                "Native Pull hook frontier differs from admitted scope; no Pull started".into(),
            ));
        }
        if let Some(auxiliary) = &self.auxiliary_paths {
            auxiliary
                .revalidate_native_pull(authorizations, &self.native_pull_auxiliary_roots()?)?;
            if paths
                .iter()
                .any(|path| auxiliary.leaf_evidence(path).is_none())
            {
                return Err(GitToolError::InvalidInput(
                    "Native Pull hook evidence missing".into(),
                ));
            }
            // The shared auxiliary store retains and revalidates these handles;
            // do not duplicate captures or charge their contents twice.
            return Ok(Vec::new());
        }

        for path in &paths {
            if !authorizations.iter().any(|token| {
                token.authorizes_path(
                    "git_pull",
                    ToolNature::Read,
                    &self.workspace,
                    path.to_str().unwrap_or_default(),
                )
            }) {
                return Err(GitToolError::InvalidInput(
                    "Native Pull hook lacks exact Read admission; no Pull started".into(),
                ));
            }
        }
        let evidence = super::super::config_state::ConfigState::capture_pull_hooks(&paths)?;
        if !self.pull_hook_evidence.is_empty() && evidence != self.pull_hook_evidence {
            return Err(GitToolError::InvalidInput(
                "Native Pull hook evidence changed after discovery; no Pull started".into(),
            ));
        }
        self.revalidate()?;
        Ok(evidence)
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
        let required_facets = self.native_pull_known_root_facets()?;
        for required in &required_facets {
            if !self.facets.iter().any(|facet| {
                facet.nature == required.nature
                    && facet.resource_kind == required.resource_kind
                    && facet.resource == required.resource
            }) {
                return Err(GitToolError::InvalidInput(
                    "Native Pull scope capability missing".into(),
                ));
            }
        }
        for required in &required_facets {
            if required.resource_kind == Some(ToolResourceKind::Path) {
                let root = required.resource.as_deref().ok_or_else(|| {
                    GitToolError::InvalidInput("Native Pull root path missing".into())
                })?;
                if !authorizations.iter().any(|token| {
                    token.authorizes_path("git_pull", required.nature, &self.workspace, root)
                }) {
                    return Err(GitToolError::InvalidInput(
                        "Native Pull root lacks fresh admission".into(),
                    ));
                }
            }
        }
        if !self.pull_transport_supported || self.pull_destinations.is_empty() {
            return Err(GitToolError::InvalidInput(
                "Native Pull transport scope is incomplete".into(),
            ));
        }
        self.validate_native_pull_configurations(authorizations)
            .await?;
        if self.native_pull_modules.as_ref().is_some_and(|modules| {
            !modules.complete() || modules.stored_module_roots().next().is_some()
        }) {
            return Err(GitToolError::InvalidInput(
                "Native Pull recursive repository configuration admission is incomplete; no Pull started".into(),
            ));
        }
        self.validate_native_pull_destination().await?;
        self.capture_native_pull_hooks(authorizations).await?;
        // The phase is deliberately unreachable from production discovery until
        // native recursive/external-effect scope discovery and activation tests
        // replace the old scheduler. Do not promote a fetch-only plan here.
        if !cfg!(test) {
            return Err(GitToolError::InvalidInput(
                "Native Pull awaits complete effect discovery and activation evidence; no command started".into(),
            ));
        }
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
