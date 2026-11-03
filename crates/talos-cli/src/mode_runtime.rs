use std::path::PathBuf;
use std::sync::{Arc, Mutex as StdMutex};

use anyhow::{Result, anyhow};
use talos_agent::Agent;
use talos_agent::context::ContextLoader;
use talos_agent::prompt::ContextFile;
use talos_config::Config;
use talos_memory::{MemoryStore, format_memory_prompt};
use talos_session::{
    PendingSubmissionStore, SessionManager, SessionMetadata, SessionRuntimeActivation,
    SessionRuntimeActivationStatus, SessionRuntimeIdentity, TodoItem, TodoPriority, TodoQuery,
    TodoRepository, TodoStatus,
};
use uuid::Uuid;

use crate::Cli;

const REQUEST_PREVIEW_COMMAND: &str = "/mock-request";
const TODO_PROMPT_MAX_ITEMS: usize = 12;
const TODO_PROMPT_MAX_CHARS: usize = 2400;
const SESSION_MODEL_ACTIVATION_PREFIX: &str = "talos:model-activation:v1:";

pub(crate) type SessionModelIdentity = SessionRuntimeIdentity;
pub(crate) type SessionModelActivation = SessionRuntimeActivation;

pub(crate) fn session_model_activation_metadata(
    activation: &SessionModelActivation,
) -> Result<SessionMetadata, serde_json::Error> {
    let mut metadata =
        session_metadata_for_model(&activation.target.model, &activation.target.provider);
    metadata.raw_content = Some(format!(
        "{SESSION_MODEL_ACTIVATION_PREFIX}{}",
        serde_json::to_string(activation)?
    ));
    Ok(metadata)
}

pub(crate) fn session_model_activation_from_metadata(
    metadata: &SessionMetadata,
) -> Option<SessionModelActivation> {
    let payload = metadata
        .raw_content
        .as_deref()?
        .strip_prefix(SESSION_MODEL_ACTIVATION_PREFIX)?;
    let activation: SessionModelActivation = serde_json::from_str(payload).ok()?;
    if activation.version != 1 {
        return None;
    }
    let expected = SessionModelActivation::new(
        activation.generation,
        activation.previous.clone(),
        activation.target.clone(),
    );
    if activation.activation_id != expected.activation_id
        || metadata.provider.as_deref() != Some(activation.target.provider.as_str())
        || metadata.model.as_deref() != Some(activation.target.model.as_str())
    {
        return None;
    }
    Some(activation)
}

pub(crate) fn request_preview_payload(input: &str) -> Option<String> {
    let trimmed = input.trim_start();
    let rest = trimmed.strip_prefix(REQUEST_PREVIEW_COMMAND)?;
    if rest.is_empty() || rest.starts_with(char::is_whitespace) {
        Some(rest.trim().to_string())
    } else {
        None
    }
}

pub(crate) fn runtime_generation_for_session(session: &talos_session::Session) -> Result<u64> {
    PendingSubmissionStore::for_session(session)
        .runtime_generation()
        .map_err(|error| anyhow!("failed to load Session runtime generation: {error}"))
}

pub(crate) fn session_metadata_for_model(model: &str, provider: &str) -> SessionMetadata {
    SessionMetadata {
        turn_id: None,
        provider: (!provider.is_empty()).then(|| provider.to_string()),
        model: (!model.is_empty()).then(|| model.to_string()),
        token_count: None,
        working_directory: std::env::current_dir()
            .ok()
            .map(|path| path.to_string_lossy().to_string()),
        reasoning: None,
        raw_content: None,
    }
}

enum LatestSessionModelInfo {
    Activation(SessionModelIdentity),
    Legacy { model: String, provider: String },
}

