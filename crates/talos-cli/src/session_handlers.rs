//! Session and provider handler functions.

use super::*;

/// Maximum discovered-model entries to persist into the provider's
/// `models` map during a single registration. Caps config growth when
/// a provider returns hundreds of model IDs.
pub(crate) const MAX_DISCOVERED_MODELS_TO_PERSIST: usize = 32;

/// Publish the UI boundary between two successfully committed sessions.
///
/// The queue belongs to the retired conversation engine, so its preview must be
/// cleared before the new session identity becomes visible. Keeping this small
/// ordered helper shared by `/new`, `/resume`, and `/fork` makes the boundary
/// independently testable without constructing a full session runtime.
pub(crate) fn emit_session_identity_after_queue_clear(
    ui_tx: &mpsc::UnboundedSender<UiOutput>,
    session_id: String,
) {
    let _ = ui_tx.send(UiOutput::SteeringQueueSnapshot(
        talos_conversation::SteeringQueueSnapshot {
            entries: vec![],
            total_count: 0,
            omitted_count: 0,
        },
    ));
    let _ = ui_tx.send(UiOutput::SessionIdentity { id: session_id });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn committed_session_boundary_clears_preview_before_identity() {
        let (ui_tx, mut ui_rx) = tokio::sync::mpsc::unbounded_channel();
        emit_session_identity_after_queue_clear(&ui_tx, "new-session".to_string());

        assert!(matches!(
            ui_rx.try_recv().expect("queue-clear output"),
            UiOutput::SteeringQueueSnapshot(talos_conversation::SteeringQueueSnapshot {
                entries,
                total_count: 0,
                omitted_count: 0,
            }) if entries.is_empty()
        ));
        assert!(matches!(
            ui_rx.try_recv().expect("session identity output"),
            UiOutput::SessionIdentity { id } if id == "new-session"
        ));
        assert!(
            ui_rx.try_recv().is_err(),
            "boundary helper emits only two outputs"
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle_session_delete(
    ui_tx: &mpsc::UnboundedSender<UiOutput>,
    workspace_root: &std::path::Path,
    session_manager: &talos_session::SessionManager,
    session_watch_rx: &watch::Receiver<talos_session::Session>,
    selection: Option<String>,
) {
    let workspace_root_str = canonical_workspace_root(workspace_root);
    let active_id = session_watch_rx.borrow().id;

    match &selection {
        None => {
            let mut sessions = match session_manager.list_workspace_sessions(&workspace_root_str) {
                Ok(s) => s,
                Err(e) => {
                    let text = format!("[Error] Failed to list sessions: {e}\n");
                    send_stream(ui_tx, MessageSource::Error, text);
                    return;
                }
            };
            if sessions.is_empty() {
                let text = "[System] No sessions found for this workspace.\n".to_string();
                send_stream(ui_tx, MessageSource::System, text);
                return;
            }
            sessions.retain(|s| s.id != active_id);
            if sessions.is_empty() {
                let text = "[System] No other sessions in this workspace to delete. The active session cannot be deleted.\n".to_string();
                send_stream(ui_tx, MessageSource::System, text);
                return;
            }
            sessions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp).then_with(|| a.id.cmp(&b.id)));

            let items: Vec<SessionPickerItem> = sessions
                .iter()
                .enumerate()
                .map(|(i, s)| SessionPickerItem {
                    command: "/delete".to_string(),
                    ordinal: i + 1,
                    timestamp: s.timestamp.to_string(),
                    message_count: s.message_count,
                    preview: if s.last_message_preview.is_empty() {
                        "(empty)".to_string()
                    } else {
                        s.last_message_preview.clone()
                    },
                })
                .collect();

            let _ = ui_tx.send(UiOutput::SessionPicker(items));
        }
        Some(arg) => {
            let mut sessions = match session_manager.list_workspace_sessions(&workspace_root_str) {
                Ok(s) => s,
                Err(e) => {
                    let text = format!("[Error] Failed to list sessions: {e}\n");
                    send_stream(ui_tx, MessageSource::Error, text);
                    return;
                }
            };
            sessions.retain(|s| s.id != active_id);
            sessions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp).then_with(|| a.id.cmp(&b.id)));

            let target = match arg.parse::<usize>() {
                Ok(n) if n >= 1 && n <= sessions.len() => &sessions[n - 1],
                _ => {
                    let text = format!(
                        "[Error] Invalid selection '{arg}'. Use /delete to pick a session.\n"
                    );
                    send_stream(ui_tx, MessageSource::Error, text);
                    return;
                }
            };

            let target_id = target.id;
            match session_manager.delete_session(&target_id) {
                Ok(()) => {
                    let text = format!("[System] Deleted session {target_id}.\n");
                    send_stream(ui_tx, MessageSource::System, text);
                }
                Err(e) => {
                    let text = format!("[Error] Failed to delete session {target_id}: {e}\n");
                    send_stream(ui_tx, MessageSource::Error, text);
                }
            }
        }
    }
}

