//! Model lifecycle helpers for the Talos CLI.
//!
//! Contains the model picker data construction and the shared session rebuild
//! logic used when switching models at runtime.

use std::sync::Arc;

use talos_agent::Agent;
use talos_agent::session::AppServerSession;
use talos_config::{Config, ReasoningOptions};
use talos_conversation::{ModelPickerData, ModelPickerItem, ModelPickerVariantItem};
use talos_core::message::Message;
use talos_core::model::{ModelCapabilities, ReasoningEffort, VariantDef};
use talos_core::session::{RuntimePolicy, SessionConfig, SessionEvent, SessionOp};
use talos_plugin::HookRegistry;
use talos_session::{Session, SessionManager};
use tokio::sync::{Mutex, mpsc, watch};

use crate::mcp_runtime::McpSessionRuntime;
use crate::registry::{
    TuiApprovalHandler, build_tui_tool_registry, register_tui_permission_aware_tools,
};
use crate::session_transition::SessionTransition;
use crate::skill_runtime::{apply_runtime_skills, discover_runtime_skills};

/// Constructs [`ModelPickerData`] from the given [`Config`].
///
/// Iterates the model catalog, checks provider authentication, and formats display strings. Models from
/// authenticated providers appear in `ready_models`; unauthenticated providers
/// are intentionally omitted from `/model` and handled by `/connect`.
pub(crate) fn build_model_picker_data(config: &Config) -> ModelPickerData {
    let catalog = config.all_models();

    let mut ready_models: Vec<ModelPickerItem> = Vec::new();
    for m in &catalog {
        let provider_authed = config.provider_authenticated(&m.provider);
        let pricing_str = m.pricing.as_ref().map(|p| {
            let input = p.input_per_1m.map(|v| format!("${v}")).unwrap_or_default();
            let output = p.output_per_1m.map(|v| format!("${v}")).unwrap_or_default();
            if input.is_empty() && output.is_empty() {
                String::new()
            } else {
                format!("{input}/{output}")
            }
        });
        let ctx_str = m
            .context_limit
            .map(|c| format!("{}K", c / 1000))
            .unwrap_or_else(|| "?".to_string());

        if provider_authed {
            ready_models.push(ModelPickerItem {
                command: "/model".to_string(),
                // Provider identity travels in the separate `provider` field.
                // Keeping this provider-side ID opaque avoids double-prefixing
                // duplicate model IDs in the structured switch lifecycle.
                model_id: m.id.clone(),
                provider: m.provider.clone(),
                label: format!("{}   {}   {}", m.id, m.provider, ctx_str),
                context_limit: m.context_limit,
                pricing: pricing_str,
                authenticated: true,
                is_current: m.id == config.model && m.provider == config.provider,
                variants: m
                    .variants
                    .iter()
                    .map(|variant| ModelPickerVariantItem {
                        variant_id: variant.id.clone(),
                        label: variant.label.clone(),
                        provider: m.provider.clone(),
                        model_id: m.id.clone(),
                    })
                    .collect(),
                variant: None,
            });
        }
    }

    let mut recent_items: Vec<ModelPickerItem> = Vec::new();
    let recent_list = crate::recent_models::load_recent_models(None);

    for entry in recent_list.entries {
        if !config.provider_authenticated(&entry.provider) {
            continue;
        }

        let m = catalog
            .iter()
            .find(|c| c.provider == entry.provider && c.id == entry.model_id);
        if let Some(m) = m {
            let is_current = entry.provider == config.provider
                && entry.model_id == config.model
                && entry.variant == config.variant;
            if is_current {
                continue;
            }

            let pricing_str = m.pricing.as_ref().map(|p| {
                let input = p.input_per_1m.map(|v| format!("${v}")).unwrap_or_default();
                let output = p.output_per_1m.map(|v| format!("${v}")).unwrap_or_default();
                if input.is_empty() && output.is_empty() {
                    String::new()
                } else {
                    format!("{input}/{output}")
                }
            });
            let ctx_str = m
                .context_limit
                .map(|c| format!("{}K", c / 1000))
                .unwrap_or_else(|| "?".to_string());

            recent_items.push(ModelPickerItem {
                command: "/model".to_string(),
                model_id: m.id.clone(),
                provider: m.provider.clone(),
                label: format!("{}   {}   {}", m.id, m.provider, ctx_str),
                context_limit: m.context_limit,
                pricing: pricing_str,
                authenticated: true,
                is_current: false,
                variants: m
                    .variants
                    .iter()
                    .map(|variant| ModelPickerVariantItem {
                        variant_id: variant.id.clone(),
                        label: variant.label.clone(),
                        provider: m.provider.clone(),
                        model_id: m.id.clone(),
                    })
                    .collect(),
                variant: entry.variant.clone(),
            });
        }
    }

    ModelPickerData {
        recent: recent_items,
        ready_models,
        setup_providers: Vec::new(),
    }
}

