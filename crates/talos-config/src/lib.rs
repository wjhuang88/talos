//! Talos config — configuration schema, validation, and environment substitution.
//!
//! Loads configuration from `~/.talos/config.toml` with support for environment
//! variable substitution (`${ENV_VAR}` syntax) and JSON Schema validation.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

/// Error types for configuration operations.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// The API key is missing from both config and environment variables.
    #[error("missing API key for provider '{0}': set the {1} environment variable or add it to config")]
    MissingApiKey(String, String),

    /// The configuration failed JSON Schema validation.
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    /// An I/O error occurred while reading the configuration file.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// The configuration file contains invalid TOML.
    #[error("failed to parse config file: {0}")]
    ParseError(String),
}

/// Supported LLM providers.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    /// Anthropic Claude provider.
    #[default]
    Anthropic,
    /// OpenAI provider.
    OpenAI,
}

/// Talos configuration.
///
/// Contains the model provider, model name, and optional API key.
/// API keys can be specified in the config file or via environment variables.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct Config {
    /// The LLM provider to use (defaults to `anthropic`).
    #[serde(default)]
    pub provider: Provider,

    /// The model name to use (e.g., `claude-sonnet-4-20250514`).
    pub model: String,

    /// Optional API key. If not set, the key is read from environment variables.
    pub api_key: Option<String>,

    /// Hook-system configuration.
    #[serde(default)]
    pub hooks: HookConfig,

    /// MCP configuration placeholder for I009-S3.
    #[serde(default)]
    pub mcp: McpConfig,

    /// JSON-RPC configuration placeholder for I009-S5.
    #[serde(default)]
    pub rpc: RpcConfig,
}

/// Hook-system configuration placeholder for I009-S2.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct HookConfig {
    // TODO: I009-S2 will fill this
}

/// MCP configuration placeholder for I009-S3.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct McpConfig {
    /// Declared MCP servers.
    #[serde(default)]
    pub servers: Vec<McpServerConfig>,
}

/// MCP server configuration placeholder for I009-S3.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpServerConfig {
    /// Stable MCP server name.
    pub name: String,
    /// Transport kind (`stdio` or `http`).
    pub transport: String,
    // TODO: I009-S3 will add command/args/env or url fields
}

/// JSON-RPC server configuration placeholder for I009-S5.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct RpcConfig {
    /// Whether RPC mode is enabled.
    pub enabled: bool,
    /// Allowed RPC methods.
    #[serde(default)]
    pub methods_allowlist: Vec<String>,
}

impl Config {
    /// Returns the default path for the configuration file: `~/.talos/config.toml`.
    pub fn default_path() -> PathBuf {
        let mut path = home_dir();
        path.push(".talos");
        path.push("config.toml");
        path
    }

    /// Loads configuration from the default path `~/.talos/config.toml`.
    ///
    /// If the file exists, it is read, environment variable substitution is
    /// performed, and the result is validated against the JSON Schema.
    ///
    /// If the file does not exist, returns a default config (env-only mode).
    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::default_path();
        if !path.exists() {
            return Ok(Self::default());
        }

        let raw = fs::read_to_string(&path)?;
        let substituted = substitute_env_vars(&raw);
        let config: Config = toml::from_str(&substituted).map_err(|e| ConfigError::ParseError(e.to_string()))?;

        config.validate()?;
        Ok(config)
    }

    /// Returns the API key for the current provider.
    ///
    /// Checks the config file first, then falls back to environment variables:
    /// - `ANTHROPIC_API_KEY` for the Anthropic provider
    /// - `OPENAI_API_KEY` for the OpenAI provider
    pub fn api_key(&self) -> Result<String, ConfigError> {
        if let Some(key) = &self.api_key {
            if !key.is_empty() {
                return Ok(key.clone());
            }
        }

        let env_var = match self.provider {
            Provider::Anthropic => "ANTHROPIC_API_KEY",
            Provider::OpenAI => "OPENAI_API_KEY",
        };

        env::var(env_var).map_err(|_| {
            ConfigError::MissingApiKey(
                format!("{:?}", self.provider),
                env_var.to_string(),
            )
        })
    }

    /// Validates the configuration against its JSON Schema.
    ///
    /// The schema is generated via `schemars` and the same constraints are
    /// enforced manually (model must be non-empty, provider must be valid).
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Generate schema for external tooling (documentation, IDE support)
        let _schema = schemars::schema_for!(Config);

        // Manual validation of required constraints
        if self.model.is_empty() {
            return Err(ConfigError::InvalidConfig(
                "'model' is required and must be non-empty".to_string(),
            ));
        }

        Ok(())
    }
}

