use super::{
    FINALIZE_PREFIX, PREPARE_PREFIX, PersistedDocument, ambiguous_finalization_error,
    parse_persisted_toml,
};
use super::{Fs, FsOperation};
use crate::ConfigError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static TXN_COUNTER: AtomicU64 = AtomicU64::new(0);
const MANIFEST_VERSION: u32 = 1;
pub(super) const CLEANUP_READY_FILE: &str = "cleanup.ready";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(super) enum Phase {
    Prepared,
    Applying,
    Committed,
    RollbackRequired,
    RolledBack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TerminalPhase {
    Committed,
    RolledBack,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum FinalizeOutcome {
    Finalized,
    ActiveRenamePending {
        transaction_id: String,
    },
    ParentSyncPending {
        transaction_id: String,
        finalize_dir: PathBuf,
    },
    CleanupPending {
        transaction_id: String,
        finalize_dir: PathBuf,
    },
}

pub(super) struct Txn {
    pub(super) cfg_existed: bool,
    pub(super) cfg_after_exists: bool,
    pub(super) cred_existed: bool,
    pub(super) cred_after_exists: bool,
    pub(super) cfg_before: Vec<u8>,
    pub(super) cfg_after: Vec<u8>,
    pub(super) cred_before: Vec<u8>,
    pub(super) cred_after: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Manifest {
    pub(super) version: u32,
    pub(super) phase: Phase,
    pub(super) transaction_id: String,
    pub(super) config_existed_before: bool,
    pub(super) config_exists_after: bool,
    pub(super) credentials_existed_before: bool,
    pub(super) credentials_exist_after: bool,
}

pub(super) fn gen_txn_id() -> String {
    let counter = TXN_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!(
        "{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0),
        counter
    )
}

pub(super) fn write_manifest(
    fs: &dyn Fs,
    dir: &Path,
    phase: Phase,
    txn_id: &str,
    txn: &Txn,
) -> Result<(), ConfigError> {
    let manifest = Manifest {
        version: MANIFEST_VERSION,
        phase,
        transaction_id: txn_id.to_string(),
        config_existed_before: txn.cfg_existed,
        config_exists_after: txn.cfg_after_exists,
        credentials_existed_before: txn.cred_existed,
        credentials_exist_after: txn.cred_after_exists,
    };
    let serialized = toml::to_string(&manifest)
        .map_err(|error| ConfigError::SerializeError(error.to_string()))?;
    fs.atomic_write(&dir.join("manifest"), serialized.as_bytes())
}

pub(super) fn parse_manifest(raw: &[u8]) -> Result<Manifest, ConfigError> {
    let manifest: Manifest = parse_persisted_toml(raw, PersistedDocument::TransactionManifest)?;
    if manifest.version != MANIFEST_VERSION {
        return Err(ConfigError::InvalidConfig(format!(
            "unsupported manifest version {} — expected {}",
            manifest.version, MANIFEST_VERSION
        )));
    }
    Ok(manifest)
}

pub(super) fn read_image(
    fs: &dyn Fs,
    dir: &Path,
    file_name: &str,
    expected: bool,
    operation: FsOperation,
) -> Result<Option<Vec<u8>>, ConfigError> {
    if !expected {
        return Ok(None);
    }
    fs.checkpoint(operation)?;
    Ok(Some(fs.read(&dir.join(file_name))?))
}

pub(super) fn validate_terminal_state(
    fs: &dyn Fs,
    journal_dir: &Path,
    config_path: &Path,
    credentials_path: &Path,
    manifest: &Manifest,
    phase: TerminalPhase,
) -> Result<(), ConfigError> {
    let (config_image, credentials_image) = match phase {
        TerminalPhase::Committed => (
            read_image(
                fs,
                journal_dir,
                "config.after",
                manifest.config_exists_after,
                FsOperation::ReadConfigAfterImage,
            )?,
            read_image(
                fs,
                journal_dir,
                "credentials.after",
                manifest.credentials_exist_after,
                FsOperation::ReadCredentialsAfterImage,
            )?,
        ),
        TerminalPhase::RolledBack => (
            read_image(
                fs,
                journal_dir,
                "config.before",
                manifest.config_existed_before,
                FsOperation::ReadConfigBeforeImage,
            )?,
            read_image(
                fs,
                journal_dir,
                "credentials.before",
                manifest.credentials_existed_before,
                FsOperation::ReadCredentialsBeforeImage,
            )?,
        ),
    };

    verify_state(
        fs,
        config_path,
        credentials_path,
        config_image.as_deref(),
        credentials_image.as_deref(),
        match phase {
            TerminalPhase::Committed => FsOperation::VerifyConfigAfter,
            TerminalPhase::RolledBack => FsOperation::VerifyConfigBefore,
        },
        match phase {
            TerminalPhase::Committed => FsOperation::VerifyCredentialsAfter,
            TerminalPhase::RolledBack => FsOperation::VerifyCredentialsBefore,
        },
        match phase {
            TerminalPhase::Committed => "Committed recovery",
            TerminalPhase::RolledBack => "RolledBack recovery",
        },
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn verify_state(
    fs: &dyn Fs,
    config_path: &Path,
    credentials_path: &Path,
    expected_config: Option<&[u8]>,
    expected_credentials: Option<&[u8]>,
    config_operation: FsOperation,
    credentials_operation: FsOperation,
    context: &str,
) -> Result<(), ConfigError> {
    fs.checkpoint(config_operation)?;
    let actual_config = fs.read_opt(config_path)?;
    fs.checkpoint(credentials_operation)?;
    let actual_credentials = fs.read_opt(credentials_path)?;

    if actual_config.as_deref() != expected_config {
        return Err(ConfigError::InvalidConfig(format!(
            "{context}: config does not match journal image"
        )));
    }
    if actual_credentials.as_deref() != expected_credentials {
        return Err(ConfigError::InvalidConfig(format!(
            "{context}: credentials do not match journal image"
        )));
    }
    Ok(())
}

pub(super) fn prepare(
    fs: &dyn Fs,
    active_dir: &Path,
    txn: &Txn,
    txn_id: &str,
) -> Result<(), ConfigError> {
    let parent = active_dir.parent().unwrap_or(Path::new("."));
    let staging_dir = parent.join(format!("{PREPARE_PREFIX}{txn_id}"));

    let staging_cleanup = |fs: &dyn Fs| {
        if fs.exists(&staging_dir) {
            let _ = fs.remove_dir(&staging_dir);
        }
    };

    fs.checkpoint(FsOperation::CreatePrepareDirectory)?;
    fs.mkdir(&staging_dir)?;

    let images = [
        (
            txn.cfg_existed,
            "config.before",
            txn.cfg_before.as_slice(),
            FsOperation::WriteConfigBeforeImage,
        ),
        (
            txn.cred_existed,
            "credentials.before",
            txn.cred_before.as_slice(),
            FsOperation::WriteCredentialsBeforeImage,
        ),
        (
            txn.cfg_after_exists,
            "config.after",
            txn.cfg_after.as_slice(),
            FsOperation::WriteConfigAfterImage,
        ),
        (
            txn.cred_after_exists,
            "credentials.after",
            txn.cred_after.as_slice(),
            FsOperation::WriteCredentialsAfterImage,
        ),
    ];

    for (exists, file_name, bytes, operation) in images {
        if !exists {
            continue;
        }
        fs.checkpoint(operation)?;
        if let Err(error) = fs.write_secure(&staging_dir.join(file_name), bytes) {
            staging_cleanup(fs);
            return Err(error);
        }
    }

    fs.checkpoint(FsOperation::WritePreparedManifest)?;
    if let Err(error) = write_manifest(fs, &staging_dir, Phase::Prepared, txn_id, txn) {
        staging_cleanup(fs);
        return Err(error);
    }

    fs.checkpoint(FsOperation::SyncPrepareDirectory)?;
    if let Err(error) = fs.sync_dir(&staging_dir) {
        staging_cleanup(fs);
        return Err(error);
    }

    if fs.exists(active_dir) {
        staging_cleanup(fs);
        return Err(ConfigError::InvalidConfig(
            "active transaction directory already exists — recover pending transaction first"
                .into(),
        ));
    }

    fs.checkpoint(FsOperation::PublishActiveDirectory)?;
    fs.rename_dir(&staging_dir, active_dir)?;
    fs.checkpoint(FsOperation::SyncTransactionParentAfterPublish)?;
    fs.sync_dir(parent)?;
    Ok(())
}

pub(super) fn apply(
    fs: &dyn Fs,
    dir: &Path,
    config_path: &Path,
    credentials_path: &Path,
    txn: &Txn,
    txn_id: &str,
) -> Result<(), ConfigError> {
    fs.checkpoint(FsOperation::WriteApplyingManifest)?;
    write_manifest(fs, dir, Phase::Applying, txn_id, txn)?;

    fs.checkpoint(FsOperation::ReplaceConfigAfter)?;
    fs.atomic_write(config_path, &txn.cfg_after)?;

    if txn.cred_after_exists {
        fs.checkpoint(FsOperation::ReplaceCredentialsAfter)?;
        fs.atomic_write(credentials_path, &txn.cred_after)?;
    } else if fs.exists(credentials_path) {
        fs.checkpoint(FsOperation::RemoveCredentialsAfter)?;
        fs.remove_file(credentials_path)?;
        if let Some(parent) = credentials_path.parent() {
            fs.checkpoint(FsOperation::SyncCredentialsParentAfterRemove)?;
            fs.sync_dir(parent)?;
        }
    }

    verify_state(
        fs,
        config_path,
        credentials_path,
        Some(txn.cfg_after.as_slice()),
        if txn.cred_after_exists {
            Some(txn.cred_after.as_slice())
        } else {
            None
        },
        FsOperation::VerifyConfigAfter,
        FsOperation::VerifyCredentialsAfter,
        "apply",
    )?;

    fs.checkpoint(FsOperation::WriteCommittedManifest)?;
    write_manifest(fs, dir, Phase::Committed, txn_id, txn)?;
    Ok(())
}

pub(super) fn rollback(
    fs: &dyn Fs,
    dir: &Path,
    config_path: &Path,
    credentials_path: &Path,
    txn: &Txn,
    txn_id: &str,
) -> Result<(), ConfigError> {
    if let Err(error) = fs
        .checkpoint(FsOperation::WriteRollbackRequiredManifest)
        .and_then(|_| write_manifest(fs, dir, Phase::RollbackRequired, txn_id, txn))
    {
        tracing::warn!("failed to write RollbackRequired manifest: {error}");
    }

    fs.checkpoint(FsOperation::RestoreConfigBefore)?;
    restore(
        fs,
        config_path,
        if txn.cfg_existed {
            Some(txn.cfg_before.as_slice())
        } else {
            None
        },
        FsOperation::RemoveConfigForBeforeAbsence,
        FsOperation::SyncConfigParentAfterRollback,
    )?;

    fs.checkpoint(FsOperation::RestoreCredentialsBefore)?;
    restore(
        fs,
        credentials_path,
        if txn.cred_existed {
            Some(txn.cred_before.as_slice())
        } else {
            None
        },
        FsOperation::RemoveCredentialsForBeforeAbsence,
        FsOperation::SyncCredentialsParentAfterRollback,
    )?;

    verify_state(
        fs,
        config_path,
        credentials_path,
        if txn.cfg_existed {
            Some(txn.cfg_before.as_slice())
        } else {
            None
        },
        if txn.cred_existed {
            Some(txn.cred_before.as_slice())
        } else {
            None
        },
        FsOperation::VerifyConfigBefore,
        FsOperation::VerifyCredentialsBefore,
        "rollback",
    )?;

    fs.checkpoint(FsOperation::WriteRolledBackManifest)?;
    write_manifest(fs, dir, Phase::RolledBack, txn_id, txn)?;
    Ok(())
}

pub(super) fn restore(
    fs: &dyn Fs,
    path: &Path,
    before: Option<&[u8]>,
    remove_operation: FsOperation,
    sync_operation: FsOperation,
) -> Result<(), ConfigError> {
    match before {
        Some(bytes) => fs.atomic_write(path, bytes)?,
        None => {
            if fs.exists(path) {
                fs.checkpoint(remove_operation)?;
                fs.remove_file(path)?;
                if let Some(parent) = path.parent() {
                    fs.checkpoint(sync_operation)?;
                    fs.sync_dir(parent)?;
                }
            }
        }
    }
    Ok(())
}

fn write_cleanup_ready(fs: &dyn Fs, finalize_dir: &Path, txn_id: &str) -> Result<(), ConfigError> {
    fs.checkpoint(FsOperation::WriteCleanupReadyMarker)?;
    fs.write_secure(&finalize_dir.join(CLEANUP_READY_FILE), txn_id.as_bytes())?;
    fs.checkpoint(FsOperation::SyncFinalizeDirectory)?;
    fs.sync_dir(finalize_dir)
}

pub(super) fn finalize_active(
    fs: &dyn Fs,
    active_dir: &Path,
    txn_id: &str,
) -> Result<FinalizeOutcome, ConfigError> {
    if !fs.exists(active_dir) {
        return Ok(FinalizeOutcome::Finalized);
    }

    let parent = active_dir.parent().unwrap_or(Path::new("."));
    let finalize_dir = parent.join(format!("{FINALIZE_PREFIX}{txn_id}"));

    if fs.exists(&finalize_dir) {
        return Err(ambiguous_finalization_error());
    }

    if fs
        .checkpoint(FsOperation::PublishFinalizeDirectory)
        .and_then(|_| fs.rename_dir(active_dir, &finalize_dir))
        .is_err()
    {
        return Ok(FinalizeOutcome::ActiveRenamePending {
            transaction_id: txn_id.to_string(),
        });
    }

    if fs
        .checkpoint(FsOperation::SyncTransactionParentAfterFinalize)
        .and_then(|_| fs.sync_dir(parent))
        .is_err()
        || write_cleanup_ready(fs, &finalize_dir, txn_id).is_err()
    {
        return Ok(FinalizeOutcome::ParentSyncPending {
            transaction_id: txn_id.to_string(),
            finalize_dir,
        });
    }

    if fs
        .checkpoint(FsOperation::CleanupFinalizeDirectory)
        .and_then(|_| fs.remove_dir(&finalize_dir))
        .is_err()
    {
        return Ok(FinalizeOutcome::CleanupPending {
            transaction_id: txn_id.to_string(),
            finalize_dir,
        });
    }

    if fs
        .checkpoint(FsOperation::SyncTransactionParentAfterCleanup)
        .and_then(|_| fs.sync_dir(parent))
        .is_err()
    {
        return Ok(FinalizeOutcome::CleanupPending {
            transaction_id: txn_id.to_string(),
            finalize_dir,
        });
    }

    Ok(FinalizeOutcome::Finalized)
}
