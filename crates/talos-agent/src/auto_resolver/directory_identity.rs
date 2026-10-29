//! Retained directory objects for asynchronous approval freshness checks.
use std::path::Path;
use std::sync::Arc;

use serde::Serialize;
use sha2::{Digest, Sha256};

/// Keeps the original object alive while comparing its serialized identity.
#[derive(Clone, Debug, Serialize)]
#[serde(transparent)]
pub struct ExecutionDirectoryIdentity {
    token: String,
    #[serde(skip)]
    _directory: Arc<cap_std::fs::Dir>,
}

impl ExecutionDirectoryIdentity {
    pub(crate) fn token(&self) -> &str {
        &self.token
    }
}

impl PartialEq for ExecutionDirectoryIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.token == other.token
    }
}

/// Captures an open directory until the approval/execution context is dropped.
///
/// Reopening the path detects replacement while the original handle prevents
/// its identifier from being recycled. This is freshness evidence, not atomic
/// execution confinement. Windows' upstream 64-bit index does not cover ReFS's
/// full 128-bit identity. On Windows cap-std also prevents rename/deletion of
/// the retained directory by opening without FILE_SHARE_DELETE.
pub fn execution_directory_identity(path: &Path) -> Option<ExecutionDirectoryIdentity> {
    std::panic::catch_unwind(|| {
        use cap_fs_ext::MetadataExt;
        let canonical = path.canonicalize().ok()?;
        let directory =
            cap_std::fs::Dir::open_ambient_dir(&canonical, cap_std::ambient_authority()).ok()?;
        let metadata = directory.dir_metadata().ok()?;
        if !metadata.is_dir() {
            return None;
        }
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_os_str().as_encoded_bytes());
        hasher.update(metadata.dev().to_le_bytes());
        hasher.update(metadata.ino().to_le_bytes());
        let digest = hasher.finalize();
        Some(ExecutionDirectoryIdentity {
            token: format!(
                "sha256:{}",
                digest
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>()
            ),
            _directory: Arc::new(directory),
        })
    })
    .ok()
    .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execution_directory_identity_detects_replacement() {
        let root = tempfile::tempdir().expect("temporary root");
        let path = root.path().join("working");
        std::fs::create_dir(&path).expect("original directory");
        let original = execution_directory_identity(&path).expect("directory identity");
        assert_eq!(
            execution_directory_identity(&path).as_ref(),
            Some(&original)
        );
        assert_eq!(
            serde_json::to_value(&original).expect("serialized token"),
            serde_json::Value::String(original.token().to_owned())
        );
        #[cfg(windows)]
        {
            // cap-std intentionally excludes FILE_SHARE_DELETE. Do not replace
            // its handle with a share-delete handle: Dir requires that invariant.
            let error = std::fs::rename(&path, root.path().join("retained"))
                .expect_err("retained Windows directory cannot be replaced");
            assert_eq!(error.raw_os_error(), Some(32));
            assert_eq!(
                execution_directory_identity(&path).as_ref(),
                Some(&original)
            );
            drop(original);
            std::fs::rename(&path, root.path().join("retained"))
                .expect("rename after releasing identity");
            std::fs::create_dir(&path).expect("replacement after release");
            assert!(execution_directory_identity(&path).is_some());
        }
        #[cfg(not(windows))]
        {
            std::fs::rename(&path, root.path().join("retained")).expect("retain original");
            // The retained handle outlives even deletion of the original pathname.
            std::fs::remove_dir(root.path().join("retained")).expect("unlink original");
            assert!(
                original
                    ._directory
                    .dir_metadata()
                    .expect("open original")
                    .is_dir()
            );
            std::fs::create_dir(&path).expect("replacement directory");
            let replacement = execution_directory_identity(&path).expect("replacement identity");
            assert_ne!(original, replacement);
        }
        assert!(execution_directory_identity(&root.path().join("missing")).is_none());
        let file = root.path().join("file");
        std::fs::write(&file, b"").expect("regular file");
        assert!(execution_directory_identity(&file).is_none());
    }
}
