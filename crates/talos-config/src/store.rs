use crate::{
    Config, ConfigError, ConfigUnsetOutcome, Credentials, builtin_provider_config, home_dir,
};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Durable two-file configuration store for provider removal and credential
/// clearing (MODEL-010 correction).
///
/// Operates on the **raw** persisted files (`config.toml` and
/// `credentials.toml`) without invoking `Config::merge_credentials`. This
/// prevents credential resurrection: a credential cleared from `config.toml`
/// is also removed from `credentials.toml` in the same logical operation, so
/// the next `Config::load()` cannot re-inject it.
///
/// Both files are written using atomic temp-file-then-rename replacement,
/// matching the pattern in `recent_models.rs` and `compact_text.rs`.
pub struct ConfigStore {
    config_path: PathBuf,
    credentials_path: PathBuf,
}

impl ConfigStore {
    /// Returns a store pointing at the default user paths
    /// (`~/.talos/config.toml` and `~/.talos/credentials.toml`).
    #[must_use]
    pub fn default_store() -> Self {
        let mut config_dir = home_dir();
        config_dir.push(".talos");
        Self {
            config_path: config_dir.join("config.toml"),
            credentials_path: config_dir.join("credentials.toml"),
        }
    }

    /// Creates a store with explicit file paths (for testing).
    #[must_use]
    pub fn with_paths(config_path: PathBuf, credentials_path: PathBuf) -> Self {
        Self {
            config_path,
            credentials_path,
        }
    }

    /// Removes a provider entry or clears a single credential from both
    /// persisted sources.
    ///
    /// See [`ConfigUnsetOutcome`] for the semantic distinction between custom
    /// provider removal, builtin provider disconnection, and api_key clearing.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::InvalidConfig`] for unsupported dotted keys or
    /// when the target is not present in either persisted source.
    pub fn unset_provider(&self, key: &str) -> Result<ConfigUnsetOutcome, ConfigError> {
        let mut config = self.load_raw_config()?;
        let mut credentials = self.load_raw_credentials()?;

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

                let outcome = if builtin_provider_config(name).is_some() {
                    ConfigUnsetOutcome::BuiltinProviderDisconnected {
                        name: (*name).to_string(),
                    }
                } else {
                    ConfigUnsetOutcome::CustomProviderRemoved {
                        name: (*name).to_string(),
                    }
                };
                (outcome, was_in_config, was_in_credentials)
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

        // Validate: the resulting config must re-serialize and re-parse.
        if config_changed {
            let config_toml = toml::to_string_pretty(&config)
                .map_err(|e| ConfigError::SerializeError(e.to_string()))?;
            let _: Config =
                toml::from_str(&config_toml).map_err(|e| ConfigError::ParseError(e.to_string()))?;
        }

        // Commit order: credentials first, then config.
        // Rationale: if we crash after writing credentials but before config,
        // the old credential is gone but config.toml still has its original
        // content (which may include the inline key). On next load, the inline
        // key from config.toml is used — no resurrection. The user can retry.
        // If we crash after config, both are committed — correct.
        if creds_changed {
            if credentials.keys.is_empty() {
                if self.credentials_path.exists() {
                    std::fs::remove_file(&self.credentials_path)?;
                }
            } else {
                let creds_toml = toml::to_string_pretty(&credentials)
                    .map_err(|e| ConfigError::SerializeError(e.to_string()))?;
                let _: Credentials = toml::from_str(&creds_toml)
                    .map_err(|e| ConfigError::ParseError(e.to_string()))?;
                atomic_write(&self.credentials_path, &creds_toml)?;
            }
        }

        if config_changed {
            let config_toml = toml::to_string_pretty(&config)
                .map_err(|e| ConfigError::SerializeError(e.to_string()))?;
            atomic_write(&self.config_path, &config_toml)?;
        }

        Ok(outcome)
    }

    fn load_raw_config(&self) -> Result<Config, ConfigError> {
        if !self.config_path.exists() {
            return Ok(Config::default());
        }
        let raw = std::fs::read_to_string(&self.config_path)?;
        toml::from_str(&raw).map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    fn load_raw_credentials(&self) -> Result<Credentials, ConfigError> {
        if !self.credentials_path.exists() {
            return Ok(Credentials::default());
        }
        let raw = std::fs::read_to_string(&self.credentials_path)?;
        toml::from_str(&raw).map_err(|e| ConfigError::ParseError(e.to_string()))
    }
}

/// Atomically replaces `path` with `content` using a temp-file-then-rename
/// pattern (write → flush → sync_all → rename).
///
/// Matches the pattern in `talos-cli/src/recent_models.rs` and
/// `talos-session/src/compact_text.rs`. On Unix, `rename(2)` is atomic; on
/// Windows, Rust uses `MoveFileExW` with `MOVEFILE_REPLACE_EXISTING`.
fn atomic_write(path: &Path, content: &str) -> Result<(), ConfigError> {
    let dir = path
        .parent()
        .ok_or_else(|| ConfigError::InvalidConfig("path has no parent directory".to_string()))?;

    std::fs::create_dir_all(dir)?;

    let temp_name = format!(
        ".{}.atomic-tmp",
        path.file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_else(|| "config".to_string())
    );
    let temp_path = dir.join(&temp_name);

    // Clean up any stale temp file from a previous failed attempt.
    let _ = std::fs::remove_file(&temp_path);

    let mut file = std::fs::File::create(&temp_path)?;
    file.write_all(content.as_bytes())?;
    file.flush()?;
    file.sync_all()?;
    drop(file);

    if let Err(e) = std::fs::rename(&temp_path, path) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(ConfigError::IoError(e));
    }

    Ok(())
}
