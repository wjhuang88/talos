use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

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
    /// Per-model reasoning/thinking configuration (ADR-034).
    #[serde(default)]
    pub reasoning: Option<ReasoningOptions>,
}

/// Reasoning effort levels for OpenAI o-series models.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    Low,
    Medium,
    High,
}

/// Per-model reasoning/thinking options.
///
/// Maps to provider-specific request fields (Anthropic `thinking` block,
/// OpenAI `reasoning_effort`, OpenAI-compatible `reasoning_content`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(default)]
pub struct ReasoningOptions {
    /// Reasoning effort for OpenAI-style providers.
    pub effort: Option<ReasoningEffort>,
    /// Token budget for Anthropic thinking blocks.
    pub budget_tokens: Option<u32>,
    /// Replay captured reasoning in request history (ADR-034 replay policy).
    pub replay: bool,
}

impl Default for ReasoningOptions {
    fn default() -> Self {
        Self {
            effort: None,
            budget_tokens: None,
            replay: true,
        }
    }
}

/// Named provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(default)]
pub struct ProviderTimeoutConfig {
    /// Maximum seconds from request dispatch to first stream event.
    /// Default: 30 seconds.
    pub first_packet_timeout_secs: u64,
    /// Maximum seconds between stream events after the first packet.
    /// Default: 90 seconds.
    pub stream_idle_timeout_secs: u64,
    /// Maximum number of retry attempts for retryable provider failures.
    /// Default: 3 attempts.
    pub max_attempts: u32,
    /// Base delay in milliseconds for exponential backoff.
    /// Default: 500ms.
    pub backoff_base_ms: u64,
    /// Maximum delay in milliseconds for exponential backoff before jitter.
    /// Default: 8000ms.
    pub backoff_max_ms: u64,
}

impl Default for ProviderTimeoutConfig {
    fn default() -> Self {
        Self {
            first_packet_timeout_secs: 30,
            stream_idle_timeout_secs: 90,
            max_attempts: 3,
            backoff_base_ms: 500,
            backoff_max_ms: 8_000,
        }
    }
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
    /// Stream timeout configuration for provider requests.
    #[serde(default)]
    pub timeout: ProviderTimeoutConfig,
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
            .field("timeout", &self.timeout)
            .finish()
    }
}

/// Configuration for runtime memory prompt injection.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct MemoryPromptConfig {
    /// Whether memory injection is enabled.
    pub enabled: bool,
    /// Maximum number of memory items to include.
    pub max_items: usize,
    /// Maximum character budget for the formatted section.
    pub max_chars: usize,
}

impl Default for MemoryPromptConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_items: 5,
            max_chars: 2000,
        }
    }
}

/// Configuration for skill discovery behavior.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct SkillConfig {
    /// When true, scan ~/.agents/skills/ as a lowest-priority discovery path.
    pub discover_shared: bool,
}

/// Configuration for the read-only loopback dashboard (ADR-031).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct DashboardConfig {
    /// When true, start the loopback dashboard server on TUI launch.
    pub enabled: bool,
    /// When true, skip the per-process bearer token and rely on loopback bind only.
    /// Defaults to true: the dashboard binds 127.0.0.1, so loopback is the
    /// only network surface. Set to false to require a per-process bearer token
    /// for additional defense-in-depth on shared or multi-user machines.
    pub loopback_only: bool,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            loopback_only: true,
        }
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

    /// Memory prompt injection configuration.
    #[serde(default)]
    pub memory_prompt: MemoryPromptConfig,

    /// Skill discovery configuration.
    #[serde(default)]
    pub skills: SkillConfig,

    /// Loopback dashboard configuration (ADR-031).
    #[serde(default)]
    pub dashboard: DashboardConfig,
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
            memory_prompt: MemoryPromptConfig::default(),
            skills: SkillConfig::default(),
            dashboard: DashboardConfig::default(),
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
