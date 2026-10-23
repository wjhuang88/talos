//! Bounded, entry-preserving source evidence for prepared Git mutations.
//! Rechecking this evidence reduces TOCTOU exposure; it is not OS confinement.

use std::io::Read;
use std::path::{Path, PathBuf};

#[cfg(test)]
#[path = "snapshot_regular_tests.rs"]
mod regular_tests;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Entry {
    Missing,
    File(std::fs::Permissions, Vec<u8>),
    Link(PathBuf),
    Directory(Vec<(std::ffi::OsString, Entry)>),
}

/// One non-recursive file snapshot with its leaf identity. Missing entries keep
/// Missing identity so later creation invalidates the evidence.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct RegularEvidence {
    pub entry: Entry,
    leaf: Option<LeafIdentity>,
}

#[derive(Debug, PartialEq, Eq)]
struct LeafIdentity {
    #[cfg(windows)]
    handle: same_file::Handle,
    #[cfg(unix)]
    device: u64,
    #[cfg(unix)]
    inode: u64,
}

impl LeafIdentity {
    fn matches_file(&self, file: &std::fs::File) -> std::io::Result<bool> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let metadata = file.metadata()?;
            Ok((self.device, self.inode) == (metadata.dev(), metadata.ino()))
        }
        #[cfg(windows)]
        {
            let handle =
                std::panic::catch_unwind(|| same_file::Handle::from_file(file.try_clone()?))
                    .map_err(|_| std::io::Error::other("Windows Git leaf comparison panicked"))??;
            Ok(self.handle == handle)
        }
        #[cfg(not(any(unix, windows)))]
        {
            let _ = file;
            Err(std::io::Error::other("Git leaf identity unsupported"))
        }
    }

    fn capture(path: &Path) -> std::io::Result<Option<Self>> {
        let metadata = match std::fs::symlink_metadata(path) {
            Ok(metadata) if metadata.is_file() => metadata,
            Ok(_) => return Err(std::io::Error::other("Git evidence leaf is not regular")),
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
                ) =>
            {
                return Ok(None);
            }
            Err(error) => return Err(error),
        };
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            Ok(Some(Self {
                device: metadata.dev(),
                inode: metadata.ino(),
            }))
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;
            if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                return Err(std::io::Error::other(
                    "Git evidence leaf is a reparse point",
                ));
            }
            let handle = std::panic::catch_unwind(|| same_file::Handle::from_path(path)).map_err(
                |_| std::io::Error::other("Windows Git leaf identity inspection panicked"),
            )??;
            Ok(Some(Self { handle }))
        }
        #[cfg(not(any(unix, windows)))]
        {
            let _ = metadata;
            Ok(Some(Self {}))
        }
    }
}

impl RegularEvidence {
    /// Compare retained regular-file evidence without allocating another file
    /// body. Identity and permissions must remain unchanged as well as bytes.
    pub(super) fn revalidate(&self, path: &Path) -> std::io::Result<()> {
        let invalid = || std::io::Error::other("Git regular evidence changed");
        if LeafIdentity::capture(path)? != self.leaf {
            return Err(invalid());
        }
        let Entry::File(permissions, expected) = &self.entry else {
            return if self.entry == Entry::Missing && self.leaf.is_none() {
                Ok(())
            } else {
                Err(invalid())
            };
        };
        let leaf = self.leaf.as_ref().ok_or_else(invalid)?;
        let mut file = open_regular(path)?;
        let metadata = file.metadata()?;
        if !metadata.is_file()
            || metadata.len() != expected.len() as u64
            || &metadata.permissions() != permissions
            || !leaf.matches_file(&file)?
        {
            return Err(invalid());
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;
            if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                return Err(invalid());
            }
        }
        let mut buffer = [0u8; 8192];
        for chunk in expected.chunks(buffer.len()) {
            file.read_exact(&mut buffer[..chunk.len()])?;
            if &buffer[..chunk.len()] != chunk {
                return Err(invalid());
            }
        }
        if file.read(&mut buffer[..1])? != 0 {
            return Err(invalid());
        }
        let after = file.metadata()?;
        if after.len() != expected.len() as u64
            || &after.permissions() != permissions
            || LeafIdentity::capture(path)? != self.leaf
        {
            return Err(invalid());
        }
        Ok(())
    }

    pub(super) fn capture(path: &Path, budget: &mut Budget) -> std::io::Result<Self> {
        let before = LeafIdentity::capture(path)?;
        let entry = Entry::capture_regular(path, budget)?;
        let after = LeafIdentity::capture(path)?;
        if before != after || before.is_some() != matches!(entry, Entry::File(..)) {
            return Err(std::io::Error::other(
                "Git evidence leaf changed during capture",
            ));
        }
        Ok(Self {
            entry,
            leaf: before,
        })
    }
}

