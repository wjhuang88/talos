use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Outcome of a `config unset providers.<name>...` mutation (MODEL-010).
///
/// Distinguishes the three supported unset semantics so the CLI can print
/// accurate user-facing messages. Never carries credential values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigUnsetOutcome {
    /// A custom (non-builtin) provider entry was fully removed.
    /// It will no longer appear under Connected or Available.
    CustomProviderRemoved { name: String },
    /// A builtin-backed provider's user configuration was removed.
    /// The provider still appears under Available via the builtin catalog;
    /// only the user-saved credential/overrides were cleared.
    BuiltinProviderDisconnected { name: String },
    /// A single `api_key` field was cleared to `None` (omitted from TOML).
    /// All other provider fields and model overrides are preserved.
    ApiKeyCleared { name: String },
}

/// Reasoning effort levels for OpenAI o-series models.
pub use talos_core::model::ReasoningEffort;

/// Wire protocol used to talk to a provider.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum ProviderProtocol {
    /// Internal absence marker for configuration fields. This is never a user-selectable protocol.
    #[default]
    #[serde(skip)]
    #[doc(hidden)]
    Unspecified,
    /// Anthropic Messages API.
    #[serde(rename = "anthropic-messages")]
    AnthropicMessages,
    /// OpenAI Chat Completions compatible API.
    #[serde(rename = "openai-chat")]
    OpenAIChat,
    /// OpenAI Responses API.
    #[serde(rename = "openai-responses")]
    OpenAIResponses,
    /// Google Gemini generateContent wire format.
    #[serde(rename = "google-gemini")]
    GoogleGemini,
}

impl ProviderProtocol {
    /// Returns whether this value represents an omitted configuration field rather than a wire family.
    #[must_use]
    pub fn is_unspecified(&self) -> bool {
        matches!(self, Self::Unspecified)
    }
}

impl std::fmt::Display for ProviderProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Unspecified => "unspecified",
            Self::AnthropicMessages => "anthropic-messages",
            Self::OpenAIChat => "openai-chat",
            Self::OpenAIResponses => "openai-responses",
            Self::GoogleGemini => "google-gemini",
        })
    }
}

/// Per-model runtime limits and capability overrides.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct ModelConfig {
    /// Azure deployment alias; omission uses the selected logical model ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub azure_deployment: Option<String>,
    /// Explicit Azure body shape, ahead of provider and catalog choices.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub azure_shape: Option<crate::AzureBodyShape>,
    /// Explicit model-specific API root; takes precedence over provider and catalog endpoints.
    ///
    /// This is a connection override, not protocol evidence. Set `protocol` explicitly when
    /// connecting a model to a non-catalog endpoint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    /// Maximum provider input context accepted by this model.
    #[serde(default)]
    pub context_limit: Option<u32>,
    /// Maximum output tokens to request from this model.
    #[serde(default)]
    pub output_limit: Option<u32>,
    /// Explicit per-model wire protocol override.
    ///
    /// `None` means continue through provider/catalog/default resolution. `Some(openai-chat)` is
    /// therefore distinguishable from omission and remains a reliable model-scoped rollback.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol: Option<ProviderProtocol>,
    /// Per-model reasoning/thinking configuration (ADR-034).
    #[serde(default)]
    pub reasoning: Option<ReasoningOptions>,
    /// Explicit reasoning capability override, independent of effort/budget options.
    /// False disables reasoning parameters and replay; omission permits catalog inference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_supported: Option<bool>,
    /// Whether this model accepts image input.
    ///
    /// When `Some(true)`, overrides the catalog default so that
    /// `image_input` capability is `Supported` for custom/discovered models
    /// that are absent from the builtin catalog. When `Some(false)`,
    /// explicitly marks the model as non-vision. When `None` (default),
    /// the capability is resolved from catalog metadata or falls back to
    /// `Unknown` for models not in the catalog.
    #[serde(default)]
    pub image_input: Option<bool>,
}

