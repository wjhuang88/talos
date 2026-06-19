//! Talos config — configuration schema, validation, and environment substitution.
//!
//! Loads configuration from `~/.talos/config.toml` with support for environment
//! variable substitution (`${ENV_VAR}` syntax) and JSON Schema validation.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

pub mod agents;
pub mod opencode;

/// Error types for configuration operations.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// The API key is missing from both config and environment variables.
    #[error(
        "missing API key for provider '{0}': set the {1} environment variable or add it to config"
    )]
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

/// Wire protocol used to talk to a provider.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderProtocol {
    /// Anthropic Messages API.
    #[serde(rename = "anthropic-messages")]
    AnthropicMessages,
    /// OpenAI Chat Completions compatible API.
    #[default]
    #[serde(rename = "openai-chat")]
    OpenAIChat,
}

/// Per-model runtime limits.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct ModelConfig {
    /// Maximum provider input context accepted by this model.
    #[serde(default)]
    pub context_limit: Option<u32>,
    /// Maximum output tokens to request from this model.
    #[serde(default)]
    pub output_limit: Option<u32>,
}

/// Named provider configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct ProviderConfig {
    #[serde(default)]
    pub protocol: ProviderProtocol,
    #[serde(default)]
    pub tool_protocol: talos_core::tool::ToolProtocol,
    #[serde(default)]
    pub base_url: Option<String>,
    /// Inline API key written directly in the config file.
    ///
    /// When set, takes precedence over the env-var lookup. The field is
    /// `skip_serializing`, so calling `toml::to_string(&config)` on a
    /// deserialized config will not echo the key back into the output.
    /// Set `chmod 600` on the config file if you use this field.
    #[serde(default, skip_serializing)]
    pub api_key: Option<String>,
    /// Environment variable containing the API key. Used as a fallback when
    /// `api_key` is not set.
    #[serde(default)]
    pub api_key_env: Option<String>,
    /// Provider-specific model configuration keyed by model name.
    #[serde(default)]
    pub models: HashMap<String, ModelConfig>,
}

/// Talos configuration.
///
/// Contains the model provider, model name, and optional API key.
/// API keys can be specified in the config file or via environment variables.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Config {
    /// Active provider name. Built-ins include `anthropic` and `openai`.
    #[serde(default = "default_provider_name")]
    pub provider: String,

    /// The model name to use (e.g., `claude-sonnet-4-20250514`).
    #[serde(default)]
    pub model: String,

    /// Named provider definitions.
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,

    /// Logging configuration.
    #[serde(default)]
    pub log: LogConfig,

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

fn default_provider_name() -> String {
    "anthropic".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            provider: default_provider_name(),
            model: String::new(),
            providers: HashMap::new(),
            log: LogConfig::default(),
            hooks: HookConfig::default(),
            mcp: McpConfig::default(),
            rpc: RpcConfig::default(),
        }
    }
}

/// Logging configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct LogConfig {
    /// Default logging level used when neither `RUST_LOG` nor `filter` is set.
    #[serde(default)]
    pub level: Option<String>,

    /// Output format for console/log-file subscribers.
    #[serde(default)]
    pub format: LogFormat,

    /// Full tracing filter expression. Overrides `level` when set.
    #[serde(default)]
    pub filter: Option<String>,
}

/// Supported log output formats for the R1 logging baseline.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// Human-readable tracing output.
    #[default]
    Pretty,
    /// Compact single-line tracing output.
    Compact,
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
    // I009-S3 begin
    /// Stable MCP server name.
    pub name: String,
    /// Transport kind (`stdio` or `http`).
    pub transport: String,
    /// Executable command for stdio transport.
    #[serde(default)]
    pub command: String,
    /// Command arguments for stdio transport.
    #[serde(default)]
    pub args: Vec<String>,
    /// Environment variables for stdio transport.
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Working directory for stdio transport.
    #[serde(default)]
    pub cwd: Option<PathBuf>,
    // I009-S3 end
}

