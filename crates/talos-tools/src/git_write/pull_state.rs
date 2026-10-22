//! Fetched target parsing for the pending prepared Pull continuation.
//! Pre-fetch evidence is used by the prepared production discovery stage.
//! Integration selection is read-only; mutation requires a separate effect plan.

use super::GitToolError;

/// Decode only admitted index bytes; never expand sparse trees or open shared indexes.
pub(super) fn decode_index(
    path: &std::path::Path,
    oid_length: usize,
) -> Result<(gix::index::State, super::snapshot::RegularEvidence), GitToolError> {
    let evidence = super::snapshot::RegularEvidence::capture(
        path,
        &mut super::snapshot::Budget {
            entries: 1,
            bytes: 16 * 1024 * 1024,
        },
    )
    .map_err(|error| GitToolError::Git(error.to_string()))?;
    let hash = match oid_length {
        40 => gix::hash::Kind::Sha1,
        64 => gix::hash::Kind::Sha256,
        _ => {
            return Err(GitToolError::InvalidInput(
                "Invalid index object format".into(),
            ));
        }
    };
    let super::snapshot::Entry::File(_, bytes) = &evidence.entry else {
        return Ok((gix::index::State::new(hash), evidence));
    };
    let state = std::panic::catch_unwind(|| -> Result<_, GitToolError> {
        let (state, checksum) = gix::index::State::from_bytes(
            bytes,
            std::time::UNIX_EPOCH.into(),
            hash,
            gix::index::decode::Options {
                thread_limit: Some(1),
                alloc_limit_bytes: Some(16 * 1024 * 1024),
                ..Default::default()
            },
        )
        .map_err(|error| GitToolError::Git(error.to_string()))?;
        if let Some(expected) = checksum {
            let content_length = bytes
                .len()
                .checked_sub(hash.len_in_bytes())
                .ok_or_else(|| GitToolError::InvalidInput("Truncated Pull index".into()))?;
            let mut hasher = gix::hash::hasher(hash);
            hasher.update(&bytes[..content_length]);
            let actual = hasher
                .try_finalize()
                .map_err(|error| GitToolError::Git(error.to_string()))?;
            if actual != expected {
                return Err(GitToolError::InvalidInput(
                    "Pull index checksum mismatch".into(),
                ));
            }
        }
        Ok(state)
    })
    .map_err(|_| GitToolError::Git("Pull index decoder panicked".into()))??;
    if state.link().is_some() {
        return Err(GitToolError::InvalidInput(
            "Pull shared index requires separate admission".into(),
        ));
    }
    Ok((state, evidence))
}

/// Retained only after the repository roots and these metadata paths receive
/// Read admission. This evidence grants no authority and is not confinement.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct PullEvidence {
    repository: super::repository_state::RepositoryState,
    roots: Vec<super::snapshot::PathIdentity>,
    index_modified: Option<std::time::SystemTime>,
    entries: Vec<(
        std::path::PathBuf,
        super::snapshot::PathIdentity,
        super::snapshot::RegularEvidence,
    )>,
}

impl PullEvidence {
    pub(super) fn require_index(
        &self,
        parsed: &super::snapshot::RegularEvidence,
    ) -> Result<(), GitToolError> {
        let path = self.repository.git_dir.join("index");
        if self
            .entries
            .iter()
            .any(|(candidate, _, evidence)| candidate == &path && evidence == parsed)
        {
            Ok(())
        } else {
            Err(GitToolError::InvalidInput(
                "Pull index changed after decoding".into(),
            ))
        }
    }

    pub(super) fn index_timestamp_nonzero(&self) -> bool {
        self.index_modified.is_some_and(|time| {
            let seconds = match time.duration_since(std::time::UNIX_EPOCH) {
                Ok(duration) => duration.as_secs() as u32,
                Err(error) => {
                    let duration = error.duration();
                    let seconds = duration
                        .as_secs()
                        .wrapping_add(u64::from(duration.subsec_nanos() != 0));
                    (seconds as u32).wrapping_neg()
                }
            };
            seconds != 0
        })
    }

    fn index_modified(
        repository: &super::repository_state::RepositoryState,
    ) -> Result<Option<std::time::SystemTime>, GitToolError> {
        match std::fs::symlink_metadata(repository.git_dir.join("index")) {
            Ok(metadata) if metadata.is_file() => metadata
                .modified()
                .map(Some)
                .map_err(|error| GitToolError::Git(error.to_string())),
            Ok(_) => Err(GitToolError::InvalidInput(
                "Pull index is not regular".into(),
            )),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(GitToolError::Git(error.to_string())),
        }
    }

    pub(super) fn required_paths(
        repository: &super::repository_state::RepositoryState,
        tracking: Option<&str>,
        _fetched: bool,
    ) -> Result<std::collections::BTreeSet<std::path::PathBuf>, GitToolError> {
        let invalid = || GitToolError::InvalidInput("Invalid Pull evidence reference".into());
        if let Some(reference) = tracking {
            std::panic::catch_unwind(|| {
                gix::refs::FullName::try_from(gix::bstr::BString::from(reference))
            })
            .map_err(|_| invalid())?
            .map_err(|_| invalid())?;
            if !reference.starts_with("refs/") {
                return Err(invalid());
            }
        }
        let mut paths = std::collections::BTreeSet::new();
        for root in [&repository.git_dir, &repository.common_dir] {
            paths.insert(root.join("logs/HEAD"));
            if let Some(reference) = tracking {
                paths.insert(root.join("logs").join(reference));
            }
        }
        paths.insert(repository.common_dir.join("shallow"));
        paths.insert(repository.common_dir.join("info/grafts"));
        paths.insert(repository.git_dir.join("MERGE_HEAD"));
        // Sparse patterns are per-worktree even when object storage is shared.
        // Bind absence too: enabling patterns after admission changes probes.
        paths.insert(repository.git_dir.join("info/sparse-checkout"));
        // The read-only unmerged-index probe below reads this exact index.
        // Root Read does not authorize descendants.
        paths.insert(repository.git_dir.join("index"));
        // Fetch opens this destination directly. Bind its absence or existing
        // regular contents before fetch as well as after it; otherwise a stable
        // symlink could redirect the admitted metadata write outside its root.
        paths.insert(repository.git_dir.join("FETCH_HEAD"));
        Ok(paths)
    }

