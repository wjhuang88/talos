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

#[cfg(test)]
#[test]
fn pure_index_decodes_both_hash_formats_without_worktree_files() {
    for (format, length) in [("sha1", 40), ("sha256", 64)] {
        let root = tempfile::tempdir().expect("fixture");
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .current_dir(root.path())
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_GLOBAL", root.path().join("absent-config"))
                .args(args)
                .output()
                .expect("fixture Git");
            assert!(output.status.success(), "{output:?}");
        };
        git(&["init", &format!("--object-format={format}")]);
        std::fs::write(root.path().join("tracked"), b"content").expect("source");
        git(&["add", "tracked"]);
        std::fs::remove_file(root.path().join("tracked")).expect("remove worktree source");
        let (state, evidence) =
            decode_index(&root.path().join(".git/index"), length).expect("index");
        assert_eq!(state.entries().len(), 1);
        assert_eq!(state.entries()[0].path(&state), b"tracked");
        assert_eq!(state.entries()[0].id.to_string().len(), length);
        assert!(matches!(evidence.entry, super::snapshot::Entry::File(..)));
        assert!(!root.path().join("tracked").exists());
        let path = root.path().join(".git/index");
        let mut bytes = std::fs::read(&path).expect("index bytes");
        let last = bytes.len() - 1;
        bytes[last] ^= 1;
        std::fs::write(&path, &bytes).expect("bad checksum");
        assert!(decode_index(&path, length).is_err());
        let trailer = bytes.len() - length / 2;
        bytes[trailer..].fill(0);
        std::fs::write(&path, &bytes).expect("skipHash checksum");
        assert!(decode_index(&path, length).is_ok());
        std::fs::write(root.path().join(".git/index"), b"truncated").expect("corrupt index");
        assert!(decode_index(&root.path().join(".git/index"), length).is_err());
    }
}