fn latest_session_model_info(session: &talos_session::Session) -> Option<LatestSessionModelInfo> {
    match PendingSubmissionStore::for_session(session).runtime_state() {
        Ok(Some(state)) if state.status == SessionRuntimeActivationStatus::Committed => {
            return Some(LatestSessionModelInfo::Activation(state.activation.target));
        }
        Ok(Some(state)) => {
            tracing::error!(
                session_id = %session.id,
                activation_id = %state.activation.activation_id,
                "Session runtime activation is staged but its transcript marker is not committed; refusing to treat it as active"
            );
            return None;
        }
        Ok(None) => {}
        Err(error) => {
            tracing::warn!(session_id = %session.id, %error, "failed to load Session runtime state; falling back to transcript metadata");
        }
    }

    let mut legacy = None;
    for entry in session.read_entries().ok()?.into_iter().rev() {
        if entry.role == "system"
            && let Some(activation) = session_model_activation_from_metadata(&entry.metadata)
        {
            return Some(LatestSessionModelInfo::Activation(activation.target));
        }
        if legacy.is_none() {
            legacy = match (entry.metadata.model, entry.metadata.provider) {
                (Some(model), Some(provider)) => {
                    Some(LatestSessionModelInfo::Legacy { model, provider })
                }
                (Some(model), None) => Some(LatestSessionModelInfo::Legacy {
                    model,
                    provider: String::new(),
                }),
                _ => None,
            };
        }
    }
    legacy
}

pub(crate) fn reconcile_session_runtime_state(session: &talos_session::Session) -> Result<()> {
    let store = PendingSubmissionStore::for_session(session);
    let Some(state) = store
        .runtime_state()
        .map_err(|error| anyhow!("failed to load Session runtime identity: {error}"))?
    else {
        return Ok(());
    };
    if state.status == SessionRuntimeActivationStatus::Committed {
        return Ok(());
    }
    let tail = session
        .read_entries()
        .map_err(|error| anyhow!("failed to inspect pending Session activation: {error}"))?
        .last()
        .and_then(|entry| session_model_activation_from_metadata(&entry.metadata));
    if tail.as_ref() == Some(&state.activation) {
        store
            .commit_runtime_activation(&state.activation.activation_id)
            .map_err(|error| anyhow!("failed to finalize Session runtime activation: {error}"))?;
        return Ok(());
    }
    Err(anyhow!(
        "Session runtime activation {} is fenced but its exact transcript marker is absent; the Session remains stopped",
        state.activation.activation_id
    ))
}

pub(crate) fn ensure_session_runtime_identity(
    config: &Config,
    session: &talos_session::Session,
) -> Result<talos_session::SessionRuntimeState> {
    let identity =
        SessionRuntimeIdentity::new(&config.provider, &config.model, config.variant.as_deref());
    let store = PendingSubmissionStore::for_session(session);
    if let Some(state) = store
        .runtime_state()
        .map_err(|error| anyhow!("failed to inspect Session runtime identity: {error}"))?
        && state.status == SessionRuntimeActivationStatus::Committed
        && state.activation.target == identity
    {
        return Ok(state);
    }
    store
        .initialize_runtime_identity(identity)
        .map_err(|error| anyhow!("failed to initialize Session runtime identity: {error}"))
}

