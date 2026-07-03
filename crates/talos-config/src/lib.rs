//! Talos config — configuration schema, validation, and environment substitution.
//!
//! Loads configuration from `~/.talos/config.toml` with support for environment
//! variable substitution (`${ENV_VAR}` syntax) and JSON Schema validation.

pub mod agents;
pub mod model;
pub mod opencode;

mod builtin;
mod config;
mod credentials;
mod env;
mod error;
#[cfg(test)]
#[allow(warnings)]
mod tests;
mod types;

pub use credentials::Credentials;
pub use error::ConfigError;
pub use types::{
    Config, DashboardConfig, HookConfig, LogConfig, LogFileConfig, LogFormat, LogRotation,
    McpConfig, McpServerConfig, MemoryPromptConfig, ModelConfig, ProviderConfig, ProviderProtocol,
    ProviderTimeoutConfig, ReasoningEffort, ReasoningOptions, RpcConfig, SkillConfig,
};

pub use builtin::builtin_provider_config;
pub(crate) use env::{home_dir, substitute_env_vars};
