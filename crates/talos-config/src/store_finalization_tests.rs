use crate::store::{Fs, FsOperation, RecoveryOutcome, RecoveryPurpose, StdFs};
use crate::{Config, ConfigError, ConfigStore, ConfigUnsetOutcome, Credentials, ProviderConfig};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::rc::Rc;

fn unique_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "talos-i157-finalization-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn make_store(dir: &Path) -> ConfigStore {
    ConfigStore::with_paths(dir.join("config.toml"), dir.join("credentials.toml"))
}

fn active_dir(dir: &Path) -> PathBuf {
    dir.join(".provider-unset-transaction")
}

fn finalize_dir(dir: &Path, transaction_id: &str) -> PathBuf {
    dir.join(format!(
        ".provider-unset-transaction.finalize.{transaction_id}"
    ))
}

fn prepare_entries(dir: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|name| name.starts_with(".provider-unset-transaction.prepare."))
        })
        .collect()
}

fn setup_two_providers(dir: &Path) -> (Vec<u8>, Vec<u8>) {
    let mut config = Config {
        provider: "custom-a".to_string(),
        model: "model-a".to_string(),
        ..Config::default()
    };
    config.providers.insert(
        "custom-a".to_string(),
        ProviderConfig {
            api_key: Some("sk-A".to_string()),
            ..ProviderConfig::default()
        },
    );
    config.providers.insert(
        "custom-b".to_string(),
        ProviderConfig {
            api_key: Some("sk-B".to_string()),
            ..ProviderConfig::default()
        },
    );

    let mut credentials = Credentials::default();
    credentials
        .keys
        .insert("custom-a".to_string(), "sk-CREDS-A".to_string());
    credentials
        .keys
        .insert("custom-b".to_string(), "sk-CREDS-B".to_string());

    let config_bytes = toml::to_string_pretty(&config).unwrap().into_bytes();
    let credential_bytes = toml::to_string_pretty(&credentials).unwrap().into_bytes();
    std::fs::write(dir.join("config.toml"), &config_bytes).unwrap();
    std::fs::write(dir.join("credentials.toml"), &credential_bytes).unwrap();
    (config_bytes, credential_bytes)
}

fn write_manifest(
    dir: &Path,
    phase: &str,
    transaction_id: &str,
    config_before: bool,
    config_after: bool,
    credentials_before: bool,
    credentials_after: bool,
) {
    let manifest = format!(
        "version = 1\nphase = \"{phase}\"\ntransaction_id = \"{transaction_id}\"\n\
         config_existed_before = {config_before}\nconfig_exists_after = {config_after}\n\
         credentials_existed_before = {credentials_before}\n\
         credentials_exist_after = {credentials_after}\n"
    );
    std::fs::write(dir.join("manifest"), manifest).unwrap();
}

fn assert_persisted_bytes(dir: &Path, config: &[u8], credentials: &[u8]) {
    assert_eq!(
        std::fs::read(dir.join("config.toml")).unwrap_or_default(),
        config
    );
    assert_eq!(
        std::fs::read(dir.join("credentials.toml")).unwrap_or_default(),
        credentials
    );
}

#[derive(Default)]
struct PlanState {
    failures: VecDeque<FsOperation>,
    observed: Vec<FsOperation>,
}

#[derive(Clone, Default)]
struct FaultPlan {
    state: Rc<RefCell<PlanState>>,
}

impl FaultPlan {
    fn fail_once(operation: FsOperation) -> Self {
        Self::fail_sequence(&[operation])
    }

    fn fail_sequence(operations: &[FsOperation]) -> Self {
        Self {
            state: Rc::new(RefCell::new(PlanState {
                failures: operations.iter().copied().collect(),
                observed: Vec::new(),
            })),
        }
    }

    fn checkpoint(&self, operation: FsOperation) -> Result<(), ConfigError> {
        let mut state = self.state.borrow_mut();
        state.observed.push(operation);
        if state.failures.front().copied() == Some(operation) {
            state.failures.pop_front();
            return Err(ConfigError::IoError(std::io::Error::other(
                "injected semantic filesystem failure",
            )));
        }
        Ok(())
    }

