//! Two-file transaction coordinator for provider removal (MODEL-010).

mod fs;
mod journal;
mod recovery;

use crate::{
    Config, ConfigError, ConfigUnsetOutcome, Credentials, builtin_provider_config, home_dir,
};
use journal::{
    FinalizeOutcome, Phase, Txn, apply, finalize_active, gen_txn_id, prepare, rollback,
    write_manifest,
};
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

pub(crate) use fs::{Fs, FsOperation, StdFs};
#[cfg(test)]
pub(crate) use recovery::RecoveryOutcome;
pub(crate) use recovery::RecoveryPurpose;

pub(crate) const ACTIVE_DIR_NAME: &str = ".provider-unset-transaction";
pub(crate) const PREPARE_PREFIX: &str = ".provider-unset-transaction.prepare.";
pub(crate) const FINALIZE_PREFIX: &str = ".provider-unset-transaction.finalize.";
const MUTATION_LOCK_NAME: &str = ".config-mutation.lock";

struct MutationLock {
    _file: File,
}

pub struct ConfigStore {
    pub(super) config_path: PathBuf,
    pub(super) credentials_path: PathBuf,
}

impl ConfigStore {
    #[must_use]
    pub fn default_store() -> Self {
        let mut dir = home_dir();
        dir.push(".talos");
        Self {
            config_path: dir.join("config.toml"),
            credentials_path: dir.join("credentials.toml"),
        }
    }

    #[must_use]
    pub fn with_paths(config_path: PathBuf, credentials_path: PathBuf) -> Self {
        Self {
            config_path,
            credentials_path,
        }
    }

    pub fn unset_provider(&self, key: &str) -> Result<ConfigUnsetOutcome, ConfigError> {
        self.run(key, &StdFs)
    }

    pub(crate) fn run(&self, key: &str, fs: &dyn Fs) -> Result<ConfigUnsetOutcome, ConfigError> {
        let _lock = self.acquire_mutation_lock()?;
        let recovery = self.recover_with_purpose(fs, RecoveryPurpose::Mutation)?;
        if !recovery.allows_mutation() {
            return Err(finalization_pending_error());
        }

        // Mutation readiness is decided before either persisted document is read
        // and before a preparation directory is created.
        let cfg_before = fs.read_opt(&self.config_path)?;
        let cred_before = fs.read_opt(&self.credentials_path)?;

        let mut config: Config = parse_optional_persisted(&cfg_before, PersistedDocument::Config)?;
        let mut creds: Credentials =
            parse_optional_persisted(&cred_before, PersistedDocument::Credentials)?;

        let outcome = mutate(&mut config, &mut creds, key)?;

        let cfg_after = serialize(&config)?;
        reparse::<Config>(&cfg_after)?;

        let cred_after_exists = !creds.keys.is_empty();
        let cred_after = if cred_after_exists {
            let serialized = serialize(&creds)?;
            reparse::<Credentials>(&serialized)?;
            serialized
        } else {
            Vec::new()
        };

        let txn = Txn {
            cfg_existed: cfg_before.is_some(),
            cfg_after_exists: true,
            cred_existed: cred_before.is_some(),
            cred_after_exists,
            cfg_before: cfg_before.unwrap_or_default(),
            cfg_after,
            cred_before: cred_before.unwrap_or_default(),
            cred_after,
        };

        let active_dir = self.txn_dir();
        let txn_id = gen_txn_id();

        prepare(fs, &active_dir, &txn, &txn_id)?;

        if !before_state_matches(fs, &self.config_path, &self.credentials_path, &txn)? {
            fs.checkpoint(FsOperation::WriteAbortedManifest)?;
            write_manifest(fs, &active_dir, Phase::Aborted, &txn_id, &txn)?;
            match finalize_active(fs, &active_dir, &txn_id) {
                Ok(FinalizeOutcome::Finalized) => {}
                Ok(pending) => log_finalize_pending("concurrent-abort", &pending),
                Err(error) => tracing::warn!(
                    error = %error,
                    "ambiguous provider configuration finalization after concurrent abort"
                ),
            }
            return Err(concurrent_mutation_error());
        }

        if let Err(apply_err) = apply(
            fs,
            &active_dir,
            &self.config_path,
            &self.credentials_path,
            &txn,
            &txn_id,
        ) {
            match rollback(
                fs,
                &active_dir,
                &self.config_path,
                &self.credentials_path,
                &txn,
                &txn_id,
            ) {
                Ok(()) => {
                    match finalize_active(fs, &active_dir, &txn_id) {
                        Ok(FinalizeOutcome::Finalized) => {}
                        Ok(pending) => log_finalize_pending("rollback", &pending),
                        Err(error) => tracing::warn!(
                            error = %error,
                            "ambiguous provider configuration finalization after rollback"
                        ),
                    }
                    return Err(apply_err);
                }
                Err(_) => {
                    return Err(ConfigError::InvalidConfig(
                        "recovery required: rollback failed after apply error".into(),
                    ));
                }
            }
        }

        match finalize_active(fs, &active_dir, &txn_id) {
            Ok(FinalizeOutcome::Finalized) => {}
            Ok(pending) => log_finalize_pending("committed", &pending),
            Err(error) => tracing::warn!(
                error = %error,
                "ambiguous provider configuration finalization after commit"
            ),
        }

        Ok(outcome)
    }

