//! Receiver layout evidence. Capture only after the corresponding Read admission.
//! Pointer targets remain path strings until a later, independently admitted wave.

use std::io::Read;
use std::path::{Path, PathBuf};

use super::{GitToolError, snapshot};

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Root {
    pub spelling: PathBuf,
    pub canonical: PathBuf,
    pub metadata_path: PathBuf,
    identity: snapshot::PathIdentity,
    marker: Marker,
}

#[derive(Debug, PartialEq, Eq)]
enum Marker {
    Missing,
    Directory(LeafIdentity),
    File(LeafIdentity, snapshot::Entry),
}

// lstat identity deliberately avoids canonicalizing the marker itself: a
// concurrently substituted symlink must not trigger an external target query.
#[derive(Debug, PartialEq, Eq)]
struct LeafIdentity {
    #[cfg(unix)]
    device: u64,
    #[cfg(unix)]
    inode: u64,
}

impl LeafIdentity {
    fn from_metadata(metadata: &std::fs::Metadata) -> Self {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            Self {
                device: metadata.dev(),
                inode: metadata.ino(),
            }
        }
        #[cfg(not(unix))]
        {
            let _ = metadata;
            Self {}
        }
    }
}

fn invalid() -> GitToolError {
    GitToolError::InvalidInput("Receiver layout evidence unavailable or changed".into())
}

/// Detached worktree occupancy evidence, captured only after exact admission
/// of the directory and every named file. Never enumerate arbitrary children.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct RebaseState {
    path: PathBuf,
    directory: Option<LeafIdentity>,
    parent: snapshot::PathIdentity,
    files: Vec<snapshot::RegularEvidence>,
}

impl RebaseState {
    pub(super) fn leaves(merge: bool) -> &'static [&'static str] {
        if merge {
            &["interactive", "head-name", "onto"]
        } else {
            &["applying", "patch", "head-name", "onto"]
        }
    }

    pub(super) fn capture(path: &Path, admitted: &Path, merge: bool) -> Result<Self, GitToolError> {
        let parent_path = path.parent().ok_or_else(invalid)?;
        let check_mapping = || -> Result<(), GitToolError> {
            if parent_path
                .canonicalize()
                .map_err(|_| invalid())?
                .join(path.file_name().ok_or_else(invalid)?)
                != admitted
            {
                return Err(invalid());
            }
            Ok(())
        };
        check_mapping()?;
        let parent = snapshot::PathIdentity::capture(parent_path).map_err(|_| invalid())?;
        let directory_identity = || match std::fs::symlink_metadata(path) {
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
                Ok(Some(LeafIdentity::from_metadata(&metadata)))
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            _ => Err(invalid()),
        };
        let directory = directory_identity()?;
        let mut budget = snapshot::Budget {
            entries: 4,
            bytes: 4 * 1024 * 1024,
        };
        let mut files = Vec::new();
        if directory.is_some() {
            for leaf in Self::leaves(merge) {
                check_mapping()?;
                if directory_identity()? != directory {
                    return Err(invalid());
                }
                files.push(
                    snapshot::RegularEvidence::capture(&path.join(leaf), &mut budget)
                        .map_err(|_| invalid())?,
                );
            }
        }
        check_mapping()?;
        if directory_identity()? != directory
            || snapshot::PathIdentity::capture(parent_path).map_err(|_| invalid())? != parent
        {
            return Err(invalid());
        }
        Ok(Self {
            path: path.to_owned(),
            directory,
            parent,
            files,
        })
    }
}

impl Root {
    pub(super) fn capture(spelling: &Path, admitted: &Path) -> Result<Self, GitToolError> {
        let canonical = spelling.canonicalize().map_err(|_| invalid())?;
        if canonical != admitted || !canonical.is_dir() {
            return Err(invalid());
        }
        let identity = snapshot::PathIdentity::capture(spelling).map_err(|_| invalid())?;
        let path = canonical.join(".git");
        let (marker, metadata_path) = match std::fs::symlink_metadata(&path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                (Marker::Missing, canonical.clone())
            }
            Ok(metadata) if metadata.is_dir() => (
                Marker::Directory(LeafIdentity::from_metadata(&metadata)),
                path,
            ),
            Ok(metadata) if metadata.is_file() => {
                let identity = LeafIdentity::from_metadata(&metadata);
                let entry = capture_pointer(&path)?;
                let snapshot::Entry::File(_, bytes) = &entry else {
                    return Err(invalid());
                };
                let raw = bytes.strip_prefix(b"gitdir: ").ok_or_else(invalid)?;
                let target = pointer_target(&canonical, raw)?;
                let after = std::fs::symlink_metadata(&path).map_err(|_| invalid())?;
                if !after.is_file() || LeafIdentity::from_metadata(&after) != identity {
                    return Err(invalid());
                }
                (Marker::File(identity, entry), target)
            }
            _ => return Err(invalid()),
        };
        if spelling.canonicalize().map_err(|_| invalid())? != canonical
            || snapshot::PathIdentity::capture(spelling).map_err(|_| invalid())? != identity
        {
            return Err(invalid());
        }
        Ok(Self {
            spelling: spelling.to_owned(),
            canonical,
            metadata_path,
            identity,
            marker,
        })
    }

    pub(super) fn revalidate(&self, admitted: &Path) -> Result<(), GitToolError> {
        if &Self::capture(&self.spelling, admitted)? != self {
            return Err(invalid());
        }
        Ok(())
    }
}

/// Metadata wave: only the admitted gitdir, layout pointers and HEAD are read.
/// `common_path` and `worktree_marker` are deliberately unresolved next resources.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Metadata {
    pub spelling: PathBuf,
    pub canonical: PathBuf,
    pub common_path: PathBuf,
    pub worktree_marker: Option<PathBuf>,
    identity: snapshot::PathIdentity,
    common_pointer: Pointer,
    worktree_pointer: Pointer,
    head: Pointer,
}

#[derive(Debug, PartialEq, Eq)]
enum Pointer {
    Missing,
    File(LeafIdentity, snapshot::Entry),
}

/// Verification of sibling pointer targets after their separate Read wave.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct SiblingTargets {
    common: snapshot::PathIdentity,
    marker: Option<(Pointer, snapshot::PathIdentity)>,
    pub backlink_target: Option<PathBuf>,
}

impl SiblingTargets {
    pub(super) fn capture(
        metadata: &Metadata,
        admitted_common: &Path,
        expected_common: &Path,
        admitted_marker: Option<&Path>,
    ) -> Result<Self, GitToolError> {
        if metadata.common_path.canonicalize().map_err(|_| invalid())? != admitted_common
            || admitted_common != expected_common
        {
            return Err(invalid());
        }
        let common =
            snapshot::PathIdentity::capture(&metadata.common_path).map_err(|_| invalid())?;
        let mut backlink_target = None;
        let marker = match (&metadata.worktree_marker, admitted_marker) {
            (Some(path), Some(admitted)) => {
                let parent = path.parent().ok_or_else(invalid)?;
                let resolved_parent = parent.canonicalize().map_err(|_| invalid())?;
                if resolved_parent.join(path.file_name().ok_or_else(invalid)?) != admitted {
                    return Err(invalid());
                }
                let parent_identity =
                    snapshot::PathIdentity::capture(parent).map_err(|_| invalid())?;
                let pointer = Pointer::capture(path)?;
                let Pointer::File(_, snapshot::Entry::File(_, bytes)) = &pointer else {
                    return Err(invalid());
                };
                let target = pointer_target(
                    &resolved_parent,
                    bytes.strip_prefix(b"gitdir: ").ok_or_else(invalid)?,
                )?;
                // A relative/aliased target receives its own next-wave Read;
                // no target resolution occurs while parsing this marker.
                backlink_target = Some(target);
                if Pointer::capture(path)? != pointer
                    || snapshot::PathIdentity::capture(parent).map_err(|_| invalid())?
                        != parent_identity
                    || parent.canonicalize().map_err(|_| invalid())? != resolved_parent
                {
                    return Err(invalid());
                }
                Some((pointer, parent_identity))
            }
            (None, None) => None,
            _ => return Err(invalid()),
        };
        if snapshot::PathIdentity::capture(&metadata.common_path).map_err(|_| invalid())? != common
        {
            return Err(invalid());
        }
        Ok(Self {
            common,
            marker,
            backlink_target,
        })
    }