/// The safe runtime projection of a selected catalog variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VariantResolution {
    /// Reasoning effort to apply to the provider request, when supported.
    pub reasoning_effort: Option<ReasoningEffort>,
    /// Bounded note for a selected variant absent from the active model catalog.
    pub diagnostic: Option<String>,
}

/// Resolves a selected variant against the active model's catalog metadata.
///
/// The legacy `"default"` identity preserves baseline behavior without a
/// diagnostic. Reasoning overrides are silently omitted when unsupported.
pub(crate) fn resolve_variant(
    variant_id: Option<&str>,
    model_variants: &[VariantDef],
    model_capabilities: &ModelCapabilities,
) -> VariantResolution {
    let Some(variant_id) = variant_id.filter(|id| *id != "default") else {
        return VariantResolution {
            reasoning_effort: None,
            diagnostic: None,
        };
    };

    let Some(variant) = model_variants
        .iter()
        .find(|variant| variant.id == variant_id)
    else {
        return VariantResolution {
            reasoning_effort: None,
            diagnostic: Some(format!(
                "Variant '{variant_id}' not found; using no variant"
            )),
        };
    };

    VariantResolution {
        reasoning_effort: variant
            .reasoning_effort
            .clone()
            .filter(|_| model_capabilities.reasoning),
        diagnostic: None,
    }
}

/// Applies a variant change to a Config in-memory.
///
/// Returns `true` when the value actually changed (so the caller can persist).
/// Always assigns — including `None` — so switching to a variant-less model
/// correctly clears any prior variant. This is the single source of truth for
/// variant-clearing semantics across all model-switch entry points.
pub(crate) fn apply_variant_change(config: &mut Config, new_variant: Option<&str>) -> bool {
    let changed = config.variant.as_deref() != new_variant;
    if changed {
        config.variant = new_variant.map(str::to_string);
    }
    changed
}

/// Resolves the model to activate after provider-level credential setup.
///
/// Provider setup rows represent a provider rather than a specific model. After
/// credentials are saved, prefer the current configured model when it belongs
/// to that provider; otherwise use the first catalog model for the provider.
/// Duplicate model IDs are provider-qualified so `Config::set_active_model`
/// resolves the intended provider.
pub(crate) fn provider_setup_target_model(config: &Config, provider: &str) -> Option<String> {
    let catalog = config.all_models();
    let mut id_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for m in &catalog {
        *id_counts.entry(m.id.as_str()).or_default() += 1;
    }

    let target = catalog
        .iter()
        .find(|m| m.provider == provider && m.id == config.model)
        .or_else(|| catalog.iter().find(|m| m.provider == provider))?;

    Some(
        if id_counts.get(target.id.as_str()).copied().unwrap_or(0) > 1 {
            format!("{}/{}", target.provider, target.id)
        } else {
            target.id.clone()
        },
    )
}

