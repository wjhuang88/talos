//! Model lifecycle helpers for the Talos CLI.
//!
//! Contains the model picker data construction and the shared session rebuild
//! logic used when switching models at runtime.

use std::sync::Arc;

use talos_agent::Agent;
use talos_agent::session::AppServerSession;
use talos_config::Config;
use talos_conversation::{ModelPickerData, ModelPickerItem, ProviderSetupItem};
use talos_core::session::{SessionConfig, SessionEvent, SessionOp};
use talos_plugin::HookRegistry;
use talos_session::Session;
use tokio::sync::{Mutex, mpsc, watch};

use crate::mcp_runtime::McpSessionRuntime;
use crate::registry::{
    TuiApprovalHandler, build_tui_tool_registry, register_tui_permission_aware_tools,
};
use crate::session_transition::SessionTransition;
use crate::skill_runtime::{apply_runtime_skills, discover_runtime_skills};

/// Constructs [`ModelPickerData`] from the given [`Config`].
///
/// Iterates the model catalog, detects duplicate model IDs across providers,
/// checks provider authentication, and formats display strings. Models from
/// authenticated providers appear in `ready_models`; unauthenticated providers
/// are grouped into `setup_providers` with their model counts.
pub(crate) fn build_model_picker_data(config: &Config) -> ModelPickerData {
    use std::collections::BTreeMap;

    let catalog = config.all_models();

    // Detect model IDs that appear under multiple providers (e.g., glm-5.2
    // under zhipu, zai, zai-coding-plan). These need provider-qualified values
    // in the picker so the correct provider is selected.
    let mut id_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for m in &catalog {
        *id_counts.entry(m.id.as_str()).or_default() += 1;
    }

    let mut ready_models: Vec<ModelPickerItem> = Vec::new();
    let mut unauthed_by_provider: BTreeMap<String, usize> = BTreeMap::new();

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

        // Provider-qualify the model ID for the picker value when duplicates exist.
        let picker_value = if id_counts.get(m.id.as_str()).copied().unwrap_or(0) > 1 {
            format!("{}/{}", m.provider, m.id)
        } else {
            m.id.clone()
        };

        if provider_authed {
            ready_models.push(ModelPickerItem {
                command: "/model".to_string(),
                model_id: picker_value,
                provider: m.provider.clone(),
                label: format!("{}   {}   {}", m.id, m.provider, ctx_str),
                context_limit: m.context_limit,
                pricing: pricing_str,
                authenticated: true,
                is_current: m.id == config.model && m.provider == config.provider,
            });
        } else {
            *unauthed_by_provider.entry(m.provider.clone()).or_default() += 1;
        }
    }

    let setup_providers: Vec<ProviderSetupItem> = unauthed_by_provider
        .into_iter()
        .map(|(provider, count)| ProviderSetupItem {
            provider,
            model_count: count,
        })
        .collect();

    ModelPickerData {
        ready_models,
        setup_providers,
    }
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
    pub api_key: String,
    pub model_id: String,
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
        api_key,
        model_id,
        provider_for_status,
        success_message,
        mock,
    } = params;

    let model_context_limit = model_config.resolve_model_limits().0;

    let mut current_session = session_watch_rx.borrow().clone();
    if let Err(e) = current_session.ensure_persisted() {
        let text = format!("[Error] Failed to create session file: {e}\n");
        send_stream(ui_tx, talos_conversation::MessageSource::Error, text);
        return false;
    }
    let history = current_session.read_messages().unwrap_or_default();

    let session_config = SessionConfig {
        print_mode: false,
        workspace_root: workspace_root.to_path_buf(),
        initial_history: history,
        model_context_limit,
    };

    let provider = crate::provider_setup::build_provider(model_config, &api_key, mock);
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
    let mut registry =
        build_tui_tool_registry(approval_handler.clone(), workspace_root.to_path_buf());
    register_tui_permission_aware_tools(&mut registry, mcp_runtime.tools(), approval_handler);

    let mut agent = Agent::with_security_and_hooks(
        provider,
        registry,
        Some(Arc::new(talos_permission::PermissionEngine::new())),
        None,
        workspace_root.to_path_buf(),
        hooks.clone(),
    );
    agent.set_tool_protocol(model_config.tool_protocol());
    if let Ok(skills) = discover_runtime_skills(workspace_root) {
        apply_runtime_skills(&mut agent, &skills);
    }
    match crate::mode_runners::context_files_for_agent(model_config, workspace_root, true) {
        Ok(files) => agent.set_context_files(files),
        Err(e) => {
            let text = format!("[Error] Failed to load context files: {e}\n");
            send_stream(ui_tx, talos_conversation::MessageSource::Error, text);
            return false;
        }
    }

    let (handle, actor) = AppServerSession::new(agent, session_config);
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
            let _ = ui_tx.send(talos_conversation::UiOutput::Status(
                talos_conversation::StatusSnapshot {
                    model_name: model_id.clone(),
                    provider: provider_for_status,
                    ..Default::default()
                },
            ));
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