    pub(super) fn verify_backlink(
        &self,
        metadata: &Metadata,
        admitted: Option<&Path>,
    ) -> Result<(), GitToolError> {
        match (&self.backlink_target, admitted) {
            (None, None) => Ok(()),
            (Some(target), Some(admitted)) => {
                if target.canonicalize().map_err(|_| invalid())? != admitted
                    || admitted != metadata.canonical
                    || snapshot::PathIdentity::capture(target).map_err(|_| invalid())?
                        != metadata.identity
                {
                    return Err(invalid());
                }
                Ok(())
            }
            _ => Err(invalid()),
        }
    }
}

impl Pointer {
    fn capture(path: &Path) -> Result<Self, GitToolError> {
        let metadata = match std::fs::symlink_metadata(path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Self::Missing),
            Ok(metadata) if metadata.is_file() => metadata,
            _ => return Err(invalid()),
        };
        let identity = LeafIdentity::from_metadata(&metadata);
        let entry = capture_pointer(path)?;
        let after = std::fs::symlink_metadata(path).map_err(|_| invalid())?;
        if !after.is_file() || LeafIdentity::from_metadata(&after) != identity {
            return Err(invalid());
        }
        Ok(Self::File(identity, entry))
    }

    fn target(&self, base: &Path) -> Result<Option<PathBuf>, GitToolError> {
        match self {
            Self::Missing => Ok(None),
            Self::File(_, snapshot::Entry::File(_, bytes)) => pointer_target(base, bytes).map(Some),
            _ => Err(invalid()),
        }
    }
}

impl Metadata {
    /// Read the branch only from the already admitted HEAD snapshot.
    pub(super) fn branch(&self) -> Result<Option<&str>, GitToolError> {
        let Pointer::File(_, snapshot::Entry::File(_, bytes)) = &self.head else {
            return Err(invalid());
        };
        let text = std::str::from_utf8(bytes).map_err(|_| invalid())?;
        let text = text.trim_end_matches(['\r', '\n']);
        if let Some(reference) = text.strip_prefix("ref: ") {
            // Non-branch symbolic chains need separately admitted resolution.
            let branch = reference.strip_prefix("refs/heads/").ok_or_else(invalid)?;
            if branch.is_empty() {
                return Err(invalid());
            }
            Ok(Some(branch))
        } else if matches!(text.len(), 40 | 64) && text.bytes().all(|b| b.is_ascii_hexdigit()) {
            Ok(None)
        } else {
            Err(invalid())
        }
    }

    /// Whether startup found a commondir pointer, even when it resolves back
    /// to the same directory. Git uses presence, not canonical inequality.
    pub(super) fn has_common(&self) -> bool {
        !matches!(self.common_pointer, Pointer::Missing)
    }

    pub(super) fn capture(spelling: &Path, admitted: &Path) -> Result<Self, GitToolError> {
        let canonical = spelling.canonicalize().map_err(|_| invalid())?;
        if canonical != admitted || !canonical.is_dir() {
            return Err(invalid());
        }
        let identity = snapshot::PathIdentity::capture(spelling).map_err(|_| invalid())?;
        let common_pointer = Pointer::capture(&canonical.join("commondir"))?;
        let worktree_pointer = Pointer::capture(&canonical.join("gitdir"))?;
        let head = Pointer::capture(&canonical.join("HEAD"))?;
        if let Pointer::File(_, snapshot::Entry::File(_, bytes)) = &head {
            validate_symbolic_target(bytes)?;
        }
        let common_path = common_pointer
            .target(&canonical)?
            .unwrap_or_else(|| canonical.clone());
        let worktree_marker = worktree_pointer.target(&canonical)?;
        if spelling.canonicalize().map_err(|_| invalid())? != canonical
            || snapshot::PathIdentity::capture(spelling).map_err(|_| invalid())? != identity
            || Pointer::capture(&canonical.join("commondir"))? != common_pointer
            || Pointer::capture(&canonical.join("gitdir"))? != worktree_pointer
            || Pointer::capture(&canonical.join("HEAD"))? != head
        {
            return Err(invalid());
        }
        Ok(Self {
            spelling: spelling.to_owned(),
            canonical,
            common_path,
            worktree_marker,
            identity,
            common_pointer,
            worktree_pointer,
            head,
        })
    }

    pub(super) fn revalidate(&self, admitted: &Path) -> Result<(), GitToolError> {
        if &Self::capture(&self.spelling, admitted)? != self {
            return Err(invalid());
        }
        Ok(())
    }
}

/// Shared-directory and linked-worktree marker evidence, captured only after
/// their own admission. Reads bounded local layout hints and inventories include
/// declarations; never discovers repositories or reads include targets.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Common {
    common_spelling: PathBuf,
    pub canonical: PathBuf,
    identity: snapshot::PathIdentity,
    marker: Option<(PathBuf, Pointer)>,
    pub backlink_target: Option<PathBuf>,
    local_config: Pointer,
    worktree_config: Option<Pointer>,
    pub local_worktrees: Vec<PathBuf>,
    pub includes: Vec<(PathBuf, PathBuf)>,
    references: Vec<ReferenceTree>,
    packed_refs: snapshot::RegularEvidence,
    pub reference_entries: usize,
    pub reference_bytes: usize,
}

/// Immediate worktree registration names only. Child contents require fresh
/// exact-path admission; this does not grant recursive directory access.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct WorktreeInventory {
    path: PathBuf,
    identity: Option<snapshot::PathIdentity>,
    pub names: Vec<std::ffi::OsString>,
}