/// JSON-RPC server configuration placeholder for I009-S5.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RpcConfig {
    /// Allowed RPC methods.
    #[serde(default)]
    pub methods_allowlist: Vec<String>,
    /// Maximum number of concurrent RPC requests.
    ///
    /// MVP is serialized request handling, so this defaults to `1`.
    #[serde(default = "RpcConfig::default_max_concurrent")]
    pub max_concurrent: usize,
}

impl RpcConfig {
    fn default_max_concurrent() -> usize {
        1
    }
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            methods_allowlist: Vec::new(),
            max_concurrent: Self::default_max_concurrent(),
        }
    }
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
        let config: Config =
            toml::from_str(&substituted).map_err(|e| ConfigError::ParseError(e.to_string()))?;

        config.validate()?;
        Ok(config)
    }

    /// Returns the API key for the current provider.
    ///
    /// Resolution order:
    /// 1. Inline `providers.<name>.api_key` from the config file.
    /// 2. Provider-specific env var from `providers.<name>.api_key_env`.
    /// 3. Built-in provider env var: `ANTHROPIC_API_KEY` or `OPENAI_API_KEY`.
    /// 4. For the built-in OpenAI provider only, additionally `OPENAI_COMPAT_API_KEY`.
    ///    This is the conventional name for keys issued by OpenAI-compatible
    ///    gateways (DashScope / Bailian / Z.ai / self-hosted). It is checked
    ///    after `OPENAI_API_KEY` so existing setups are unaffected.
    pub fn api_key(&self) -> Result<String, ConfigError> {
        let provider = self.active_provider_config();

        if let Some(key) = provider.api_key.as_deref()
            && !key.is_empty()
        {
            return Ok(key.to_string());
        }

        let primary = provider
            .api_key_env
            .as_deref()
            .unwrap_or(match self.provider.as_str() {
                "anthropic" => "ANTHROPIC_API_KEY",
                "openai" => "OPENAI_API_KEY",
                _ => "",
            });

        if !primary.is_empty()
            && let Ok(key) = env::var(primary)
            && !key.is_empty()
        {
            return Ok(key);
        }

        if self.provider == "openai"
            && let Ok(key) = env::var("OPENAI_COMPAT_API_KEY")
            && !key.is_empty()
        {
            return Ok(key);
        }

        Err(ConfigError::MissingApiKey(
            self.provider.clone(),
            if self.provider == "openai" {
                "OPENAI_API_KEY or OPENAI_COMPAT_API_KEY".to_string()
            } else if primary.is_empty() {
                format!("providers.{}.api_key or api_key_env", self.provider)
            } else {
                primary.to_string()
            },
        ))
    }

    /// Returns the optional base URL override.
    ///
    /// `None` means "use the provider's hard-coded default endpoint".
    /// `Some(url)` is sent verbatim to the provider's HTTP client via
    /// `with_base_url()`. Honored for both OpenAI and Anthropic providers.
    pub fn base_url(&self) -> Option<String> {
        self.providers
            .get(&self.provider)
            .and_then(|p| p.base_url.clone())
            .or_else(|| builtin_provider_config(&self.provider).and_then(|p| p.base_url))
    }

    /// Returns the active provider protocol.
    #[must_use]
    pub fn provider_protocol(&self) -> ProviderProtocol {
        self.active_provider_config().protocol
    }

    #[must_use]
    pub fn tool_protocol(&self) -> talos_core::tool::ToolProtocol {
        self.active_provider_config().tool_protocol
    }

    /// Returns the configured context limit for the active provider/model.
    #[must_use]
    pub fn context_limit(&self) -> Option<u32> {
        self.active_model_config()
            .and_then(|model| model.context_limit)
    }

    /// Returns the configured output limit for the active provider/model.
    #[must_use]
    pub fn output_limit(&self) -> Option<u32> {
        self.active_model_config()
            .and_then(|model| model.output_limit)
    }

    /// Returns the active provider config with built-in defaults applied.
    #[must_use]
    pub fn active_provider_config(&self) -> ProviderConfig {
        let mut config = builtin_provider_config(&self.provider).unwrap_or_default();
        if let Some(user_config) = self.providers.get(&self.provider) {
            if !matches!(user_config.protocol, ProviderProtocol::OpenAIChat)
                || builtin_provider_config(&self.provider).is_none()
            {
                config.protocol = user_config.protocol.clone();
            }
            if user_config.base_url.is_some() {
                config.base_url = user_config.base_url.clone();
            }
            if user_config.api_key.is_some() {
                config.api_key = user_config.api_key.clone();
            }
            if user_config.api_key_env.is_some() {
                config.api_key_env = user_config.api_key_env.clone();
            }
            config.models.extend(user_config.models.clone());
        }
        config
    }

    fn active_model_config(&self) -> Option<ModelConfig> {
        self.active_provider_config()
            .models
            .get(&self.model)
            .cloned()
    }

    /// Validates the configuration against its JSON Schema.
    ///
    /// The schema is generated via `schemars` and the same constraints are
    /// enforced manually (model must be non-empty, provider must be valid).
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Generate schema for external tooling (documentation, IDE support)
        let _schema = schemars::schema_for!(Config);

        // Manual validation of required constraints
        if self.provider.trim().is_empty() {
            return Err(ConfigError::InvalidConfig(
                "'provider' is required and must be non-empty".to_string(),
            ));
        }

        if self.model.trim().is_empty() {
            return Err(ConfigError::InvalidConfig(
                "'model' is required and must be non-empty".to_string(),
            ));
        }

        let provider = self.active_provider_config();
        if self.providers.contains_key(&self.provider)
            && provider.api_key.is_none()
            && provider.api_key_env.is_none()
        {
            return Err(ConfigError::InvalidConfig(format!(
                "provider '{}' must set api_key or api_key_env",
                self.provider
            )));
        }

        Ok(())
    }

    /// Import opencode-style provider definitions and merge them into this config.
    ///
    /// Imports are one-way: opencode config is translated into Talos
    /// `ProviderConfig` values, but Talos config remains the source of truth
    /// after import.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::ParseError`] if the input is not valid JSON or
    /// does not match the expected opencode provider schema.
    pub fn import_opencode_providers(&mut self, json: &str) -> Result<(), ConfigError> {
        let imported = opencode::import_opencode_providers(json)?;
        self.providers.extend(imported);
        Ok(())
    }
}