    pub fn recover_pending() -> Result<(), ConfigError> {
        let store = Self::default_store();
        let _lock = store.acquire_mutation_lock()?;
        store.recover(&StdFs)
    }

    /// Applies one semantic configuration change to the latest persisted state.
    ///
    /// The mutation lock covers recovery, reading, mutation, validation of the
    /// serialized representation, and durable replacement. Callers should keep
    /// `update` short and must not perform user interaction or network I/O in
    /// the closure.
    ///
    /// Unlike [`Config::save`], this API does not write a caller-owned snapshot
    /// back wholesale. It reloads `config.toml` after acquiring the same lock
    /// used by provider removal, preventing an older interactive snapshot from
    /// restoring a provider or credential that another process removed.
    ///
    /// # Errors
    ///
    /// Returns an error if recovery is incomplete, the persisted configuration
    /// cannot be parsed, the callback rejects the change, serialization fails,
    /// or the durable replacement cannot be completed.
    pub fn update_config(
        &self,
        update: impl FnOnce(&mut Config) -> Result<(), ConfigError>,
    ) -> Result<Config, ConfigError> {
        let _lock = self.acquire_mutation_lock()?;
        let recovery = self.recover_with_purpose(&StdFs, RecoveryPurpose::Mutation)?;
        if !recovery.allows_mutation() {
            return Err(finalization_pending_error());
        }

        let persisted = StdFs.read_opt(&self.config_path)?;
        let mut config: Config = parse_optional_persisted(&persisted, PersistedDocument::Config)?;
        update(&mut config)?;

        let serialized = serialize(&config)?;
        reparse::<Config>(&serialized)?;
        StdFs.atomic_write(&self.config_path, &serialized)?;

        self.load_effective_unlocked()
    }

    pub(crate) fn replace_config_snapshot(&self, config: &Config) -> Result<(), ConfigError> {
        let _lock = self.acquire_mutation_lock()?;
        let recovery = self.recover_with_purpose(&StdFs, RecoveryPurpose::Mutation)?;
        if !recovery.allows_mutation() {
            return Err(finalization_pending_error());
        }
        let serialized = serialize(config)?;
        reparse::<Config>(&serialized)?;
        StdFs.atomic_write(&self.config_path, &serialized)
    }

    pub(crate) fn load_credentials_snapshot(&self) -> Result<Credentials, ConfigError> {
        let _lock = self.acquire_mutation_lock()?;
        let _ = self.recover_with_purpose(&StdFs, RecoveryPurpose::Load)?;
        let persisted = StdFs.read_opt(&self.credentials_path)?;
        parse_optional_persisted(&persisted, PersistedDocument::Credentials)
    }

    pub(crate) fn replace_credentials_snapshot(
        &self,
        credentials: &Credentials,
    ) -> Result<(), ConfigError> {
        let _lock = self.acquire_mutation_lock()?;
        let recovery = self.recover_with_purpose(&StdFs, RecoveryPurpose::Mutation)?;
        if !recovery.allows_mutation() {
            return Err(finalization_pending_error());
        }
        let serialized = serialize(credentials)?;
        reparse::<Credentials>(&serialized)?;
        StdFs.atomic_write(&self.credentials_path, &serialized)
    }