impl WorktreeInventory {
    pub(super) fn capture(path: &Path, admitted: &Path) -> Result<Self, GitToolError> {
        let parent = path.parent().ok_or_else(invalid)?;
        let parent_identity = snapshot::PathIdentity::capture(parent).map_err(|_| invalid())?;
        if parent
            .canonicalize()
            .map_err(|_| invalid())?
            .join(path.file_name().ok_or_else(invalid)?)
            != admitted
        {
            return Err(invalid());
        }
        let (identity, leaf) = match std::fs::symlink_metadata(path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (None, None),
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => (
                Some(snapshot::PathIdentity::capture(path).map_err(|_| invalid())?),
                Some(LeafIdentity::from_metadata(&metadata)),
            ),
            _ => return Err(invalid()),
        };
        let check_mapping = || -> Result<(), GitToolError> {
            if snapshot::PathIdentity::capture(parent).map_err(|_| invalid())? != parent_identity
                || parent
                    .canonicalize()
                    .map_err(|_| invalid())?
                    .join(path.file_name().ok_or_else(invalid)?)
                    != admitted
            {
                return Err(invalid());
            }
            match (&leaf, std::fs::symlink_metadata(path)) {
                (None, Err(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                (Some(expected), Ok(current))
                    if current.is_dir()
                        && !current.file_type().is_symlink()
                        && &LeafIdentity::from_metadata(&current) == expected
                        && path.canonicalize().ok().as_deref() == Some(admitted)
                        && snapshot::PathIdentity::capture(path).ok() == identity =>
                {
                    Ok(())
                }
                _ => Err(invalid()),
            }
        };
        check_mapping()?;
        let mut names = Vec::new();
        if identity.is_some() {
            for entry in std::fs::read_dir(path).map_err(|_| invalid())? {
                if names.len() == 1024 {
                    return Err(GitToolError::InvalidInput(
                        "Receiver worktree inventory exceeds 1024 entries".into(),
                    ));
                }
                names.push(entry.map_err(|_| invalid())?.file_name());
            }
            names.sort();
        }
        check_mapping()?;
        Ok(Self {
            path: path.to_owned(),
            identity,
            names,
        })
    }

    pub(super) fn revalidate(&self, admitted: &Path) -> Result<(), GitToolError> {
        if &Self::capture(&self.path, admitted)? != self {
            return Err(invalid());
        }
        Ok(())
    }
}

// A receiver's onbranch condition can follow several symbolic refs without
// changing HEAD itself. Retain both files namespaces before config inspection.
// The prepared caller must admit the closed receiver-reference-tree capability
// and check the exact shared/private refs roots before entering this tree walk.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum ReferenceTree {
    File(snapshot::RegularEvidence),
    Directory(
        snapshot::PathIdentity,
        Vec<(std::ffi::OsString, ReferenceTree)>,
    ),
}

impl ReferenceTree {
    /// Capture one separately admitted refs root using the caller's shared budget.
    pub(super) fn capture_admitted(
        path: &Path,
        admitted: &Path,
        budget: &mut snapshot::Budget,
    ) -> Result<Self, GitToolError> {
        if path != admitted {
            return Err(invalid());
        }
        Self::capture(path, budget, 0)
    }
    fn capture(
        path: &Path,
        budget: &mut snapshot::Budget,
        depth: usize,
    ) -> Result<Self, GitToolError> {
        if depth > 128 || budget.entries == 0 {
            return Err(invalid());
        }
        let parent = path.parent().ok_or_else(invalid)?;
        let parent_identity = snapshot::PathIdentity::capture(parent).map_err(|_| invalid())?;
        if parent.canonicalize().map_err(|_| invalid())? != parent {
            return Err(invalid());
        }
        let captured = match std::fs::symlink_metadata(path) {
            Ok(metadata) if metadata.is_dir() => {
                budget.entries -= 1;
                let leaf = LeafIdentity::from_metadata(&metadata);
                let identity = snapshot::PathIdentity::capture(path).map_err(|_| invalid())?;
                if path.canonicalize().map_err(|_| invalid())? != path
                    || std::fs::symlink_metadata(path)
                        .ok()
                        .filter(|current| current.is_dir())
                        .map(|current| LeafIdentity::from_metadata(&current))
                        .as_ref()
                        != Some(&leaf)
                {
                    return Err(invalid());
                }
                let mut children = Vec::new();
                for child in std::fs::read_dir(path).map_err(|_| invalid())? {
                    let child = child.map_err(|_| invalid())?;
                    children.push((
                        child.file_name(),
                        Self::capture(&child.path(), budget, depth + 1)?,
                    ));
                }
                children.sort_by(|left, right| left.0.cmp(&right.0));
                if snapshot::PathIdentity::capture(path).map_err(|_| invalid())? != identity
                    || path.canonicalize().map_err(|_| invalid())? != path
                    || std::fs::symlink_metadata(path)
                        .ok()
                        .filter(|current| current.is_dir())
                        .map(|current| LeafIdentity::from_metadata(&current))
                        .as_ref()
                        != Some(&leaf)
                {
                    return Err(invalid());
                }
                Ok(Self::Directory(identity, children))
            }
            Ok(metadata) if !metadata.is_file() => Err(invalid()),
            Ok(_) => Self::file(path, budget),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Self::file(path, budget),
            Err(_) => Err(invalid()),
        }?;
        if snapshot::PathIdentity::capture(parent).map_err(|_| invalid())? != parent_identity
            || parent.canonicalize().map_err(|_| invalid())? != parent
        {
            return Err(invalid());
        }
        Ok(captured)
    }

    fn file(path: &Path, budget: &mut snapshot::Budget) -> Result<Self, GitToolError> {
        let evidence = snapshot::RegularEvidence::capture(path, budget).map_err(|_| invalid())?;
        if let snapshot::Entry::File(_, bytes) = &evidence.entry {
            validate_symbolic_target(bytes)?;
        }
        Ok(Self::File(evidence))
    }
}

pub(super) fn validate_symbolic_target(bytes: &[u8]) -> Result<(), GitToolError> {
    let Some(target) = bytes.strip_prefix(b"ref:") else {
        return Ok(());
    };
    let target = target.trim_ascii();
    // Reject pseudo refs and cross-worktree aliases: they would name inputs
    // outside the two namespaces retained here. Never follow the target path.
    let valid = std::panic::catch_unwind(|| {
        target.starts_with(b"refs/")
            && gix::refs::FullName::try_from(gix::bstr::BString::from(target)).is_ok()
    })
    .unwrap_or(false);
    if !valid {
        return Err(GitToolError::InvalidInput(
            "Receiver symbolic reference outside retained refs namespaces; use a separately authorized Git invocation".into(),
        ));
    }
    Ok(())
}

impl Common {
    #[cfg(test)]
    pub(super) fn capture(
        metadata: &Metadata,
        admitted_common: &Path,
        admitted_marker: Option<&Path>,
    ) -> Result<Self, GitToolError> {
        Self::capture_with_budget(
            metadata,
            admitted_common,
            admitted_marker,
            &mut snapshot::Budget {
                entries: 65_536,
                bytes: 16 * 1024 * 1024,
            },
        )
    }

