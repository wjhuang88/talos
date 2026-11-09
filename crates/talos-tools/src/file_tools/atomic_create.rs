use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};

use cap_fs_ext::MetadataExt;
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};
use talos_core::tool::AtomicCreateCapability;

/// Directory-capability-backed atomic creator for workspace-local new files.
pub struct CapStdAtomicCreateCapability {
    root: Dir,
    path: PathBuf,
}

impl CapStdAtomicCreateCapability {
    /// Opens and holds the workspace root directory capability.
    pub fn open(root: &Path) -> io::Result<Self> {
        if !root.is_absolute() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "atomic create root must be absolute",
            ));
        }
        Ok(Self {
            root: Dir::open_ambient_dir(root, ambient_authority())?,
            path: root.to_owned(),
        })
    }
}

/// Open the nearest existing authorized parent, retaining all missing components
/// as a capability-relative suffix. This helper does not grant authorization.
pub(super) fn authorized_parent(
    path: &Path,
) -> io::Result<(CapStdAtomicCreateCapability, PathBuf)> {
    let mut parent = path
        .parent()
        .ok_or_else(|| io::Error::other("missing parent"))?;
    while !parent.exists() {
        parent = parent
            .parent()
            .ok_or_else(|| io::Error::other("missing ancestor"))?;
    }
    let relative = path
        .strip_prefix(parent)
        .map_err(|_| io::Error::other("invalid authorized parent"))?
        .to_path_buf();
    Ok((CapStdAtomicCreateCapability::open(parent)?, relative))
}

impl CapStdAtomicCreateCapability {
    fn validate_identity(&self) -> io::Result<()> {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let current = Dir::open_ambient_dir(&self.path, ambient_authority())?;
            let a = self.root.dir_metadata()?;
            let b = current.dir_metadata()?;
            if (a.dev(), a.ino()) != (b.dev(), b.ino()) {
                return Err(io::Error::other("write parent identity changed"));
            }
            Ok(())
        }))
        .map_err(|_| io::Error::other("write parent identity unavailable"))?
    }

    pub(super) fn create_new_with_parents(
        &self,
        relative: &Path,
        contents: &[u8],
    ) -> io::Result<()> {
        if relative.is_absolute()
            || relative
                .components()
                .any(|part| !matches!(part, Component::Normal(_)))
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "invalid capability-relative path",
            ));
        }
        self.validate_identity()?;
        if let Some(parent) = relative.parent() {
            self.root.create_dir_all(parent)?;
        }
        self.create_new(relative, contents)
    }
}

impl AtomicCreateCapability for CapStdAtomicCreateCapability {
    fn create_new(&self, relative_path: &Path, contents: &[u8]) -> io::Result<()> {
        if relative_path.is_absolute()
            || relative_path.components().any(|component| {
                matches!(
                    component,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "atomic create path must be relative and traversal-free",
            ));
        }
        self.validate_identity()?;
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        let mut file = self.root.open_with(relative_path, &options)?;
        file.write_all(contents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_without_clobbering_and_rejects_traversal() {
        let dir = tempfile::tempdir().expect("tempdir");
        let capability = CapStdAtomicCreateCapability::open(dir.path()).expect("capability");
        capability
            .create_new(Path::new("new.txt"), b"first")
            .expect("create");
        assert_eq!(
            std::fs::read(dir.path().join("new.txt")).expect("read"),
            b"first"
        );
        assert!(
            capability
                .create_new(Path::new("new.txt"), b"second")
                .is_err()
        );
        assert!(capability.create_new(Path::new("../escape"), b"x").is_err());
    }
}
