//! Local repository evidence retained by a Git effect plan.
//! This does not capture effective global/environment configuration or grant authority.

use std::path::{Path, PathBuf};

use super::{GitToolError, snapshot};

/// Directory evidence discovered after the child gitdir Read admission. This
/// reads only its commondir pointer, never newly discovered shared contents.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct ChildLocation {
    pub git_dir: PathBuf,
    pub common_dir: PathBuf,
    git_identity: snapshot::PathIdentity,
    common_identity: snapshot::PathIdentity,
    pointer: snapshot::Entry,
}

impl ChildLocation {
    pub(super) fn capture(git_dir: &Path) -> Result<Self, GitToolError> {
        let invalid =
            |_| GitToolError::InvalidInput("Submodule directory identity unavailable".into());
        let git_identity = snapshot::PathIdentity::capture(git_dir).map_err(invalid)?;
        let canonical_git = git_dir.canonicalize().map_err(invalid)?;
        if !canonical_git.is_dir() {
            return Err(GitToolError::InvalidInput(
                "Submodule gitdir is not a directory".into(),
            ));
        }
        let pointer_path = canonical_git.join("commondir");
        match std::fs::symlink_metadata(&pointer_path) {
            Ok(metadata) if !metadata.is_file() || metadata.file_type().is_symlink() => {
                return Err(GitToolError::InvalidInput(
                    "Submodule commondir must be a regular file".into(),
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(invalid(error)),
        }
        let pointer = snapshot::Entry::capture(
            &pointer_path,
            &mut snapshot::Budget {
                entries: 1,
                bytes: 16 * 1024,
            },
        )
        .map_err(invalid)?;
        let common_dir = match &pointer {
            snapshot::Entry::Missing => canonical_git.clone(),
            snapshot::Entry::File(_, bytes) => {
                let text = std::str::from_utf8(bytes).map_err(|_| {
                    GitToolError::InvalidInput("Submodule commondir is not UTF-8".into())
                })?;
                let text = text.trim_end_matches(['\r', '\n']);
                if text.is_empty() || bytes.contains(&0) {
                    return Err(GitToolError::InvalidInput(
                        "Invalid submodule commondir path".into(),
                    ));
                }
                canonical_git.join(text).canonicalize().map_err(invalid)?
            }
            _ => {
                return Err(GitToolError::InvalidInput(
                    "Submodule commondir changed during inspection".into(),
                ));
            }
        };
        if !common_dir.is_dir() {
            return Err(GitToolError::InvalidInput(
                "Submodule common directory is unavailable".into(),
            ));
        }
        let common_identity = snapshot::PathIdentity::capture(&common_dir).map_err(invalid)?;
        if snapshot::PathIdentity::capture(git_dir).map_err(invalid)? != git_identity {
            return Err(GitToolError::InvalidInput(
                "Submodule directory identity changed during inspection".into(),
            ));
        }
        Ok(Self {
            git_dir: canonical_git,
            common_dir,
            git_identity,
            common_identity,
            pointer,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct RepositoryState {
    pub workdir: PathBuf,
    pub git_dir: PathBuf,
    pub common_dir: PathBuf,
    pub index: PathBuf,
    identities: Vec<snapshot::PathIdentity>,
    entries: Vec<(PathBuf, snapshot::Entry)>,
    object_layout: Vec<(PathBuf, bool)>,
    references: Vec<(PathBuf, snapshot::Entry)>,
}

impl RepositoryState {
    #[cfg(test)]
    pub(super) fn changed_fields(&self, current: &Self) -> Vec<&'static str> {
        let mut changed = Vec::new();
        macro_rules! compare {
            ($($field:ident),+ $(,)?) => {$(
                if self.$field != current.$field {
                    changed.push(stringify!($field));
                }
            )+};
        }
        compare!(
            workdir,
            git_dir,
            common_dir,
            index,
            identities,
            entries,
            object_layout,
            references
        );
        changed
    }

    /// Check retained location pointers before rediscovering a repository.
    /// A changed pointer must not cause reads from a newly selected root under
    /// the old root's admission. The caller also binds directory identities.
    pub(super) fn revalidate_location(&self) -> Result<(), GitToolError> {
        for (path, expected) in [&self.workdir, &self.git_dir, &self.common_dir]
            .into_iter()
            .zip(&self.identities)
        {
            if snapshot::PathIdentity::capture(path).ok().as_ref() != Some(expected) {
                return Err(GitToolError::InvalidInput(
                    "Git repository root identity changed".into(),
                ));
            }
        }
        let marker = self.workdir.join(".git");
        let pointers = [
            marker,
            self.git_dir.join("commondir"),
            self.git_dir.join("gitdir"),
        ];
        let mut budget = snapshot::Budget {
            entries: pointers.len(),
            bytes: 48 * 1024,
        };
        for (path, expected) in &self.entries {
            if pointers.contains(path) {
                let current =
                    snapshot::Entry::capture_regular(path, &mut budget).map_err(|_| {
                        GitToolError::InvalidInput(
                            "Git repository location pointer unavailable".into(),
                        )
                    })?;
                if &current != expected {
                    return Err(GitToolError::InvalidInput(
                        "Git repository location pointer changed".into(),
                    ));
                }
            }
        }
        Ok(())
    }

    pub(super) fn capture(workspace: &Path) -> Result<Self, GitToolError> {
        std::panic::catch_unwind(|| Self::capture_inner(workspace)).unwrap_or_else(|_| {
            Err(GitToolError::Git(
                "repository inspection failed; no Git operation prepared".into(),
            ))
        })
    }

    fn capture_inner(workspace: &Path) -> Result<Self, GitToolError> {
        let invalid = |error: std::io::Error| GitToolError::InvalidInput(error.to_string());
        let repo = super::discover_repo(workspace)?;
        let workdir = repo
            .workdir()
            .ok_or_else(|| GitToolError::InvalidInput("Git mutation requires a worktree".into()))?
            .canonicalize()
            .map_err(invalid)?;
        let git_dir = repo.git_dir().canonicalize().map_err(invalid)?;
        let common_dir = repo.common_dir().canonicalize().map_err(invalid)?;
        let identities = [&workdir, &git_dir, &common_dir]
            .into_iter()
            .map(|path| snapshot::PathIdentity::capture(path).map_err(invalid))
            .collect::<Result<Vec<_>, _>>()?;
        let index_path = repo.index_path();
        let object_layout = capture_object_layout(&common_dir)?;
        // HEAD's symbolic name can remain unchanged while its loose target
        // moves. Bind both shared and per-worktree reference namespaces.
        let mut reference_roots = vec![common_dir.join("refs")];
        if git_dir != common_dir {
            reference_roots.push(git_dir.join("refs"));
        }
        let mut reference_budget = snapshot::Budget {
            entries: 65_536,
            bytes: 16 * 1024 * 1024,
        };
        let references = reference_roots
            .into_iter()
            .map(|path| {
                let entry =
                    snapshot::Entry::capture(&path, &mut reference_budget).map_err(invalid)?;
                if !reference_entries_are_regular(&entry) {
                    return Err(GitToolError::InvalidInput(
                        "Git reference links require a separately authorized invocation".into(),
                    ));
                }
                Ok((path, entry))
            })
            .collect::<Result<Vec<_>, GitToolError>>()?;
        for entry in std::fs::read_dir(&git_dir).map_err(invalid)? {
            if entry
                .map_err(invalid)?
                .file_name()
                .as_encoded_bytes()
                .starts_with(b"sharedindex.")
            {
                return Err(GitToolError::InvalidInput(
                    "Git split-index layouts require a separately authorized invocation".into(),
                ));
            }
        }
        let index = index_path
            .parent()
            .ok_or_else(|| GitToolError::InvalidInput("index parent is unavailable".into()))?
            .canonicalize()
            .map_err(invalid)?
            .join(index_path.file_name().ok_or_else(|| {
                GitToolError::InvalidInput("index filename is unavailable".into())
            })?);
        let mut paths = vec![
            index.clone(),
            git_dir.join("HEAD"),
            git_dir.join("commondir"),
            git_dir.join("gitdir"),
            git_dir.join("config.worktree"),
            common_dir.join("config"),
            common_dir.join("packed-refs"),
            common_dir.join("objects/info/alternates"),
        ];
        // Linked worktree discovery uses this file. The main worktree's .git
        // directory contains objects and is deliberately not recursively read.
        if workdir.join(".git") != git_dir {
            paths.push(workdir.join(".git"));
        }
        paths.sort();
        paths.dedup();
        let mut budget = snapshot::Budget {
            entries: paths.len(),
            bytes: 16 * 1024 * 1024,
        };
        let entries: Vec<(PathBuf, snapshot::Entry)> = paths
            .into_iter()
            .map(|path| {
                let entry = snapshot::Entry::capture(&path, &mut budget).map_err(invalid)?;
                // Git follows a metadata symlink. Recording only its link text
                // would fail to bind the bytes Git actually consumes.
                if !matches!(entry, snapshot::Entry::File(..) | snapshot::Entry::Missing) {
                    return Err(GitToolError::InvalidInput(
                        "Git metadata must be regular files; request a separately authorized Git invocation for this layout".into(),
                    ));
                }
                Ok((path, entry))
            })
            .collect::<Result<_, _>>()?;
        if entries.iter().any(|(path, entry)| {
            path == &common_dir.join("objects/info/alternates")
                && matches!(entry, snapshot::Entry::File(_, bytes) if !bytes.is_empty())
        }) {
            return Err(GitToolError::InvalidInput(
                "Git alternate object stores require a separately authorized invocation".into(),
            ));
        }
        Ok(Self {
            workdir,
            git_dir,
            common_dir,
            index,
            identities,
            entries,
            object_layout,
            references,
        })
    }
}

fn reference_entries_are_regular(entry: &snapshot::Entry) -> bool {
    match entry {
        snapshot::Entry::Link(_) => false,
        snapshot::Entry::Directory(children) => children
            .iter()
            .all(|(_, child)| reference_entries_are_regular(child)),
        snapshot::Entry::File(..) | snapshot::Entry::Missing => true,
    }
}

/// Commit-specific state; ordinary staging does not consume this evidence.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct CommitState(Vec<(PathBuf, snapshot::Entry)>);

impl CommitState {
    pub(super) fn rerere_cache_exists(&self, repository: &RepositoryState) -> bool {
        self.0.iter().any(|(path, entry)| {
            path == &repository.common_dir.join("rr-cache")
                && matches!(entry, snapshot::Entry::Directory(_))
        })
    }

    pub(super) fn has_rerere_records(&self, repository: &RepositoryState) -> bool {
        self.0.iter().any(|(path, entry)| {
            path == &repository.git_dir.join("MERGE_RR")
                && !matches!(entry, snapshot::Entry::Missing)
                && !matches!(entry, snapshot::Entry::File(_, bytes) if bytes.is_empty())
        })
    }

    pub(super) fn capture(repository: &RepositoryState) -> Result<Self, GitToolError> {
        let mut budget = snapshot::Budget {
            entries: 65_536,
            bytes: 16 * 1024 * 1024,
        };
        let mut paths = vec![
            repository.common_dir.join("logs"),
            repository.common_dir.join("rr-cache"),
        ];
        for name in [
            "COMMIT_EDITMSG",
            "MERGE_HEAD",
            "MERGE_MSG",
            "MERGE_MODE",
            "SQUASH_MSG",
            "CHERRY_PICK_HEAD",
            "REVERT_HEAD",
            "AUTO_MERGE",
            "MERGE_RR",
            "sequencer",
            "rebase-merge",
            "rebase-apply",
            "MERGE_AUTOSTASH",
            "logs",
        ] {
            paths.push(repository.git_dir.join(name));
        }
        paths.sort();
        paths.dedup();
        let mut entries = Vec::new();
        for path in paths {
            let directory = matches!(
                path.file_name().and_then(|name| name.to_str()),
                Some("logs" | "rr-cache" | "sequencer" | "rebase-merge" | "rebase-apply")
            );
            // Inspect the root type before recursive capture: scalar metadata
            // must not cause directory traversal, even when rerere is disabled.
            match std::fs::symlink_metadata(&path) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    return Err(GitToolError::InvalidInput(
                        "Git commit metadata links require a separately authorized invocation"
                            .into(),
                    ));
                }
                Ok(metadata)
                    if (directory && !metadata.is_dir()) || (!directory && !metadata.is_file()) =>
                {
                    return Err(GitToolError::InvalidInput(
                        "Git commit metadata has an unexpected entry type".into(),
                    ));
                }
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(_) => {
                    return Err(GitToolError::InvalidInput(
                        "Git commit metadata cannot be inspected".into(),
                    ));
                }
            }
            let entry = snapshot::Entry::capture(&path, &mut budget).map_err(|_| {
                GitToolError::InvalidInput("Git commit metadata cannot be captured".into())
            })?;
            if !reference_entries_are_regular(&entry) {
                return Err(GitToolError::InvalidInput(
                    "Git commit metadata links require a separately authorized invocation".into(),
                ));
            }
            if path == repository.git_dir.join("MERGE_AUTOSTASH")
                && entry != snapshot::Entry::Missing
            {
                return Err(GitToolError::InvalidInput("Git merge autostash can modify the worktree; use a separately authorized invocation".into()));
            }
            entries.push((path, entry));
        }
        Ok(Self(entries))
    }
}

// Bind the entire object namespace without reading object contents. Git may
// follow links at the root, shard, pack or individual object; none are safe to
// authorize as writes confined to common_dir. Bound enumeration and fail closed.
fn capture_object_layout(common_dir: &Path) -> Result<Vec<(PathBuf, bool)>, GitToolError> {
    let mut pending = vec![common_dir.join("objects")];
    let mut layout = Vec::new();
    while let Some(path) = pending.pop() {
        if layout.len() >= 65_536 {
            return Err(GitToolError::InvalidInput(
                "Git object layout inspection limit exceeded".into(),
            ));
        }
        let metadata = std::fs::symlink_metadata(&path)
            .map_err(|error| GitToolError::InvalidInput(error.to_string()))?;
        if metadata.file_type().is_symlink() || !(metadata.is_dir() || metadata.is_file()) {
            return Err(GitToolError::InvalidInput(
                "Git object links or special entries require a separately authorized invocation"
                    .into(),
            ));
        }
        if metadata.is_dir() {
            for entry in std::fs::read_dir(&path)
                .map_err(|error| GitToolError::InvalidInput(error.to_string()))?
            {
                if pending.len() + layout.len() >= 65_536 {
                    return Err(GitToolError::InvalidInput(
                        "Git object layout inspection limit exceeded".into(),
                    ));
                }
                pending.push(
                    entry
                        .map_err(|error| GitToolError::InvalidInput(error.to_string()))?
                        .path(),
                );
            }
        }
        layout.push((path, metadata.is_dir()));
    }
    layout.sort();
    Ok(layout)
}

#[cfg(test)]
mod tests {
    #[test]
    fn child_location_binds_commondir_without_reading_shared_contents() {
        let root = tempfile::tempdir().expect("fixture");
        let git_dir = root.path().join("gitdir");
        let shared = root.path().join("shared");
        let other = root.path().join("other");
        for path in [&git_dir, &shared, &other] {
            std::fs::create_dir(path).expect("directory");
        }
        std::fs::write(git_dir.join("commondir"), "../shared\n").expect("pointer");
        std::fs::write(shared.join("config"), "invalid configuration").expect("unread contents");
        let before = ChildLocation::capture(&git_dir).expect("location only");
        assert_eq!(
            before.common_dir,
            shared.canonicalize().expect("shared path")
        );
        std::fs::write(shared.join("config"), "different invalid configuration")
            .expect("change contents");
        assert_eq!(
            before,
            ChildLocation::capture(&git_dir).expect("unchanged location")
        );
        std::fs::write(git_dir.join("commondir"), "../other\n").expect("retarget");
        assert_ne!(
            before,
            ChildLocation::capture(&git_dir).expect("retargeted location")
        );
    }

    #[cfg(unix)]
    #[test]
    fn child_location_trims_all_pointer_line_endings_but_preserves_spaces() {
        let root = tempfile::tempdir().expect("fixture");
        let git_dir = root.path().join("gitdir");
        let shared = root.path().join("shared ");
        std::fs::create_dir(&git_dir).expect("gitdir");
        std::fs::create_dir(&shared).expect("shared");
        std::fs::write(git_dir.join("commondir"), "../shared \r\n\n\r\n").expect("pointer");
        let location = ChildLocation::capture(&git_dir).expect("location");
        assert_eq!(
            location.common_dir,
            shared.canonicalize().expect("shared path")
        );
        std::fs::write(git_dir.join("commondir"), "\r\n\n").expect("empty pointer");
        assert!(ChildLocation::capture(&git_dir).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn child_location_detects_directory_replacement_and_refuses_pointer_links() {
        let root = tempfile::tempdir().expect("fixture");
        let git_dir = root.path().join("gitdir");
        std::fs::create_dir(&git_dir).expect("directory");
        let before = ChildLocation::capture(&git_dir).expect("before");
        std::fs::rename(&git_dir, root.path().join("old")).expect("replace directory");
        std::fs::create_dir(&git_dir).expect("new directory");
        assert_ne!(
            before,
            ChildLocation::capture(&git_dir).expect("new identity")
        );
        std::os::unix::fs::symlink(root.path().join("outside"), git_dir.join("commondir"))
            .expect("pointer link");
        assert!(
            ChildLocation::capture(&git_dir)
                .expect_err("link refused")
                .to_string()
                .contains("regular file")
        );
    }

    #[test]
    fn commit_metadata_rejects_directory_as_scalar_and_file_as_cache() {
        let root = tempfile::tempdir().expect("repository");
        git(root.path(), &["init"]);
        let repository = RepositoryState::capture(root.path()).expect("repository");
        let records = repository.git_dir.join("MERGE_RR");
        std::fs::create_dir(&records).expect("invalid records directory");
        assert!(
            CommitState::capture(&repository)
                .expect_err("wrong type")
                .to_string()
                .contains("unexpected entry type")
        );
        std::fs::remove_dir(&records).expect("remove fixture");
        std::fs::write(repository.common_dir.join("rr-cache"), "invalid cache file")
            .expect("cache file");
        assert!(
            CommitState::capture(&repository)
                .expect_err("wrong cache type")
                .to_string()
                .contains("unexpected entry type")
        );
    }

    #[cfg(unix)]
    #[test]
    fn commit_metadata_refuses_rerere_symlinks() {
        let root = tempfile::tempdir().expect("repository");
        let outside = tempfile::tempdir().expect("outside");
        git(root.path(), &["init"]);
        let repository = RepositoryState::capture(root.path()).expect("repository");
        std::os::unix::fs::symlink(
            outside.path().join("missing"),
            repository.git_dir.join("MERGE_RR"),
        )
        .expect("link");
        assert!(
            CommitState::capture(&repository)
                .expect_err("link refused")
                .to_string()
                .contains("metadata links")
        );
    }

    #[test]
    fn commit_state_binds_merge_inputs_and_refuses_autostash() {
        let root = tempfile::tempdir().expect("repository");
        git(root.path(), &["init"]);
        let repository = RepositoryState::capture(root.path()).expect("repository state");
        let before = CommitState::capture(&repository).expect("before");
        std::fs::write(repository.git_dir.join("MERGE_MSG"), "merge message")
            .expect("merge message");
        assert_ne!(before, CommitState::capture(&repository).expect("after"));
        std::fs::write(repository.git_dir.join("MERGE_AUTOSTASH"), "object").expect("autostash");
        assert!(
            CommitState::capture(&repository)
                .expect_err("autostash")
                .to_string()
                .contains("autostash")
        );
    }

    #[test]
    fn packed_and_linked_worktree_references_invalidate_snapshot() {
        let root = tempfile::tempdir().expect("repository");
        let main = root.path().join("main");
        std::fs::create_dir(&main).expect("main");
        git(&main, &["init"]);
        for message in ["first", "second"] {
            git(
                &main,
                &[
                    "-c",
                    "user.name=Test",
                    "-c",
                    "user.email=test@example.invalid",
                    "-c",
                    "commit.gpgSign=false",
                    "commit",
                    "--allow-empty",
                    "-m",
                    message,
                ],
            );
        }
        git(&main, &["worktree", "add", "-b", "linked", "../linked"]);
        let linked = root.path().join("linked");
        git(&main, &["pack-refs", "--all", "--prune"]);
        let before = RepositoryState::capture(&linked).expect("packed state");
        git(&main, &["update-ref", "refs/heads/linked", "HEAD~1"]);
        git(&main, &["pack-refs", "--all", "--prune"]);
        let after = RepositoryState::capture(&linked).expect("moved packed state");
        assert_eq!(before.references, after.references, "loose refs unchanged");
        assert_eq!(before.object_layout, after.object_layout);
        assert_ne!(before, after, "shared packed ref movement must be bound");

        git(&linked, &["update-ref", "refs/worktree/private", "HEAD"]);
        let private = RepositoryState::capture(&linked).expect("private reference");
        assert_ne!(after.references, private.references);
        let main_before = RepositoryState::capture(&main).expect("main before");
        git(
            &linked,
            &["update-ref", "refs/worktree/private", "HEAD@{1}"],
        );
        assert_ne!(
            private,
            RepositoryState::capture(&linked).expect("private moved")
        );
        assert_eq!(
            main_before,
            RepositoryState::capture(&main).expect("main unchanged")
        );
    }

    #[test]
    fn loose_reference_changes_invalidate_snapshot_without_new_objects() {
        let root = tempfile::tempdir().expect("repository");
        git(root.path(), &["init"]);
        for message in ["first", "second"] {
            git(
                root.path(),
                &[
                    "-c",
                    "user.name=Test",
                    "-c",
                    "user.email=test@example.invalid",
                    "-c",
                    "commit.gpgSign=false",
                    "commit",
                    "--allow-empty",
                    "-m",
                    message,
                ],
            );
        }
        let before = RepositoryState::capture(root.path()).expect("before");
        git(root.path(), &["update-ref", "HEAD", "HEAD~1"]);
        let after = RepositoryState::capture(root.path()).expect("after");
        assert_eq!(
            before.entries, after.entries,
            "HEAD symbolic text and index stay unchanged"
        );
        assert_eq!(
            before.object_layout, after.object_layout,
            "no object created"
        );
        assert_ne!(
            before, after,
            "loose ref movement must invalidate admission evidence"
        );
    }

    #[cfg(unix)]
    #[test]
    fn reference_directory_symlink_is_refused() {
        let root = tempfile::tempdir().expect("repository");
        let outside = tempfile::tempdir().expect("outside");
        git(root.path(), &["init"]);
        std::fs::remove_dir(root.path().join(".git/refs/heads")).expect("empty heads");
        std::os::unix::fs::symlink(outside.path(), root.path().join(".git/refs/heads"))
            .expect("link");
        assert!(
            RepositoryState::capture(root.path())
                .expect_err("link refused")
                .to_string()
                .contains("reference links")
        );
    }
    use super::*;

    fn git(root: &Path, args: &[&str]) {
        let output = std::process::Command::new("git")
            .current_dir(root)
            // Fixtures compare object layouts immediately after commits. Wait
            // for maintenance so its temporary lock cannot race the snapshots.
            .args([
                "-c",
                "maintenance.autoDetach=false",
                "-c",
                "gc.autoDetach=false",
            ])
            .args(args)
            .output()
            .expect("fixture Git");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn snapshot_detects_index_head_and_local_config_changes() {
        let root = tempfile::tempdir().expect("fixture");
        git(root.path(), &["init"]);
        let initial = RepositoryState::capture(root.path()).expect("initial");
        assert_eq!(
            initial,
            RepositoryState::capture(root.path()).expect("unchanged")
        );
        std::fs::write(root.path().join("file"), "data").expect("file");
        git(root.path(), &["add", "file"]);
        let staged = RepositoryState::capture(root.path()).expect("staged");
        assert_ne!(initial, staged);
        git(root.path(), &["symbolic-ref", "HEAD", "refs/heads/other"]);
        let head = RepositoryState::capture(root.path()).expect("HEAD");
        assert_ne!(staged, head);
        git(root.path(), &["config", "core.autocrlf", "false"]);
        assert_ne!(head, RepositoryState::capture(root.path()).expect("config"));
    }

    #[test]
    fn alternate_and_split_index_layouts_are_refused() {
        let root = tempfile::tempdir().expect("fixture");
        git(root.path(), &["init"]);
        let alternate = root.path().join(".git/objects/info/alternates");
        std::fs::write(&alternate, "/external/objects\n").expect("alternate");
        assert!(
            RepositoryState::capture(root.path())
                .expect_err("alternate refused")
                .to_string()
                .contains("alternate object stores")
        );
        std::fs::remove_file(alternate).expect("remove fixture alternate");
        std::fs::write(root.path().join("file"), "data").expect("file");
        git(root.path(), &["add", "file"]);
        git(root.path(), &["update-index", "--split-index"]);
        assert!(
            RepositoryState::capture(root.path())
                .expect_err("split index refused")
                .to_string()
                .contains("split-index")
        );
    }

    #[cfg(unix)]
    #[test]
    fn object_root_shard_and_file_links_are_refused() {
        use std::os::unix::fs::symlink;
        let root = tempfile::tempdir().expect("fixture");
        let outside = tempfile::tempdir().expect("outside");
        git(root.path(), &["init"]);
        let objects = root.path().join(".git/objects");
        std::fs::rename(&objects, root.path().join("saved-objects")).expect("save objects");
        symlink(outside.path(), &objects).expect("root link");
        assert!(RepositoryState::capture(root.path()).is_err());
        std::fs::remove_file(&objects).expect("remove root link");
        std::fs::rename(root.path().join("saved-objects"), &objects).expect("restore objects");
        let shard = objects.join("aa");
        symlink(outside.path(), &shard).expect("shard link");
        assert!(RepositoryState::capture(root.path()).is_err());
        std::fs::remove_file(&shard).expect("remove shard link");
        std::fs::create_dir(&shard).expect("shard");
        symlink(outside.path().join("object"), shard.join("object")).expect("object link");
        assert!(RepositoryState::capture(root.path()).is_err());
        assert!(
            std::fs::read_dir(outside.path())
                .expect("outside")
                .next()
                .is_none()
        );
    }

    #[test]
    fn linked_worktree_snapshot_uses_its_own_index_and_shared_config() {
        let root = tempfile::tempdir().expect("fixture");
        let main = root.path().join("main");
        std::fs::create_dir(&main).expect("main");
        git(&main, &["init"]);
        git(
            &main,
            &[
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.invalid",
                "commit",
                "--allow-empty",
                "-m",
                "initial",
            ],
        );
        git(&main, &["worktree", "add", "-b", "linked", "../linked"]);
        let linked = root.path().join("linked");
        let state = RepositoryState::capture(&linked).expect("linked state");
        let main_state = RepositoryState::capture(&main).expect("main state");
        assert_ne!(state.index, main_state.index);
        assert_ne!(state.git_dir, main_state.git_dir);
        assert_eq!(state.common_dir, main_state.common_dir);
        assert_eq!(state.index, state.git_dir.join("index"));
        std::fs::write(linked.join("file"), "data").expect("file");
        git(&linked, &["add", "file"]);
        assert_ne!(state, RepositoryState::capture(&linked).expect("changed"));
        let main_after = RepositoryState::capture(&main).expect("main after linked staging");
        assert_eq!(
            main_state.entries, main_after.entries,
            "main index/config unchanged"
        );
        assert_ne!(
            main_state.object_layout, main_after.object_layout,
            "shared object addition invalidates both worktrees' evidence"
        );
    }

    #[cfg(unix)]
    #[test]
    fn metadata_symlink_is_refused_without_reading_its_target() {
        let root = tempfile::tempdir().expect("fixture");
        git(root.path(), &["init"]);
        std::fs::write(root.path().join("outside-index"), "not an index").expect("target");
        std::os::unix::fs::symlink("../outside-index", root.path().join(".git/index"))
            .expect("index symlink");
        let error = RepositoryState::capture(root.path()).expect_err("reject indirection");
        assert!(error.to_string().contains("metadata must be regular files"));
    }
}
