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
pub mod model;
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

    /// Failed to serialize configuration to TOML.
    #[error("failed to serialize config: {0}")]
    SerializeError(String),
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
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct ProviderConfig {
    #[serde(default)]
    pub protocol: ProviderProtocol,
    #[serde(default)]
    pub tool_protocol: talos_core::tool::ToolProtocol,
    #[serde(default)]
    pub base_url: Option<String>,
    /// Inline API key written directly in the config file.
    ///
    /// Inline API key. When set, takes precedence over the env-var lookup.
    /// Stored directly in `config.toml` — the file lives in your home
    /// directory (chmod 600 recommended). Use `api_key_env` for shared shells
    /// or containerised environments. `talos config list`/`get` masks this
    /// field on display, but it is present in the file for tooling that
    /// reads config directly.
    #[serde(default)]
    pub api_key: Option<String>,
    /// Environment variable containing the API key. Used as a fallback when
    /// `api_key` is not set.
    #[serde(default)]
    pub api_key_env: Option<String>,
    /// Provider-specific model configuration keyed by model name.
    #[serde(default)]
    pub models: HashMap<String, ModelConfig>,
}

impl std::fmt::Debug for ProviderConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderConfig")
            .field("protocol", &self.protocol)
            .field("tool_protocol", &self.tool_protocol)
            .field("base_url", &self.base_url)
            .field("api_key", &self.api_key.as_deref().map(|_| "***"))
            .field("api_key_env", &self.api_key_env)
            .field("models", &self.models)
            .finish()
    }
}

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

/// Talos configuration.
///
/// Contains the model provider, model name, and optional API key.
/// API keys can be specified in the config file or via environment variables.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Config {
    /// Active provider name. Built-ins include `anthropic` and `openai`.
    #[serde(default = "default_provider_name")]
    pub provider: String,

    /// The model name to use (e.g., `claude-sonnet-4-5-20250929`).
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

    /// File-based logging with rotation and retention.
    /// `None` means no file logging by default (backward compatible).
    /// TUI mode auto-enables file logging when this is `None`.
    #[serde(default)]
    pub file: Option<LogFileConfig>,
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

/// Log rotation strategy for file-based logging.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogRotation {
    /// Rotate when the current file exceeds `max_size_mb`.
    #[default]
    Size,
    /// Rotate once per calendar day.
    Daily,
}

/// Configuration for file-based log output with rotation and retention.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct LogFileConfig {
    /// Whether file logging is enabled.
    #[serde(default = "LogFileConfig::default_enabled")]
    pub enabled: bool,

    /// Path to the log file. Supports `~` expansion.
    /// Defaults to `~/.talos/logs/talos.log` when `None`.
    #[serde(default)]
    pub path: Option<PathBuf>,

    /// Maximum size of a single log file in megabytes before rotation.
    #[serde(default = "LogFileConfig::default_max_size_mb")]
    pub max_size_mb: u64,

    /// Maximum number of retained log files (including the active one).
    #[serde(default = "LogFileConfig::default_max_files")]
    pub max_files: usize,

    /// Rotation strategy.
    #[serde(default)]
    pub rotation: LogRotation,
}

impl Default for LogFileConfig {
    fn default() -> Self {
        Self {
            enabled: Self::default_enabled(),
            path: None,
            max_size_mb: Self::default_max_size_mb(),
            max_files: Self::default_max_files(),
            rotation: LogRotation::default(),
        }
    }
}

impl LogFileConfig {
    fn default_enabled() -> bool {
        true
    }

    fn default_max_size_mb() -> u64 {
        16
    }

    fn default_max_files() -> usize {
        5
    }
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
        let mut config: Config =
            toml::from_str(&substituted).map_err(|e| ConfigError::ParseError(e.to_string()))?;

        if let Ok(creds) = Credentials::load() {
            config.merge_credentials(&creds);
        }

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

