//! Native Pull scope discovery without fetch scheduling or integration prediction.

use super::*;

impl AddPlan {
    // Native preparation gathers scope only; it does not fetch, predict new
    // commits, or choose an integration command. Sender routing stays on the
    // legacy path until recursive configuration admission is complete.
    pub(super) async fn discover_native_pull_scope(
        mut self: Box<Self>,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<Box<Self>, GitToolError> {
        self.revalidate()?;
        self.require_pull_parent_metadata(authorizations)?;
        self.pull_hook_evidence = self.capture_native_pull_hooks(authorizations).await?;
        let mut inventory = self.pull_inventory.take().ok_or_else(|| {
            GitToolError::InvalidInput("Native Pull parent inventory missing".into())
        })?;
        let parent_ready = self.advance_pull_parent_inventory(&mut inventory, authorizations)?;
        self.pull_inventory = Some(inventory);
        if !parent_ready {
            self.phase = Phase::PullNativePreparation;
            self.identity = PreparedPlanIdentity::new();
            return Ok(self);
        }
        if self.native_pull_modules.is_none() {
            self.native_pull_modules =
                Some(super::super::pull_inventory::Inventory::stored_modules(
                    &self.repository.common_dir,
                    self.pull_reference_budget.clone(),
                )?);
        }
        let modules = self.native_pull_modules.as_mut().ok_or_else(|| {
            GitToolError::InvalidInput("Native Pull module inventory missing".into())
        })?;
        // The parent metadata capability is not a grant to read child configs.
        // Request every module-name frontier before inspecting its identities.
        let admitted = |path: &Path| {
            authorizations.iter().any(|token| {
                token.authorizes_path(
                    "git_pull",
                    ToolNature::Read,
                    &self.workspace,
                    path.to_str().unwrap_or_default(),
                )
            })
        };
        if modules.iter_paths().all(&admitted) && !modules.complete() {
            modules.advance(admitted)?;
        }
        for path in modules.iter_paths() {
            if !self.facets.iter().any(|facet| {
                facet.nature == ToolNature::Read && facet.resource.as_deref() == path.to_str()
            }) {
                self.facets.push(path_facet(
                    ToolNature::Read,
                    path,
                    "native Pull stored-module identity frontier; config contents deferred",
                )?);
            }
        }
        if !modules.complete() {
            self.phase = Phase::PullNativePreparation;
            self.identity = PreparedPlanIdentity::new();
            return Ok(self);
        }
        // These roots are candidates, not yet validated repositories. Exact
        // config/metadata admission must precede child configuration discovery.
        let roots = modules
            .stored_module_roots()
            .map(Path::to_owned)
            .collect::<Vec<_>>();
        // Batch the chain request with the fixed-file frontier. It is consumed
        // only after startup targets have independently resolved and been readmitted.
        let request_chain = !roots.is_empty()
            && !self.facets.iter().any(|facet| {
                facet.nature == ToolNature::Read
                    && facet.resource_kind == Some(ToolResourceKind::Remote)
                    && facet.resource.as_deref() == Some("git:configuration-chain")
            });
        if request_chain {
            self.facets.push(ToolPermissionFacet::with_resource(
                ToolNature::Read,
                "git:configuration-chain",
                ToolResourceKind::Remote,
            ));
        }
        let mut metadata_ready = true;
        for root in &roots {
            for leaf in ["HEAD", "commondir", "gitdir", "config", "config.worktree"] {
                let path = root.join(leaf);
                metadata_ready &= admitted(&path);
                if !self.facets.iter().any(|facet| {
                    facet.nature == ToolNature::Read && facet.resource.as_deref() == path.to_str()
                }) {
                    self.facets.push(path_facet(
                        ToolNature::Read,
                        &path,
                        "native Pull stored child fixed configuration/layout frontier",
                    )?);
                }
            }
        }
        if !metadata_ready || request_chain {
            self.phase = Phase::PullNativePreparation;
            self.identity = PreparedPlanIdentity::new();
            return Ok(self);
        }
        // Metadata::capture reads all three fixed pointers. Check the complete
        // frontier first; a parent metadata grant cannot admit these contents.
        // Pointer targets remain declarations: publishing them does not open
        // an external common directory or worktree marker.
        for root in roots {
            self.child_read(&root, authorizations)?;
            let metadata = super::super::receiver_state::Metadata::capture(&root, &root)?;
            if let Some(path) = &metadata.worktree_marker {
                self.facets.push(path_facet(
                    ToolNature::Read,
                    path,
                    "native Pull child layout target; contents require fresh admission",
                )?);
            }
            self.native_pull_metadata.insert(root, metadata);
        }
        let modules = self.native_pull_modules.as_mut().ok_or_else(|| {
            GitToolError::InvalidInput("Native Pull module inventory missing".into())
        })?;
        let mut new_targets = false;
        for (root, metadata) in &self.native_pull_metadata {
            if metadata.common_path != *root {
                new_targets |= modules.retain_common_target(&metadata.common_path)?;
            }
        }
        if modules.has_worktree_targets() {
            let resolution_admitted = Self::pull_resolution_admitted(&self.facets);
            if !resolution_admitted {
                self.facets.push(ToolPermissionFacet::with_resource(
                    ToolNature::Read,
                    "git:pull-worktree-resolution",
                    ToolResourceKind::Remote,
                ).with_description("resolve frozen child commondir and startup core.worktree paths, including external component metadata; no target contents"));
            }
            if new_targets || !resolution_admitted {
                self.phase = Phase::PullNativePreparation;
                self.identity = PreparedPlanIdentity::new();
                return Ok(self);
            }
            if modules.resolve_worktree_targets()? {
                for metadata in self.native_pull_metadata.values() {
                    if let Some(common) = modules.resolved_common_target(&metadata.common_path) {
                        for path in [common.to_owned(), common.join("config")] {
                            self.facets.push(path_facet(ToolNature::Read, &path,
                                "native Pull resolved child common configuration; contents deferred")?);
                        }
                    }
                }
                for target in modules.resolved_startup_worktrees() {
                    self.facets.push(path_facet(
                        ToolNature::Read,
                        target,
                        "native Pull resolved startup worktree; entry requires fresh Read admission",
                    )?);
                }
                self.phase = Phase::PullNativePreparation;
                self.identity = PreparedPlanIdentity::new();
                return Ok(self);
            }
        }
        // Check the entire resolved frontier before reading any startup
        // source. Raw pointer spelling tokens cannot substitute here.
        let common_roots = self
            .native_pull_metadata
            .iter()
            .map(|(root, metadata)| {
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
                Ok((root.clone(), common.to_owned(), metadata.has_common()))
            })
            .collect::<Result<Vec<_>, GitToolError>>()?;
        for (_, common, _) in &common_roots {
            self.child_read(common, authorizations)?;
            self.child_read(&common.join("config"), authorizations)?;
        }
        let mut new_startup_targets = false;
        for (root, common, has_common) in common_roots {
            if let Some(startup) = self.pull_child_startup.get(&root) {
                startup.revalidate()?;
                continue;
            }
            let startup = super::super::config_state::ChildStartupLayout::capture_native_pull(
                &root,
                &common,
                has_common,
                &self.pull_reference_budget,
            )?;
            for target in &startup.targets {
                new_startup_targets |= self
                    .native_pull_modules
                    .as_mut()
                    .ok_or_else(|| {
                        GitToolError::InvalidInput("Native Pull module inventory missing".into())
                    })?
                    .retain_startup_worktree_target(target)?;
            }
            self.pull_child_startup.insert(root, startup);
        }
        if new_startup_targets {
            if !Self::pull_resolution_admitted(&self.facets) {
                self.facets.push(ToolPermissionFacet::with_resource(
                    ToolNature::Read,
                    "git:pull-worktree-resolution",
                    ToolResourceKind::Remote,
                ).with_description("resolve frozen child startup core.worktree paths, including external component metadata; no target contents"));
            }
            self.phase = Phase::PullNativePreparation;
            self.identity = PreparedPlanIdentity::new();
            return Ok(self);
        }
        for target in self
            .native_pull_modules
            .as_ref()
            .ok_or_else(|| {
                GitToolError::InvalidInput("Native Pull module inventory missing".into())
            })?
            .resolved_startup_worktrees()
        {
            if self.child_read(target, authorizations)? != target {
                return Err(GitToolError::InvalidInput(
                    "Native Pull startup worktree differs from actual target admission".into(),
                ));
            }
        }
        if !self
            .discover_native_pull_configurations(authorizations)
            .await?
        {
            self.phase = Phase::PullNativePreparation;
            self.identity = PreparedPlanIdentity::new();
            return Ok(self);
        }
        if !self.discover_native_pull_remotes(authorizations).await? {
            self.phase = Phase::PullNativePreparation;
            self.identity = PreparedPlanIdentity::new();
            return Ok(self);
        }
        let (remote, destinations) = self.resolve_native_pull_destination().await?;
        self.pull_remote = Some(remote);
        self.pull_destinations = destinations;
        self.pull_transport_supported = true;
        self.facets.extend(self.native_pull_parent_facets()?);
        self.phase = Phase::PullNativeMutation;
        self.identity = PreparedPlanIdentity::new();
        self.revalidate()?;
        Ok(self)
    }
}