    fn assert_consumed_in_order(&self, expected: &[FsOperation]) {
        let state = self.state.borrow();
        assert!(
            state.failures.is_empty(),
            "planned failures were not consumed: {:?}",
            state.failures
        );

        let mut expected_index = 0;
        for operation in &state.observed {
            if expected.get(expected_index) == Some(operation) {
                expected_index += 1;
            }
        }
        assert_eq!(
            expected_index,
            expected.len(),
            "expected operations were not observed in order; observed: {:?}",
            state.observed
        );
    }

    fn observed(&self) -> Vec<FsOperation> {
        self.state.borrow().observed.clone()
    }
}

struct FaultFs {
    plan: FaultPlan,
}

impl FaultFs {
    fn new(plan: FaultPlan) -> Self {
        Self { plan }
    }
}

impl Fs for FaultFs {
    fn checkpoint(&self, operation: FsOperation) -> Result<(), ConfigError> {
        self.plan.checkpoint(operation)
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn read(&self, path: &Path) -> Result<Vec<u8>, ConfigError> {
        std::fs::read(path).map_err(ConfigError::IoError)
    }

    fn atomic_write(&self, path: &Path, content: &[u8]) -> Result<(), ConfigError> {
        crate::atomic_file::durable_replace(path, content)
    }

    fn write_secure(&self, path: &Path, content: &[u8]) -> Result<(), ConfigError> {
        crate::atomic_file::write_file_synced(path, content)
    }

    fn mkdir(&self, path: &Path) -> Result<(), ConfigError> {
        crate::atomic_file::create_dir_secure(path)
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
        crate::atomic_file::sync_dir(dir)
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

struct ConcurrentWriteFs {
    config_path: PathBuf,
    credentials_path: PathBuf,
    config_bytes: Vec<u8>,
    credential_bytes: Vec<u8>,
    fired: Cell<bool>,
}

impl ConcurrentWriteFs {
    fn new(
        config_path: PathBuf,
        credentials_path: PathBuf,
        config_bytes: Vec<u8>,
        credential_bytes: Vec<u8>,
    ) -> Self {
        Self {
            config_path,
            credentials_path,
            config_bytes,
            credential_bytes,
            fired: Cell::new(false),
        }
    }
}

impl Fs for ConcurrentWriteFs {
    fn checkpoint(&self, operation: FsOperation) -> Result<(), ConfigError> {
        if operation == FsOperation::VerifyConfigBeforeApply && !self.fired.replace(true) {
            crate::atomic_file::durable_replace(&self.config_path, &self.config_bytes)?;
            crate::atomic_file::durable_replace(&self.credentials_path, &self.credential_bytes)?;
        }
        Ok(())
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn read(&self, path: &Path) -> Result<Vec<u8>, ConfigError> {
        std::fs::read(path).map_err(ConfigError::IoError)
    }

    fn atomic_write(&self, path: &Path, content: &[u8]) -> Result<(), ConfigError> {
        crate::atomic_file::durable_replace(path, content)
    }

    fn write_secure(&self, path: &Path, content: &[u8]) -> Result<(), ConfigError> {
        crate::atomic_file::write_file_synced(path, content)
    }

    fn mkdir(&self, path: &Path) -> Result<(), ConfigError> {
        crate::atomic_file::create_dir_secure(path)
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
        crate::atomic_file::sync_dir(dir)
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

#[test]
fn stale_before_snapshot_is_aborted_without_overwriting_concurrent_state() {
    let dir = unique_dir("stale-before-cas");
    setup_two_providers(&dir);
    let store = make_store(&dir);

    let mut concurrent_config: Config =
        toml::from_str(&std::fs::read_to_string(dir.join("config.toml")).unwrap()).unwrap();
    concurrent_config.providers.remove("custom-b");
    let concurrent_config_bytes = toml::to_string_pretty(&concurrent_config)
        .unwrap()
        .into_bytes();

    let mut concurrent_credentials: Credentials =
        toml::from_str(&std::fs::read_to_string(dir.join("credentials.toml")).unwrap()).unwrap();
    concurrent_credentials.keys.remove("custom-b");
    let concurrent_credential_bytes = toml::to_string_pretty(&concurrent_credentials)
        .unwrap()
        .into_bytes();

    let fs = ConcurrentWriteFs::new(
        dir.join("config.toml"),
        dir.join("credentials.toml"),
        concurrent_config_bytes.clone(),
        concurrent_credential_bytes.clone(),
    );
    let error = store.run("providers.custom-a", &fs).unwrap_err();
    assert!(error.to_string().contains("changed concurrently"));
    assert_persisted_bytes(&dir, &concurrent_config_bytes, &concurrent_credential_bytes);
    assert!(!active_dir(&dir).exists());

    let loaded = store.load_effective().unwrap();
    assert!(loaded.providers.contains_key("custom-a"));
    assert!(!loaded.providers.contains_key("custom-b"));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn stale_prepared_journal_recovery_aborts_without_rollback() {
    let dir = unique_dir("stale-prepared-recovery");
    let (config_before, credentials_before) = setup_two_providers(&dir);
    let active = active_dir(&dir);
    std::fs::create_dir_all(&active).unwrap();
    std::fs::write(active.join("config.before"), &config_before).unwrap();
    std::fs::write(active.join("credentials.before"), &credentials_before).unwrap();
    write_manifest(
        &active,
        "Prepared",
        "stale-prepared",
        true,
        true,
        true,
        true,
    );

    let mut newer_config: Config =
        toml::from_str(&String::from_utf8(config_before.clone()).unwrap()).unwrap();
    newer_config.providers.remove("custom-b");
    let newer_config = toml::to_string_pretty(&newer_config).unwrap().into_bytes();
    let mut newer_credentials: Credentials =
        toml::from_str(&String::from_utf8(credentials_before.clone()).unwrap()).unwrap();
    newer_credentials.keys.remove("custom-b");
    let newer_credentials = toml::to_string_pretty(&newer_credentials)
        .unwrap()
        .into_bytes();
    std::fs::write(dir.join("config.toml"), &newer_config).unwrap();
    std::fs::write(dir.join("credentials.toml"), &newer_credentials).unwrap();

    make_store(&dir).recover(&StdFs).unwrap();
    assert_persisted_bytes(&dir, &newer_config, &newer_credentials);
    assert!(!active.exists());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cleanup_ready_marker_is_resynced_before_mutation_becomes_safe() {
    let dir = unique_dir("cleanup-marker-resync");
    setup_two_providers(&dir);
    let store = make_store(&dir);

    let first_plan = FaultPlan::fail_once(FsOperation::SyncFinalizeDirectory);
    store
        .run("providers.custom-a", &FaultFs::new(first_plan.clone()))
        .unwrap();
    first_plan.assert_consumed_in_order(&[FsOperation::SyncFinalizeDirectory]);

    let second_plan = FaultPlan::fail_once(FsOperation::SyncFinalizeDirectory);
    let error = store
        .run("providers.custom-b", &FaultFs::new(second_plan.clone()))
        .unwrap_err();
    assert!(error.to_string().contains("finalization is pending"));
    second_plan.assert_consumed_in_order(&[FsOperation::SyncFinalizeDirectory]);
    assert!(prepare_entries(&dir).is_empty());

    let third_plan = FaultPlan::fail_once(FsOperation::CleanupFinalizeDirectory);
    let outcome = store
        .run("providers.custom-b", &FaultFs::new(third_plan.clone()))
        .unwrap();
    assert_eq!(
        outcome,
        ConfigUnsetOutcome::CustomProviderRemoved {
            name: "custom-b".to_string()
        }
    );
    third_plan.assert_consumed_in_order(&[FsOperation::CleanupFinalizeDirectory]);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn foreign_prepare_directory_is_never_deleted_automatically() {
    let dir = unique_dir("foreign-prepare");
    setup_two_providers(&dir);
    let prepare = dir.join(".provider-unset-transaction.prepare.foreign-process");
    std::fs::create_dir_all(&prepare).unwrap();
    std::fs::write(prepare.join("owner"), b"other-process").unwrap();

    make_store(&dir).load_effective().unwrap();
    assert_eq!(
        std::fs::read(prepare.join("owner")).unwrap(),
        b"other-process"
    );
    assert!(prepare.exists());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn ordered_apply_rollback_and_recovery_failures_preserve_journal() {
    let dir = unique_dir("ordered-composite");
    let (config_before, credentials_before) = setup_two_providers(&dir);
    let store = make_store(&dir);

    let first_plan = FaultPlan::fail_sequence(&[
        FsOperation::ReplaceCredentialsAfter,
        FsOperation::RestoreConfigBefore,
    ]);
    let first_fs = FaultFs::new(first_plan.clone());
    let result = store.run("providers.custom-a", &first_fs);
    assert!(result.is_err());
    assert!(active_dir(&dir).exists());
    first_plan.assert_consumed_in_order(&[
        FsOperation::ReplaceCredentialsAfter,
        FsOperation::RestoreConfigBefore,
    ]);

    let second_plan = FaultPlan::fail_once(FsOperation::RestoreCredentialsBefore);
    let second_fs = FaultFs::new(second_plan.clone());
    let result = store.recover(&second_fs);
    assert!(result.is_err());
    assert!(active_dir(&dir).exists());
    assert_eq!(
        std::fs::read(dir.join("config.toml")).unwrap(),
        config_before
    );
    second_plan.assert_consumed_in_order(&[FsOperation::RestoreCredentialsBefore]);

    store.recover(&StdFs).unwrap();
    assert!(!active_dir(&dir).exists());
    assert_persisted_bytes(&dir, &config_before, &credentials_before);

    store.recover(&StdFs).unwrap();
    assert_persisted_bytes(&dir, &config_before, &credentials_before);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn parent_sync_pending_blocks_mutation_before_prepare_and_retries() {
    let dir = unique_dir("parent-sync-pending");
    setup_two_providers(&dir);
    let store = make_store(&dir);

    let finalize_plan = FaultPlan::fail_once(FsOperation::SyncTransactionParentAfterFinalize);
    let finalize_fs = FaultFs::new(finalize_plan.clone());
    let outcome = store.run("providers.custom-a", &finalize_fs).unwrap();
    assert_eq!(
        outcome,
        ConfigUnsetOutcome::CustomProviderRemoved {
            name: "custom-a".to_string()
        }
    );
    finalize_plan.assert_consumed_in_order(&[FsOperation::SyncTransactionParentAfterFinalize]);
    assert!(!active_dir(&dir).exists());

    let residues: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|name| name.starts_with(".provider-unset-transaction.finalize."))
        })
        .collect();
    assert_eq!(residues.len(), 1);

    let retry_plan = FaultPlan::fail_once(FsOperation::SyncFinalizeResidueParent);
    let retry_fs = FaultFs::new(retry_plan.clone());
    let error = store.run("providers.custom-b", &retry_fs).unwrap_err();
    assert!(error.to_string().contains("finalization is pending"));
    retry_plan.assert_consumed_in_order(&[FsOperation::SyncFinalizeResidueParent]);
    assert!(prepare_entries(&dir).is_empty());
    assert!(
        !retry_plan
            .observed()
            .contains(&FsOperation::CreatePrepareDirectory)
    );

    let second_outcome = store.unset_provider("providers.custom-b").unwrap();
    assert_eq!(
        second_outcome,
        ConfigUnsetOutcome::CustomProviderRemoved {
            name: "custom-b".to_string()
        }
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cleanup_pending_is_load_safe_and_does_not_block_next_mutation() {
    let dir = unique_dir("cleanup-pending");
    setup_two_providers(&dir);
    let store = make_store(&dir);

    let first_plan = FaultPlan::fail_once(FsOperation::CleanupFinalizeDirectory);
    let first_fs = FaultFs::new(first_plan.clone());
    store.run("providers.custom-a", &first_fs).unwrap();
    first_plan.assert_consumed_in_order(&[FsOperation::CleanupFinalizeDirectory]);

    let second_plan = FaultPlan::fail_once(FsOperation::CleanupFinalizeDirectory);
    let second_fs = FaultFs::new(second_plan.clone());
    let outcome = store.run("providers.custom-b", &second_fs).unwrap();
    assert_eq!(
        outcome,
        ConfigUnsetOutcome::CustomProviderRemoved {
            name: "custom-b".to_string()
        }
    );
    second_plan.assert_consumed_in_order(&[FsOperation::CleanupFinalizeDirectory]);
    assert!(prepare_entries(&dir).is_empty());

    let loaded = store.load_effective().unwrap();
    assert!(!loaded.providers.contains_key("custom-a"));
    assert!(!loaded.providers.contains_key("custom-b"));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn ambiguous_active_and_finalize_evidence_fails_closed_and_is_preserved() {
    let dir = unique_dir("ambiguous");
    let (config, credentials) = setup_two_providers(&dir);
    let transaction_id = "ambiguous-1";

    let active = active_dir(&dir);
    let finalize = finalize_dir(&dir, transaction_id);
    std::fs::create_dir_all(&active).unwrap();
    std::fs::create_dir_all(&finalize).unwrap();

    for journal in [&active, &finalize] {
        std::fs::write(journal.join("config.after"), &config).unwrap();
        std::fs::write(journal.join("credentials.after"), &credentials).unwrap();
        write_manifest(journal, "Committed", transaction_id, true, true, true, true);
    }

    let store = make_store(&dir);
    let error = store.load_effective().unwrap_err();
    assert!(error.to_string().contains("ambiguous"));
    assert!(active.exists());
    assert!(finalize.exists());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn valid_finalize_residue_is_verified_synced_and_cleaned() {
    let dir = unique_dir("valid-residue");
    let (config, credentials) = setup_two_providers(&dir);
    let transaction_id = "residue-1";
    let finalize = finalize_dir(&dir, transaction_id);
    std::fs::create_dir_all(&finalize).unwrap();
    std::fs::write(finalize.join("config.after"), &config).unwrap();
    std::fs::write(finalize.join("credentials.after"), &credentials).unwrap();
    write_manifest(
        &finalize,
        "Committed",
        transaction_id,
        true,
        true,
        true,
        true,
    );

    let store = make_store(&dir);
    let outcome = store
        .recover_with_purpose(&StdFs, RecoveryPurpose::Mutation)
        .unwrap();
    assert!(matches!(outcome, RecoveryOutcome::Clean));
    assert!(!finalize.exists());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn malformed_finalize_residue_fails_closed_without_leaking_marker() {
    let dir = unique_dir("malformed-residue");
    setup_two_providers(&dir);
    let finalize = finalize_dir(&dir, "broken-1");
    std::fs::create_dir_all(&finalize).unwrap();
    std::fs::write(
        finalize.join("manifest"),
        "marker = \"sk-FINALIZE-MARKER\"\nbroken = [",
    )
    .unwrap();

    let store = make_store(&dir);
    let error = store.load_effective().unwrap_err();
    let display = error.to_string();
    let debug = format!("{error:?}");
    assert!(!display.contains("FINALIZE-MARKER"));
    assert!(!debug.contains("FINALIZE-MARKER"));
    assert!(finalize.exists());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn rollback_absence_parent_sync_failure_retains_recoverable_journal() {
    let dir = unique_dir("absence-parent-sync");
    let mut credentials = Credentials::default();
    credentials
        .keys
        .insert("credentials-only".to_string(), "sk-ONLY".to_string());
    let credentials_before = toml::to_string_pretty(&credentials).unwrap().into_bytes();
    std::fs::write(dir.join("credentials.toml"), &credentials_before).unwrap();

    let store = make_store(&dir);
    let plan = FaultPlan::fail_sequence(&[
        FsOperation::WriteCommittedManifest,
        FsOperation::SyncConfigParentAfterRollback,
    ]);
    let fault_fs = FaultFs::new(plan.clone());
    let result = store.run("providers.credentials-only", &fault_fs);
    assert!(result.is_err());
    plan.assert_consumed_in_order(&[
        FsOperation::WriteCommittedManifest,
        FsOperation::SyncConfigParentAfterRollback,
    ]);
    assert!(active_dir(&dir).exists());

    store.recover(&StdFs).unwrap();
    assert!(!dir.join("config.toml").exists());
    assert_eq!(
        std::fs::read(dir.join("credentials.toml")).unwrap(),
        credentials_before
    );
    assert!(!active_dir(&dir).exists());
    let _ = std::fs::remove_dir_all(dir);
}

#[cfg(unix)]
#[test]
fn finalize_symlink_residue_is_not_followed_or_deleted() {
    use std::os::unix::fs::symlink;

    let dir = unique_dir("symlink-residue");
    setup_two_providers(&dir);
    let outside = unique_dir("symlink-outside");
    std::fs::write(outside.join("keep"), b"do-not-delete").unwrap();

    let link = finalize_dir(&dir, "symlink-1");
    symlink(&outside, &link).unwrap();

    let store = make_store(&dir);
    let error = store.load_effective().unwrap_err();
    assert!(error.to_string().contains("not a real directory"));
    assert_eq!(
        std::fs::read(outside.join("keep")).unwrap(),
        b"do-not-delete"
    );
    assert!(link.symlink_metadata().is_ok());

    let _ = std::fs::remove_file(link);
    let _ = std::fs::remove_dir_all(dir);
    let _ = std::fs::remove_dir_all(outside);
}
