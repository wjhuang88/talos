use crate::{ConfigError, ConfigStore, home_dir};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
        ConfigStore::default_store().load_credentials_snapshot()
    }

    /// Persists the complete credentials snapshot to the default path.
    ///
    /// Talos runtime flows use [`ConfigStore::update_config`] or the provider
    /// removal transaction instead of holding this snapshot across user
    /// interaction.
    pub fn save(&self) -> Result<(), ConfigError> {
        ConfigStore::default_store().replace_credentials_snapshot(self)
    }
}