fn builtin_provider_config(name: &str) -> Option<ProviderConfig> {
    match name {
        "anthropic" => Some(ProviderConfig {
            protocol: ProviderProtocol::AnthropicMessages,
            tool_protocol: Default::default(),
            base_url: None,
            api_key: None,
            api_key_env: Some("ANTHROPIC_API_KEY".to_string()),
            models: HashMap::from([
                (
                    "claude-sonnet-4-20250514".to_string(),
                    ModelConfig {
                        context_limit: Some(200_000),
                        output_limit: Some(4096),
                    },
                ),
                (
                    "claude-opus-4-20250514".to_string(),
                    ModelConfig {
                        context_limit: Some(200_000),
                        output_limit: Some(4096),
                    },
                ),
            ]),
        }),
        "openai" => Some(ProviderConfig {
            protocol: ProviderProtocol::OpenAIChat,
            tool_protocol: Default::default(),
            base_url: None,
            api_key: None,
            api_key_env: Some("OPENAI_API_KEY".to_string()),
            models: HashMap::from([
                (
                    "gpt-4o".to_string(),
                    ModelConfig {
                        context_limit: Some(128_000),
                        output_limit: Some(4096),
                    },
                ),
                (
                    "gpt-4o-mini".to_string(),
                    ModelConfig {
                        context_limit: Some(128_000),
                        output_limit: Some(4096),
                    },
                ),
            ]),
        }),
        _ => None,
    }
}

