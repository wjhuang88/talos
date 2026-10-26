//! Structure of receiver metadata that a push may read or modify. This is
//! path-based evidence, not OS containment. Capture requires the dedicated
//! receiver-metadata capability and exact admission of the owning roots.

use std::path::{Path, PathBuf};

use super::{GitToolError, snapshot};

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Storage {
    root: PathBuf,
    identity: snapshot::PathIdentity,
    entries: Vec<Node>,
}

#[derive(Debug, PartialEq, Eq)]
struct Node {
    path: PathBuf,
    identity: Option<snapshot::PathIdentity>,
    directory: bool,
    size: u64,
    modified: Option<std::time::SystemTime>,
}

fn invalid() -> GitToolError {
    GitToolError::InvalidInput(
        "Receiver metadata structure changed, escapes its admitted root, or exceeds the plan limit"
            .into(),
    )
}

impl Storage {
    pub(super) fn capture(
        root: &Path,
        admitted: &Path,
        remaining: &mut usize,
    ) -> Result<Self, GitToolError> {
        if root != admitted || root.canonicalize().map_err(|_| invalid())? != admitted {
            return Err(invalid());
        }
        let identity = snapshot::PathIdentity::capture(root).map_err(|_| invalid())?;
        let mut entries = Vec::new();
        // Ordinary receive-pack and its foreground maintenance operate in these
        // metadata namespaces. Config/hooks have separately bound exact evidence.
        for name in [
            "objects",
            "refs",
            "logs",
            "worktrees",
            "HEAD",
            "packed-refs",
            "shallow",
            "info",
            "gc.log",
            "gc.pid",
            "maintenance.lock",
            "rr-cache",
            "MERGE_RR",
            "MERGE_RR.lock",
        ] {
            capture_node(&root.join(name), &mut entries, remaining, 0)?;
        }
        if root.canonicalize().map_err(|_| invalid())? != admitted
            || snapshot::PathIdentity::capture(root).map_err(|_| invalid())? != identity
        {
            return Err(invalid());
        }
        Ok(Self {
            root: root.to_owned(),
            identity,
            entries,
        })
    }
}

fn capture_node(
    path: &Path,
    entries: &mut Vec<Node>,
    remaining: &mut usize,
    depth: usize,
) -> Result<(), GitToolError> {
    if depth > 128 || *remaining == 0 {
        return Err(invalid());
    }
    *remaining -= 1;
    let parent = path.parent().ok_or_else(invalid)?;
    if parent.canonicalize().map_err(|_| invalid())? != parent {
        return Err(invalid());
    }
    let parent_identity = snapshot::PathIdentity::capture(parent).map_err(|_| invalid())?;
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => Some(metadata),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(_) => return Err(invalid()),
    };
    let Some(metadata) = metadata else {
        entries.push(Node {
            path: path.to_owned(),
            identity: None,
            directory: false,
            size: 0,
            modified: None,
        });
        if !matches!(
            std::fs::symlink_metadata(path),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound
        ) || snapshot::PathIdentity::capture(parent).map_err(|_| invalid())? != parent_identity
            || parent.canonicalize().map_err(|_| invalid())? != parent
        {
            return Err(invalid());
        }
        return Ok(());
    };
    if !metadata.is_file() && !metadata.is_dir() {
        return Err(invalid());
    }
    // A pathname inside the admitted root does not grant writes through another
    // hard link to the same file outside that root. Do not assume Git always
    // replaces metadata files instead of modifying their existing inode.
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.is_file() && metadata.nlink() != 1 {
            return Err(invalid());
        }
    }
    let identity = snapshot::PathIdentity::capture(path).map_err(|_| invalid())?;
    if path.canonicalize().map_err(|_| invalid())? != path {
        return Err(invalid());
    }
    let directory = metadata.is_dir();
    let node_index = entries.len();
    entries.push(Node {
        path: path.to_owned(),
        identity: Some(identity),
        directory,
        size: metadata.len(),
        modified: metadata.modified().ok(),
    });
    if directory {
        check_leaf(path, &metadata)?;
        let mut children = Vec::new();
        for child in std::fs::read_dir(path).map_err(|_| invalid())? {
            if children.len() >= *remaining {
                return Err(invalid());
            }
            children.push(child.map_err(|_| invalid())?.path());
        }
        children.sort();
        for child in children {
            capture_node(&child, entries, remaining, depth + 1)?;
        }
    }
    check_leaf(path, &metadata)?;
    if Some(snapshot::PathIdentity::capture(path).map_err(|_| invalid())?)
        != entries[node_index].identity
        || snapshot::PathIdentity::capture(parent).map_err(|_| invalid())? != parent_identity
        || parent.canonicalize().map_err(|_| invalid())? != parent
    {
        return Err(invalid());
    }
    Ok(())
}

fn check_leaf(path: &Path, before: &std::fs::Metadata) -> Result<(), GitToolError> {
    let after = std::fs::symlink_metadata(path).map_err(|_| invalid())?;
    if after.file_type() != before.file_type()
        || after.len() != before.len()
        || after.modified().ok() != before.modified().ok()
        || path.canonicalize().map_err(|_| invalid())? != path
    {
        return Err(invalid());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if (before.dev(), before.ino(), before.nlink()) != (after.dev(), after.ino(), after.nlink())
        {
            return Err(invalid());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_binds_missing_entries_and_shared_budget_without_reading_object_bytes() {
        let directory = tempfile::tempdir().expect("fixture");
        let root = directory.path().canonicalize().expect("canonical");
        let before = Storage::capture(&root, &root, &mut 100).expect("empty");
        std::fs::create_dir(root.join("objects")).expect("objects");
        let file = std::fs::File::create(root.join("objects/object")).expect("object");
        file.set_len(64 * 1024 * 1024).expect("sparse object");
        let after = Storage::capture(&root, &root, &mut 100).expect("metadata only");
        assert_ne!(before, after);
        assert_eq!(
            after,
            Storage::capture(&root, &root, &mut 100).expect("same")
        );
        assert!(Storage::capture(&root, &root, &mut 2).is_err());
        assert!(Storage::capture(&root, &root.join("wrong"), &mut 100).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn storage_refuses_metadata_symlink_redirect() {
        let directory = tempfile::tempdir().expect("fixture");
        let root = directory.path().canonicalize().expect("canonical");
        let external = tempfile::tempdir().expect("external");
        for namespace in ["objects", "rr-cache", "MERGE_RR", "MERGE_RR.lock"] {
            std::os::unix::fs::symlink(external.path(), root.join(namespace)).expect("redirect");
            assert!(
                Storage::capture(&root, &root, &mut 100).is_err(),
                "{namespace}"
            );
            std::fs::remove_file(root.join(namespace)).expect("remove fixture link");
        }
    }

    #[cfg(unix)]
    #[test]
    fn storage_refuses_hardlinked_metadata_outside_admitted_root() {
        let directory = tempfile::tempdir().expect("fixture");
        let root = directory.path().canonicalize().expect("canonical");
        let external = tempfile::tempdir().expect("external");
        let source = external.path().join("head");
        std::fs::write(&source, "ref: refs/heads/main\n").expect("external metadata");
        std::fs::hard_link(&source, root.join("HEAD")).expect("hard link");
        assert!(Storage::capture(&root, &root, &mut 100).is_err());
        assert_eq!(
            std::fs::read_to_string(source).expect("external unchanged"),
            "ref: refs/heads/main\n"
        );
    }
}