/// Per-model reasoning/thinking options.
///
/// Maps to provider-specific request fields (Anthropic `thinking` block,
/// OpenAI `reasoning_effort`, OpenAI-compatible `reasoning_content`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(default)]
pub struct ReasoningOptions {
    /// Reasoning effort for OpenAI and confirmed Gemini level-based models.
    pub effort: Option<ReasoningEffort>,
    /// Token budget for Anthropic thinking and confirmed Gemini budget-based models.
    pub budget_tokens: Option<u32>,
    /// Replay captured reasoning in request history (ADR-034 replay policy).
    pub replay: bool,
}
/// A named invocation preset for a specific model (ADR-048).
///
/// A variant projects existing `ReasoningOptions` into a user-selectable
/// named choice. It contains no secrets or arbitrary request JSON.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[allow(dead_code)]
pub struct VariantDef {
    /// Stable identifier, e.g. "default", "high-reasoning".
    pub id: String,
    /// Display label, e.g. "High Reasoning".
    pub label: String,
    /// Optional reasoning effort override.
    #[serde(default)]
    pub reasoning_effort: Option<ReasoningEffort>,
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
    /// Maximum seconds from request dispatch (`send().await`) to the arrival of
    /// response headers. This protects against providers that accept the TCP
    /// connection but never return headers, which would cause `send().await` to
    /// hang indefinitely. This timeout covers only the dispatch → headers phase;
    /// stream parsing after headers is protected by `first_packet_timeout_secs`
    /// and `stream_idle_timeout_secs`.
    /// Default: 300 seconds.
    pub dispatch_timeout_secs: u64,
    /// Maximum seconds from request dispatch to first stream event.
    /// Default: 30 seconds.
    pub first_packet_timeout_secs: u64,
    /// Maximum seconds between stream events after the first packet.
    /// Default: 90 seconds.
    pub stream_idle_timeout_secs: u64,
    /// Maximum number of retry attempts for retryable provider failures.
    /// Default: 5 retries after the initial request.
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
            dispatch_timeout_secs: 300,
            first_packet_timeout_secs: 30,
            stream_idle_timeout_secs: 90,
            max_attempts: 5,
            backoff_base_ms: 500,
            backoff_max_ms: 8_000,
        }
    }
}

/// Explicit authentication/destination profile for the native Gemini wire.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(tag = "transport", rename_all = "kebab-case", deny_unknown_fields)]
pub enum GeminiTransportConfig {
    /// Official AI Studio destination with the provider's configured API key.
    AiStudio,
    /// Standard Vertex project/location endpoint with request-owned ADC.
    Vertex {
        /// Google Cloud project identifier or project number.
        project: String,
        /// Regional location or `global`.
        location: String,
        /// Allow fixed metadata credentials only when local ADC is absent.
        #[serde(default)]
        allow_metadata: bool,
    },
}

/// Named provider configuration.
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct ProviderConfig {
    /// Explicit Azure resource and authentication binding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub azure: Option<crate::AzureTransportConfig>,
    /// Explicit wire protocol override. Omitted means protocol selection is resolved later.
    #[serde(default, skip_serializing_if = "ProviderProtocol::is_unspecified")]
    pub protocol: ProviderProtocol,
    #[serde(default)]
    pub tool_protocol: talos_core::tool::ToolProtocol,
    /// Explicit Gemini profile. Required when selecting `google-gemini`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gemini: Option<GeminiTransportConfig>,
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
            .field("gemini", &self.gemini)
            .field("azure", &self.azure)
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct SkillConfig {
    /// When true, scan ~/.agents/skills/ as a lowest-priority discovery path.
    pub discover_shared: bool,
}

impl Default for SkillConfig {
    fn default() -> Self {
        Self {
            discover_shared: true,
        }
    }
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

/// Configuration for bounded model-assisted `auto` mode.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct AutoConfig {
    /// Whether bounded assistance is attempted when a later resolver is available.
    #[serde(default = "default_auto_enabled")]
    pub enabled: bool,
    /// Optional validated presentation locale for auto-review explanations.
    ///
    /// This is presentation-only and never affects permission authority or request identity.
    #[serde(default)]
    pub locale: Option<String>,
}

fn default_auto_enabled() -> bool {
    true
}

impl Default for AutoConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            locale: None,
        }
    }
}

/// Persisted search backend selection. Optional provider availability never changes this value.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SearchBackendConfig {
    /// Zero-key automatic search; currently resolves only to DuckDuckGo compatibility.
    #[default]
    Auto,
    /// Explicit zero-key DuckDuckGo selection.
    DuckDuckGo,
    /// Explicit premium Tavily selection; credential is resolved separately.
    Tavily,
    /// Explicit caller-configured SearXNG selection.
    SearXng,
}

/// Persisted web-search selection and explicit fallback policy.
#[derive(Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(default)]
pub struct SearchConfig {
    /// Explicit backend intent. Defaults to zero-key Auto.
    pub backend: SearchBackendConfig,
    /// For Tavily/SearXNG only, attempt zero-key Auto after execution failure/no valid result.
    pub fallback_to_auto: bool,
    /// Explicit SearXNG base URL. Ignored nowhere: projection rejects it for other backends.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            backend: SearchBackendConfig::Auto,
            fallback_to_auto: false,
            url: None,
        }
    }
}