fn open_regular(path: &Path) -> std::io::Result<std::fs::File> {
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT;
        options.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
    }
    options.open(path)
}

/// Shared budget across all selected roots, not a separate allowance per file.
pub(super) struct Budget {
    pub entries: usize,
    pub bytes: usize,
}

/// Path resolution evidence, without reading file contents. This reduces
/// substitution windows but does not confine subsequent path-based operations.
/// Windows retains live handles to keep volume/file-index comparisons valid;
/// same-file's 64-bit index comparison does not fully identify ReFS files.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct PathIdentity {
    resolved: PathBuf,
    #[cfg(unix)]
    ancestors: Vec<(u64, u64)>,
    #[cfg(windows)]
    ancestors: Vec<same_file::Handle>,
}

impl PathIdentity {
    /// Whether the retained canonical target is the original absolute spelling.
    /// This inspects evidence only; callers must revalidate before reading.
    pub(super) fn is_unredirected(&self, path: &Path) -> bool {
        self.resolved == path
    }

    /// Revalidate retained ancestors one at a time rather than constructing a
    /// second identity vector (and a second set of Windows handles).
    pub(super) fn revalidate(&self, path: &Path) -> std::io::Result<()> {
        let changed = || std::io::Error::other("Git path identity changed");
        if path.canonicalize()? != self.resolved {
            return Err(changed());
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            for (path, expected) in self.resolved.ancestors().zip(&self.ancestors) {
                let metadata = std::fs::metadata(path)?;
                if (metadata.dev(), metadata.ino()) != *expected {
                    return Err(changed());
                }
            }
        }
        #[cfg(windows)]
        {
            for (path, expected) in self.resolved.ancestors().zip(&self.ancestors) {
                if &windows_path_handle(path)? != expected {
                    return Err(changed());
                }
            }
        }
        Ok(())
    }

    pub(super) fn capture(path: &Path) -> std::io::Result<Self> {
        let resolved = path.canonicalize()?;
        #[cfg(unix)]
        let ancestors = {
            use std::os::unix::fs::MetadataExt;
            resolved
                .ancestors()
                .map(|path| {
                    let metadata = std::fs::metadata(path)?;
                    Ok((metadata.dev(), metadata.ino()))
                })
                .collect::<std::io::Result<Vec<_>>>()?
        };
        #[cfg(windows)]
        let ancestors = resolved
            .ancestors()
            .map(windows_path_handle)
            .collect::<std::io::Result<Vec<_>>>()?;
        Ok(Self {
            resolved,
            #[cfg(any(unix, windows))]
            ancestors,
        })
    }
}

#[cfg(windows)]
fn windows_path_handle(path: &Path) -> std::io::Result<same_file::Handle> {
    std::panic::catch_unwind(|| {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::{
            FILE_FLAG_BACKUP_SEMANTICS, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
        };
        // Metadata-only access permits ancestor renames while retaining identity.
        let file = std::fs::OpenOptions::new()
            .access_mode(0)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
            .open(path)?;
        same_file::Handle::from_file(file)
    })
    .map_err(|_| std::io::Error::other("Windows Git path identity inspection panicked"))?
}