    /// Capture only after fresh metadata Read admission. For post-fetch capture,
    /// retain the original location guard while accepting newly fetched content;
    /// callers must separately retain and validate configuration/worktree evidence.
    #[cfg(test)]
    pub(super) fn capture(
        repository: &super::repository_state::RepositoryState,
        tracking: Option<&str>,
        fetched: bool,
    ) -> Result<Self, GitToolError> {
        Self::capture_with_budget(repository, tracking, fetched, &Default::default())
    }

    pub(super) fn capture_with_budget(
        repository: &super::repository_state::RepositoryState,
        tracking: Option<&str>,
        fetched: bool,
        budget: &super::pull_inventory::ReferenceBudget,
    ) -> Result<Self, GitToolError> {
        let invalid = || {
            GitToolError::InvalidInput(
                "Pull reference/reflog evidence unavailable or changed".into(),
            )
        };
        repository.revalidate_location()?;
        // Fetch changes references and object storage, so its old content
        // snapshot cannot serve as the post-fetch baseline. Validate retained
        // roots/pointers BEFORE rediscovery, and again before accepting it.
        let refreshed;
        let repository = if fetched {
            refreshed = super::repository_state::RepositoryState::capture(&repository.workdir)?;
            repository.revalidate_location()?;
            if refreshed.workdir != repository.workdir
                || refreshed.git_dir != repository.git_dir
                || refreshed.common_dir != repository.common_dir
                || refreshed.index != repository.index
            {
                return Err(invalid());
            }
            &refreshed
        } else {
            repository
        };
        let paths = Self::required_paths(repository, tracking, fetched)?;
        let index_modified = Self::index_modified(repository)?;
        let mut roots = Vec::new();
        for root in [&repository.git_dir, &repository.common_dir] {
            roots.push(super::snapshot::PathIdentity::capture(root).map_err(|_| invalid())?);
        }
        roots.push(
            super::snapshot::PathIdentity::capture(&repository.workdir).map_err(|_| invalid())?,
        );
        let mut entries = Vec::new();
        for path in paths {
            let root = if path.starts_with(&repository.git_dir) {
                &repository.git_dir
            } else {
                &repository.common_dir
            };
            // Bind the closest existing parent even for missing reflogs. Never
            // read through a parent that resolves outside the admitted root.
            let parent = path
                .parent()
                .ok_or_else(invalid)?
                .ancestors()
                .find(|parent| parent.exists())
                .ok_or_else(invalid)?;
            let resolved = parent.canonicalize().map_err(|_| invalid())?;
            if !resolved.starts_with(root) || resolved != parent {
                return Err(invalid());
            }
            let identity = super::snapshot::PathIdentity::capture(parent).map_err(|_| invalid())?;
            let evidence = budget.capture(&path).map_err(|_| invalid())?;
            if super::snapshot::PathIdentity::capture(parent).map_err(|_| invalid())? != identity {
                return Err(invalid());
            }
            entries.push((path, identity, evidence));
        }
        repository.revalidate_location()?;
        let current = super::repository_state::RepositoryState::capture(&repository.workdir)?;
        if &current != repository {
            return Err(invalid());
        }
        for (root, identity) in [
            &repository.git_dir,
            &repository.common_dir,
            &repository.workdir,
        ]
        .into_iter()
        .zip(&roots)
        {
            if &super::snapshot::PathIdentity::capture(root).map_err(|_| invalid())? != identity {
                return Err(invalid());
            }
        }
        if Self::index_modified(repository)? != index_modified {
            return Err(invalid());
        }
        Ok(Self {
            repository: current,
            roots,
            index_modified,
            entries,
        })
    }

    pub(super) fn revalidate(&self) -> Result<(), GitToolError> {
        let invalid = || {
            GitToolError::InvalidInput(
                "Pull reference/reflog evidence changed after inspection".into(),
            )
        };
        let check_roots = || -> Result<(), GitToolError> {
            for (root, identity) in [
                &self.repository.git_dir,
                &self.repository.common_dir,
                &self.repository.workdir,
            ]
            .into_iter()
            .zip(&self.roots)
            {
                identity.revalidate(root).map_err(|_| invalid())?;
            }
            Ok(())
        };
        // Refuse changed roots/pointers before any metadata content is read.
        check_roots()?;
        self.repository.revalidate_location()?;
        if Self::index_modified(&self.repository)? != self.index_modified {
            return Err(invalid());
        }
        for (path, identity, evidence) in &self.entries {
            let root = if path.starts_with(&self.repository.git_dir) {
                &self.repository.git_dir
            } else {
                &self.repository.common_dir
            };
            // A newly created intermediate directory changes the retained
            // closest parent even when the leaf remains absent.
            let parent = path
                .parent()
                .ok_or_else(invalid)?
                .ancestors()
                .find(|parent| parent.exists())
                .ok_or_else(invalid)?;
            let resolved = parent.canonicalize().map_err(|_| invalid())?;
            if resolved != parent || !resolved.starts_with(root) {
                return Err(invalid());
            }
            identity.revalidate(parent).map_err(|_| invalid())?;
            evidence.revalidate(path).map_err(|_| invalid())?;
            identity.revalidate(parent).map_err(|_| invalid())?;
        }
        self.repository.revalidate_location()?;
        if super::repository_state::RepositoryState::capture(&self.repository.workdir)?
            != self.repository
        {
            return Err(invalid());
        }
        check_roots()?;
        if Self::index_modified(&self.repository)? != self.index_modified {
            return Err(invalid());
        }
        Ok(())
    }