pub(crate) async fn handle_provider_setup(
    ui_tx: &mpsc::UnboundedSender<UiOutput>,
    config: &Config,
    provider: &str,
) {
    if config.provider_authenticated(provider) {
        let data = build_model_picker_data(config);
        let _ = ui_tx.send(UiOutput::ModelPicker(data));
        return;
    }

    let _ = ui_tx.send(UiOutput::CredentialRequest(
        talos_conversation::CredentialRequestData {
            provider: provider.to_string(),
            model_id: None,
            connect_mode: false,
            default_base_url: None,
        },
    ));
}

pub(crate) async fn handle_connect(
    ui_tx: &mpsc::UnboundedSender<UiOutput>,
    config: &Config,
    provider: &str,
) {
    if provider.is_empty() {
        let data = build_connect_picker_data(config);
        let _ = ui_tx.send(UiOutput::ConnectPicker(data));
        return;
    }

    if config.provider_authenticated(provider) {
        send_stream(
            ui_tx,
            MessageSource::System,
            format!("[System] Provider '{provider}' is already connected.\n"),
        );
        return;
    }

    // Precedence: existing user config base_url > models.toml provider default >
    // builtin hardcoded config > None.
    let default_base_url = config
        .providers
        .get(provider)
        .and_then(|p| p.base_url.clone())
        .or_else(|| {
            talos_config::model::builtin_providers()
                .iter()
                .find(|p| p.id == provider)
                .and_then(|p| {
                    let base_url = p.api_base_url.as_deref()?;
                    Some(match p.protocol {
                        Some(talos_config::ProviderProtocol::AnthropicMessages) => {
                            let mut url = base_url.trim().trim_end_matches('/').to_string();
                            if !url.to_ascii_lowercase().ends_with("/messages") {
                                url.push_str("/messages");
                            }
                            url
                        }
                        _ => talos_config::normalize_provider_endpoint(base_url).base_url,
                    })
                })
        })
        .or_else(|| talos_config::builtin_provider_config(provider).and_then(|p| p.base_url));

    let _ = ui_tx.send(UiOutput::CredentialRequest(
        talos_conversation::CredentialRequestData {
            provider: provider.to_string(),
            model_id: None,
            connect_mode: true,
            default_base_url,
        },
    ));
}

pub(crate) async fn handle_register_custom_provider(
    ui_tx: &mpsc::UnboundedSender<UiOutput>,
    config: &Config,
    name: &str,
    protocol: &str,
    base_url: &str,
    api_key: &str,
) -> Option<Config> {
    if let Err(e) = talos_config::validate_provider_name(name) {
        send_stream(ui_tx, MessageSource::Error, format!("[Error] {e}\n"));
        return None;
    }
    let typed_protocol = match talos_config::validate_provider_protocol(protocol) {
        Ok(p) => p,
        Err(e) => {
            send_stream(ui_tx, MessageSource::Error, format!("[Error] {e}\n"));
            return None;
        }
    };
    let endpoint = match talos_config::validate_provider_base_url(base_url) {
        Ok(e) => e,
        Err(e) => {
            send_stream(ui_tx, MessageSource::Error, format!("[Error] {e}\n"));
            return None;
        }
    };
    if api_key.trim().is_empty() {
        send_stream(
            ui_tx,
            MessageSource::Error,
            "[Error] API key cannot be empty.\n".to_string(),
        );
        return None;
    }

    let is_update = config.providers.contains_key(name);
    if is_update {
        send_stream(
            ui_tx,
            MessageSource::System,
            format!(
                "[System] Updating existing provider '{name}'. Unrelated providers and models are preserved.\n"
            ),
        );
    }

    let discovery_base_url = endpoint.base_url.clone();
    let discovery_protocol = typed_protocol.clone();

    let mut new_config = config.clone();
    let provider_entry = new_config.providers.entry(name.to_string()).or_default();
    provider_entry.protocol = typed_protocol;
    provider_entry.base_url = Some(endpoint.base_url);
    provider_entry.api_key = Some(api_key.to_string());
    if provider_entry.api_key_env.is_none() {
        provider_entry.api_key_env = Some(format!("{}_API_KEY", name.to_uppercase()));
    }

    // Run discovery BEFORE persisting so we can atomically write provider
    // + discovered models in a single save. If discovery fails we still
    // save the provider entry alone (R9: provider registration must not
    // be coupled to discovery success, but discovery results must be
    // persisted atomically with the provider when both succeed).
    let discovery_outcome = crate::provider_discovery::discover_provider_models(
        &discovery_base_url,
        api_key,
        discovery_protocol,
    )
    .await;

    let mut discovered_count = 0usize;
    match &discovery_outcome {
        Ok(models) if !models.is_empty() => {
            for model_id in models.iter().take(MAX_DISCOVERED_MODELS_TO_PERSIST) {
                provider_entry.models.entry(model_id.clone()).or_default();
            }
            discovered_count = models.len();
        }
        _ => {}
    }

    if let Err(e) = new_config.save() {
        send_stream(
            ui_tx,
            MessageSource::Error,
            format!("[Error] Failed to save provider config: {e}\n"),
        );
        return None;
    }

    send_stream(
        ui_tx,
        MessageSource::System,
        format!(
            "[System] Custom provider '{name}' {}.\n",
            if is_update { "updated" } else { "registered" }
        ),
    );

    match discovery_outcome {
        Ok(models) if !models.is_empty() => {
            let preview: Vec<String> = models
                .iter()
                .take(10)
                .map(|m| format!("  - {name}/{m}"))
                .collect();
            let preview_text = preview.join("\n");
            let extra = if models.len() > 10 {
                format!("\n[System] ... and {} more.", models.len() - 10)
            } else {
                String::new()
            };
            let persisted = discovered_count.min(MAX_DISCOVERED_MODELS_TO_PERSIST);
            send_stream(
                ui_tx,
                MessageSource::System,
                format!(
                    "[System] Discovered {discovered_count} model(s) from '{name}'. The first {persisted} were saved to ~/.talos/config.toml so they appear in the /model picker.\n[System] Preview:\n{preview_text}{extra}\n[System] Run /model and select {name}/<model-id> to activate it. The provider+model are applied atomically when you pick.\n",
                ),
            );
        }
        Ok(_) => {
            send_stream(
                ui_tx,
                MessageSource::System,
                format!(
                    "[System] Provider '{name}' returned an empty model list. You can manually add a model in ~/.talos/config.toml under [providers.{name}.models.<model_id>].\n"
                ),
            );
        }
        Err(e) => {
            send_stream(
                ui_tx,
                MessageSource::System,
                format!(
                    "[System] Model discovery from '{name}' failed: {e}. You can manually add a model in ~/.talos/config.toml under [providers.{name}.models.<model_id>].\n"
                ),
            );
        }
    }

    Some(new_config)
}