    pub(super) fn capture_with_budget(
        metadata: &Metadata,
        admitted_common: &Path,
        admitted_marker: Option<&Path>,
        reference_budget: &mut snapshot::Budget,
    ) -> Result<Self, GitToolError> {
        let canonical = metadata.common_path.canonicalize().map_err(|_| invalid())?;
        if canonical != admitted_common || !canonical.is_dir() {
            return Err(invalid());
        }
        let identity =
            snapshot::PathIdentity::capture(&metadata.common_path).map_err(|_| invalid())?;
        let (marker, backlink_target) = match (&metadata.worktree_marker, admitted_marker) {
            (None, None) => (None, None),
            (Some(path), Some(admitted)) => {
                // Resolve the parent only, then reject leaf links in Pointer.
                // Verify the admitted mapping before opening any marker bytes.
                let parent = path.parent().ok_or_else(invalid)?;
                let resolved_parent = parent.canonicalize().map_err(|_| invalid())?;
                let resolved = resolved_parent.join(path.file_name().ok_or_else(invalid)?);
                if resolved != admitted {
                    return Err(invalid());
                }
                let before = Pointer::capture(path)?;
                let Pointer::File(_, snapshot::Entry::File(_, bytes)) = &before else {
                    return Err(invalid());
                };
                let raw = bytes.strip_prefix(b"gitdir: ").ok_or_else(invalid)?;
                let target = pointer_target(&resolved_parent, raw)?;
                if Pointer::capture(path)? != before {
                    return Err(invalid());
                }
                (Some((path.clone(), before)), Some(target))
            }
            _ => return Err(invalid()),
        };
        if snapshot::PathIdentity::capture(&metadata.common_path).map_err(|_| invalid())?
            != identity
        {
            return Err(invalid());
        }
        let local_config = Pointer::capture(&canonical.join("config"))?;
        let local = local_layout_hint(&local_config, &metadata.canonical)?;
        // Reftable needs a separate backend snapshot; do not mistake its refs
        // directory stubs for a complete files-backend inventory.
        match std::fs::symlink_metadata(canonical.join("reftable")) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            _ => return Err(GitToolError::InvalidInput(
                "Receiver reftable evidence is not supported; use a separately authorized Git invocation".into(),
            )),
        }
        let before_entries = reference_budget.entries;
        let before_bytes = reference_budget.bytes;
        let mut references = vec![ReferenceTree::capture(
            &canonical.join("refs"),
            reference_budget,
            0,
        )?];
        if canonical != metadata.canonical {
            references.push(ReferenceTree::capture(
                &metadata.canonical.join("refs"),
                reference_budget,
                0,
            )?);
        }
        let packed_refs =
            snapshot::RegularEvidence::capture(&canonical.join("packed-refs"), reference_budget)
                .map_err(|_| invalid())?;
        let mut includes: Vec<_> = local
            .includes
            .into_iter()
            .map(|path| (canonical.clone(), path))
            .collect();
        let mut local_worktrees: Vec<_> = local.worktree.into_iter().collect();
        let worktree_config = if local.worktree_config {
            let config = Pointer::capture(&metadata.canonical.join("config.worktree"))?;
            let hint = local_layout_hint(&config, &metadata.canonical)?;
            includes.extend(
                hint.includes
                    .into_iter()
                    .map(|path| (metadata.canonical.clone(), path)),
            );
            if let Some(path) = hint.worktree
                && !local_worktrees.contains(&path)
            {
                local_worktrees.push(path);
            }
            Some(config)
        } else {
            None
        };
        if let Some(expected) = &worktree_config
            && &Pointer::capture(&metadata.canonical.join("config.worktree"))? != expected
        {
            return Err(invalid());
        }
        if Pointer::capture(&canonical.join("config"))? != local_config
            || snapshot::PathIdentity::capture(&metadata.common_path).map_err(|_| invalid())?
                != identity
        {
            return Err(invalid());
        }
        Ok(Self {
            references,
            packed_refs,
            reference_entries: before_entries - reference_budget.entries,
            reference_bytes: before_bytes - reference_budget.bytes,
            local_config,
            worktree_config,
            local_worktrees,
            includes,
            common_spelling: metadata.common_path.clone(),
            canonical,
            identity,
            marker,
            backlink_target,
        })
    }

    /// Verify the backlink only in the next stage, after its target received
    /// independent Read admission. Matching bytes alone cannot establish layout.
    pub(super) fn verify_backlink(
        &self,
        metadata: &Metadata,
        admitted_target: Option<&Path>,
    ) -> Result<(), GitToolError> {
        match (&self.backlink_target, admitted_target) {
            (None, None) => Ok(()),
            (Some(target), Some(admitted)) => {
                let resolved = target.canonicalize().map_err(|_| invalid())?;
                if resolved != admitted
                    || resolved != metadata.canonical
                    || snapshot::PathIdentity::capture(target).map_err(|_| invalid())?
                        != metadata.identity
                {
                    return Err(GitToolError::InvalidInput(
                        "Receiver worktree backlink does not match admitted gitdir".into(),
                    ));
                }
                Ok(())
            }
            _ => Err(invalid()),
        }
    }

    #[cfg(test)]
    pub(super) fn revalidate(
        &self,
        metadata: &Metadata,
        common: &Path,
        marker: Option<&Path>,
    ) -> Result<(), GitToolError> {
        self.revalidate_with_budget(
            metadata,
            common,
            marker,
            &mut snapshot::Budget {
                entries: 65_536,
                bytes: 16 * 1024 * 1024,
            },
        )
    }

    pub(super) fn revalidate_with_budget(
        &self,
        metadata: &Metadata,
        common: &Path,
        marker: Option<&Path>,
        budget: &mut snapshot::Budget,
    ) -> Result<(), GitToolError> {
        if &Self::capture_with_budget(metadata, common, marker, budget)? != self {
            return Err(invalid());
        }
        Ok(())
    }
}

// Bootstrap hint only: never expand includes or claim effective configuration.
// The caller retains the file evidence and admits the returned path separately.
#[derive(Default)]
struct LocalLayoutHint {
    worktree: Option<PathBuf>,
    worktree_config: bool,
    includes: Vec<PathBuf>,
}

fn local_layout_hint(config: &Pointer, gitdir: &Path) -> Result<LocalLayoutHint, GitToolError> {
    let Pointer::File(_, snapshot::Entry::File(_, bytes)) = config else {
        return Ok(LocalLayoutHint::default());
    };
    std::panic::catch_unwind(|| {
        let file = gix::config::File::from_bytes_no_includes(
            bytes,
            gix::config::file::Metadata::default(),
            Default::default(),
        )
        .map_err(|_| invalid())?;
        if let Ok(storage) = file.raw_value("extensions.refStorage")
            && !storage.eq_ignore_ascii_case(b"files")
        {
            return Err(GitToolError::InvalidInput(
                "Receiver reference backend requires separate effect binding".into(),
            ));
        }
        let mut includes = Vec::new();
        for section in file.sections() {
            let header = section.header();
            let name: &[u8] = header.name().as_ref();
            if (name.eq_ignore_ascii_case(b"include") && header.subsection_name().is_none())
                || (name.eq_ignore_ascii_case(b"includeif") && header.subsection_name().is_some())
            {
                // Inventory all declarations, including inactive conditions.
                // No condition evaluation or target access occurs here.
                for value in section.body().values("path") {
                    let bytes: &[u8] = value.as_ref();
                    let text = std::str::from_utf8(bytes).map_err(|_| invalid())?;
                    if text.is_empty() || text.contains(['\0', '\n', '\r']) {
                        return Err(invalid());
                    }
                    includes.push(PathBuf::from(text));
                }
            }
        }
        let worktree_config = file
            .boolean("extensions.worktreeConfig")
            .map_err(|_| invalid())?
            .unwrap_or(false);
        let Ok(value) = file.raw_value("core.worktree") else {
            return Ok(LocalLayoutHint {
                worktree: None,
                worktree_config,
                includes,
            });
        };
        let bytes: &[u8] = value.as_ref();
        let text = std::str::from_utf8(bytes).map_err(|_| invalid())?;
        if text.is_empty() || text.starts_with('~') || text.contains(['\0', '\n', '\r']) {
            return Err(invalid());
        }
        super::remote_state::validate_local_path_syntax(Path::new(text))?;
        Ok(LocalLayoutHint {
            worktree: Some(gitdir.join(text)),
            worktree_config,
            includes,
        })
    })
    .unwrap_or_else(|_| Err(invalid()))
}

