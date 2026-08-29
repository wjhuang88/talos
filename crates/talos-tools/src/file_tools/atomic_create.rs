use std::io::{self, Write};
use std::path::{Component, Path};

use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};
use talos_core::tool::AtomicCreateCapability;

/// Directory-capability-backed atomic creator for workspace-local new files.
pub struct CapStdAtomicCreateCapability {
    root: Dir,
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
        })
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
        assert!(capability
            .create_new(Path::new("new.txt"), b"second")
            .is_err());
        assert!(capability
            .create_new(Path::new("../escape"), b"x")
            .is_err());
    }
}
