//! Talos CLI — primary command-line interface.
//!
//! Supports print mode (`-p`) for streaming LLM responses to stdout,
//! interactive mode for conversational agent sessions, and optional
//! stdin pipe input and CLI argument overrides.
//!
//! # Session Commands
//!
//! - `--search <query>`: Full-text search across indexed session messages
//! - `--list`: List recent sessions from the SQLite index
//! - `--resume`: Interactive session selection from recent sessions
//! - `--continue`: Resume the most recent session automatically
//! - `--session <id>`: Resume a specific session by UUID

mod approval;
mod colors;
mod event_loop;
mod exploration_cli;
mod governance;
mod governance_mutation;
mod init_wizard;
mod logging;
mod mcp_runtime;
mod memory_cli;
mod mode_inline;
mod mode_print;
mod mode_runners;
mod mode_runtime;
mod model_lifecycle;
mod models_browser;
mod permissions;
mod provider_setup;
mod registry;
mod runtime_adapter;
mod session_setup;
mod session_transition;
mod skill_runtime;
mod storage;
#[cfg(test)]
mod test_support;
mod todo_view;
mod tui_bridge;
mod validation;

use std::io::{self, IsTerminal};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use talos_config::{Config, ProviderProtocol};
use talos_plugin::{HookRegistry, LoggingHandler};

use crate::logging::init_logger;
use crate::mode_runners::{
    run_inline_mode, run_interactive_mode, run_mcp_server, run_print_mode, run_rpc_mode,
    run_tui_mode,
};
use crate::session_setup::{run_learned_mode, run_list_mode, run_search_mode};

/// Runtime mode selection.
#[derive(Debug, Clone, ValueEnum)]
pub enum Mode {
    /// Print mode.
    Print,
    /// TUI mode.
    Tui,
    /// Interactive mode.
    Interactive,
    /// MCP server placeholder.
    McpServer,
    /// JSON-RPC placeholder.
    Rpc,
}

/// Top-level subcommands for talos.
#[derive(Subcommand, Clone)]
pub(crate) enum TalosCommand {
    /// Run first-run setup wizard for model configuration.
    Init {
        /// Print setup instructions without launching interactive wizard.
        #[arg(long)]
        non_interactive: bool,
    },
    /// Local storage visibility and maintenance.
    Storage {
        #[command(subcommand)]
        command: storage::StorageCommand,
    },
    /// Memory operations.
    Memory {
        #[command(subcommand)]
        command: memory_cli::MemoryCommand,
    },
    /// Exploration operations.
    Explore {
        #[command(subcommand)]
        command: exploration_cli::ExploreCommand,
    },
    /// Validation planning.
    Validate {
        #[command(subcommand)]
        command: validation::ValidateCommand,
    },
    /// Governance preview/write gates.
    Governance {
        #[command(subcommand)]
        command: governance_mutation::GovernanceCommand,
    },
    /// Permission planning and inspection.
    Permissions {
        #[command(subcommand)]
        command: permissions::PermissionsCommand,
    },
    /// Configuration operations.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
}

/// Subcommands for `talos config`.
#[derive(Subcommand, Clone)]
pub(crate) enum ConfigCommand {
    /// Print all configuration settings (secrets masked).
    List,
    /// Get a single configuration value by dotted key.
    Get {
        /// Dotted key path (e.g., "model", "providers.anthropic.api_key_env").
        key: String,
    },
    /// Set a configuration value and persist to disk.
    Set {
        /// Dotted key path (e.g., "model", "providers.anthropic.base_url").
        key: String,
        /// New value.
        value: String,
    },
}

#[derive(Parser, Clone)]
#[command(name = "talos", version, about = "Next-generation agent runtime")]
pub(crate) struct Cli {
    #[arg(help = "The prompt to send to the agent.")]
    prompt: Option<String>,

    #[command(subcommand)]
    command: Option<TalosCommand>,

