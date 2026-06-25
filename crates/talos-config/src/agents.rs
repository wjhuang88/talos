//! Shared Agent config import — `~/.agents/talos/config.toml` read-only compatibility layer.
//!
//! Talos reads from its own namespace (`talos/`) under the shared `~/.agents/`
//! directory. This keeps the rest of `~/.agents/` available for other tools
//! and shared standards to evolve independently.
//!
//! # Format
//!
//! TOML, same schema as `~/.talos/config.toml`.  Only the `[providers]` section
//! is imported; other sections (`[log]`, `[hooks]`, `[mcp]`, `[rpc]`) are
//! Talos-specific and belong in `~/.talos/config.toml`.
//!
//! ```toml
//! # ~/.agents/talos/config.toml
//! [providers.anthropic]
//! protocol = "anthropic-messages"
//! api_key_env = "ANTHROPIC_API_KEY"
//!
//! [providers.anthropic.models.claude-sonnet-4-5-20250929]
//! context_limit = 200000
//! output_limit = 8192
//! ```
//!
//! # Precedence
//!
//! Shared Agent config is the lowest priority source.  CLI flags, env vars,
//! and `~/.talos/config.toml` all take precedence.  Shared config provides
//! defaults only when Talos-owned config is absent.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::{ConfigError, ProviderConfig, home_dir};

/// Default path for Talos's shared Agent config: `~/.agents/talos/config.toml`.
pub fn default_agents_config_path() -> PathBuf {
    let mut path = home_dir();
    path.push(".agents");
    path.push("talos");
    path.push("config.toml");
    path
}

/// TOML-only config document for `~/.agents/talos/config.toml`.
/// Only the `[providers]` section is meaningful; other sections are
/// Talos-specific and belong in `~/.talos/config.toml`.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct AgentsConfig {
    providers: HashMap<String, ProviderConfig>,
}

/// Import provider configuration from `~/.agents/talos/config.toml`.
///
/// Returns a map of provider name → [`ProviderConfig`] suitable for merging
/// into the main Talos config.  The caller is responsible for respecting
/// config precedence (Talos-owned config overrides shared imports).
///
/// # Errors
///
/// Returns [`ConfigError::ParseError`] if the file contains invalid TOML.
/// Returns [`ConfigError::IoError`] if the file cannot be read.
pub fn import_agents_config(
    path: Option<&Path>,
) -> Result<HashMap<String, ProviderConfig>, ConfigError> {
    let file_path = path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(default_agents_config_path);

    let content = std::fs::read_to_string(&file_path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ConfigError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("shared agent config not found: {}", file_path.display()),
            ))
        } else {
            ConfigError::IoError(e)
        }
    })?;

    let config: AgentsConfig = toml::from_str(&content)
        .map_err(|e| ConfigError::ParseError(format!("invalid {0}: {e}", file_path.display())))?;

    Ok(config.providers)
}

/// Merge shared Agent config providers into the main Talos config.
///
/// Talos-owned providers (from `~/.talos/config.toml`) take precedence.
/// Shared Agent providers are only added when no Talos-owned provider
/// with the same name exists.
pub fn merge_agents_providers(
    config: &mut crate::Config,
    agents_providers: HashMap<String, ProviderConfig>,
) {
    for (name, provider_config) in agents_providers {
        config.providers.entry(name).or_insert(provider_config);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp_toml(name: &str, content: &str) -> (PathBuf, PathBuf) {
        let dir = std::env::temp_dir().join(format!("talos-agents-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create test dir");
        let path = dir.join("config.toml");
        let mut file = std::fs::File::create(&path).expect("create");
        file.write_all(content.as_bytes()).expect("write");
        drop(file);
        (dir, path)
    }

    #[test]
    fn import_empty_providers() {
        let toml = "";
        let (dir, path) = write_temp_toml("empty", toml);
        let result = import_agents_config(Some(&path)).expect("import");
        assert!(result.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn import_single_provider_with_models() {
        let toml = r#"
[providers.anthropic]
protocol = "anthropic-messages"
api_key_env = "ANTHROPIC_API_KEY"

[providers.anthropic.models.claude-sonnet-4-5-20250929]
context_limit = 200000
output_limit = 8192
"#;
        let (dir, path) = write_temp_toml("single", toml);
        let result = import_agents_config(Some(&path)).expect("import");

        assert_eq!(result.len(), 1);
        let provider = result.get("anthropic").expect("anthropic provider");
        assert_eq!(
            provider.protocol,
            crate::ProviderProtocol::AnthropicMessages
        );
        assert_eq!(provider.api_key_env.as_deref(), Some("ANTHROPIC_API_KEY"));
        let model = provider
            .models
            .get("claude-sonnet-4-5-20250929")
            .expect("model");
        assert_eq!(model.context_limit, Some(200_000));
        assert_eq!(model.output_limit, Some(8192));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn import_with_base_url() {
        let toml = r#"
[providers.custom]
protocol = "openai-chat"
base_url = "https://api.example.com/v1"
api_key_env = "CUSTOM_KEY"
"#;
        let (dir, path) = write_temp_toml("baseurl", toml);
        let result = import_agents_config(Some(&path)).expect("import");

        let provider = result.get("custom").expect("custom provider");
        assert_eq!(provider.protocol, crate::ProviderProtocol::OpenAIChat);
        assert_eq!(
            provider.base_url.as_deref(),
            Some("https://api.example.com/v1")
        );
        assert_eq!(provider.api_key_env.as_deref(), Some("CUSTOM_KEY"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn import_missing_file_returns_error() {
        let nonexistent = PathBuf::from("/nonexistent/agents/talos/config.toml");
        let result = import_agents_config(Some(&nonexistent));
        assert!(result.is_err());
    }

    #[test]
    fn import_invalid_toml_returns_error() {
        let toml = "not valid toml {{";
        let (dir, path) = write_temp_toml("invalid", toml);
        let result = import_agents_config(Some(&path));
        assert!(result.is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn merge_agents_providers_does_not_overwrite_existing() {
        let mut config = crate::Config::default();
        config.providers.insert(
            "anthropic".to_string(),
            ProviderConfig {
                protocol: crate::ProviderProtocol::AnthropicMessages,
                api_key_env: Some("MY_KEY".to_string()),
                ..Default::default()
            },
        );

        let mut agents = HashMap::new();
        agents.insert(
            "anthropic".to_string(),
            ProviderConfig {
                protocol: crate::ProviderProtocol::OpenAIChat,
                api_key_env: Some("AGENTS_KEY".to_string()),
                ..Default::default()
            },
        );
        agents.insert(
            "openai".to_string(),
            ProviderConfig {
                protocol: crate::ProviderProtocol::OpenAIChat,
                api_key_env: Some("OPENAI_KEY".to_string()),
                ..Default::default()
            },
        );

        merge_agents_providers(&mut config, agents);

        let existing = config.providers.get("anthropic").expect("anthropic");
        assert_eq!(existing.api_key_env.as_deref(), Some("MY_KEY"));

        let new = config.providers.get("openai").expect("openai");
        assert_eq!(new.api_key_env.as_deref(), Some("OPENAI_KEY"));
    }

    #[test]
    fn default_agents_config_path_ends_with_agents_talos_config_toml() {
        let path = default_agents_config_path();
        let s = path.to_string_lossy();
        assert!(s.contains(".agents"), "path should contain .agents: {s}");
        assert!(s.contains("talos"), "path should contain talos: {s}");
        assert!(
            s.ends_with("config.toml"),
            "path should end with config.toml: {s}"
        );
    }
}