/// Retained only after the repository roots and these metadata paths receive
/// Read admission. This evidence grants no authority and is not confinement.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct PullEvidence {
    repository: super::repository_state::RepositoryState,
    roots: Vec<super::snapshot::PathIdentity>,
    tracking: Option<String>,
    fetched: bool,
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
    pub(super) fn capture(
        repository: &super::repository_state::RepositoryState,
        tracking: Option<&str>,
        fetched: bool,
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
        let mut budget = super::snapshot::Budget {
            entries: paths.len(),
            bytes: 16 * 1024 * 1024,
        };
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
            let evidence = super::snapshot::RegularEvidence::capture(&path, &mut budget)
                .map_err(|_| invalid())?;
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
            tracking: tracking.map(str::to_owned),
            fetched,
            index_modified,
            entries,
        })
    }

    pub(super) fn revalidate(&self) -> Result<(), GitToolError> {
        // Refuse changed roots/pointers before rediscovery can read new roots.
        for (root, identity) in [
            &self.repository.git_dir,
            &self.repository.common_dir,
            &self.repository.workdir,
        ]
        .into_iter()
        .zip(&self.roots)
        {
            if &super::snapshot::PathIdentity::capture(root).map_err(|_| {
                GitToolError::InvalidInput("Pull repository root unavailable".into())
            })? != identity
            {
                return Err(GitToolError::InvalidInput(
                    "Pull repository root changed".into(),
                ));
            }
        }
        self.repository.revalidate_location()?;
        if Self::capture(&self.repository, self.tracking.as_deref(), self.fetched)? != *self {
            return Err(GitToolError::InvalidInput(
                "Pull reference/reflog evidence changed after inspection".into(),
            ));
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

#[cfg(unix)]
#[cfg(test)]
#[test]
fn pull_sparse_patterns_are_exact_evidence_even_when_absent() {
    let root = tempfile::tempdir().expect("fixture");
    let output = std::process::Command::new("git")
        .arg("init")
        .arg(root.path())
        .output()
        .expect("Git fixture");
    assert!(output.status.success());
    let repository =
        super::repository_state::RepositoryState::capture(root.path()).expect("repository");
    let patterns = repository.git_dir.join("info/sparse-checkout");
    assert!(
        PullEvidence::required_paths(&repository, None, false)
            .expect("paths")
            .contains(&patterns)
    );
    let absent = PullEvidence::capture(&repository, None, false).expect("absent patterns");
    std::fs::write(&patterns, b"/*\n!/*/\n").expect("create patterns");
    assert!(absent.revalidate().is_err());
    let present = PullEvidence::capture(&repository, None, false).expect("present patterns");
    std::fs::write(&patterns, b"/*\n").expect("change patterns");
    assert!(present.revalidate().is_err());
    std::fs::remove_file(&patterns).expect("remove patterns");
    std::os::unix::fs::symlink(root.path().join("external"), &patterns).expect("redirect");
    assert!(PullEvidence::capture(&repository, None, false).is_err());
}

#[cfg(unix)]
#[cfg(test)]
#[test]
fn pull_evidence_refuses_reflog_escape_oversize_and_missing_leaf_creation() {
    let root = tempfile::tempdir().expect("fixture");
    let work = root.path().join("work");
    std::fs::create_dir(&work).expect("worktree");
    let output = std::process::Command::new("git")
        .args(["init", "-b", "Topic"])
        .current_dir(&work)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", root.path().join("absent-config"))
        .output()
        .expect("fixture Git");
    assert!(output.status.success());
    let repository = super::repository_state::RepositoryState::capture(&work).expect("repository");
    let clean = PullEvidence::capture(&repository, None, false).expect("clean merge state");
    clean.require_no_pending_merge().expect("no pending merge");
    let index_path = repository.git_dir.join("index");
    std::fs::write(work.join("tracked"), b"timestamp evidence").expect("index source");
    let add = std::process::Command::new("git")
        .args(["add", "tracked"])
        .current_dir(&work)
        .output()
        .expect("index fixture");
    assert!(add.status.success());
    let indexed_repository =
        super::repository_state::RepositoryState::capture(&work).expect("index repository");
    let indexed = PullEvidence::capture(&indexed_repository, None, false).expect("index evidence");
    std::fs::File::options()
        .write(true)
        .open(&index_path)
        .expect("index handle")
        .set_modified(std::time::UNIX_EPOCH)
        .expect("change only timestamp");
    assert!(
        indexed.revalidate().is_err(),
        "index timestamp drift invalidates evidence"
    );
    std::fs::remove_file(&index_path).expect("remove index fixture");
    let repository =
        super::repository_state::RepositoryState::capture(&work).expect("repository after index");
    std::fs::write(repository.git_dir.join("MERGE_HEAD"), b"").expect("pending merge marker");
    assert!(
        clean.revalidate().is_err(),
        "new merge invalidates admission evidence"
    );
    let pending = PullEvidence::capture(&repository, None, false).expect("pending merge evidence");
    assert!(
        pending.require_no_pending_merge().is_err(),
        "even an empty marker blocks fetch"
    );
    std::fs::remove_file(repository.git_dir.join("MERGE_HEAD")).expect("remove merge marker");
    assert!(PullEvidence::capture(&repository, Some("refs/../outside"), false).is_err());
    let parent = repository.git_dir.join("logs/refs/remotes");
    std::fs::create_dir_all(&parent).expect("logs parent");
    let outside = root.path().join("outside");
    std::fs::create_dir(&outside).expect("outside");
    std::os::unix::fs::symlink(&outside, parent.join("origin")).expect("escaped parent");
    assert!(PullEvidence::capture(&repository, Some("refs/remotes/origin/Topic"), false).is_err());
    std::fs::remove_file(parent.join("origin")).expect("remove fixture link");
    let internal = repository.git_dir.join("private");
    std::fs::create_dir(&internal).expect("internal target");
    std::os::unix::fs::symlink(&internal, parent.join("origin")).expect("internal parent link");
    assert!(
        PullEvidence::capture(&repository, Some("refs/remotes/origin/Topic"), false).is_err(),
        "root-internal retarget still needs its own Read admission"
    );
    std::fs::remove_file(parent.join("origin")).expect("remove internal fixture link");
    std::fs::create_dir(parent.join("origin")).expect("real parent");
    let before = PullEvidence::capture(&repository, Some("refs/remotes/origin/Topic"), false)
        .expect("missing leaf evidence");
    let path = parent.join("origin/Topic");
    std::fs::write(&path, "created after inspection").expect("create leaf");
    assert!(before.revalidate().is_err());
    std::fs::File::create(&path)
        .expect("large leaf")
        .set_len(16 * 1024 * 1024 + 1)
        .expect("oversize");
    assert!(PullEvidence::capture(&repository, Some("refs/remotes/origin/Topic"), false).is_err());
    std::fs::write(&path, "").expect("restore bounded leaf");
    let bound = PullEvidence::capture(&repository, Some("refs/remotes/origin/Topic"), false)
        .expect("bound location");
    std::fs::write(
        repository.git_dir.join("commondir"),
        outside.to_str().expect("outside path"),
    )
    .expect("redirect common root");
    let error = bound
        .revalidate()
        .expect_err("redirect must fail before inspecting new root");
    assert!(
        error.to_string().contains("location pointer changed"),
        "{error}"
    );
}

#[cfg(test)]
#[test]
fn postfetch_evidence_rebaselines_changed_metadata_only_under_retained_location() {
    let root = tempfile::tempdir().expect("fixture");
    let source = root.path().join("source");
    let local = root.path().join("local");
    let git = |cwd: &std::path::Path, args: &[&str]| {
        let output = std::process::Command::new("git")
            .current_dir(cwd)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", root.path().join("absent-config"))
            .args([
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
            ])
            .args(args)
            .output()
            .expect("fixture Git");
        assert!(output.status.success(), "{args:?}: {output:?}");
        output.stdout
    };
    std::fs::create_dir(&source).expect("source");
    git(&source, &["init", "-b", "main"]);
    std::fs::write(source.join("file"), b"one\n").expect("source file");
    git(&source, &["add", "file"]);
    git(&source, &["commit", "-m", "one"]);
    git(
        root.path(),
        &[
            "clone",
            source.to_str().expect("source"),
            local.to_str().expect("local"),
        ],
    );
    let original = super::repository_state::RepositoryState::capture(&local).expect("before");
    let prefetch = PullEvidence::capture(&original, Some("refs/remotes/origin/main"), false)
        .expect("prefetch evidence");
    std::fs::write(source.join("file"), b"two\n").expect("source change");
    git(&source, &["add", "file"]);
    git(&source, &["commit", "-m", "two"]);
    let expected = git(&source, &["rev-parse", "HEAD"]);
    git(&local, &["fetch", "--no-recurse-submodules", "origin"]);
    original
        .revalidate_location()
        .expect("old location retained");
    assert_ne!(
        original,
        super::repository_state::RepositoryState::capture(&local).expect("changed graph")
    );
    assert!(
        prefetch.revalidate().is_err(),
        "old content evidence is stale"
    );
    let fetched = PullEvidence::capture(&original, Some("refs/remotes/origin/main"), true)
        .expect("new content under old roots");
    assert_eq!(
        merge_candidates(fetched.fetch_head().expect("FETCH_HEAD"), 40).expect("candidate"),
        [String::from_utf8(expected).expect("OID").trim().to_owned()]
    );
    fetched.revalidate().expect("stable post-fetch evidence");
    let path = original.git_dir.join("FETCH_HEAD");
    let saved = std::fs::read(&path).expect("fetched bytes");
    std::fs::write(&path, b"replacement\n").expect("drift");
    assert!(fetched.revalidate().is_err());
    std::fs::write(&path, saved).expect("restore");
    let redirected = root.path().join("unapproved");
    std::fs::create_dir(&redirected).expect("external root");
    std::fs::write(
        original.git_dir.join("commondir"),
        redirected.to_str().expect("path"),
    )
    .expect("redirect pointer");
    let error = PullEvidence::capture(&original, None, true)
        .expect_err("old admission must not read the new location");
    assert!(
        error.to_string().contains("location pointer changed"),
        "{error}"
    );
    std::fs::remove_file(original.git_dir.join("commondir")).expect("restore pointer");
    std::fs::rename(&original.git_dir, local.join("saved-gitdir")).expect("retain original");
    std::fs::create_dir(&original.git_dir).expect("replace directory");
    let error = PullEvidence::capture(&original, None, true)
        .expect_err("replacement identity must fail before repository rediscovery");
    assert!(
        error.to_string().contains("root identity changed"),
        "{error}"
    );
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

#[cfg(test)]
#[test]
fn tracking_mapping_preserves_first_merge_and_positive_fetch_order() {
    let config = b"branch.Topic.remote\nother\0branch.Topic.merge\nrefs/heads/main\0branch.Topic.merge\nrefs/heads/second\0remote.other.fetch\n^refs/heads/main\0remote.other.fetch\n+refs/heads/*:refs/custom/*\0remote.other.fetch\nrefs/heads/main:refs/alternate/main\0";
    assert_eq!(
        tracking_reference(config, "Topic", "other")
            .expect("mapping")
            .as_deref(),
        Some("refs/custom/main")
    );
    assert!(
        tracking_reference(config, "Topic", "origin")
            .expect("different remote")
            .is_none()
    );
    assert!(
        tracking_reference(config, "topic", "other")
            .expect("case")
            .is_none()
    );
    assert_eq!(
        tracking_reference(
            b"branch.Topic.remote\n.\0branch.Topic.merge\nrefs/heads/main\0",
            "Topic",
            "."
        )
        .expect("local")
        .as_deref(),
        Some("refs/heads/main")
    );
    assert!(tracking_reference(b"branch.Topic.remote\nother\0branch.Topic.merge\nrefs/heads/main\0remote.other.fetch\nrefs/heads/**:refs/x/*\0", "Topic", "other").is_err());
}

#[cfg(test)]
#[test]
fn fetch_remote_preserves_branch_and_single_remote_defaults() {
    let config = b"remote.Up.stream.url\n/path\0branch.Topic.remote\n.\0branch.topic.remote\nother\0remote.pushdefault\npush-only\0";
    assert_eq!(
        fetch_remote(config, Some("Topic"), None).expect("local"),
        "."
    );
    assert_eq!(
        fetch_remote(config, Some("topic"), None).expect("branch"),
        "other"
    );
    assert_eq!(fetch_remote(config, None, None).expect("sole"), "Up.stream");
    assert_eq!(
        fetch_remote(config, Some("Topic"), Some("override")).expect("explicit"),
        "override"
    );
    let mut multiple = config.to_vec();
    multiple.extend_from_slice(b"remote.second.fetch\n+refs/heads/*:refs/remotes/second/*\0");
    assert_eq!(
        fetch_remote(&multiple, None, None).expect("fallback"),
        "origin"
    );
    assert!(fetch_remote(b"branch.Topic.remote\0", Some("Topic"), None).is_err());
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct BeforeFetch {
    branch: Option<String>,
    head: Option<String>,
    fork_point: Option<String>,
    policy: Policy,
    fetch_recursion: FetchRecursion,
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
        self.integration_stage(fetched, oid_length, false, run)
            .await
    }

    async fn integration_stage<F, Fut>(
        &self,
        fetched: &[u8],
        oid_length: usize,
        reconciled: bool,
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
        if current.head != self.head {
            match (&self.head, &current.head) {
                (Some(original), Some(current)) if !reconciled => {
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
        let mut args = if rebase && !can_ff {
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
            let mut args = vec!["rebase".into()];
            match self.policy.rebase {
                Rebase::Merges => args.push("--rebase-merges".into()),
                Rebase::Interactive => args.push("--interactive".into()),
                _ => {}
            }
            args.extend(["--onto".into(), heads[0].clone(), upstream]);
            args
        } else {
            let mut args = vec!["merge".into()];
            let ff = if rebase && can_ff {
                &FastForward::Only
            } else {
                &self.policy.fast_forward
            };
            match ff {
                FastForward::Allow => args.push("--ff".into()),
                FastForward::Never => args.push("--no-ff".into()),
                FastForward::Only => args.push("--ff-only".into()),
                FastForward::Unspecified => {}
            }
            // Preserve FETCH_HEAD descriptions for merge messages. Its bytes
            // must remain bound to the final integration approval.
            args.push("FETCH_HEAD".into());
            args
        };
        args.insert(
            1,
            if self.policy.autostash {
                "--autostash"
            } else {
                "--no-autostash"
            }
            .into(),
        );
        Ok(Integration::Command(args))
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
        Ok(Self {
            branch,
            head,
            fork_point,
            policy,
            fetch_recursion,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Integration {
    Unborn(String),
    Command(Vec<String>),
    Reconcile { original: String, current: String },
}

#[derive(Debug, PartialEq, Eq)]
enum Rebase {
    Off,
    On,
    Merges,
    Interactive,
}

#[derive(Debug, PartialEq, Eq)]
enum FastForward {
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

#[cfg(test)]
#[test]
fn prefetch_evidence_distinguishes_detached_unborn_and_rejects_invalid_probes() {
    let oid = format!("{}\n", "a".repeat(40));
    let detached =
        BeforeFetch::capture(b"", None, Some(oid.as_bytes()), None, 40).expect("detached");
    assert!(detached.branch.is_none() && detached.head.is_some());
    let unborn =
        BeforeFetch::capture(b"", Some(b"refs/heads/Topic\n"), None, None, 40).expect("unborn");
    assert_eq!(unborn.branch.as_deref(), Some("Topic"));
    assert!(unborn.head.is_none());
    let born = BeforeFetch::capture(
        b"",
        Some(b"refs/heads/Topic\n"),
        Some(oid.as_bytes()),
        None,
        40,
    )
    .expect("born branch");
    assert!(!unborn.same_checkout(&born));
    assert!(!born.same_checkout(&detached));
    let changed_oid = format!("{}\n", "b".repeat(40));
    let moved = BeforeFetch::capture(
        b"",
        Some(b"refs/heads/Topic\n"),
        Some(changed_oid.as_bytes()),
        None,
        40,
    )
    .expect("moved branch");
    assert!(!born.same_checkout(&moved));
    let with_fork = BeforeFetch::capture(
        b"",
        Some(b"refs/heads/Topic\n"),
        Some(oid.as_bytes()),
        Some(changed_oid.as_bytes()),
        40,
    )
    .expect("fork point discovered");
    assert!(born.same_checkout(&with_fork));
    assert!(
        BeforeFetch::capture(b"", None, Some(oid.as_bytes()), Some(oid.as_bytes()), 40).is_err()
    );
    for bad in [
        b"refs/tags/tag\n".as_slice(),
        b"refs/heads/\n",
        b"refs/heads/Topic\nextra\n",
    ] {
        assert!(BeforeFetch::capture(b"", Some(bad), None, None, 40).is_err());
    }
    for bad in ["0".repeat(40), "g".repeat(40), "a".repeat(39)] {
        assert!(
            BeforeFetch::capture(b"", None, Some(format!("{bad}\n").as_bytes()), None, 40).is_err()
        );
    }
}

#[cfg(test)]
#[test]
fn pull_fetch_recursion_preserves_modes_and_declaration_order() {
    for (config, expected) in [
        (b"".as_slice(), FetchRecursion::Default),
        (
            b"submodule.Child.fetchrecursesubmodules\ntrue\0",
            FetchRecursion::Default,
        ),
        (
            b"fetch.recursesubmodules\non-demand\0",
            FetchRecursion::OnDemand,
        ),
        (b"fetch.recursesubmodules\0", FetchRecursion::On),
        (b"fetch.recursesubmodules\n\0", FetchRecursion::Off),
        (
            b"submodule.recurse\ntrue\0fetch.recursesubmodules\nfalse\0",
            FetchRecursion::Off,
        ),
        (
            b"fetch.recursesubmodules\nfalse\0submodule.recurse\ntrue\0",
            FetchRecursion::On,
        ),
        (
            b"submodule.recurse\nfalse\0fetch.recursesubmodules\non-demand\0",
            FetchRecursion::OnDemand,
        ),
        (
            b"fetch.recursesubmodules\non-demand\0submodule.recurse\nfalse\0",
            FetchRecursion::Off,
        ),
        (
            b"fetch.recursesubmodules\ntrue\0fetch.recursesubmodules\nfalse\0",
            FetchRecursion::Off,
        ),
    ] {
        let retained = BeforeFetch::capture(config, None, None, None, 40).expect("retained policy");
        assert_eq!(retained.fetch_recursion, expected, "{config:?}");
    }
    // An invalid earlier declaration is fatal even if a later value overrides
    // it, because native fetch validates configuration before parsing argv.
    for config in [
        b"fetch.recursesubmodules\ninvalid\0fetch.recursesubmodules\nfalse\0".as_slice(),
        b"submodule.recurse\non-demand\0fetch.recursesubmodules\nfalse\0",
        b"fetch.recursesubmodules\nON-DEMAND\0",
        b"fetch.recursesubmodules\nfalse",
    ] {
        assert!(
            BeforeFetch::capture(config, None, None, None, 40).is_err(),
            "{config:?}"
        );
    }
}

#[cfg(test)]
#[test]
fn pull_policy_preserves_branch_override_and_autostash_precedence() {
    let config = b"pull.rebase\ntrue\0branch.Topic.rebase\nfalse\0branch.topic.rebase\nmerges\0pull.ff\nonly\0rebase.autostash\ntrue\0merge.autostash\nfalse\0";
    let topic = Policy::capture(config, Some("Topic")).expect("branch policy");
    assert_eq!(topic.rebase, Rebase::Off);
    assert!(!topic.autostash);
    assert_eq!(topic.fast_forward, FastForward::Only);
    let lower = Policy::capture(config, Some("topic")).expect("case-sensitive branch");
    assert_eq!(lower.rebase, Rebase::Merges);
    assert!(lower.autostash);
    let detached = Policy::capture(config, None).expect("detached policy");
    assert_eq!(detached.rebase, Rebase::On);
    assert!(detached.autostash);
    let mut override_config = config.to_vec();
    override_config.extend_from_slice(b"pull.autostash\nfalse\0");
    assert!(
        !Policy::capture(&override_config, None)
            .expect("override")
            .autostash
    );
    let default = Policy::capture(b"", None).expect("defaults");
    assert!(default.rebase_unspecified);
    assert_eq!(default.fast_forward, FastForward::Unspecified);
    assert!(!default.autostash);
    for invalid in [
        b"pull.rebase\npreserve\0".as_slice(),
        b"pull.ff\ninvalid\0",
        b"pull.autostash\ninvalid\0",
    ] {
        assert!(Policy::capture(invalid, None).is_err());
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
    for line in bytes.split_inclusive(|byte| *byte == b'\n') {
        let oid = line.strip_suffix(b"\n").ok_or_else(invalid)?;
        if oid.len() != width
            || !oid.iter().all(u8::is_ascii_hexdigit)
            || oid.iter().all(|byte| *byte == b'0')
        {
            return Err(invalid());
        }
        result.push(
            std::str::from_utf8(oid)
                .map_err(|_| invalid())?
                .to_ascii_lowercase(),
        );
    }
    result.sort();
    result.dedup();
    Ok(result)
}

#[cfg(test)]
#[test]
fn fetched_heads_preserve_merge_order_and_ignore_description_bytes() {
    for width in [40, 64] {
        let first = "a".repeat(width);
        let second = "b".repeat(width);
        let ignored = "c".repeat(width);
        let mut bytes =
            format!("{first}\t\tfirst\n{ignored}\tnot-for-merge\tignored\n{second}\t\t")
                .into_bytes();
        bytes.extend_from_slice(b"non-utf8 \xff\tcomment\n");
        assert_eq!(
            merge_candidates(&bytes, width).expect("heads"),
            [first.clone(), second.clone()]
        );
        assert_eq!(
            fetched_tips(&bytes, width, false).expect("all fetched tips"),
            [first, ignored, second]
        );
    }
    assert!(merge_candidates(b"", 40).expect("empty").is_empty());
}

#[cfg(test)]
#[test]
fn pull_graph_listings_accept_only_complete_object_ids() {
    for width in [40, 64] {
        let oid = "a".repeat(width);
        assert_eq!(
            graph_oids(format!("{oid}\n{}\n", oid.to_uppercase()).as_bytes(), width)
                .expect("normalized tips"),
            std::slice::from_ref(&oid)
        );
        for bad in [
            oid.clone(),
            format!("{}\n", "0".repeat(width)),
            format!("{}\n", "g".repeat(width)),
            format!("{oid}\n--all\n"),
            format!("{oid}\0\n"),
        ] {
            assert!(graph_oids(bad.as_bytes(), width).is_err());
        }
    }
    assert!(graph_oids(b"", 40).expect("empty refs").is_empty());
    assert!(graph_oids(b"", 32).is_err());
    assert!(graph_oids(&vec![b'a'; 1024 * 1024 + 1], 40).is_err());
}

#[cfg(test)]
#[test]
fn malformed_fetch_evidence_cannot_become_an_approved_target() {
    let oid = "a".repeat(40);
    for bytes in [
        format!("{oid}\t\tunterminated"),
        format!("{oid}\tunknown\tdescription\n"),
        format!("{oid}\t\n"),
        format!("{}\t\tzero\n", "0".repeat(40)),
        format!("{}\t\tinvalid\n", "g".repeat(40)),
    ] {
        assert!(merge_candidates(bytes.as_bytes(), 40).is_err());
    }
    assert!(merge_candidates(&vec![b'x'; 4 * 1024 * 1024 + 1], 40).is_err());
    assert!(merge_candidates(format!("{oid}\t\tline\n").repeat(65_537).as_bytes(), 40).is_err());
}

#[cfg(test)]
#[test]
fn real_fetch_selects_upstream_only_from_all_fetched_branches() {
    let source = tempfile::tempdir().expect("source");
    let destination = tempfile::tempdir().expect("destination");
    let git = |cwd: &std::path::Path, args: &[&str]| {
        let output = std::process::Command::new("git")
            .current_dir(cwd)
            .args(args)
            .output()
            .expect("fixture Git");
        assert!(output.status.success(), "{output:?}");
        String::from_utf8(output.stdout).expect("Git output")
    };
    git(source.path(), &["init", "-b", "upstream"]);
    git(
        source.path(),
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgSign=false",
            "commit",
            "--allow-empty",
            "-m",
            "base",
        ],
    );
    let expected = git(source.path(), &["rev-parse", "HEAD"]).trim().to_owned();
    git(source.path(), &["branch", "unrelated"]);
    git(destination.path(), &["init", "-b", "local"]);
    git(
        destination.path(),
        &[
            "remote",
            "add",
            "origin",
            source.path().to_str().expect("source path"),
        ],
    );
    git(
        destination.path(),
        &["config", "branch.local.remote", "origin"],
    );
    git(
        destination.path(),
        &["config", "branch.local.merge", "refs/heads/upstream"],
    );
    git(
        destination.path(),
        &["-c", "maintenance.auto=false", "fetch", "origin"],
    );
    let evidence =
        std::fs::read(destination.path().join(".git/FETCH_HEAD")).expect("fetch evidence");
    assert!(
        evidence
            .windows(b"not-for-merge".len())
            .any(|bytes| bytes == b"not-for-merge")
    );
    assert_eq!(
        merge_candidates(&evidence, expected.len()).expect("upstream candidate"),
        [expected]
    );
}

#[cfg(test)]
#[tokio::test]
async fn real_pull_obeys_branch_ff_and_autostash_policy() {
    for scenario in [
        "branch-merge",
        "branch-rebase",
        "rewritten-upstream",
        "configured-remote",
        "custom-tracking",
        "missing-tracking",
        "fast-forward",
        "merge-no-ff",
        "unspecified-policy",
        "ff-only",
        "stash",
        "no-stash",
    ] {
        let root = tempfile::tempdir().expect("fixture");
        let source = root.path().join("source");
        let local = root.path().join("local");
        std::fs::create_dir(&source).expect("source directory");
        let run = |cwd: &std::path::Path, args: &[&str]| {
            std::process::Command::new("git")
                .current_dir(cwd)
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_GLOBAL", root.path().join("absent-config"))
                .env("GIT_TERMINAL_PROMPT", "0")
                .env("GIT_EDITOR", "true")
                .args([
                    "-c",
                    "user.name=Fixture",
                    "-c",
                    "user.email=fixture@example.invalid",
                    "-c",
                    "commit.gpgSign=false",
                    "-c",
                    "maintenance.auto=false",
                ])
                .args(args)
                .output()
                .expect("fixture Git")
        };
        let git = |cwd: &std::path::Path, args: &[&str]| {
            let output = run(cwd, args);
            assert!(output.status.success(), "{scenario} {args:?}: {output:?}");
            String::from_utf8(output.stdout).expect("output")
        };
        git(&source, &["init", "-b", "Topic"]);
        std::fs::write(source.join("dirty"), "base\n").expect("base file");
        git(&source, &["add", "."]);
        git(&source, &["commit", "-m", "base"]);
        let base = git(&source, &["rev-parse", "HEAD"]);
        let old_upstream = if scenario == "rewritten-upstream" {
            git(
                &source,
                &["commit", "--allow-empty", "-m", "obsolete upstream"],
            );
            Some(git(&source, &["rev-parse", "HEAD"]))
        } else {
            None
        };
        git(
            root.path(),
            &[
                "clone",
                source.to_str().expect("source"),
                local.to_str().expect("local"),
            ],
        );
        if scenario != "fast-forward" {
            std::fs::write(local.join("local-only"), "local\n").expect("local change");
            git(&local, &["add", "."]);
            git(&local, &["commit", "-m", "local"]);
        }
        let before = git(&local, &["rev-parse", "HEAD"]);
        if let Some(old) = &old_upstream {
            assert_eq!(
                git(
                    &local,
                    &["merge-base", "--fork-point", "origin/Topic", "HEAD"]
                ),
                *old
            );
            git(&source, &["reset", "--hard", base.trim()]);
        }
        std::fs::write(source.join("remote-only"), "remote\n").expect("remote change");
        git(&source, &["add", "."]);
        git(&source, &["commit", "-m", "remote"]);
        let remote = git(&source, &["rev-parse", "HEAD"]);
        if scenario == "configured-remote" {
            git(&local, &["remote", "rename", "origin", "Up.stream"]);
        }
        if scenario == "custom-tracking" {
            git(
                &local,
                &[
                    "config",
                    "--replace-all",
                    "remote.origin.fetch",
                    "+refs/heads/*:refs/custom/*",
                ],
            );
            git(
                &local,
                &[
                    "update-ref",
                    "refs/custom/Topic",
                    "refs/remotes/origin/Topic",
                ],
            );
        }
        let merging = matches!(scenario, "branch-merge" | "merge-no-ff");
        git(
            &local,
            &[
                "config",
                "pull.rebase",
                if merging { "true" } else { "false" },
            ],
        );
        git(
            &local,
            &[
                "config",
                "branch.Topic.rebase",
                if merging { "false" } else { "true" },
            ],
        );
        if scenario == "unspecified-policy" {
            git(&local, &["config", "--unset", "pull.rebase"]);
            git(&local, &["config", "--unset", "branch.Topic.rebase"]);
        }
        if scenario == "ff-only" {
            git(&local, &["config", "pull.ff", "only"]);
        }
        if scenario == "merge-no-ff" {
            git(&local, &["config", "pull.ff", "false"]);
        }
        if matches!(scenario, "stash" | "no-stash") {
            git(
                &local,
                &[
                    "config",
                    "rebase.autostash",
                    if scenario == "stash" { "false" } else { "true" },
                ],
            );
            git(
                &local,
                &[
                    "config",
                    "pull.autostash",
                    if scenario == "stash" { "true" } else { "false" },
                ],
            );
            std::fs::write(local.join("dirty"), "uncommitted\n").expect("dirty worktree");
        }
        let config = run(&local, &["config", "--null", "--list"]);
        assert!(config.status.success());
        let policy = Policy::capture(&config.stdout, Some("Topic")).expect("policy");
        let selected = fetch_remote(&config.stdout, Some("Topic"), None).expect("remote selection");
        assert_eq!(
            selected,
            if scenario == "configured-remote" {
                "Up.stream"
            } else {
                "origin"
            }
        );
        assert_eq!(
            policy.rebase,
            if merging || scenario == "unspecified-policy" {
                Rebase::Off
            } else {
                Rebase::On
            }
        );
        assert_eq!(policy.autostash, scenario == "stash");
        let tracking =
            tracking_reference(&config.stdout, "Topic", &selected).expect("upstream mapping");
        let actual = git(
            &local,
            &["rev-parse", "--symbolic-full-name", "@{upstream}"],
        );
        assert_eq!(tracking.as_deref(), Some(actual.trim()));
        if scenario == "missing-tracking" {
            git(
                &local,
                &["update-ref", "-d", tracking.as_deref().expect("tracking")],
            );
        }
        let repository =
            super::repository_state::RepositoryState::capture(&local).expect("repository evidence");
        let before_evidence = PullEvidence::capture(&repository, tracking.as_deref(), false)
            .expect("pre-fetch reference/reflog evidence");
        let retained = BeforeFetch::probe(&config.stdout, tracking.as_deref(), 40, |args| {
            let output = run(&local, &args.iter().map(String::as_str).collect::<Vec<_>>());
            async move { Ok(output) }
        })
        .await
        .expect("actual pre-fetch probes");
        before_evidence
            .revalidate()
            .expect("stable pre-fetch probes");
        if scenario == "branch-rebase" {
            let reflog = local
                .join(".git/logs")
                .join(tracking.as_deref().expect("tracking"));
            let saved = match std::fs::read(&reflog) {
                Ok(bytes) => Some(bytes),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                Err(error) => panic!("tracking reflog: {error}"),
            };
            std::fs::create_dir_all(reflog.parent().expect("reflog parent"))
                .expect("reflog directory");
            // Capture after constructing the missing parent hierarchy, so this
            // regression isolates leaf creation/content from parent movement.
            let before_evidence = PullEvidence::capture(&repository, tracking.as_deref(), false)
                .expect("reflog leaf evidence");
            let mut changed = saved.clone().unwrap_or_default();
            changed.extend_from_slice(b"changed evidence\n");
            std::fs::write(&reflog, changed).expect("mutate reflog only");
            assert!(
                before_evidence.revalidate().is_err(),
                "unchanged refs cannot hide reflog movement"
            );
            if let Some(saved) = saved {
                std::fs::write(&reflog, saved).expect("restore reflog");
            } else {
                std::fs::remove_file(&reflog).expect("restore missing reflog");
            }
            before_evidence.revalidate().expect("restored evidence");
        }
        if scenario == "missing-tracking" {
            assert_eq!(retained.fork_point, None);
        }
        // Compare the explicit fetch/integration boundary with native pull on
        // the same repository, including its reflogs and configuration.
        fn copy_tree(source: &std::path::Path, destination: &std::path::Path) {
            std::fs::create_dir(destination).expect("copy directory");
            for entry in std::fs::read_dir(source).expect("copy entries") {
                let entry = entry.expect("entry");
                let target = destination.join(entry.file_name());
                if entry.file_type().expect("entry type").is_dir() {
                    copy_tree(&entry.path(), &target);
                } else {
                    std::fs::copy(entry.path(), target).expect("copy fixture file");
                }
            }
        }
        let split = root.path().join("split");
        copy_tree(&local, &split);
        git(&split, &["fetch", &selected]);
        let fetched = std::fs::read(split.join(".git/FETCH_HEAD")).expect("fetched evidence");
        if scenario == "branch-rebase" {
            git(&split, &["update-ref", "HEAD", remote.trim()]);
            let moved = retained
                .integration(&fetched, 40, |args| {
                    let output = run(&split, &args.iter().map(String::as_str).collect::<Vec<_>>());
                    async move { Ok(output) }
                })
                .await
                .expect("post-fetch HEAD movement needs explicit reconciliation");
            assert!(
                matches!(moved, Integration::Reconcile { original, current, .. } if original == before.trim() && current == remote.trim())
            );
            git(&split, &["update-ref", "HEAD", before.trim()]);
        }
        let fetched_repository =
            super::repository_state::RepositoryState::capture(&split).expect("fetched repository");
        let fetched_evidence =
            PullEvidence::capture(&fetched_repository, tracking.as_deref(), true)
                .expect("post-fetch evidence");
        assert_eq!(
            fetched_evidence.fetch_head().expect("bound FETCH_HEAD"),
            fetched
        );
        let failed_probe = retained
            .integration(&fetched, 40, |_| async {
                Err(GitToolError::Git("fixture collector failure".into()))
            })
            .await
            .expect_err("infrastructure failure must propagate");
        assert!(
            failed_probe
                .to_string()
                .contains("fixture collector failure")
        );
        let integration = retained
            .integration(
                fetched_evidence.fetch_head().expect("bound FETCH_HEAD"),
                40,
                |args| {
                    let output = run(&split, &args.iter().map(String::as_str).collect::<Vec<_>>());
                    async move { Ok(output) }
                },
            )
            .await;
        fetched_evidence
            .revalidate()
            .expect("stable integration probes");
        if scenario == "branch-rebase" {
            let path = split.join(".git/FETCH_HEAD");
            let mut changed = fetched.clone();
            changed.extend_from_slice(b"changed evidence\n");
            std::fs::write(&path, changed).expect("change FETCH_HEAD after inspection");
            assert!(fetched_evidence.revalidate().is_err());
            std::fs::write(&path, &fetched).expect("restore FETCH_HEAD");
            fetched_evidence.revalidate().expect("restored FETCH_HEAD");
        }
        let split_success = match integration {
            Ok(Integration::Command(args)) => {
                run(&split, &args.iter().map(String::as_str).collect::<Vec<_>>())
                    .status
                    .success()
            }
            Ok(Integration::Unborn(_) | Integration::Reconcile { .. }) => {
                panic!("unchanged born fixture")
            }
            Err(error) => {
                assert!(
                    matches!(scenario, "ff-only" | "unspecified-policy"),
                    "unexpected integration refusal: {error}"
                );
                false
            }
        };
        let output = run(&local, &["pull"]);
        let refused = matches!(scenario, "ff-only" | "no-stash" | "unspecified-policy");
        assert_eq!(output.status.success(), !refused, "{scenario}: {output:?}");
        assert_eq!(
            split_success,
            output.status.success(),
            "split/native status: {scenario}"
        );
        assert_eq!(
            git(&split, &["rev-parse", "HEAD^{tree}"]),
            git(&local, &["rev-parse", "HEAD^{tree}"])
        );
        assert_eq!(
            git(&split, &["log", "--format=%s"]),
            git(&local, &["log", "--format=%s"])
        );
        if refused {
            assert_eq!(git(&local, &["rev-parse", "HEAD"]), before);
        } else {
            let parents = git(&local, &["rev-list", "--parents", "-n", "1", "HEAD"]);
            assert_eq!(
                parents.split_whitespace().count(),
                if merging { 3 } else { 2 }
            );
            git(
                &local,
                &["merge-base", "--is-ancestor", remote.trim(), "HEAD"],
            );
            if let Some(old) = &old_upstream {
                assert_eq!(retained.head.as_deref(), Some(before.trim()));
                assert_eq!(retained.fork_point.as_deref(), Some(old.trim()));
                let obsolete = run(&local, &["merge-base", "--is-ancestor", old.trim(), "HEAD"]);
                assert_eq!(
                    obsolete.status.code(),
                    Some(1),
                    "obsolete upstream replayed"
                );
                assert_eq!(git(&local, &["rev-list", "--count", "HEAD"]).trim(), "3");
            }
        }
        if matches!(scenario, "stash" | "no-stash") {
            assert_eq!(
                std::fs::read(local.join("dirty")).expect("preserved edits"),
                b"uncommitted\n"
            );
            assert_eq!(
                std::fs::read(split.join("dirty")).expect("split preserved edits"),
                b"uncommitted\n"
            );
            assert_eq!(
                git(&split, &["stash", "list"]),
                git(&local, &["stash", "list"])
            );
        }
    }
}

#[cfg(test)]
#[cfg(unix)]
#[test]
fn pull_prefetch_binds_fetch_head_destination_before_mutation() {
    let root = tempfile::tempdir().expect("fixture");
    let work = root.path().join("work");
    std::fs::create_dir(&work).expect("worktree");
    let output = std::process::Command::new("git")
        .args(["init", "-b", "Topic"])
        .current_dir(&work)
        .output()
        .expect("fixture Git");
    assert!(output.status.success());
    let repository = super::repository_state::RepositoryState::capture(&work).expect("repository");
    let path = repository.git_dir.join("FETCH_HEAD");
    let absent = PullEvidence::capture(&repository, None, false).expect("absent destination");
    std::fs::write(&path, b"previous fetch\n").expect("create destination");
    assert!(absent.revalidate().is_err());
    let present = PullEvidence::capture(&repository, None, false).expect("regular destination");
    present.revalidate().expect("unchanged destination");
    std::fs::write(&path, b"changed fetch\n").expect("change destination");
    assert!(present.revalidate().is_err());
    std::fs::remove_file(&path).expect("remove destination");
    let outside = root.path().join("sentinel");
    std::fs::write(&outside, b"preserved\n").expect("sentinel");
    std::os::unix::fs::symlink(&outside, &path).expect("redirect destination");
    assert!(PullEvidence::capture(&repository, None, false).is_err());
    assert!(absent.revalidate().is_err());
    assert_eq!(
        std::fs::read(outside).expect("sentinel retained"),
        b"preserved\n"
    );
}

#[cfg(test)]
#[tokio::test]
async fn real_pull_current_branch_fetch_reconciles_index_and_worktree() {
    for scenario in ["clean", "disjoint-edit", "conflicting-edit"] {
        let root = tempfile::tempdir().expect("fixture");
        let source = root.path().join("source");
        let native = root.path().join("native");
        let split = root.path().join("split");
        std::fs::create_dir(&source).expect("source");
        let run = |cwd: &std::path::Path, args: &[&str]| {
            std::process::Command::new("git")
                .current_dir(cwd)
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_GLOBAL", root.path().join("absent-config"))
                .env("GIT_TERMINAL_PROMPT", "0")
                .env("GIT_EDITOR", "true")
                .args([
                    "-c",
                    "user.name=Fixture",
                    "-c",
                    "user.email=fixture@example.invalid",
                    "-c",
                    "commit.gpgSign=false",
                    "-c",
                    "maintenance.auto=false",
                ])
                .args(args)
                .output()
                .expect("fixture Git")
        };
        let git = |cwd: &std::path::Path, args: &[&str]| {
            let output = run(cwd, args);
            assert!(output.status.success(), "{scenario} {args:?}: {output:?}");
            String::from_utf8(output.stdout).expect("output")
        };
        git(&source, &["init", "-b", "Topic"]);
        std::fs::write(source.join("changed"), "base\n").expect("base");
        std::fs::write(source.join("preserved"), "base\n").expect("base");
        git(&source, &["add", "."]);
        git(&source, &["commit", "-m", "base"]);
        for local in [&native, &split] {
            git(
                root.path(),
                &[
                    "clone",
                    source.to_str().expect("source"),
                    local.to_str().expect("local"),
                ],
            );
            git(
                local,
                &[
                    "config",
                    "remote.origin.fetch",
                    "+refs/heads/Topic:refs/heads/Topic",
                ],
            );
            git(local, &["config", "pull.rebase", "false"]);
            if scenario != "clean" {
                let path = if scenario == "disjoint-edit" {
                    "preserved"
                } else {
                    "changed"
                };
                std::fs::write(local.join(path), "local edit\n").expect("dirty edit");
            }
        }
        let original = git(&split, &["rev-parse", "HEAD"]);
        std::fs::write(source.join("changed"), "remote edit\n").expect("remote");
        git(&source, &["add", "."]);
        git(&source, &["commit", "-m", "remote"]);
        let expected = git(&source, &["rev-parse", "HEAD"]);
        let retained = BeforeFetch::capture(
            b"pull.rebase\nfalse\0",
            Some(b"refs/heads/Topic\n"),
            Some(original.as_bytes()),
            None,
            40,
        )
        .expect("before fetch");
        let native_result = run(&native, &["pull", "origin"]);
        git(&split, &["fetch", "--update-head-ok", "origin"]);
        assert_eq!(git(&split, &["rev-parse", "HEAD"]), expected);
        assert_eq!(
            std::fs::read(split.join("changed")).expect("pre-reconciliation"),
            if scenario == "conflicting-edit" {
                b"local edit\n".as_slice()
            } else {
                b"base\n".as_slice()
            }
        );
        let fetched = std::fs::read(split.join(".git/FETCH_HEAD")).expect("FETCH_HEAD");
        // Native pull reconciles a moved checked-out HEAD before rejecting
        // missing merge candidates. Candidate parsing must not skip this step.
        let without_candidates = retained
            .integration(b"", 40, |args| {
                let output = run(&split, &args.iter().map(String::as_str).collect::<Vec<_>>());
                async move { Ok(output) }
            })
            .await
            .expect("reconciliation before candidate validation");
        assert!(matches!(without_candidates, Integration::Reconcile { .. }));
        let integration = retained
            .integration(&fetched, 40, |args| {
                let output = run(&split, &args.iter().map(String::as_str).collect::<Vec<_>>());
                async move { Ok(output) }
            })
            .await
            .expect("selected integration");
        let Integration::Reconcile {
            original: old,
            current,
        } = integration
        else {
            panic!("missing explicit reconciliation");
        };
        assert_eq!(old, original.trim());
        assert_eq!(current, expected.trim());
        let reconciled = run(&split, &["read-tree", "-u", "-m", &old, &current]);
        let split_success = if reconciled.status.success() {
            let next = retained
                .integration_stage(&fetched, 40, true, |args| {
                    let output = run(&split, &args.iter().map(String::as_str).collect::<Vec<_>>());
                    async move { Ok(output) }
                })
                .await
                .expect("integration after reconciliation");
            let Integration::Command(args) = next else {
                panic!("expected merge");
            };
            run(&split, &args.iter().map(String::as_str).collect::<Vec<_>>())
                .status
                .success()
        } else {
            false
        };
        assert_eq!(split_success, native_result.status.success(), "{scenario}");
        assert_eq!(split_success, scenario != "conflicting-edit");
        assert_eq!(git(&native, &["rev-parse", "HEAD"]), expected);
        for path in ["changed", "preserved"] {
            assert_eq!(
                std::fs::read(native.join(path)).expect("native bytes"),
                std::fs::read(split.join(path)).expect("split bytes"),
                "{scenario} {path}"
            );
        }
        for args in [
            vec!["ls-files", "--stage"],
            vec!["status", "--porcelain=v1"],
            vec!["diff", "--binary"],
            vec!["diff", "--cached", "--binary"],
        ] {
            assert_eq!(
                git(&native, &args),
                git(&split, &args),
                "{scenario} {args:?}"
            );
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn real_pull_unborn_preserves_local_index_and_worktree() {
    for scenario in ["clean", "staged-disjoint", "untracked-conflict"] {
        let root = tempfile::tempdir().expect("fixture");
        let source = root.path().join("source");
        let native = root.path().join("native");
        let split = root.path().join("split");
        let run = |cwd: &std::path::Path, args: &[&str]| {
            std::process::Command::new("git")
                .current_dir(cwd)
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_GLOBAL", root.path().join("absent-config"))
                .env("GIT_TERMINAL_PROMPT", "0")
                .args([
                    "-c",
                    "user.name=Fixture",
                    "-c",
                    "user.email=fixture@example.invalid",
                    "-c",
                    "commit.gpgSign=false",
                    "-c",
                    "maintenance.auto=false",
                ])
                .args(args)
                .output()
                .expect("fixture Git")
        };
        let git = |cwd: &std::path::Path, args: &[&str]| {
            let output = run(cwd, args);
            assert!(output.status.success(), "{scenario} {args:?}: {output:?}");
            String::from_utf8(output.stdout).expect("output")
        };
        std::fs::create_dir(&source).expect("source");
        git(&source, &["init", "-b", "Topic"]);
        std::fs::write(source.join("remote"), "remote content\n").expect("remote file");
        git(&source, &["add", "."]);
        git(&source, &["commit", "-m", "initial"]);
        for local in [&native, &split] {
            std::fs::create_dir(local).expect("local");
            git(local, &["init", "-b", "Topic"]);
            git(
                local,
                &["remote", "add", "origin", source.to_str().expect("source")],
            );
            git(local, &["config", "pull.rebase", "false"]);
            match scenario {
                "staged-disjoint" => {
                    std::fs::write(local.join("local"), "staged content\n").expect("local file");
                    git(local, &["add", "local"]);
                }
                "untracked-conflict" => {
                    std::fs::write(local.join("remote"), "local content\n").expect("conflict");
                }
                _ => {}
            }
        }
        let retained = BeforeFetch::capture(
            b"pull.rebase\nfalse\0",
            Some(b"refs/heads/Topic\n"),
            None,
            None,
            40,
        )
        .expect("unborn state");
        let native_result = run(&native, &["pull", "origin", "Topic"]);
        git(&split, &["fetch", "--update-head-ok", "origin", "Topic"]);
        let fetched = std::fs::read(split.join(".git/FETCH_HEAD")).expect("FETCH_HEAD");
        let selected = retained
            .integration(&fetched, 40, |args| {
                let output = run(&split, &args.iter().map(String::as_str).collect::<Vec<_>>());
                async move { Ok(output) }
            })
            .await
            .expect("unborn selection");
        let Integration::Unborn(target) = selected else {
            panic!("expected unborn plan")
        };
        let empty = git(&split, &["hash-object", "-t", "tree", "--stdin"]);
        let checkout = run(&split, &["read-tree", "-u", "-m", empty.trim(), &target]);
        let split_success = checkout.status.success()
            && run(
                &split,
                &[
                    "update-ref",
                    "-m",
                    "initial pull",
                    "HEAD",
                    &target,
                    "0000000000000000000000000000000000000000",
                ],
            )
            .status
            .success();
        assert_eq!(split_success, native_result.status.success(), "{scenario}");
        assert_eq!(split_success, scenario != "untracked-conflict");
        for args in [
            vec!["rev-parse", "--verify", "--quiet", "HEAD"],
            vec!["ls-files", "--stage"],
            vec!["status", "--porcelain=v1"],
            vec!["diff", "--binary"],
            vec!["diff", "--cached", "--binary"],
        ] {
            let expected = run(&native, &args);
            let actual = run(&split, &args);
            assert_eq!(
                expected.status.code(),
                actual.status.code(),
                "{scenario} {args:?}"
            );
            assert_eq!(expected.stdout, actual.stdout, "{scenario} {args:?}");
        }
        for path in ["remote", "local"] {
            assert_eq!(
                std::fs::read(native.join(path)).ok(),
                std::fs::read(split.join(path)).ok(),
                "{scenario} {path}"
            );
        }
    }
}