    pub(super) fn require_no_pending_merge(&self) -> Result<(), GitToolError> {
        let merge_head = self.repository.git_dir.join("MERGE_HEAD");
        let entry = self
            .entries
            .iter()
            .find(|(path, _, _)| path == &merge_head)
            .ok_or_else(|| GitToolError::Git("Missing Pull merge-state evidence".into()))?;
        if !matches!(entry.2.entry, super::snapshot::Entry::Missing) {
            return Err(GitToolError::InvalidInput(
                "Pull requires concluding the pending merge before fetch".into(),
            ));
        }
        Ok(())
    }

    pub(super) fn fetch_head(&self) -> Result<&[u8], GitToolError> {
        let path = self.repository.git_dir.join("FETCH_HEAD");
        self.entries
            .iter()
            .find_map(|(entry_path, _, entry)| {
                if entry_path == &path
                    && let super::snapshot::Entry::File(_, bytes) = &entry.entry
                {
                    Some(bytes.as_slice())
                } else {
                    None
                }
            })
            .ok_or_else(|| GitToolError::InvalidInput("Pull FETCH_HEAD evidence missing".into()))
    }
}

pub(super) fn fetch_remote(
    config: &[u8],
    branch: Option<&str>,
    explicit: Option<&str>,
) -> Result<String, GitToolError> {
    let values = super::config_state::parse_host(config)?;
    let invalid = || GitToolError::InvalidInput("Invalid Pull remote configuration".into());
    let configured = branch
        .map(|branch| format!("branch.{branch}.remote"))
        .and_then(|key| values.get(key.as_bytes()))
        .and_then(|values| values.last());
    let remote = if let Some(explicit) = explicit {
        explicit.to_owned()
    } else if let Some(value) = configured {
        std::str::from_utf8(value.as_deref().ok_or_else(invalid)?)
            .map_err(|_| invalid())?
            .to_owned()
    } else {
        let mut names = std::collections::BTreeSet::new();
        for key in values.keys().filter(|key| key.starts_with(b"remote.")) {
            let key = std::str::from_utf8(key).map_err(|_| invalid())?;
            let Some((name, _)) = key[7..].rsplit_once('.') else {
                continue;
            };
            if !name.is_empty() && !name.starts_with('/') {
                names.insert(name);
            }
        }
        if names.len() == 1 {
            names.first().ok_or_else(invalid)?.to_string()
        } else {
            "origin".to_owned()
        }
    };
    super::validate_git_positional(&remote)?;
    Ok(remote)
}

pub(super) fn tracking_reference(
    config: &[u8],
    branch: &str,
    remote: &str,
) -> Result<Option<String>, GitToolError> {
    let invalid = || GitToolError::InvalidInput("Invalid Pull upstream mapping".into());
    let values = super::config_state::parse_host(config)?;
    let remote_key = format!("branch.{branch}.remote");
    let branch_remote = values
        .get(remote_key.as_bytes())
        .and_then(|values| values.last())
        .and_then(|value| value.as_deref());
    if branch_remote != Some(remote.as_bytes()) {
        return Ok(None);
    }
    let merge_key = format!("branch.{branch}.merge");
    let Some(source) = values
        .get(merge_key.as_bytes())
        .and_then(|values| values.first())
    else {
        return Ok(None);
    };
    let source =
        std::str::from_utf8(source.as_deref().ok_or_else(invalid)?).map_err(|_| invalid())?;
    let fetch_key = format!("remote.{remote}.fetch");
    for value in values.get(fetch_key.as_bytes()).into_iter().flatten() {
        let bytes = value.as_deref().ok_or_else(invalid)?;
        let spec = std::panic::catch_unwind(|| {
            gix::refspec::parse(
                gix::bstr::BStr::new(bytes),
                gix::refspec::parse::Operation::Fetch,
            )
        })
        .map_err(|_| invalid())?
        .map_err(|_| invalid())?;
        if bytes.starts_with(b"^") {
            continue;
        }
        let (Some(from), Some(to)) = (spec.source(), spec.destination()) else {
            continue;
        };
        let from = std::str::from_utf8(from.as_ref()).map_err(|_| invalid())?;
        let to = std::str::from_utf8(to.as_ref()).map_err(|_| invalid())?;
        let mapped = if let Some((prefix, suffix)) = from.split_once('*') {
            source
                .strip_prefix(prefix)
                .and_then(|middle| middle.strip_suffix(suffix))
                .map(|middle| to.replace('*', middle))
        } else if from == source {
            Some(to.to_owned())
        } else {
            None
        };
        if let Some(mapped) = mapped {
            return Ok(Some(mapped));
        }
    }
    if remote == "." && source.starts_with("refs/") {
        return Ok(Some(source.to_owned()));
    }
    if remote == "." {
        return Err(GitToolError::InvalidInput(
            "Local Pull upstream needs admitted DWIM reference discovery".into(),
        ));
    }
    Ok(None)
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct BeforeFetch {
    branch: Option<String>,
    head: Option<String>,
    fork_point: Option<String>,
    policy: Policy,
    fetch_recursion: FetchRecursion,
    child_recursion: std::collections::BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    child_activity: std::collections::BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    child_urls: std::collections::BTreeSet<Vec<u8>>,
    active_pathspecs: Option<Vec<Option<Vec<u8>>>>,
}

/// Retain the configured mode even though the parent command disables native
/// recursion. Default must remain distinct: it still permits `.gitmodules`
/// defaults and per-submodule overrides during separately admitted discovery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FetchRecursion {
    Default,
    Off,
    On,
    OnDemand,
}