pub(crate) async fn handle_connect_with_credential(
    ui_tx: &mpsc::UnboundedSender<UiOutput>,
    config: &Config,
    cred: talos_conversation::CredentialResponseData,
) -> Option<Config> {
    let mut new_config = config.clone();
    new_config.set_provider_credential(&cred.provider, &cred.api_key);

    let provider_entry = new_config
        .providers
        .entry(cred.provider.clone())
        .or_default();
    if provider_entry.api_key_env.is_none() {
        provider_entry.api_key_env = match cred.provider.as_str() {
            "anthropic" => Some("ANTHROPIC_API_KEY".to_string()),
            "openai" => Some("OPENAI_API_KEY".to_string()),
            _ => Some(format!("{}_API_KEY", cred.provider.to_uppercase())),
        };
    }
    // `cred.base_url` is already resolved by the TUI credential panel to
    // either the user-typed value or the request's `default_base_url`.
    // `None` here means neither was available, so the existing (or absent)
    // `base_url` is left untouched — never overwritten with an empty value.
    if let Some(base_url) = cred.base_url.as_ref() {
        let endpoint = talos_config::normalize_provider_endpoint(base_url);
        provider_entry.protocol = endpoint.protocol;
        provider_entry.base_url = Some(endpoint.base_url);
    }

    if let Err(e) = new_config.save() {
        send_stream(
            ui_tx,
            MessageSource::Error,
            format!("[Error] Failed to save provider config: {e}\n"),
        );
        return None;
    }

    send_stream(
        ui_tx,
        MessageSource::System,
        format!(
            "[System] Provider '{}' connected. Use /model to browse its models.\n",
            cred.provider
        ),
    );
    Some(new_config)
}

