use crate::{ConfigError, home_dir};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Credentials store — maps provider names to API keys.
///
/// Stored separately from the main config (`~/.talos/credentials.toml`) to
/// keep secrets out of `config.toml`, which may be shared or committed.
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Credentials {
    /// Provider name → API key mapping.
    #[serde(flatten)]
    pub keys: HashMap<String, String>,
}

impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credentials")
            .field("keys", &format!("{} key(s) [redacted]", self.keys.len()))
            .finish()
    }
}

impl Credentials {
    /// Returns the default path for the credentials file: `~/.talos/credentials.toml`.
    pub fn default_path() -> PathBuf {
        let mut path = home_dir();
        path.push(".talos");
        path.push("credentials.toml");
        path
    }

    /// Loads credentials from the default path.
    ///
    /// Returns an empty credentials store if the file does not exist.
    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::default_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = fs::read_to_string(&path)?;
        let creds: Credentials =
            toml::from_str(&raw).map_err(|e| ConfigError::ParseError(e.to_string()))?;
        Ok(creds)
    }

    /// Persists credentials to the default path.
    pub fn save(&self) -> Result<(), ConfigError> {
        let path = Self::default_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let toml_str =
            toml::to_string_pretty(self).map_err(|e| ConfigError::SerializeError(e.to_string()))?;
        fs::write(&path, toml_str)?;
        Ok(())
    }
}
