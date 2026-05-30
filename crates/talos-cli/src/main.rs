//! Talos CLI — primary command-line interface.
//!
//! Supports print mode (`-p`) for streaming LLM responses to stdout,
//! with optional stdin pipe input and CLI argument overrides.

use std::io::{self, IsTerminal, Read, Write};
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use talos_agent::Agent;
use talos_config::{Config, Provider};
use talos_core::message::AgentEvent;
use talos_provider::AnthropicProvider;
use tokio::sync::broadcast;

#[derive(Parser)]
#[command(name = "talos", version, about = "Next-generation agent runtime")]
struct Cli {
    /// The prompt to send to the agent.
    prompt: Option<String>,

    /// Print mode: stream the response to stdout and exit.
    #[arg(short, long)]
    print: bool,

    /// Override the model name (e.g., `claude-sonnet-4-20250514`).
    #[arg(short, long)]
    model: Option<String>,

    /// Override the provider (`anthropic` or `openai`).
    #[arg(long)]
    provider: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if !cli.print {
        eprintln!("Error: print mode (-p) is required. Interactive mode is not yet implemented.");
        std::process::exit(1);
    }

    run_print_mode(cli).await
}

async fn run_print_mode(cli: Cli) -> Result<()> {
    // Load configuration
    let mut config = Config::load().context("failed to load configuration")?;

    // Apply CLI overrides
    if let Some(ref model) = cli.model {
        config.model = model.clone();
    }
    if let Some(ref provider_str) = cli.provider {
        config.provider = parse_provider(provider_str)?;
    }

    // Validate model is set
    if config.model.is_empty() {
        eprintln!("Error: no model configured. Set 'model' in ~/.talos/config.toml or pass --model.");
        std::process::exit(1);
    }

    // Get API key
    let api_key = config.api_key().map_err(|e| anyhow!("{}", e))?;

    // Resolve prompt: positional arg, or stdin pipe, or error
    let prompt = resolve_prompt(cli.prompt)?;

    // Create provider and agent
    let provider = Arc::new(AnthropicProvider::new(api_key, &config.model));
    let agent = Agent::new(provider);

    // Set up broadcast channel for streaming events
    let (event_tx, mut event_rx) = broadcast::channel::<AgentEvent>(32);

    // Spawn the agent run in a separate task
    let _run_handle = tokio::spawn(async move { agent.run_streaming(prompt, event_tx).await });

    // Main task: receive events and print to stdout
    let mut stdout = io::stdout().lock();
    loop {
        match event_rx.recv().await {
            Ok(AgentEvent::TextDelta { delta }) => {
                print!("{delta}");
                stdout.flush().context("failed to flush stdout")?;
            }
            Ok(AgentEvent::TurnEnd { .. }) => {
                println!();
                return Ok(());
            }
            Ok(AgentEvent::Error { message }) => {
                eprintln!("Error: {message}");
                std::process::exit(1);
            }
            Ok(AgentEvent::TurnStart | AgentEvent::ToolCall { .. } | AgentEvent::ToolResult { .. }) => {
                // Ignore non-text events in print mode
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                eprintln!("Warning: dropped {n} event(s) due to slow consumer");
            }
            Err(broadcast::error::RecvError::Closed) => {
                eprintln!("Error: event channel closed before TurnEnd");
                std::process::exit(1);
            }
        }
    }
}

/// Resolves the prompt from CLI argument or stdin pipe.
fn resolve_prompt(cli_prompt: Option<String>) -> Result<String> {
    if let Some(prompt) = cli_prompt {
        return Ok(prompt);
    }

    // Check if stdin is a pipe (not a terminal)
    if !io::stdin().is_terminal() {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("failed to read from stdin")?;
        let trimmed = buffer.trim().to_string();
        if trimmed.is_empty() {
            return Err(anyhow!("stdin is empty"));
        }
        return Ok(trimmed);
    }

    Err(anyhow!(
        "no prompt provided. Usage: talos \"your prompt\" -p, or echo \"prompt\" | talos -p"
    ))
}

/// Parses a provider string into a [`Provider`] enum.
fn parse_provider(s: &str) -> Result<Provider> {
    match s.to_lowercase().as_str() {
        "anthropic" => Ok(Provider::Anthropic),
        "openai" => Ok(Provider::OpenAI),
        other => Err(anyhow!("unknown provider '{}': supported values are 'anthropic' and 'openai'", other)),
    }
}