fn send_stream(
    ui_tx: &mpsc::UnboundedSender<talos_conversation::UiOutput>,
    source: talos_conversation::MessageSource,
    text: String,
) {
    use futures::stream;
    use talos_conversation::{StreamMessage, UiOutput};

    let _ = ui_tx.send(UiOutput::Stream(StreamMessage {
        source,
        stream: Box::pin(stream::once(async move { text })),
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_config::ProviderConfig;

    #[test]
    fn ready_models_have_correct_provider_and_context_limit() {
        let mut config = Config::default();
        config.model = "claude-sonnet-4-5-20250929".to_string();
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
    fn duplicate_model_ids_get_provider_qualified_values() {
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
            "zhipu".to_string(),
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

        if glm_entries.len() > 1 {
            for entry in &glm_entries {
                assert!(
                    entry.model_id.contains('/'),
                    "Duplicate model 'glm-5.2' should have provider-qualified value, got: {}",
                    entry.model_id
                );
            }
        }
    }

    #[test]
    fn unauthenticated_providers_appear_in_setup_providers() {
        let mut config = Config::default();
        config.model = "claude-sonnet-4-5-20250929".to_string();
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

        let openai_setup: Vec<_> = data
            .setup_providers
            .iter()
            .filter(|p| p.provider == "openai")
            .collect();
        assert!(
            !openai_setup.is_empty(),
            "Expected openai in setup_providers when unauthenticated"
        );
        assert!(
            openai_setup[0].model_count > 0,
            "openai setup_providers entry should have model_count > 0"
        );

        let anthropic_setup: Vec<_> = data
            .setup_providers
            .iter()
            .filter(|p| p.provider == "anthropic")
            .collect();
        assert!(
            anthropic_setup.is_empty(),
            "anthropic should not be in setup_providers when authenticated"
        );
    }

    #[test]
    fn is_current_flags_active_model_and_provider() {
        let mut config = Config::default();
        config.model = "claude-sonnet-4-5-20250929".to_string();
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
            current_models[0].model_id, "claude-sonnet-4-5-20250929",
            "Current model should be claude-sonnet-4-5-20250929"
        );
        assert_eq!(
            current_models[0].provider, "anthropic",
            "Current model provider should be anthropic"
        );

        for m in &data.ready_models {
            if m.model_id != "claude-sonnet-4-5-20250929" || m.provider != "anthropic" {
                assert!(
                    !m.is_current,
                    "Model {} ({}) should not be current",
                    m.model_id, m.provider
                );
            }
        }
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
        config.model = "claude-sonnet-4-5-20250929".to_string();
        config.provider = "anthropic".to_string();

        let target = provider_setup_target_model(&config, "openai").expect("target model");

        assert!(!target.is_empty());
        let provider = config
            .all_models()
            .into_iter()
            .find(|m| m.id == target || format!("{}/{}", m.provider, m.id) == target)
            .map(|m| m.provider)
            .expect("target exists in catalog");
        assert_eq!(provider, "openai");
    }
}
