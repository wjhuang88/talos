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
mod init_wizard;
mod logging;
mod mcp_runtime;
mod memory_cli;
mod mode_runners;
mod model_lifecycle;
mod provider_setup;
mod registry;
mod runtime_adapter;
mod session_setup;
mod session_transition;
mod skill_runtime;
mod storage;
mod tui_bridge;

use std::io::{self, IsTerminal};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use talos_config::Config;
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
        help = "Override the model name (e.g., `claude-sonnet-4-20250514`)."
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
        help = "Set a configuration value (e.g. 'model=claude-sonnet-4-20250514')."
    )]
    config_set: Option<String>,

    #[arg(
        long = "available-models",
        help = "List available models from the builtin model catalog, grouped by provider with authentication status."
    )]
    available_models: bool,

    #[arg(
        long = "use-model",
        value_name = "MODEL_ID",
        help = "Set the active model (e.g. 'claude-sonnet-4-20250514'). Persists to config.toml."
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
        help = "Import model metadata from a models.dev JSON file."
    )]
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

    if let Some(path) = &cli.import_models {
        return run_import_models(path);
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
        return run_models();
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

fn run_import_models(path: &PathBuf) -> Result<()> {
    let json = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("failed to read {}: {e}", path.display()))?;

    let models = talos_config::model::import_models_dev(&json)
        .map_err(|e| anyhow::anyhow!("failed to parse models.dev data: {e}"))?;

    let cache_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".talos")
        .join("cache")
        .join("models");
    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| anyhow::anyhow!("failed to create cache dir: {e}"))?;

    let cache_path = cache_dir.join("models.json");
    std::fs::write(&cache_path, &json)
        .map_err(|e| anyhow::anyhow!("failed to write cache: {e}"))?;

    println!("Imported {} models from models.dev", models.len());
    println!("Cached to {}", cache_path.display());

    let mut by_provider: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for m in &models {
        *by_provider.entry(m.provider.clone()).or_insert(0) += 1;
    }
    for (provider, count) in by_provider.iter() {
        println!("  {provider}: {count}");
    }

    Ok(())
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

fn run_models() -> Result<()> {
    let config = Config::load().context("failed to load configuration")?;
    let catalog = talos_config::model::builtin_models();

    let mut by_provider: std::collections::BTreeMap<
        String,
        Vec<&talos_config::model::ModelMetadata>,
    > = std::collections::BTreeMap::new();
    for m in &catalog {
        by_provider.entry(m.provider.clone()).or_default().push(m);
    }

    for (provider, models) in &by_provider {
        let authed = config.provider_authenticated(provider);
        let status = if authed { "Ready" } else { "Setup required" };
        println!("\n{provider}  —  {status}");

        let mut sorted = models.clone();
        sorted.sort_by(|a, b| a.id.cmp(&b.id));
        for m in sorted {
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
            println!("  {}  (ctx: {ctx}){pricing}", m.id);
        }
    }

    Ok(())
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
        _ => anyhow::bail!("unsupported config key: '{key}'"),
    }
}

fn config_set_dotted(config: &mut Config, key: &str, value: &str) -> Result<()> {
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
        _ => anyhow::bail!("unsupported config key for set: '{key}'"),
    }
}

#[cfg(test)]
#[allow(warnings)]
mod tests;