    #[arg(
        short,
        long,
        help = "Print mode: stream the response to stdout and exit."
    )]
    print: bool,

    #[arg(
        short,
        long,
        help = "Override the model name (e.g., `claude-sonnet-4-5`)."
    )]
    model: Option<String>,

    #[arg(long, help = "Override the provider (`anthropic` or `openai`).")]
    provider: Option<String>,

    #[arg(long, help = "Launch terminal UI instead of readline loop.")]
    tui: bool,

    #[arg(
        long,
        conflicts_with_all = ["tui", "repl", "print"],
        help = "Inline terminal mode: Codex-like UX, no alt-screen, preserves scrollback."
    )]
    inline: bool,

    #[arg(
        long,
        conflicts_with = "tui",
        help = "Force the readline interactive REPL (default is TUI on a TTY)."
    )]
    repl: bool,

    #[arg(long, help = "Skip loading workspace context.")]
    no_context: bool,

    #[arg(short = 'c', long, help = "Resume the most recent session.")]
    r#continue: bool,

    #[arg(short = 'r', long, help = "List sessions and prompt for selection.")]
    resume: bool,

    #[arg(long, help = "Resume a specific session by ID.")]
    session: Option<String>,

    #[arg(long, help = "Fork from a specific session ID, creating a new branch.")]
    fork: Option<String>,

    #[arg(long, help = "Search session messages with full-text search.")]
    search: Option<String>,

    #[arg(long, help = "List recent sessions from the index.")]
    list: bool,

    #[arg(
        long,
        default_value = "20",
        help = "Maximum results for --search or --list."
    )]
    limit: usize,

    #[arg(long, help = "Override the default system prompt entirely.")]
    system_prompt: Option<String>,

    #[arg(long, help = "Append additional instructions to the system prompt.")]
    append_system_prompt: Option<String>,

    #[arg(
        short = 'w',
        long,
        value_name = "PATH",
        help = "Set the workspace root directory (default: current working directory)."
    )]
    workspace: Option<String>,

    #[arg(
        long,
        help = "Use mock LLM provider for testing (no API key required)."
    )]
    mock: bool,

    #[arg(
        long,
        help = "Skip the first-run model setup wizard (for CI / non-interactive use)."
    )]
    no_init: bool,

    #[arg(
        long,
        conflicts_with_all = ["config_get", "config_set"],
        help = "List all configuration values."
    )]
    config_list: bool,

    #[arg(
        long,
        value_name = "KEY",
        conflicts_with_all = ["config_list", "config_set"],
        help = "Get a single configuration value by dotted key (e.g. 'model', 'providers.anthropic.api_key_env')."
    )]
    config_get: Option<String>,

    #[arg(
        long,
        value_name = "KEY=VALUE",
        conflicts_with_all = ["config_list", "config_get"],
        help = "Set a configuration value (e.g. 'model=claude-sonnet-4-5')."
    )]
    config_set: Option<String>,

    #[arg(
        long = "available-models",
        help = "List models from the builtin catalog, grouped by provider. Output is bounded unless --available-models-all is set."
    )]
    available_models: bool,

    #[arg(
        long = "available-models-filter",
        value_name = "QUERY",
        requires = "available_models",
        help = "Filter --available-models output by provider, model id, or provider/model substring."
    )]
    available_models_filter: Option<String>,

    #[arg(
        long = "available-models-limit",
        value_name = "N",
        default_value_t = 120,
        requires = "available_models",
        help = "Maximum model rows printed by --available-models unless --available-models-all is set."
    )]
    available_models_limit: usize,

    #[arg(
        long = "available-models-all",
        requires = "available_models",
        help = "Print every matching model row from --available-models."
    )]
    available_models_all: bool,

    #[arg(
        long = "available-models-browser",
        help = "Open an independent terminal browser for the built-in model catalog."
    )]
    available_models_browser: bool,

    #[arg(
        long = "use-model",
        value_name = "MODEL_ID",
        help = "Set the active model (e.g. 'anthropic/claude-sonnet-4-5'). Persists to config.toml."
    )]
    use_model: Option<String>,

    #[arg(
        long,
        conflicts_with_all = ["tui", "repl", "inline", "print"],
        help = "Re-run the first-run setup wizard: enter TUI with the model picker auto-opened."
    )]
    init: bool,

    #[arg(long, help = "Display learned patterns from the evolution engine.")]
    learned: bool,

    #[arg(
        long,
        help = "Print read-only governance status: manifest, active iteration, open iterations, validation result."
    )]
    governance_status: bool,

    #[arg(long, value_enum, help = "Explicit runtime mode.")]
    mode: Option<Mode>,

    #[arg(
        long,
        value_name = "PATH",
        help = "Import model metadata from a models.dev JSON file (deprecated: use BUILD_MODELS=1 at build time instead; this flag is a no-op)."
    )]
    #[allow(dead_code)]
    import_models: Option<PathBuf>,

    // I009-S3 begin
    #[cfg(debug_assertions)]
    #[arg(
        long,
        value_name = "PATH",
        help = "Use local fixture MCP server binary (tests/dev only)."
    )]
    mcp_server_fixture: Option<PathBuf>,
    // I009-S3 end
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Subcommand dispatch (takes priority over flat flags)
    if let Some(TalosCommand::Init { non_interactive }) = &cli.command {
        return init_wizard::run_init_wizard(*non_interactive).await;
    }

    if let Some(TalosCommand::Storage { command }) = &cli.command {
        return crate::storage::run_storage_command(command.clone());
    }

    if let Some(TalosCommand::Memory { command }) = &cli.command {
        return crate::memory_cli::run_memory_command(command.clone());
    }

    if let Some(TalosCommand::Explore { command }) = &cli.command {
        return crate::exploration_cli::run_explore_command(command.clone());
    }

    if let Some(TalosCommand::Validate { command }) = &cli.command {
        return crate::validation::run_validate_command(command.clone());
    }

    if let Some(TalosCommand::Governance { command }) = &cli.command {
        return crate::governance_mutation::run_governance_command(command.clone());
    }

    if let Some(TalosCommand::Permissions { command }) = &cli.command {
        return crate::permissions::run_permissions_command(command.clone());
    }

    if let Some(TalosCommand::Config { command }) = &cli.command {
        return match command {
            ConfigCommand::List => run_config_list(),
            ConfigCommand::Get { key } => run_config_get(key),
            ConfigCommand::Set { key, value } => run_config_set(&format!("{key}={value}")),
        };
    }

    if let Some(_path) = &cli.import_models {
        eprintln!(
            "Note: --import-models is deprecated. Use BUILD_MODELS=1 cargo build to regenerate \
             the built-in model catalog at build time. This flag no longer writes to catalog.db."
        );
        return Ok(());
    }

    if cli.config_list {
        return run_config_list();
    }
    if let Some(key) = &cli.config_get {
        return run_config_get(key);
    }
    if let Some(kv) = &cli.config_set {
        return run_config_set(kv);
    }

    if cli.available_models {
        return run_models(
            cli.available_models_filter.as_deref(),
            cli.available_models_limit,
            cli.available_models_all,
        );
    }
    if cli.available_models_browser {
        return models_browser::run_available_models_browser(
            cli.available_models_filter.as_deref(),
        );
    }
    if let Some(model_id) = &cli.use_model {
        return run_use_model(model_id);
    }

    if cli.governance_status {
        return crate::governance::run_governance_status();
    }

    if cli.init {
        let mut config = Config::load().context("failed to load configuration")?;
        config.model.clear();
        config.save().context("failed to save configuration")?;
        // Falls through to the TUI path which will auto-open the model picker
    }

    if matches!(cli.mode, Some(Mode::McpServer)) {
        return run_mcp_server().await;
    }

    let terminal_ui = cli.tui
        || (!cli.print
            && cli.search.is_none()
            && !cli.list
            && !cli.learned
            && !matches!(cli.mode, Some(Mode::Rpc))
            && io::stdin().is_terminal());
    let config_for_logging = Config::load().ok();
    init_logger(
        config_for_logging.as_ref().map(|config| &config.log),
        terminal_ui,
    );

    if cli.search.is_some() {
        return run_search_mode(cli);
    }

    if cli.list {
        return run_list_mode(cli);
    }

    if cli.learned {
        return run_learned_mode(cli);
    }

    if matches!(cli.mode, Some(Mode::Rpc)) {
        return run_rpc_mode(cli).await;
    }

    if cli.print {
        return run_print_mode(cli).await;
    }

    if cli.tui {
        return run_tui_mode(cli).await;
    }

    if cli.inline {
        return run_inline_mode(cli).await;
    }

    if cli.repl {
        return run_interactive_mode(cli).await;
    }

    if !io::stdin().is_terminal() {
        return run_print_mode(cli).await;
    }

    run_tui_mode(cli).await
}

