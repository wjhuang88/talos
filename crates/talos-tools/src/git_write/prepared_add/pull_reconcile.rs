//! Refresh evidence after an independently admitted Pull reconciliation.

use super::super as git_write;
use super::*;

impl AddPlan {
    // Called only after an admitted reconciliation command succeeds. The same
    // stage's worktree Read scope covers refreshed sources; the resulting
    // operation always returns to fresh hook discovery and mutation approval.
    pub(super) async fn refresh_reconciled_pull(
        mut self: Box<Self>,
    ) -> Result<Box<Self>, GitToolError> {
        let current = match self.pull_integration.as_ref() {
            Some(git_write::pull_state::Integration::Reconcile { current, .. }) => current.clone(),
            _ => {
                return Err(GitToolError::Git(
                    "Missing completed Pull reconciliation".into(),
                ));
            }
        };
        self.repository.revalidate_location()?;
        let config = self
            .config
            .as_ref()
            .ok_or_else(|| GitToolError::Git("Missing Pull configuration".into()))?;
        config.revalidate()?;
        // read-tree must not silently rebaseline configuration or external
        // evidence, even though its own index/worktree changes are expected.
        if let Some(inventory) = &self.pull_inventory {
            inventory.revalidate()?;
        }
        let records = config.pull_records()?;
        let before = self
            .pull_before
            .as_ref()
            .ok_or_else(|| GitToolError::Git("Missing Pull pre-fetch state".into()))?;
        let tracking = before
            .branch()
            .map(|branch| {
                git_write::pull_state::tracking_reference(
                    &records,
                    branch,
                    self.pull_remote
                        .as_deref()
                        .ok_or_else(|| GitToolError::Git("Missing Pull remote".into()))?,
                )
            })
            .transpose()?
            .flatten();
        let fetched = self
            .pull_evidence
            .as_ref()
            .ok_or_else(|| GitToolError::Git("Missing Pull fetch evidence".into()))?
            .fetch_head()?
            .to_vec();
        // Retain the previously approved bytes; compare them to fresh evidence
        // without revalidating the deliberately changed index.
        let repository = RepositoryState::capture(&self.repository.workdir)?;
        let evidence = git_write::pull_state::PullEvidence::capture_with_budget(
            &repository,
            tracking.as_deref(),
            true,
            &self.pull_reference_budget,
        )?;
        if evidence.fetch_head()? != fetched {
            return Err(GitToolError::InvalidInput(
                "Pull FETCH_HEAD changed during reconciliation".into(),
            ));
        }
        let mut budget = snapshot::Budget {
            entries: 4096,
            bytes: 4 * 1024 * 1024,
        };
        for attribute in &mut self.pull_attributes {
            if attribute.path.starts_with(&repository.workdir)
                && !attribute.path.starts_with(&repository.git_dir)
                && !attribute.path.starts_with(&repository.common_dir)
            {
                *attribute = PullAttribute::capture(&attribute.path, &mut budget)?;
            } else {
                attribute.revalidate(&mut budget)?;
            }
        }
        self.pull_modules_evidence = Some(PullAttribute::capture(
            &repository.workdir.join(".gitmodules"),
            &mut snapshot::Budget {
                entries: 1,
                bytes: 1024 * 1024,
            },
        )?);
        self.repository = repository;
        self.pull_evidence = Some(evidence);
        let refresh_sources = self.pull_sources.take().is_some();
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
                    "Pull reconciliation object format unavailable".into(),
                ));
            }
        };
        if refresh_sources {
            let (index, index_evidence) =
                git_write::pull_state::decode_index(&self.repository.git_dir.join("index"), width)?;
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
                            "Pull reconciliation sparse discovery failed".into(),
                        ));
                    }
                    encoded.extend(pull_sparse_names(entry.path(&index), &output.stdout)?);
                } else if entry.mode.bits() != 0o160000 {
                    encoded.extend_from_slice(entry.path(&index));
                    encoded.push(0);
                }
            }
            self.pull_evidence
                .as_ref()
                .ok_or_else(|| GitToolError::Git("Missing Pull index evidence".into()))?
                .require_index(&index_evidence)?;
            let paths = pull_tracked_read_paths(&self.repository.workdir, &encoded)?;
            let mut names = Vec::new();
            for path in paths {
                names.push(
                    path.strip_prefix(&self.repository.workdir)
                        .ok()
                        .and_then(Path::to_str)
                        .ok_or_else(|| {
                            GitToolError::InvalidInput("Invalid reconciled source path".into())
                        })?
                        .to_owned(),
                );
                let mut parent = path.parent();
                while let Some(directory) =
                    parent.filter(|path| path.starts_with(&self.repository.workdir))
                {
                    let attribute = directory.join(".gitattributes");
                    if !self.pull_attribute_paths.contains(&attribute) {
                        self.pull_attributes
                            .push(PullAttribute::capture(&attribute, &mut budget)?);
                        self.pull_attribute_paths.push(attribute);
                    }
                    parent = directory.parent();
                }
            }
            self.pull_sources = Some(PullSources::capture(&self.repository.workdir, names)?);
            self.validate_pull_source_filters().await?;
        }
        self.pull_integration = Some(
            self.pull_before
                .as_ref()
                .ok_or_else(|| GitToolError::Git("Missing Pull pre-fetch state".into()))?
                .integration_stage(&fetched, width, Some(&current), |args| {
                    self.pull_probe(args)
                })
                .await?,
        );
        self.revalidate()?;
        self.identity = PreparedPlanIdentity::new();
        self.phase = Phase::PullIntegration;
        Ok(self)
    }
}