// Never recurse or follow a leaf symlink, including a substitution between stat
// and open on Unix. Ancestor/content races remain path-based TOCTOU residuals.
fn capture_pointer(path: &Path) -> Result<snapshot::Entry, GitToolError> {
    let metadata = std::fs::symlink_metadata(path).map_err(|_| invalid())?;
    if !metadata.is_file() || metadata.len() > 16 * 1024 {
        return Err(invalid());
    }
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options.open(path).map_err(|_| invalid())?;
    let opened = file.metadata().map_err(|_| invalid())?;
    if !opened.is_file() || opened.len() > 16 * 1024 {
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
    file.take(16 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| invalid())?;
    if bytes.len() > 16 * 1024 {
        return Err(invalid());
    }
    Ok(snapshot::Entry::File(opened.permissions(), bytes))
}

fn pointer_target(base: &Path, bytes: &[u8]) -> Result<PathBuf, GitToolError> {
    let bytes = bytes.strip_suffix(b"\n").unwrap_or(bytes);
    let bytes = bytes.strip_suffix(b"\r").unwrap_or(bytes);
    let text = std::str::from_utf8(bytes).map_err(|_| invalid())?;
    if text.is_empty() || text.contains(['\0', '\n', '\r']) {
        return Err(invalid());
    }
    super::remote_state::validate_local_path_syntax(Path::new(text))?;
    Ok(base.join(text))
}

#[cfg(test)]
mod tests {
    #[test]
    fn sibling_targets_defer_relative_backlink_and_bind_marker() {
        let root = tempfile::tempdir().expect("fixture");
        let base = root.path().canonicalize().expect("canonical");
        let gitdir = base.join("worktrees/sibling");
        let worktree = base.join("checkout");
        std::fs::create_dir_all(&gitdir).expect("gitdir");
        std::fs::create_dir(&worktree).expect("worktree");
        let marker = worktree.join(".git");
        std::fs::write(gitdir.join("commondir"), "../..\n").expect("common pointer");
        std::fs::write(gitdir.join("gitdir"), marker.to_str().expect("marker path"))
            .expect("worktree pointer");
        std::fs::write(&marker, "gitdir: ../worktrees/sibling\n").expect("relative backlink");
        let metadata = super::Metadata::capture(&gitdir, &gitdir).expect("metadata");
        assert!(super::SiblingTargets::capture(&metadata, &base, &base, None).is_err());
        let before = super::SiblingTargets::capture(&metadata, &base, &base, Some(&marker))
            .expect("targets");
        assert!(before.verify_backlink(&metadata, None).is_err());
        before
            .verify_backlink(&metadata, Some(&gitdir))
            .expect("admitted backlink");
        assert!(
            super::SiblingTargets::capture(&metadata, &base, &worktree, Some(&marker)).is_err()
        );
        std::fs::write(&marker, "gitdir: ../not-yet-admitted\n").expect("new target");
        let changed = super::SiblingTargets::capture(&metadata, &base, &base, Some(&marker))
            .expect("capture without opening target");
        assert_ne!(before, changed);
        assert!(!base.join("not-yet-admitted").exists());
        assert!(changed.verify_backlink(&metadata, Some(&gitdir)).is_err());
    }

    #[test]
    fn rebase_state_binds_directory_and_each_occupancy_file() {
        let root = tempfile::tempdir().expect("fixture");
        for (name, merge) in [("rebase-apply", false), ("rebase-merge", true)] {
            let path = root.path().canonicalize().expect("canonical").join(name);
            let missing = super::RebaseState::capture(&path, &path, merge).expect("missing");
            std::fs::create_dir(&path).expect("directory");
            let empty = super::RebaseState::capture(&path, &path, merge).expect("empty");
            assert_ne!(missing, empty);
            for leaf in super::RebaseState::leaves(merge) {
                let file = path.join(leaf);
                std::fs::write(&file, "refs/heads/occupied\n").expect("state");
                let populated =
                    super::RebaseState::capture(&path, &path, merge).expect("populated");
                assert_ne!(empty, populated);
                std::fs::write(&file, "refs/heads/modified\n").expect("change");
                assert_ne!(
                    populated,
                    super::RebaseState::capture(&path, &path, merge).expect("changed")
                );
                std::fs::remove_file(&file).expect("restore");
                assert_eq!(
                    empty,
                    super::RebaseState::capture(&path, &path, merge).expect("restored")
                );
            }
            // Unrelated descendants are outside this exact-file capability.
            std::fs::create_dir(path.join("unread-child")).expect("unrelated");
            assert_eq!(
                empty,
                super::RebaseState::capture(&path, &path, merge).expect("no enumeration")
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn rebase_state_refuses_directory_and_file_symlinks() {
        let root = tempfile::tempdir().expect("fixture");
        let outside = tempfile::tempdir().expect("outside");
        let path = root
            .path()
            .canonicalize()
            .expect("canonical")
            .join("rebase-merge");
        std::os::unix::fs::symlink(outside.path(), &path).expect("directory link");
        assert!(super::RebaseState::capture(&path, &path, true).is_err());
        std::fs::remove_file(&path).expect("remove link");
        std::fs::create_dir(&path).expect("directory");
        std::os::unix::fs::symlink(outside.path().join("secret"), path.join("head-name"))
            .expect("file link");
        assert!(super::RebaseState::capture(&path, &path, true).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn rebase_state_rejects_parent_retarget_against_admitted_path() {
        let root = tempfile::tempdir().expect("fixture");
        let first = tempfile::tempdir().expect("first");
        let second = tempfile::tempdir().expect("second");
        let link = root.path().join("gitdir");
        std::os::unix::fs::symlink(first.path(), &link).expect("parent link");
        let path = link.join("rebase-merge");
        let admitted = first
            .path()
            .canonicalize()
            .expect("canonical")
            .join("rebase-merge");
        super::RebaseState::capture(&path, &admitted, true).expect("original mapping");
        std::fs::remove_file(&link).expect("remove link");
        std::os::unix::fs::symlink(second.path(), &link).expect("retarget");
        assert!(super::RebaseState::capture(&path, &admitted, true).is_err());
    }

    use super::*;

    #[test]
    fn worktree_inventory_binds_names_and_defers_child_contents() {
        let root = tempfile::tempdir().expect("fixture");
        let path = root
            .path()
            .canonicalize()
            .expect("canonical")
            .join("worktrees");
        let missing = WorktreeInventory::capture(&path, &path).expect("absent");
        std::fs::create_dir(&path).expect("registrations");
        assert!(missing.revalidate(&path).is_err());
        std::fs::write(path.join("not-a-directory"), "not opened").expect("candidate");
        let inventory = WorktreeInventory::capture(&path, &path).expect("names only");
        assert_eq!(
            inventory.names,
            vec![std::ffi::OsString::from("not-a-directory")]
        );
        std::fs::write(path.join("not-a-directory"), "changed content").expect("change");
        inventory
            .revalidate(&path)
            .expect("content not in directory inventory");
        assert!(WorktreeInventory::capture(&path, &path.with_extension("wrong")).is_err());
        std::fs::create_dir(path.join("new")).expect("new registration");
        assert!(inventory.revalidate(&path).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn worktree_inventory_refuses_root_link_but_does_not_follow_child_links() {
        let root = tempfile::tempdir().expect("fixture");
        let base = root.path().canonicalize().expect("canonical");
        let path = base.join("worktrees");
        std::fs::create_dir(&path).expect("registrations");
        std::os::unix::fs::symlink(base.join("absent"), path.join("child")).expect("child");
        assert_eq!(
            WorktreeInventory::capture(&path, &path)
                .expect("names")
                .names
                .len(),
            1
        );
        let alias = base.join("alias");
        std::os::unix::fs::symlink(&path, &alias).expect("root link");
        assert!(WorktreeInventory::capture(&alias, &path).is_err());
        let before = WorktreeInventory::capture(&path, &path).expect("original directory");
        std::fs::rename(&path, base.join("old-worktrees")).expect("retain directory inode");
        std::fs::create_dir(&path).expect("replacement directory");
        std::os::unix::fs::symlink(base.join("absent"), path.join("child")).expect("same name");
        assert!(before.revalidate(&path).is_err());
        std::fs::remove_file(path.join("child")).expect("remove child");
        std::fs::remove_dir(&path).expect("remove replacement");
        std::os::unix::fs::symlink(base.join("old-worktrees"), &path).expect("retarget");
        assert!(before.revalidate(&path).is_err());
    }

    #[test]
    fn receiver_refs_bind_chains_packed_and_private_namespaces() {
        let directory = tempfile::tempdir().expect("fixture");
        let common = directory.path().canonicalize().expect("canonical");
        let gitdir = common.join("worktrees/linked");
        std::fs::create_dir_all(gitdir.join("refs/worktree")).expect("private refs");
        std::fs::create_dir_all(common.join("refs/heads")).expect("shared refs");
        std::fs::write(gitdir.join("commondir"), "../..\n").expect("commondir");
        std::fs::write(gitdir.join("HEAD"), "ref: refs/heads/first\n").expect("head");
        std::fs::write(common.join("refs/heads/first"), "ref: refs/heads/second\n").expect("chain");
        let metadata = Metadata::capture(&gitdir, &gitdir).expect("metadata");
        let before = Common::capture(&metadata, &common, None).expect("capture unborn chain");
        before
            .revalidate(&metadata, &common, None)
            .expect("unchanged");
        std::fs::write(common.join("refs/heads/first"), "ref: refs/heads/third\n")
            .expect("retarget");
        metadata.revalidate(&gitdir).expect("HEAD unchanged");
        assert!(before.revalidate(&metadata, &common, None).is_err());
        for path in [
            common.join("packed-refs"),
            gitdir.join("refs/worktree/private"),
            common.join("refs/heads/third"),
        ] {
            let before = Common::capture(&metadata, &common, None).expect("capture");
            std::fs::write(&path, "0000000000000000000000000000000000000001\n")
                .expect("changed ref");
            assert!(
                before.revalidate(&metadata, &common, None).is_err(),
                "{}",
                path.display()
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn receiver_refs_reject_replacements_links_and_unbound_targets() {
        let directory = tempfile::tempdir().expect("fixture");
        let base = directory.path().canonicalize().expect("canonical");
        std::fs::create_dir_all(base.join("refs/heads")).expect("refs");
        let path = base.join("refs/heads/main");
        let bytes = "0000000000000000000000000000000000000001\n";
        std::fs::write(&path, bytes).expect("ref");
        let metadata = Metadata::capture(&base, &base).expect("metadata");
        let before = Common::capture(&metadata, &base, None).expect("capture");
        std::fs::rename(&path, base.join("retained-inode")).expect("retain inode");
        std::fs::write(&path, bytes).expect("identical replacement");
        assert!(before.revalidate(&metadata, &base, None).is_err());
        std::fs::remove_file(&path).expect("remove");
        std::os::unix::fs::symlink(base.join("not-opened"), &path).expect("leaf link");
        assert!(Common::capture(&metadata, &base, None).is_err());
        std::fs::remove_file(&path).expect("remove link");
        for bytes in [
            "ref: worktrees/other/HEAD\n",
            "ref: refs/../../outside\n",
            "ref: FETCH_HEAD\n",
        ] {
            std::fs::write(&path, bytes).expect("unbound reference");
            assert!(Common::capture(&metadata, &base, None).is_err());
        }
        std::fs::remove_file(&path).expect("remove ref");
        std::fs::remove_dir(base.join("refs/heads")).expect("remove directory");
        std::os::unix::fs::symlink(base.join("not-opened"), base.join("refs/heads"))
            .expect("directory link");
        assert!(Common::capture(&metadata, &base, None).is_err());
    }

    #[test]
    fn receiver_refs_refuse_unbound_backend_and_share_capture_budget() {
        let directory = tempfile::tempdir().expect("fixture");
        let base = directory.path().canonicalize().expect("canonical");
        let metadata = Metadata::capture(&base, &base).expect("metadata");
        std::fs::write(base.join("config"), "[extensions]\nrefStorage = reftable\n")
            .expect("config");
        assert!(Common::capture(&metadata, &base, None).is_err());
        std::fs::remove_file(base.join("config")).expect("remove config");
        std::fs::create_dir(base.join("reftable")).expect("backend directory");
        assert!(Common::capture(&metadata, &base, None).is_err());
        std::fs::create_dir(base.join("refs")).expect("refs");
        std::fs::write(base.join("refs/one"), "first").expect("ref one");
        std::fs::write(base.join("refs/two"), "second").expect("ref two");
        assert!(
            ReferenceTree::capture(
                &base.join("refs"),
                &mut snapshot::Budget {
                    entries: 2,
                    bytes: 100
                },
                0
            )
            .is_err()
        );
        assert!(
            ReferenceTree::capture(
                &base.join("refs"),
                &mut snapshot::Budget {
                    entries: 10,
                    bytes: 10
                },
                0
            )
            .is_err()
        );
    }

    #[test]
    fn receiver_reference_budget_spans_common_and_sibling_roots() {
        let directory = tempfile::tempdir().expect("fixture");
        let base = directory.path().canonicalize().expect("canonical");
        std::fs::create_dir(base.join("refs")).expect("refs");
        std::fs::write(base.join("refs/main"), "1234").expect("reference");
        let metadata = Metadata::capture(&base, &base).expect("metadata");
        let mut budget = super::snapshot::Budget {
            entries: 4,
            bytes: 7,
        };
        let common =
            Common::capture_with_budget(&metadata, &base, None, &mut budget).expect("common");
        assert_eq!(common.reference_entries, 3);
        assert_eq!(common.reference_bytes, 4);
        assert_eq!(budget.entries, 1);
        assert_eq!(budget.bytes, 3);
        let sibling = base.join("sibling");
        std::fs::create_dir(&sibling).expect("sibling");
        std::fs::write(sibling.join("private"), "1234").expect("private ref");
        assert!(super::ReferenceTree::capture_admitted(&sibling, &sibling, &mut budget).is_err());
        let mut bytes_only = super::snapshot::Budget {
            entries: 10,
            bytes: 3,
        };
        assert!(
            super::ReferenceTree::capture_admitted(&sibling, &sibling, &mut bytes_only).is_err()
        );
    }

    #[test]
    fn metadata_revalidation_binds_branch_conditional_head() {
        let directory = tempfile::tempdir().expect("fixture");
        let base = directory.path().canonicalize().expect("canonical");
        let path = base.join("HEAD");
        std::fs::write(&path, "ref: refs/heads/before\n").expect("head");
        let before = Metadata::capture(&base, &base).expect("capture");
        assert_eq!(before.branch().expect("branch"), Some("before"));
        before.revalidate(&base).expect("unchanged");
        std::fs::write(&path, "ref: refs/heads/after\n").expect("switch");
        assert!(before.revalidate(&base).is_err());
        assert_eq!(before.branch().expect("retained branch"), Some("before"));
        std::fs::write(&path, format!("{}\n", "a".repeat(40))).expect("detached");
        let detached = Metadata::capture(&base, &base).expect("capture detached");
        assert_eq!(detached.branch().expect("detached branch"), None);
        std::fs::write(&path, "ref: refs/custom/indirect\n").expect("symbolic");
        let indirect = Metadata::capture(&base, &base).expect("capture symbolic");
        assert!(indirect.branch().is_err());
    }

    #[test]
    fn host_layout_bootstrap_does_not_expand_global_or_include_configuration() {
        let directory = tempfile::tempdir().expect("fixture");
        // Git for Windows does not accept every Rust verbatim path spelling.
        let base = if cfg!(windows) {
            directory.path().to_owned()
        } else {
            directory.path().canonicalize().expect("canonical")
        };
        let repository = base.join("repository");
        let command = || {
            let mut command = std::process::Command::new("git");
            command
                .env_clear()
                .env("PATH", std::env::var_os("PATH").unwrap_or_default())
                .env("HOME", &base)
                .env("XDG_CONFIG_HOME", base.join("xdg"))
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_TRACE2", "0")
                .env("GIT_TRACE2_EVENT", "0")
                .env("GIT_TRACE2_PERF", "0");
            command
        };
        let output = command()
            .arg("init")
            .arg(&repository)
            .output()
            .expect("init");
        assert!(output.status.success(), "{output:?}");
        let gitdir = repository.join(".git");
        std::fs::write(
            base.join(".gitconfig"),
            "[extensions]\nworktreeConfig=true\n[core]\nworktree=../missing-global\n",
        )
        .expect("global config");
        std::fs::write(
            gitdir.join("included"),
            "[extensions]\nworktreeConfig=true\n[core]\nworktree=../missing-include\n",
        )
        .expect("include");
        std::fs::write(gitdir.join("config.worktree"), "[probe]\nworktree=active\n")
            .expect("worktree config");
        for enabled in [false, true] {
            std::fs::write(
                gitdir.join("config"),
                format!(
                    "[core]\nrepositoryFormatVersion=0\nbare=false\n[include]\npath=included\n{}",
                    if enabled {
                        "[extensions]\nworktreeConfig=true\n"
                    } else {
                        ""
                    }
                ),
            )
            .expect("config");
            let metadata = Metadata::capture(&gitdir, &gitdir).expect("metadata");
            let evidence = Common::capture(&metadata, &gitdir, None).expect("bootstrap");
            assert_eq!(evidence.worktree_config.is_some(), enabled);
            assert!(evidence.local_worktrees.is_empty());
            let output = command()
                .current_dir(&gitdir)
                .args(["--git-dir=.", "config", "--null", "--list", "--includes"])
                .output()
                .expect("config query");
            assert!(output.status.success());
            let records: Vec<_> = output.stdout.split(|byte| *byte == 0).collect();
            assert_eq!(
                records.contains(&b"probe.worktree\nactive".as_slice()),
                enabled
            );
            assert!(records.contains(&b"core.worktree\n../missing-global".as_slice()));
            assert!(records.contains(&b"core.worktree\n../missing-include".as_slice()));
        }
    }

    #[test]
    fn include_inventory_rejects_implicit_and_empty_paths() {
        let directory = tempfile::tempdir().expect("fixture");
        let base = directory.path().canonicalize().expect("canonical");
        let path = base.join("config");
        for body in [
            "path\n",
            "path\nother=/outside\n",
            "path=\n",
            "path=\"\"\n",
            "path=valid\nPaTh\nother=/outside\n",
        ] {
            for header in ["[include]", "[includeIf \"onbranch:never\"]"] {
                std::fs::write(&path, format!("{header}\n{body}")).expect("config");
                let config = Pointer::capture(&path).expect("capture");
                assert!(
                    local_layout_hint(&config, &base).is_err(),
                    "{header}: {body}"
                );
            }
        }
    }

    #[test]
    fn include_inventory_preserves_all_declarations_and_source_bases() {
        let directory = tempfile::tempdir().expect("fixture");
        let base = directory.path().canonicalize().expect("canonical");
        let gitdir = base.join("linked");
        std::fs::create_dir(&gitdir).expect("gitdir");
        std::fs::write(gitdir.join("commondir"), "..\n").expect("commondir");
        std::fs::write(base.join("config"), "[extensions]\nworktreeConfig=true\n[include]\npath=first\npath=second\n[includeIf \"onbranch:never\"]\npath=../conditional\n[include \"not-an-include\"]\npath=ignored\n").expect("config");
        std::fs::write(
            gitdir.join("config.worktree"),
            "[include]\npath=\"local file\"\n",
        )
        .expect("worktree config");
        let metadata = Metadata::capture(&gitdir, &gitdir).expect("metadata");
        let common = Common::capture(&metadata, &base, None).expect("common");
        assert_eq!(
            common.includes,
            vec![
                (base.clone(), PathBuf::from("first")),
                (base.clone(), PathBuf::from("second")),
                (base.clone(), PathBuf::from("../conditional")),
                (gitdir, PathBuf::from("local file")),
            ]
        );
        assert!(!base.join("first").exists());
        assert!(!base.join("second").exists());
    }

    #[test]
    fn worktree_specific_layout_hints_retain_both_candidate_paths() {
        for enabled in [false, true] {
            let directory = tempfile::tempdir().expect("fixture");
            let base = directory.path().canonicalize().expect("canonical");
            std::fs::write(base.join("config"), format!("[extensions]\nworktreeConfig = {enabled}\n[core]\nworktree = ../shared-worktree\n")).expect("config");
            std::fs::write(
                base.join("config.worktree"),
                "[core]\nworktree = ../specific-worktree\n",
            )
            .expect("worktree config");
            let metadata = Metadata::capture(&base, &base).expect("metadata");
            let before = Common::capture(&metadata, &base, None).expect("capture");
            assert_eq!(before.local_worktrees.len(), if enabled { 2 } else { 1 });
            assert_eq!(before.worktree_config.is_some(), enabled);
            assert!(
                before
                    .local_worktrees
                    .contains(&base.join("../shared-worktree"))
            );
            if enabled {
                assert!(
                    before
                        .local_worktrees
                        .contains(&base.join("../specific-worktree"))
                );
            }
            std::fs::write(
                base.join("config.worktree"),
                "[core]\nworktree = ../changed\n",
            )
            .expect("change");
            assert_eq!(before.revalidate(&metadata, &base, None).is_err(), enabled);
        }
    }

    #[test]
    fn local_config_hint_is_bounded_and_does_not_expand_includes() {
        let directory = tempfile::tempdir().expect("fixture");
        let base = directory.path().canonicalize().expect("canonical");
        let path = base.join("config");
        std::fs::write(&path, "[include]\npath = absent.conf\n[core]\nworktree = ../first\nworktree = \"../space name\"\n").expect("config");
        let metadata = Metadata::capture(&base, &base).expect("metadata");
        let common = Common::capture(&metadata, &base, None).expect("bootstrap");
        assert_eq!(common.local_worktrees, vec![base.join("../space name")]);
        assert!(!base.join("absent.conf").exists());
        std::fs::write(&path, "[core]\nworktree = ../changed\n").expect("change");
        assert!(common.revalidate(&metadata, &base, None).is_err());
        std::fs::write(&path, vec![b'x'; 16 * 1024 + 1]).expect("oversize");
        assert!(Common::capture(&metadata, &base, None).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn backlink_relative_parent_uses_canonical_marker_parent() {
        let directory = tempfile::tempdir().expect("fixture");
        let base = directory.path().canonicalize().expect("canonical");
        let real = base.join("real/worktree");
        std::fs::create_dir_all(&real).expect("worktree");
        let gitdir = base.join("real/metadata");
        std::fs::create_dir(&gitdir).expect("gitdir");
        let alias = base.join("alias");
        std::os::unix::fs::symlink(&real, &alias).expect("alias");
        std::fs::write(real.join(".git"), "gitdir: ../metadata\n").expect("marker");
        std::fs::write(
            gitdir.join("gitdir"),
            format!("{}\n", alias.join(".git").display()),
        )
        .expect("backpointer");
        let metadata = Metadata::capture(&gitdir, &gitdir).expect("metadata");
        let common = Common::capture(&metadata, &gitdir, Some(&real.join(".git"))).expect("common");
        assert_eq!(common.backlink_target, Some(real.join("../metadata")));
        common
            .verify_backlink(&metadata, Some(&gitdir))
            .expect("canonical-base backlink");
        assert!(!base.join("metadata").exists());
    }

    #[test]
    fn backlink_requires_admission_and_original_gitdir_identity() {
        let directory = tempfile::tempdir().expect("fixture");
        let base = directory.path().canonicalize().expect("canonical");
        let gitdir = base.join("metadata");
        let marker = base.join(".git");
        std::fs::create_dir(&gitdir).expect("gitdir");
        std::fs::write(gitdir.join("gitdir"), format!("{}\n", marker.display())).expect("pointer");
        std::fs::write(&marker, "gitdir: metadata\n").expect("marker");
        let metadata = Metadata::capture(&gitdir, &gitdir).expect("metadata");
        let common = Common::capture(&metadata, &gitdir, Some(&marker)).expect("common");
        assert!(common.verify_backlink(&metadata, None).is_err());
        assert!(common.verify_backlink(&metadata, Some(&base)).is_err());
        common
            .verify_backlink(&metadata, Some(&gitdir))
            .expect("matching backlink");
        std::fs::write(&marker, "gitdir: .\n").expect("inconsistent pointer");
        let wrong = Common::capture(&metadata, &gitdir, Some(&marker))
            .expect("capture wrong target without following");
        assert!(wrong.verify_backlink(&metadata, Some(&base)).is_err());
        #[cfg(unix)]
        {
            std::fs::rename(&gitdir, base.join("old-metadata")).expect("retain inode");
            std::fs::create_dir(&gitdir).expect("same-path replacement");
            assert!(common.verify_backlink(&metadata, Some(&gitdir)).is_err());
        }
    }

    #[test]
    fn common_wave_binds_marker_and_defers_backlink_target() {
        let directory = tempfile::tempdir().expect("fixture");
        let base = directory.path().canonicalize().expect("canonical");
        let gitdir = base.join("metadata");
        let common = base.join("common");
        let marker = base.join(".git");
        std::fs::create_dir(&gitdir).expect("gitdir");
        std::fs::create_dir(&common).expect("common");
        std::fs::write(gitdir.join("commondir"), "../common\n").expect("pointer");
        std::fs::write(gitdir.join("gitdir"), format!("{}\n", marker.display())).expect("pointer");
        std::fs::write(&marker, "gitdir: not-created\n").expect("marker");
        let metadata = Metadata::capture(&gitdir, &gitdir).expect("metadata");
        assert!(Common::capture(&metadata, &common, None).is_err());
        assert!(Common::capture(&metadata, &common, Some(&gitdir)).is_err());
        let before = Common::capture(&metadata, &common, Some(&marker)).expect("common");
        assert_eq!(before.backlink_target, Some(base.join("not-created")));
        before
            .revalidate(&metadata, &common, Some(&marker))
            .expect("unchanged");
        std::fs::write(&marker, "gitdir: changed\n").expect("retarget");
        assert!(
            before
                .revalidate(&metadata, &common, Some(&marker))
                .is_err()
        );
        assert!(!base.join("not-created").exists());
    }

    #[test]
    fn metadata_without_pointers_keeps_admitted_directory() {
        let directory = tempfile::tempdir().expect("metadata");
        let canonical = directory.path().canonicalize().expect("canonical");
        let before = Metadata::capture(directory.path(), &canonical).expect("capture");
        assert_eq!(before.common_path, canonical);
        assert!(before.worktree_marker.is_none());
        before.revalidate(&canonical).expect("unchanged");
        std::fs::write(canonical.join("gitdir"), "../external/.git\n").expect("new backpointer");
        assert!(before.revalidate(&canonical).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn metadata_rejects_pointer_symlinks_and_identical_byte_replacement() {
        let directory = tempfile::tempdir().expect("metadata");
        let canonical = directory.path().canonicalize().expect("canonical");
        let pointer = canonical.join("commondir");
        std::fs::write(&pointer, "../common\n").expect("pointer");
        let before = Metadata::capture(directory.path(), &canonical).expect("capture");
        std::fs::rename(&pointer, canonical.join("old-pointer")).expect("retain inode");
        std::fs::write(&pointer, "../common\n").expect("replacement");
        assert!(before.revalidate(&canonical).is_err());
        std::fs::remove_file(&pointer).expect("remove");
        std::os::unix::fs::symlink(canonical.join("nonexistent"), &pointer).expect("symlink");
        assert!(Metadata::capture(directory.path(), &canonical).is_err());
    }

    #[test]
    fn root_marker_changes_invalidate_evidence_without_opening_target() {
        let receiver = tempfile::tempdir().expect("receiver");
        let canonical = receiver.path().canonicalize().expect("canonical");
        let absent = Root::capture(receiver.path(), &canonical).expect("root");
        let target = canonical.join("not-created");
        std::fs::write(
            canonical.join(".git"),
            format!("gitdir: {}\n", target.display()),
        )
        .expect("marker");
        assert!(absent.revalidate(&canonical).is_err());
        let before = Root::capture(receiver.path(), &canonical).expect("gitfile");
        assert_eq!(before.metadata_path, target);
        before.revalidate(&canonical).expect("unchanged");
        std::fs::write(canonical.join(".git"), "gitdir: other\n").expect("retarget");
        assert!(before.revalidate(&canonical).is_err());
        assert!(!target.exists());
    }

    #[cfg(unix)]
    #[test]
    fn identical_marker_replacement_and_root_symlink_retarget_are_rejected() {
        let receiver = tempfile::tempdir().expect("receiver");
        let canonical = receiver.path().canonicalize().expect("canonical");
        let marker = canonical.join(".git");
        std::fs::write(&marker, "gitdir: external\n").expect("marker");
        let before = Root::capture(receiver.path(), &canonical).expect("capture");
        std::fs::rename(&marker, canonical.join("old-marker")).expect("retain inode");
        std::fs::write(&marker, "gitdir: external\n").expect("replacement");
        assert!(before.revalidate(&canonical).is_err());
        let links = tempfile::tempdir().expect("links");
        let link = links.path().join("receiver");
        std::os::unix::fs::symlink(&canonical, &link).expect("link");
        let linked = Root::capture(&link, &canonical).expect("linked root");
        std::fs::remove_file(&link).expect("unlink");
        std::os::unix::fs::symlink(links.path(), &link).expect("retarget");
        assert!(linked.revalidate(&canonical).is_err());
    }

    #[test]
    fn directory_marker_is_not_recursively_read() {
        let receiver = tempfile::tempdir().expect("receiver");
        let canonical = receiver.path().canonicalize().expect("canonical");
        std::fs::create_dir(canonical.join(".git")).expect("gitdir");
        std::fs::create_dir(canonical.join(".git/.git")).expect("nested marker");
        let before = Root::capture(receiver.path(), &canonical).expect("nonrecursive capture");
        before.revalidate(&canonical).expect("same layout");
        std::fs::rename(canonical.join(".git"), canonical.join("old-git"))
            .expect("retain directory");
        std::fs::write(canonical.join(".git"), "gitdir: external\n").expect("change kind");
        assert!(before.revalidate(&canonical).is_err());
    }
}