pub(crate) fn build_hook_registry(include_evolution: bool) -> Arc<HookRegistry> {
    let mut registry = HookRegistry::new();
    registry.register(Arc::new(LoggingHandler::new()));
    if include_evolution {
        match talos_evolution::EvolutionHookHandler::open_default(
            talos_evolution::EvolutionConfig::default(),
            None,
        ) {
            Ok(Some(handler)) => registry.register(Arc::new(handler)),
            Ok(None) => {}
            Err(e) => eprintln!("Warning: evolution disabled: {e}"),
        }
    }
    Arc::new(registry)
}

fn run_config_list() -> Result<()> {
    let config = Config::load().context("failed to load configuration")?;
    let toml_str = toml::to_string_pretty(&config)
        .map_err(|e| anyhow::anyhow!("failed to serialize config: {e}"))?;
    let masked = mask_secrets(&toml_str, &config);
    println!("{masked}");
    Ok(())
}

fn run_config_get(key: &str) -> Result<()> {
    let config = Config::load().context("failed to load configuration")?;
    let value = config_get_dotted(&config, key)?;
    if is_secret_key(key) {
        println!("***");
    } else {
        println!("{value}");
    }
    Ok(())
}

fn run_config_set(kv: &str) -> Result<()> {
    let (key, value) = kv
        .split_once('=')
        .ok_or_else(|| anyhow::anyhow!("invalid format: expected KEY=VALUE (got '{kv}')"))?;
    let mut config = Config::load().context("failed to load configuration")?;
    config_set_dotted(&mut config, key.trim(), value.trim())?;
    config
        .validate()
        .context("configuration validation failed — value not saved")?;
    config.save().context("failed to save configuration")?;
    println!(
        "Set {key} = {}",
        if is_secret_key(key) {
            "***".to_string()
        } else {
            value.trim().to_string()
        }
    );
    Ok(())
}