    /// Loads the effective configuration from this store's explicit paths.
    ///
    /// Recovers any pending provider-unset transaction before reading.
    /// Applies environment variable substitution and credential merge
    /// using the same semantics as [`Config::load`].
    ///
    /// This is an additive public API (pre-1.0, non-breaking).
    /// It is not a general-purpose transaction API.
    pub fn load_effective(&self) -> Result<Config, ConfigError> {
        let _lock = self.acquire_mutation_lock()?;
        let _ = self.recover_with_purpose(&StdFs, RecoveryPurpose::Load)?;
        self.load_effective_unlocked()
    }

    fn load_effective_unlocked(&self) -> Result<Config, ConfigError> {
        if !self.config_path.exists() {
            let mut config = Config::default();
            if self.credentials_path.exists() {
                let raw = std::fs::read(&self.credentials_path).map_err(ConfigError::IoError)?;
                let creds: Credentials =
                    parse_persisted_toml(&raw, PersistedDocument::Credentials)?;
                config.merge_credentials(&creds);
            }
            return Ok(config);
        }

        let raw = std::fs::read(&self.config_path).map_err(ConfigError::IoError)?;
        let config_str = std::str::from_utf8(&raw)
            .map_err(|_| ConfigError::ParseError("config.toml is not valid UTF-8".into()))?;
        let substituted = crate::substitute_env_vars(config_str);
        let mut config: Config = toml::from_str(&substituted)
            .map_err(|_| ConfigError::ParseError("config.toml is not valid TOML".into()))?;

        if self.credentials_path.exists() {
            let raw = std::fs::read(&self.credentials_path).map_err(ConfigError::IoError)?;
            let creds: Credentials = parse_persisted_toml(&raw, PersistedDocument::Credentials)?;
            config.merge_credentials(&creds);
        }

        Ok(config)
    }

    fn acquire_mutation_lock(&self) -> Result<MutationLock, ConfigError> {
        let parent = self.txn_dir_parent();
        std::fs::create_dir_all(&parent).map_err(ConfigError::IoError)?;
        let file = open_lock_file(&parent.join(MUTATION_LOCK_NAME))?;
        file.lock().map_err(ConfigError::IoError)?;
        Ok(MutationLock { _file: file })
    }

    pub(super) fn txn_dir_parent(&self) -> PathBuf {
        self.config_path
            .parent()
            .map(std::path::Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    }

    pub(super) fn txn_dir(&self) -> PathBuf {
        self.txn_dir_parent().join(ACTIVE_DIR_NAME)
    }

    pub(super) fn finalize_dir(&self, transaction_id: &str) -> PathBuf {
        self.txn_dir_parent()
            .join(format!("{FINALIZE_PREFIX}{transaction_id}"))
    }
}

#[cfg(unix)]
fn open_lock_file(path: &Path) -> Result<File, ConfigError> {
    use std::os::unix::fs::OpenOptionsExt;

    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(path)
        .map_err(ConfigError::IoError)
}

#[cfg(not(unix))]
fn open_lock_file(path: &Path) -> Result<File, ConfigError> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .map_err(ConfigError::IoError)
}

pub(super) fn finalization_pending_error() -> ConfigError {
    ConfigError::InvalidConfig(
        "provider configuration finalization is pending; retry after recovery".into(),
    )
}

pub(super) fn concurrent_mutation_error() -> ConfigError {
    ConfigError::InvalidConfig(
        "provider configuration changed concurrently; retry the mutation".into(),
    )
}

pub(in crate::store) fn before_state_matches(
    fs: &dyn Fs,
    config_path: &std::path::Path,
    credentials_path: &std::path::Path,
    txn: &Txn,
) -> Result<bool, ConfigError> {
    fs.checkpoint(FsOperation::VerifyConfigBeforeApply)?;
    let actual_config = fs.read_opt(config_path)?;
    fs.checkpoint(FsOperation::VerifyCredentialsBeforeApply)?;
    let actual_credentials = fs.read_opt(credentials_path)?;

    Ok(actual_config.as_deref()
        == if txn.cfg_existed {
            Some(txn.cfg_before.as_slice())
        } else {
            None
        }
        && actual_credentials.as_deref()
            == if txn.cred_existed {
                Some(txn.cred_before.as_slice())
            } else {
                None
            })
}