/// Parameters for the shared session rebuild logic when switching models.
///
/// This struct bundles all the context needed to rebuild a session for a new
/// model, parameterized by the already-resolved `api_key` so that both the
/// normal model switch path and the credential-first path can share the same
/// implementation.
pub(crate) struct RebuildSessionParams<'a> {
    pub transition: &'a Arc<Mutex<SessionTransition>>,
    pub ui_tx: &'a mpsc::UnboundedSender<talos_conversation::UiOutput>,
    pub model_config: &'a Config,
    pub hooks: &'a Arc<HookRegistry>,
    pub workspace_root: &'a std::path::Path,
    pub mcp_config: &'a talos_config::McpConfig,
    pub session_watch_tx: &'a watch::Sender<Session>,
    pub sq_tx_watch_tx: &'a watch::Sender<mpsc::Sender<SessionOp>>,
    pub bridge_rx_update_tx:
        &'a mpsc::UnboundedSender<(Session, mpsc::UnboundedReceiver<SessionEvent>)>,
    pub session_watch_rx: &'a watch::Receiver<Session>,
    pub session_manager: &'a SessionManager,
    pub api_key: String,
    pub previous_model: String,
    pub previous_provider: String,
    pub model_id: String,
    pub variant: Option<String>,
    pub provider_for_status: String,
    pub success_message: String,
    pub mock: bool,
}