fn run_models(filter: Option<&str>, limit: usize, show_all: bool) -> Result<()> {
    let config = Config::load().context("failed to load configuration")?;
    let catalog = talos_config::model::builtin_models();
    let filter = normalize_model_filter(filter);
    let effective_limit = if show_all { usize::MAX } else { limit };

    let mut by_provider: std::collections::BTreeMap<
        String,
        Vec<&talos_config::model::ModelMetadata>,
    > = std::collections::BTreeMap::new();
    for m in &catalog {
        if !model_matches_filter(m, filter.as_deref()) {
            continue;
        }
        by_provider.entry(m.provider.clone()).or_default().push(m);
    }

    let matched_count = by_provider.values().map(Vec::len).sum::<usize>();
    let shown_count = matched_count.min(effective_limit);
    println!(
        "Built-in model catalog: {matched_count} matching models across {} providers.",
        by_provider.len()
    );
    if !show_all && matched_count > shown_count {
        println!(
            "Showing first {shown_count}. Use --available-models-filter <query> to narrow results or --available-models-all to print all."
        );
    }
    if matched_count == 0 {
        return Ok(());
    }

    let mut printed = 0usize;
    for (provider, models) in &by_provider {
        if printed >= effective_limit {
            break;
        }

        let authed = config.provider_authenticated(provider);
        let status = if authed { "Ready" } else { "Setup required" };
        println!("\n{provider}  —  {status}");

        let mut sorted = models.clone();
        sorted.sort_by(|a, b| a.id.cmp(&b.id));
        for m in sorted {
            if printed >= effective_limit {
                break;
            }
            let ctx = m
                .context_limit
                .map(|c| format!("{}K", c / 1000))
                .unwrap_or_else(|| "?".to_string());
            let pricing = m
                .pricing
                .as_ref()
                .map(|p| {
                    let inp = p.input_per_1m.unwrap_or(0.0);
                    let out = p.output_per_1m.unwrap_or(0.0);
                    format!("  ${inp:.2}/${out:.2}/1M tok")
                })
                .unwrap_or_default();
            println!("  {}  (ctx: {ctx}){pricing}", available_model_name(m));
            printed += 1;
        }
    }

    if !show_all && matched_count > printed {
        println!(
            "\n... {} more matching models omitted. Use --available-models-all to print every row.",
            matched_count - printed
        );
    }

    Ok(())
}