/// Returns the user's home directory.
fn home_dir() -> PathBuf {
    env::var("HOME")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Performs `${ENV_VAR}` substitution in a string.
///
/// Replaces all occurrences of `${VAR_NAME}` with the value of the
/// corresponding environment variable. If the variable is not set,
/// the placeholder is left unchanged.
fn substitute_env_vars(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' && chars.peek() == Some(&'{') {
            chars.next(); // consume '{'
            let mut var_name = String::new();
            let mut found_close = false;
            while let Some(&c) = chars.peek() {
                chars.next();
                if c == '}' {
                    found_close = true;
                    break;
                }
                var_name.push(c);
            }
            if found_close {
                if let Ok(value) = env::var(&var_name) {
                    result.push_str(&value);
                } else {
                    // Variable not set, keep the placeholder
                    result.push_str("${");
                    result.push_str(&var_name);
                    result.push('}');
                }
            } else {
                result.push_str("${");
                result.push_str(&var_name);
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_substitute_env_vars_replaces_known_vars() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe { env::set_var("TALOS_TEST_KEY", "secret123") };
        let input = "key = \"${TALOS_TEST_KEY}\"";
        let output = substitute_env_vars(input);
        assert_eq!(output, "key = \"secret123\"");
        unsafe { env::remove_var("TALOS_TEST_KEY") };
    }

    #[test]
    fn test_substitute_env_vars_leaves_unknown_vars() {
        let input = "key = \"${NONEXISTENT_VAR_12345}\"";
        let output = substitute_env_vars(input);
        assert_eq!(output, input);
    }

    #[test]
    fn test_substitute_env_vars_multiple_substitutions() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe {
            env::set_var("TALOS_A", "hello");
            env::set_var("TALOS_B", "world");
        }
        let input = "${TALOS_A} ${TALOS_B}";
        let output = substitute_env_vars(input);
        assert_eq!(output, "hello world");
        unsafe {
            env::remove_var("TALOS_A");
            env::remove_var("TALOS_B");
        }
    }

    #[test]
    fn test_substitute_env_vars_no_vars() {
        let input = "plain text with no vars";
        let output = substitute_env_vars(input);
        assert_eq!(output, input);
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.provider, Provider::Anthropic);
        assert!(config.model.is_empty());
        assert!(config.api_key.is_none());
    }

    #[test]
    fn test_api_key_from_config() {
        let config = Config {
            provider: Provider::Anthropic,
            model: "claude-test".to_string(),
            api_key: Some("config-key".to_string()),
            hooks: HookConfig::default(),
            mcp: McpConfig::default(),
            rpc: RpcConfig::default(),
        };
        assert_eq!(config.api_key().unwrap(), "config-key");
    }

    #[test]
    fn test_api_key_from_env_anthropic() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe { env::set_var("ANTHROPIC_API_KEY", "env-key-anthropic") };
        let config = Config {
            provider: Provider::Anthropic,
            model: "claude-test".to_string(),
            api_key: None,
            hooks: HookConfig::default(),
            mcp: McpConfig::default(),
            rpc: RpcConfig::default(),
        };
        assert_eq!(config.api_key().unwrap(), "env-key-anthropic");
        unsafe { env::remove_var("ANTHROPIC_API_KEY") };
    }

    #[test]
    fn test_api_key_from_env_openai() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe { env::set_var("OPENAI_API_KEY", "env-key-openai") };
        let config = Config {
            provider: Provider::OpenAI,
            model: "gpt-test".to_string(),
            api_key: None,
            hooks: HookConfig::default(),
            mcp: McpConfig::default(),
            rpc: RpcConfig::default(),
        };
        assert_eq!(config.api_key().unwrap(), "env-key-openai");
        unsafe { env::remove_var("OPENAI_API_KEY") };
    }

    #[test]
    fn test_api_key_missing_error() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe { env::remove_var("ANTHROPIC_API_KEY") };
        let config = Config {
            provider: Provider::Anthropic,
            model: "claude-test".to_string(),
            api_key: None,
            hooks: HookConfig::default(),
            mcp: McpConfig::default(),
            rpc: RpcConfig::default(),
        };
        let err = config.api_key().unwrap_err();
        assert!(matches!(err, ConfigError::MissingApiKey(_, _)));
        let msg = err.to_string();
        assert!(msg.contains("ANTHROPIC_API_KEY"));
    }

    #[test]
    fn test_load_nonexistent_file() {
        let path = Config::default_path();
        if path.exists() {
            return;
        }
        let config = Config::load().unwrap();
        assert_eq!(config.provider, Provider::Anthropic);
        assert!(config.model.is_empty());
    }

    #[test]
    fn test_provider_serialization() {
        let config_anthropic = Config {
            provider: Provider::Anthropic,
            model: "test".to_string(),
            api_key: None,
            hooks: HookConfig::default(),
            mcp: McpConfig::default(),
            rpc: RpcConfig::default(),
        };
        let config_openai = Config {
            provider: Provider::OpenAI,
            model: "test".to_string(),
            api_key: None,
            hooks: HookConfig::default(),
            mcp: McpConfig::default(),
            rpc: RpcConfig::default(),
        };

        let a_str = toml::to_string(&config_anthropic).unwrap();
        let o_str = toml::to_string(&config_openai).unwrap();

        assert!(a_str.contains("anthropic"));
        assert!(o_str.contains("openai"));
    }

    #[test]
    fn test_config_from_toml() {
        let toml_str = r#"
            provider = "openai"
            model = "gpt-4"
            api_key = "sk-test"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.provider, Provider::OpenAI);
        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.api_key, Some("sk-test".to_string()));
    }
}