/// Builds [`talos_conversation::ConnectPickerData`] for the `/connect` picker.
///
/// Uses the compiled-in `models.toml` data (`[[providers]]` for display name,
/// API base URL, docs URL; `[[models]]` for model counts per provider).
pub(crate) fn build_connect_picker_data(config: &Config) -> talos_conversation::ConnectPickerData {
    use std::collections::BTreeMap;
    use talos_conversation::{ConnectPickerData, ConnectPickerItem};

    let all = talos_config::model::builtin_models();
    let mut model_counts: BTreeMap<String, usize> = BTreeMap::new();
    for m in &all {
        *model_counts.entry(m.provider.clone()).or_default() += 1;
    }

    let providers: BTreeMap<String, talos_config::model::BuiltinProvider> =
        talos_config::model::builtin_providers()
            .into_iter()
            .map(|p| (p.id.clone(), p))
            .collect();

    let mut connected = Vec::new();
    let mut available = Vec::new();

    for (provider_id, count) in model_counts {
        let has_credential = config.provider_authenticated(&provider_id);
        let (name, api_base_url, doc_url) = providers
            .get(&provider_id)
            .map(|p| (p.name.clone(), p.api_base_url.clone(), p.doc_url.clone()))
            .unwrap_or_else(|| (provider_id.clone(), None, None));
        let item = ConnectPickerItem {
            provider: provider_id.clone(),
            name,
            model_count: count,
            api_base_url,
            has_credential,
            doc_url,
        };
        if has_credential {
            connected.push(item);
        } else {
            available.push(item);
        }
    }

    ConnectPickerData {
        connected,
        available,
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle_session_model(
    transition: &Arc<Mutex<SessionTransition>>,
    ui_tx: &mpsc::UnboundedSender<UiOutput>,
    config: &Config,
    hooks: &Arc<HookRegistry>,
    workspace_root: &std::path::Path,
    mcp_config: &talos_config::McpConfig,
    session_watch_tx: &watch::Sender<talos_session::Session>,
    sq_tx_watch_tx: &watch::Sender<mpsc::Sender<SessionOp>>,
    bridge_rx_update_tx: &mpsc::UnboundedSender<(
        talos_session::Session,
        mpsc::UnboundedReceiver<SessionEvent>,
    )>,
    session_watch_rx: &watch::Receiver<talos_session::Session>,
    session_manager: &talos_session::SessionManager,
    model_id: String,
    provider_hint: Option<String>,
    mock: bool,
) -> Option<Config> {
    if model_id.is_empty() {
        let data = build_model_picker_data(config);
        let _ = ui_tx.send(UiOutput::ModelPicker(data));
        return None;
    }

    let (parsed_model_id, variant) = if let Some(idx) = model_id.rfind('@') {
        (
            model_id[..idx].to_string(),
            Some(model_id[idx + 1..].to_string()),
        )
    } else {
        (model_id.clone(), None)
    };

    // P1-fix: when the caller supplies an explicit provider (e.g. from
    // the /model picker's UserInput::SwitchModel), use the
    // provider-qualified form so Config::set_active_model resolves
    // unambiguously even when two providers share a model_id.
    let resolve_id = match &provider_hint {
        Some(p) if !p.is_empty() => format!("{p}/{parsed_model_id}"),
        _ => parsed_model_id.clone(),
    };

    let previous_model = config.model.clone();
    let previous_provider = config.provider.clone();
    let mut model_config = config.clone();
    if let Err(e) = model_config.set_active_model(&resolve_id) {
        let text = format!("[Error] Unknown model '{parsed_model_id}': {e}\n");
        send_stream(ui_tx, MessageSource::Error, text);
        return None;
    }

    if crate::model_lifecycle::apply_variant_change(&mut model_config, variant.as_deref())
        && let Err(e) = model_config.save()
    {
        tracing::warn!("Failed to persist model variant: {e}");
    }

    let provider_name = model_config.provider.clone();

    if config.model == parsed_model_id
        && config.provider == provider_name
        && config.variant == variant
    {
        return None;
    }

    if !model_config.provider_authenticated(&provider_name) {
        let _ = ui_tx.send(UiOutput::CredentialRequest(
            talos_conversation::CredentialRequestData {
                provider: provider_name,
                model_id: Some(model_id.clone()),
                connect_mode: false,
                default_base_url: None,
            },
        ));
        return None;
    }

    let api_key = match model_config.api_key() {
        Ok(k) => k,
        Err(e) => {
            let text = format!("[Error] Failed to resolve API key for {provider_name}: {e}\n");
            send_stream(ui_tx, MessageSource::Error, text);
            return None;
        }
    };

    if rebuild_session_for_model(RebuildSessionParams {
        transition,
        ui_tx,
        model_config: &model_config,
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
        model_id: parsed_model_id.clone(),
        variant: model_config.variant.clone(),
        provider_for_status: provider_name.clone(),
        success_message: format!("[System] Switched to model {parsed_model_id}.\n"),
        mock,
    })
    .await
    {
        if let Err(e) = model_config.save() {
            let text = format!("[Error] Model switched, but failed to persist config: {e}\n");
            send_stream(ui_tx, MessageSource::Error, text);
        }
        Some(model_config)
    } else {
        None
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle_session_model_with_credential(
    transition: &Arc<Mutex<SessionTransition>>,
    ui_tx: &mpsc::UnboundedSender<UiOutput>,
    config: &Config,
    hooks: &Arc<HookRegistry>,
    workspace_root: &std::path::Path,
    mcp_config: &talos_config::McpConfig,
    session_watch_tx: &watch::Sender<talos_session::Session>,
    sq_tx_watch_tx: &watch::Sender<mpsc::Sender<SessionOp>>,
    bridge_rx_update_tx: &mpsc::UnboundedSender<(
        talos_session::Session,
        mpsc::UnboundedReceiver<SessionEvent>,
    )>,
    session_watch_rx: &watch::Receiver<talos_session::Session>,
    session_manager: &talos_session::SessionManager,
    cred: talos_conversation::CredentialResponseData,
    mock: bool,
) -> Option<Config> {
    let previous_model = config.model.clone();
    let previous_provider = config.provider.clone();
    let mut model_config = config.clone();
    model_config.set_provider_credential(&cred.provider, &cred.api_key);
    if let Err(e) = model_config.save() {
        let text = format!("[Error] Failed to persist credentials: {e}\n");
        send_stream(ui_tx, MessageSource::Error, text);
        return None;
    }

    let model_id = match &cred.model_id {
        Some(id) => id.clone(),
        None => match provider_setup_target_model(&model_config, &cred.provider) {
            Some(id) => id,
            None => {
                let text = format!(
                    "[Error] Credentials saved, but no models are configured for provider '{}'.\n",
                    cred.provider
                );
                send_stream(ui_tx, MessageSource::Error, text);
                return None;
            }
        },
    };

    let (parsed_model_id, variant) = if let Some(idx) = model_id.rfind('@') {
        (
            model_id[..idx].to_string(),
            Some(model_id[idx + 1..].to_string()),
        )
    } else {
        (model_id.clone(), None)
    };

    if let Err(e) = model_config.set_active_model(&parsed_model_id) {
        let text = format!("[Error] Unknown model '{parsed_model_id}': {e}\n");
        send_stream(ui_tx, MessageSource::Error, text);
        return None;
    }

    if crate::model_lifecycle::apply_variant_change(&mut model_config, variant.as_deref())
        && let Err(e) = model_config.save()
    {
        tracing::warn!("Failed to persist model variant: {e}");
    }

    let api_key = cred.api_key.clone();
    let provider_for_status = model_config.provider.clone();

    if rebuild_session_for_model(RebuildSessionParams {
        transition,
        ui_tx,
        model_config: &model_config,
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
        model_id: parsed_model_id.clone(),
        variant: model_config.variant.clone(),
        provider_for_status,
        success_message: format!(
            "[System] Credentials saved. Switched to model {parsed_model_id}.\n"
        ),
        mock,
    })
    .await
    {
        if let Err(e) = model_config.save() {
            let text = format!("[Error] Model switched, but failed to persist config: {e}\n");
            send_stream(ui_tx, MessageSource::Error, text);
        }
        Some(model_config)
    } else {
        None
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle_session_new(
    transition: &Arc<Mutex<SessionTransition>>,
    ui_tx: &mpsc::UnboundedSender<UiOutput>,
    config: &Config,
    api_key: &str,
    hooks: &Arc<HookRegistry>,
    workspace_root: &std::path::Path,
    session_manager: &talos_session::SessionManager,
    mcp_config: &talos_config::McpConfig,
    session_watch_tx: &watch::Sender<talos_session::Session>,
    sq_tx_watch_tx: &watch::Sender<mpsc::Sender<SessionOp>>,
    bridge_rx_update_tx: &mpsc::UnboundedSender<(
        talos_session::Session,
        mpsc::UnboundedReceiver<SessionEvent>,
    )>,
    model_context_limit: u32,
    mock: bool,
) {
    let mut transition = transition.lock().await;

    let session_manager = session_manager.clone();
    let workspace_root_str = canonical_workspace_root(workspace_root);
    let new_session = match session_manager.defer_create_session("talos", &workspace_root_str) {
        Ok(s) => s,
        Err(e) => {
            let text = format!("[Error] Failed to create new session: {e}\n");
            send_stream(ui_tx, MessageSource::Error, text);
            return;
        }
    };

    let new_history: Vec<Message> = vec![];
    let session_config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: workspace_root.to_path_buf(),
        initial_history: new_history,
        model_context_limit,
    };

    let provider = build_provider(config, api_key, mock);
    let approval_handler = Arc::new(TuiApprovalHandler::new(
        ui_tx.clone(),
        workspace_root.to_path_buf(),
    ));
    let mcp_runtime = match McpSessionRuntime::start(mcp_config, hooks.clone()).await {
        Ok(r) => r,
        Err(e) => {
            let text = format!("[Error] Failed to start MCP runtime: {e}\n");
            send_stream(ui_tx, MessageSource::Error, text);
            return;
        }
    };
    mcp_runtime.report_startup_failures();
    let (sched_tools, sched_pending) = talos_agent::create_scheduler_tools();
    let mut registry = build_tui_tool_registry(
        approval_handler.clone(),
        workspace_root.to_path_buf(),
        new_session.id,
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
    agent.set_tool_protocol(config.tool_protocol());
    if let Ok(skills) = discover_runtime_skills(workspace_root, config.skills.discover_shared) {
        apply_runtime_skills(&mut agent, &skills);
    }
    maybe_set_memory_provider(&mut agent, config);
    set_todo_prompt_provider(&mut agent, &session_manager, &new_session);

    let (handle, mut actor) = AppServerSession::new(agent, session_config);
    let _sched_join = sched_pending.spawn(
        handle.sq_tx.clone(),
        tokio_util::sync::CancellationToken::new(),
    );
    actor.set_persistence(
        new_session.clone(),
        crate::mode_runtime::session_metadata_for_model(&config.model, &config.provider),
    );

    // Clone for watch channel update after commit (new_session is moved into prepare).
    let new_session_for_watch = new_session.clone();
    if let Err(e) = transition.prepare(handle, new_session) {
        let _ = std::fs::remove_file(&new_session_for_watch.file_path);
        let text = format!("[Error] Failed to prepare new session: {e}\n");
        send_stream(ui_tx, MessageSource::Error, text);
        return;
    }

    match transition.commit(actor) {
        Ok(result) => {
            let _ = session_watch_tx.send(new_session_for_watch.clone());
            let _ = sq_tx_watch_tx.send(result.new_handle.sq_tx.clone());
            if bridge_rx_update_tx
                .send((new_session_for_watch.clone(), result.new_handle.eq_rx))
                .is_err()
            {
                eprintln!(
                    "[Error] Bridge forwarder unavailable; new session events will not be persisted or displayed."
                );
            }
            emit_session_identity_after_queue_clear(ui_tx, new_session_for_watch.id.to_string());
            let text = "[System] New session started. Previous session preserved.\n".to_string();
            send_stream(ui_tx, MessageSource::System, text);
        }
        Err(e) => {
            transition.rollback();
            let _ = std::fs::remove_file(&new_session_for_watch.file_path);
            let text =
                format!("[Error] Failed to commit new session: {e}. Old session remains active.\n");
            send_stream(ui_tx, MessageSource::Error, text);
        }
    }
}

/// Handle `/resume` — list candidates or resume a specific session.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle_session_resume(
    transition: &Arc<Mutex<SessionTransition>>,
    ui_tx: &mpsc::UnboundedSender<UiOutput>,
    config: &Config,
    api_key: &str,
    hooks: &Arc<HookRegistry>,
    workspace_root: &std::path::Path,
    session_manager: &talos_session::SessionManager,
    mcp_config: &talos_config::McpConfig,
    session_watch_tx: &watch::Sender<talos_session::Session>,
    sq_tx_watch_tx: &watch::Sender<mpsc::Sender<SessionOp>>,
    bridge_rx_update_tx: &mpsc::UnboundedSender<(
        talos_session::Session,
        mpsc::UnboundedReceiver<SessionEvent>,
    )>,
    _model_context_limit: u32,
    session_id: Option<String>,
    mock: bool,
) -> Option<Config> {
    let mut transition = transition.lock().await;

    let workspace_root_str = canonical_workspace_root(workspace_root);

    let target_session = match &session_id {
        Some(id) => {
            // Try parsing as ordinal (1-based) first, then fall back to UUID.
            if let Ok(n) = id.parse::<usize>() {
                let sessions = match session_manager.list_workspace_sessions(&workspace_root_str) {
                    Ok(s) => s,
                    Err(e) => {
                        let text = format!("[Error] Failed to list sessions: {e}\n");
                        send_stream(ui_tx, MessageSource::Error, text);
                        return None;
                    }
                };
                if sessions.is_empty() {
                    let text = "[System] No sessions found for this workspace.\n".to_string();
                    send_stream(ui_tx, MessageSource::System, text);
                    return None;
                }
                let mut sessions = sessions;
                sessions
                    .sort_by(|a, b| b.timestamp.cmp(&a.timestamp).then_with(|| a.id.cmp(&b.id)));
                if n == 0 || n > sessions.len() {
                    let text = format!(
                        "[Error] Invalid session number {n}. Valid range: 1-{}.\n",
                        sessions.len()
                    );
                    send_stream(ui_tx, MessageSource::Error, text);
                    return None;
                }
                let selected = &sessions[n - 1];
                let selected_id = selected.id.to_string();
                match session_manager.resume_session(&selected_id) {
                    Ok(s) => s,
                    Err(e) => {
                        let text = format!("[Error] Session '{id}' not found or invalid: {e}\n");
                        send_stream(ui_tx, MessageSource::Error, text);
                        return None;
                    }
                }
            } else {
                // Fall back to treating it as a UUID (backward compat).
                match session_manager.resume_session(id) {
                    Ok(s) => s,
                    Err(e) => {
                        let text = format!("[Error] Session '{id}' not found or invalid: {e}\n");
                        send_stream(ui_tx, MessageSource::Error, text);
                        return None;
                    }
                }
            }
        }
        None => {
            let sessions = match session_manager.list_workspace_sessions(&workspace_root_str) {
                Ok(s) => s,
                Err(e) => {
                    let text = format!("[Error] Failed to list sessions: {e}\n");
                    send_stream(ui_tx, MessageSource::Error, text);
                    return None;
                }
            };

            if sessions.is_empty() {
                let text = "[System] No sessions found for this workspace.\n".to_string();
                send_stream(ui_tx, MessageSource::System, text);
                return None;
            }

            let mut sessions = sessions;
            sessions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp).then_with(|| a.id.cmp(&b.id)));

            let items: Vec<SessionPickerItem> = sessions
                .iter()
                .enumerate()
                .map(|(i, s)| SessionPickerItem {
                    command: "/resume".to_string(),
                    ordinal: i + 1,
                    timestamp: s.timestamp.to_string(),
                    message_count: s.message_count,
                    preview: if s.last_message_preview.is_empty() {
                        "(empty)".to_string()
                    } else {
                        s.last_message_preview.clone()
                    },
                })
                .collect();

            let _ = ui_tx.send(UiOutput::SessionPicker(items));
            return None;
        }
    };

    let mut resume_config = config.clone();
    apply_session_model_to_config(&mut resume_config, &target_session);
    let resume_api_key = match resume_config.api_key() {
        Ok(key) => key,
        Err(e) if mock => {
            tracing::warn!("failed to resolve resumed session api key in mock mode: {e}");
            api_key.to_string()
        }
        Err(e) => {
            let text = format!(
                "[Error] Failed to resolve API key for resumed session model '{}': {e}\n",
                resume_config.model
            );
            send_stream(ui_tx, MessageSource::Error, text);
            return None;
        }
    };
    let resume_model_context_limit = resume_config.resolve_model_limits().0;

    let resume_history = match target_session.read_messages() {
        Ok(h) => h,
        Err(e) => {
            let text = format!("[Error] Failed to read session history: {e}\n");
            send_stream(ui_tx, MessageSource::Error, text);
            return None;
        }
    };

    let resume_history_for_hydrate = resume_history.clone();
    let session_config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: workspace_root.to_path_buf(),
        initial_history: resume_history,
        model_context_limit: resume_model_context_limit,
    };

    let provider = build_provider(&resume_config, &resume_api_key, mock);
    let approval_handler = Arc::new(TuiApprovalHandler::new(
        ui_tx.clone(),
        workspace_root.to_path_buf(),
    ));
    let mcp_runtime = match McpSessionRuntime::start(mcp_config, hooks.clone()).await {
        Ok(r) => r,
        Err(e) => {
            let text = format!("[Error] Failed to start MCP runtime: {e}\n");
            send_stream(ui_tx, MessageSource::Error, text);
            return None;
        }
    };
    mcp_runtime.report_startup_failures();
    let (sched_tools, sched_pending) = talos_agent::create_scheduler_tools();
    let mut registry = build_tui_tool_registry(
        approval_handler.clone(),
        workspace_root.to_path_buf(),
        target_session.id,
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
    agent.set_tool_protocol(resume_config.tool_protocol());
    if let Ok(skills) =
        discover_runtime_skills(workspace_root, resume_config.skills.discover_shared)
    {
        apply_runtime_skills(&mut agent, &skills);
    }
    maybe_set_memory_provider(&mut agent, &resume_config);
    set_todo_prompt_provider(&mut agent, session_manager, &target_session);
    match context_files_for_agent(&resume_config, workspace_root, true) {
        Ok(files) => agent.set_context_files(files),
        Err(e) => {
            let text = format!("[Error] Failed to load context files: {e}\n");
            send_stream(ui_tx, MessageSource::Error, text);
            return None;
        }
    }

    let (handle, mut actor) = AppServerSession::new(agent, session_config);
    let _sched_join = sched_pending.spawn(
        handle.sq_tx.clone(),
        tokio_util::sync::CancellationToken::new(),
    );
    actor.set_persistence(
        target_session.clone(),
        crate::mode_runtime::session_metadata_for_model(
            &resume_config.model,
            &resume_config.provider,
        ),
    );

    // Clone for watch channel update after commit (target_session is moved into prepare).
    let target_session_for_watch = target_session.clone();
    if let Err(e) = transition.prepare(handle, target_session) {
        let _ = std::fs::remove_file(&target_session_for_watch.file_path);
        let text = format!("[Error] Failed to prepare resume: {e}\n");
        send_stream(ui_tx, MessageSource::Error, text);
        return None;
    }

    match transition.commit(actor) {
        Ok(result) => {
            let _ = session_watch_tx.send(target_session_for_watch.clone());
            let _ = sq_tx_watch_tx.send(result.new_handle.sq_tx.clone());
            if bridge_rx_update_tx
                .send((target_session_for_watch.clone(), result.new_handle.eq_rx))
                .is_err()
            {
                eprintln!(
                    "[Error] Bridge forwarder unavailable; resumed session events will not be persisted or displayed."
                );
            }
            let _ = ui_tx.send(UiOutput::HydrateHistory(resume_history_for_hydrate));
            emit_session_identity_after_queue_clear(ui_tx, target_session_for_watch.id.to_string());
            let text = format!(
                "[System] Resumed session {}.\n",
                target_session_for_watch.id
            );
            send_stream(ui_tx, MessageSource::System, text);
            Some(resume_config)
        }
        Err(e) => {
            transition.rollback();
            let _ = std::fs::remove_file(&target_session_for_watch.file_path);
            let text =
                format!("[Error] Failed to commit resume: {e}. Old session remains active.\n");
            send_stream(ui_tx, MessageSource::Error, text);
            None
        }
    }
}

/// Handle `/fork` — clone the active session's durable history into a child session.
///
/// Copies the source JSONL file to a new path with a fresh UUID, creates a new
/// [`talos_session::Session`], and swaps the agent context. The source session
/// remains byte-for-byte unchanged.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle_session_fork(
    transition: &Arc<Mutex<SessionTransition>>,
    ui_tx: &mpsc::UnboundedSender<UiOutput>,
    config: &Config,
    api_key: &str,
    hooks: &Arc<HookRegistry>,
    workspace_root: &std::path::Path,
    session_manager: &talos_session::SessionManager,
    mcp_config: &talos_config::McpConfig,
    session_watch_tx: &watch::Sender<talos_session::Session>,
    sq_tx_watch_tx: &watch::Sender<mpsc::Sender<SessionOp>>,
    bridge_rx_update_tx: &mpsc::UnboundedSender<(
        talos_session::Session,
        mpsc::UnboundedReceiver<SessionEvent>,
    )>,
    model_context_limit: u32,
    session_watch_rx: &watch::Receiver<talos_session::Session>,
    mock: bool,
) {
    let mut transition = transition.lock().await;

    let source_session = session_watch_rx.borrow().clone();

    let source_bytes = match source_session.snapshot_bytes() {
        Ok(b) => b,
        Err(e) => {
            let text = format!("[Error] Failed to read source session file: {e}\n");
            send_stream(ui_tx, MessageSource::Error, text);
            return;
        }
    };

    let fork_history = match source_session.read_messages() {
        Ok(h) => h,
        Err(e) => {
            let text = format!("[Error] Failed to read source session history: {e}\n");
            send_stream(ui_tx, MessageSource::Error, text);
            return;
        }
    };

    let workspace_root_str = canonical_workspace_root(workspace_root);
    let child_session = match session_manager.defer_create_session("talos", &workspace_root_str) {
        Ok(s) => s,
        Err(e) => {
            let text = format!("[Error] Failed to create child session: {e}\n");
            send_stream(ui_tx, MessageSource::Error, text);
            return;
        }
    };
    let child_id = child_session.id;

    let child_path = child_session.file_path.clone();
    if let Some(parent) = child_path.parent()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        let text = format!("[Error] Failed to create child session directory: {e}\n");
        send_stream(ui_tx, MessageSource::Error, text);
        return;
    }

    if let Err(e) = std::fs::write(&child_path, &source_bytes) {
        let text = format!("[Error] Failed to clone session history: {e}\n");
        send_stream(ui_tx, MessageSource::Error, text);
        return;
    }

    let session_config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: workspace_root.to_path_buf(),
        initial_history: fork_history,
        model_context_limit,
    };

    let provider = build_provider(config, api_key, mock);
    let approval_handler = Arc::new(TuiApprovalHandler::new(
        ui_tx.clone(),
        workspace_root.to_path_buf(),
    ));
    let mcp_runtime = match McpSessionRuntime::start(mcp_config, hooks.clone()).await {
        Ok(r) => r,
        Err(e) => {
            let text = format!("[Error] Failed to start MCP runtime: {e}\n");
            send_stream(ui_tx, MessageSource::Error, text);
            return;
        }
    };
    mcp_runtime.report_startup_failures();
    let (sched_tools, sched_pending) = talos_agent::create_scheduler_tools();
    let mut registry = build_tui_tool_registry(
        approval_handler.clone(),
        workspace_root.to_path_buf(),
        child_session.id,
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
    agent.set_tool_protocol(config.tool_protocol());
    if let Ok(skills) = discover_runtime_skills(workspace_root, config.skills.discover_shared) {
        apply_runtime_skills(&mut agent, &skills);
    }
    maybe_set_memory_provider(&mut agent, config);
    set_todo_prompt_provider(&mut agent, session_manager, &child_session);

    let (handle, mut actor) = AppServerSession::new(agent, session_config);
    let _sched_join = sched_pending.spawn(
        handle.sq_tx.clone(),
        tokio_util::sync::CancellationToken::new(),
    );
    actor.set_persistence(
        child_session.clone(),
        crate::mode_runtime::session_metadata_for_model(&config.model, &config.provider),
    );

    // Clone for watch channel update after commit (child_session is moved into prepare).
    let child_session_for_watch = child_session.clone();
    if let Err(e) = transition.prepare(handle, child_session) {
        let _ = std::fs::remove_file(&child_session_for_watch.file_path);
        let text = format!("[Error] Failed to prepare fork: {e}\n");
        send_stream(ui_tx, MessageSource::Error, text);
        return;
    }

    match transition.commit(actor) {
        Ok(result) => {
            let _ = session_watch_tx.send(child_session_for_watch.clone());
            let _ = sq_tx_watch_tx.send(result.new_handle.sq_tx.clone());
            if bridge_rx_update_tx
                .send((child_session_for_watch.clone(), result.new_handle.eq_rx))
                .is_err()
            {
                eprintln!(
                    "[Error] Bridge forwarder unavailable; forked session events will not be persisted or displayed."
                );
            }
            emit_session_identity_after_queue_clear(ui_tx, child_session_for_watch.id.to_string());
            let text = format!(
                "[System] Forked session {child_id} (source: {}).\n",
                result.old_session.id
            );
            send_stream(ui_tx, MessageSource::System, text);
        }
        Err(e) => {
            transition.rollback();
            let _ = std::fs::remove_file(&child_session_for_watch.file_path);
            let text = format!("[Error] Failed to commit fork: {e}. Old session remains active.\n");
            send_stream(ui_tx, MessageSource::Error, text);
        }
    }
}