fn available_model_name(model: &talos_config::model::ModelMetadata) -> String {
    format!("{}/{}", model.provider, model.id)
}

fn normalize_model_filter(filter: Option<&str>) -> Option<String> {
    filter
        .map(str::trim)
        .filter(|query| !query.is_empty())
        .map(|query| query.to_lowercase())
}

fn model_matches_filter(model: &talos_config::model::ModelMetadata, filter: Option<&str>) -> bool {
    let Some(query) = filter else {
        return true;
    };
    model.provider.to_lowercase().contains(query)
        || model.id.to_lowercase().contains(query)
        || available_model_name(model).to_lowercase().contains(query)
}

fn run_use_model(model_id: &str) -> Result<()> {
    let mut config = Config::load().context("failed to load configuration")?;
    config
        .set_active_model(model_id)
        .with_context(|| format!("unknown model '{model_id}'"))?;

    if !config.provider_authenticated(&config.provider) {
        eprintln!(
            "Note: provider '{}' needs credentials. Set with:",
            config.provider
        );
        eprintln!(
            "  talos --config-set providers.{}.api_key=YOUR_KEY",
            config.provider
        );
    }

    config.save().context("failed to save configuration")?;
    println!("Active model set to {model_id}.");
    Ok(())
}

pub(crate) fn mask_secrets(toml_str: &str, _config: &Config) -> String {
    toml_str
        .lines()
        .map(|line| {
            if let Some(eq) = line.find('=') {
                let key = line[..eq].trim();
                if key == "api_key" {
                    return format!("{key} = ***");
                }
            }
            line.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn is_secret_key(key: &str) -> bool {
    let lower = key.to_lowercase();
    lower.ends_with("api_key") || lower.ends_with("secret") || lower.ends_with("token")
}

pub(crate) fn config_get_dotted(config: &Config, key: &str) -> Result<String> {
    let parts: Vec<&str> = key.split('.').collect();
    match parts.as_slice() {
        ["provider"] => Ok(config.provider.clone()),
        ["model"] => Ok(config.model.clone()),
        ["providers", name, "protocol"] => config
            .providers
            .get(*name)
            .map(|p| format!("{:?}", p.protocol))
            .ok_or_else(|| anyhow::anyhow!("provider '{name}' not found")),
        ["providers", name, "base_url"] => Ok(config
            .providers
            .get(*name)
            .and_then(|p| p.base_url.clone())
            .unwrap_or_default()),
        ["providers", name, "api_key_env"] => Ok(config
            .providers
            .get(*name)
            .and_then(|p| p.api_key_env.clone())
            .unwrap_or_default()),
        ["providers", name, "api_key"] => Ok(config
            .providers
            .get(*name)
            .and_then(|p| p.api_key.clone())
            .unwrap_or_default()),
        ["providers", name, "models", model, "context_limit"] => config
            .providers
            .get(*name)
            .and_then(|p| p.models.get(*model))
            .and_then(|m| m.context_limit)
            .map(|v| v.to_string())
            .ok_or_else(|| anyhow::anyhow!("not found")),
        ["providers", name, "models", model, "output_limit"] => config
            .providers
            .get(*name)
            .and_then(|p| p.models.get(*model))
            .and_then(|m| m.output_limit)
            .map(|v| v.to_string())
            .ok_or_else(|| anyhow::anyhow!("not found")),
        ["dashboard", "enabled"] => Ok(config.dashboard.enabled.to_string()),
        ["dashboard", "loopback_only"] => Ok(config.dashboard.loopback_only.to_string()),
        _ => anyhow::bail!("unsupported config key: '{key}'"),
    }
}

pub(crate) fn config_set_dotted(config: &mut Config, key: &str, value: &str) -> Result<()> {
    let parts: Vec<&str> = key.split('.').collect();
    match parts.as_slice() {
        ["model"] => {
            config.model = value.to_string();
            Ok(())
        }
        ["provider"] => {
            config.provider = value.to_string();
            Ok(())
        }
        ["providers", name, "protocol"] => {
            let protocol = match value {
                "anthropic-messages" => ProviderProtocol::AnthropicMessages,
                "openai-chat" => ProviderProtocol::OpenAIChat,
                _ => anyhow::bail!(
                    "unknown protocol '{value}': must be 'anthropic-messages' or 'openai-chat'"
                ),
            };
            config
                .providers
                .entry((*name).to_string())
                .or_insert_with(|| talos_config::ProviderConfig {
                    ..Default::default()
                })
                .protocol = protocol;
            Ok(())
        }
        ["providers", name, "api_key_env"] => {
            config
                .providers
                .entry((*name).to_string())
                .or_insert_with(|| talos_config::ProviderConfig {
                    ..Default::default()
                })
                .api_key_env = Some(value.to_string());
            Ok(())
        }
        ["providers", name, "base_url"] => {
            config
                .providers
                .entry((*name).to_string())
                .or_insert_with(|| talos_config::ProviderConfig {
                    ..Default::default()
                })
                .base_url = if value.is_empty() {
                None
            } else {
                Some(value.to_string())
            };
            Ok(())
        }
        ["providers", name, "api_key"] => {
            config.set_provider_credential(name, value);
            Ok(())
        }
        ["providers", name, "models", model, "context_limit"] => {
            let limit: u32 = value.parse().map_err(|_| {
                anyhow::anyhow!("invalid context_limit '{value}': must be a positive integer")
            })?;
            config
                .providers
                .entry((*name).to_string())
                .or_insert_with(|| talos_config::ProviderConfig {
                    ..Default::default()
                })
                .models
                .entry((*model).to_string())
                .or_default()
                .context_limit = Some(limit);
            Ok(())
        }
        ["providers", name, "models", model, "output_limit"] => {
            let limit: u32 = value.parse().map_err(|_| {
                anyhow::anyhow!("invalid output_limit '{value}': must be a positive integer")
            })?;
            config
                .providers
                .entry((*name).to_string())
                .or_insert_with(|| talos_config::ProviderConfig {
                    ..Default::default()
                })
                .models
                .entry((*model).to_string())
                .or_default()
                .output_limit = Some(limit);
            Ok(())
        }
        ["dashboard", "enabled"] => {
            config.dashboard.enabled = parse_config_bool(value, key)?;
            Ok(())
        }
        ["dashboard", "loopback_only"] => {
            config.dashboard.loopback_only = parse_config_bool(value, key)?;
            Ok(())
        }
        _ => anyhow::bail!("unsupported config key for set: '{key}'"),
    }
}

fn parse_config_bool(value: &str, key: &str) -> Result<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => anyhow::bail!("invalid boolean for '{key}': expected true or false"),
    }
}

#[cfg(test)]
#[allow(warnings)]
mod tests;
