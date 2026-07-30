//! Two-file transaction coordinator for provider removal (MODEL-010).

use crate::{
    Config, ConfigError, ConfigUnsetOutcome, Credentials, atomic_file, builtin_provider_config,
    home_dir,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static TXN_COUNTER: AtomicU64 = AtomicU64::new(0);

const MANIFEST_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
enum Phase {
    Prepared,
    Applying,
    Committed,
    RollbackRequired,
    RolledBack,
}

pub struct ConfigStore {
    config_path: PathBuf,
    credentials_path: PathBuf,
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
        self.recover(fs)?;

        let active = self.txn_dir();
        if fs.exists(&active) {
            return Err(ConfigError::InvalidConfig(
                "provider configuration finalization is pending; retry after recovery".into(),
            ));
        }

        let cfg_before = fs.read_opt(&self.config_path)?;
        let cred_before = fs.read_opt(&self.credentials_path)?;

        let mut config: Config = parse_strict(&cfg_before)?;
        let mut creds: Credentials = parse_strict(&cred_before)?;

        let outcome = mutate(&mut config, &mut creds, key)?;

        let cfg_after = serialize(&config)?;
        reparse::<Config>(&cfg_after)?;

        let cred_after_exists = !creds.keys.is_empty();
        let cred_after = if cred_after_exists {
            let s = serialize(&creds)?;
            reparse::<Credentials>(&s)?;
            s
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
                    if let Err(e) = finalize(fs, &active_dir, &txn_id) {
                        tracing::warn!("finalization pending after rollback: {e}");
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

        if let Err(e) = finalize(fs, &active_dir, &txn_id) {
            tracing::warn!("finalization pending after committed: {e}");
        }
        Ok(outcome)
    }

    pub fn recover_pending() -> Result<(), ConfigError> {
        Self::default_store().recover(&StdFs)
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
        self.recover(&StdFs)?;

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
            let raw_creds = std::fs::read(&self.credentials_path).map_err(ConfigError::IoError)?;
            let creds: Credentials =
                parse_persisted_toml(&raw_creds, PersistedDocument::Credentials)?;
            config.merge_credentials(&creds);
        }

        Ok(config)
    }

    pub(crate) fn recover(&self, fs: &dyn Fs) -> Result<(), ConfigError> {
        let parent = self.txn_dir_parent();
        let dir = self.txn_dir();

        fs.checkpoint(FsOperation::CleanupPrepareResidues)?;
        self.cleanup_residues(fs, &parent);

        if !fs.exists(&dir) {
            return Ok(());
        }

        let manifest_path = dir.join("manifest");
        if !fs.exists(&manifest_path) {
            return Err(ConfigError::InvalidConfig(
                "pending transaction directory exists without manifest — \
                 manual recovery required"
                    .into(),
            ));
        }

        fs.checkpoint(FsOperation::ReadActiveManifest)?;
        let raw = fs.read(&manifest_path)?;
        let manifest = parse_manifest(&raw)?;

        match manifest.phase {
            Phase::Committed => {
                fs.checkpoint(FsOperation::ReadConfigAfterImage)?;
                let cfg_after = if manifest.config_exists_after {
                    Some(fs.read(&dir.join("config.after"))?)
                } else {
                    None
                };
                fs.checkpoint(FsOperation::ReadCredentialsAfterImage)?;
                let cred_after = if manifest.credentials_exist_after {
                    Some(fs.read(&dir.join("credentials.after"))?)
                } else {
                    None
                };

                fs.checkpoint(FsOperation::VerifyConfigAfter)?;
                let cfg_actual = fs.read_opt(&self.config_path)?;
                fs.checkpoint(FsOperation::VerifyCredentialsAfter)?;
                let cred_actual = fs.read_opt(&self.credentials_path)?;

                if cfg_actual.as_deref() != cfg_after.as_deref() {
                    return Err(ConfigError::InvalidConfig(
                        "Committed recovery: config does not match after image".into(),
                    ));
                }
                if cred_actual.as_deref() != cred_after.as_deref() {
                    return Err(ConfigError::InvalidConfig(
                        "Committed recovery: credentials does not match after image".into(),
                    ));
                }

                let _ = finalize(fs, &dir, &manifest.transaction_id);
                Ok(())
            }
            Phase::Prepared | Phase::Applying | Phase::RollbackRequired => {
                fs.checkpoint(FsOperation::ReadConfigBeforeImage)?;
                let cfg_before = if manifest.config_existed_before {
                    Some(fs.read(&dir.join("config.before"))?)
                } else {
                    None
                };
                fs.checkpoint(FsOperation::ReadCredentialsBeforeImage)?;
                let cred_before = if manifest.credentials_existed_before {
                    Some(fs.read(&dir.join("credentials.before"))?)
                } else {
                    None
                };

                fs.checkpoint(FsOperation::RestoreConfigBefore)?;
                restore(fs, &self.config_path, cfg_before.as_deref())?;
                fs.checkpoint(FsOperation::RestoreCredentialsBefore)?;
                restore(fs, &self.credentials_path, cred_before.as_deref())?;

                fs.checkpoint(FsOperation::VerifyConfigBefore)?;
                let cfg_actual = fs.read_opt(&self.config_path)?;
                fs.checkpoint(FsOperation::VerifyCredentialsBefore)?;
                let cred_actual = fs.read_opt(&self.credentials_path)?;
                if cfg_actual.as_deref() != cfg_before.as_deref() {
                    return Err(ConfigError::InvalidConfig(
                        "recovery verification failed for config".into(),
                    ));
                }
                if cred_actual.as_deref() != cred_before.as_deref() {
                    return Err(ConfigError::InvalidConfig(
                        "recovery verification failed for credentials".into(),
                    ));
                }

                fs.checkpoint(FsOperation::WriteRolledBackManifest)?;
                write_manifest(
                    fs,
                    &dir,
                    Phase::RolledBack,
                    &manifest.transaction_id,
                    &Txn {
                        cfg_existed: manifest.config_existed_before,
                        cfg_after_exists: manifest.config_exists_after,
                        cred_existed: manifest.credentials_existed_before,
                        cred_after_exists: manifest.credentials_exist_after,
                        cfg_before: cfg_before.unwrap_or_default(),
                        cfg_after: Vec::new(),
                        cred_before: cred_before.unwrap_or_default(),
                        cred_after: Vec::new(),
                    },
                )?;

                let _ = finalize(fs, &dir, &manifest.transaction_id);
                Ok(())
            }
            Phase::RolledBack => {
                fs.checkpoint(FsOperation::ReadConfigBeforeImage)?;
                let cfg_before = if manifest.config_existed_before {
                    Some(fs.read(&dir.join("config.before"))?)
                } else {
                    None
                };
                fs.checkpoint(FsOperation::ReadCredentialsBeforeImage)?;
                let cred_before = if manifest.credentials_existed_before {
                    Some(fs.read(&dir.join("credentials.before"))?)
                } else {
                    None
                };

                fs.checkpoint(FsOperation::VerifyConfigBefore)?;
                let cfg_actual = fs.read_opt(&self.config_path)?;
                fs.checkpoint(FsOperation::VerifyCredentialsBefore)?;
                let cred_actual = fs.read_opt(&self.credentials_path)?;
                if cfg_actual.as_deref() != cfg_before.as_deref() {
                    return Err(ConfigError::InvalidConfig(
                        "RolledBack recovery: config does not match before image".into(),
                    ));
                }
                if cred_actual.as_deref() != cred_before.as_deref() {
                    return Err(ConfigError::InvalidConfig(
                        "RolledBack recovery: credentials does not match before image".into(),
                    ));
                }

                let _ = finalize(fs, &dir, &manifest.transaction_id);
                Ok(())
            }
        }
    }

    fn txn_dir_parent(&self) -> PathBuf {
        self.config_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    }

    fn txn_dir(&self) -> PathBuf {
        self.txn_dir_parent().join(".provider-unset-transaction")
    }

    fn cleanup_residues(&self, fs: &dyn Fs, parent: &Path) {
        let prepare_prefix = ".provider-unset-transaction.prepare.";
        if let Ok(entries) = fs.list_dir(parent) {
            for entry in entries {
                if let Some(name) = entry.file_name().and_then(|n| n.to_str())
                    && name.starts_with(prepare_prefix)
                {
                    let _ = fs.remove_dir(&entry);
                }
            }
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
            if let Some(p) = config.providers.get_mut(*name)
                && p.api_key.is_some()
            {
                p.api_key = None;
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

struct Txn {
    cfg_existed: bool,
    cfg_after_exists: bool,
    cred_existed: bool,
    cred_after_exists: bool,
    cfg_before: Vec<u8>,
    cfg_after: Vec<u8>,
    cred_before: Vec<u8>,
    cred_after: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    version: u32,
    phase: Phase,
    transaction_id: String,
    config_existed_before: bool,
    config_exists_after: bool,
    credentials_existed_before: bool,
    credentials_exist_after: bool,
}

fn gen_txn_id() -> String {
    let n = TXN_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!(
        "{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0),
        n
    )
}

fn write_manifest(
    fs: &dyn Fs,
    dir: &Path,
    phase: Phase,
    txn_id: &str,
    t: &Txn,
) -> Result<(), ConfigError> {
    let m = Manifest {
        version: MANIFEST_VERSION,
        phase,
        transaction_id: txn_id.to_string(),
        config_existed_before: t.cfg_existed,
        config_exists_after: t.cfg_after_exists,
        credentials_existed_before: t.cred_existed,
        credentials_exist_after: t.cred_after_exists,
    };
    let toml_str = toml::to_string(&m).map_err(|e| ConfigError::SerializeError(e.to_string()))?;
    fs.atomic_write(&dir.join("manifest"), toml_str.as_bytes())
}

fn parse_manifest(raw: &[u8]) -> Result<Manifest, ConfigError> {
    let s = std::str::from_utf8(raw)
        .map_err(|_| ConfigError::ParseError("transaction manifest is not valid UTF-8".into()))?;
    let m: Manifest = toml::from_str(s)
        .map_err(|_| ConfigError::ParseError("transaction manifest is not valid TOML".into()))?;
    if m.version != MANIFEST_VERSION {
        return Err(ConfigError::InvalidConfig(format!(
            "unsupported manifest version {} — expected {}",
            m.version, MANIFEST_VERSION
        )));
    }
    Ok(m)
}

fn prepare(fs: &dyn Fs, active_dir: &Path, t: &Txn, txn_id: &str) -> Result<(), ConfigError> {
    let parent = active_dir.parent().unwrap_or(Path::new("."));
    let staging_dir = parent.join(format!(".provider-unset-transaction.prepare.{txn_id}"));

    let staging_cleanup = |fs: &dyn Fs| {
        if fs.exists(&staging_dir) {
            let _ = fs.remove_dir(&staging_dir);
        }
    };

    fs.checkpoint(FsOperation::CreatePrepareDirectory)?;
    fs.mkdir(&staging_dir)?;
    if t.cfg_existed {
        fs.checkpoint(FsOperation::WriteConfigBeforeImage)?;
        if let Err(e) = fs.write_secure(&staging_dir.join("config.before"), &t.cfg_before) {
            staging_cleanup(fs);
            return Err(e);
        }
    }
    if t.cred_existed {
        fs.checkpoint(FsOperation::WriteCredentialsBeforeImage)?;
        if let Err(e) = fs.write_secure(&staging_dir.join("credentials.before"), &t.cred_before) {
            staging_cleanup(fs);
            return Err(e);
        }
    }
    if t.cfg_after_exists {
        fs.checkpoint(FsOperation::WriteConfigAfterImage)?;
        if let Err(e) = fs.write_secure(&staging_dir.join("config.after"), &t.cfg_after) {
            staging_cleanup(fs);
            return Err(e);
        }
    }
    if t.cred_after_exists {
        fs.checkpoint(FsOperation::WriteCredentialsAfterImage)?;
        if let Err(e) = fs.write_secure(&staging_dir.join("credentials.after"), &t.cred_after) {
            staging_cleanup(fs);
            return Err(e);
        }
    }
    fs.checkpoint(FsOperation::WritePreparedManifest)?;
    if let Err(e) = write_manifest(fs, &staging_dir, Phase::Prepared, txn_id, t) {
        staging_cleanup(fs);
        return Err(e);
    }
    fs.checkpoint(FsOperation::SyncPrepareDirectory)?;
    if let Err(e) = fs.sync_dir(&staging_dir) {
        staging_cleanup(fs);
        return Err(e);
    }

    if fs.exists(active_dir) {
        staging_cleanup(fs);
        return Err(ConfigError::InvalidConfig(
            "active transaction directory already exists — \
             recover pending transaction first"
                .into(),
        ));
    }

    fs.checkpoint(FsOperation::PublishActiveDirectory)?;
    fs.rename_dir(&staging_dir, active_dir)?;
    fs.checkpoint(FsOperation::SyncTransactionParentAfterPublish)?;
    fs.sync_dir(parent)?;
    Ok(())
}

fn apply(
    fs: &dyn Fs,
    dir: &Path,
    cfg: &Path,
    cred: &Path,
    t: &Txn,
    txn_id: &str,
) -> Result<(), ConfigError> {
    fs.checkpoint(FsOperation::WriteApplyingManifest)?;
    write_manifest(fs, dir, Phase::Applying, txn_id, t)?;
    fs.checkpoint(FsOperation::ReplaceConfigAfter)?;
    fs.atomic_write(cfg, &t.cfg_after)?;

    if t.cred_after_exists {
        fs.checkpoint(FsOperation::ReplaceCredentialsAfter)?;
        fs.atomic_write(cred, &t.cred_after)?;
    } else if fs.exists(cred) {
        fs.checkpoint(FsOperation::RemoveCredentialsAfter)?;
        fs.remove_file(cred)?;
        if let Some(parent) = cred.parent() {
            fs.checkpoint(FsOperation::SyncCredentialsParentAfterRemove)?;
            fs.sync_dir(parent)?;
        }
    }

    fs.checkpoint(FsOperation::VerifyConfigAfter)?;
    let cfg_actual = fs.read_opt(cfg)?;
    if cfg_actual.as_deref() != Some(t.cfg_after.as_slice()) {
        return Err(ConfigError::IoError(std::io::Error::other(
            "config after-state verification failed",
        )));
    }

    fs.checkpoint(FsOperation::VerifyCredentialsAfter)?;
    let cred_actual = fs.read_opt(cred)?;
    let cred_expected: Option<&[u8]> = if t.cred_after_exists {
        Some(&t.cred_after)
    } else {
        None
    };
    if cred_actual.as_deref() != cred_expected {
        return Err(ConfigError::IoError(std::io::Error::other(
            "credentials after-state verification failed",
        )));
    }

    fs.checkpoint(FsOperation::WriteCommittedManifest)?;
    write_manifest(fs, dir, Phase::Committed, txn_id, t)?;
    Ok(())
}

fn rollback(
    fs: &dyn Fs,
    dir: &Path,
    cfg: &Path,
    cred: &Path,
    t: &Txn,
    txn_id: &str,
) -> Result<(), ConfigError> {
    fs.checkpoint(FsOperation::WriteRollbackRequiredManifest)?;
    if let Err(e) = write_manifest(fs, dir, Phase::RollbackRequired, txn_id, t) {
        tracing::warn!("failed to write RollbackRequired manifest: {e}");
    }

    fs.checkpoint(FsOperation::RestoreConfigBefore)?;
    restore(
        fs,
        cfg,
        if t.cfg_existed {
            Some(&t.cfg_before)
        } else {
            None
        },
    )?;
    fs.checkpoint(FsOperation::RestoreCredentialsBefore)?;
    restore(
        fs,
        cred,
        if t.cred_existed {
            Some(&t.cred_before)
        } else {
            None
        },
    )?;

    fs.checkpoint(FsOperation::VerifyConfigBefore)?;
    let cfg_actual = fs.read_opt(cfg)?;
    fs.checkpoint(FsOperation::VerifyCredentialsBefore)?;
    let cred_actual = fs.read_opt(cred)?;
    if cfg_actual.as_deref()
        != (if t.cfg_existed {
            Some(&t.cfg_before[..])
        } else {
            None
        })
    {
        return Err(ConfigError::InvalidConfig(
            "rollback verification failed for config".into(),
        ));
    }
    if cred_actual.as_deref()
        != (if t.cred_existed {
            Some(&t.cred_before[..])
        } else {
            None
        })
    {
        return Err(ConfigError::InvalidConfig(
            "rollback verification failed for credentials".into(),
        ));
    }

    fs.checkpoint(FsOperation::WriteRolledBackManifest)?;
    write_manifest(fs, dir, Phase::RolledBack, txn_id, t)?;

    Ok(())
}

fn restore(fs: &dyn Fs, path: &Path, before: Option<&[u8]>) -> Result<(), ConfigError> {
    match before {
        Some(bytes) => fs.atomic_write(path, bytes)?,
        None => {
            if fs.exists(path) {
                fs.remove_file(path)?;
                if let Some(parent) = path.parent() {
                    fs.sync_dir(parent)?;
                }
            }
        }
    }
    Ok(())
}

fn finalize(fs: &dyn Fs, active_dir: &Path, txn_id: &str) -> Result<(), ConfigError> {
    if !fs.exists(active_dir) {
        return Ok(());
    }

    let parent = active_dir.parent().unwrap_or(Path::new("."));
    let finalize_dir = parent.join(format!(".provider-unset-transaction.finalize.{txn_id}"));

    if fs.exists(&finalize_dir) {
        return Err(ConfigError::InvalidConfig(format!(
            "ambiguous finalization: both active and finalize directory exist for transaction {txn_id}"
        )));
    }

    fs.checkpoint(FsOperation::PublishFinalizeDirectory)?;
    fs.rename_dir(active_dir, &finalize_dir)?;
    fs.checkpoint(FsOperation::SyncTransactionParentAfterFinalize)?;
    fs.sync_dir(parent)?;

    fs.checkpoint(FsOperation::CleanupFinalizeDirectory)?;
    let _ = fs.remove_dir(&finalize_dir);
    Ok(())
}

fn serialize<T: serde::Serialize>(val: &T) -> Result<Vec<u8>, ConfigError> {
    toml::to_string_pretty(val)
        .map(|s| s.into_bytes())
        .map_err(|e| ConfigError::SerializeError(e.to_string()))
}

#[allow(dead_code)]
pub(crate) enum PersistedDocument {
    Config,
    Credentials,
    TransactionManifest,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum FsOperation {
    CleanupPrepareResidues,
    CleanupFinalizeResidues,
    ReadActiveManifest,
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
    CleanupFinalizeDirectory,
}

fn parse_persisted_toml<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
    doc: PersistedDocument,
) -> Result<T, ConfigError> {
    let kind = match doc {
        PersistedDocument::Config => "config.toml",
        PersistedDocument::Credentials => "credentials.toml",
        PersistedDocument::TransactionManifest => "transaction manifest",
    };
    let s = std::str::from_utf8(bytes)
        .map_err(|_| ConfigError::ParseError(format!("{kind} is not valid UTF-8")))?;
    toml::from_str(s).map_err(|_| ConfigError::ParseError(format!("{kind} is not valid TOML")))
}

fn parse_strict<T>(opt: &Option<Vec<u8>>) -> Result<T, ConfigError>
where
    T: Default + serde::de::DeserializeOwned,
{
    match opt {
        None => Ok(T::default()),
        Some(bytes) => {
            let s = std::str::from_utf8(bytes).map_err(|_| {
                ConfigError::ParseError("persisted document is not valid UTF-8".into())
            })?;
            toml::from_str(s)
                .map_err(|_| ConfigError::ParseError("persisted document is not valid TOML".into()))
        }
    }
}

fn reparse<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<(), ConfigError> {
    toml::from_str::<T>(&String::from_utf8_lossy(bytes))
        .map_err(|_| ConfigError::ParseError("serialized output is not valid TOML".into()))?;
    Ok(())
}

pub(crate) trait Fs {
    #[allow(dead_code)]
    fn checkpoint(&self, _op: FsOperation) -> Result<(), ConfigError> {
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
}

pub(crate) struct StdFs;

impl Fs for StdFs {
    fn exists(&self, p: &Path) -> bool {
        p.exists()
    }
    fn read(&self, p: &Path) -> Result<Vec<u8>, ConfigError> {
        std::fs::read(p).map_err(ConfigError::IoError)
    }
    fn atomic_write(&self, p: &Path, c: &[u8]) -> Result<(), ConfigError> {
        atomic_file::durable_replace(p, c)
    }
    fn write_secure(&self, p: &Path, c: &[u8]) -> Result<(), ConfigError> {
        atomic_file::write_file_synced(p, c)
    }
    fn mkdir(&self, p: &Path) -> Result<(), ConfigError> {
        atomic_file::create_dir_secure(p)
    }
    fn remove_file(&self, p: &Path) -> Result<(), ConfigError> {
        std::fs::remove_file(p).map_err(ConfigError::IoError)
    }
    fn remove_dir(&self, p: &Path) -> Result<(), ConfigError> {
        std::fs::remove_dir_all(p).map_err(ConfigError::IoError)
    }
    fn rename_dir(&self, from: &Path, to: &Path) -> Result<(), ConfigError> {
        std::fs::rename(from, to).map_err(ConfigError::IoError)
    }
    fn sync_dir(&self, d: &Path) -> Result<(), ConfigError> {
        atomic_file::sync_dir(d)
    }
    fn list_dir(&self, d: &Path) -> Result<Vec<PathBuf>, ConfigError> {
        std::fs::read_dir(d)
            .map_err(ConfigError::IoError)?
            .map(|e| e.map(|e| e.path()).map_err(ConfigError::IoError))
            .collect()
    }
}
