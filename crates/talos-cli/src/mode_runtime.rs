use std::path::PathBuf;
use std::sync::{Arc, Mutex as StdMutex};

use anyhow::{Result, anyhow};
use talos_agent::Agent;
use talos_agent::context::ContextLoader;
use talos_agent::prompt::ContextFile;
use talos_config::Config;
use talos_memory::{MemoryStore, format_memory_prompt};
use talos_session::SessionMetadata;

use crate::Cli;

const REQUEST_PREVIEW_COMMAND: &str = "/mock-request";

pub(crate) fn request_preview_payload(input: &str) -> Option<String> {
    let trimmed = input.trim_start();
    let rest = trimmed.strip_prefix(REQUEST_PREVIEW_COMMAND)?;
    if rest.is_empty() || rest.starts_with(char::is_whitespace) {
        Some(rest.trim().to_string())
    } else {
        None
    }
}

pub(crate) fn session_metadata_for_model(model: &str, provider: &str) -> SessionMetadata {
    SessionMetadata {
        provider: (!provider.is_empty()).then(|| provider.to_string()),
        model: (!model.is_empty()).then(|| model.to_string()),
        token_count: None,
        working_directory: std::env::current_dir()
            .ok()
            .map(|path| path.to_string_lossy().to_string()),
    }
}

fn latest_session_model_info(session: &talos_session::Session) -> Option<(String, String)> {
    session
        .read_entries()
        .ok()?
        .into_iter()
        .rev()
        .find_map(
            |entry| match (entry.metadata.model, entry.metadata.provider) {
                (Some(model), Some(provider)) => Some((model, provider)),
                (Some(model), None) => Some((model, String::new())),
                _ => None,
            },
        )
}

pub(crate) fn apply_session_model_to_config(config: &mut Config, session: &talos_session::Session) {
    let Some((model, provider)) = latest_session_model_info(session) else {
        return;
    };
    let model_ref = if provider.is_empty() || model.starts_with(&format!("{provider}/")) {
        model
    } else {
        format!("{provider}/{model}")
    };
    if let Err(e) = config.set_active_model(&model_ref) {
        tracing::warn!(
            session_id = %session.id,
            model = %model_ref,
            "failed to restore session model metadata: {e}"
        );
    }
}

pub(crate) fn maybe_set_memory_provider(agent: &mut Agent, config: &Config) {
    if !config.memory_prompt.enabled {
        return;
    }
    let mem_config = talos_memory::MemoryPromptConfig {
        enabled: config.memory_prompt.enabled,
        max_items: config.memory_prompt.max_items,
        max_chars: config.memory_prompt.max_chars,
    };
    let memory_db = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".talos")
        .join("memory.db");
    let memory_store = Arc::new(StdMutex::new(None::<MemoryStore>));
    let provider = std::sync::Arc::new(move |query: &str| -> Option<String> {
        if !memory_db.exists() {
            tracing::debug!(
                path = %memory_db.display(),
                "memory store not initialized, skipping memory injection"
            );
            return None;
        }

        let mut store_guard = match memory_store.lock() {
            Ok(guard) => guard,
            Err(e) => {
                tracing::warn!("memory store lock poisoned, skipping memory injection: {e}");
                return None;
            }
        };
        if store_guard.is_none() {
            match MemoryStore::open(&memory_db) {
                Ok(store) => *store_guard = Some(store),
                Err(e) => {
                    tracing::warn!("memory store unavailable, skipping memory injection: {e}");
                    return None;
                }
            }
        }
        format_memory_prompt(store_guard.as_ref()?, query, &mem_config)
    });
    agent.set_memory_provider(provider);
}

pub(crate) fn model_metadata_context_file(config: &Config) -> ContextFile {
    let (context_limit, output_limit) = config.resolve_model_limits();
    let output_limit = output_limit
        .map(|limit| limit.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    ContextFile {
        path: "TALOS_MODEL.md".into(),
        content: format!(
            "# Current Model\n\
             Provider: {}\n\
             Model: {}\n\
             Provider protocol: {:?}\n\
             Tool protocol: {:?}\n\
             Context limit: {} tokens\n\
             Output limit: {} tokens\n",
            config.provider,
            config.model,
            config.provider_protocol(),
            config.tool_protocol(),
            context_limit,
            output_limit
        ),
    }
}

pub(crate) fn context_files_for_agent(
    config: &Config,
    workspace_root: &std::path::Path,
    include_workspace_context: bool,
) -> Result<Vec<ContextFile>> {
    let mut files = vec![model_metadata_context_file(config)];
    if include_workspace_context {
        let context = ContextLoader::new(workspace_root.to_path_buf())
            .load()
            .map_err(|e| anyhow!("{e}"))?;
        if !context.is_empty() {
            files.push(ContextFile {
                path: "AGENTS.md".into(),
                content: context,
            });
        }
    }
    Ok(files)
}

pub(crate) fn apply_mcp_fixture_config(config: &mut Config, cli: &Cli) {
    #[cfg(debug_assertions)]
    if let Some(path) = cli.mcp_server_fixture.clone() {
        config.mcp.servers = vec![talos_config::McpServerConfig {
            name: "fixture".to_string(),
            transport: "stdio".to_string(),
            command: path.to_string_lossy().to_string(),
            args: Vec::new(),
            env: std::collections::HashMap::from([(
                "ECHO_PREFIX".to_string(),
                "fixture".to_string(),
            )]),
            cwd: std::env::current_dir().ok(),
        }];
    }

    #[cfg(not(debug_assertions))]
    let _ = (config, cli);
}
