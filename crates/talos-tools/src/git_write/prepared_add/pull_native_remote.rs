//! Native URL selection after bounded or exact source admission; never connects.

use super::*;

pub(super) struct RemoteEvidence {
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
        for (root, config) in &self.native_pull_configs {
            let remote = config.fetch_remote(self.native_pull_metadata[root].branch()?)?;
            config.prepare_pull_transport(&remote)?;
            let paths = self.native_pull_remote_sources(root, &remote)?;
            sources.push((root.clone(), remote, paths));
        }
        if sources.is_empty() {
            return Ok(true);
        }
        let roots = self.native_pull_auxiliary_roots()?;
        let auxiliary = self.auxiliary_paths.as_mut().ok_or_else(|| {
            GitToolError::InvalidInput("Native Pull auxiliary evidence missing".into())
        })?;
        auxiliary.extend(
            "git_pull",
            sources
                .iter()
                .flat_map(|(_, _, paths)| paths.iter().cloned()),
            authorizations,
        )?;
        auxiliary.add_read_frontier(&mut self.facets)?;
        // Resolve metadata first, then require the complete actual-target
        // frontier before reading any content or asking Git for a URL.
        let ready = auxiliary.targets().all(|path| {
            auxiliary.native_pull_target_read(path, authorizations, &roots)
                || authorizations.iter().any(|token| {
                    token.authorizes_path(
                        "git_pull",
                        ToolNature::Read,
                        &self.workspace,
                        path.to_str().unwrap_or_default(),
                    )
                })
        });
        if !ready {
            return Ok(false);
        }
        auxiliary.capture_native_pull(authorizations, &[], &self.pull_reference_budget, &roots)?;
        for (root, remote, _) in sources {
            if self.native_pull_remotes.contains_key(&root) {
                self.validate_native_pull_remote(&root, &remote, authorizations)
                    .await?;
                continue;
            }
            let destination = self.inspect_native_pull_remote(&root, &remote).await?;
            self.pull_reference_budget
                .retain_remote_destination(&destination)?;
            self.native_pull_configs[&root].revalidate()?;
            let evidence = RemoteEvidence { destination };
            self.validate_native_pull_remote_sources(&root, &remote, authorizations)?;
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
        self.validate_native_pull_remote_sources(root, remote, authorizations)?;
        if self.inspect_native_pull_remote(root, remote).await? != evidence.destination {
            return Err(GitToolError::InvalidInput(
                "Native Pull child remote changed; values withheld".into(),
            ));
        }
        self.native_pull_configs[root].revalidate()?;
        self.validate_native_pull_remote_sources(root, remote, authorizations)
    }

    fn validate_native_pull_remote_sources(
        &self,
        root: &Path,
        remote: &str,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<(), GitToolError> {
        let auxiliary = self.auxiliary_paths.as_ref().ok_or_else(|| {
            GitToolError::InvalidInput("Native Pull auxiliary evidence missing".into())
        })?;
        auxiliary.revalidate_native_pull(authorizations, &self.native_pull_auxiliary_roots()?)?;
        if self
            .native_pull_remote_sources(root, remote)?
            .iter()
            .any(|path| auxiliary.leaf_evidence(path).is_none())
        {
            return Err(GitToolError::InvalidInput(
                "Native Pull legacy remote source evidence missing".into(),
            ));
        }
        Ok(())
    }
}