impl Entry {
    /// Capture one configuration source without descending into directories or
    /// following leaf links. Ancestor/content races remain TOCTOU residuals.
    pub(super) fn capture_regular(path: &Path, budget: &mut Budget) -> std::io::Result<Self> {
        let invalid = || {
            std::io::Error::other(
                "Git configuration source changed, is not regular, or exceeds its limit",
            )
        };
        if cfg!(not(any(unix, windows))) {
            return Err(invalid());
        }
        budget.entries = budget.entries.checked_sub(1).ok_or_else(invalid)?;
        let before = match std::fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
                ) =>
            {
                return Ok(Self::Missing);
            }
            Err(error) => return Err(error),
        };
        if !before.is_file() || before.len() > budget.bytes as u64 {
            return Err(invalid());
        }
        let file = open_regular(path)?;
        let opened = file.metadata()?;
        if !opened.is_file() || opened.len() > budget.bytes as u64 {
            return Err(invalid());
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if (before.dev(), before.ino()) != (opened.dev(), opened.ino()) {
                return Err(invalid());
            }
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;
            if before.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
                || opened.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
            {
                return Err(invalid());
            }
        }
        let bytes = read_regular_body(file, &mut budget.bytes)?;
        let after = std::fs::symlink_metadata(path)?;
        if !after.is_file() {
            return Err(invalid());
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if (after.dev(), after.ino()) != (opened.dev(), opened.ino()) {
                return Err(invalid());
            }
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;
            if after.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                return Err(invalid());
            }
        }
        Ok(Self::File(opened.permissions(), bytes))
    }

    pub(super) fn capture(path: &Path, budget: &mut Budget) -> std::io::Result<Self> {
        Self::capture_at(path, budget, 0, false)
    }

    pub(super) fn capture_worktree(path: &Path, budget: &mut Budget) -> std::io::Result<Self> {
        Self::capture_at(path, budget, 0, true)
    }

    fn capture_at(
        path: &Path,
        budget: &mut Budget,
        depth: usize,
        worktree_root: bool,
    ) -> std::io::Result<Self> {
        let limit =
            || std::io::Error::other("Git source snapshot limit exceeded; no operation prepared");
        if depth > 128 || budget.entries == 0 {
            return Err(limit());
        }
        budget.entries -= 1;
        let metadata = match std::fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            // A target descendant behind a regular-file ancestor does not
            // exist yet (e.g. checkout replacing `file` with `file/nested`).
            // Capture the ancestor separately; never attempt to read through it.
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
                ) =>
            {
                return Ok(Self::Missing);
            }
            Err(error) => return Err(error),
        };
        if metadata.file_type().is_symlink() {
            let target = std::fs::read_link(path)?;
            budget.bytes = budget
                .bytes
                .checked_sub(target.as_os_str().len())
                .ok_or_else(limit)?;
            return Ok(Self::Link(target));
        }
        if metadata.is_file() {
            if metadata.len() > budget.bytes as u64 {
                return Err(limit());
            }
            let mut data = Vec::new();
            std::fs::File::open(path)?
                .take((budget.bytes as u64).saturating_add(1))
                .read_to_end(&mut data)?;
            budget.bytes = budget.bytes.checked_sub(data.len()).ok_or_else(limit)?;
            return Ok(Self::File(metadata.permissions(), data));
        }
        if metadata.is_dir() {
            let mut children = Vec::new();
            for child in std::fs::read_dir(path)? {
                let child = child?;
                let name = child.file_name();
                // Root metadata is not a staging source. A nested repository is
                // a gitlink with separate HEAD state, not an ordinary directory.
                if name
                    .to_str()
                    .is_some_and(|name| name.eq_ignore_ascii_case(".git"))
                {
                    if !worktree_root {
                        return Err(std::io::Error::other(
                            "nested Git repository cannot be prepared as a directory source",
                        ));
                    }
                    continue;
                }
                budget.bytes = budget.bytes.checked_sub(name.len()).ok_or_else(limit)?;
                children.push((
                    name,
                    Self::capture_at(&child.path(), budget, depth + 1, false)?,
                ));
            }
            children.sort_by(|left, right| left.0.cmp(&right.0));
            return Ok(Self::Directory(children));
        }
        Err(std::io::Error::other(
            "unsupported Git staging source entry type",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(any(unix, windows))]
    #[test]
    fn path_identity_retains_leaf_and_ancestor_identity() {
        let root = tempfile::tempdir().expect("fixture");
        let parent = root.path().join("parent");
        std::fs::create_dir(&parent).expect("parent");
        let leaf = parent.join("leaf");
        std::fs::write(&leaf, b"unchanged").expect("leaf");
        let identity = PathIdentity::capture(&leaf).expect("identity");
        assert_eq!(identity, PathIdentity::capture(&leaf).expect("unchanged"));
        identity.revalidate(&leaf).expect("retained identity");
        // Windows may refuse renaming a directory with an open descendant,
        // even when every identity handle shares DELETE. Move the same leaf
        // out first; retain all handles while replacing its empty parent.
        let staged_leaf = root.path().join("staged-leaf");
        std::fs::rename(&leaf, &staged_leaf).expect("stage original leaf");
        let saved = root.path().join("saved");
        std::fs::rename(&parent, &saved).expect("move parent");
        std::fs::create_dir(&parent).expect("replace parent");
        std::fs::rename(&staged_leaf, &leaf).expect("retain original leaf");
        assert_ne!(identity, PathIdentity::capture(&leaf).expect("new parent"));
        assert!(identity.revalidate(&leaf).is_err());
        std::fs::rename(&leaf, &staged_leaf).expect("stage leaf for restoration");
        std::fs::remove_dir(&parent).expect("remove replacement");
        std::fs::rename(&saved, &parent).expect("restore parent");
        std::fs::rename(&staged_leaf, &leaf).expect("restore original leaf");
        assert_eq!(identity, PathIdentity::capture(&leaf).expect("restored"));
        identity.revalidate(&leaf).expect("restored identity");
        std::fs::rename(&leaf, parent.join("original-leaf")).expect("move leaf");
        std::fs::write(&leaf, b"unchanged").expect("identical replacement bytes");
        assert_ne!(identity, PathIdentity::capture(&leaf).expect("new leaf"));
        assert!(identity.revalidate(&leaf).is_err());
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn regular_evidence_rejects_identical_byte_inode_replacement() {
        let root = tempfile::tempdir().expect("fixture");
        let path = root.path().join("config");
        std::fs::write(&path, "same bytes").expect("source");
        let evidence = RegularEvidence::capture(
            &path,
            &mut Budget {
                entries: 2,
                bytes: 100,
            },
        )
        .expect("evidence");
        std::fs::write(root.path().join("replacement"), "same bytes").expect("replacement");
        std::fs::rename(root.path().join("replacement"), &path).expect("replace");
        let current = RegularEvidence::capture(
            &path,
            &mut Budget {
                entries: 2,
                bytes: 100,
            },
        )
        .expect("current");
        assert_ne!(evidence, current);
        assert_eq!(evidence.entry, current.entry);
    }

    #[test]
    fn regular_capture_refuses_directory_without_charging_child_contents() {
        let root = tempfile::tempdir().expect("fixture");
        std::fs::write(root.path().join("child"), b"private bytes").expect("child");
        let mut budget = Budget {
            entries: 10,
            bytes: 100,
        };
        assert!(Entry::capture_regular(root.path(), &mut budget).is_err());
        assert_eq!(budget.entries, 9);
        assert_eq!(budget.bytes, 100);
        assert!(
            matches!(Entry::capture_regular(&root.path().join("child"), &mut budget).expect("regular"), Entry::File(_, bytes) if bytes == b"private bytes")
        );
        assert_eq!(budget.bytes, 87);
        assert!(
            Entry::capture_regular(
                &root.path().join("child"),
                &mut Budget {
                    entries: 1,
                    bytes: 3
                }
            )
            .is_err()
        );
    }

    #[cfg(unix)]
    #[test]
    fn regular_capture_refuses_leaf_symlink() {
        let root = tempfile::tempdir().expect("fixture");
        std::fs::write(root.path().join("target"), "private").expect("target");
        std::os::unix::fs::symlink("target", root.path().join("link")).expect("link");
        assert!(
            Entry::capture_regular(
                &root.path().join("link"),
                &mut Budget {
                    entries: 2,
                    bytes: 100
                }
            )
            .is_err()
        );
    }

    fn snapshot(path: &Path) -> Entry {
        Entry::capture(
            path,
            &mut Budget {
                entries: 64,
                bytes: 4096,
            },
        )
        .expect("snapshot")
    }

    #[test]
    fn selected_tree_detects_content_addition_and_deletion() {
        let root = tempfile::tempdir().expect("fixture");
        let file = root.path().join("file");
        std::fs::write(&file, "before").expect("file");
        let before = snapshot(root.path());
        assert_eq!(before, snapshot(root.path()));
        std::fs::write(&file, "after!").expect("same length edit");
        assert_ne!(before, snapshot(root.path()));
        std::fs::write(&file, "before").expect("restore");
        std::fs::write(root.path().join("new"), "new").expect("new file");
        assert_ne!(before, snapshot(root.path()));
        std::fs::remove_file(&file).expect("remove fixture");
        assert_eq!(snapshot(&file), Entry::Missing);
    }

    #[test]
    fn budget_is_shared_and_never_returns_partial_tree() {
        let root = tempfile::tempdir().expect("fixture");
        std::fs::write(root.path().join("a"), "123").expect("file");
        std::fs::write(root.path().join("b"), "456").expect("file");
        assert!(
            Entry::capture(
                root.path(),
                &mut Budget {
                    entries: 10,
                    bytes: 7
                }
            )
            .is_err()
        );
        assert!(
            Entry::capture(
                root.path(),
                &mut Budget {
                    entries: 2,
                    bytes: 100
                }
            )
            .is_err()
        );
    }

    #[test]
    fn nested_repository_is_refused_before_a_partial_snapshot() {
        let root = tempfile::tempdir().expect("fixture");
        let nested = root.path().join("nested");
        std::fs::create_dir(&nested).expect("nested directory");
        std::fs::create_dir(nested.join(".git")).expect("nested metadata");
        let error = Entry::capture(
            root.path(),
            &mut Budget {
                entries: 64,
                bytes: 4096,
            },
        )
        .expect_err("nested Git requires a distinct plan");
        assert!(error.to_string().contains("nested Git repository"));
        assert!(
            Entry::capture(
                &nested,
                &mut Budget {
                    entries: 64,
                    bytes: 4096
                },
            )
            .is_err(),
            "directly selected nested repository must also fail"
        );
    }

    #[test]
    fn only_explicit_worktree_root_can_exclude_its_own_metadata() {
        let root = tempfile::tempdir().expect("fixture");
        std::fs::create_dir(root.path().join(".git")).expect("metadata");
        assert!(
            Entry::capture(
                root.path(),
                &mut Budget {
                    entries: 64,
                    bytes: 4096
                }
            )
            .is_err()
        );
        assert!(
            Entry::capture_worktree(
                root.path(),
                &mut Budget {
                    entries: 64,
                    bytes: 4096
                }
            )
            .is_ok()
        );
    }

    #[cfg(unix)]
    #[test]
    fn final_symlink_records_link_not_target_content() {
        let root = tempfile::tempdir().expect("fixture");
        let target = root.path().join("target");
        let link = root.path().join("link");
        std::fs::write(&target, "before").expect("target");
        std::os::unix::fs::symlink(&target, &link).expect("symlink");
        let before = snapshot(&link);
        std::fs::write(&target, "after").expect("target edit");
        assert_eq!(before, snapshot(&link));
        std::fs::remove_file(&target).expect("remove target");
        assert_eq!(
            before,
            snapshot(&link),
            "broken link retains entry identity"
        );
    }
}

// Charge bytes even when a growing file or a partial I/O failure rejects capture.
fn read_regular_body(reader: impl Read, remaining: &mut usize) -> std::io::Result<Vec<u8>> {
    let available = *remaining;
    let mut bytes = Vec::new();
    let result = reader
        .take((available as u64).saturating_add(1))
        .read_to_end(&mut bytes);
    *remaining = available.saturating_sub(bytes.len());
    result?;
    if bytes.len() > available {
        return Err(std::io::Error::other(
            "Git regular evidence exceeds its limit",
        ));
    }
    Ok(bytes)
}

#[cfg(test)]
#[path = "snapshot_budget_tests.rs"]
mod budget_tests;
