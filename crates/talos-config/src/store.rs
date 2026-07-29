use crate::{
    Config, ConfigError, ConfigUnsetOutcome, Credentials, builtin_provider_config, home_dir,
};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct ConfigStore {
    config_path: PathBuf,
    credentials_path: PathBuf,
}

impl ConfigStore {
    #[must_use]
    pub fn default_store() -> Self {
        let mut config_dir = home_dir();
        config_dir.push(".talos");
        Self {
            config_path: config_dir.join("config.toml"),
            credentials_path: config_dir.join("credentials.toml"),
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
        self.unset_provider_inner(key, &StdFs)
    }

    pub(crate) fn unset_provider_inner(
        &self,
        key: &str,
        fs: &dyn TransactionFs,
    ) -> Result<ConfigUnsetOutcome, ConfigError> {
        let mut config = self.load_raw_config(fs)?;
        let mut credentials = self.load_raw_credentials(fs)?;

        let parts: Vec<&str> = key.split('.').collect();
        let (outcome, config_changed, creds_changed) = match parts.as_slice() {
            ["providers", name] => {
                let was_in_config = config.providers.remove(*name).is_some();
                let was_in_credentials = credentials.keys.remove(*name).is_some();

                if !was_in_config && !was_in_credentials {
                    return Err(ConfigError::InvalidConfig(format!(
                        "provider '{name}' not found in user configuration or credentials"
                    )));
                }

                let oc = if builtin_provider_config(name).is_some() {
                    ConfigUnsetOutcome::BuiltinProviderDisconnected {
                        name: (*name).to_string(),
                    }
                } else {
                    ConfigUnsetOutcome::CustomProviderRemoved {
                        name: (*name).to_string(),
                    }
                };
                (oc, was_in_config, was_in_credentials)
            }
            ["providers", name, "api_key"] => {
                let mut cfg_changed = false;
                let mut crd_changed = false;

                if let Some(provider) = config.providers.get_mut(*name)
                    && provider.api_key.is_some()
                {
                    provider.api_key = None;
                    cfg_changed = true;
                }

                if credentials.keys.remove(*name).is_some() {
                    crd_changed = true;
                }

                if !cfg_changed && !crd_changed {
                    return Err(ConfigError::InvalidConfig(format!(
                        "providers.{name}.api_key not found in user configuration or credentials"
                    )));
                }

                (
                    ConfigUnsetOutcome::ApiKeyCleared {
                        name: (*name).to_string(),
                    },
                    cfg_changed,
                    crd_changed,
                )
            }
            _ => {
                return Err(ConfigError::InvalidConfig(format!(
                    "unsupported unset key: '{key}' — only 'providers.<name>' and \
                     'providers.<name>.api_key' are supported"
                )));
            }
        };

        if config_changed {
            let config_toml = toml::to_string_pretty(&config)
                .map_err(|e| ConfigError::SerializeError(e.to_string()))?;
            let _: Config =
                toml::from_str(&config_toml).map_err(|e| ConfigError::ParseError(e.to_string()))?;
        }

        // Canonical write order for removal: config FIRST, then credentials.
        // If we crash after config but before credentials, the provider is gone
        // from config.toml while the old credential lingers in credentials.toml.
        // On the next Config::load(), merge_credentials skips orphan credentials
        // for non-builtin providers absent from config — preventing resurrection.
        // The orphan is naturally cleaned up on the next unset operation.
        if config_changed {
            let config_toml = toml::to_string_pretty(&config)
                .map_err(|e| ConfigError::SerializeError(e.to_string()))?;
            fs.atomic_write(&self.config_path, &config_toml)?;
        }

        if creds_changed {
            if credentials.keys.is_empty() {
                if fs.exists(&self.credentials_path) {
                    fs.remove_file(&self.credentials_path)?;
                }
            } else {
                let creds_toml = toml::to_string_pretty(&credentials)
                    .map_err(|e| ConfigError::SerializeError(e.to_string()))?;
                let _: Credentials = toml::from_str(&creds_toml)
                    .map_err(|e| ConfigError::ParseError(e.to_string()))?;
                fs.atomic_write(&self.credentials_path, &creds_toml)?;
            }
        }

        Ok(outcome)
    }

    fn load_raw_config(&self, fs: &dyn TransactionFs) -> Result<Config, ConfigError> {
        if !fs.exists(&self.config_path) {
            return Ok(Config::default());
        }
        let raw = fs.read_to_string(&self.config_path)?;
        toml::from_str(&raw).map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    fn load_raw_credentials(&self, fs: &dyn TransactionFs) -> Result<Credentials, ConfigError> {
        if !fs.exists(&self.credentials_path) {
            return Ok(Credentials::default());
        }
        let raw = fs.read_to_string(&self.credentials_path)?;
        toml::from_str(&raw).map_err(|e| ConfigError::ParseError(e.to_string()))
    }
}

pub(crate) trait TransactionFs {
    fn exists(&self, path: &Path) -> bool;
    fn read_to_string(&self, path: &Path) -> Result<String, ConfigError>;
    fn atomic_write(&self, path: &Path, content: &str) -> Result<(), ConfigError>;
    fn remove_file(&self, path: &Path) -> Result<(), ConfigError>;
}

struct StdFs;

impl TransactionFs for StdFs {
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn read_to_string(&self, path: &Path) -> Result<String, ConfigError> {
        std::fs::read_to_string(path).map_err(ConfigError::IoError)
    }

    fn atomic_write(&self, path: &Path, content: &str) -> Result<(), ConfigError> {
        atomic_write_impl(path, content)
    }

    fn remove_file(&self, path: &Path) -> Result<(), ConfigError> {
        std::fs::remove_file(path).map_err(ConfigError::IoError)
    }
}

fn atomic_write_impl(path: &Path, content: &str) -> Result<(), ConfigError> {
    let dir = path
        .parent()
        .ok_or_else(|| ConfigError::InvalidConfig("path has no parent directory".to_string()))?;

    std::fs::create_dir_all(dir)?;

    let counter = TEMP_COUNTER.fetch_add(1, Ordering::SeqCst);
    let temp_name = format!(
        ".{}.tmp.{}.{}",
        path.file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_else(|| "config".to_string()),
        std::process::id(),
        counter
    );
    let temp_path = dir.join(&temp_name);

    let _ = std::fs::remove_file(&temp_path);

    create_secure_file(&temp_path, path)?;
    {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .open(&temp_path)
            .map_err(|e| {
                let _ = std::fs::remove_file(&temp_path);
                ConfigError::IoError(e)
            })?;
        file.write_all(content.as_bytes()).map_err(|e| {
            let _ = std::fs::remove_file(&temp_path);
            ConfigError::IoError(e)
        })?;
        file.flush().map_err(|e| {
            let _ = std::fs::remove_file(&temp_path);
            ConfigError::IoError(e)
        })?;
        file.sync_all().map_err(|e| {
            let _ = std::fs::remove_file(&temp_path);
            ConfigError::IoError(e)
        })?;
    }

    copy_permissions(&temp_path, path)?;

    if let Err(e) = std::fs::rename(&temp_path, path) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(ConfigError::IoError(e));
    }

    sync_parent_dir(dir);

    Ok(())
}

#[cfg(unix)]
fn create_secure_file(temp_path: &Path, ref_path: &Path) -> Result<(), ConfigError> {
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    if let Ok(meta) = std::fs::metadata(ref_path) {
        opts.mode(meta.permissions().mode());
    } else {
        opts.mode(0o600);
    }
    opts.open(temp_path).map_err(ConfigError::IoError)?;
    Ok(())
}

#[cfg(not(unix))]
fn create_secure_file(temp_path: &Path, _ref_path: &Path) -> Result<(), ConfigError> {
    std::fs::File::create(temp_path).map_err(ConfigError::IoError)?;
    Ok(())
}

#[cfg(unix)]
fn copy_permissions(temp_path: &Path, ref_path: &Path) -> Result<(), ConfigError> {
    if let Ok(meta) = std::fs::metadata(ref_path) {
        std::fs::set_permissions(temp_path, meta.permissions()).map_err(|e| {
            let _ = std::fs::remove_file(temp_path);
            ConfigError::IoError(e)
        })?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn copy_permissions(_temp_path: &Path, _ref_path: &Path) -> Result<(), ConfigError> {
    Ok(())
}

#[cfg(unix)]
fn sync_parent_dir(dir: &Path) {
    if let Ok(dir_file) = std::fs::File::open(dir) {
        let _ = dir_file.sync_all();
    }
}

#[cfg(not(unix))]
fn sync_parent_dir(_dir: &Path) {}

#[cfg(test)]
pub(crate) fn atomic_write_for_test(path: &Path, content: &str) -> Result<(), ConfigError> {
    atomic_write_impl(path, content)
}