/// Returns the user's home directory.
pub(crate) fn home_dir() -> PathBuf {
    if let Some(home) = env::var("HOME").ok().filter(|h| !h.is_empty()) {
        return PathBuf::from(home);
    }
    if let Some(profile) = env::var("USERPROFILE").ok().filter(|p| !p.is_empty()) {
        return PathBuf::from(profile);
    }
    let drive = env::var("HOMEDRIVE").unwrap_or_default();
    let path = env::var("HOMEPATH").unwrap_or_default();
    if !drive.is_empty() && !path.is_empty() {
        return PathBuf::from(format!("{drive}{path}"));
    }
    PathBuf::from(".")
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
#[allow(warnings)]
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
        assert_eq!(config.provider, "anthropic");
        assert!(config.model.is_empty());
        assert!(config.providers.is_empty());
        assert_eq!(config.log, LogConfig::default());
        assert_eq!(
            config.provider_protocol(),
            ProviderProtocol::AnthropicMessages
        );
    }

    #[test]
    fn test_api_key_from_env_anthropic() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe { env::set_var("ANTHROPIC_API_KEY", "env-key-anthropic") };
        let config = Config {
            provider: "anthropic".to_string(),
            model: "claude-test".to_string(),
            providers: HashMap::new(),
            log: LogConfig::default(),
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
            provider: "openai".to_string(),
            model: "gpt-test".to_string(),
            providers: HashMap::new(),
            log: LogConfig::default(),
            hooks: HookConfig::default(),
            mcp: McpConfig::default(),
            rpc: RpcConfig::default(),
        };
        assert_eq!(config.api_key().unwrap(), "env-key-openai");
        unsafe { env::remove_var("OPENAI_API_KEY") };
    }

    #[test]
    fn test_api_key_from_env_openai_compat() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe { env::remove_var("OPENAI_API_KEY") };
        unsafe { env::set_var("OPENAI_COMPAT_API_KEY", "bailian-style-key") };
        let config = Config {
            provider: "openai".to_string(),
            model: "glm-5".to_string(),
            providers: HashMap::new(),
            log: LogConfig::default(),
            hooks: HookConfig::default(),
            mcp: McpConfig::default(),
            rpc: RpcConfig::default(),
        };
        assert_eq!(config.api_key().unwrap(), "bailian-style-key");
        unsafe { env::remove_var("OPENAI_COMPAT_API_KEY") };
    }

    #[test]
    fn test_api_key_openai_prefers_explicit_env_over_compat_env() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe { env::set_var("OPENAI_API_KEY", "real-openai-key") };
        unsafe { env::set_var("OPENAI_COMPAT_API_KEY", "bailian-key") };
        let config = Config {
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            providers: HashMap::new(),
            log: LogConfig::default(),
            hooks: HookConfig::default(),
            mcp: McpConfig::default(),
            rpc: RpcConfig::default(),
        };
        assert_eq!(config.api_key().unwrap(), "real-openai-key");
        unsafe { env::remove_var("OPENAI_API_KEY") };
        unsafe { env::remove_var("OPENAI_COMPAT_API_KEY") };
    }

    #[test]
    fn test_api_key_anthropic_does_not_check_openai_compat_env() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe { env::remove_var("ANTHROPIC_API_KEY") };
        unsafe { env::set_var("OPENAI_COMPAT_API_KEY", "should-not-be-used") };
        let config = Config {
            provider: "anthropic".to_string(),
            model: "claude-test".to_string(),
            providers: HashMap::new(),
            log: LogConfig::default(),
            hooks: HookConfig::default(),
            mcp: McpConfig::default(),
            rpc: RpcConfig::default(),
        };
        let err = config.api_key().unwrap_err();
        assert!(matches!(err, ConfigError::MissingApiKey(_, _)));
        let msg = err.to_string();
        assert!(msg.contains("ANTHROPIC_API_KEY"));
        assert!(!msg.contains("OPENAI_COMPAT_API_KEY"));
        unsafe { env::remove_var("OPENAI_COMPAT_API_KEY") };
    }

    #[test]
    fn test_api_key_missing_error() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe { env::remove_var("ANTHROPIC_API_KEY") };
        let config = Config {
            provider: "anthropic".to_string(),
            model: "claude-test".to_string(),
            providers: HashMap::new(),
            log: LogConfig::default(),
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
    fn test_base_url_getter() {
        let config = Config {
            provider: "dashscope".to_string(),
            model: "glm-5".to_string(),
            providers: HashMap::from([(
                "dashscope".to_string(),
                ProviderConfig {
                    protocol: ProviderProtocol::OpenAIChat,
                    tool_protocol: Default::default(),
                    base_url: Some("https://example.com/v1".to_string()),
                    api_key_env: Some("DASHSCOPE_API_KEY".to_string()),
                    ..Default::default()
                },
            )]),
            log: LogConfig::default(),
            hooks: HookConfig::default(),
            mcp: McpConfig::default(),
            rpc: RpcConfig::default(),
        };
        assert_eq!(config.base_url().as_deref(), Some("https://example.com/v1"));
    }

    #[test]
    fn test_base_url_default_is_none() {
        let config = Config::default();
        assert_eq!(config.base_url(), None);
    }

    #[test]
    fn test_base_url_parsed_from_toml() {
        let toml_str = r#"
            provider = "dashscope"
            model = "glm-5"

            [providers.dashscope]
            protocol = "openai-chat"
            base_url = "https://token-plan.cn-beijing.maas.aliyuncs.com/compatible-mode/v1"
            api_key_env = "DASHSCOPE_API_KEY"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(
            config.base_url().as_deref(),
            Some("https://token-plan.cn-beijing.maas.aliyuncs.com/compatible-mode/v1")
        );
    }

    #[test]
    fn test_custom_provider_api_key_env() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe { env::set_var("DASHSCOPE_API_KEY", "dashscope-key") };
        let config = Config {
            provider: "dashscope".to_string(),
            model: "glm-5".to_string(),
            providers: HashMap::from([(
                "dashscope".to_string(),
                ProviderConfig {
                    protocol: ProviderProtocol::OpenAIChat,
                    tool_protocol: Default::default(),
                    base_url: Some("https://example.com/v1".to_string()),
                    api_key_env: Some("DASHSCOPE_API_KEY".to_string()),
                    ..Default::default()
                },
            )]),
            log: LogConfig::default(),
            hooks: HookConfig::default(),
            mcp: McpConfig::default(),
            rpc: RpcConfig::default(),
        };

        assert_eq!(config.api_key().unwrap(), "dashscope-key");
        unsafe { env::remove_var("DASHSCOPE_API_KEY") };
    }

    #[test]
    fn test_model_limits_from_builtin_and_custom_providers() {
        let builtin = Config {
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            providers: HashMap::new(),
            log: LogConfig::default(),
            hooks: HookConfig::default(),
            mcp: McpConfig::default(),
            rpc: RpcConfig::default(),
        };
        assert_eq!(builtin.context_limit(), Some(128_000));

        let custom = Config {
            provider: "dashscope".to_string(),
            model: "glm-5".to_string(),
            providers: HashMap::from([(
                "dashscope".to_string(),
                ProviderConfig {
                    protocol: ProviderProtocol::OpenAIChat,
                    tool_protocol: Default::default(),
                    base_url: Some("https://example.com/v1".to_string()),
                    api_key: None,
                    api_key_env: Some("DASHSCOPE_API_KEY".to_string()),
                    models: HashMap::from([(
                        "glm-5".to_string(),
                        ModelConfig {
                            context_limit: Some(202_752),
                            output_limit: Some(4096),
                        },
                    )]),
                },
            )]),
            log: LogConfig::default(),
            hooks: HookConfig::default(),
            mcp: McpConfig::default(),
            rpc: RpcConfig::default(),
        };
        assert_eq!(custom.context_limit(), Some(202_752));
        assert_eq!(custom.output_limit(), Some(4096));
    }

    #[test]
    fn test_log_config_parsed_from_toml() {
        let toml_str = r#"
            provider = "openai"
            model = "glm-5"

            [log]
            level = "warn"
            format = "compact"
            filter = "talos_provider=debug,talos_agent=info"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.log.level.as_deref(), Some("warn"));
        assert_eq!(config.log.format, LogFormat::Compact);
        assert_eq!(
            config.log.filter.as_deref(),
            Some("talos_provider=debug,talos_agent=info")
        );
    }

    #[test]
    fn test_log_config_defaults() {
        let config = Config::default();
        assert_eq!(config.log.level, None);
        assert_eq!(config.log.format, LogFormat::Pretty);
        assert_eq!(config.log.filter, None);
    }

    #[test]
    fn test_load_nonexistent_file() {
        let path = Config::default_path();
        if path.exists() {
            return;
        }
        let config = Config::load().unwrap();
        assert_eq!(config.provider, "anthropic");
        assert!(config.model.is_empty());
    }

    #[test]
    fn test_provider_serialization() {
        let config_anthropic = Config {
            provider: "anthropic".to_string(),
            model: "test".to_string(),
            providers: HashMap::new(),
            log: LogConfig::default(),
            hooks: HookConfig::default(),
            mcp: McpConfig::default(),
            rpc: RpcConfig::default(),
        };
        let config_openai = Config {
            provider: "openai".to_string(),
            model: "test".to_string(),
            providers: HashMap::new(),
            log: LogConfig::default(),
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
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.provider, "openai");
        assert_eq!(config.model, "gpt-4");
    }

    #[test]
    fn test_inline_api_key_parsed_from_toml() {
        let toml_str = r#"
            provider = "dashscope"
            model = "glm-5"

            [providers.dashscope]
            protocol = "openai-chat"
            base_url = "https://example.com/v1"
            api_key = "sk-inline-secret"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.api_key().unwrap(), "sk-inline-secret");
    }

    #[test]
    fn test_inline_api_key_precedence_over_env() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe { env::set_var("DASHSCOPE_API_KEY", "env-key-should-not-be-used") };
        let config: Config = toml::from_str(
            r#"
            provider = "dashscope"
            model = "glm-5"

            [providers.dashscope]
            protocol = "openai-chat"
            api_key = "inline-key-wins"
            api_key_env = "DASHSCOPE_API_KEY"
        "#,
        )
        .unwrap();
        assert_eq!(config.api_key().unwrap(), "inline-key-wins");
        unsafe { env::remove_var("DASHSCOPE_API_KEY") };
    }

    #[test]
    fn test_inline_api_key_anthropic_overrides_builtin() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe { env::remove_var("ANTHROPIC_API_KEY") };
        let config: Config = toml::from_str(
            r#"
            provider = "anthropic"
            model = "claude-test"

            [providers.anthropic]
            api_key = "inline-anthropic-key"
        "#,
        )
        .unwrap();
        assert_eq!(config.api_key().unwrap(), "inline-anthropic-key");
    }

    #[test]
    fn test_validate_accepts_either_api_key_or_api_key_env() {
        let with_inline = Config {
            provider: "custom".to_string(),
            model: "model-x".to_string(),
            providers: HashMap::from([(
                "custom".to_string(),
                ProviderConfig {
                    protocol: ProviderProtocol::OpenAIChat,
                    tool_protocol: Default::default(),
                    api_key: Some("inline".to_string()),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };
        assert!(with_inline.validate().is_ok());

        let with_env = Config {
            provider: "custom".to_string(),
            model: "model-x".to_string(),
            providers: HashMap::from([(
                "custom".to_string(),
                ProviderConfig {
                    protocol: ProviderProtocol::OpenAIChat,
                    tool_protocol: Default::default(),
                    api_key_env: Some("CUSTOM_KEY".to_string()),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };
        assert!(with_env.validate().is_ok());
    }

    #[test]
    fn test_validate_rejects_neither_api_key_nor_api_key_env() {
        let config = Config {
            provider: "custom".to_string(),
            model: "model-x".to_string(),
            providers: HashMap::from([(
                "custom".to_string(),
                ProviderConfig {
                    protocol: ProviderProtocol::OpenAIChat,
                    tool_protocol: Default::default(),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("api_key or api_key_env"));
    }

    #[test]
    fn test_inline_api_key_not_serialized_back() {
        let config: Config = toml::from_str(
            r#"
            provider = "dashscope"
            model = "glm-5"

            [providers.dashscope]
            protocol = "openai-chat"
            api_key = "sk-very-secret"
            api_key_env = "DASHSCOPE_API_KEY"
        "#,
        )
        .unwrap();
        let serialized = toml::to_string(&config).unwrap();
        assert!(!serialized.contains("sk-very-secret"));
        assert!(!serialized.contains("api_key ="));
    }
}