impl std::fmt::Debug for SearchConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SearchConfig")
            .field("backend", &self.backend)
            .field("fallback_to_auto", &self.fallback_to_auto)
            .field("url", &self.url.as_ref().map(|_| "<redacted>"))
            .finish()
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

    /// Selected variant ID (ADR-048). Defaults to "default".
    #[serde(default)]
    pub variant: Option<String>,

    /// Named provider definitions.
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,

    /// Explicit web-search backend selection and fallback policy.
    #[serde(default)]
    pub search: SearchConfig,

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

    /// Bounded model-assisted permission mode configuration (ADR-064).
    #[serde(default)]
    pub auto: AutoConfig,
}

fn default_provider_name() -> String {
    "anthropic".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            provider: default_provider_name(),
            model: String::new(),
            variant: None,
            providers: HashMap::new(),
            search: SearchConfig::default(),
            log: LogConfig::default(),
            hooks: HookConfig::default(),
            mcp: McpConfig::default(),
            rpc: RpcConfig::default(),
            memory_prompt: MemoryPromptConfig::default(),
            skills: SkillConfig::default(),
            dashboard: DashboardConfig::default(),
            auto: AutoConfig::default(),
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

/// Log rotation strategy for file-based log output.
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
    #[serde(default)]
    pub declarations: Vec<HookDeclaration>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HookDeclaration {
    pub event: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
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
    /// Transport kind (`stdio`, `sse`, `streamable_http`, or `http` alias).
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
    /// Remote MCP endpoint URL for SSE or Streamable HTTP transports.
    #[serde(default)]
    pub url: Option<String>,
    /// Explicit JSON-RPC POST URL for legacy SSE transports.
    ///
    /// When unset, Talos reads the initial SSE `endpoint` event.
    #[serde(default)]
    pub sse_post_url: Option<String>,
    /// Non-secret custom HTTP headers for remote MCP transports.
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Environment variable containing a bearer token.
    #[serde(default)]
    pub auth_token_env: Option<String>,
    /// Environment variable containing the full Authorization header value.
    #[serde(default)]
    pub authorization_env: Option<String>,
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

#[cfg(test)]
mod gemini_profile_tests {
    use super::*;
    #[test]
    fn adc_key_exemption_requires_effective_gemini_vertex_profile() {
        let mut config = Config {
            provider: "fixture-adc".into(),
            model: "unknown-model".into(),
            providers: HashMap::from([(
                "fixture-adc".into(),
                ProviderConfig {
                    protocol: ProviderProtocol::GoogleGemini,
                    gemini: Some(GeminiTransportConfig::Vertex {
                        project: "project".into(),
                        location: "global".into(),
                        allow_metadata: false,
                    }),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
        assert_eq!(config.api_key().expect("ADC"), "");
        config
            .providers
            .get_mut("fixture-adc")
            .expect("provider")
            .protocol = ProviderProtocol::OpenAIChat;
        assert!(config.validate().is_err());
        assert!(config.api_key().is_err());
        let provider = config.providers.get_mut("fixture-adc").expect("provider");
        provider.protocol = ProviderProtocol::GoogleGemini;
        provider.gemini = Some(GeminiTransportConfig::AiStudio);
        assert!(config.validate().is_err());
        assert!(config.api_key().is_err());
    }

    #[test]
    fn profiles_roundtrip_without_changing_old_provider_defaults() {
        let old: ProviderConfig = toml::from_str("protocol = 'openai-chat'").expect("old config");
        assert!(old.gemini.is_none());
        for profile in [
            GeminiTransportConfig::AiStudio,
            GeminiTransportConfig::Vertex {
                project: "project".into(),
                location: "global".into(),
                allow_metadata: false,
            },
        ] {
            let config = ProviderConfig {
                protocol: ProviderProtocol::GoogleGemini,
                gemini: Some(profile),
                ..Default::default()
            };
            assert_eq!(
                toml::from_str::<ProviderConfig>(&toml::to_string(&config).expect("serialize"))
                    .expect("restore"),
                config
            );
        }
        assert!(toml::from_str::<ProviderConfig>("protocol='google-gemini'\n[gemini]\ntransport='vertex'\nproject='p'\nlocation='global'\nunknown=true").is_err());
    }
}
