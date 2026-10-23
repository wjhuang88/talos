//! Native Pull scope discovery without fetch scheduling or integration prediction.

use super::*;

impl AddPlan {
    pub(super) fn require_native_pull_module_read(
        &self,
        path: &Path,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<(), GitToolError> {
        if !self.native_pull_module_read(path, authorizations)? {
            self.child_read(path, authorizations)?;
        }
        Ok(())
    }

    // Fixed layout leaves and already captured, nonredirected configuration
    // origins of retained stored modules. This never admits arbitrary paths.
    // Inventory and configuration evidence are revalidated before content use.
    pub(super) fn native_pull_module_read(
        &self,
        path: &Path,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<bool, GitToolError> {
        if !authorizations
            .iter()
            .any(|token| token.authorizes_git_pull_bounded_discovery())
            || !authorizations.iter().any(|token| {
                token.authorizes_path(
                    "git_pull",
                    ToolNature::Read,
                    &self.workspace,
                    self.repository.git_dir.to_str().unwrap_or_default(),
                )
            })
        {
            return Ok(false);
        }
        let Some(modules) = &self.native_pull_modules else {
            return Ok(false);
        };
        let covered = modules.stored_module_roots().any(|root| {
            root.starts_with(&self.repository.git_dir)
                && (path == root
                    || ["HEAD", "commondir", "gitdir", "config", "config.worktree"]
                        .iter()
                        .any(|leaf| path == root.join(leaf))
                    || (path.starts_with(root)
                        && self
                            .native_pull_configs
                            .get(root)
                            .is_some_and(|config| config.source_is_unredirected(path))))
        });
        if covered {
            modules.revalidate()?;
        }
        Ok(covered)
    }

    // Native preparation gathers scope only; it does not fetch, predict new
    // commits, or choose an integration command. Sender routing stays on the
    // legacy path until recursive configuration admission is complete.
    pub(super) async fn discover_native_pull_scope(
        mut self: Box<Self>,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<Box<Self>, GitToolError> {
        self.revalidate()?;
        // execute_stage has consumed the complete immutable profile proof.
        // Bind this dedicated scope to the displayed, retained parent gitdir;
        // an exact Read of a directory alone never grants descendant reads.
        let bounded_modules = self.facets.iter().any(|facet| {
            facet.nature == ToolNature::Read
                && facet.resource_kind == Some(ToolResourceKind::Remote)
                && facet.resource.as_deref() == Some("git:pull-bounded-discovery")
        }) && authorizations
            .iter()
            .any(|token| token.authorizes_git_pull_bounded_discovery())
            && authorizations.iter().any(|token| {
                token.authorizes_path(
                    "git_pull",
                    ToolNature::Read,
                    &self.workspace,
                    self.repository.git_dir.to_str().unwrap_or_default(),
                )
            });
        if !self.facets.iter().any(|facet| {
            facet.nature == ToolNature::Read
                && facet.resource_kind == Some(ToolResourceKind::Remote)
                && facet.resource.as_deref() == Some("git:auxiliary-path-resolution")
        }) {
            self.facets.push(ToolPermissionFacet::with_resource(
                ToolNature::Read,
                "git:auxiliary-path-resolution",
                ToolResourceKind::Remote,
            ).with_description("resolve frozen native Pull child hook paths; contents require exact Read admission"));
            self.facets.push(path_facet(
                ToolNature::Read,
                &self.repository.git_dir,
                "source anchor for native Pull child hook resolution",
            )?);
        }

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
                    &self.repository.git_dir,
                    self.pull_reference_budget.clone(),
                )?);
        }
        let modules = self.native_pull_modules.as_mut().ok_or_else(|| {
            GitToolError::InvalidInput("Native Pull module inventory missing".into())
        })?;
        // The dedicated discovery scope permits internal module traversal.
        // Inventory charges the shared budget, rejects links and revalidates
        // retained directory identities on each advance. Outside this root,
        // keep exact frontier admission; parent metadata alone is insufficient.
        let admitted = |path: &Path| {
            (bounded_modules && path.starts_with(&self.repository.git_dir))
                || authorizations.iter().any(|token| {
                    token.authorizes_path(
                        "git_pull",
                        ToolNature::Read,
                        &self.workspace,
                        path.to_str().unwrap_or_default(),
                    )
                })
        };
        while modules.iter_paths().all(&admitted) && !modules.complete() {
            modules.advance(admitted)?;
        }
        for path in modules.iter_paths() {
            if bounded_modules && path.starts_with(&self.repository.git_dir) {
                continue;
            }
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
        // These roots are candidates, not yet validated repositories. Fixed
        // files require either dedicated bounded discovery or exact admission.
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
            for path in std::iter::once(root.clone()).chain(
                ["HEAD", "commondir", "gitdir", "config", "config.worktree"]
                    .map(|leaf| root.join(leaf)),
            ) {
                metadata_ready &= self.native_pull_module_read(&path, authorizations)?
                    || authorizations.iter().any(|token| {
                        token.authorizes_path(
                            "git_pull",
                            ToolNature::Read,
                            &self.workspace,
                            path.to_str().unwrap_or_default(),
                        )
                    });
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
        // Capture all three fixed pointers against the invocation-wide budget.
        // A generic parent metadata grant cannot admit these contents.
        // Pointer targets remain declarations: publishing them does not open
        // an external common directory or worktree marker.
        for root in roots {
            // Root identities were inspected by the admitted inventory; the
            // fixed frontier above checks the dedicated or exact authority.
            // Entry revalidation already checked retained pointer evidence.
            // Do not replace it or republish its targets on every continuation.
            if self.native_pull_metadata.contains_key(&root) {
                continue;
            }
            let metadata = super::super::receiver_state::Metadata::capture_with_budget(
                &root,
                &root,
                Some(&self.pull_reference_budget),
            )?;
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
            for path in [common.clone(), common.join("config")] {
                self.require_native_pull_module_read(&path, authorizations)?;
            }
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
        self.facets.extend(self.native_pull_known_root_facets()?);
        self.phase = Phase::PullNativeMutation;
        self.identity = PreparedPlanIdentity::new();
        self.revalidate()?;
        Ok(self)
    }
}