impl FetchRecursion {
    fn capture(config: &[u8]) -> Result<Self, GitToolError> {
        // Validate record framing, but do not iterate the keyed map: Git
        // applies both global keys to one variable in declaration order.
        super::config_state::parse_host(config)?;
        let mut mode = Self::Default;
        for record in config
            .split(|byte| *byte == 0)
            .filter(|record| !record.is_empty())
        {
            let (key, value) = match record.iter().position(|byte| *byte == b'\n') {
                Some(split) => (&record[..split], Some(&record[split + 1..])),
                None => (record, None),
            };
            if key.eq_ignore_ascii_case(b"submodule.recurse") {
                mode = if super::config_state::git_boolean(value)? {
                    Self::On
                } else {
                    Self::Off
                };
            } else if key.eq_ignore_ascii_case(b"fetch.recursesubmodules") {
                mode = if value == Some(b"on-demand".as_slice()) {
                    Self::OnDemand
                } else if super::config_state::git_boolean(value)? {
                    Self::On
                } else {
                    Self::Off
                };
            }
        }
        Ok(mode)
    }

    fn description(self) -> &'static str {
        match self {
            Self::Default => "default (child policy remains to be discovered)",
            Self::Off => "off",
            Self::On => "on",
            Self::OnDemand => "on-demand",
        }
    }
}

impl BeforeFetch {
    /// Resolve native historical-candidate precedence without filesystem reads.
    /// `None` requires admitted pathspec/attribute evaluation; it is not inactive.
    pub(super) fn child_active(
        &self,
        name: &str,
        path: &str,
        workdir: &std::path::Path,
    ) -> Result<Option<bool>, GitToolError> {
        self.child_active_with_attributes(name, path, workdir, None)
    }

    /// Evaluate attributes only from caller-supplied, admitted in-memory rules.
    pub(super) fn child_active_with_attributes(
        &self,
        name: &str,
        path: &str,
        workdir: &std::path::Path,
        attributes: Option<(&gix::attrs::Search, &gix::attrs::search::MetadataCollection)>,
    ) -> Result<Option<bool>, GitToolError> {
        if let Some(value) = self.child_activity.get(name.as_bytes()) {
            return super::config_state::git_boolean(value.as_deref()).map(Some);
        }
        if let Some(values) = &self.active_pathspecs {
            let patterns = values
                .iter()
                .map(|value| {
                    let value = value.as_deref().ok_or_else(|| {
                        GitToolError::InvalidInput(
                            "Valueless submodule.active pathspec is invalid".into(),
                        )
                    })?;
                    std::panic::catch_unwind(|| gix::pathspec::parse(value, Default::default()))
                        .map_err(|_| GitToolError::InvalidInput("Pathspec parser panicked".into()))?
                        .map_err(|_| {
                            GitToolError::InvalidInput("Invalid submodule.active pathspec".into())
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;
            if attributes.is_none()
                && patterns
                    .iter()
                    .any(|pattern| !pattern.attributes.is_empty())
            {
                return Ok(None);
            }
            let mut search = std::panic::catch_unwind(|| {
                gix::pathspec::Search::from_specs(patterns, None, workdir)
            })
            .map_err(|_| GitToolError::InvalidInput("Pathspec matcher panicked".into()))?
            .map_err(|_| GitToolError::InvalidInput("Invalid submodule.active pathspec".into()))?;
            let matched = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                search
                    .pattern_matching_relative_path(
                        gix::bstr::BStr::new(path.as_bytes()),
                        Some(true),
                        &mut |path, case, is_dir, out| {
                            let Some((rules, collection)) = attributes else {
                                return false;
                            };
                            out.initialize(collection);
                            rules.pattern_matching_relative_path(path, case, Some(is_dir), out);
                            // Unmatched attributes are Unspecified, which is a
                            // valid match for the pathspec's !attr requirement.
                            true
                        },
                    )
                    .is_some_and(|item| !item.is_excluded())
            }))
            .map_err(|_| GitToolError::InvalidInput("Pathspec matcher panicked".into()))?;
            return Ok(Some(matched));
        }
        Ok(Some(self.child_urls.contains(name.as_bytes())))
    }

    /// Resolve only the named candidate. Explicit global policy bypasses even
    /// invalid per-module values, as native fetch does. No child paths are read.
    pub(super) fn child_recursion(
        &self,
        name: &str,
        historical: Option<super::pull_children::Recursion>,
    ) -> Result<super::pull_children::Recursion, GitToolError> {
        use super::pull_children::Recursion;
        match self.fetch_recursion {
            FetchRecursion::Off => return Ok(Recursion::Off),
            FetchRecursion::On => return Ok(Recursion::On),
            FetchRecursion::OnDemand => return Ok(Recursion::OnDemand),
            FetchRecursion::Default => {}
        }
        if let Some(value) = self.child_recursion.get(name.as_bytes()) {
            return if value.as_deref() == Some(b"on-demand".as_slice()) {
                Ok(Recursion::OnDemand)
            } else {
                super::config_state::git_boolean(value.as_deref()).map(|enabled| {
                    if enabled {
                        Recursion::On
                    } else {
                        Recursion::Off
                    }
                })
            };
        }
        Ok(match historical {
            Some(Recursion::Off) => Recursion::Off,
            Some(Recursion::On) => Recursion::On,
            _ => Recursion::OnDemand,
        })
    }

    pub(super) fn fetch_recursion_description(&self) -> &'static str {
        self.fetch_recursion.description()
    }

    pub(super) fn policy_requires_clean_worktree(
        config: &[u8],
        branch: Option<&str>,
    ) -> Result<bool, GitToolError> {
        let policy = Policy::capture(config, branch)?;
        Ok(policy.rebase != Rebase::Off && !policy.autostash)
    }

