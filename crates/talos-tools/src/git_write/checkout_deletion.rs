//! Exact-admitted inventory of child directories that checkout may remove.
//! Directory admission permits names only; child reads wait for fresh admission.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::{GitToolError, checkout_state::TargetParents, snapshot};

pub(super) struct Inventory {
    workdir: PathBuf,
    roots: Vec<String>,
    parents: TargetParents,
    pending: Vec<PathBuf>,
    identities: BTreeMap<PathBuf, DeletionIdentity>,
    directories: BTreeMap<PathBuf, Vec<PathBuf>>,
    files: BTreeMap<PathBuf, snapshot::RegularEvidence>,
    remaining_bytes: usize,
}

#[derive(PartialEq, Eq)]
struct DeletionIdentity {
    path: snapshot::PathIdentity,
    special: Option<SpecialKind>,
}

#[derive(PartialEq, Eq)]
enum SpecialKind {
    #[cfg(unix)]
    Fifo,
    #[cfg(unix)]
    Socket,
}

fn special_kind(metadata: &std::fs::Metadata) -> Option<SpecialKind> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt;
        let kind = metadata.file_type();
        if kind.is_fifo() {
            Some(SpecialKind::Fifo)
        } else if kind.is_socket() {
            Some(SpecialKind::Socket)
        } else {
            None
        }
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        None
    }
}

fn invalid() -> GitToolError {
    GitToolError::InvalidInput(
        "Checkout deletion inventory changed, redirected, or exceeded its limit".into(),
    )
}

impl Inventory {
    // Names are already discovered gitlinks; do not inspect their descendants.
    pub(super) fn new(workdir: &Path, roots: Vec<String>) -> Result<Self, GitToolError> {
        let parents = TargetParents::capture(workdir, &roots)?;
        let mut pending: Vec<_> = roots.iter().map(|root| workdir.join(root)).collect();
        pending.sort();
        pending.dedup();
        Ok(Self {
            workdir: workdir.to_owned(),
            roots,
            parents,
            pending,
            identities: BTreeMap::new(),
            directories: BTreeMap::new(),
            files: BTreeMap::new(),
            remaining_bytes: 16 * 1024 * 1024,
        })
    }

    pub(super) fn paths(&self) -> Vec<PathBuf> {
        self.identities
            .keys()
            .cloned()
            .chain(self.pending.iter().cloned())
            .collect()
    }

    pub(super) fn roots(&self) -> Vec<PathBuf> {
        self.roots
            .iter()
            .map(|root| self.workdir.join(root))
            .collect()
    }

    pub(super) fn complete(&self) -> bool {
        self.pending.is_empty()
    }

    pub(super) fn revalidate(&self) -> Result<(), GitToolError> {
        if TargetParents::capture(&self.workdir, &self.roots)? != self.parents {
            return Err(invalid());
        }
        // Parent paths sort before children. Reject their replacement before
        // looking up a descendant or opening any retained file contents.
        for (path, expected) in &self.identities {
            if &identity(path)? != expected {
                return Err(invalid());
            }
        }
        for (path, expected) in &self.directories {
            if &children(path)? != expected {
                return Err(invalid());
            }
        }
        let mut budget = snapshot::Budget {
            entries: 4096,
            bytes: 16 * 1024 * 1024,
        };
        for (path, expected) in &self.files {
            if &snapshot::RegularEvidence::capture(path, &mut budget).map_err(|_| invalid())?
                != expected
            {
                return Err(invalid());
            }
        }
        Ok(())
    }

    pub(super) fn advance(&mut self, admitted: impl Fn(&Path) -> bool) -> Result<(), GitToolError> {
        if self.paths().iter().any(|path| !admitted(path)) {
            return Err(invalid());
        }
        self.revalidate()?;
        let mut next = Vec::new();
        let retained_entries = self.identities.len() + self.pending.len();
        let mut budget = snapshot::Budget {
            entries: 4096,
            bytes: self.remaining_bytes,
        };
        for path in &self.pending {
            let before = identity(path)?;
            let metadata = std::fs::symlink_metadata(path).map_err(|_| invalid())?;
            if metadata.is_dir() {
                let entries = children(path)?;
                if retained_entries + next.len() + entries.len() > 4096 {
                    return Err(invalid());
                }
                next.extend(entries.iter().cloned());
                self.directories.insert(path.clone(), entries);
            } else if metadata.is_file() {
                self.files.insert(
                    path.clone(),
                    snapshot::RegularEvidence::capture(path, &mut budget).map_err(|_| invalid())?,
                );
            } else if special_kind(&metadata).is_none() {
                return Err(invalid());
            }
            if identity(path)? != before {
                return Err(invalid());
            }
            self.identities.insert(path.clone(), before);
        }
        next.sort();
        next.dedup();
        self.pending = next;
        self.remaining_bytes = budget.bytes;
        self.revalidate()
    }
}