/// Shared session rebuild logic for model switching.
///
/// Encapsulates the common agent-build + transition logic: resolves model
/// limits, ensures session persistence, reads history, builds SessionConfig,
/// constructs provider+registry+agent, prepares+commits the SessionTransition,
/// and updates watch channels.
///
/// The caller is responsible for resolving the `api_key` and constructing the
/// `success_message` and `provider_for_status` strings, which differ between
/// the normal model switch path and the credential-first path.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn rebuild_session_for_model(params: RebuildSessionParams<'_>) -> bool {
    let RebuildSessionParams {
        transition,
        ui_tx,
        model_config,
        hooks,
        workspace_root,
        mcp_config,
        session_watch_tx,
        sq_tx_watch_tx,
        bridge_rx_update_tx,
        session_watch_rx,
        session_manager,
        api_key,
        previous_model,
        previous_provider,
        model_id,
        variant,
        provider_for_status,
        success_message,
        mock,
    } = params;

    let mut runtime_model_config = model_config.clone();
    let all_models = model_config.all_models();
    let metadata = talos_config::model::find_model_by_provider(
        &all_models,
        &model_config.provider,
        &model_config.model,
    );
    let resolution = metadata.map_or_else(
        || resolve_variant(variant.as_deref(), &[], &ModelCapabilities::default()),
        |model| resolve_variant(variant.as_deref(), &model.variants, &model.capabilities),
    );
    if let Some(diagnostic) = resolution.diagnostic.as_deref() {
        tracing::warn!(
            provider = %model_config.provider,
            model = %model_config.model,
            "{diagnostic}"
        );
    }
    if let Some(reasoning_effort) = resolution.reasoning_effort {
        let provider = runtime_model_config
            .providers
            .entry(runtime_model_config.provider.clone())
            .or_default();
        let reasoning = provider
            .models
            .entry(runtime_model_config.model.clone())
            .or_default()
            .reasoning
            .get_or_insert_with(ReasoningOptions::default);
        reasoning.effort = Some(reasoning_effort);
    }

    let model_context_limit = runtime_model_config.resolve_model_limits().0;

    let mut current_session = session_watch_rx.borrow().clone();
    if let Err(e) = current_session.ensure_persisted() {
        let text = format!("[Error] Failed to create session file: {e}\n");
        send_stream(ui_tx, talos_conversation::MessageSource::Error, text);
        return false;
    }
    let mut history = current_session.read_messages().unwrap_or_default();
    let switch_marker = model_switch_marker(
        &previous_provider,
        &previous_model,
        &model_config.provider,
        &model_config.model,
    );
    history.push(switch_marker.clone());

    let session_config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: workspace_root.to_path_buf(),
        initial_history: history,
        model_context_limit,
    };

    let provider = crate::provider_setup::build_provider(&runtime_model_config, &api_key, mock);
    let approval_handler = Arc::new(TuiApprovalHandler::new(
        ui_tx.clone(),
        workspace_root.to_path_buf(),
    ));
    let mcp_runtime = match McpSessionRuntime::start(mcp_config, hooks.clone()).await {
        Ok(r) => r,
        Err(e) => {
            let text = format!("[Error] Failed to start MCP runtime: {e}\n");
            send_stream(ui_tx, talos_conversation::MessageSource::Error, text);
            return false;
        }
    };
    mcp_runtime.report_startup_failures();
    let (sched_tools, sched_pending) = talos_agent::create_scheduler_tools();
    let mut registry = build_tui_tool_registry(
        approval_handler.clone(),
        workspace_root.to_path_buf(),
        current_session.id,
        sched_tools,
    );
    register_tui_permission_aware_tools(&mut registry, mcp_runtime.tools(), approval_handler);

    let mut agent = Agent::with_security_and_hooks(
        provider,
        registry,
        Some(Arc::new(talos_permission::PermissionEngine::new())),
        None,
        workspace_root.to_path_buf(),
        hooks.clone(),
    );
    agent.set_tool_protocol(runtime_model_config.tool_protocol());
    crate::mode_runtime::set_image_input_capability(&mut agent, &runtime_model_config);
    if let Ok(skills) =
        discover_runtime_skills(workspace_root, runtime_model_config.skills.discover_shared)
    {
        apply_runtime_skills(&mut agent, &skills);
    }
    crate::mode_runtime::maybe_set_memory_provider(&mut agent, &runtime_model_config);
    crate::mode_runtime::set_todo_prompt_provider(&mut agent, session_manager, &current_session);
    match crate::mode_runners::context_files_for_agent(&runtime_model_config, workspace_root, true)
    {
        Ok(files) => agent.set_context_files(files),
        Err(e) => {
            let text = format!("[Error] Failed to load context files: {e}\n");
            send_stream(ui_tx, talos_conversation::MessageSource::Error, text);
            return false;
        }
    }

    let (handle, mut actor) = AppServerSession::new(agent, session_config);
    let _sched_join = sched_pending.spawn(
        handle.sq_tx.clone(),
        tokio_util::sync::CancellationToken::new(),
    );
    actor.set_persistence(
        current_session.clone(),
        crate::mode_runtime::session_metadata_for_model(
            &runtime_model_config.model,
            &runtime_model_config.provider,
        ),
    );
    let session_for_prepare = current_session.clone();
    if let Err(e) = transition.lock().await.prepare(handle, session_for_prepare) {
        let text = format!("[Error] Failed to prepare model switch: {e}\n");
        send_stream(ui_tx, talos_conversation::MessageSource::Error, text);
        return false;
    }

    let mut transition_guard = transition.lock().await;
    match transition_guard.commit(actor) {
        Ok(result) => {
            let _ = session_watch_tx.send(current_session.clone());
            let _ = sq_tx_watch_tx.send(result.new_handle.sq_tx.clone());
            if bridge_rx_update_tx
                .send((current_session.clone(), result.new_handle.eq_rx))
                .is_err()
            {
                eprintln!(
                    "[Error] Bridge forwarder unavailable; model switch events will not be persisted or displayed."
                );
            }
            send_stream(
                ui_tx,
                talos_conversation::MessageSource::System,
                success_message,
            );
            let marker_metadata = crate::mode_runtime::session_metadata_for_model(
                &runtime_model_config.model,
                &runtime_model_config.provider,
            );
            if let Err(e) = current_session.append_with_metadata(&switch_marker, marker_metadata) {
                eprintln!("Warning: failed to persist model switch marker: {e}");
            }
            let (ctx_limit, _) = runtime_model_config.resolve_model_limits();
            let all_models = runtime_model_config.all_models();
            let meta = talos_config::model::find_model_by_provider(
                &all_models,
                &runtime_model_config.provider,
                &runtime_model_config.model,
            );
            let pricing = meta.and_then(|m| m.pricing.as_ref());
            let _ = ui_tx.send(talos_conversation::UiOutput::Status(
                talos_conversation::StatusSnapshot {
                    model_name: model_id.clone(),
                    provider: provider_for_status,
                    context_limit: Some(ctx_limit),
                    input_price_per_million: pricing.and_then(|p| p.input_per_1m),
                    output_price_per_million: pricing.and_then(|p| p.output_per_1m),
                    variant: variant.clone(),
                    ..Default::default()
                },
            ));

            let mut recent = crate::recent_models::load_recent_models(None);
            recent.record(crate::recent_models::RecentModelEntry {
                provider: runtime_model_config.provider.clone(),
                model_id: runtime_model_config.model.clone(),
                variant,
            });
            if let Err(e) = crate::recent_models::save_recent_models(&recent, None) {
                tracing::warn!("Failed to persist recent models: {e}");
            }

            true
        }
        Err(e) => {
            transition_guard.rollback();
            let text = format!(
                "[Error] Failed to commit model switch: {e}. Previous model remains active.\n"
            );
            send_stream(ui_tx, talos_conversation::MessageSource::Error, text);
            false
        }
    }
}