    pub(super) fn requires_clean_worktree(&self) -> bool {
        self.policy.rebase != Rebase::Off && !self.policy.autostash
    }

    pub(super) fn same_checkout(&self, other: &Self) -> bool {
        self.branch == other.branch && self.head == other.head
    }

    pub(super) fn requires_empty_initial_index(&self) -> bool {
        self.head.is_none() && self.policy.rebase != Rebase::Off
    }

    pub(super) fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }

    // Run only after fetch and admitted object inspection. The caller must bind
    // FETCH_HEAD and these graph observations to the next mutation admission.
    pub(super) async fn integration<F, Fut>(
        &self,
        fetched: &[u8],
        oid_length: usize,
        run: F,
    ) -> Result<Integration, GitToolError>
    where
        F: FnMut(Vec<String>) -> Fut,
        Fut: std::future::Future<Output = Result<std::process::Output, GitToolError>>,
    {
        self.integration_stage(fetched, oid_length, None, run).await
    }

    pub(super) async fn integration_stage<F, Fut>(
        &self,
        fetched: &[u8],
        oid_length: usize,
        reconciled_head: Option<&str>,
        mut run: F,
    ) -> Result<Integration, GitToolError>
    where
        F: FnMut(Vec<String>) -> Fut,
        Fut: std::future::Future<Output = Result<std::process::Output, GitToolError>>,
    {
        let completed = |output: std::process::Output| {
            if output.status.success() && output.stderr.is_empty() {
                return Ok(Some(output.stdout));
            }
            if output.status.code() == Some(1)
                && output.stdout.is_empty()
                && output.stderr.is_empty()
            {
                return Ok(None);
            }
            Err(GitToolError::Git(
                "Pull post-fetch HEAD probe failed".into(),
            ))
        };
        let branch =
            completed(run(vec!["symbolic-ref".into(), "--quiet".into(), "HEAD".into()]).await?)?;
        let head = completed(
            run(vec![
                "rev-parse".into(),
                "--verify".into(),
                "--quiet".into(),
                "--end-of-options".into(),
                "HEAD^{commit}".into(),
            ])
            .await?,
        )?;
        let current = Self::capture(b"", branch.as_deref(), head.as_deref(), None, oid_length)?;
        if current.branch != self.branch {
            return Err(GitToolError::InvalidInput(
                "Pull branch changed during fetch".into(),
            ));
        }
        if reconciled_head.is_some_and(|expected| current.head.as_deref() != Some(expected)) {
            return Err(GitToolError::InvalidInput(
                "Pull HEAD changed after worktree reconciliation; request fresh discovery and authorization".into(),
            ));
        }
        if current.head != self.head {
            match (&self.head, &current.head) {
                (Some(original), Some(current)) if reconciled_head.is_none() => {
                    return Ok(Integration::Reconcile {
                        original: original.clone(),
                        current: current.clone(),
                    });
                }
                (Some(_), Some(_)) => {}
                _ => {
                    return Err(GitToolError::InvalidInput(
                        "Pull HEAD birth/deletion during fetch requires separate reconciliation"
                            .into(),
                    ));
                }
            }
        }
        let heads = merge_candidates(fetched, oid_length)?;
        if heads.is_empty() {
            return Err(GitToolError::InvalidInput(
                "Pull fetched no merge candidates".into(),
            ));
        }
        let Some(original) = &self.head else {
            if heads.len() != 1 {
                return Err(GitToolError::InvalidInput(
                    "Cannot merge multiple branches into empty HEAD".into(),
                ));
            }
            return Ok(Integration::Unborn(heads[0].clone()));
        };
        if heads.len() > 1
            && (self.policy.rebase != Rebase::Off || self.policy.fast_forward == FastForward::Only)
        {
            return Err(GitToolError::InvalidInput(
                "Pull rebase or ff-only requires a single merge candidate".into(),
            ));
        }
        let ancestor = |output: std::process::Output| {
            if output.stdout.is_empty() && output.stderr.is_empty() {
                match output.status.code() {
                    Some(0) => return Ok(true),
                    Some(1) => return Ok(false),
                    _ => {}
                }
            }
            Err(GitToolError::Git("Pull ancestry probe failed".into()))
        };
        let can_ff = heads.len() == 1
            && ancestor(
                run(vec![
                    "merge-base".into(),
                    "--is-ancestor".into(),
                    original.clone(),
                    heads[0].clone(),
                ])
                .await?,
            )?;
        let mut up_to_date = true;
        if !can_ff {
            for head in &heads {
                if !ancestor(
                    run(vec![
                        "merge-base".into(),
                        "--is-ancestor".into(),
                        head.clone(),
                        original.clone(),
                    ])
                    .await?,
                )? {
                    up_to_date = false;
                    break;
                }
            }
        }
        let divergent = !can_ff && !up_to_date;
        if divergent && self.policy.fast_forward == FastForward::Only {
            return Err(GitToolError::InvalidInput(
                "Pull cannot fast-forward divergent branches".into(),
            ));
        }
        if divergent
            && self.policy.fast_forward == FastForward::Unspecified
            && self.policy.rebase_unspecified
        {
            return Err(GitToolError::InvalidInput(
                "Pull needs a configured reconciliation policy for divergent branches".into(),
            ));
        }
        let rebase =
            self.policy.rebase != Rebase::Off && self.policy.fast_forward != FastForward::Only;
        let integration = if rebase && !can_ff {
            let mut upstream = heads[0].clone();
            if let Some(fork) = &self.fork_point {
                let output = run(vec![
                    "merge-base".into(),
                    "--octopus".into(),
                    current.head.as_ref().unwrap_or(original).clone(),
                    heads[0].clone(),
                    fork.clone(),
                ])
                .await?;
                let base = if output.status.success() {
                    Self::capture(b"", None, Some(&output.stdout), None, oid_length)?.head
                } else if output.status.code() == Some(1)
                    && output.stdout.is_empty()
                    && output.stderr.is_empty()
                {
                    None
                } else {
                    return Err(GitToolError::Git(
                        "Pull octopus merge-base probe failed".into(),
                    ));
                };
                if base.as_ref() != Some(fork) {
                    upstream = fork.clone();
                }
            }
            Integration::Rebase {
                onto: heads[0].clone(),
                upstream,
                mode: self.policy.rebase.clone(),
                autostash: self.policy.autostash,
            }
        } else {
            Integration::Merge {
                heads,
                fast_forward: if rebase && can_ff {
                    FastForward::Only
                } else {
                    self.policy.fast_forward.clone()
                },
                autostash: self.policy.autostash,
            }
        };
        Ok(integration)
    }

    pub(super) async fn probe<F, Fut>(
        config: &[u8],
        tracking_ref: Option<&str>,
        oid_length: usize,
        mut run: F,
    ) -> Result<Self, GitToolError>
    where
        F: FnMut(Vec<String>) -> Fut,
        Fut: std::future::Future<Output = Result<std::process::Output, GitToolError>>,
    {
        let completed = |output: std::process::Output| {
            if output.status.success() {
                return Ok(Some(output.stdout));
            }
            if output.status.code() == Some(1)
                && output.stdout.is_empty()
                && output.stderr.is_empty()
            {
                return Ok(None);
            }
            Err(GitToolError::Git("Pull pre-fetch probe failed".into()))
        };
        let branch =
            completed(run(vec!["symbolic-ref".into(), "--quiet".into(), "HEAD".into()]).await?)?;
        let head = completed(
            run(vec![
                "rev-parse".into(),
                "--verify".into(),
                "--quiet".into(),
                "--end-of-options".into(),
                "HEAD^{commit}".into(),
            ])
            .await?,
        )?;
        // Validate branch/HEAD and policy before any fork-point object traversal.
        let mut state =
            Self::capture(config, branch.as_deref(), head.as_deref(), None, oid_length)?;
        if state.policy.rebase != Rebase::Off
            && state.branch.is_some()
            && state.head.is_some()
            && let Some(reference) = tracking_ref
        {
            super::validate_git_positional(reference)?;
            let tracking = completed(
                run(vec![
                    "rev-parse".into(),
                    "--verify".into(),
                    "--quiet".into(),
                    "--end-of-options".into(),
                    format!("{reference}^{{commit}}"),
                ])
                .await?,
            )?;
            // A configured upstream can be absent before the first fetch.
            // Validate successful evidence before traversing its reflog.
            Self::capture(
                config,
                branch.as_deref(),
                tracking.as_deref(),
                None,
                oid_length,
            )?;
            let fork = if tracking.is_some() {
                completed(
                    run(vec![
                        "merge-base".into(),
                        "--fork-point".into(),
                        reference.into(),
                        "HEAD".into(),
                    ])
                    .await?,
                )?
            } else {
                None
            };
            state = Self::capture(
                config,
                branch.as_deref(),
                head.as_deref(),
                fork.as_deref(),
                oid_length,
            )?;
        }
        let branch_after =
            completed(run(vec!["symbolic-ref".into(), "--quiet".into(), "HEAD".into()]).await?)?;
        let head_after = completed(
            run(vec![
                "rev-parse".into(),
                "--verify".into(),
                "--quiet".into(),
                "--end-of-options".into(),
                "HEAD^{commit}".into(),
            ])
            .await?,
        )?;
        if branch_after != branch || head_after != head {
            return Err(GitToolError::InvalidInput(
                "Pull HEAD changed during pre-fetch probes".into(),
            ));
        }
        Ok(state)
    }

    // Inputs come from separately admitted, completed probes. Transport errors
    // never become probe output. Retain these values across fetch, not re-probe
    // the updated remote-tracking ref to reconstruct the original fork point.
    fn capture(
        config: &[u8],
        branch: Option<&[u8]>,
        head: Option<&[u8]>,
        fork_point: Option<&[u8]>,
        oid_length: usize,
    ) -> Result<Self, GitToolError> {
        let invalid = || GitToolError::InvalidInput("Invalid Pull pre-fetch evidence".into());
        if !matches!(oid_length, 40 | 64) {
            return Err(invalid());
        }
        let oid = |bytes: &[u8]| {
            let bytes = bytes.strip_suffix(b"\n").ok_or_else(invalid)?;
            if bytes.len() != oid_length
                || !bytes.iter().all(u8::is_ascii_hexdigit)
                || bytes.iter().all(|byte| *byte == b'0')
            {
                return Err(invalid());
            }
            Ok(String::from_utf8(bytes.to_vec())
                .map_err(|_| invalid())?
                .to_ascii_lowercase())
        };
        let branch = branch
            .map(|bytes| {
                let bytes = bytes.strip_suffix(b"\n").ok_or_else(invalid)?;
                let name = std::str::from_utf8(bytes)
                    .map_err(|_| invalid())?
                    .strip_prefix("refs/heads/")
                    .ok_or_else(invalid)?;
                if name.is_empty() || bytes.contains(&0) || bytes.contains(&b'\n') {
                    return Err(invalid());
                }
                Ok(name.to_owned())
            })
            .transpose()?;
        let head = head.map(oid).transpose()?;
        let fork_point = fork_point.map(oid).transpose()?;
        if fork_point.is_some() && (branch.is_none() || head.is_none()) {
            return Err(invalid());
        }
        let policy = Policy::capture(config, branch.as_deref())?;
        let fetch_recursion = FetchRecursion::capture(config)?;
        let parsed = super::config_state::parse_host(config)?;
        let mut child_activity = std::collections::BTreeMap::new();
        let mut child_urls = std::collections::BTreeSet::new();
        for (key, values) in &parsed {
            if let Some(key) = key.strip_prefix(b"submodule.") {
                if let Some(name) = key.strip_suffix(b".active")
                    && let Some(value) = values.last()
                {
                    child_activity.insert(name.to_vec(), value.clone());
                }
                if let Some(name) = key.strip_suffix(b".url")
                    && values.last().is_some_and(Option::is_some)
                {
                    child_urls.insert(name.to_vec());
                }
            }
        }
        let active_pathspecs = parsed.get(b"submodule.active".as_slice()).cloned();
        let child_recursion = parsed
            .into_iter()
            .filter_map(|(key, mut values)| {
                let name = key
                    .strip_prefix(b"submodule.")?
                    .strip_suffix(b".fetchrecursesubmodules")?;
                Some((name.to_vec(), values.pop()?))
            })
            .collect();
        Ok(Self {
            branch,
            head,
            fork_point,
            policy,
            fetch_recursion,
            child_recursion,
            child_activity,
            child_urls,
            active_pathspecs,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Integration {
    Unborn(String),
    Merge {
        heads: Vec<String>,
        fast_forward: FastForward,
        autostash: bool,
    },
    Rebase {
        onto: String,
        upstream: String,
        mode: Rebase,
        autostash: bool,
    },
    Reconcile {
        original: String,
        current: String,
    },
}

impl Integration {
    /// Fixed commands for this retained operation, without starting a process.
    /// Callers must stop at the first failure; reconciliation needs fresh
    /// evidence and a separately admitted next integration afterwards.
    pub(super) fn commands(&self) -> Result<Vec<Vec<String>>, GitToolError> {
        std::panic::catch_unwind(|| self.commands_inner()).unwrap_or_else(|_| {
            Err(GitToolError::Git(
                "Pull integration command preparation failed".into(),
            ))
        })
    }

    fn commands_inner(&self) -> Result<Vec<Vec<String>>, GitToolError> {
        let ids: Vec<&str> = match self {
            Self::Unborn(target) => vec![target],
            Self::Merge { heads, .. } => heads.iter().map(String::as_str).collect(),
            Self::Rebase { onto, upstream, .. } => vec![onto, upstream],
            Self::Reconcile { original, current } => vec![original, current],
        };
        let length = ids.first().map_or(0, |id| id.len());
        if !matches!(length, 40 | 64)
            || ids
                .iter()
                .any(|id| id.len() != length || !id.bytes().all(|byte| byte.is_ascii_hexdigit()))
        {
            return Err(GitToolError::InvalidInput(
                "Invalid Pull integration object identity".into(),
            ));
        }
        if let Some(command) = self.command() {
            return Ok(vec![command]);
        }
        match self {
            Self::Unborn(target) => {
                let hash = if length == 40 {
                    gix::hash::Kind::Sha1
                } else {
                    gix::hash::Kind::Sha256
                };
                let mut hasher = gix::hash::hasher(hash);
                hasher.update(b"tree 0\0");
                let empty = hasher
                    .try_finalize()
                    .map_err(|error| GitToolError::Git(error.to_string()))?;
                Ok(vec![
                    vec![
                        "read-tree".into(),
                        "-u".into(),
                        "-m".into(),
                        empty.to_string(),
                        target.clone(),
                    ],
                    vec![
                        "update-ref".into(),
                        "-m".into(),
                        "initial pull".into(),
                        "HEAD".into(),
                        target.clone(),
                        "0".repeat(length),
                    ],
                ])
            }
            Self::Reconcile { original, current } => Ok(vec![vec![
                "read-tree".into(),
                "-u".into(),
                "-m".into(),
                original.clone(),
                current.clone(),
            ]]),
            _ => Err(GitToolError::InvalidInput(
                "Missing Pull integration command".into(),
            )),
        }
    }

    fn command(&self) -> Option<Vec<String>> {
        let (operation, autostash) = match self {
            Self::Merge { autostash, .. } => ("merge", autostash),
            Self::Rebase { autostash, .. } => ("rebase", autostash),
            _ => return None,
        };
        let mut args = vec![
            operation.into(),
            if *autostash {
                "--autostash"
            } else {
                "--no-autostash"
            }
            .into(),
        ];
        match self {
            Self::Merge { fast_forward, .. } => {
                match fast_forward {
                    FastForward::Allow => args.push("--ff".into()),
                    FastForward::Never => args.push("--no-ff".into()),
                    FastForward::Only => args.push("--ff-only".into()),
                    FastForward::Unspecified => {}
                }
                args.push("FETCH_HEAD".into());
            }
            Self::Rebase {
                onto,
                upstream,
                mode,
                ..
            } => {
                match mode {
                    Rebase::Merges => args.push("--rebase-merges".into()),
                    Rebase::Interactive => args.push("--interactive".into()),
                    _ => {}
                }
                args.extend(["--onto".into(), onto.clone(), upstream.clone()]);
            }
            _ => return None,
        }
        Some(args)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Rebase {
    Off,
    On,
    Merges,
    Interactive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum FastForward {
    Unspecified,
    Allow,
    Never,
    Only,
}

#[derive(Debug, PartialEq, Eq)]
struct Policy {
    rebase: Rebase,
    rebase_unspecified: bool,
    fast_forward: FastForward,
    autostash: bool,
}

/// Select integration policy before fetching, without predicting fetched commits.
pub(super) fn native_rebase_mode(
    config: &[u8],
    branch: Option<&str>,
) -> Result<Rebase, GitToolError> {
    Ok(Policy::capture(config, branch)?.rebase)
}

impl Policy {
    // Input is the admitted host configuration list, without origin fields.
    // Keep subsection case and duplicate declaration order via the common parser.
    fn capture(config: &[u8], branch: Option<&str>) -> Result<Self, GitToolError> {
        let values = super::config_state::parse_host(config)?;
        let get = |key: &[u8]| values.get(key).and_then(|values| values.last());
        let branch_key = branch.map(|branch| format!("branch.{branch}.rebase"));
        let rebase_value = branch_key
            .as_ref()
            .and_then(|key| get(key.as_bytes()))
            .or_else(|| get(b"pull.rebase"));
        let rebase = match rebase_value {
            None => Rebase::Off,
            Some(value) => match value.as_deref() {
                Some(b"merges" | b"m") => Rebase::Merges,
                Some(b"interactive" | b"i") => Rebase::Interactive,
                value => {
                    if super::config_state::git_boolean(value)? {
                        Rebase::On
                    } else {
                        Rebase::Off
                    }
                }
            },
        };
        let fast_forward = match get(b"pull.ff") {
            None => FastForward::Unspecified,
            Some(value) => match value.as_deref() {
                Some(b"only") => FastForward::Only,
                value => {
                    if super::config_state::git_boolean(value)? {
                        FastForward::Allow
                    } else {
                        FastForward::Never
                    }
                }
            },
        };
        let fallback = if rebase == Rebase::Off {
            b"merge.autostash".as_slice()
        } else {
            b"rebase.autostash".as_slice()
        };
        let autostash = get(b"pull.autostash")
            .or_else(|| get(fallback))
            .map(|value| super::config_state::git_boolean(value.as_deref()))
            .transpose()?
            .unwrap_or(false);
        Ok(Self {
            rebase,
            rebase_unspecified: rebase_value.is_none(),
            fast_forward,
            autostash,
        })
    }
}

/// Keep merge candidates in FETCH_HEAD order, without interpreting or exposing
/// the remote-description field (which may contain credentials or non-UTF-8).
fn merge_candidates(bytes: &[u8], oid_length: usize) -> Result<Vec<String>, GitToolError> {
    fetched_tips(bytes, oid_length, true)
}

/// Include not-for-merge tips for recursive fetch: their histories may introduce
/// child commits even though they are not candidates for parent integration.
pub(super) fn fetched_tips(
    bytes: &[u8],
    oid_length: usize,
    merge_only: bool,
) -> Result<Vec<String>, GitToolError> {
    let invalid =
        || GitToolError::InvalidInput("Invalid or oversized Pull FETCH_HEAD evidence".into());
    if !matches!(oid_length, 40 | 64) || bytes.len() > 4 * 1024 * 1024 {
        return Err(invalid());
    }
    let mut candidates = Vec::new();
    for (index, line) in bytes.split_inclusive(|byte| *byte == b'\n').enumerate() {
        if index >= 65_536 {
            return Err(invalid());
        }
        let line = line.strip_suffix(b"\n").ok_or_else(invalid)?;
        let mut fields = line.splitn(3, |byte| *byte == b'\t');
        let oid = fields.next().ok_or_else(invalid)?;
        let marker = fields.next().ok_or_else(invalid)?;
        let _description = fields.next().ok_or_else(invalid)?;
        if oid.len() != oid_length
            || !oid.iter().all(u8::is_ascii_hexdigit)
            || oid.iter().all(|byte| *byte == b'0')
        {
            return Err(invalid());
        }
        match marker {
            b"" => candidates.push(
                std::str::from_utf8(oid)
                    .map_err(|_| invalid())?
                    .to_ascii_lowercase(),
            ),
            b"not-for-merge" if !merge_only => candidates.push(
                std::str::from_utf8(oid)
                    .map_err(|_| invalid())?
                    .to_ascii_lowercase(),
            ),
            b"not-for-merge" => {}
            _ => return Err(invalid()),
        }
    }
    Ok(candidates)
}

/// Validate bounded object-ID listings before retaining them or using them as
/// revision arguments. No ref name or untrusted revision syntax is accepted.
pub(super) fn graph_oids(bytes: &[u8], width: usize) -> Result<Vec<String>, GitToolError> {
    let invalid = || GitToolError::InvalidInput("Invalid or oversized Pull graph listing".into());
    if !matches!(width, 40 | 64) || bytes.len() > 1024 * 1024 {
        return Err(invalid());
    }
    let mut result = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for line in bytes.split_inclusive(|byte| *byte == b'\n') {
        let oid = line.strip_suffix(b"\n").ok_or_else(invalid)?;
        if oid.len() != width
            || !oid.iter().all(u8::is_ascii_hexdigit)
            || oid.iter().all(|byte| *byte == b'0')
        {
            return Err(invalid());
        }
        let oid = std::str::from_utf8(oid)
            .map_err(|_| invalid())?
            .to_ascii_lowercase();
        if seen.insert(oid.clone()) {
            result.push(oid);
        }
    }
    // Keep revision walk order: child name/path association must use the first
    // encountered changed commit, not lexicographic object-ID order.
    Ok(result)
}

#[cfg(test)]
#[path = "pull_evidence_tests.rs"]
mod evidence_tests;

#[cfg(test)]
#[path = "pull_state_tests/evidence.rs"]
mod state_evidence_tests;

#[cfg(test)]
#[path = "pull_state_tests/policy.rs"]
mod policy_tests;

#[cfg(test)]
#[path = "pull_state_tests/activity.rs"]
mod activity_tests;

#[cfg(test)]
#[path = "pull_state_tests/merge.rs"]
mod merge_tests;

#[cfg(test)]
#[path = "pull_state_tests/reconcile.rs"]
mod reconcile_tests;

#[cfg(test)]
#[path = "pull_state_tests/unborn.rs"]
mod unborn_tests;
