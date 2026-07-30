use crate::{ConfigError, atomic_file};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum FsOperation {
    CleanupPrepareResidues,
    ReadActiveManifest,
    ReadFinalizeManifest,
    ReadCleanupReadyMarker,
    ReadConfigBeforeImage,
    ReadCredentialsBeforeImage,
    ReadConfigAfterImage,
    ReadCredentialsAfterImage,
    CreatePrepareDirectory,
    WriteConfigBeforeImage,
    WriteCredentialsBeforeImage,
    WriteConfigAfterImage,
    WriteCredentialsAfterImage,
    WritePreparedManifest,
    SyncPrepareDirectory,
    PublishActiveDirectory,
    SyncTransactionParentAfterPublish,
    WriteApplyingManifest,
    ReplaceConfigAfter,
    ReplaceCredentialsAfter,
    RemoveCredentialsAfter,
    SyncCredentialsParentAfterRemove,
    VerifyConfigAfter,
    VerifyCredentialsAfter,
    WriteCommittedManifest,
    WriteRollbackRequiredManifest,
    RestoreConfigBefore,
    RestoreCredentialsBefore,
    RemoveConfigForBeforeAbsence,
    RemoveCredentialsForBeforeAbsence,
    SyncConfigParentAfterRollback,
    SyncCredentialsParentAfterRollback,
    VerifyConfigBefore,
    VerifyCredentialsBefore,
    WriteRolledBackManifest,
    PublishFinalizeDirectory,
    SyncTransactionParentAfterFinalize,
    SyncFinalizeResidueParent,
    WriteCleanupReadyMarker,
    SyncFinalizeDirectory,
    CleanupFinalizeDirectory,
    SyncTransactionParentAfterCleanup,
}

pub(crate) trait Fs {
    fn checkpoint(&self, _operation: FsOperation) -> Result<(), ConfigError> {
        Ok(())
    }

    fn exists(&self, path: &Path) -> bool;
    fn read(&self, path: &Path) -> Result<Vec<u8>, ConfigError>;

    fn read_opt(&self, path: &Path) -> Result<Option<Vec<u8>>, ConfigError> {
        if self.exists(path) {
            Ok(Some(self.read(path)?))
        } else {
            Ok(None)
        }
    }

    fn atomic_write(&self, path: &Path, content: &[u8]) -> Result<(), ConfigError>;
    fn write_secure(&self, path: &Path, content: &[u8]) -> Result<(), ConfigError>;
    fn mkdir(&self, path: &Path) -> Result<(), ConfigError>;
    fn remove_file(&self, path: &Path) -> Result<(), ConfigError>;
    fn remove_dir(&self, path: &Path) -> Result<(), ConfigError>;
    fn rename_dir(&self, from: &Path, to: &Path) -> Result<(), ConfigError>;
    fn sync_dir(&self, dir: &Path) -> Result<(), ConfigError>;
    fn list_dir(&self, dir: &Path) -> Result<Vec<PathBuf>, ConfigError>;

    fn is_real_dir(&self, path: &Path) -> Result<bool, ConfigError> {
        let metadata = std::fs::symlink_metadata(path).map_err(ConfigError::IoError)?;
        let file_type = metadata.file_type();
        Ok(file_type.is_dir() && !file_type.is_symlink())
    }
}

pub(crate) struct StdFs;

impl Fs for StdFs {
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn read(&self, path: &Path) -> Result<Vec<u8>, ConfigError> {
        std::fs::read(path).map_err(ConfigError::IoError)
    }

    fn atomic_write(&self, path: &Path, content: &[u8]) -> Result<(), ConfigError> {
        atomic_file::durable_replace(path, content)
    }

    fn write_secure(&self, path: &Path, content: &[u8]) -> Result<(), ConfigError> {
        atomic_file::write_file_synced(path, content)
    }

    fn mkdir(&self, path: &Path) -> Result<(), ConfigError> {
        atomic_file::create_dir_secure(path)
    }

    fn remove_file(&self, path: &Path) -> Result<(), ConfigError> {
        std::fs::remove_file(path).map_err(ConfigError::IoError)
    }

    fn remove_dir(&self, path: &Path) -> Result<(), ConfigError> {
        std::fs::remove_dir_all(path).map_err(ConfigError::IoError)
    }

    fn rename_dir(&self, from: &Path, to: &Path) -> Result<(), ConfigError> {
        std::fs::rename(from, to).map_err(ConfigError::IoError)
    }

    fn sync_dir(&self, dir: &Path) -> Result<(), ConfigError> {
        atomic_file::sync_dir(dir)
    }

    fn list_dir(&self, dir: &Path) -> Result<Vec<PathBuf>, ConfigError> {
        std::fs::read_dir(dir)
            .map_err(ConfigError::IoError)?
            .map(|entry| {
                entry
                    .map(|entry| entry.path())
                    .map_err(ConfigError::IoError)
            })
            .collect()
    }
}
