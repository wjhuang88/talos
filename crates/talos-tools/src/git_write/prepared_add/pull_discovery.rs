//! Prepared Pull discovery stages.

use super::super as git_write;
use super::*;

impl AddPlan {
    pub(super) async fn validate_pull_source_filters(&self) -> Result<(), GitToolError> {
        if let Some(sources) = &self.pull_sources {
            for batch in sources.paths.chunks(16) {
                let mut args = vec![
                    "check-attr".into(),
                    "-z".into(),
                    "filter".into(),
                    "--".into(),
                ];
                args.extend(batch.iter().cloned());
                let output = self.pull_probe(args).await?;
                if !output.status.success() || !output.stderr.is_empty() {
                    return Err(GitToolError::Git("Pull filter inspection failed".into()));
                }
                let fields = output.stdout.split(|byte| *byte == 0).collect::<Vec<_>>();
                if fields.last() != Some(&b"".as_slice()) || fields.len() - 1 != batch.len() * 3 {
                    return Err(GitToolError::InvalidInput(
                        "Invalid Pull filter inspection output".into(),
                    ));
                }
                for (fields, path) in fields[..fields.len() - 1]
                    .as_chunks::<3>()
                    .0
                    .iter()
                    .zip(batch)
                {
                    if fields[0] != path.as_bytes()
                        || fields[1] != b"filter"
                        || !self
                            .config
                            .as_ref()
                            .is_some_and(|config| config.filter_attribute_is_inert(fields[2]))
                    {
                        return Err(GitToolError::InvalidInput("Pull tracked source uses a configured filter; requires separate authorization".into()));
                    }
                }
                self.revalidate()?;
            }
        }
        Ok(())
    }

    pub(super) async fn pull_probe(
        &self,
        args: Vec<String>,
    ) -> Result<std::process::Output, GitToolError> {
        // Tree listings carry one record per path, unlike small scalar probes.
        // Keep repeated listings within the unchanged cumulative output budget.
        let output_limit = if args.first().is_some_and(|arg| {
            matches!(
                arg.as_str(),
                "ls-tree" | "for-each-ref" | "rev-list" | "cat-file" | "diff-tree"
            )
        }) {
            1024 * 1024
        } else {
            16 * 1024
        };
        self.revalidate()?;
        let mut command = tokio::process::Command::new(&self.program);
        command
            .current_dir(&self.repository.workdir)
            .arg("--no-pager")
            .args(args)
            .env_clear()
            .envs(&self.environment);
        self.run_command_output(command, std::time::Duration::from_secs(10), output_limit)
            .await
    }

    pub(super) fn pull_resolution_admitted(facets: &[ToolPermissionFacet]) -> bool {
        facets.iter().any(|facet| {
            facet.nature == ToolNature::Read
                && facet.resource_kind == Some(ToolResourceKind::Remote)
                && facet.resource.as_deref() == Some("git:pull-worktree-resolution")
        })
    }

    // Only called after execute_stage consumes the complete current prepared
    // proof. Remote capabilities are covered by that profile proof; the tokens
    // returned from it additionally bind exact target paths below.
    pub(super) fn advance_pull_worktree_targets(
        inventory: &mut git_write::pull_inventory::Inventory,
        common: &Path,
        facets: &mut Vec<ToolPermissionFacet>,
        workspace: &Path,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
        admitted_resolution: bool,
    ) -> Result<bool, GitToolError> {
        let discovered = inventory.discover_worktree_targets(common)?;
        if !inventory.has_worktree_targets() {
            return Ok(false);
        }
        if !admitted_resolution && !Self::pull_resolution_admitted(facets) {
            facets.push(ToolPermissionFacet::with_resource(
                ToolNature::Read,
                "git:pull-worktree-resolution",
                ToolResourceKind::Remote,
                ).with_description("resolve retained linked-worktree pointers, including external component metadata and link text; no file contents, directory enumeration, execution or writes"));
        }
        // Freeze newly discovered inputs in a fresh plan before inspecting
        // their chains, even when an earlier pointer used this capability.
        if discovered || !admitted_resolution {
            return Ok(true);
        }
        if inventory.resolve_worktree_targets()? {
            // Caller publishes resolved exact Path facets for a fresh approval.
            return Ok(true);
        }
        inventory.bind_worktree_targets(|path| {
            authorizations
                .iter()
                .find(|token| {
                    token.authorizes_path(
                        "git_pull",
                        ToolNature::Read,
                        workspace,
                        path.to_str().unwrap_or_default(),
                    )
                })
                .map(|token| token.normalized_path().to_owned())
        })
    }