fn model_switch_marker(
    previous_provider: &str,
    previous_model: &str,
    new_provider: &str,
    new_model: &str,
) -> Message {
    Message::System {
        content: format!(
            "[System] Model switch: {previous_provider}/{previous_model} -> {new_provider}/{new_model}.\n[System] Active model for subsequent requests: {new_provider}/{new_model}."
        ),
        cache_markers: Vec::new(),
    }
}

fn send_stream(
    ui_tx: &mpsc::UnboundedSender<talos_conversation::UiOutput>,
    source: talos_conversation::MessageSource,
    text: String,
) {
    use talos_conversation::{ContentOutput, UiOutput};

    let _ = ui_tx.send(UiOutput::Content(ContentOutput::Block { source, text }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_config::ProviderConfig;
    use talos_core::model::{ModelCapabilities, ReasoningEffort, VariantDef};
    use talos_core::tool::ToolRegistry;
    use talos_provider::mock::MockProvider;
    use uuid::Uuid;

    #[test]
    fn ready_models_have_correct_provider_and_context_limit() {
        let mut config = Config::default();
        config.model = "claude-sonnet-4-5".to_string();
        config.provider = "anthropic".to_string();
        config.providers.insert(
            "anthropic".to_string(),
            ProviderConfig {
                api_key: Some("sk-test-key".to_string()),
                ..Default::default()
            },
        );

        let data = build_model_picker_data(&config);

        let anthropic_models: Vec<_> = data
            .ready_models
            .iter()
            .filter(|m| m.provider == "anthropic")
            .collect();
        assert!(
            !anthropic_models.is_empty(),
            "Expected at least one anthropic model in ready_models"
        );

        for m in &data.ready_models {
            assert!(
                m.authenticated,
                "Model {} should be authenticated",
                m.model_id
            );
        }
    }

    #[test]
    fn model_switch_marker_includes_previous_and_new_identity() {
        let marker = model_switch_marker("anthropic", "claude-old", "openai", "gpt-new");
        let Message::System { content, .. } = marker else {
            panic!("model switch marker must be a system message");
        };

        assert!(content.contains("anthropic/claude-old"));
        assert!(content.contains("openai/gpt-new"));
        assert!(content.contains("Active model for subsequent requests"));
    }

    #[test]
    fn model_switch_marker_survives_session_jsonl_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let session = talos_session::Session::new(
            Uuid::new_v4(),
            "test".into(),
            String::new(),
            dir.path().join("session.jsonl"),
        );
        let marker = model_switch_marker("anthropic", "claude-old", "openai", "gpt-new");

        session
            .append_with_metadata(
                &marker,
                talos_session::SessionMetadata {
                    provider: Some("openai".into()),
                    model: Some("gpt-new".into()),
                    ..Default::default()
                },
            )
            .unwrap();

        let messages = session.read_messages().unwrap();
        assert_eq!(messages.len(), 1);
        let Message::System { content, .. } = &messages[0] else {
            panic!("round-tripped marker must remain a system message");
        };
        assert!(content.contains("anthropic/claude-old"));
        assert!(content.contains("openai/gpt-new"));

        let entries = session.read_entries().unwrap();
        assert_eq!(entries[0].metadata.provider, Some("openai".into()));
        assert_eq!(entries[0].metadata.model, Some("gpt-new".into()));
    }

    #[tokio::test]
    async fn model_switch_marker_is_visible_in_request_preview() {
        let marker = model_switch_marker("anthropic", "claude-old", "openai", "gpt-new");
        let provider = MockProvider::new().with_request_debug_builder(|messages| {
            let system_messages: Vec<_> = messages
                .iter()
                .filter_map(|message| match message {
                    Message::System { content, .. } => Some(content.clone()),
                    _ => None,
                })
                .collect();
            serde_json::json!({ "systems": system_messages }).to_string()
        });
        let agent = Agent::with_security(
            Arc::new(provider),
            ToolRegistry::new(),
            Some(Arc::new(talos_permission::PermissionEngine::new())),
            None,
            std::path::PathBuf::from("/tmp"),
        );

        let preview = agent
            .preview_request("continue".to_string(), vec![marker])
            .await
            .unwrap()
            .unwrap();

        assert!(preview.contains("Model switch"));
        assert!(preview.contains("openai/gpt-new"));
    }

    #[test]
    fn duplicate_model_ids_keep_provider_side_ids_for_structured_switching() {
        let mut config = Config::default();
        config.model = "glm-5.2".to_string();
        config.provider = "zai".to_string();
        config.providers.insert(
            "zai".to_string(),
            ProviderConfig {
                api_key: Some("sk-zai-key".to_string()),
                ..Default::default()
            },
        );
        config.providers.insert(
            "zhipuai".to_string(),
            ProviderConfig {
                api_key: Some("sk-zhipu-key".to_string()),
                ..Default::default()
            },
        );

        let data = build_model_picker_data(&config);

        let glm_entries: Vec<_> = data
            .ready_models
            .iter()
            .filter(|m| m.model_id.contains("glm-5.2"))
            .collect();

        assert!(
            glm_entries.len() > 1,
            "fixture must contain duplicate glm-5.2 IDs"
        );
        assert!(glm_entries.iter().all(|entry| entry.model_id == "glm-5.2"));
        assert!(glm_entries.iter().any(|entry| entry.provider == "zai"));
        assert!(glm_entries.iter().any(|entry| entry.provider == "zhipuai"));

        let selected = glm_entries
            .iter()
            .find(|entry| entry.provider == "zai")
            .expect("zai duplicate must be selectable");
        let mut selected_config = config.clone();
        selected_config
            .set_active_model(&format!("{}/{}", selected.provider, selected.model_id))
            .expect("structured provider + raw model ID must resolve a duplicate");
        assert_eq!(selected_config.provider, "zai");
        assert_eq!(selected_config.model, "glm-5.2");
    }

    #[test]
    fn unauthenticated_providers_are_omitted_from_model_picker() {
        let mut config = Config::default();
        config.model = "claude-sonnet-4-5".to_string();
        config.provider = "anthropic".to_string();
        config.providers.insert(
            "anthropic".to_string(),
            ProviderConfig {
                api_key: Some("sk-test-key".to_string()),
                ..Default::default()
            },
        );
        // openai has no api_key and no env var set — unauthenticated.
        config.providers.insert(
            "openai".to_string(),
            ProviderConfig {
                ..Default::default()
            },
        );

        let data = build_model_picker_data(&config);

        assert!(
            data.setup_providers.is_empty(),
            "unauthenticated providers belong in /connect, not /model"
        );
        assert!(
            data.ready_models.iter().all(|m| m.authenticated),
            "/model picker must contain only authenticated providers"
        );
        assert!(
            data.ready_models.iter().all(|m| m.provider != "openai"),
            "unauthenticated openai models must be omitted from /model"
        );
    }

    #[test]
    fn is_current_flags_active_model_and_provider() {
        let mut config = Config::default();
        config.model = "claude-sonnet-4-5".to_string();
        config.provider = "anthropic".to_string();
        config.providers.insert(
            "anthropic".to_string(),
            ProviderConfig {
                api_key: Some("sk-test-key".to_string()),
                ..Default::default()
            },
        );

        let data = build_model_picker_data(&config);

        let current_models: Vec<_> = data.ready_models.iter().filter(|m| m.is_current).collect();
        assert_eq!(
            current_models.len(),
            1,
            "Expected exactly one current model, found {}",
            current_models.len()
        );
        assert_eq!(
            current_models[0].model_id, "claude-sonnet-4-5",
            "Structured picker model IDs must remain provider-side IDs"
        );
        assert_eq!(
            current_models[0].provider, "anthropic",
            "Current model provider should be anthropic"
        );

        for m in &data.ready_models {
            if m.model_id != "claude-sonnet-4-5" || m.provider != "anthropic" {
                assert!(
                    !m.is_current,
                    "Model {} ({}) should not be current",
                    m.model_id, m.provider
                );
            }
        }
    }

    #[test]
    fn model_picker_includes_only_declared_variants() {
        let mut config = Config::default();
        config.providers.insert(
            "openai".to_string(),
            ProviderConfig {
                api_key: Some("sk-test-key".to_string()),
                ..Default::default()
            },
        );

        let data = build_model_picker_data(&config);
        let o3 = data
            .ready_models
            .iter()
            .find(|model| model.provider == "openai" && model.model_id == "o3")
            .expect("openai/o3 is in the picker");
        assert_eq!(o3.variants.len(), 2);
        assert_eq!(o3.variants[0].variant_id, "high-reasoning");
        assert_eq!(o3.variants[1].variant_id, "low-reasoning");

        let gpt_4o = data
            .ready_models
            .iter()
            .find(|model| model.provider == "openai" && model.model_id == "gpt-4o")
            .expect("openai/gpt-4o is in the picker");
        assert!(gpt_4o.variants.is_empty());
    }

    fn reasoning_variant() -> VariantDef {
        VariantDef {
            id: "high-reasoning".to_string(),
            label: "High Reasoning".to_string(),
            reasoning_effort: Some(ReasoningEffort::High),
        }
    }

    #[test]
    fn resolve_variant_without_selection_uses_baseline() {
        let resolution =
            resolve_variant(None, &[reasoning_variant()], &ModelCapabilities::default());

        assert_eq!(resolution.reasoning_effort, None);
        assert_eq!(resolution.diagnostic, None);
    }

    #[test]
    fn resolve_variant_unknown_selection_reports_bounded_diagnostic() {
        let resolution = resolve_variant(
            Some("removed-variant"),
            &[reasoning_variant()],
            &ModelCapabilities::default(),
        );

        assert_eq!(resolution.reasoning_effort, None);
        assert_eq!(
            resolution.diagnostic.as_deref(),
            Some("Variant 'removed-variant' not found; using no variant")
        );
    }

    #[test]
    fn resolve_variant_applies_reasoning_effort_when_supported() {
        let resolution = resolve_variant(
            Some("high-reasoning"),
            &[reasoning_variant()],
            &ModelCapabilities {
                reasoning: true,
                ..Default::default()
            },
        );

        assert_eq!(resolution.reasoning_effort, Some(ReasoningEffort::High));
        assert_eq!(resolution.diagnostic, None);
    }

    #[test]
    fn resolve_variant_omits_reasoning_effort_when_unsupported() {
        let resolution = resolve_variant(
            Some("high-reasoning"),
            &[reasoning_variant()],
            &ModelCapabilities::default(),
        );

        assert_eq!(resolution.reasoning_effort, None);
        assert_eq!(resolution.diagnostic, None);
    }

    #[test]
    fn resolve_variant_without_reasoning_override_is_valid() {
        let variant = VariantDef {
            id: "preset".to_string(),
            label: "Preset".to_string(),
            reasoning_effort: None,
        };
        let resolution = resolve_variant(
            Some("preset"),
            &[variant],
            &ModelCapabilities {
                reasoning: true,
                ..Default::default()
            },
        );

        assert_eq!(resolution.reasoning_effort, None);
        assert_eq!(resolution.diagnostic, None);
    }

    #[test]
    fn provider_setup_target_prefers_current_model_for_provider() {
        let mut config = Config::default();
        config.model = "glm-5.2".to_string();
        config.provider = "zai".to_string();

        let target = provider_setup_target_model(&config, "zai").expect("target model");

        assert_eq!(target, "zai/glm-5.2");
    }

    #[test]
    fn provider_setup_target_falls_back_to_first_provider_model() {
        let mut config = Config::default();
        config.model = "claude-sonnet-4-5".to_string();
        config.provider = "anthropic".to_string();

        let target = provider_setup_target_model(&config, "anthropic").expect("target model");

        assert!(!target.is_empty());
        let found = config
            .all_models()
            .into_iter()
            .find(|m| m.id == target || format!("{}/{}", m.provider, m.id) == target)
            .expect("target exists in catalog");
        assert_eq!(found.provider, "anthropic");
        // provider_setup_target_model for current provider returns the exact current model
        assert_eq!(found.id, "claude-sonnet-4-5");
    }

    // Regression coverage for the Oracle-identified variant-clearing bug:
    // switching from `Some(variant)` to `None` (variant-less model) must clear
    // `Config.variant` and report a change so the caller persists it.
    #[test]
    fn apply_variant_change_clears_when_switching_to_none() {
        let mut config = Config::default();
        config.variant = Some("high-reasoning".to_string());

        let changed = apply_variant_change(&mut config, None);
        assert!(changed, "switching Some → None must report a change");
        assert!(config.variant.is_none(), "variant must be cleared");
    }

    #[test]
    fn apply_variant_change_sets_when_switching_to_some() {
        let mut config = Config::default();
        config.variant = None;

        let changed = apply_variant_change(&mut config, Some("low-reasoning"));
        assert!(changed, "switching None → Some must report a change");
        assert_eq!(config.variant.as_deref(), Some("low-reasoning"));
    }

    #[test]
    fn apply_variant_change_updates_when_switching_between_variants() {
        let mut config = Config::default();
        config.variant = Some("high-reasoning".to_string());

        let changed = apply_variant_change(&mut config, Some("low-reasoning"));
        assert!(
            changed,
            "switching Some → Some(different) must report a change"
        );
        assert_eq!(config.variant.as_deref(), Some("low-reasoning"));
    }

    #[test]
    fn apply_variant_change_noop_when_value_matches() {
        let mut config = Config::default();
        config.variant = Some("high-reasoning".to_string());

        let changed = apply_variant_change(&mut config, Some("high-reasoning"));
        assert!(!changed, "identical value must not report a change");
        assert_eq!(config.variant.as_deref(), Some("high-reasoning"));
    }

    #[test]
    fn apply_variant_change_noop_when_both_none() {
        let mut config = Config::default();
        config.variant = None;

        let changed = apply_variant_change(&mut config, None);
        assert!(!changed, "None → None must not report a change");
        assert!(config.variant.is_none());
    }
}