pub(crate) fn apply_session_model_to_config(config: &mut Config, session: &talos_session::Session) {
    let Some(model_info) = latest_session_model_info(session) else {
        return;
    };
    let (model, provider, exact_variant) = match model_info {
        LatestSessionModelInfo::Activation(identity) => {
            (identity.model, identity.provider, Some(identity.variant))
        }
        LatestSessionModelInfo::Legacy { model, provider } => (model, provider, None),
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
        return;
    }
    if let Some(variant) = exact_variant {
        crate::model_lifecycle::apply_variant_change(config, variant.as_deref());
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

pub(crate) fn set_request_budget_spec(agent: &mut Agent, config: &Config) {
    agent.set_request_budget_spec(talos_agent::RequestBudgetSpec::new(
        crate::provider_setup::effective_output_limit(config),
    ));
}

pub(crate) fn set_image_input_capability(agent: &mut Agent, config: &Config) {
    let cap = config.resolved_image_input_capability();
    agent.set_image_input_supported(cap.allows_attachment());
}

pub(crate) fn set_todo_prompt_provider(
    agent: &mut Agent,
    session_manager: &SessionManager,
    session: &talos_session::Session,
) {
    let session_manager = session_manager.clone();
    let session_id = session.id;
    let provider = Arc::new(move || -> Option<String> {
        let repo = match session_manager.todo_repository() {
            Ok(repo) => repo,
            Err(err) => {
                tracing::warn!("todo repository unavailable, skipping prompt injection: {err}");
                return None;
            }
        };
        format_session_todo_prompt(&repo, session_id)
    });
    agent.set_todo_section_provider(provider);
}

pub(crate) fn format_session_todo_prompt(
    repo: &TodoRepository,
    session_id: Uuid,
) -> Option<String> {
    let mut items = repo.list(session_id, TodoQuery::default()).ok()?;
    items.retain(|item| item.status != TodoStatus::Completed);
    if items.is_empty() {
        return None;
    }

    items.sort_by(|a, b| {
        status_rank(a.status)
            .cmp(&status_rank(b.status))
            .then_with(|| priority_rank(b.priority).cmp(&priority_rank(a.priority)))
            .then_with(|| a.created_at.cmp(&b.created_at))
            .then_with(|| a.id.cmp(&b.id))
    });

    let total = items.len();
    let mut section = String::from(
        "Active items from the current session todo list. Treat this as advisory planning context; use todo_* tools for updates.\n",
    );
    for item in items.iter().take(TODO_PROMPT_MAX_ITEMS) {
        let line = format_todo_prompt_item(item);
        if section.len() + line.len() > TODO_PROMPT_MAX_CHARS {
            section
                .push_str("- ... todo prompt budget reached; use /todo or todo_query for more.\n");
            return Some(section);
        }
        section.push_str(&line);
    }
    if total > TODO_PROMPT_MAX_ITEMS {
        section.push_str(&format!(
            "- ... {} more active item(s) omitted by prompt item budget.\n",
            total - TODO_PROMPT_MAX_ITEMS
        ));
    }
    Some(section)
}

fn format_todo_prompt_item(item: &TodoItem) -> String {
    let short_id = item.id.to_string();
    let short_id = &short_id[..8];
    let mut line = format!(
        "- [{status}][{priority}] {short_id} {title}\n",
        status = item.status.as_str(),
        priority = item.priority.as_str(),
        title = truncate_chars(item.title.trim(), 160),
    );
    if let Some(description) = item.description.as_deref() {
        let description = description.trim();
        if !description.is_empty() {
            line.push_str(&format!("  note: {}\n", truncate_chars(description, 220)));
        }
    }
    if !item.tags.is_empty() {
        line.push_str(&format!("  tags: {}\n", item.tags.join(", ")));
    }
    line
}

fn priority_rank(priority: TodoPriority) -> u8 {
    match priority {
        TodoPriority::Critical => 4,
        TodoPriority::High => 3,
        TodoPriority::Medium => 2,
        TodoPriority::Low => 1,
    }
}

fn status_rank(status: TodoStatus) -> u8 {
    match status {
        TodoStatus::InProgress => 0,
        TodoStatus::Blocked => 1,
        TodoStatus::Todo => 2,
        TodoStatus::Completed => 3,
    }
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    let mut end = 0;
    for (count, (idx, ch)) in value.char_indices().enumerate() {
        if count == max_chars {
            return format!("{}...", &value[..end]);
        }
        end = idx + ch.len_utf8();
    }
    value[..end].to_string()
}

pub(crate) fn model_metadata_context_file(config: &Config) -> ContextFile {
    let (context_limit, output_limit) = config.resolve_model_limits();
    let protocol = config.resolve_provider_protocol();
    let output_limit = output_limit
        .map(|limit| limit.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    ContextFile {
        path: "TALOS_MODEL.md".into(),
        content: format!(
            "# Current Model\n\
             Provider: {}\n\
             Model: {}\n\
             Provider protocol: {}\n\
             Protocol source: {}\n\
             Tool protocol: {:?}\n\
             Context limit: {} tokens\n\
             Output limit: {} tokens\n",
            config.provider,
            config.model,
            protocol.protocol,
            protocol.source,
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
            url: None,
            sse_post_url: None,
            headers: std::collections::HashMap::new(),
            auth_token_env: None,
            authorization_env: None,
        }];
    }

    #[cfg(not(debug_assertions))]
    let _ = (config, cli);
}

/// Allocate an operation identity without recycling IDs on session transitions.
pub(crate) fn manual_compact_operation(
    generation: u64,
) -> anyhow::Result<talos_core::session::SessionOp> {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let operation_id = NEXT
        .try_update(
            std::sync::atomic::Ordering::Relaxed,
            std::sync::atomic::Ordering::Relaxed,
            |id| id.checked_add(1),
        )
        .map_err(|_| anyhow::anyhow!("manual compaction operation IDs exhausted"))?
        + 1;
    Ok(talos_core::session::SessionOp::Compact {
        session_generation: generation,
        operation_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_session::{CreateTodo, TodoPriority};

    #[tokio::test]
    #[allow(deprecated)]
    async fn reasoning_wire_budget_matches_registered_agent_output_reservation() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let mut measured = Vec::new();
        for (output, thinking) in [(4096, 1024), (8192, 6000)] {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .expect("loopback");
            let url = format!("http://{}", listener.local_addr().expect("address"));
            let server = tokio::spawn(async move {
                tokio::time::timeout(std::time::Duration::from_secs(5), async {
                    let (mut socket, _) = listener.accept().await.expect("request");
                    let mut bytes = Vec::new();
                    let mut chunk = [0; 4096];
                    let body = loop {
                        let count = socket.read(&mut chunk).await.expect("read");
                        assert_ne!(count, 0);
                        bytes.extend_from_slice(&chunk[..count]);
                        assert!(bytes.len() <= 128 * 1024, "bounded fixture request");
                        if let Some(end) = bytes.windows(4).position(|b| b == b"\r\n\r\n") {
                            let headers = std::str::from_utf8(&bytes[..end]).expect("headers");
                            let length: usize = headers.lines().find_map(|line| {
                                let (name, value) = line.split_once(':')?;
                                name.eq_ignore_ascii_case("content-length").then(|| value.trim().parse().expect("length"))
                            }).expect("content length");
                            if bytes.len() >= end + 4 + length {
                                break serde_json::from_slice::<serde_json::Value>(&bytes[end + 4..end + 4 + length]).expect("body");
                            }
                        }
                    };
                    let response = concat!(
                        "event: message_start\ndata: {\"type\":\"message_start\",\"message\":{\"id\":\"fixture\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"reasoning-fixture\",\"stop_reason\":null,\"usage\":{\"input_tokens\":1,\"output_tokens\":0}}}\n\n",
                        "event: content_block_start\ndata: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\n",
                        "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"ok\"}}\n\n",
                        "event: content_block_stop\ndata: {\"type\":\"content_block_stop\",\"index\":0}\n\n",
                        "event: message_delta\ndata: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":1}}\n\n",
                        "event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n"
                    );
                    let wire = format!("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response}", response.len());
                    socket.write_all(wire.as_bytes()).await.expect("response");
                    body
                }).await.expect("fixture deadline")
            });
            let mut config = Config {
                provider: "fixture".into(),
                model: "reasoning-fixture".into(),
                ..Default::default()
            };
            config.providers.insert(
                "fixture".into(),
                talos_config::ProviderConfig {
                    protocol: talos_config::ProviderProtocol::AnthropicMessages,
                    base_url: Some(url),
                    models: std::collections::HashMap::from([(
                        "reasoning-fixture".into(),
                        talos_config::ModelConfig {
                            context_limit: Some(32_000),
                            output_limit: Some(output),
                            reasoning_supported: Some(true),
                            reasoning: Some(talos_config::ReasoningOptions {
                                budget_tokens: Some(thinking),
                                ..Default::default()
                            }),
                            ..Default::default()
                        },
                    )]),
                    ..Default::default()
                },
            );
            let provider = talos_provider::openai_responses::factory::build_configured_provider(
                &config,
                "fixture-key",
            )
            .expect("configured adapter");
            let mut agent = Agent::new(provider.clone(), talos_core::tool::ToolRegistry::new());
            agent
                .enable_model_checkpoint_tools()
                .expect("registered tools");
            set_request_budget_spec(&mut agent, &config);
            let reserved = agent.request_budget_spec().requested_output_tokens;
            let root = tempfile::tempdir().expect("workspace");
            let (handle, mut actor) = talos_agent::session::AppServerSession::new(
                agent,
                talos_core::session::SessionConfig {
                    runtime_policy: talos_core::session::RuntimePolicy::interactive(),
                    workspace_root: root.path().to_path_buf(),
                    initial_history: vec![],
                    model_context_limit: 32_000,
                },
            );
            let join = tokio::spawn(async move { actor.run().await });
            let mut events = handle.eq_rx;
            handle
                .sq_tx
                .send(talos_core::session::SessionOp::Submit {
                    message: "bounded thinking request".into(),
                })
                .await
                .expect("submit");
            let mut budget = None;
            let mut completed = false;
            tokio::time::timeout(std::time::Duration::from_secs(5), async {
                while let Some(event) = events.recv().await {
                    if let talos_core::session::SessionEvent::TurnEvent { payload, .. } = event {
                        match payload {
                            talos_core::session::TurnEventPayload::Progress { event: talos_core::message::AgentEvent::ContextBudget { budget: value } } => budget = Some(value),
                            talos_core::session::TurnEventPayload::Completed { status } => {
                                assert!(matches!(status, talos_core::session::TurnCompletionStatus::Success { .. }), "{status:?}");
                                completed = true;
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            }).await.expect("Actor deadline");
            assert!(completed);
            let budget = budget.expect("actual admission budget");
            assert_eq!(budget.limit, Some(32_000));
            measured.push(budget.estimated_tokens);
            handle
                .sq_tx
                .send(talos_core::session::SessionOp::Shutdown)
                .await
                .expect("shutdown");
            tokio::time::timeout(std::time::Duration::from_secs(5), join)
                .await
                .expect("shutdown deadline")
                .expect("Actor joined");
            let body = server.await.expect("server");
            for name in ["compact_context", "history_search"] {
                assert_eq!(
                    body["tools"]
                        .as_array()
                        .expect("registered wire tools")
                        .iter()
                        .filter(|tool| tool["name"] == name)
                        .count(),
                    1
                );
            }
            assert_eq!(body["max_tokens"], reserved);
            assert_eq!(body["thinking"]["budget_tokens"], thinking);
            assert_eq!(reserved, output);
            assert!(
                thinking < reserved,
                "thinking consumes the total output allocation; do not add it twice"
            );
        }
        assert_eq!(
            measured[1] - measured[0],
            4096,
            "only the total output reservation changes admission cost"
        );
    }

    #[test]
    fn todo_prompt_includes_only_active_items() {
        let dir = tempfile::tempdir().expect("operation should succeed");
        let manager = SessionManager::with_dir(dir.path().to_path_buf());
        let session = manager
            .create_session("project", "")
            .expect("operation should succeed");
        let repo = manager.todo_repository().expect("operation should succeed");

        repo.create(CreateTodo {
            session_id: session.id,
            title: "Active item".to_string(),
            description: Some("Keep this visible".to_string()),
            priority: TodoPriority::High,
            assigned_to_turn: None,
            tags: vec!["ship".to_string()],
        })
        .expect("operation should succeed");
        let completed = repo
            .create(CreateTodo {
                session_id: session.id,
                title: "Completed item".to_string(),
                description: None,
                priority: TodoPriority::Critical,
                assigned_to_turn: None,
                tags: vec![],
            })
            .expect("operation should succeed");
        repo.update_status(session.id, completed.id, TodoStatus::Completed)
            .expect("operation should succeed");

        let prompt =
            format_session_todo_prompt(&repo, session.id).expect("operation should succeed");
        assert!(prompt.contains("Active item"));
        assert!(prompt.contains("Keep this visible"));
        assert!(prompt.contains("tags: ship"));
        assert!(!prompt.contains("Completed item"));
    }

    #[test]
    fn todo_prompt_is_bounded_by_item_count() {
        let dir = tempfile::tempdir().expect("operation should succeed");
        let manager = SessionManager::with_dir(dir.path().to_path_buf());
        let session = manager
            .create_session("project", "")
            .expect("operation should succeed");
        let repo = manager.todo_repository().expect("operation should succeed");

        for index in 0..(TODO_PROMPT_MAX_ITEMS + 2) {
            repo.create(CreateTodo {
                session_id: session.id,
                title: format!("Item {index}"),
                description: None,
                priority: TodoPriority::Medium,
                assigned_to_turn: None,
                tags: vec![],
            })
            .expect("operation should succeed");
        }

        let prompt =
            format_session_todo_prompt(&repo, session.id).expect("operation should succeed");
        assert!(prompt.contains("2 more active item(s) omitted"));
        assert_eq!(prompt.matches("- [").count(), TODO_PROMPT_MAX_ITEMS);
    }

    fn activation_test_session(name: &str) -> talos_session::Session {
        let dir = tempfile::tempdir().expect("operation should succeed");
        let root = dir.keep();
        let path = root.join(format!("{name}.jsonl"));
        std::fs::write(&path, b"").expect("operation should succeed");
        talos_session::Session::new(Uuid::new_v4(), "test".into(), String::new(), path)
    }

    fn append_activation(session: &talos_session::Session, activation: &SessionModelActivation) {
        session
            .append_with_metadata(
                &talos_core::message::Message::System {
                    content: format!("[System] activation {}", activation.activation_id),
                    cache_markers: Vec::new(),
                },
                session_model_activation_metadata(activation).expect("operation should succeed"),
            )
            .expect("operation should succeed");
    }

    fn provider_reasoning_effort(config: &Config) -> Option<String> {
        let provider = crate::provider_setup::build_provider(config, "sk-test-session", false)
            .expect("provider");
        let preview = provider
            .request_preview(&[talos_core::message::Message::User {
                content: "session reconstruction probe".to_string(),
            }])
            .expect("real provider request preview");
        preview
            .pointer("/body/reasoning_effort")
            .or_else(|| preview.pointer("/body/reasoning/effort"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
    }

    #[test]
    fn session_owned_activation_survives_repeated_production_compaction() {
        use talos_core::message::Message;

        let session = activation_test_session("compaction-stable-activation");
        let low = SessionModelIdentity::new("openai", "o3", Some("low-reasoning"));
        let high = SessionModelIdentity::new("openai", "o3", Some("high-reasoning"));
        let store = PendingSubmissionStore::for_session(&session);
        store
            .initialize_runtime_identity(low.clone())
            .expect("operation should succeed");
        let activation = SessionModelActivation::new(1, low, high.clone());
        store
            .stage_runtime_activation(0, &activation)
            .expect("operation should succeed");
        append_activation(&session, &activation);
        store
            .commit_runtime_activation(&activation.activation_id)
            .expect("operation should succeed");

        for index in 0..205 {
            session
                .append(&Message::User {
                    content: format!("post-activation-turn-{index}"),
                })
                .expect("operation should succeed");
        }
        session
            .compact_archived(50)
            .expect("operation should succeed");
        session
            .compact_archived(50)
            .expect("operation should succeed");

        assert_eq!(
            session
                .read_entries()
                .expect("operation should succeed")
                .iter()
                .filter(|entry| {
                    session_model_activation_from_metadata(&entry.metadata).is_some()
                })
                .count(),
            0,
            "compaction must not retain or duplicate an old model-visible marker"
        );
        assert_eq!(
            PendingSubmissionStore::for_session(&session)
                .runtime_state()
                .expect("operation should succeed")
                .expect("operation should succeed")
                .activation
                .target,
            high
        );

        let mut restarted_global = Config::default();
        restarted_global
            .set_active_model("openai/o3")
            .expect("builtin openai/o3 model");
        crate::model_lifecycle::apply_variant_change(&mut restarted_global, Some("low-reasoning"));
        apply_session_model_to_config(&mut restarted_global, &session);

        assert_eq!(restarted_global.variant.as_deref(), Some("high-reasoning"));
        assert_eq!(
            provider_reasoning_effort(&restarted_global).as_deref(),
            Some("high")
        );
    }

    #[test]
    fn initial_session_identity_wins_over_later_global_variant_without_switch_marker() {
        let session = activation_test_session("initial-session-identity");
        let mut initial = Config::default();
        initial
            .set_active_model("openai/o3")
            .expect("builtin openai/o3 model");
        crate::model_lifecycle::apply_variant_change(&mut initial, Some("high-reasoning"));
        ensure_session_runtime_identity(&initial, &session).expect("operation should succeed");
        assert!(
            session
                .read_entries()
                .expect("operation should succeed")
                .is_empty()
        );

        let mut later_global = initial.clone();
        crate::model_lifecycle::apply_variant_change(&mut later_global, Some("low-reasoning"));
        apply_session_model_to_config(&mut later_global, &session);

        assert_eq!(later_global.variant.as_deref(), Some("high-reasoning"));
        assert_eq!(
            provider_reasoning_effort(&later_global).as_deref(),
            Some("high")
        );
    }

    #[test]
    fn resume_reuses_committed_switched_activation_with_matching_target() {
        let session = activation_test_session("resume-matching-switched-activation");
        let low = SessionModelIdentity::new("openai", "o3", Some("low-reasoning"));
        let high = SessionModelIdentity::new("openai", "o3", Some("high-reasoning"));
        let store = PendingSubmissionStore::for_session(&session);
        store
            .initialize_runtime_identity(low.clone())
            .expect("operation should succeed");
        let activation = SessionModelActivation::new(1, low, high.clone());
        store
            .stage_runtime_activation(0, &activation)
            .expect("operation should succeed");
        append_activation(&session, &activation);
        store
            .commit_runtime_activation(&activation.activation_id)
            .expect("operation should succeed");

        let mut resumed = Config::default();
        resumed
            .set_active_model("openai/o3")
            .expect("builtin openai/o3 model");
        crate::model_lifecycle::apply_variant_change(&mut resumed, Some("high-reasoning"));
        let state = ensure_session_runtime_identity(&resumed, &session)
            .expect("matching committed target must be idempotent");

        assert_eq!(state.activation, activation);
        assert_eq!(state.activation.target, high);
    }

    #[test]
    fn resume_rejects_identity_that_differs_from_committed_target() {
        let session = activation_test_session("resume-mismatched-activation");
        let mut initial = Config::default();
        initial
            .set_active_model("openai/o3")
            .expect("builtin openai/o3 model");
        crate::model_lifecycle::apply_variant_change(&mut initial, Some("high-reasoning"));
        ensure_session_runtime_identity(&initial, &session).expect("operation should succeed");

        crate::model_lifecycle::apply_variant_change(&mut initial, Some("low-reasoning"));
        let error = ensure_session_runtime_identity(&initial, &session)
            .expect_err("a genuine target mismatch must remain fenced");

        assert!(
            error.to_string().contains("runtime activation conflict"),
            "{error}"
        );
    }

    #[test]
    fn session_activation_restores_exact_variant_and_reasoning_options() {
        let session = activation_test_session("restore-variant");
        let activation = SessionModelActivation::new(
            7,
            SessionModelIdentity::new("openai", "o3", Some("low-reasoning")),
            SessionModelIdentity::new("openai", "o3", Some("high-reasoning")),
        );
        append_activation(&session, &activation);

        let mut config = Config {
            variant: Some("low-reasoning".into()),
            ..Default::default()
        };
        apply_session_model_to_config(&mut config, &session);

        assert_eq!(config.provider, "openai");
        assert_eq!(config.model, "o3");
        assert_eq!(config.variant.as_deref(), Some("high-reasoning"));

        assert_eq!(provider_reasoning_effort(&config).as_deref(), Some("high"));
    }

    #[test]
    fn session_activation_normalizes_default_and_clears_stale_variant() {
        let session = activation_test_session("clear-variant");
        let activation = SessionModelActivation::new(
            8,
            SessionModelIdentity::new("openai", "o3", Some("high-reasoning")),
            SessionModelIdentity::new("openai", "gpt-4o", Some("default")),
        );
        assert_eq!(activation.target.variant, None);
        append_activation(&session, &activation);

        let mut config = Config {
            variant: Some("high-reasoning".into()),
            ..Default::default()
        };
        apply_session_model_to_config(&mut config, &session);

        assert_eq!(config.provider, "openai");
        assert_eq!(config.model, "gpt-4o");
        assert_eq!(config.variant, None);
        assert_eq!(provider_reasoning_effort(&config), None);
    }

    #[test]
    fn unknown_persisted_variant_restores_identity_but_uses_safe_fallback() {
        let session = activation_test_session("unknown-variant");
        let activation = SessionModelActivation::new(
            9,
            SessionModelIdentity::new("openai", "o3", None),
            SessionModelIdentity::new("openai", "o3", Some("deleted-variant")),
        );
        append_activation(&session, &activation);

        let mut config = Config::default();
        apply_session_model_to_config(&mut config, &session);
        assert_eq!(config.variant.as_deref(), Some("deleted-variant"));

        let (_, resolution) = crate::model_lifecycle::materialize_runtime_model_config(&config);
        assert_eq!(resolution.reasoning_effort, None);
        assert!(resolution.diagnostic.is_some());
        assert_eq!(provider_reasoning_effort(&config), None);
    }
}