fn identity(path: &Path) -> Result<DeletionIdentity, GitToolError> {
    let metadata = std::fs::symlink_metadata(path).map_err(|_| invalid())?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes()
            & windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT
            != 0
        {
            return Err(invalid());
        }
    }
    let special = special_kind(&metadata);
    if !(metadata.is_dir() || metadata.is_file() || special.is_some())
        || path.canonicalize().map_err(|_| invalid())? != path
    {
        return Err(invalid());
    }
    Ok(DeletionIdentity {
        path: snapshot::PathIdentity::capture(path).map_err(|_| invalid())?,
        special,
    })
}

fn children(path: &Path) -> Result<Vec<PathBuf>, GitToolError> {
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(path).map_err(|_| invalid())? {
        if paths.len() >= 4096 {
            return Err(invalid());
        }
        paths.push(entry.map_err(|_| invalid())?.path());
    }
    paths.sort();
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn socket_deletion_requires_admission_and_rejects_replacement_without_connecting() {
        use std::os::unix::net::UnixListener;

        let temp = tempfile::tempdir().expect("fixture");
        let root = temp.path().canonicalize().expect("root");
        let child = root.join("child");
        std::fs::create_dir(&child).expect("child");
        let socket = child.join("socket");
        let listener = UnixListener::bind(&socket).expect("socket");
        listener
            .set_nonblocking(true)
            .expect("nonblocking listener");
        let mut inventory = Inventory::new(&root, vec!["child".into()]).expect("inventory");
        inventory.advance(|path| path == child).expect("names");
        assert!(inventory.advance(|path| path != socket).is_err());
        assert!(!inventory.identities.contains_key(&socket));
        inventory.advance(|_| true).expect("socket metadata only");
        assert!(inventory.complete());
        assert!(inventory.files.is_empty());
        assert_eq!(
            listener.accept().expect_err("no connection").kind(),
            std::io::ErrorKind::WouldBlock
        );
        std::fs::rename(&socket, root.join("original-socket")).expect("retain socket inode");
        let _replacement = UnixListener::bind(&socket).expect("replacement socket");
        assert!(inventory.revalidate().is_err());
        std::fs::remove_file(&socket).expect("remove replacement socket");
        assert!(
            std::process::Command::new("mkfifo")
                .arg(&socket)
                .status()
                .expect("FIFO replacement")
                .success()
        );
        assert!(inventory.revalidate().is_err());
        std::fs::remove_file(&socket).expect("remove FIFO replacement");
        std::fs::write(&socket, b"replacement").expect("regular replacement");
        assert!(inventory.revalidate().is_err());
        std::fs::remove_file(&socket).expect("remove regular replacement");
        std::os::unix::fs::symlink(root.join("original-socket"), &socket)
            .expect("link replacement");
        assert!(inventory.revalidate().is_err());
    }

    #[cfg(unix)]
    #[test]
    fn fifo_deletion_needs_admission_and_rejects_identity_drift_without_opening() {
        let temp = tempfile::tempdir().expect("fixture");
        let root = temp.path().canonicalize().expect("root");
        let child = root.join("child");
        std::fs::create_dir(&child).expect("child");
        let fifo = child.join("pipe");
        assert!(
            std::process::Command::new("mkfifo")
                .arg(&fifo)
                .status()
                .expect("FIFO")
                .success()
        );
        let mut inventory = Inventory::new(&root, vec!["child".into()]).expect("inventory");
        inventory.advance(|path| path == child).expect("names");
        assert!(inventory.advance(|path| path != fifo).is_err());
        assert!(!inventory.identities.contains_key(&fifo));
        inventory
            .advance(|_| true)
            .expect("FIFO metadata only, no writer");
        assert!(inventory.complete());
        assert!(inventory.files.is_empty());
        std::fs::rename(&fifo, root.join("original-pipe")).expect("retain original inode");
        assert!(
            std::process::Command::new("mkfifo")
                .arg(&fifo)
                .status()
                .expect("FIFO")
                .success()
        );
        assert!(inventory.revalidate().is_err());
        std::fs::remove_file(&fifo).expect("remove replacement FIFO");
        std::fs::write(&fifo, b"replacement regular file").expect("regular replacement");
        assert!(inventory.revalidate().is_err());
        std::fs::remove_file(&fifo).expect("remove regular replacement");
        std::os::unix::fs::symlink(root.join("original-pipe"), &fifo).expect("link replacement");
        assert!(inventory.revalidate().is_err());
    }

    #[test]
    fn checkout_deletion_requires_child_admission_and_detects_late_changes() {
        let temp = tempfile::tempdir().expect("fixture");
        let root = temp.path().canonicalize().expect("canonical root");
        let child = root.join("child");
        std::fs::create_dir(&child).expect("child");
        let file = child.join("untracked");
        std::fs::write(&file, b"preserve").expect("file");
        let mut inventory = Inventory::new(&root, vec!["child".into()]).expect("inventory");
        inventory
            .advance(|path| path == child)
            .expect("directory names");
        assert!(!inventory.complete());
        assert!(inventory.files.is_empty());
        assert!(inventory.advance(|path| path == child).is_err());
        assert!(inventory.files.is_empty());
        inventory.advance(|_| true).expect("child admitted");
        assert!(inventory.complete());
        std::fs::write(&file, b"changed").expect("change");
        assert!(inventory.revalidate().is_err());
        std::fs::write(&file, b"preserve").expect("restore");
        inventory.revalidate().expect("restored bytes");
        std::fs::write(child.join("late"), b"late").expect("late child");
        assert!(inventory.revalidate().is_err());
    }

    #[cfg(unix)]
    #[test]
    fn checkout_deletion_rejects_replacement_and_symlink_entries() {
        let temp = tempfile::tempdir().expect("fixture");
        let root = temp.path().canonicalize().expect("canonical root");
        let child = root.join("child");
        std::fs::create_dir(&child).expect("child");
        let mut inventory = Inventory::new(&root, vec!["child".into()]).expect("inventory");
        inventory.advance(|_| true).expect("empty directory");
        std::fs::rename(&child, root.join("original")).expect("retain inode");
        std::fs::create_dir(&child).expect("replacement");
        assert!(inventory.revalidate().is_err());
        std::os::unix::fs::symlink(root.join("outside"), child.join("link")).expect("link");
        let mut inventory = Inventory::new(&root, vec!["child".into()]).expect("new inventory");
        inventory.advance(|_| true).expect("names only");
        assert!(inventory.advance(|_| true).is_err());
    }

    #[test]
    fn checkout_deletion_enforces_cumulative_bytes_and_entry_limit() {
        let temp = tempfile::tempdir().expect("fixture");
        let root = temp.path().canonicalize().expect("root");
        let child = root.join("child");
        std::fs::create_dir_all(child.join("nested")).expect("nested");
        std::fs::write(child.join("first"), b"1234").expect("first");
        std::fs::write(child.join("nested/second"), b"5678").expect("second");
        let mut inventory = Inventory::new(&root, vec!["child".into()]).expect("inventory");
        inventory.remaining_bytes = 7;
        inventory.advance(|_| true).expect("root names");
        inventory
            .advance(|_| true)
            .expect("first file and nested names");
        assert_eq!(inventory.remaining_bytes, 3);
        assert!(inventory.advance(|_| true).is_err());
        assert_eq!(
            std::fs::read(child.join("nested/second")).expect("preserved"),
            b"5678"
        );

        let wide = root.join("wide");
        std::fs::create_dir(&wide).expect("wide");
        for index in 0..4096 {
            std::fs::write(wide.join(index.to_string()), []).expect("entry");
        }
        let mut inventory = Inventory::new(&root, vec!["wide".into()]).expect("inventory");
        assert!(
            inventory.advance(|_| true).is_err(),
            "root plus 4096 children exceeds total budget"
        );
        assert!(inventory.files.is_empty());
        assert_eq!(
            std::fs::read_dir(wide)
                .expect("preserved directory")
                .count(),
            4096
        );
    }
}