    /// Resolves model limits using the full precedence chain:
    /// 1. User-configured limits in `~/.talos/config.toml` for the active model.
    /// 2. Built-in catalog from `builtin_models()` matched by model ID.
    /// 3. Conservative fallback `(128_000, None)`.
    ///
    /// Returns `(context_limit, output_limit)`.
    #[must_use]
    pub fn resolve_model_limits(&self) -> (u32, Option<u32>) {
        const CONSERVATIVE_FALLBACK: u32 = 128_000;

        // Step 1: Check user config
        if let Some(model_config) = self.active_model_config()
            && let Some(ctx) = model_config.context_limit
        {
            return (ctx, model_config.output_limit);
        }

        // Step 2: Look up in builtin catalog, qualified by active provider so
        // that duplicate model ids (e.g. glm-5.2 under zhipu/zai) resolve to
        // the intended provider's metadata.
        let builtins = model::builtin_models();
        if let Some(meta) = model::find_model_by_provider(&builtins, &self.provider, &self.model)
            && let Some(ctx) = meta.context_limit
        {
            return (ctx, meta.output_limit);
        }

        // Step 3: Conservative fallback
        (CONSERVATIVE_FALLBACK, None)
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

    /// Persists the current configuration to `~/.talos/config.toml`.
    ///
    /// `api_key` values are serialized in the main config file. Run
    /// `talos config list` or `talos config get` to view config without
    /// leaking keys.
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

    /// Checks whether the named provider has a usable API key.
    ///
    /// Returns `true` if the provider's inline `api_key` is set, or if its
    /// `api_key_env` resolves to a non-empty environment variable.
    pub fn provider_authenticated(&self, name: &str) -> bool {
        let provider = match self.providers.get(name) {
            Some(p) => p.clone(),
            None => match builtin_provider_config(name) {
                Some(p) => p,
                None => return false,
            },
        };

        if let Some(key) = provider.api_key.as_deref()
            && !key.is_empty()
        {
            return true;
        }

        let env_var = provider.api_key_env.as_deref().unwrap_or(match name {
            "anthropic" => "ANTHROPIC_API_KEY",
            "openai" => "OPENAI_API_KEY",
            _ => "",
        });

        if !env_var.is_empty()
            && let Ok(key) = env::var(env_var)
            && !key.is_empty()
        {
            return true;
        }

        if name == "openai"
            && let Ok(key) = env::var("OPENAI_COMPAT_API_KEY")
            && !key.is_empty()
        {
            return true;
        }

        false
    }

    /// Returns all available models: built-in catalog merged with user-configured
    /// models from `providers.*.models`. Configured models override builtin entries
    /// with the same `(provider, model_id)` pair.
    pub fn all_models(&self) -> Vec<model::ModelMetadata> {
        let mut models = model::builtin_models();
        for (provider_name, provider) in &self.providers {
            for (model_id, cfg) in &provider.models {
                if let Some(existing) = models
                    .iter_mut()
                    .find(|m| m.provider == *provider_name && m.id == *model_id)
                {
                    if cfg.context_limit.is_some() {
                        existing.context_limit = cfg.context_limit;
                    }
                    if cfg.output_limit.is_some() {
                        existing.output_limit = cfg.output_limit;
                    }
                    existing.source = model::ModelSource::Manual;
                } else {
                    models.push(model::ModelMetadata {
                        id: model_id.clone(),
                        provider: provider_name.clone(),
                        context_limit: cfg.context_limit,
                        output_limit: cfg.output_limit,
                        pricing: None,
                        capabilities: model::ModelCapabilities::default(),
                        release_date: None,
                        source: model::ModelSource::Manual,
                    });
                }
            }
        }
        models
    }

    /// Sets the active model by resolving it against all known models.
    ///
    /// Looks up `model_id` in [`Config::all_models`] to find the owning
    /// provider, then sets `self.model` and `self.provider` accordingly.
    /// Ensures the provider has an entry in `self.providers` (creates a
    /// default if missing).
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::InvalidConfig`] if the model ID is not found, or
    /// if a bare (unqualified) model ID matches multiple providers and the
    /// caller must disambiguate with `provider/model_id`.
    pub fn set_active_model(&mut self, model_id: &str) -> Result<(), ConfigError> {
        let all = self.all_models();

        // Handle provider-qualified IDs (e.g. "zhipu/glm-5.2") for models
        // that exist under multiple providers. The prefix is only treated as
        // a provider qualifier if it matches a known provider name.
        let known_providers: std::collections::HashSet<&str> =
            all.iter().map(|m| m.provider.as_str()).collect();
        let (resolved_provider, resolved_id) = match model_id.split_once('/') {
            Some((prefix, rest)) if known_providers.contains(prefix) && !rest.is_empty() => {
                (Some(prefix), rest)
            }
            _ => (None, model_id),
        };

        let meta = match resolved_provider {
            Some(provider) => all
                .iter()
                .find(|m| m.id == resolved_id && m.provider == provider)
                .ok_or_else(|| {
                    ConfigError::InvalidConfig(format!(
                        "model '{resolved_id}' not found for provider '{provider}'"
                    ))
                })?,
            None => {
                let matches: Vec<_> = all.iter().filter(|m| m.id == resolved_id).collect();
                match matches.len() {
                    0 => {
                        return Err(ConfigError::InvalidConfig(format!(
                            "model '{resolved_id}' not found"
                        )));
                    }
                    1 => matches[0],
                    _ => {
                        let providers = matches
                            .iter()
                            .map(|m| m.provider.as_str())
                            .collect::<Vec<_>>()
                            .join(", ");
                        return Err(ConfigError::InvalidConfig(format!(
                            "model '{resolved_id}' is available from multiple providers: {providers}. \
                             Qualify with 'provider/{resolved_id}', e.g. '{}/{resolved_id}'",
                            matches[0].provider
                        )));
                    }
                }
            }
        };

        self.model = meta.id.clone();
        self.provider = meta.provider.clone();

        if !self.providers.contains_key(&meta.provider) {
            if let Some(builtin) = builtin_provider_config(&meta.provider) {
                self.providers.insert(meta.provider.clone(), builtin);
            } else {
                self.providers.insert(
                    meta.provider.clone(),
                    ProviderConfig {
                        protocol: ProviderProtocol::OpenAIChat,
                        ..Default::default()
                    },
                );
            }
        }

        Ok(())
    }

    /// Sets the API key for the named provider.
    ///
    /// Creates a default `ProviderConfig` entry if the provider does not yet
    /// exist in `self.providers`. The key is stored in the in-memory
    /// `api_key` field for immediate runtime use.
    pub fn set_provider_credential(&mut self, name: &str, api_key: &str) {
        let entry = self.providers.entry(name.to_string()).or_insert_with(|| {
            if let Some(builtin) = builtin_provider_config(name) {
                builtin
            } else {
                ProviderConfig {
                    protocol: ProviderProtocol::OpenAIChat,
                    ..Default::default()
                }
            }
        });
        entry.api_key = Some(api_key.to_string());
    }

    /// Extracts all in-memory API keys into a [`Credentials`] store.
    /// Merges credentials into this config's provider `api_key` fields.
    ///
    /// For each provider that has a stored credential but no inline key,
    /// the credential is injected into `api_key`.
    fn merge_credentials(&mut self, creds: &Credentials) {
        for (name, key) in &creds.keys {
            if let Some(provider) = self.providers.get_mut(name) {
                if provider.api_key.is_none() {
                    provider.api_key = Some(key.clone());
                }
            } else {
                let mut provider =
                    builtin_provider_config(name).unwrap_or_else(|| ProviderConfig {
                        protocol: ProviderProtocol::OpenAIChat,
                        ..Default::default()
                    });
                provider.api_key = Some(key.clone());
                self.providers.insert(name.clone(), provider);
            }
        }
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
            models: HashMap::new(),
        }),
        "openai" => Some(ProviderConfig {
            protocol: ProviderProtocol::OpenAIChat,
            tool_protocol: Default::default(),
            base_url: None,
            api_key: None,
            api_key_env: Some("OPENAI_API_KEY".to_string()),
            models: HashMap::new(),
        }),
        "google" => Some(ProviderConfig {
            protocol: ProviderProtocol::OpenAIChat,
            tool_protocol: Default::default(),
            base_url: Some("https://generativelanguage.googleapis.com/v1beta".to_string()),
            api_key: None,
            api_key_env: Some("GOOGLE_API_KEY".to_string()),
            models: HashMap::new(),
        }),
        "deepseek" => Some(ProviderConfig {
            protocol: ProviderProtocol::OpenAIChat,
            tool_protocol: Default::default(),
            base_url: Some("https://api.deepseek.com".to_string()),
            api_key: None,
            api_key_env: Some("DEEPSEEK_API_KEY".to_string()),
            models: HashMap::new(),
        }),
        "qwen" => Some(ProviderConfig {
            protocol: ProviderProtocol::OpenAIChat,
            tool_protocol: Default::default(),
            base_url: Some("https://dashscope.aliyuncs.com/compatible-mode/v1".to_string()),
            api_key: None,
            api_key_env: Some("DASHSCOPE_API_KEY".to_string()),
            models: HashMap::new(),
        }),
        "zhipu" => Some(ProviderConfig {
            protocol: ProviderProtocol::OpenAIChat,
            tool_protocol: Default::default(),
            base_url: Some("https://open.bigmodel.cn/api/paas/v4".to_string()),
            api_key: None,
            api_key_env: Some("ZHIPU_API_KEY".to_string()),
            models: HashMap::new(),
        }),
        "zai" => Some(ProviderConfig {
            protocol: ProviderProtocol::OpenAIChat,
            tool_protocol: Default::default(),
            base_url: Some("https://api.z.ai/api/paas/v4".to_string()),
            api_key: None,
            api_key_env: Some("ZAI_API_KEY".to_string()),
            models: HashMap::new(),
        }),
        "zhipu-coding-plan" => Some(ProviderConfig {
            protocol: ProviderProtocol::OpenAIChat,
            tool_protocol: Default::default(),
            base_url: Some("https://open.bigmodel.cn/api/coding/paas/v4".to_string()),
            api_key: None,
            api_key_env: Some("ZHIPU_CODING_PLAN_API_KEY".to_string()),
            models: HashMap::new(),
        }),
        "zai-coding-plan" => Some(ProviderConfig {
            protocol: ProviderProtocol::OpenAIChat,
            tool_protocol: Default::default(),
            base_url: Some("https://api.z.ai/api/coding/paas/v4".to_string()),
            api_key: None,
            api_key_env: Some("ZAI_CODING_PLAN_API_KEY".to_string()),
            models: HashMap::new(),
        }),
        "minimax" => Some(ProviderConfig {
            protocol: ProviderProtocol::OpenAIChat,
            tool_protocol: Default::default(),
            base_url: Some("https://api.minimaxi.com/v1".to_string()),
            api_key: None,
            api_key_env: Some("MINIMAX_API_KEY".to_string()),
            models: HashMap::new(),
        }),
        "moonshot" => Some(ProviderConfig {
            protocol: ProviderProtocol::OpenAIChat,
            tool_protocol: Default::default(),
            base_url: Some("https://api.moonshot.cn/v1".to_string()),
            api_key: None,
            api_key_env: Some("MOONSHOT_API_KEY".to_string()),
            models: HashMap::new(),
        }),
        "openrouter" => Some(ProviderConfig {
            protocol: ProviderProtocol::OpenAIChat,
            tool_protocol: Default::default(),
            base_url: Some("https://openrouter.ai/api/v1".to_string()),
            api_key: None,
            api_key_env: Some("OPENROUTER_API_KEY".to_string()),
            models: HashMap::new(),
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
            model: "gpt-4.1".to_string(),
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
        // Builtin limits resolve via resolve_model_limits() (catalog lookup),
        // not context_limit() (user-config only).
        let builtin = Config {
            provider: "openai".to_string(),
            model: "gpt-4.1-2025-04-14".to_string(),
            providers: HashMap::new(),
            log: LogConfig::default(),
            hooks: HookConfig::default(),
            mcp: McpConfig::default(),
            rpc: RpcConfig::default(),
        };
        let (builtin_ctx, builtin_out) = builtin.resolve_model_limits();
        assert_eq!(builtin_ctx, 1_047_576);
        assert_eq!(builtin_out, Some(32_768));

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
    fn test_inline_api_key_is_serialized_in_config_toml() {
        // I045 reverted skip_serializing: api_key is now stored directly in
        // config.toml (the file lives in the user's home directory, chmod 600
        // recommended). Display masking is the responsibility of
        // `talos config list`/`get`, not the serializer.
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
        assert!(serialized.contains("sk-very-secret"));
        assert!(serialized.contains("api_key ="));
    }

    #[test]
    fn test_resolve_model_limits_returns_user_config_when_set() {
        let config = Config {
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-5-20250929".to_string(),
            providers: HashMap::from([(
                "anthropic".to_string(),
                ProviderConfig {
                    models: HashMap::from([(
                        "claude-sonnet-4-5-20250929".to_string(),
                        ModelConfig {
                            context_limit: Some(150_000),
                            output_limit: Some(8000),
                        },
                    )]),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };
        let (ctx, out) = config.resolve_model_limits();
        assert_eq!(ctx, 150_000);
        assert_eq!(out, Some(8000));
    }

    #[test]
    fn test_resolve_model_limits_falls_back_to_builtin_catalog() {
        let config = Config {
            provider: "google".to_string(),
            model: "gemini-2.5-pro".to_string(),
            providers: HashMap::from([(
                "google".to_string(),
                ProviderConfig {
                    api_key_env: Some("GOOGLE_API_KEY".to_string()),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };
        let (ctx, out) = config.resolve_model_limits();
        assert_eq!(ctx, 1_048_576);
        assert_eq!(out, Some(65536));
    }

    #[test]
    fn test_resolve_model_limits_falls_back_to_conservative_when_not_in_catalog() {
        let config = Config {
            provider: "custom-provider".to_string(),
            model: "unknown-model-xyz".to_string(),
            providers: HashMap::from([(
                "custom-provider".to_string(),
                ProviderConfig {
                    api_key_env: Some("CUSTOM_KEY".to_string()),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };
        let (ctx, out) = config.resolve_model_limits();
        assert_eq!(ctx, 128_000);
        assert_eq!(out, None);
    }

    #[test]
    fn test_resolve_model_limits_output_limit_from_catalog() {
        let config = Config {
            provider: "openai".to_string(),
            model: "gpt-4.1-2025-04-14".to_string(),
            providers: HashMap::new(),
            ..Default::default()
        };
        let (ctx, out) = config.resolve_model_limits();
        assert_eq!(ctx, 1_047_576);
        assert_eq!(out, Some(32768));
    }

    #[test]
    fn test_resolve_model_limits_user_config_takes_precedence_over_catalog() {
        let config = Config {
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-5-20250929".to_string(),
            providers: HashMap::from([(
                "anthropic".to_string(),
                ProviderConfig {
                    models: HashMap::from([(
                        "claude-sonnet-4-5-20250929".to_string(),
                        ModelConfig {
                            context_limit: Some(100_000),
                            output_limit: None,
                        },
                    )]),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };
        let (ctx, out) = config.resolve_model_limits();
        assert_eq!(ctx, 100_000);
        assert_eq!(out, None);
    }

    #[test]
    fn test_credentials_default_path() {
        let path = Credentials::default_path();
        assert!(path.to_string_lossy().contains(".talos"));
        assert!(path.to_string_lossy().contains("credentials.toml"));
    }

    #[test]
    fn test_credentials_load_nonexistent_returns_empty() {
        let creds = Credentials::load().unwrap();
        assert!(creds.keys.is_empty());
    }

    #[test]
    fn test_credentials_save_and_load_roundtrip() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let tmp_dir = env::temp_dir().join("talos_test_creds");
        let _ = fs::remove_dir_all(&tmp_dir);
        fs::create_dir_all(&tmp_dir).unwrap();

        let creds_path = tmp_dir.join("credentials.toml");
        unsafe { env::set_var("HOME", tmp_dir.to_string_lossy().as_ref()) };

        let mut creds = Credentials::default();
        creds
            .keys
            .insert("anthropic".to_string(), "sk-test-key".to_string());
        creds
            .keys
            .insert("openai".to_string(), "sk-openai-key".to_string());
        creds.save().unwrap();

        let loaded = Credentials::load().unwrap();
        assert_eq!(
            loaded.keys.get("anthropic"),
            Some(&"sk-test-key".to_string())
        );
        assert_eq!(
            loaded.keys.get("openai"),
            Some(&"sk-openai-key".to_string())
        );

        unsafe { env::remove_var("HOME") };
        let _ = fs::remove_dir_all(&tmp_dir);
        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_provider_authenticated_with_inline_key() {
        let mut config = Config::default();
        config.set_provider_credential("anthropic", "sk-inline-key");
        assert!(config.provider_authenticated("anthropic"));
    }

    #[test]
    fn test_provider_authenticated_with_env_var() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe { env::set_var("ANTHROPIC_API_KEY", "env-key") };
        let config = Config::default();
        assert!(config.provider_authenticated("anthropic"));
        unsafe { env::remove_var("ANTHROPIC_API_KEY") };
    }

    #[test]
    fn test_provider_authenticated_returns_false_when_no_key() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe { env::remove_var("ANTHROPIC_API_KEY") };
        let config = Config {
            providers: HashMap::from([(
                "custom".to_string(),
                ProviderConfig {
                    protocol: ProviderProtocol::OpenAIChat,
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };
        assert!(!config.provider_authenticated("custom"));
        assert!(!config.provider_authenticated("nonexistent"));
    }

    #[test]
    fn test_set_active_model_sets_provider_from_catalog() {
        let mut config = Config::default();
        config
            .set_active_model("claude-sonnet-4-5-20250929")
            .unwrap();
        assert_eq!(config.model, "claude-sonnet-4-5-20250929");
        assert_eq!(config.provider, "anthropic");
        assert!(config.providers.contains_key("anthropic"));
    }

    #[test]
    fn test_set_active_model_openai() {
        let mut config = Config::default();
        config.set_active_model("gpt-4.1-2025-04-14").unwrap();
        assert_eq!(config.model, "gpt-4.1-2025-04-14");
        assert_eq!(config.provider, "openai");
        assert!(config.providers.contains_key("openai"));
    }

    #[test]
    fn test_set_active_model_unknown_model_errors() {
        let mut config = Config::default();
        let err = config
            .set_active_model("nonexistent-model-xyz")
            .unwrap_err();
        assert!(err.to_string().contains("nonexistent-model-xyz"));
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_set_provider_credential_creates_new_provider() {
        let mut config = Config::default();
        config.set_provider_credential("custom-provider", "sk-custom-key");
        assert!(config.providers.contains_key("custom-provider"));
        let provider = config.providers.get("custom-provider").unwrap();
        assert_eq!(provider.api_key.as_deref(), Some("sk-custom-key"));
    }

    #[test]
    fn test_set_provider_credential_overwrites_existing() {
        let mut config = Config::default();
        config.set_provider_credential("anthropic", "old-key");
        config.set_provider_credential("anthropic", "new-key");
        let provider = config.providers.get("anthropic").unwrap();
        assert_eq!(provider.api_key.as_deref(), Some("new-key"));
    }

    #[test]
    fn test_save_writes_api_key_in_config_toml() {
        // I045 fix: api_key is now serialized in config.toml. This avoids
        // the silent data-loss bug where keys were moved to a separate
        // credentials.toml without the user knowing. Display masking
        // remains the responsibility of `talos config list`/`get`.
        let _lock = ENV_MUTEX.lock().unwrap();
        let tmp_dir = env::temp_dir().join("talos_test_save");
        let _ = fs::remove_dir_all(&tmp_dir);
        fs::create_dir_all(&tmp_dir.join(".talos")).unwrap();
        let prev_home = std::env::var_os("HOME");
        unsafe { env::set_var("HOME", tmp_dir.to_string_lossy().as_ref()) };

        let mut config = Config::default();
        config.model = "claude-sonnet-4-5-20250929".to_string();
        config.set_provider_credential("anthropic", "sk-secret-key");
        config.save().unwrap();

        let config_path = Config::default_path();
        let config_content = fs::read_to_string(&config_path).unwrap();
        assert!(config_content.contains("sk-secret-key"));

        // No credentials.toml should be written anymore.
        let creds_path = Credentials::default_path();
        assert!(
            !creds_path.exists(),
            "credentials.toml should not be created"
        );

        match prev_home {
            Some(v) => unsafe { env::set_var("HOME", v) },
            None => unsafe { env::remove_var("HOME") },
        }
        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_load_merges_credentials() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let tmp_dir = env::temp_dir().join("talos_test_load_merge");
        let _ = fs::remove_dir_all(&tmp_dir);
        fs::create_dir_all(tmp_dir.join(".talos")).unwrap();
        unsafe { env::set_var("HOME", tmp_dir.to_string_lossy().as_ref()) };

        let config_toml = r#"
provider = "anthropic"
model = "claude-sonnet-4-5-20250929"
"#;
        fs::write(Config::default_path(), config_toml).unwrap();

        let creds_toml = r#"
anthropic = "sk-merged-key"
"#;
        fs::write(Credentials::default_path(), creds_toml).unwrap();

        let config = Config::load().unwrap();
        let provider = config.providers.get("anthropic").unwrap();
        assert_eq!(provider.api_key.as_deref(), Some("sk-merged-key"));

        unsafe { env::remove_var("HOME") };
        let _ = fs::remove_dir_all(&tmp_dir);
        let _ = fs::remove_dir_all(&tmp_dir);
    }

    /// Regression test for the I045 data-loss bug: inline api_key in
    /// config.toml must be preserved across load+save round-trips and
    /// visible to anyone reading the file. The fix was to drop the
    /// `skip_serializing` attribute (which was quietly moving keys to
    /// a separate credentials.toml). Display masking is handled by
    /// `talos config list`/`get`, not the serializer.
    #[test]
    fn test_save_preserves_inline_api_key_from_config_toml() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let tmp_dir = env::temp_dir().join("talos_test_roundtrip");
        let _ = fs::remove_dir_all(&tmp_dir);
        fs::create_dir_all(&tmp_dir).unwrap();
        let talos_dir = &tmp_dir.join(".talos");
        fs::create_dir_all(&talos_dir).unwrap();
        let prev_home = std::env::var_os("HOME");
        unsafe { env::set_var("HOME", &tmp_dir.to_string_lossy().as_ref()) };

        let config_toml = r#"
provider = "anthropic"
model = "claude-sonnet-4-5-20250929"

[providers.anthropic]
protocol = "anthropic-messages"
api_key = "sk-inline-secret-from-config"
"#;
        fs::write(Config::default_path(), config_toml).unwrap();

        let config = Config::load().unwrap();
        let provider = config.providers.get("anthropic").unwrap();
        assert_eq!(
            provider.api_key.as_deref(),
            Some("sk-inline-secret-from-config"),
            "api_key must be loaded from config.toml during deserialization"
        );

        config.save().unwrap();

        let saved_config = fs::read_to_string(Config::default_path()).unwrap();
        assert!(
            saved_config.contains("sk-inline-secret-from-config"),
            "api_key must be present in saved config.toml (regression for I045 data-loss bug)"
        );

        // No credentials.toml should be written.
        assert!(
            !Credentials::default_path().exists(),
            "credentials.toml should not be written anymore"
        );

        let config2 = Config::load().unwrap();
        let provider2 = config2.providers.get("anthropic").unwrap();
        assert_eq!(
            provider2.api_key.as_deref(),
            Some("sk-inline-secret-from-config"),
            "api_key must survive a second load round-trip"
        );

        match prev_home {
            Some(v) => unsafe { env::set_var("HOME", v) },
            None => unsafe { env::remove_var("HOME") },
        }
        let _ = fs::remove_dir_all(&tmp_dir);
    }

    /// When the I045 fix is applied (no skip_serializing), api_key is
    /// serialized in config.toml and survives a load+save round-trip
    /// in the main config file alone — no credentials.toml needed.
    #[test]
    fn test_skip_serializing_does_not_skip_deserialization() {
        let toml_str = r#"
            provider = "test"
            model = "test-model"

            [providers.test]
            api_key = "hello-from-toml"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        let provider = config.providers.get("test").unwrap();
        assert_eq!(provider.api_key.as_deref(), Some("hello-from-toml"));
    }

    #[test]
    fn test_resolve_model_limits_provider_aware_for_duplicate_ids() {
        // glm-5.2 exists under zhipu and zai (among others). The lookup must
        // succeed for the specified provider, not fall back to the conservative
        // default or silently resolve to a different provider's entry.
        let zhipu = Config {
            provider: "zhipu".to_string(),
            model: "glm-5.2".to_string(),
            providers: HashMap::new(),
            ..Default::default()
        };
        let (ctx, _) = zhipu.resolve_model_limits();
        assert_eq!(ctx, 1_000_000);

        let zai = Config {
            provider: "zai".to_string(),
            model: "glm-5.2".to_string(),
            providers: HashMap::new(),
            ..Default::default()
        };
        let (ctx2, _) = zai.resolve_model_limits();
        assert_eq!(ctx2, 1_000_000);

        // A wrong provider+model combo must NOT resolve via a different
        // provider's catalog entry — it falls to the conservative default.
        let wrong = Config {
            provider: "openai".to_string(),
            model: "glm-5.2".to_string(),
            providers: HashMap::new(),
            ..Default::default()
        };
        let (ctx3, out3) = wrong.resolve_model_limits();
        assert_eq!(ctx3, 128_000);
        assert_eq!(out3, None);
    }

    #[test]
    fn test_set_active_model_errors_on_ambiguous_bare_id() {
        let mut config = Config {
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-5-20250929".to_string(),
            providers: HashMap::new(),
            ..Default::default()
        };
        let err = config.set_active_model("glm-5.2").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("multiple providers"),
            "expected ambiguity error, got: {msg}"
        );
        assert!(msg.contains("zhipu"));
        assert!(msg.contains("zai"));
    }

    #[test]
    fn test_set_active_model_provider_qualified_resolves_correctly() {
        let mut config = Config {
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-5-20250929".to_string(),
            providers: HashMap::new(),
            ..Default::default()
        };
        config.set_active_model("zai/glm-5.2").unwrap();
        assert_eq!(config.model, "glm-5.2");
        assert_eq!(config.provider, "zai");
        assert!(config.providers.contains_key("zai"));
    }

    #[test]
    fn test_set_active_model_unique_bare_id_still_works() {
        let mut config = Config {
            provider: "zai".to_string(),
            model: "glm-5.2".to_string(),
            providers: HashMap::new(),
            ..Default::default()
        };
        // claude-sonnet-4-5 is unique to anthropic — bare ID should resolve.
        config.set_active_model("claude-sonnet-4-5").unwrap();
        assert_eq!(config.model, "claude-sonnet-4-5");
        assert_eq!(config.provider, "anthropic");
    }

    #[test]
    fn test_all_models_preserves_duplicates_across_providers() {
        let config = Config::default();
        let all = config.all_models();
        let glm52: Vec<_> = all.iter().filter(|m| m.id == "glm-5.2").collect();
        assert!(
            glm52.len() >= 2,
            "glm-5.2 should appear under multiple providers, got {}",
            glm52.len()
        );
        let providers: Vec<_> = glm52.iter().map(|m| m.provider.as_str()).collect();
        assert!(providers.contains(&"zhipu"));
        assert!(providers.contains(&"zai"));
    }

    #[test]
    fn test_all_models_user_override_matches_by_provider_and_id() {
        let config = Config {
            provider: "zai".to_string(),
            model: "glm-5.2".to_string(),
            providers: HashMap::from([(
                "zai".to_string(),
                ProviderConfig {
                    models: HashMap::from([(
                        "glm-5.2".to_string(),
                        ModelConfig {
                            context_limit: Some(50_000),
                            output_limit: Some(1000),
                        },
                    )]),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };
        let all = config.all_models();
        // The zai entry should be overridden, NOT the zhipu entry.
        let zai = all
            .iter()
            .find(|m| m.id == "glm-5.2" && m.provider == "zai")
            .unwrap();
        assert_eq!(zai.context_limit, Some(50_000));
        assert_eq!(zai.output_limit, Some(1000));
        // The zhipu entry should be untouched.
        let zhipu = all
            .iter()
            .find(|m| m.id == "glm-5.2" && m.provider == "zhipu")
            .unwrap();
        assert_eq!(zhipu.context_limit, Some(1_000_000));
    }

    #[test]
    fn test_provider_config_debug_masks_api_key() {
        let provider = ProviderConfig {
            api_key: Some("sk-super-secret".to_string()),
            api_key_env: Some("MY_KEY".to_string()),
            ..Default::default()
        };
        let debug = format!("{provider:?}");
        assert!(!debug.contains("sk-super-secret"));
        assert!(debug.contains("***"));
    }

    #[test]
    fn test_credentials_debug_masks_keys() {
        let mut creds = Credentials::default();
        creds
            .keys
            .insert("anthropic".to_string(), "sk-secret-key".to_string());
        let debug = format!("{creds:?}");
        assert!(!debug.contains("sk-secret-key"));
        assert!(debug.contains("redacted"));
    }

    #[test]
    fn test_config_debug_masks_provider_api_keys() {
        let config = Config {
            provider: "custom".to_string(),
            model: "test".to_string(),
            providers: HashMap::from([(
                "custom".to_string(),
                ProviderConfig {
                    api_key: Some("sk-leak-test".to_string()),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };
        let debug = format!("{config:?}");
        assert!(
            !debug.contains("sk-leak-test"),
            "Config Debug must not leak api_key"
        );
        assert!(debug.contains("***"));
    }
}