pub(super) fn ambiguous_finalization_error() -> ConfigError {
    ConfigError::InvalidConfig(
        "ambiguous provider configuration finalization evidence; manual recovery required".into(),
    )
}

fn log_finalize_pending(context: &str, outcome: &FinalizeOutcome) {
    match outcome {
        FinalizeOutcome::Finalized => {}
        FinalizeOutcome::ActiveRenamePending { transaction_id } => {
            tracing::warn!(
                transaction_id = %transaction_id,
                context = %context,
                "provider configuration finalization rename is pending"
            );
        }
        FinalizeOutcome::ParentSyncPending { transaction_id, .. } => {
            tracing::warn!(
                transaction_id = %transaction_id,
                context = %context,
                "provider configuration finalization parent sync is pending"
            );
        }
        FinalizeOutcome::CleanupPending { transaction_id, .. } => {
            tracing::warn!(
                transaction_id = %transaction_id,
                context = %context,
                "provider configuration finalization cleanup is pending"
            );
        }
    }
}

fn mutate(
    config: &mut Config,
    creds: &mut Credentials,
    key: &str,
) -> Result<ConfigUnsetOutcome, ConfigError> {
    let parts: Vec<&str> = key.split('.').collect();
    match parts.as_slice() {
        ["providers", name] => {
            let in_cfg = config.providers.remove(*name).is_some();
            let in_cred = creds.keys.remove(*name).is_some();
            if !in_cfg && !in_cred {
                return Err(ConfigError::InvalidConfig(format!(
                    "provider '{name}' not found"
                )));
            }
            Ok(if builtin_provider_config(name).is_some() {
                ConfigUnsetOutcome::BuiltinProviderDisconnected {
                    name: (*name).to_string(),
                }
            } else {
                ConfigUnsetOutcome::CustomProviderRemoved {
                    name: (*name).to_string(),
                }
            })
        }
        ["providers", name, "api_key"] => {
            let mut changed = false;
            if let Some(provider) = config.providers.get_mut(*name)
                && provider.api_key.is_some()
            {
                provider.api_key = None;
                changed = true;
            }
            if creds.keys.remove(*name).is_some() {
                changed = true;
            }
            if !changed {
                return Err(ConfigError::InvalidConfig(format!(
                    "providers.{name}.api_key not found"
                )));
            }
            Ok(ConfigUnsetOutcome::ApiKeyCleared {
                name: (*name).to_string(),
            })
        }
        _ => Err(ConfigError::InvalidConfig(format!(
            "unsupported unset key: '{key}'"
        ))),
    }
}

fn serialize<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, ConfigError> {
    toml::to_string_pretty(value)
        .map(String::into_bytes)
        .map_err(|error| ConfigError::SerializeError(error.to_string()))
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum PersistedDocument {
    Config,
    Credentials,
    TransactionManifest,
}

pub(crate) fn parse_persisted_toml<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
    document: PersistedDocument,
) -> Result<T, ConfigError> {
    let kind = match document {
        PersistedDocument::Config => "config.toml",
        PersistedDocument::Credentials => "credentials.toml",
        PersistedDocument::TransactionManifest => "transaction manifest",
    };
    let string = std::str::from_utf8(bytes)
        .map_err(|_| ConfigError::ParseError(format!("{kind} is not valid UTF-8")))?;
    toml::from_str(string).map_err(|_| ConfigError::ParseError(format!("{kind} is not valid TOML")))
}

fn parse_optional_persisted<T>(
    value: &Option<Vec<u8>>,
    document: PersistedDocument,
) -> Result<T, ConfigError>
where
    T: Default + serde::de::DeserializeOwned,
{
    match value {
        None => Ok(T::default()),
        Some(bytes) => parse_persisted_toml(bytes, document),
    }
}

fn reparse<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<(), ConfigError> {
    let string = std::str::from_utf8(bytes)
        .map_err(|_| ConfigError::ParseError("serialized output is not valid UTF-8".into()))?;
    toml::from_str::<T>(string)
        .map_err(|_| ConfigError::ParseError("serialized output is not valid TOML".into()))?;
    Ok(())
}
