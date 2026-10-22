//! Native URL selection after exact legacy-source admission; never connects.

use super::*;

pub(super) struct RemoteEvidence {
    files: Vec<(
        PathBuf,
        snapshot::RegularEvidence,
        Option<snapshot::PathIdentity>,
    )>,
    pub destination: String,
}

impl AddPlan {
    pub(super) fn native_pull_remote_sources(
        &self,
        root: &Path,
        remote: &str,
    ) -> Result<Vec<PathBuf>, GitToolError> {
        // Named remotes may contain nested components, but must not turn the
        // legacy source declaration into a different absolute/parent path.
        if Path::new(remote)
            .components()
            .any(|part| !matches!(part, std::path::Component::Normal(_)))
        {
            return Err(GitToolError::InvalidInput(
                "Native Pull child remote name requires separate source discovery".into(),
            ));
        }
        let metadata = self.native_pull_metadata.get(root).ok_or_else(|| {
            GitToolError::InvalidInput("Native Pull child metadata missing".into())
        })?;
        let common = if metadata.common_path == root {
            root
        } else {
            self.native_pull_modules
                .as_ref()
                .and_then(|modules| modules.resolved_common_target(&metadata.common_path))
                .ok_or_else(|| {
                    GitToolError::InvalidInput("Native Pull common target unresolved".into())
                })?
        };
        Ok(["remotes", "branches"]
            .into_iter()
            .map(|name| common.join(name).join(remote))
            .collect())
    }

    async fn inspect_native_pull_remote(
        &self,
        root: &Path,
        remote: &str,
    ) -> Result<String, GitToolError> {
        let mut command = tokio::process::Command::new(&self.program);
        command
            .current_dir(root)
            .arg("--no-pager")
            .arg("--git-dir")
            .arg(host_path::directory_argument(root)?.as_os_str())
            .args(["ls-remote", "--get-url", "--", remote])
            .env_clear()
            .envs(&self.environment);
        let output = self
            .run_command(command, std::time::Duration::from_secs(10), 16 * 1024)
            .await
            .map_err(|_| {
                GitToolError::InvalidInput(
                    "Native Pull child URL inspection failed; values withheld".into(),
                )
            })?;
        let value = std::str::from_utf8(&output)
            .ok()
            .and_then(|value| value.strip_suffix('\n'))
            .filter(|value| !value.is_empty() && !value.contains(['\0', '\n', '\r']))
            .ok_or_else(|| {
                GitToolError::InvalidInput(
                    "Native Pull child URL is invalid; values withheld".into(),
                )
            })?;
        Ok(value.to_owned())
    }

    pub(super) async fn discover_native_pull_remotes(
        &mut self,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<bool, GitToolError> {
        let mut sources = Vec::new();
        let mut ready = true;
        for (root, config) in &self.native_pull_configs {
            let remote = config.fetch_remote(self.native_pull_metadata[root].branch()?)?;
            config.prepare_pull_transport(&remote)?;
            let paths = self.native_pull_remote_sources(root, &remote)?;
            for path in &paths {
                if !self.facets.iter().any(|facet| {
                    facet.nature == ToolNature::Read
                        && facet.resource_kind == Some(ToolResourceKind::Path)
                        && facet.resource.as_deref() == path.to_str()
                }) {
                    self.facets.push(path_facet(
                        ToolNature::Read,
                        path,
                        "native Pull child legacy remote source; contents deferred",
                    )?);
                    ready = false;
                }
            }
            sources.push((root.clone(), remote, paths));
        }
        if !ready {
            return Ok(false);
        }
        for (_, _, paths) in &sources {
            for path in paths {
                self.child_read(path, authorizations)?;
            }
        }
        for (root, remote, paths) in sources {
            if self.native_pull_remotes.contains_key(&root) {
                self.validate_native_pull_remote(&root, &remote, authorizations)
                    .await?;
                continue;
            }
            let mut files = Vec::new();
            for path in paths {
                if path.canonicalize().is_ok_and(|actual| actual != path) {
                    return Err(GitToolError::InvalidInput(
                        "Native Pull legacy source link requires separate target admission".into(),
                    ));
                }
                let evidence = self.pull_reference_budget.capture(&path)?;
                let identity = if matches!(evidence.entry, snapshot::Entry::Missing) {
                    None
                } else {
                    Some(snapshot::PathIdentity::capture(&path).map_err(|_| {
                        GitToolError::InvalidInput(
                            "Native Pull legacy source identity unavailable".into(),
                        )
                    })?)
                };
                files.push((path, evidence, identity));
            }
            let destination = self.inspect_native_pull_remote(&root, &remote).await?;
            self.pull_reference_budget
                .retain_remote_destination(&destination)?;
            self.native_pull_configs[&root].revalidate()?;
            let evidence = RemoteEvidence { files, destination };
            evidence.revalidate()?;
            self.native_pull_remotes.insert(root.clone(), evidence);
            self.facets.extend(
                self.native_pull_child_remote_facets(&root, &self.native_pull_configs[&root])?,
            );
        }
        // Destination facets join the final mutation request. No destination
        // is contacted here; execution consumes fresh approval for that request.
        Ok(true)
    }

    pub(super) async fn validate_native_pull_remote(
        &self,
        root: &Path,
        remote: &str,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<(), GitToolError> {
        let evidence = self.native_pull_remotes.get(root).ok_or_else(|| {
            GitToolError::InvalidInput("Native Pull child remote evidence missing".into())
        })?;
        for (path, _, _) in &evidence.files {
            self.child_read(path, authorizations)?;
        }
        evidence.revalidate()?;
        if self.inspect_native_pull_remote(root, remote).await? != evidence.destination {
            return Err(GitToolError::InvalidInput(
                "Native Pull child remote changed; values withheld".into(),
            ));
        }
        self.native_pull_configs[root].revalidate()?;
        evidence.revalidate()
    }
}

impl RemoteEvidence {
    fn revalidate(&self) -> Result<(), GitToolError> {
        for (path, evidence, identity) in &self.files {
            if snapshot::PathIdentity::capture(path).ok().as_ref() != identity.as_ref()
                || evidence.revalidate(path).is_err()
            {
                return Err(GitToolError::InvalidInput(
                    "Native Pull child legacy remote source changed".into(),
                ));
            }
        }
        Ok(())
    }
}
