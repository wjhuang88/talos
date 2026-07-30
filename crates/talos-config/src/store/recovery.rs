use super::journal::{
    CLEANUP_READY_FILE, FinalizeOutcome, Manifest, Phase, TerminalPhase as JournalTerminalPhase,
    Txn, finalize_active, parse_manifest, read_image, restore, validate_terminal_state,
    verify_state, write_manifest,
};
use super::{
    ConfigStore, FINALIZE_PREFIX, Fs, FsOperation, PREPARE_PREFIX, ambiguous_finalization_error,
    before_state_matches,
};
use crate::ConfigError;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecoveryPurpose {
    Load,
    Mutation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TerminalPhase {
    Committed,
    RolledBack,
    Aborted,
}

impl From<TerminalPhase> for JournalTerminalPhase {
    fn from(value: TerminalPhase) -> Self {
        match value {
            TerminalPhase::Committed => Self::Committed,
            TerminalPhase::RolledBack => Self::RolledBack,
            TerminalPhase::Aborted => Self::Aborted,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RecoveryOutcome {
    Clean,
    TerminalActivePending {
        transaction_id: String,
        phase: TerminalPhase,
    },
    ParentSyncPending {
        transaction_id: String,
        phase: TerminalPhase,
        finalize_dir: PathBuf,
    },
    CleanupPending {
        transaction_id: String,
        phase: TerminalPhase,
        finalize_dir: PathBuf,
    },
}

impl RecoveryOutcome {
    pub(crate) fn allows_mutation(&self) -> bool {
        matches!(self, Self::Clean | Self::CleanupPending { .. })
    }
}

impl ConfigStore {
    pub(crate) fn recover(&self, fs: &dyn Fs) -> Result<(), ConfigError> {
        self.recover_with_purpose(fs, RecoveryPurpose::Load)
            .map(|_| ())
    }

    pub(crate) fn recover_with_purpose(
        &self,
        fs: &dyn Fs,
        purpose: RecoveryPurpose,
    ) -> Result<RecoveryOutcome, ConfigError> {
        let parent = self.txn_dir_parent();
        self.cleanup_prepare_residues(fs, &parent)?;

        let active_dir = self.txn_dir();
        let mut active_outcome = RecoveryOutcome::Clean;

        if fs.exists(&active_dir) {
            active_outcome = self.recover_active(fs, &active_dir)?;
            if matches!(
                active_outcome,
                RecoveryOutcome::TerminalActivePending { .. }
                    | RecoveryOutcome::ParentSyncPending { .. }
            ) {
                return Ok(active_outcome);
            }
        }

        let residue_outcome = self.recover_finalize_residues(fs, &parent)?;
        let outcome = merge_recovery_outcomes(active_outcome, residue_outcome);

        if matches!(purpose, RecoveryPurpose::Mutation) && !outcome.allows_mutation() {
            tracing::debug!("provider configuration mutation blocked by pending finalization");
        }

        Ok(outcome)
    }

    fn recover_active(
        &self,
        fs: &dyn Fs,
        active_dir: &Path,
    ) -> Result<RecoveryOutcome, ConfigError> {
        let manifest_path = active_dir.join("manifest");
        if !fs.exists(&manifest_path) {
            return Err(ConfigError::InvalidConfig(
                "pending transaction directory exists without manifest — manual recovery required"
                    .into(),
            ));
        }

        fs.checkpoint(FsOperation::ReadActiveManifest)?;
        let manifest = parse_manifest(&fs.read(&manifest_path)?)?;

        let same_id_finalize = self.finalize_dir(&manifest.transaction_id);
        if fs.exists(&same_id_finalize) {
            return Err(ambiguous_finalization_error());
        }

        match manifest.phase {
            Phase::Committed => {
                validate_terminal_state(
                    fs,
                    active_dir,
                    &self.config_path,
                    &self.credentials_path,
                    &manifest,
                    JournalTerminalPhase::Committed,
                )?;
                Ok(map_finalize_outcome(
                    finalize_active(fs, active_dir, &manifest.transaction_id)?,
                    TerminalPhase::Committed,
                ))
            }
            Phase::Prepared => {
                let cfg_before = read_image(
                    fs,
                    active_dir,
                    "config.before",
                    manifest.config_existed_before,
                    FsOperation::ReadConfigBeforeImage,
                )?;
                let cred_before = read_image(
                    fs,
                    active_dir,
                    "credentials.before",
                    manifest.credentials_existed_before,
                    FsOperation::ReadCredentialsBeforeImage,
                )?;
                let txn = Txn {
                    cfg_existed: manifest.config_existed_before,
                    cfg_after_exists: manifest.config_exists_after,
                    cred_existed: manifest.credentials_existed_before,
                    cred_after_exists: manifest.credentials_exist_after,
                    cfg_before: cfg_before.unwrap_or_default(),
                    cfg_after: Vec::new(),
                    cred_before: cred_before.unwrap_or_default(),
                    cred_after: Vec::new(),
                };

                if !before_state_matches(fs, &self.config_path, &self.credentials_path, &txn)? {
                    fs.checkpoint(FsOperation::WriteAbortedManifest)?;
                    write_manifest(
                        fs,
                        active_dir,
                        Phase::Aborted,
                        &manifest.transaction_id,
                        &txn,
                    )?;
                    return Ok(map_finalize_outcome(
                        finalize_active(fs, active_dir, &manifest.transaction_id)?,
                        TerminalPhase::Aborted,
                    ));
                }

                recover_before_state(fs, self, active_dir, &manifest, txn)
            }
            Phase::Applying | Phase::RollbackRequired => {
                let cfg_before = read_image(
                    fs,
                    active_dir,
                    "config.before",
                    manifest.config_existed_before,
                    FsOperation::ReadConfigBeforeImage,
                )?;
                let cred_before = read_image(
                    fs,
                    active_dir,
                    "credentials.before",
                    manifest.credentials_existed_before,
                    FsOperation::ReadCredentialsBeforeImage,
                )?;
                recover_before_state(
                    fs,
                    self,
                    active_dir,
                    &manifest,
                    Txn {
                        cfg_existed: manifest.config_existed_before,
                        cfg_after_exists: manifest.config_exists_after,
                        cred_existed: manifest.credentials_existed_before,
                        cred_after_exists: manifest.credentials_exist_after,
                        cfg_before: cfg_before.unwrap_or_default(),
                        cfg_after: Vec::new(),
                        cred_before: cred_before.unwrap_or_default(),
                        cred_after: Vec::new(),
                    },
                )
            }
            Phase::Aborted => Ok(map_finalize_outcome(
                finalize_active(fs, active_dir, &manifest.transaction_id)?,
                TerminalPhase::Aborted,
            )),
            Phase::RolledBack => {
                validate_terminal_state(
                    fs,
                    active_dir,
                    &self.config_path,
                    &self.credentials_path,
                    &manifest,
                    JournalTerminalPhase::RolledBack,
                )?;
                Ok(map_finalize_outcome(
                    finalize_active(fs, active_dir, &manifest.transaction_id)?,
                    TerminalPhase::RolledBack,
                ))
            }
        }
    }

    fn recover_finalize_residues(
        &self,
        fs: &dyn Fs,
        parent: &Path,
    ) -> Result<RecoveryOutcome, ConfigError> {
        if !fs.exists(parent) {
            return Ok(RecoveryOutcome::Clean);
        }

        let mut outcome = RecoveryOutcome::Clean;

        for entry in fs.list_dir(parent)? {
            let Some(file_name) = entry.file_name() else {
                continue;
            };
            let Some(name) = file_name.to_str() else {
                continue;
            };
            let Some(transaction_id) = name.strip_prefix(FINALIZE_PREFIX) else {
                continue;
            };
            if transaction_id.is_empty() {
                return Err(ConfigError::InvalidConfig(
                    "finalization residue has an invalid transaction identifier".into(),
                ));
            }

            if !fs.is_real_dir(&entry)? {
                return Err(ConfigError::InvalidConfig(
                    "finalization residue is not a real directory".into(),
                ));
            }

            let cleanup_ready = entry.join(CLEANUP_READY_FILE);
            if fs.exists(&cleanup_ready) {
                fs.checkpoint(FsOperation::ReadCleanupReadyMarker)?;
                let marker = fs.read(&cleanup_ready)?;
                if marker != transaction_id.as_bytes() {
                    return Err(ConfigError::InvalidConfig(
                        "finalization cleanup marker transaction identifier mismatch".into(),
                    ));
                }
                if fs
                    .checkpoint(FsOperation::SyncFinalizeDirectory)
                    .and_then(|_| fs.sync_dir(&entry))
                    .is_err()
                {
                    return Ok(RecoveryOutcome::ParentSyncPending {
                        transaction_id: transaction_id.to_string(),
                        phase: TerminalPhase::Committed,
                        finalize_dir: entry,
                    });
                }
                outcome = merge_recovery_outcomes(
                    outcome,
                    cleanup_ready_residue(fs, parent, &entry, transaction_id, None),
                );
                continue;
            }

            let manifest_path = entry.join("manifest");
            if !fs.exists(&manifest_path) {
                return Err(ConfigError::InvalidConfig(
                    "finalization residue exists without manifest".into(),
                ));
            }

            fs.checkpoint(FsOperation::ReadFinalizeManifest)?;
            let manifest = parse_manifest(&fs.read(&manifest_path)?)?;
            if manifest.transaction_id != transaction_id {
                return Err(ConfigError::InvalidConfig(
                    "finalization residue transaction identifier mismatch".into(),
                ));
            }

            let terminal_phase = terminal_phase(&manifest)?;
            validate_terminal_state(
                fs,
                &entry,
                &self.config_path,
                &self.credentials_path,
                &manifest,
                terminal_phase.into(),
            )?;

            if fs
                .checkpoint(FsOperation::SyncFinalizeResidueParent)
                .and_then(|_| fs.sync_dir(parent))
                .is_err()
                || fs
                    .checkpoint(FsOperation::WriteCleanupReadyMarker)
                    .and_then(|_| fs.write_secure(&cleanup_ready, transaction_id.as_bytes()))
                    .is_err()
                || fs
                    .checkpoint(FsOperation::SyncFinalizeDirectory)
                    .and_then(|_| fs.sync_dir(&entry))
                    .is_err()
            {
                return Ok(RecoveryOutcome::ParentSyncPending {
                    transaction_id: transaction_id.to_string(),
                    phase: terminal_phase,
                    finalize_dir: entry,
                });
            }

            outcome = merge_recovery_outcomes(
                outcome,
                cleanup_ready_residue(fs, parent, &entry, transaction_id, Some(terminal_phase)),
            );
        }

        Ok(outcome)
    }

    fn cleanup_prepare_residues(&self, fs: &dyn Fs, parent: &Path) -> Result<(), ConfigError> {
        if !fs.exists(parent) {
            return Ok(());
        }

        for entry in fs.list_dir(parent)? {
            let Some(file_name) = entry.file_name() else {
                continue;
            };
            let Some(name) = file_name.to_str() else {
                continue;
            };
            if !name.starts_with(PREPARE_PREFIX) {
                continue;
            }

            // Preparation directories are unpublished but may belong to a live
            // concurrent process. Never delete an unowned preparation directory.
            // A future lease/owner-token design may add bounded garbage collection.
            let _ = fs.is_real_dir(&entry)?;
        }

        Ok(())
    }
}

fn recover_before_state(
    fs: &dyn Fs,
    store: &ConfigStore,
    active_dir: &Path,
    manifest: &Manifest,
    txn: Txn,
) -> Result<RecoveryOutcome, ConfigError> {
    fs.checkpoint(FsOperation::RestoreConfigBefore)?;
    restore(
        fs,
        &store.config_path,
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
        &store.credentials_path,
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
        &store.config_path,
        &store.credentials_path,
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
        "recovery",
    )?;

    fs.checkpoint(FsOperation::WriteRolledBackManifest)?;
    write_manifest(
        fs,
        active_dir,
        Phase::RolledBack,
        &manifest.transaction_id,
        &txn,
    )?;

    Ok(map_finalize_outcome(
        finalize_active(fs, active_dir, &manifest.transaction_id)?,
        TerminalPhase::RolledBack,
    ))
}

fn cleanup_ready_residue(
    fs: &dyn Fs,
    parent: &Path,
    entry: &Path,
    transaction_id: &str,
    phase: Option<TerminalPhase>,
) -> RecoveryOutcome {
    let phase = phase.unwrap_or(TerminalPhase::Committed);
    if fs
        .checkpoint(FsOperation::CleanupFinalizeDirectory)
        .and_then(|_| fs.remove_dir(entry))
        .is_err()
    {
        return RecoveryOutcome::CleanupPending {
            transaction_id: transaction_id.to_string(),
            phase,
            finalize_dir: entry.to_path_buf(),
        };
    }

    if fs
        .checkpoint(FsOperation::SyncTransactionParentAfterCleanup)
        .and_then(|_| fs.sync_dir(parent))
        .is_err()
    {
        return RecoveryOutcome::CleanupPending {
            transaction_id: transaction_id.to_string(),
            phase,
            finalize_dir: entry.to_path_buf(),
        };
    }

    RecoveryOutcome::Clean
}

fn terminal_phase(manifest: &Manifest) -> Result<TerminalPhase, ConfigError> {
    match manifest.phase {
        Phase::Committed => Ok(TerminalPhase::Committed),
        Phase::RolledBack => Ok(TerminalPhase::RolledBack),
        Phase::Aborted => Ok(TerminalPhase::Aborted),
        _ => Err(ConfigError::InvalidConfig(
            "finalization residue contains a non-terminal manifest".into(),
        )),
    }
}

fn map_finalize_outcome(outcome: FinalizeOutcome, phase: TerminalPhase) -> RecoveryOutcome {
    match outcome {
        FinalizeOutcome::Finalized => RecoveryOutcome::Clean,
        FinalizeOutcome::ActiveRenamePending { transaction_id } => {
            RecoveryOutcome::TerminalActivePending {
                transaction_id,
                phase,
            }
        }
        FinalizeOutcome::ParentSyncPending {
            transaction_id,
            finalize_dir,
        } => RecoveryOutcome::ParentSyncPending {
            transaction_id,
            phase,
            finalize_dir,
        },
        FinalizeOutcome::CleanupPending {
            transaction_id,
            finalize_dir,
        } => RecoveryOutcome::CleanupPending {
            transaction_id,
            phase,
            finalize_dir,
        },
    }
}

fn merge_recovery_outcomes(left: RecoveryOutcome, right: RecoveryOutcome) -> RecoveryOutcome {
    use RecoveryOutcome::{Clean, CleanupPending, ParentSyncPending, TerminalActivePending};

    match (left, right) {
        (
            TerminalActivePending {
                transaction_id,
                phase,
            },
            _,
        ) => TerminalActivePending {
            transaction_id,
            phase,
        },
        (
            _,
            TerminalActivePending {
                transaction_id,
                phase,
            },
        ) => TerminalActivePending {
            transaction_id,
            phase,
        },
        (
            ParentSyncPending {
                transaction_id,
                phase,
                finalize_dir,
            },
            _,
        ) => ParentSyncPending {
            transaction_id,
            phase,
            finalize_dir,
        },
        (
            _,
            ParentSyncPending {
                transaction_id,
                phase,
                finalize_dir,
            },
        ) => ParentSyncPending {
            transaction_id,
            phase,
            finalize_dir,
        },
        (
            CleanupPending {
                transaction_id,
                phase,
                finalize_dir,
            },
            _,
        ) => CleanupPending {
            transaction_id,
            phase,
            finalize_dir,
        },
        (
            _,
            CleanupPending {
                transaction_id,
                phase,
                finalize_dir,
            },
        ) => CleanupPending {
            transaction_id,
            phase,
            finalize_dir,
        },
        (Clean, Clean) => Clean,
    }
}
