//! Bounded directory and ignore evidence for commit's status diagnostics.

use std::io::Read;
use std::path::{Path, PathBuf};

use super::{GitToolError, snapshot};

#[derive(Debug, PartialEq, Eq)]
pub(super) struct StatusState {
    identity: snapshot::PathIdentity,
    directories: Vec<(PathBuf, snapshot::PathIdentity)>,
    entries: Vec<(PathBuf, std::fs::FileType)>,
    ignores: Vec<(PathBuf, snapshot::Entry)>,
}

impl StatusState {
    /// Requires admitted worktree Read authority. Checks directory resolution
    /// before and after reads to reduce TOCTOU exposure, without OS confinement.
    /// Ordinary untracked bodies are excluded; gitlinks have separate evidence.
    pub(super) fn capture(root: &Path, gitlinks: &[PathBuf]) -> Result<Self, GitToolError> {
        Self::capture_rules(root, gitlinks, ".gitignore", false)
    }

    /// Pack delta selection can consult attributes for historical paths that
    /// are no longer tracked. Capture directory rules without ordinary bodies.
    pub(super) fn capture_pack_attributes(root: &Path) -> Result<Self, GitToolError> {
        Self::capture_rules(root, &[], ".gitattributes", true)
    }

    fn capture_rules(
        root: &Path,
        gitlinks: &[PathBuf],
        rule: &str,
        reject_links: bool,
    ) -> Result<Self, GitToolError> {
        let invalid = |_| GitToolError::InvalidInput("Git status evidence unavailable".into());
        let identity = snapshot::PathIdentity::capture(root).map_err(invalid)?;
        let resolved_root = root.canonicalize().map_err(invalid)?;
        let mut pending = vec![(
            root.to_owned(),
            0usize,
            directory_identity(root, &resolved_root)?,
        )];
        let mut directories = Vec::new();
        let mut entries = Vec::new();
        let mut ignores = Vec::new();
        let mut budget = snapshot::Budget {
            entries: 65_536,
            bytes: 4 * 1024 * 1024,
        };
        while let Some((directory, depth, expected)) = pending.pop() {
            if depth > 128 {
                return Err(GitToolError::InvalidInput(
                    "Git status directory limit exceeded".into(),
                ));
            }
            let resolved = resolved_root.join(directory.strip_prefix(root).map_err(|_| {
                GitToolError::InvalidInput("Git status directory escaped worktree".into())
            })?);
            if directory_identity(&directory, &resolved)? != expected {
                return Err(GitToolError::InvalidInput(
                    "Git status directory identity changed".into(),
                ));
            }
            let ignore = directory.join(rule);
            let entry = capture_ignore(&ignore, &mut budget).map_err(invalid)?;
            ignores.push((ignore, entry));
            for child in std::fs::read_dir(&directory).map_err(invalid)? {
                let child = child.map_err(invalid)?;
                let path = child.path();
                if child
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.eq_ignore_ascii_case(".git"))
                {
                    if directory != root {
                        return Err(GitToolError::InvalidInput("Untracked nested repository status requires separate discovery; use a separately authorized invocation".into()));
                    }
                    continue;
                }
                budget.entries = budget
                    .entries
                    .checked_sub(1)
                    .ok_or_else(|| invalid(std::io::Error::other("entry limit")))?;
                budget.bytes = budget
                    .bytes
                    .checked_sub(child.file_name().len())
                    .ok_or_else(|| invalid(std::io::Error::other("name limit")))?;
                let kind = child.file_type().map_err(invalid)?;
                if reject_links && kind.is_symlink() {
                    return Err(GitToolError::InvalidInput(
                        "Push attribute traversal through worktree symlinks requires separate authorization".into(),
                    ));
                }
                if kind.is_dir() && !gitlinks.contains(&path) {
                    let child_identity =
                        directory_identity(&path, &resolved.join(child.file_name()))?;
                    pending.push((path.clone(), depth + 1, child_identity));
                }
                entries.push((path, kind));
            }
            if directory_identity(&directory, &resolved)? != expected {
                return Err(GitToolError::InvalidInput(
                    "Git status directory identity changed".into(),
                ));
            }
            directories.push((directory, expected));
        }
        directories.sort_by(|left, right| left.0.cmp(&right.0));
        for (directory, expected) in &directories {
            let relative = directory.strip_prefix(root).map_err(|_| {
                GitToolError::InvalidInput("Git status directory escaped worktree".into())
            })?;
            if &directory_identity(directory, &resolved_root.join(relative))? != expected {
                return Err(GitToolError::InvalidInput(
                    "Git status directory identity changed".into(),
                ));
            }
        }
        entries.sort_by(|left, right| left.0.cmp(&right.0));
        ignores.sort_by(|left, right| left.0.cmp(&right.0));
        if snapshot::PathIdentity::capture(root).map_err(invalid)? != identity {
            return Err(GitToolError::InvalidInput(
                "Git status worktree identity changed".into(),
            ));
        }
        Ok(Self {
            identity,
            directories,
            entries,
            ignores,
        })
    }
}

