//! Prepared Pull children stages.

use super::super as git_write;
use super::*;

impl AddPlan {
    pub(super) async fn discover_pull_child_attributes(
        mut self: Box<Self>,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<Box<Self>, GitToolError> {
        // Check the entire frontier before capturing any newly discovered file.
        for path in &self.pull_attribute_paths {
            if !authorizations.iter().any(|token| {
                token.authorizes_path(
                    "git_pull",
                    ToolNature::Read,
                    &self.workspace,
                    path.to_str().unwrap_or_default(),
                )
            }) {
                return Err(GitToolError::InvalidInput(
                    "Pull child attributes lack exact Read admission; fetched effects are retained"
                        .into(),
                ));
            }
        }
        self.revalidate()?;
        let mut budget = snapshot::Budget {
            entries: 4096,
            bytes: 4 * 1024 * 1024,
        };
        let evidence = self
            .pull_attribute_paths
            .iter()
            .map(|path| PullAttribute::capture(path, &mut budget))
            .collect::<Result<Vec<_>, _>>()?;
        self.revalidate()?;
        self.pull_attributes = evidence;
        self.revalidate()?;
        self.resolve_pull_child_activity().await?;
        self.select_pull_child_identity()?;
        Ok(self)
    }

    pub(super) fn selected_pull_child_paths(&self) -> Vec<PathBuf> {
        // Git schedules selected current-index entries before historical changes.
        // Current `On` candidates do not require a changed gitlink or historical
        // activity; policy selection already happened in select_current().
        let mut paths: Vec<_> = self
            .pull_current_fetch_children
            .keys()
            .map(|path| self.repository.workdir.join(path))
            .collect();
        for child in self.pull_changed_children.values() {
            if matches!(
                child.activity,
                Some(git_write::pull_children::Activity::Enabled)
            ) && child.effective_recursion != Some(git_write::pull_children::Recursion::Off)
            {
                let path = self.repository.workdir.join(&child.path);
                if !paths.contains(&path) {
                    paths.push(path);
                }
            }
        }
        paths
    }

    pub(super) fn pull_child_name(&self, workdir: &Path) -> Option<&str> {
        if let Some(path) = self
            .pull_current_fetch_children
            .keys()
            .find(|path| self.repository.workdir.join(path) == workdir)
        {
            return self
                .pull_current_modules
                .as_ref()?
                .module_names
                .get(path)
                .map(String::as_str);
        }
        self.pull_changed_children
            .iter()
            .find(|(_, child)| self.repository.workdir.join(&child.path) == workdir)
            .map(|(name, _)| name.as_str())
    }

    pub(super) fn pull_child_objects(&self, workdir: &Path) -> Vec<String> {
        let name = self.pull_child_name(workdir);
        self.pull_changed_children
            .iter()
            .filter(|(key, child)| {
                name.map_or_else(
                    || self.repository.workdir.join(&child.path) == workdir,
                    |name| key.as_str() == name,
                )
            })
            .flat_map(|(_, child)| child.objects.iter().cloned())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    pub(super) fn pull_child_unconditional(&self, workdir: &Path) -> bool {
        use git_write::pull_children::Recursion;
        // Current-index tasks are scheduled first and claim the module name.
        // A historical task cannot upgrade the winning task's recursion mode.
        if let Some((_, mode)) = self
            .pull_current_fetch_children
            .iter()
            .find(|(path, _)| self.repository.workdir.join(path) == workdir)
        {
            return *mode == Recursion::On;
        }
        self.pull_changed_children.iter().any(|(name, child)| {
            self.pull_child_name(workdir).map_or_else(
                || self.repository.workdir.join(&child.path) == workdir,
                |selected| selected == name,
            ) && child.effective_recursion == Some(Recursion::On)
        })
    }

    pub(super) fn select_pull_child_identity(&mut self) -> Result<(), GitToolError> {
        let paths = self.selected_pull_child_paths();
        for path in &paths {
            self.facets.push(path_facet(
                ToolNature::Read,
                path,
                "Pull selected child identity",
            )?);
            self.facets.push(path_facet(
                ToolNature::Read,
                &path.join(".git"),
                "Pull selected child Git marker",
            )?);
            // Publish the name-based fallback in the same identity wave. This
            // grants no permission to inspect it before this wave is approved,
            // and does not make an unavailable checkout a usable repository.
            if let Some(candidate) = self.pull_child_name(path).and_then(|name| {
                git_write::pull_children::legacy_metadata_candidate(
                    &self.repository.common_dir,
                    name,
                )
            }) {
                self.facets.push(path_facet(
                    ToolNature::Read,
                    &candidate,
                    "Pull selected child legacy metadata candidate; layout validation deferred",
                )?);
                for prefix in git_write::pull_children::legacy_metadata_prefix_reads(
                    &self.repository.common_dir,
                    self.pull_child_name(path).ok_or_else(|| {
                        GitToolError::InvalidInput("Pull child module name missing".into())
                    })?,
                ) {
                    self.facets.push(path_facet(
                        ToolNature::Read,
                        &prefix,
                        "Pull legacy metadata prefix identity and repository collision checks; reads deferred",
                    )?);
                }
            }
        }
        self.identity = PreparedPlanIdentity::new();
        self.phase = if !paths.is_empty() {
            Phase::PullChildIdentity
        } else {
            Phase::PullIntegration
        };
        Ok(())
    }

    pub(super) fn discover_pull_child_identity(
        mut self: Box<Self>,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<Box<Self>, GitToolError> {
        self.revalidate()?;
        if !self.pull_child_layouts.is_empty() {
            return self.discover_pull_child_metadata(authorizations);
        }
        let mut seen = std::collections::BTreeSet::new();
        for path in self.selected_pull_child_paths() {
            let name = self.pull_child_name(&path).map(str::to_owned);
            if name.as_ref().is_some_and(|name| seen.contains(name)) {
                continue;
            }
            let marker = path.join(".git");
            for candidate in [&path, &marker] {
                self.child_read(candidate, authorizations)?;
            }
            let layout = git_write::receiver_state::Root::capture(
                &path,
                self.child_read(&path, authorizations)?,
            )?;
            if layout.metadata_path == layout.canonical {
                return Err(GitToolError::InvalidInput(
                    "Pull selected child has no Git marker".into(),
                ));
            }
            for candidate in std::iter::once(layout.metadata_path.clone())
                .chain(["commondir", "gitdir", "HEAD"].map(|leaf| layout.metadata_path.join(leaf)))
            {
                self.facets.push(path_facet(
                    ToolNature::Read,
                    &candidate,
                    "Pull child Git directory and fixed layout pointers; targets deferred",
                )?);
            }
            self.pull_child_layouts.push((layout, None));
            if let Some(name) = name {
                seen.insert(name);
            }
        }
        self.identity = PreparedPlanIdentity::new();
        Ok(self)
    }

    pub(super) fn discover_pull_child_metadata(
        mut self: Box<Self>,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<Box<Self>, GitToolError> {
        if self
            .pull_child_layouts
            .iter()
            .any(|(_, metadata)| metadata.is_none())
        {
            let mut discovered = Vec::new();
            for (layout, _) in &self.pull_child_layouts {
                self.child_read(&layout.metadata_path, authorizations)?;
                for leaf in ["commondir", "gitdir", "HEAD"] {
                    self.child_read(&layout.metadata_path.join(leaf), authorizations)?;
                }
            }
            for (layout, _) in &self.pull_child_layouts {
                let metadata = git_write::receiver_state::Metadata::capture(
                    &layout.metadata_path,
                    self.child_read(&layout.metadata_path, authorizations)?,
                )?;
                if !self.child_backlink_matches(
                    layout,
                    metadata.worktree_marker.as_deref(),
                    authorizations,
                )? {
                    return Err(GitToolError::InvalidInput(
                        "Pull child backlink names an unadmitted worktree".into(),
                    ));
                }
                discovered.push(metadata);
            }
            for ((_, slot), metadata) in self.pull_child_layouts.iter_mut().zip(discovered) {
                self.facets.push(path_facet(
                    ToolNature::Read,
                    &metadata.common_path,
                    "Pull child shared metadata identity; configuration contents deferred",
                )?);
                *slot = Some(metadata);
            }
            if self.pull_child_layouts.iter().any(|(_, metadata)| {
                metadata.as_ref().is_none_or(|metadata| {
                    self.child_read(&metadata.common_path, authorizations)
                        .is_err()
                })
            }) {
                self.identity = PreparedPlanIdentity::new();
                return Ok(self);
            }
        }
        for (layout, metadata) in &self.pull_child_layouts {
            let metadata = metadata.as_ref().ok_or_else(|| {
                GitToolError::InvalidInput("Pull child metadata evidence missing".into())
            })?;
            let admitted = self.child_read(&metadata.common_path, authorizations)?;
            let location =
                git_write::repository_state::ChildLocation::capture(&layout.metadata_path)?;
            if location.common_dir != admitted {
                return Err(GitToolError::InvalidInput(
                    "Pull child common directory differs from admission".into(),
                ));
            }
            self.facets.push(
                ToolPermissionFacet::with_resource(
                    ToolNature::Read,
                    "git:configuration-chain",
                    ToolResourceKind::Remote,
                )
                .with_description("discover Pull child Git configuration chain"),
            );
            self.facets.push(path_facet(
                ToolNature::Read,
                &location.common_dir.join("config"),
                "Pull selected child configuration",
            )?);
            self.facets.push(path_facet(
                ToolNature::Read,
                &location.common_dir,
                "Pull selected child Git metadata for object admission",
            )?);
            self.facets.push(path_facet(
                ToolNature::Read,
                &location.git_dir,
                "Pull selected child private Git metadata for object admission",
            )?);
            self.facets.push(
                ToolPermissionFacet::with_resource(
                    ToolNature::Read,
                    "git:pull-repository-metadata",
                    ToolResourceKind::Remote,
                )
                .with_description("bounded graph discovery in this Pull's individually bound child metadata roots"),
            );
            self.facets.push(path_facet(
                ToolNature::Read,
                &location.git_dir.join("config.worktree"),
                "Pull child fixed startup worktree configuration",
            )?);
            self.child_locations
                .push((layout.spelling.clone(), location));
        }
        self.revalidate()?;
        self.identity = PreparedPlanIdentity::new();
        self.phase = Phase::PullChildConfiguration;
        Ok(self)
    }

    pub(super) async fn discover_pull_child_configuration(
        mut self: Box<Self>,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<Box<Self>, GitToolError> {
        // Capture from the admitted profile before any inventory can append
        // facets. One child's new request cannot authorize another child.
        let resolution_admitted = Self::pull_resolution_admitted(&self.facets);
        if !self.facets.iter().any(|facet| {
            facet.nature == ToolNature::Read
                && facet.resource_kind == Some(ToolResourceKind::Remote)
                && facet.resource.as_deref() == Some("git:configuration-chain")
        }) {
            return Err(GitToolError::InvalidInput(
                "Pull child configuration-chain capability missing".into(),
            ));
        }
        if self.pull_config_sources_discovered {
            for path in &self.pull_config_source_paths {
                if !authorizations.iter().any(|token| {
                    token.authorizes_path(
                        "git_pull",
                        ToolNature::Read,
                        &self.workspace,
                        path.to_str().unwrap_or_default(),
                    )
                }) {
                    return Err(GitToolError::InvalidInput(
                        "Pull child configuration include source lacks exact Read admission".into(),
                    ));
                }
            }
            // Retain the original evidence until both its sources and the new
            // discovery output have been checked. Clearing it here would let
            // a changed include chain replace the admitted configuration.
            self.revalidate()?;
        }
        for (workdir, location) in &self.child_locations {
            let historical = self.pull_child_is_historical(workdir);
            if !authorizations.iter().any(|token| {
                token.authorizes_path(
                    "git_pull",
                    ToolNature::Read,
                    &self.workspace,
                    location.common_dir.to_str().unwrap_or_default(),
                )
            }) {
                return Err(GitToolError::InvalidInput(
                    "Pull child Git metadata lacks dedicated Read admission".into(),
                ));
            }
            let config_path = location.common_dir.join("config");
            if !authorizations.iter().any(|token| {
                token.authorizes_path(
                    "git_pull",
                    ToolNature::Read,
                    &self.workspace,
                    config_path.to_str().unwrap_or_default(),
                )
            }) {
                return Err(GitToolError::InvalidInput(
                    "Pull child configuration lacks exact Read admission".into(),
                ));
            }
            self.child_read(&location.git_dir.join("config.worktree"), authorizations)?;
            if !self.pull_config_sources_discovered {
                let metadata = self
                    .pull_child_layouts
                    .iter()
                    .find(|(root, _)| &root.spelling == workdir)
                    .and_then(|(_, metadata)| metadata.as_ref())
                    .ok_or_else(|| {
                        GitToolError::InvalidInput(
                            "Pull child startup metadata evidence missing".into(),
                        )
                    })?;
                let startup = git_write::config_state::ChildStartupLayout::capture(
                    &location.git_dir,
                    &location.common_dir,
                    metadata.has_common(),
                )?;
                for target in startup.targets.iter().filter(|_| !historical) {
                    self.facets.push(path_facet(
                        ToolNature::Read,
                        target,
                        "Pull child raw startup worktree; resolution deferred",
                    )?);
                }
                self.pull_child_startup.insert(workdir.clone(), startup);
            } else {
                let startup = self.pull_child_startup.get(workdir).ok_or_else(|| {
                    GitToolError::InvalidInput("Pull child startup evidence missing".into())
                })?;
                startup.revalidate()?;
                for target in startup.targets.iter().filter(|_| !historical) {
                    self.child_read(target, authorizations)?;
                }
            }
            // Retain startup bytes even when historical fetch overrides their
            // worktree target. Current tasks still require target admission.
            let output = self
                .inspect_child_bootstrap_config(&location.git_dir, workdir)
                .await?;
            if self.pull_config_sources_discovered {
                let expected = self
                    .child_configs
                    .iter()
                    .find(|(path, _)| path == workdir)
                    .ok_or_else(|| {
                        GitToolError::InvalidInput(
                            "Pull child configuration evidence missing".into(),
                        )
                    })?;
                // Compare origins before snapshotting: a newly introduced
                // include has no exact source admission in this continuation.
                expected.1.verify_host(&output)?;
                expected.1.revalidate()?;
                continue;
            }
            let config = git_write::config_state::ChildConfig::capture(&output, workdir)?;
            for (path, _) in &config.files {
                if !self.pull_config_source_paths.contains(path) {
                    self.pull_config_source_paths.push(path.clone());
                }
                if !self.facets.iter().any(|facet| {
                    facet.nature == ToolNature::Read
                        && facet.resource_kind == Some(ToolResourceKind::Path)
                        && facet.resource.as_deref() == path.to_str()
                }) {
                    self.facets.push(path_facet(
                        ToolNature::Read,
                        path,
                        "Pull child configuration include-chain source",
                    )?);
                }
            }
            let metadata = self
                .pull_child_layouts
                .iter()
                .find(|(root, _)| &root.spelling == workdir)
                .and_then(|(_, metadata)| metadata.as_ref())
                .ok_or_else(|| {
                    GitToolError::InvalidInput("Pull child HEAD evidence missing".into())
                })?;
            let remote = config.fetch_remote(metadata.branch()?)?;
            config.prepare_pull_transport(&remote)?;
            // Fetch does not chdir to the worktree. Relative hooks and remote
            // paths are interpreted from its execution directory, the gitdir.
            for path in config.pull_hook_paths(
                &location.common_dir,
                &location.git_dir,
                &self.environment,
            )? {
                self.facets.push(path_facet(
                    ToolNature::Read,
                    &path,
                    "Pull child fetch hook; contents deferred until exact Read admission",
                )?);
            }
            let destinations = config.fetch_destinations(&remote)?;
            for url in &destinations {
                if let Some(path) =
                    git_write::remote_state::local_destination(url, &location.git_dir)?
                {
                    self.facets.push(path_facet(
                        ToolNature::Read,
                        &path,
                        "Pull child local fetch source; contents and fetch deferred",
                    )?);
                    continue;
                }
                self.facets.push(
                    ToolPermissionFacet::with_resource(
                        ToolNature::Network,
                        url.clone(),
                        ToolResourceKind::Remote,
                    )
                    .with_description(
                        "Pull selected child remote discovery; fetch remains deferred",
                    ),
                );
            }
            self.pull_child_fetch_targets
                .insert(workdir.clone(), (remote, destinations));
            let inventory = git_write::pull_inventory::Inventory::with_reference_budget(
                &location.git_dir,
                &location.common_dir,
                self.pull_reference_budget.clone(),
            )?;
            self.pull_child_inventories
                .insert(workdir.clone(), inventory);
            self.child_configs.push((workdir.clone(), config));
        }
        if !self.pull_config_sources_discovered {
            self.pull_config_sources_discovered = true;
            self.identity = PreparedPlanIdentity::new();
            return Ok(self);
        }
        self.revalidate()?;
        if !self.facets.iter().any(|facet| {
            facet.nature == ToolNature::Read
                && facet.resource_kind == Some(ToolResourceKind::Remote)
                && facet.resource.as_deref() == Some("git:pull-repository-metadata")
        }) {
            return Err(GitToolError::InvalidInput(
                "Pull child graph discovery lacks dedicated metadata Read admission".into(),
            ));
        }
        // The closed capability applies only to these retained child roots,
        // with their own exact grants, never to arbitrary paths or other tools.
        for (_, location) in &self.child_locations {
            self.child_read(&location.git_dir, authorizations)?;
            self.child_read(&location.common_dir, authorizations)?;
        }
        let workdirs: Vec<_> = self
            .child_locations
            .iter()
            .map(|(path, _)| path.clone())
            .collect();
        for workdir in workdirs {
            self.bind_pull_child_worktree(&workdir, authorizations)?;
        }
        self.revalidate()?;
        let mut pending = false;
        let mut hook_paths = std::collections::BTreeSet::new();
        for (workdir, config) in &self.child_configs {
            let (_, location) = self
                .child_locations
                .iter()
                .find(|(path, _)| path == workdir)
                .ok_or_else(|| {
                    GitToolError::InvalidInput("Pull child metadata roots missing".into())
                })?;
            hook_paths.extend(config.pull_hook_paths(
                &location.common_dir,
                &location.git_dir,
                &self.environment,
            )?);
        }
        for path in &hook_paths {
            pending |= self.child_read(path, authorizations).is_err();
            if !self.facets.iter().any(|facet| {
                facet.nature == ToolNature::Read
                    && facet.resource_kind == Some(ToolResourceKind::Path)
                    && facet.resource.as_deref() == path.to_str()
            }) {
                self.facets.push(path_facet(
                    ToolNature::Read,
                    path,
                    "Pull child fetch hook; contents deferred until exact Read admission",
                )?);
            }
        }
        if !pending {
            self.pull_child_hook_evidence =
                git_write::config_state::ConfigState::capture_pull_hooks(
                    &hook_paths.into_iter().collect::<Vec<_>>(),
                )?;
        }
        for (workdir, inventory) in &mut self.pull_child_inventories {
            let location = self
                .child_locations
                .iter()
                .find(|(path, _)| path == workdir)
                .map(|(_, location)| location)
                .ok_or_else(|| {
                    GitToolError::InvalidInput("Pull child metadata roots missing".into())
                })?;
            let metadata_path = |path: &Path| {
                path.starts_with(&location.git_dir) || path.starts_with(&location.common_dir)
            };
            let admitted = |path: &Path| {
                metadata_path(path)
                    || authorizations.iter().any(|token| {
                        token.authorizes_path(
                            "git_pull",
                            ToolNature::Read,
                            &self.workspace,
                            path.to_str().unwrap_or_default(),
                        )
                    })
            };
            // Directory depth within approved metadata roots consumes no new
            // runner stage. External reference frontiers still wait for exact
            // grants, including each newly discovered descendant.
            while !inventory.complete() && inventory.iter_paths().all(&admitted) {
                inventory.advance(admitted)?;
            }
            pending |= !inventory.complete();
            for path in inventory.iter_exact_paths(&location.git_dir, &location.common_dir) {
                if !self.facets.iter().any(|facet| {
                    facet.nature == ToolNature::Read
                        && facet.resource_kind == Some(ToolResourceKind::Path)
                        && facet.resource.as_deref() == path.to_str()
                }) {
                    self.facets.push(path_facet(
                        ToolNature::Read,
                        path,
                        "Pull child object and reference inventory frontier",
                    )?);
                }
            }
        }
        self.identity = PreparedPlanIdentity::new();
        if pending {
            return Ok(self);
        }
        for (workdir, location) in &self.child_locations {
            let inventory = self
                .pull_child_inventories
                .get_mut(workdir)
                .ok_or_else(|| {
                    GitToolError::InvalidInput("Pull child object inventory missing".into())
                })?;
            inventory.bind_alternates(&location.git_dir, &location.common_dir)?;
            inventory.bind_references(&location.git_dir, &location.common_dir)?;
            let discovered = Self::advance_pull_worktree_targets(
                inventory,
                &location.common_dir,
                &mut self.facets,
                &self.workspace,
                authorizations,
                resolution_admitted,
            )?;
            pending |= discovered;
            for path in inventory.iter_exact_paths(&location.git_dir, &location.common_dir) {
                if !self.facets.iter().any(|facet| {
                    facet.nature == ToolNature::Read
                        && facet.resource_kind == Some(ToolResourceKind::Path)
                        && facet.resource.as_deref() == path.to_str()
                }) {
                    self.facets.push(path_facet(
                        ToolNature::Read,
                        path,
                        "Pull linked worktree target or reference frontier; descendant reads deferred",
                    )?);
                }
            }
        }
        if pending {
            return Ok(self);
        }
        self.validate_pull_child_object_types().await?;
        if self
            .child_locations
            .iter()
            .all(|(path, _)| self.pull_children_fetched.contains(path))
        {
            self.phase = Phase::PullIntegration;
        } else {
            for (path, location) in &self.child_locations {
                if self.pull_children_fetched.contains(path) {
                    continue;
                }
                let (_, destinations) =
                    self.pull_child_fetch_targets.get(path).ok_or_else(|| {
                        GitToolError::InvalidInput("Pull child fetch destination missing".into())
                    })?;
                for destination in destinations {
                    self.facets.push(
                        ToolPermissionFacet::with_resource(
                            ToolNature::Network,
                            destination.clone(),
                            ToolResourceKind::Remote,
                        )
                        .with_description("Pull pending child fetch destination"),
                    );
                }
                for root in [&location.git_dir, &location.common_dir] {
                    self.facets.push(path_facet(
                        ToolNature::Write,
                        root,
                        "Pull child fetch writes under this identity-bound metadata root",
                    )?);
                }
            }
            self.facets.push(
                ToolPermissionFacet::with_resource(
                    ToolNature::Write,
                    "git:pull-repository-metadata",
                    ToolResourceKind::Remote,
                )
                .with_description(
                    "Child fetch only; native recursion disabled; integration needs fresh approval",
                ),
            );
            self.phase = Phase::PullChildFetch;
        }
        Ok(self)
    }

    pub(super) async fn execute_pull_child_fetch(
        mut self: Box<Self>,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<Box<Self>, GitToolError> {
        self.revalidate()?;
        if !self.facets.iter().any(|facet| {
            facet.nature == ToolNature::Write
                && facet.resource_kind == Some(ToolResourceKind::Remote)
                && facet.resource.as_deref() == Some("git:pull-repository-metadata")
        }) {
            return Err(GitToolError::InvalidInput(
                "Pull child metadata Write capability missing".into(),
            ));
        }
        // Check every selected root and transport before starting the first child.
        for (workdir, location) in &self.child_locations {
            if self.pull_children_fetched.contains(workdir) {
                continue;
            }
            // Parent graph evidence cannot be silently rebased by a child fetch.
            if self.pull_inventory.as_ref().is_some_and(|inventory| {
                inventory.iter_paths().any(|path| {
                    [&location.git_dir, &location.common_dir]
                        .iter()
                        .any(|root| path.starts_with(root) || root.starts_with(path))
                })
            }) {
                return Err(GitToolError::InvalidInput(
                    "Pull child fetch overlaps retained parent graph; coordinated refresh required"
                        .into(),
                ));
            }
            for root in [&location.git_dir, &location.common_dir] {
                if !authorizations.iter().any(|token| {
                    token.authorizes_path(
                        "git_pull",
                        ToolNature::Write,
                        &self.workspace,
                        root.to_str().unwrap_or_default(),
                    )
                }) {
                    return Err(GitToolError::InvalidInput(
                        "Pull child metadata root lacks Write admission".into(),
                    ));
                }
            }
            let (_, destinations) =
                self.pull_child_fetch_targets.get(workdir).ok_or_else(|| {
                    GitToolError::InvalidInput("Pull child fetch destination missing".into())
                })?;
            if destinations.is_empty()
                || !destinations
                    .iter()
                    .all(|destination| pull_network_transport_supported(destination))
            {
                return Err(GitToolError::InvalidInput("Pull child transport requires separately validated path, helper, credential, or network effects".into()));
            }
        }
        if !cfg!(test) {
            return Err(GitToolError::InvalidInput("Pull child execution awaits recursive retry, integration and independent activation gates; no child fetch started".into()));
        }
        for (workdir, location) in &self.child_locations {
            if self.pull_children_fetched.contains(workdir) {
                continue;
            }
            self.revalidate()?;
            let (worktree, _) = self.pull_child_worktrees.get(workdir).ok_or_else(|| {
                GitToolError::InvalidInput("Pull child effective worktree missing".into())
            })?;
            let objects = self.pull_child_objects(workdir);
            let unconditional = self.pull_child_unconditional(workdir);
            // Never use discovery's presence result across the write approval.
            if !unconditional
                && self
                    .pull_child_has_commits(workdir, &location.git_dir, worktree, &objects)
                    .await?
            {
                self.pull_children_fetched.insert(workdir.clone());
                continue;
            }
            self.revalidate()?;
            let (remote, _) = self.pull_child_fetch_targets.get(workdir).ok_or_else(|| {
                GitToolError::InvalidInput("Pull child fetch remote missing".into())
            })?;
            let mut command = self.pull_child_fetch_command(workdir, &location.git_dir)?;
            command.args([
                "-c",
                "maintenance.autoDetach=false",
                "-c",
                "gc.autoDetach=false",
                "fetch",
                "--no-recurse-submodules",
                "--",
                remote,
            ]);
            let output = self
                .run_command_output(command, std::time::Duration::from_secs(120), 1024 * 1024)
                .await?;
            if !output.status.success() {
                return Err(GitToolError::Git("Pull child fetch failed; previously fetched metadata is retained; inspect repository before retrying".into()));
            }
            // Refresh every inventory that observes the mutated roots, including
            // linked siblings sharing a common object/ref store. No further child
            // may run until this evidence has converged under Read admission.
            let roots = [&location.git_dir, &location.common_dir];
            let affected: Vec<_> = self
                .pull_child_inventories
                .iter()
                .filter(|(_, inventory)| {
                    inventory.iter_paths().any(|path| {
                        roots
                            .iter()
                            .any(|root| path.starts_with(root) || root.starts_with(path))
                    })
                })
                .map(|(path, _)| path.clone())
                .collect();
            self.pull_children_fetched.insert(workdir.clone());
            for path in affected {
                let location = self
                    .child_locations
                    .iter()
                    .find(|(key, _)| key == &path)
                    .map(|(_, location)| location)
                    .ok_or_else(|| {
                        GitToolError::InvalidInput("Pull affected child location missing".into())
                    })?;
                self.pull_child_inventories.insert(
                    path,
                    git_write::pull_inventory::Inventory::with_reference_budget(
                        &location.git_dir,
                        &location.common_dir,
                        self.pull_reference_budget.clone(),
                    )?,
                );
            }
            break;
        }
        self.facets
            .retain(|facet| !matches!(facet.nature, ToolNature::Write | ToolNature::Network));
        self.identity = PreparedPlanIdentity::new();
        self.phase = Phase::PullChildConfiguration;
        self.discover_pull_child_configuration(authorizations).await
    }

    pub(super) async fn validate_pull_child_object_types(&self) -> Result<(), GitToolError> {
        for (workdir, location) in &self.child_locations {
            let objects = self.pull_child_objects(workdir);
            let (worktree, _) = self.pull_child_worktrees.get(workdir).ok_or_else(|| {
                GitToolError::InvalidInput("Pull child effective worktree missing".into())
            })?;
            // Only reject invalid types here. Presence must be re-probed at the
            // fetch consumer; never retain it across the next approval.
            self.pull_child_has_commits(workdir, &location.git_dir, worktree, &objects)
                .await?;
        }
        Ok(())
    }

    pub(super) async fn pull_child_has_commits(
        &self,
        workdir: &Path,
        gitdir: &Path,
        worktree: &Path,
        objects: &[String],
    ) -> Result<bool, GitToolError> {
        if objects.is_empty() {
            return Ok(true);
        }
        if !self
            .pull_child_commit_objects(workdir, gitdir, worktree, objects)
            .await?
        {
            return Ok(false);
        }
        // Match native submodule_has_commits: mere object existence is not
        // enough; every requested commit must be reachable from an existing ref.
        // This and cat-file are transient observations, not an atomic snapshot.
        self.revalidate()?;
        let mut command = self.pull_child_command(workdir, gitdir, worktree)?;
        command.args(["rev-list", "-n", "1", "--stdin", "--not", "--all"]);
        let input = format!("{}\n", objects.join("\n")).into_bytes();
        let output = self
            .run_command_output_with_input(
                command,
                std::time::Duration::from_secs(10),
                16 * 1024,
                Some(input),
            )
            .await?;
        if !output.status.success() {
            return Ok(false);
        }
        if output.stdout.is_empty() {
            return Ok(true);
        }
        let oid = output
            .stdout
            .strip_suffix(b"\n")
            .ok_or_else(|| GitToolError::Git("Invalid Pull child reachability response".into()))?;
        if !matches!(oid.len(), 40 | 64) || !oid.iter().all(u8::is_ascii_hexdigit) {
            return Err(GitToolError::Git(
                "Invalid Pull child reachability response".into(),
            ));
        }
        Ok(false)
    }

    pub(super) fn pull_child_is_historical(&self, selected: &Path) -> bool {
        !self
            .pull_current_fetch_children
            .keys()
            .any(|path| self.repository.workdir.join(path) == selected)
            && self
                .pull_changed_children
                .values()
                .any(|child| self.repository.workdir.join(&child.path) == selected)
    }

    pub(super) fn pull_child_fetch_command(
        &self,
        selected: &Path,
        gitdir: &Path,
    ) -> Result<tokio::process::Command, GitToolError> {
        let mut command = tokio::process::Command::new(&self.program);
        command
            .current_dir(gitdir)
            .arg("--no-pager")
            .arg("--git-dir")
            .arg(host_path::directory_argument(gitdir)?.as_os_str())
            .env_clear()
            .envs(&self.environment);
        // Git schedules index tasks first, including OnDemand tasks. Only
        // historical tasks override worktree setup; this is unrelated to the
        // unconditional-fetch policy and to whether a checkout is populated.
        if self.pull_child_is_historical(selected) {
            command.arg("--work-tree=.");
        }
        Ok(command)
    }

    // Object probes use their independently bound layout, not fetch scheduling.
    pub(super) fn pull_child_command(
        &self,
        workdir: &Path,
        gitdir: &Path,
        worktree: &Path,
    ) -> Result<tokio::process::Command, GitToolError> {
        let mut command = tokio::process::Command::new(&self.program);
        command
            .current_dir(workdir)
            .arg("--no-pager")
            .arg("--git-dir")
            .arg(host_path::directory_argument(gitdir)?.as_os_str())
            .arg("--work-tree")
            .arg(host_path::directory_argument(worktree)?.as_os_str())
            .env_clear()
            .envs(&self.environment);
        Ok(command)
    }

    pub(super) async fn pull_child_commit_objects(
        &self,
        workdir: &Path,
        gitdir: &Path,
        worktree: &Path,
        objects: &[String],
    ) -> Result<bool, GitToolError> {
        if objects.is_empty() {
            return Ok(true);
        }
        let mut input = Vec::new();
        for oid in objects {
            if !matches!(oid.len(), 40 | 64) || !oid.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Err(GitToolError::InvalidInput(
                    "Invalid Pull child object ID".into(),
                ));
            }
            input.extend_from_slice(oid.as_bytes());
            input.push(b'\n');
            if input.len() > 256 * 1024 {
                return Err(GitToolError::InvalidInput(
                    "Pull child object input limit exceeded".into(),
                ));
            }
        }
        self.revalidate()?;
        let mut command = self.pull_child_command(workdir, gitdir, worktree)?;
        command.args(["cat-file", "--batch-check=%(objectname) %(objecttype)"]);
        let output = self
            .run_command_output_with_input(
                command,
                std::time::Duration::from_secs(10),
                1024 * 1024,
                Some(input),
            )
            .await?;
        if !output.status.success() {
            return Err(GitToolError::Git(
                "Pull child object inspection failed".into(),
            ));
        }
        git_write::pull_children::commit_batch_output(objects, &output.stdout)
    }

    pub(super) fn bind_pull_child_worktree(
        &mut self,
        workdir: &Path,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<(), GitToolError> {
        let startup = self.pull_child_startup.get(workdir).ok_or_else(|| {
            GitToolError::InvalidInput("Pull child startup evidence missing".into())
        })?;
        startup.revalidate()?;
        let target = if self.pull_child_is_historical(workdir) {
            self.child_locations
                .iter()
                .find(|(path, _)| path == workdir)
                .map(|(_, location)| location.git_dir.clone())
                .ok_or_else(|| {
                    GitToolError::InvalidInput("Pull historical child metadata missing".into())
                })?
        } else {
            startup
                .targets
                .iter()
                .next()
                .cloned()
                .unwrap_or_else(|| workdir.to_owned())
        };
        let admitted = self.child_read(&target, authorizations)?;
        let identity = snapshot::PathIdentity::capture(&target).map_err(|_| {
            GitToolError::InvalidInput("Pull child startup worktree identity unavailable".into())
        })?;
        if target.canonicalize().ok().as_deref() != Some(admitted) || !target.is_dir() {
            return Err(GitToolError::InvalidInput(
                "Pull child startup worktree changed after admission".into(),
            ));
        }
        self.pull_child_worktrees
            .insert(workdir.to_owned(), (target, identity));
        Ok(())
    }

    pub(super) async fn resolve_pull_child_activity(&mut self) -> Result<(), GitToolError> {
        use git_write::pull_children::Activity;
        let mut index_rules = BTreeMap::new();
        for path in &self.pull_attribute_paths {
            let Some(relative) = path
                .strip_prefix(&self.repository.workdir)
                .ok()
                .and_then(Path::to_str)
            else {
                continue;
            };
            let listing = self
                .pull_probe(vec![
                    "ls-files".into(),
                    "--stage".into(),
                    "--".into(),
                    relative.to_owned(),
                ])
                .await?;
            if !listing.status.success() || !listing.stderr.is_empty() {
                return Err(GitToolError::Git(
                    "Pull index attribute lookup failed".into(),
                ));
            }
            if !listing.stdout.is_empty() {
                let output = self
                    .pull_probe(vec![
                        "cat-file".into(),
                        "blob".into(),
                        format!(":{relative}"),
                    ])
                    .await?;
                if !output.status.success() || !output.stderr.is_empty() {
                    return Err(GitToolError::Git(
                        "Pull index attribute blob read failed".into(),
                    ));
                }
                index_rules.insert(path.clone(), output.stdout);
            }
        }
        for (name, child) in &mut self.pull_changed_children {
            let Some(Activity::Pathspec { attributes }) = &child.activity else {
                continue;
            };
            let mut sources: Vec<_> = self
                .pull_activity_globals
                .iter()
                .map(|path| (path.clone(), false, true))
                .collect();
            sources.extend(
                attributes
                    .iter()
                    .enumerate()
                    .map(|(depth, path)| (self.repository.workdir.join(path), true, depth == 0)),
            );
            sources.push((
                self.repository.common_dir.join("info/attributes"),
                false,
                true,
            ));
            let (rules, metadata) = std::panic::catch_unwind(|| {
                let mut rules = gix::attrs::Search::default();
                let mut metadata = gix::attrs::search::MetadataCollection::default();
                rules.add_patterns_buffer(
                    b"[attr]binary -diff -merge -text",
                    "[builtin]".into(),
                    None,
                    &mut metadata,
                    true,
                );
                for (path, local, macros) in sources {
                    let evidence = self
                        .pull_attributes
                        .iter()
                        .find(|entry| entry.path == path)
                        .ok_or_else(|| {
                            GitToolError::Git("Missing admitted Pull activity attributes".into())
                        })?;
                    match &evidence.evidence.entry {
                        snapshot::Entry::File(_, bytes) => rules.add_patterns_buffer(
                            bytes,
                            path,
                            local.then_some(self.repository.workdir.as_path()),
                            &mut metadata,
                            macros,
                        ),
                        snapshot::Entry::Missing if !local => {}
                        // Git may fall back to the index for a missing
                        // worktree attribute file. This stage has not yet
                        // admitted the index blob, so refuse rather than
                        // silently evaluating an incomplete rule set.
                        snapshot::Entry::Missing => {
                            if let Some(bytes) = index_rules.get(&path) {
                                rules.add_patterns_buffer(
                                    bytes,
                                    path,
                                    Some(self.repository.workdir.as_path()),
                                    &mut metadata,
                                    macros,
                                );
                            }
                        }
                        _ => {
                            return Err(GitToolError::InvalidInput(
                                "Invalid Pull activity attribute evidence".into(),
                            ));
                        }
                    }
                }
                Ok((rules, metadata))
            })
            .map_err(|_| GitToolError::Git("Pull attribute parser panicked".into()))??;
            let active = self
                .pull_before
                .as_ref()
                .ok_or_else(|| GitToolError::Git("Missing Pull activity policy".into()))?
                .child_active_with_attributes(
                    name,
                    &child.path,
                    &self.repository.workdir,
                    Some((&rules, &metadata)),
                )?
                .ok_or_else(|| GitToolError::Git("Unresolved Pull activity attributes".into()))?;
            child.activity = Some(if active {
                Activity::Enabled
            } else {
                Activity::Disabled
            });
        }
        Ok(())
    }

    // Fixed path-only discovery: reported executable contents require a new proof.
}
