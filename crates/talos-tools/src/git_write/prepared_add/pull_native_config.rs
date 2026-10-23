//! Retained native Pull child configuration and destination evidence.

use super::*;
use std::collections::BTreeSet;

impl AddPlan {
    fn native_pull_child_hook_paths(
        &self,
        root: &Path,
        common: &Path,
        config: &super::super::config_state::ChildConfig,
    ) -> Result<Vec<PathBuf>, GitToolError> {
        let startup = self.pull_child_startup.get(root).ok_or_else(|| {
            GitToolError::InvalidInput("Native Pull child startup evidence missing".into())
        })?;
        let modules = self.native_pull_modules.as_ref().ok_or_else(|| {
            GitToolError::InvalidInput("Native Pull child inventory missing".into())
        })?;
        let worktrees = startup
            .targets
            .iter()
            .map(|path| {
                modules
                    .resolved_startup_worktree(path)
                    .map(Path::to_owned)
                    .ok_or_else(|| {
                        GitToolError::InvalidInput(
                            "Native Pull child startup target unresolved".into(),
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        config.native_pull_hook_paths(common, root, &worktrees, &self.environment)
    }

    pub(super) fn native_pull_auxiliary_roots(&self) -> Result<Vec<PathBuf>, GitToolError> {
        self.repository.revalidate_location()?;
        let mut roots = vec![
            self.repository.workdir.clone(),
            self.repository.git_dir.clone(),
            self.repository.common_dir.clone(),
        ];
        if let Some(modules) = &self.native_pull_modules {
            modules.revalidate()?;
            roots.extend(modules.stored_module_roots().map(Path::to_owned));
        }
        Ok(roots)
    }

    // The named discovery authority permits Git's normal include-chain read,
    // not subsequent use of those files without their own exact Read proof.
    // Keep original evidence: a changed chain must never replace the approved one.
    pub(super) async fn discover_native_pull_configurations(
        &mut self,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<bool, GitToolError> {
        if self.native_pull_metadata.is_empty() {
            return Ok(true);
        }
        if !self.facets.iter().any(|facet| {
            facet.nature == ToolNature::Read
                && facet.resource_kind == Some(ToolResourceKind::Remote)
                && facet.resource.as_deref() == Some("git:configuration-chain")
        }) {
            self.facets.push(ToolPermissionFacet::with_resource(
                ToolNature::Read,
                "git:configuration-chain",
                ToolResourceKind::Remote,
            ).with_description("discover native Pull child system/global/include configuration chains; values withheld"));
            return Ok(false);
        }
        // Check every retained source before any revalidation or host probe.
        for config in self.native_pull_configs.values() {
            for (path, _) in &config.files {
                self.require_native_pull_module_read(path, authorizations)?;
            }
        }
        for config in self.native_pull_configs.values() {
            config.revalidate()?;
        }
        let mut discovered = false;
        for root in self.native_pull_metadata.keys() {
            let output = self.inspect_receiver_config_at(root).await?;
            if let Some(config) = self.native_pull_configs.get(root) {
                config.verify_host(&output)?;
                continue;
            }
            let config = super::super::config_state::ChildConfig::capture_native_pull(
                &output,
                root,
                &self.pull_reference_budget,
            )?;
            if output != self.inspect_receiver_config_at(root).await? {
                return Err(GitToolError::InvalidInput(
                    "Native Pull child configuration changed during discovery; values withheld"
                        .into(),
                ));
            }
            for (path, _) in &config.files {
                if !self.facets.iter().any(|facet| {
                    facet.nature == ToolNature::Read
                        && facet.resource_kind == Some(ToolResourceKind::Path)
                        && facet.resource.as_deref() == path.to_str()
                }) {
                    self.facets.push(path_facet(
                        ToolNature::Read,
                        path,
                        "native Pull child configuration-chain source; use requires fresh Read admission",
                    )?);
                }
            }
            let remote = config.fetch_remote(self.native_pull_metadata[root].branch()?)?;
            for path in self.native_pull_remote_sources(root, &remote)? {
                self.facets.push(path_facet(
                    ToolNature::Read,
                    &path,
                    "native Pull child legacy remote source; contents deferred",
                )?);
            }
            let metadata = &self.native_pull_metadata[root];
            let common = if metadata.common_path == *root {
                root.as_path()
            } else {
                self.native_pull_modules
                    .as_ref()
                    .and_then(|modules| modules.resolved_common_target(&metadata.common_path))
                    .ok_or_else(|| {
                        GitToolError::InvalidInput(
                            "Native Pull child common target unresolved".into(),
                        )
                    })?
            };
            let hooks = self.native_pull_child_hook_paths(root, common, &config)?;
            if let Some(auxiliary) = self.auxiliary_paths.as_mut() {
                auxiliary.extend(self.operation.name(), hooks.iter().cloned(), authorizations)?;
            } else {
                self.auxiliary_paths =
                    Some(super::super::auxiliary_paths::AuxiliaryPaths::resolve(
                        self.operation.name(),
                        &self.workspace,
                        &self.repository.git_dir,
                        hooks.iter().cloned(),
                        authorizations,
                    )?);
            }
            self.auxiliary_paths
                .as_ref()
                .ok_or_else(|| {
                    GitToolError::InvalidInput("Native Pull auxiliary evidence missing".into())
                })?
                .add_read_frontier(&mut self.facets)?;
            self.native_pull_fetch_hooks.extend(hooks);
            self.native_pull_configs.insert(root.clone(), config);
            discovered = true;
        }
        // Newly retained evidence does not itself require another approval:
        // continue immediately when this immutable proof already covers every
        // published source and resolved auxiliary target. Otherwise return the
        // frontier without capturing unapproved auxiliary contents or executing Pull.
        let mut admitted = true;
        let auxiliary_roots = self.native_pull_auxiliary_roots()?;
        if discovered {
            for config in self.native_pull_configs.values() {
                for (path, entry) in &config.files {
                    // Parent discovery already retained ordinary host origins
                    // under the configuration-chain capability. Reusing that
                    // same frozen source for internal plan construction does
                    // not require the future execution proof. Newly introduced
                    // external child includes still publish a fresh frontier.
                    let retained_parent_source = self.config.as_ref().is_some_and(|parent| {
                        parent
                            .files
                            .iter()
                            .any(|(source, retained)| source == path && retained == entry)
                    });
                    admitted &= retained_parent_source
                        || self.native_pull_module_read(path, authorizations)?
                        || authorizations.iter().any(|token| {
                            token.authorizes_path(
                                "git_pull",
                                ToolNature::Read,
                                &self.workspace,
                                path.to_str().unwrap_or_default(),
                            )
                        });
                }
            }
            if let Some(auxiliary) = &self.auxiliary_paths {
                admitted &= auxiliary.targets().all(|path| {
                    auxiliary.native_pull_target_read(path, authorizations, &auxiliary_roots)
                        || authorizations.iter().any(|token| {
                            token.authorizes_path(
                                "git_pull",
                                ToolNature::Read,
                                &self.workspace,
                                path.to_str().unwrap_or_default(),
                            )
                        })
                });
            }
        }
        if admitted {
            self.auxiliary_paths
                .as_mut()
                .ok_or_else(|| {
                    GitToolError::InvalidInput("Native Pull auxiliary evidence missing".into())
                })?
                .capture_native_pull(
                    authorizations,
                    &self
                        .native_pull_fetch_hooks
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>(),
                    &self.pull_reference_budget,
                    &auxiliary_roots,
                )?;
        }
        Ok(admitted)
    }

    // Scope derivation from frozen native URL selection; never contacts the source.
    pub(super) fn native_pull_child_remote_facets(
        &self,
        gitdir: &Path,
        config: &super::super::config_state::ChildConfig,
    ) -> Result<Vec<ToolPermissionFacet>, GitToolError> {
        let metadata = self.native_pull_metadata.get(gitdir).ok_or_else(|| {
            GitToolError::InvalidInput("Native Pull child HEAD evidence missing".into())
        })?;
        let remote = config.fetch_remote(metadata.branch()?)?;
        config.prepare_pull_transport(&remote)?;
        let destination = &self
            .native_pull_remotes
            .get(gitdir)
            .ok_or_else(|| {
                GitToolError::InvalidInput("Native Pull child remote evidence missing".into())
            })?
            .destination;
        Ok(vec![Self::native_pull_source_facet(destination, gitdir)?])
    }

    // Called only after consuming the exact plan proof. All source grants are
    // checked before reopening any retained configuration; named discovery
    // authority covers Git's normal chain probe, including newly reported origins.
    pub(super) async fn validate_native_pull_configurations(
        &self,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<(), GitToolError> {
        if self.native_pull_configs.len() != self.native_pull_metadata.len() {
            return Err(GitToolError::InvalidInput(
                "Native Pull child configuration evidence incomplete".into(),
            ));
        }
        let mut expected_fetch_hooks = BTreeSet::new();
        // Ordinary config setup enters startup worktrees and follows fixed
        // commondir pointers. Recheck these target grants before its probe.
        if !self.native_pull_metadata.is_empty() {
            let modules = self.native_pull_modules.as_ref().ok_or_else(|| {
                GitToolError::InvalidInput("Native Pull child inventory missing".into())
            })?;
            for (root, metadata) in &self.native_pull_metadata {
                self.require_native_pull_module_read(root, authorizations)?;
                for leaf in ["HEAD", "commondir", "gitdir", "config", "config.worktree"] {
                    self.require_native_pull_module_read(&root.join(leaf), authorizations)?;
                }
                let common = if metadata.common_path == *root {
                    root.as_path()
                } else {
                    modules
                        .resolved_common_target(&metadata.common_path)
                        .ok_or_else(|| {
                            GitToolError::InvalidInput(
                                "Native Pull child common target unresolved".into(),
                            )
                        })?
                };
                self.require_native_pull_module_read(common, authorizations)?;
                self.require_native_pull_module_read(&common.join("config"), authorizations)?;
                expected_fetch_hooks.extend(self.native_pull_child_hook_paths(
                    root,
                    common,
                    &self.native_pull_configs[root],
                )?);
            }
            for target in modules.resolved_startup_worktrees() {
                if self.child_read(target, authorizations)? != target {
                    return Err(GitToolError::InvalidInput(
                        "Native Pull startup worktree differs from actual target admission".into(),
                    ));
                }
            }
        }
        for config in self.native_pull_configs.values() {
            for (path, _) in &config.files {
                self.require_native_pull_module_read(path, authorizations)?;
            }
        }
        if expected_fetch_hooks != self.native_pull_fetch_hooks {
            return Err(GitToolError::InvalidInput(
                "Native Pull child fetch hook frontier changed".into(),
            ));
        }
        if !self.native_pull_configs.is_empty() {
            let auxiliary = self.auxiliary_paths.as_ref().ok_or_else(|| {
                GitToolError::InvalidInput("Native Pull auxiliary evidence missing".into())
            })?;
            auxiliary
                .revalidate_native_pull(authorizations, &self.native_pull_auxiliary_roots()?)?;
            for hook in &self.native_pull_fetch_hooks {
                if auxiliary.leaf_evidence(hook).is_none() {
                    return Err(GitToolError::InvalidInput(
                        "Native Pull child fetch hook evidence missing".into(),
                    ));
                }
            }
        }
        if !self.native_pull_configs.is_empty()
            && !self.facets.iter().any(|facet| {
                facet.nature == ToolNature::Read
                    && facet.resource_kind == Some(ToolResourceKind::Remote)
                    && facet.resource.as_deref() == Some("git:configuration-chain")
            })
        {
            return Err(GitToolError::InvalidInput(
                "Native Pull child configuration discovery authority missing".into(),
            ));
        }
        for (root, config) in &self.native_pull_configs {
            config.revalidate()?;
            config.verify_host(&self.inspect_receiver_config_at(root).await?)?;
            config.revalidate()?;
            let remote = config.fetch_remote(self.native_pull_metadata[root].branch()?)?;
            self.validate_native_pull_remote(root, &remote, authorizations)
                .await?;
            for required in self.native_pull_child_remote_facets(root, config)? {
                if !self.facets.iter().any(|facet| {
                    facet.nature == required.nature
                        && facet.resource_kind == required.resource_kind
                        && facet.resource == required.resource
                }) {
                    return Err(GitToolError::InvalidInput(
                        "Native Pull child destination differs from admitted scope".into(),
                    ));
                }
            }
        }
        Ok(())
    }
}