    pub(super) fn advance_pull_parent_inventory(
        &mut self,
        inventory: &mut git_write::pull_inventory::Inventory,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<bool, GitToolError> {
        let gitdir = &self.repository.git_dir;
        let common = &self.repository.common_dir;
        let exact = |path: &Path| {
            authorizations.iter().find(|token| {
                token.authorizes_path(
                    "git_pull",
                    ToolNature::Read,
                    &self.workspace,
                    path.to_str().unwrap_or_default(),
                )
            })
        };
        let admitted = |path: &Path| {
            path.starts_with(gitdir) || path.starts_with(common) || exact(path).is_some()
        };
        let mut pending = inventory
            .iter_exact_paths(gitdir, common)
            .any(|path| exact(path).is_none());
        if !pending {
            while !inventory.complete() && inventory.iter_paths().all(&admitted) {
                inventory.advance(admitted)?;
            }
            pending = !inventory.complete();
            if !pending {
                inventory.bind_alternates(gitdir, common)?;
                inventory.bind_references(gitdir, common)?;
                let resolution_admitted = Self::pull_resolution_admitted(&self.facets);
                pending = Self::advance_pull_worktree_targets(
                    inventory,
                    common,
                    &mut self.facets,
                    &self.workspace,
                    authorizations,
                    resolution_admitted,
                )?;
            }
        }
        for path in inventory.iter_exact_paths(gitdir, common) {
            if !self.facets.iter().any(|facet| {
                facet.nature == ToolNature::Read
                    && facet.resource_kind == Some(ToolResourceKind::Path)
                    && facet.resource.as_deref() == path.to_str()
            }) {
                self.facets.push(path_facet(
                    ToolNature::Read,
                    path,
                    "Pull parent linked-worktree target or reference frontier; reads deferred",
                )?);
            }
        }
        Ok(!pending)
    }

    pub(super) async fn discover_pull_prefetch(
        mut self: Box<Self>,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<Box<Self>, GitToolError> {
        self.revalidate()?;
        // execute_stage consumed the immutable profile proof before entering
        // discovery. The closed capability additionally requires each retained
        // metadata root's exact Read token; it never covers worktree sources.
        if !self.facets.iter().any(|facet| {
            facet.nature == ToolNature::Read
                && facet.resource_kind == Some(ToolResourceKind::Remote)
                && facet.resource.as_deref() == Some("git:pull-repository-metadata")
        }) || [&self.repository.git_dir, &self.repository.common_dir]
            .iter()
            .any(|root| {
                !authorizations.iter().any(|token| {
                    token.authorizes_path(
                        "git_pull",
                        ToolNature::Read,
                        &self.workspace,
                        root.to_str().unwrap_or_default(),
                    )
                })
            })
        {
            return Err(GitToolError::InvalidInput(
                "Pull graph discovery lacks dedicated metadata Read admission".into(),
            ));
        }
        let metadata_roots = [
            self.repository.git_dir.clone(),
            self.repository.common_dir.clone(),
        ];
        let metadata_path = |path: &Path| metadata_roots.iter().any(|root| path.starts_with(root));
        if self.pull_graph_paths.is_none() {
            let mut inventory = self
                .pull_inventory
                .take()
                .ok_or_else(|| GitToolError::Git("Missing Pull graph inventory".into()))?;
            if !self.advance_pull_parent_inventory(&mut inventory, authorizations)? {
                self.pull_inventory = Some(inventory);
                self.identity = PreparedPlanIdentity::new();
                return Ok(self);
            }
            let index_path = self.repository.git_dir.join("index");
            if !authorizations.iter().any(|token| {
                token.authorizes_path(
                    "git_pull",
                    ToolNature::Read,
                    &self.workspace,
                    index_path.to_str().unwrap_or_default(),
                )
            }) {
                return Err(GitToolError::InvalidInput(
                    "Pull index discovery lacks exact Read admission".into(),
                ));
            }
            let format = self
                .pull_probe(vec!["rev-parse".into(), "--show-object-format".into()])
                .await?;
            let length = match (
                format.status.success(),
                format.stderr.is_empty(),
                format.stdout.as_slice(),
            ) {
                (true, true, b"sha1\n") => 40,
                (true, true, b"sha256\n") => 64,
                _ => {
                    return Err(GitToolError::Git(
                        "Pull index object format unavailable".into(),
                    ));
                }
            };
            let (index, _) = git_write::pull_state::decode_index(&index_path, length)?;
            let branch_output = self
                .pull_probe(vec![
                    "symbolic-ref".into(),
                    "--quiet".into(),
                    "--no-recurse".into(),
                    "HEAD".into(),
                ])
                .await?;
            let branch = match branch_output.status.code() {
                Some(0) if branch_output.stderr.is_empty() => Some(
                    std::str::from_utf8(&branch_output.stdout)
                        .ok()
                        .and_then(|text| text.strip_prefix("refs/heads/"))
                        .and_then(|text| text.strip_suffix('\n'))
                        .ok_or_else(|| {
                            GitToolError::InvalidInput("Invalid Pull branch discovery".into())
                        })?,
                ),
                Some(1) if branch_output.stdout.is_empty() && branch_output.stderr.is_empty() => {
                    None
                }
                _ => {
                    return Err(GitToolError::Git(
                        "Pull policy branch discovery failed".into(),
                    ));
                }
            };
            let config = self
                .config
                .as_ref()
                .ok_or_else(|| GitToolError::Git("Missing Pull configuration".into()))?;
            let needs_sources = git_write::pull_state::BeforeFetch::policy_requires_clean_worktree(
                &config.pull_records()?,
                branch,
            )?;
            let objects_ready = {
                [&self.repository.git_dir, &self.repository.common_dir]
                    .iter()
                    .all(|root| {
                        let objects = root.join("objects");
                        // Replacement refs may be symbolic and resolve into any
                        // refs subtree, so admitting refs/replace alone is insufficient.
                        let references = root.join("refs");
                        let packed = root.join("packed-refs");
                        inventory.subtree_complete(&objects)
                            && inventory.subtree_complete(&references)
                            && inventory.subtree_complete(&packed)
                    })
            };
            let mut encoded = Vec::new();
            for entry in index.entries() {
                if !needs_sources {
                    break;
                }
                if entry.mode.bits() == 0o040000 && objects_ready {
                    inventory.revalidate()?;
                    self.config
                        .as_ref()
                        .ok_or_else(|| GitToolError::Git("Missing Pull configuration".into()))?
                        .prepare_pull_inspection()?;
                    let output = self
                        .pull_probe(vec![
                            "ls-tree".into(),
                            "-r".into(),
                            "-z".into(),
                            entry.id.to_string(),
                        ])
                        .await?;
                    inventory.revalidate()?;
                    if !output.status.success() || !output.stderr.is_empty() {
                        return Err(GitToolError::Git(
                            "Pull sparse tree discovery failed".into(),
                        ));
                    }
                    encoded.extend(pull_sparse_names(entry.path(&index), &output.stdout)?);
                    continue;
                }
                if matches!(entry.mode.bits(), 0o040000 | 0o160000) {
                    continue;
                }
                encoded.extend_from_slice(entry.path(&index));
                encoded.push(0);
            }
            let mut source_paths = pull_tracked_read_paths(&self.repository.workdir, &encoded)?;
            let mut attributes = std::collections::BTreeSet::new();
            for source in &source_paths {
                let mut parent = source.parent();
                while let Some(directory) = parent {
                    if !directory.starts_with(&self.repository.workdir) {
                        break;
                    }
                    attributes.insert(directory.join(".gitattributes"));
                    parent = directory.parent();
                }
            }
            if needs_sources {
                attributes.insert(self.repository.common_dir.join("info/attributes"));
            }
            // Post-fetch combined diffs also honor diff.orderFile, even when
            // pre-fetch cleanliness checks do not need worktree sources.
            if let Some(path) =
                config.pull_diff_order_file(&self.repository.workdir, &self.environment)?
            {
                attributes.insert(path);
            }
            for variable in ["GIT_ATTR_SYSTEM", "GIT_ATTR_GLOBAL"]
                .into_iter()
                .filter(|_| needs_sources)
            {
                attributes.insert(
                    self.inspect_attribute_path_at(&self.repository.workdir, variable)
                        .await?,
                );
            }
            self.pull_attribute_paths = attributes.into_iter().collect();
            source_paths.extend(self.pull_attribute_paths.iter().cloned());
            // Current module declarations are worktree input, never covered by
            // the dedicated metadata capability. Admit even an absent file.
            source_paths.push(self.repository.workdir.join(".gitmodules"));
            let mut paths = inventory.capture_paths()?;
            paths.extend(source_paths);
            if inventory.complete() {
                // Fixed evidence names are known at configuration discovery
                // and already have exact facets, even when absent on disk.
                paths.extend(git_write::pull_state::PullEvidence::required_paths(
                    &self.repository,
                    None,
                    false,
                )?);
                paths.sort();
                paths.dedup();
            }
            let mut added_facets = false;
            for path in &paths {
                if metadata_path(path) && !self.pull_attribute_paths.contains(path) {
                    continue;
                }
                if !self.facets.iter().any(|facet| {
                    facet.nature == ToolNature::Read
                        && facet.resource_kind == Some(ToolResourceKind::Path)
                        && facet.resource.as_deref() == path.to_str()
                }) {
                    self.facets.push(path_facet(
                        ToolNature::Read,
                        path,
                        "Pull graph exact-path discovery and read evidence",
                    )?);
                    added_facets = true;
                }
            }
            let complete = inventory.complete();
            if complete {
                self.pull_graph_paths = Some(paths);
            }
            self.pull_inventory = Some(inventory);
            if !complete || added_facets {
                self.identity = PreparedPlanIdentity::new();
                return Ok(self);
            }
            // Only an unchanged completed frontier can use the current proof.
            // Newly discovered source paths wait for fresh admission above.
            // Inspect under that same
            // admission instead of spending an identical-profile continuation.
        }
        // The dedicated metadata admission covers internal graph evidence.
        // Worktree sources and attribute files still require exact Read tokens.
        let paths = self.pull_graph_paths.as_ref().filter(|paths| !paths.is_empty())
            .ok_or_else(|| GitToolError::InvalidInput(
                "Pull reference/reflog/object graph read inventory requires separate discovery and admission; no prefetch probe performed".into(),
            ))?;
        for path in paths {
            if (!metadata_path(path) || self.pull_attribute_paths.contains(path))
                && !authorizations.iter().any(|token| {
                    token.authorizes_path(
                        "git_pull",
                        ToolNature::Read,
                        &self.workspace,
                        path.to_str().unwrap_or_default(),
                    )
                })
            {
                return Err(GitToolError::InvalidInput(
                    "Pull graph resource lacks exact Read admission".into(),
                ));
            }
        }
        self.inspect_pull_prefetch().await
    }

    pub(super) async fn inspect_pull_prefetch(
        mut self: Box<Self>,
    ) -> Result<Box<Self>, GitToolError> {
        self.revalidate()?;
        let Operation::Pull(pull) = &self.operation else {
            return Err(GitToolError::Git("Invalid Pull discovery operation".into()));
        };
        let config = self
            .config
            .as_ref()
            .ok_or_else(|| GitToolError::Git("missing Pull configuration evidence".into()))?;
        config.prepare_pull_inspection()?;
        let records = config.pull_records()?;
        let format = self
            .pull_probe(vec!["rev-parse".into(), "--show-object-format".into()])
            .await?;
        let oid_length = match (
            format.status.success(),
            format.stderr.is_empty(),
            format.stdout.as_slice(),
        ) {
            (true, true, b"sha1\n") => 40,
            (true, true, b"sha256\n") => 64,
            _ => return Err(GitToolError::Git("Pull object format probe failed".into())),
        };
        let (index, index_evidence) = git_write::pull_state::decode_index(
            &self.repository.git_dir.join("index"),
            oid_length,
        )?;
        if index.entries().iter().any(|entry| entry.stage_raw() != 0) {
            return Err(GitToolError::InvalidInput(
                "Pull requires resolving index conflicts before fetch".into(),
            ));
        }
        let initial =
            git_write::pull_state::BeforeFetch::probe(&records, None, oid_length, |args| {
                self.pull_probe(args)
            })
            .await?;
        let remote = git_write::pull_state::fetch_remote(
            &records,
            initial.branch(),
            pull.remote.as_deref(),
        )?;
        config.prepare_pull_transport(&remote)?;
        let destinations = git_write::remote_state::destinations(
            &self
                .config
                .as_ref()
                .ok_or_else(|| GitToolError::Git("missing Pull config".into()))?
                .pull_remote_records()?,
            &remote,
            false,
        )?;
        let tracking = initial
            .branch()
            .map(|branch| git_write::pull_state::tracking_reference(&records, branch, &remote))
            .transpose()?
            .flatten();
        let mut supplemental_paths_added = false;
        let mut source_names = Vec::new();
        if initial.requires_clean_worktree() && self.pull_graph_paths.is_some() {
            // A symbolic branch chain can resolve to a different policy than
            // direct HEAD discovery. Discover all fixed attribute paths here
            // too; newly needed paths still wait for fresh exact admission.
            let mut fixed_attributes = vec![self.repository.common_dir.join("info/attributes")];
            if let Some(path) =
                config.pull_diff_order_file(&self.repository.workdir, &self.environment)?
            {
                fixed_attributes.push(path);
            }
            for variable in ["GIT_ATTR_SYSTEM", "GIT_ATTR_GLOBAL"] {
                fixed_attributes.push(
                    self.inspect_attribute_path_at(&self.repository.workdir, variable)
                        .await?,
                );
            }
            for path in fixed_attributes {
                if !self.pull_attribute_paths.contains(&path) {
                    self.pull_attribute_paths.push(path);
                }
            }
            let mut encoded = Vec::new();
            for entry in index.entries() {
                if entry.mode.bits() == 0o040000 {
                    let output = self
                        .pull_probe(vec![
                            "ls-tree".into(),
                            "-r".into(),
                            "-z".into(),
                            entry.id.to_string(),
                        ])
                        .await?;
                    if !output.status.success() || !output.stderr.is_empty() {
                        return Err(GitToolError::Git(
                            "Pull sparse tree discovery failed".into(),
                        ));
                    }
                    encoded.extend(pull_sparse_names(entry.path(&index), &output.stdout)?);
                    continue;
                }
                if entry.mode.bits() != 0o160000 {
                    encoded.extend_from_slice(entry.path(&index));
                    encoded.push(0);
                }
            }
            let mut names = pull_tracked_read_paths(&self.repository.workdir, &encoded)?;
            for path in &names {
                source_names.push(
                    path.strip_prefix(&self.repository.workdir)
                        .ok()
                        .and_then(Path::to_str)
                        .ok_or_else(|| {
                            GitToolError::InvalidInput("Invalid Pull source path".into())
                        })?
                        .to_owned(),
                );
                let mut parent = path.parent();
                while let Some(directory) =
                    parent.filter(|directory| directory.starts_with(&self.repository.workdir))
                {
                    let attribute = directory.join(".gitattributes");
                    if !self.pull_attribute_paths.contains(&attribute) {
                        self.pull_attribute_paths.push(attribute);
                    }
                    parent = directory.parent();
                }
            }
            names.extend(self.pull_attribute_paths.iter().cloned());
            if let Some(paths) = &mut self.pull_graph_paths {
                for path in names {
                    if !paths.contains(&path) {
                        self.facets.push(path_facet(
                            ToolNature::Read,
                            &path,
                            "Pull rebase tracked source inspection",
                        )?);
                        paths.push(path);
                        supplemental_paths_added = true;
                    }
                }
            }
        }
        // The separately admitted metadata scope includes absent internal
        // evidence names as well as existing descendants. Retain their paths
        // for revalidation without spending a new exact-path admission stage.
        if let Some(paths) = &mut self.pull_graph_paths {
            for path in git_write::pull_state::PullEvidence::required_paths(
                &self.repository,
                tracking.as_deref(),
                false,
            )? {
                if !path.starts_with(&self.repository.git_dir)
                    && !path.starts_with(&self.repository.common_dir)
                {
                    return Err(GitToolError::InvalidInput(
                        "Pull evidence escaped the admitted metadata roots".into(),
                    ));
                }
                if !paths.contains(&path) {
                    paths.push(path);
                }
            }
        }
        if supplemental_paths_added {
            self.identity = PreparedPlanIdentity::new();
            return Ok(self);
        }
        if self.pull_graph_paths.is_some() {
            let mut budget = snapshot::Budget {
                entries: 4096,
                bytes: 4 * 1024 * 1024,
            };
            self.pull_attributes = self
                .pull_attribute_paths
                .iter()
                .map(|path| PullAttribute::capture(path, &mut budget))
                .collect::<Result<_, _>>()?;
        }
        self.pull_modules_evidence = Some(PullAttribute::capture(
            &self.repository.workdir.join(".gitmodules"),
            &mut snapshot::Budget {
                entries: 1,
                bytes: 1024 * 1024,
            },
        )?);
        if initial.requires_clean_worktree() && self.pull_graph_paths.is_some() {
            self.pull_sources = Some(PullSources::capture(
                &self.repository.workdir,
                source_names,
            )?);
            self.validate_pull_source_filters().await?;
        }
        let evidence = git_write::pull_state::PullEvidence::capture_with_budget(
            &self.repository,
            tracking.as_deref(),
            false,
            &self.pull_reference_budget,
        )?;
        evidence.require_no_pending_merge()?;
        evidence.require_index(&index_evidence)?;
        if initial.requires_empty_initial_index()
            && (!index.entries().is_empty() || evidence.index_timestamp_nonzero())
        {
            return Err(GitToolError::InvalidInput(
                "Pull with rebase cannot update an unborn branch with staged changes".into(),
            ));
        }
        if self.pull_sources.is_some() && !initial.requires_empty_initial_index() {
            for cached in [true, false] {
                self.revalidate()?;
                evidence.revalidate()?;
                let mut command = tokio::process::Command::new(&self.program);
                // Quiet/exit-code can report stale stat data as dirty when
                // refresh is disabled. Numstat performs content comparison;
                // any output (including a mode-only 0/0 record) is a change.
                command
                    .current_dir(&self.repository.workdir)
                    .args([
                        "-c",
                        "diff.autoRefreshIndex=false",
                        "--no-pager",
                        "diff",
                        "--numstat",
                        "-z",
                        "--no-renames",
                        "--no-ext-diff",
                        "--no-textconv",
                    ])
                    .env_clear()
                    .envs(&self.environment)
                    .env("GIT_OPTIONAL_LOCKS", "0");
                if cached {
                    command.args(["--cached", "--ignore-submodules=all", "HEAD", "--"]);
                } else {
                    command.args(["--ignore-submodules=all", "--"]);
                }
                let output = self
                    .run_command_output(command, std::time::Duration::from_secs(10), 16 * 1024)
                    .await?;
                evidence.revalidate()?;
                self.revalidate()?;
                match output.status.code() {
                    Some(0) if output.stderr.is_empty() && output.stdout.is_empty() => {}
                    // A successful diff with records proves the source is
                    // dirty even when Git also warns about line conversion.
                    // Refuse without exposing diagnostic contents. A clean
                    // result still requires empty stdout and stderr above.
                    Some(0) if !output.stdout.is_empty() => {
                        return Err(GitToolError::InvalidInput(
                            if cached {
                                "Pull with rebase requires a clean index before fetch"
                            } else {
                                "Pull with rebase requires a clean worktree before fetch"
                            }
                            .into(),
                        ));
                    }
                    _ => {
                        return Err(GitToolError::Git(
                            "Pull clean-worktree inspection failed".into(),
                        ));
                    }
                }
            }
        }

        let before = git_write::pull_state::BeforeFetch::probe(
            &records,
            tracking.as_deref(),
            oid_length,
            |args| self.pull_probe(args),
        )
        .await?;
        if !before.same_checkout(&initial) {
            return Err(GitToolError::InvalidInput(
                "Pull branch or HEAD changed during upstream discovery".into(),
            ));
        }
        let refs = self
            .pull_probe(vec!["for-each-ref".into(), "--format=%(objectname)".into()])
            .await?;
        if !refs.status.success() || !refs.stderr.is_empty() {
            return Err(GitToolError::Git(
                "Pull pre-fetch reference listing failed".into(),
            ));
        }
        self.pull_ref_tips = Some(git_write::pull_state::graph_oids(&refs.stdout, oid_length)?);
        evidence.revalidate()?;
        self.revalidate()?;
        let recursion = before.fetch_recursion_description();
        self.pull_before = Some(before);
        self.pull_evidence = Some(evidence);
        self.pull_remote = Some(remote);
        self.pull_destinations = destinations.clone();
        self.pull_transport_supported = destinations
            .iter()
            .all(|destination| pull_network_transport_supported(destination));
        for destination in destinations {
            self.facets.push(
                ToolPermissionFacet::with_resource(
                    ToolNature::Network,
                    destination,
                    ToolResourceKind::Remote,
                )
                .with_description("Pull fetch network destination"),
            );
        }
        // Fetch writes only identity-bound repository metadata. Integration has
        // a separate later plan; this facet is not worktree authority.
        self.facets.push(path_facet(
            ToolNature::Write,
            &self.repository.git_dir,
            "Pull fetch metadata writes under the identity-bound gitdir",
        )?);
        if self.repository.common_dir != self.repository.git_dir {
            self.facets.push(path_facet(
                ToolNature::Write,
                &self.repository.common_dir,
                "Pull fetch metadata writes under the identity-bound common dir",
            )?);
        }
        self.facets.push(ToolPermissionFacet::with_resource(
            ToolNature::Write,
            "git:pull-repository-metadata",
            ToolResourceKind::Remote,
        ).with_description(format!(
            "Parent fetch only; native recursion disabled. Retained recursive-fetch policy: {recursion}; child effects require fresh admission"
        )));
        self.identity = PreparedPlanIdentity::new();
        self.phase = Phase::PullFetch;
        Ok(self)
    }

    pub(super) async fn discover_pull_postfetch(
        mut self: Box<Self>,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<Box<Self>, GitToolError> {
        self.repository.revalidate_location()?;
        if !self.facets.iter().any(|facet| {
            facet.nature == ToolNature::Read
                && facet.resource_kind == Some(ToolResourceKind::Remote)
                && facet.resource.as_deref() == Some("git:pull-repository-metadata")
        }) || [&self.repository.git_dir, &self.repository.common_dir]
            .iter()
            .any(|root| {
                !authorizations.iter().any(|token| {
                    token.authorizes_path(
                        "git_pull",
                        ToolNature::Read,
                        &self.workspace,
                        root.to_str().unwrap_or_default(),
                    )
                })
            })
        {
            return Err(GitToolError::InvalidInput(
                "Post-fetch discovery lacks dedicated metadata Read admission".into(),
            ));
        }
        let mut inventory = match self.pull_inventory.take() {
            Some(inventory) => inventory,
            None => git_write::pull_inventory::Inventory::with_reference_budget(
                &self.repository.git_dir,
                &self.repository.common_dir,
                self.pull_reference_budget.clone(),
            )?,
        };
        if !self.advance_pull_parent_inventory(&mut inventory, authorizations)? {
            self.pull_inventory = Some(inventory);
            self.identity = PreparedPlanIdentity::new();
            return Ok(self);
        }
        let config = self
            .config
            .as_ref()
            .ok_or_else(|| GitToolError::Git("Missing Pull configuration".into()))?;
        config.revalidate()?;
        let records = config.pull_records()?;
        let before = self
            .pull_before
            .as_ref()
            .ok_or_else(|| GitToolError::Git("Missing Pull pre-fetch state".into()))?;
        let remote = self
            .pull_remote
            .as_deref()
            .ok_or_else(|| GitToolError::Git("Missing Pull remote".into()))?;
        let tracking = before
            .branch()
            .map(|branch| git_write::pull_state::tracking_reference(&records, branch, remote))
            .transpose()?
            .flatten();
        let evidence = git_write::pull_state::PullEvidence::capture_with_budget(
            &self.repository,
            tracking.as_deref(),
            true,
            &self.pull_reference_budget,
        )?;
        self.repository.revalidate_location()?;
        let repository = RepositoryState::capture(&self.repository.workdir)?;
        evidence.revalidate()?;
        inventory.revalidate()?;
        self.repository = repository;
        self.pull_evidence = Some(evidence);
        self.pull_inventory = Some(inventory);
        self.revalidate()?;
        let format = self
            .pull_probe(vec!["rev-parse".into(), "--show-object-format".into()])
            .await?;
        let width = match (
            format.status.success(),
            format.stderr.is_empty(),
            format.stdout.as_slice(),
        ) {
            (true, true, b"sha1\n") => 40,
            (true, true, b"sha256\n") => 64,
            _ => {
                return Err(GitToolError::Git(
                    "Pull post-fetch object format unavailable".into(),
                ));
            }
        };
        let (index, index_evidence) =
            git_write::pull_state::decode_index(&self.repository.git_dir.join("index"), width)?;
        let mut candidates = git_write::pull_children::Index::parse(&index, width)?;
        for (prefix, oid) in &candidates.sparse {
            let listing = self
                .pull_probe(vec![
                    "ls-tree".into(),
                    "-r".into(),
                    "-z".into(),
                    "--full-tree".into(),
                    oid.clone(),
                ])
                .await?;
            if !listing.status.success() || !listing.stderr.is_empty() {
                return Err(GitToolError::Git(
                    "Pull index child expansion failed".into(),
                ));
            }
            let tree = git_write::pull_children::Tree::parse(&listing.stdout, width)?;
            for (path, oid) in tree.gitlinks {
                let joined = format!("{prefix}/{path}");
                if candidates.gitlinks.insert(joined, oid).is_some() {
                    return Err(GitToolError::InvalidInput(
                        "Duplicate Pull index child".into(),
                    ));
                }
            }
        }
        self.pull_evidence
            .as_ref()
            .ok_or_else(|| GitToolError::Git("Missing post-fetch index evidence".into()))?
            .require_index(&index_evidence)?;
        self.pull_index_children = candidates.gitlinks;
        let modules_evidence = self
            .pull_modules_evidence
            .as_ref()
            .ok_or_else(|| GitToolError::Git("Missing current Pull module Read evidence".into()))?;
        let mut current_modules = git_write::pull_children::Tree::parse(b"", width)?;
        match &modules_evidence.evidence.entry {
            snapshot::Entry::File(_, bytes) => {
                current_modules.bind_module_declarations(bytes, true)?;
            }
            snapshot::Entry::Missing => {
                // Git falls back from the missing worktree file to index, then
                // HEAD. All object reads stay inside the admitted parent roots.
                let mut blob = candidates.modules_blob;
                if blob.is_none() {
                    let head = self
                        .pull_probe(vec![
                            "rev-parse".into(),
                            "--verify".into(),
                            "--quiet".into(),
                            "HEAD".into(),
                        ])
                        .await?;
                    match head.status.code() {
                        Some(0) if head.stderr.is_empty() => {
                            let oid = git_write::pull_state::graph_oids(&head.stdout, width)?;
                            if oid.len() != 1 {
                                return Err(GitToolError::Git("Invalid current Pull HEAD".into()));
                            }
                            let listing = self
                                .pull_probe(vec![
                                    "ls-tree".into(),
                                    "-z".into(),
                                    "--full-tree".into(),
                                    oid[0].clone(),
                                    "--".into(),
                                    ".gitmodules".into(),
                                ])
                                .await?;
                            if !listing.status.success() || !listing.stderr.is_empty() {
                                return Err(GitToolError::Git(
                                    "Current Pull module tree lookup failed".into(),
                                ));
                            }
                            blob = git_write::pull_children::Tree::parse(&listing.stdout, width)?
                                .modules_blob;
                        }
                        Some(1) if head.stdout.is_empty() && head.stderr.is_empty() => {}
                        _ => {
                            return Err(GitToolError::Git(
                                "Current Pull HEAD lookup failed".into(),
                            ));
                        }
                    }
                }
                if let Some(blob) = blob {
                    let output = self
                        .pull_probe(vec!["cat-file".into(), "blob".into(), blob])
                        .await?;
                    if !output.status.success() || !output.stderr.is_empty() {
                        return Err(GitToolError::Git(
                            "Current Pull module blob read failed".into(),
                        ));
                    }
                    current_modules.bind_module_declarations(&output.stdout, true)?;
                }
            }
            _ => {
                return Err(GitToolError::InvalidInput(
                    "Current Pull module file is not regular".into(),
                ));
            }
        }
        self.pull_current_modules = Some(current_modules);
        let fetched = git_write::pull_state::fetched_tips(
            self.pull_evidence
                .as_ref()
                .ok_or_else(|| GitToolError::Git("Missing post-fetch evidence".into()))?
                .fetch_head()?,
            width,
            false,
        )?;
        let before_tips = self.pull_ref_tips.as_ref().ok_or_else(|| {
            GitToolError::Git("Missing pre-fetch reference tips for child discovery".into())
        })?;
        if !fetched.is_empty() {
            let mut args = vec!["rev-list".into()];
            args.extend(fetched);
            args.push("--not".into());
            args.extend(before_tips.iter().cloned());
            args.push("--".into());
            // Bound argv independently of output; large ref sets must not reach
            // host argument limits or silently lose exclusions.
            if args.iter().map(|arg| arg.len() + 1).sum::<usize>() > 32 * 1024 {
                return Err(GitToolError::InvalidInput(
                    "Pull graph revision argument budget exhausted".into(),
                ));
            }
            let graph = self.pull_probe(args).await?;
            if !graph.status.success() || !graph.stderr.is_empty() {
                return Err(GitToolError::Git(
                    "Pull fetched graph traversal failed".into(),
                ));
            }
            self.pull_new_commits = git_write::pull_state::graph_oids(&graph.stdout, width)?;
        }
        for commit in &self.pull_new_commits {
            let output = self
                .pull_probe(vec![
                    "ls-tree".into(),
                    "-r".into(),
                    "-z".into(),
                    "--full-tree".into(),
                    commit.clone(),
                    "--".into(),
                ])
                .await?;
            if !output.status.success() || !output.stderr.is_empty() {
                return Err(GitToolError::Git("Pull child tree listing failed".into()));
            }
            let mut tree = git_write::pull_children::Tree::parse(&output.stdout, width)?;
            if let Some(blob) = &tree.modules_blob {
                let modules = self
                    .pull_probe(vec!["cat-file".into(), "blob".into(), blob.clone()])
                    .await?;
                if !modules.status.success() || !modules.stderr.is_empty() {
                    return Err(GitToolError::Git("Pull module blob read failed".into()));
                }
                tree.bind_modules(&modules.stdout)?;
            }
            // Combined diff selects paths changed against every parent. The
            // retained full tree supplies new-side modes/OIDs without reading
            // child repositories or invoking submodule diff helpers.
            let changes = self
                .pull_probe(vec![
                    "diff-tree".into(),
                    "-r".into(),
                    "--cc".into(),
                    "--name-only".into(),
                    "-z".into(),
                    "--no-commit-id".into(),
                    "--no-renames".into(),
                    "--no-ext-diff".into(),
                    "--no-textconv".into(),
                    "--ignore-submodules=none".into(),
                    commit.clone(),
                    "--".into(),
                ])
                .await?;
            if !changes.status.success() || !changes.stderr.is_empty() {
                return Err(GitToolError::Git(
                    "Pull child change discovery failed".into(),
                ));
            }
            tree.bind_changes(&changes.stdout)?;
            self.pull_child_trees.insert(commit.clone(), tree);
        }
        self.pull_changed_children = git_write::pull_children::collect_named_changes(
            &self.pull_new_commits,
            &self.pull_child_trees,
        )?;
        let before = self
            .pull_before
            .as_ref()
            .ok_or_else(|| GitToolError::Git("Missing Pull pre-fetch recursion policy".into()))?;
        self.pull_current_fetch_children = self
            .pull_current_modules
            .as_ref()
            .ok_or_else(|| GitToolError::Git("Missing current Pull module declarations".into()))?
            .select_current(
                &self.pull_index_children,
                &self.pull_changed_children,
                |name, recursion| before.child_recursion(name, recursion),
            )?;
        for (name, child) in &mut self.pull_changed_children {
            use git_write::pull_children::Activity;
            child.activity = Some(
                match before.child_active(name, &child.path, &self.repository.workdir) {
                    Ok(Some(true)) => Activity::Enabled,
                    Ok(Some(false)) => Activity::Disabled,
                    Ok(None) => Activity::pathspec(&child.path)?,
                    Err(_) => Activity::Invalid,
                },
            );
            // Selection still needs current-index/active-policy evidence. Keep
            // invalid current policy dormant until this candidate is selected.
            child.effective_recursion = Some(
                before
                    .child_recursion(name, child.historical_recursion)
                    .unwrap_or(git_write::pull_children::Recursion::Invalid),
            );
        }
        self.pull_integration = Some(
            self.pull_before
                .as_ref()
                .ok_or_else(|| GitToolError::Git("Missing Pull pre-fetch state".into()))?
                .integration(
                    self.pull_evidence
                        .as_ref()
                        .ok_or_else(|| GitToolError::Git("Missing post-fetch evidence".into()))?
                        .fetch_head()?,
                    width,
                    |args| self.pull_probe(args),
                )
                .await?,
        );
        self.revalidate()?;
        self.identity = PreparedPlanIdentity::new();
        self.phase = Phase::PullIntegration;
        let mut attributes = std::collections::BTreeSet::new();
        for child in self.pull_changed_children.values() {
            if let Some(git_write::pull_children::Activity::Pathspec { attributes: paths }) =
                &child.activity
            {
                attributes.extend(paths.iter().map(|path| self.repository.workdir.join(path)));
            }
        }
        if !attributes.is_empty() {
            attributes.insert(self.repository.common_dir.join("info/attributes"));
            self.pull_activity_globals.clear();
            for variable in ["GIT_ATTR_SYSTEM", "GIT_ATTR_GLOBAL"] {
                let path = self
                    .inspect_attribute_path_at(&self.repository.workdir, variable)
                    .await?;
                attributes.insert(path.clone());
                self.pull_activity_globals.push(path);
            }
            for path in attributes {
                if !self.pull_attribute_paths.contains(&path) {
                    self.facets.push(path_facet(
                        ToolNature::Read,
                        &path,
                        "Pull child activity attribute evidence",
                    )?);
                    self.pull_attribute_paths.push(path);
                }
            }
            self.phase = Phase::PullChildAttributes;
        } else {
            self.select_pull_child_identity()?;
        }
        Ok(self)
    }
}