fn directory_identity(
    path: &Path,
    resolved: &Path,
) -> Result<snapshot::PathIdentity, GitToolError> {
    let invalid =
        |_| GitToolError::InvalidInput("Git status directory identity unavailable".into());
    let metadata = std::fs::symlink_metadata(path).map_err(invalid)?;
    if !metadata.is_dir() || path.canonicalize().map_err(invalid)? != resolved {
        return Err(GitToolError::InvalidInput(
            "Git status directory resolution changed".into(),
        ));
    }
    snapshot::PathIdentity::capture(path).map_err(invalid)
}

// Deliberately non-recursive, even if the entry changes type during capture.
fn capture_ignore(path: &Path, budget: &mut snapshot::Budget) -> std::io::Result<snapshot::Entry> {
    let invalid = || std::io::Error::other("Git status ignore rules unavailable or over budget");
    budget.entries = budget.entries.checked_sub(1).ok_or_else(invalid)?;
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(snapshot::Entry::Missing);
        }
        Err(error) => return Err(error),
    };
    if !metadata.is_file() || metadata.len() > budget.bytes as u64 {
        return Err(invalid());
    }
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options.open(path)?;
    let opened = file.metadata()?;
    if !opened.is_file() || opened.len() > budget.bytes as u64 {
        return Err(invalid());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if (metadata.dev(), metadata.ino()) != (opened.dev(), opened.ino()) {
            return Err(invalid());
        }
    }
    let mut bytes = Vec::new();
    file.take((budget.bytes as u64).saturating_add(1))
        .read_to_end(&mut bytes)?;
    budget.bytes = budget.bytes.checked_sub(bytes.len()).ok_or_else(invalid)?;
    Ok(snapshot::Entry::File(opened.permissions(), bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_attributes_cover_untracked_historical_directories_without_file_bodies() {
        let root = tempfile::tempdir().expect("fixture");
        let nested = root.path().join("formerly-tracked/deep");
        std::fs::create_dir_all(&nested).expect("directories");
        std::fs::write(nested.join("ordinary"), "before").expect("body");
        let before = StatusState::capture_pack_attributes(root.path()).expect("capture");
        std::fs::write(nested.join("ordinary"), "after").expect("body");
        assert_eq!(
            before,
            StatusState::capture_pack_attributes(root.path()).expect("capture")
        );
        std::fs::write(nested.join(".gitattributes"), "* -delta\n").expect("attributes");
        assert_ne!(
            before,
            StatusState::capture_pack_attributes(root.path()).expect("capture")
        );
    }

    #[cfg(unix)]
    #[test]
    fn pack_attribute_traversal_refuses_external_directory_links() {
        let root = tempfile::tempdir().expect("fixture");
        let external = tempfile::tempdir().expect("external");
        std::fs::write(external.path().join(".gitattributes"), "private").expect("rules");
        std::os::unix::fs::symlink(external.path(), root.path().join("old-directory"))
            .expect("link");
        assert!(StatusState::capture_pack_attributes(root.path()).is_err());
    }

    #[test]
    fn ignore_directory_is_rejected_without_consuming_body_budget() {
        let root = tempfile::tempdir().expect("fixture");
        let ignore = root.path().join(".gitignore");
        std::fs::create_dir(&ignore).expect("directory");
        std::fs::write(ignore.join("private"), "must not read").expect("body");
        let mut budget = snapshot::Budget {
            entries: 10,
            bytes: 100,
        };
        assert!(capture_ignore(&ignore, &mut budget).is_err());
        assert_eq!(budget.bytes, 100);
        assert_eq!(budget.entries, 9);
    }

    #[cfg(unix)]
    #[test]
    fn ignore_links_and_special_entries_are_rejected() {
        let root = tempfile::tempdir().expect("fixture");
        let target = root.path().join("rules");
        let ignore = root.path().join(".gitignore");
        std::fs::write(&target, "private").expect("rules");
        std::os::unix::fs::symlink(&target, &ignore).expect("link");
        assert!(StatusState::capture(root.path(), &[]).is_err());
        std::fs::remove_file(&ignore).expect("remove link");
        let _socket = std::os::unix::net::UnixListener::bind(&ignore).expect("socket");
        assert!(StatusState::capture(root.path(), &[]).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn queued_directory_replacement_and_links_in_ancestors_are_rejected() {
        let root = tempfile::tempdir().expect("fixture");
        let path = root.path().join("child");
        std::fs::create_dir(&path).expect("child");
        let resolved = path.canonicalize().expect("canonical");
        let before = directory_identity(&path, &resolved).expect("identity");
        std::fs::rename(&path, root.path().join("old")).expect("retain old inode");
        std::fs::create_dir(&path).expect("replacement");
        assert_ne!(
            before,
            directory_identity(&path, &resolved).expect("identity")
        );
        std::fs::remove_dir(&path).expect("remove replacement");
        std::os::unix::fs::symlink(root.path().join("old"), &path).expect("link");
        assert!(directory_identity(&path, &resolved).is_err());
        std::fs::create_dir(root.path().join("old/nested")).expect("nested");
        assert!(directory_identity(&path.join("nested"), &resolved.join("nested")).is_err());
    }

    #[test]
    fn status_binds_untracked_names_and_ignore_rules_without_file_bodies() {
        let root = tempfile::tempdir().expect("fixture");
        std::fs::create_dir(root.path().join("untracked")).expect("directory");
        let file = root.path().join("untracked/file");
        std::fs::write(&file, "first").expect("file");
        let before = StatusState::capture(root.path(), &[]).expect("capture");
        std::fs::write(&file, "different body").expect("edit");
        assert_eq!(
            before,
            StatusState::capture(root.path(), &[]).expect("same names")
        );
        std::fs::write(root.path().join("untracked/.gitignore"), "file\n").expect("ignore");
        assert_ne!(
            before,
            StatusState::capture(root.path(), &[]).expect("ignore changed")
        );
        std::fs::create_dir(root.path().join("untracked/.git")).expect("nested");
        assert!(StatusState::capture(root.path(), &[]).is_err());
        assert!(StatusState::capture(root.path(), &[root.path().join("untracked")]).is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn status_does_not_follow_untracked_links_or_open_special_file_bodies() {
        let root = tempfile::tempdir().expect("fixture");
        let outside = tempfile::tempdir().expect("outside");
        std::os::unix::fs::symlink(outside.path(), root.path().join("link")).expect("link");
        let _socket =
            std::os::unix::net::UnixListener::bind(root.path().join("socket")).expect("socket");
        let before = StatusState::capture(root.path(), &[]).expect("capture");
        std::fs::write(outside.path().join(".gitignore"), "changed\n").expect("external rule");
        assert_eq!(
            before,
            StatusState::capture(root.path(), &[]).expect("external not read")
        );
    }
}
