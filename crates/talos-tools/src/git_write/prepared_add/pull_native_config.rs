//! Retained native Pull child configuration and destination evidence.

use super::*;

impl AddPlan {
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
                self.child_read(path, authorizations)?;
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
            self.facets
                .extend(self.native_pull_child_remote_facets(root, &config)?);
            self.native_pull_configs.insert(root.clone(), config);
            discovered = true;
        }
        Ok(!discovered)
    }

    // Pure scope derivation from frozen config/HEAD; never contacts the source.
    fn native_pull_child_remote_facets(
        &self,
        gitdir: &Path,
        config: &super::super::config_state::ChildConfig,
    ) -> Result<Vec<ToolPermissionFacet>, GitToolError> {
        let metadata = self.native_pull_metadata.get(gitdir).ok_or_else(|| {
            GitToolError::InvalidInput("Native Pull child HEAD evidence missing".into())
        })?;
        let remote = config.fetch_remote(metadata.branch()?)?;
        config.prepare_pull_transport(&remote)?;
        let mut facets = Vec::new();
        for destination in config.fetch_destinations(&remote)? {
            if let Some(path) = super::super::remote_state::local_destination(&destination, gitdir)?
            {
                facets.push(path_facet(
                    ToolNature::Read,
                    &path,
                    "native Pull child configured local source; source contents deferred",
                )?);
            } else {
                facets.push(ToolPermissionFacet::with_resource(
                    ToolNature::Network,
                    destination,
                    ToolResourceKind::Remote,
                ));
            }
        }
        Ok(facets)
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
        // Ordinary config setup enters startup worktrees and follows fixed
        // commondir pointers. Recheck these target grants before its probe.
        if !self.native_pull_metadata.is_empty() {
            let modules = self.native_pull_modules.as_ref().ok_or_else(|| {
                GitToolError::InvalidInput("Native Pull child inventory missing".into())
            })?;
            for (root, metadata) in &self.native_pull_metadata {
                self.child_read(root, authorizations)?;
                for leaf in ["HEAD", "commondir", "gitdir", "config", "config.worktree"] {
                    self.child_read(&root.join(leaf), authorizations)?;
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
                self.child_read(common, authorizations)?;
                self.child_read(&common.join("config"), authorizations)?;
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
                self.child_read(path, authorizations)?;
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
