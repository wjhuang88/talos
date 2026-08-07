//! Provider connection and model activation workflows.

use super::super::*;
use talos_config::ConfigStore;

pub(super) fn provider_qualified_model_reference(provider: &str, model_id: &str) -> String {
    if model_id.starts_with(&format!("{provider}/")) {
        model_id.to_string()
    } else {
        format!("{provider}/{model_id}")
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
    let mut discovered_models = Vec::new();
    match &discovery_outcome {
        Ok(models) if !models.is_empty() => {
            discovered_count = models.len();
            discovered_models.extend(
                models
                    .iter()
                    .take(MAX_DISCOVERED_MODELS_TO_PERSIST)
                    .cloned(),
            );
        }
        _ => {}
    }

    let provider_name = name.to_string();
    let provider_protocol = typed_protocol;
    let provider_base_url = endpoint.base_url;
    let provider_api_key = api_key.to_string();
    let new_config = match ConfigStore::default_store().update_config(|current| {
        let provider_entry = current.providers.entry(provider_name.clone()).or_default();
        provider_entry.protocol = provider_protocol;
        provider_entry.base_url = Some(provider_base_url);
        provider_entry.api_key = Some(provider_api_key);
        if provider_entry.api_key_env.is_none() {
            provider_entry.api_key_env = Some(format!("{}_API_KEY", provider_name.to_uppercase()));
        }
        for model_id in discovered_models {
            provider_entry.models.entry(model_id).or_default();
        }
        Ok(())
    }) {
        Ok(config) => config,
        Err(e) => {
            send_stream(
                ui_tx,
                MessageSource::Error,
                format!("[Error] Failed to save provider config: {e}\n"),
            );
            return None;
        }
    };

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
    _config: &Config,
    cred: talos_conversation::CredentialResponseData,
) -> Option<Config> {
    let provider = cred.provider.clone();
    let api_key = cred.api_key.clone();
    let base_url = cred.base_url.clone();
    let new_config = match ConfigStore::default_store().update_config(|current| {
        current.set_provider_credential(&provider, &api_key);

        let provider_entry = current.providers.entry(provider.clone()).or_default();
        if provider_entry.api_key_env.is_none() {
            provider_entry.api_key_env = match provider.as_str() {
                "anthropic" => Some("ANTHROPIC_API_KEY".to_string()),
                "openai" => Some("OPENAI_API_KEY".to_string()),
                _ => Some(format!("{}_API_KEY", provider.to_uppercase())),
            };
        }
        // `base_url` is already resolved by the TUI credential panel to
        // either the user-typed value or the request's `default_base_url`.
        // `None` means the existing value must remain untouched.
        if let Some(base_url) = base_url.as_ref() {
            let endpoint = talos_config::normalize_provider_endpoint(base_url);
            provider_entry.protocol = endpoint.protocol;
            provider_entry.base_url = Some(endpoint.base_url);
        }
        Ok(())
    }) {
        Ok(config) => config,
        Err(e) => {
            send_stream(
                ui_tx,
                MessageSource::Error,
                format!("[Error] Failed to save provider config: {e}\n"),
            );
            return None;
        }
    };

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

fn same_model_activation_identity(current: &Config, requested: &Config) -> bool {
    crate::mode_runtime::SessionModelIdentity::new(
        &current.provider,
        &current.model,
        current.variant.as_deref(),
    ) == crate::mode_runtime::SessionModelIdentity::new(
        &requested.provider,
        &requested.model,
        requested.variant.as_deref(),
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle_session_model(
    transition: &Arc<Mutex<SessionTransition>>,
    ui_tx: &mpsc::UnboundedSender<UiOutput>,
    config: &Config,
    runtime_builder: &TuiRuntimeBuilder,
    session_watch_tx: &watch::Sender<talos_session::Session>,
    sq_tx_watch_tx: &watch::Sender<mpsc::Sender<SessionOp>>,
    bridge_rx_update_tx: &mpsc::UnboundedSender<(
        talos_session::Session,
        mpsc::UnboundedReceiver<SessionEvent>,
    )>,
    session_watch_rx: &watch::Receiver<talos_session::Session>,
    model_id: String,
    provider_hint: Option<String>,
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
    let previous_variant = config.variant.clone();
    let mut model_config = config.clone();
    if let Err(e) = model_config.set_active_model(&resolve_id) {
        let text = format!("[Error] Unknown model '{parsed_model_id}': {e}\n");
        send_stream(ui_tx, MessageSource::Error, text);
        return None;
    }

    crate::model_lifecycle::apply_variant_change(&mut model_config, variant.as_deref());

    let provider_name = model_config.provider.clone();

    if same_model_activation_identity(config, &model_config) {
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

    let _resolved_api_key = match model_config.api_key() {
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
        runtime_builder,
        session_watch_tx,
        sq_tx_watch_tx,
        bridge_rx_update_tx,
        session_watch_rx,
        previous_model,
        previous_provider,
        previous_variant,
        model_id: parsed_model_id.clone(),
        variant: model_config.variant.clone(),
        provider_for_status: provider_name.clone(),
        success_message: format!("[System] Switched to model {parsed_model_id}.\n"),
    })
    .await
    {
        match ConfigStore::default_store().update_config(|current| {
            current.set_active_model(&resolve_id)?;
            crate::model_lifecycle::apply_variant_change(current, variant.as_deref());
            Ok(())
        }) {
            Ok(committed) => Some(committed),
            Err(e) => {
                let text = format!("[Error] Model switched, but failed to persist config: {e}\n");
                send_stream(ui_tx, MessageSource::Error, text);
                Some(model_config)
            }
        }
    } else {
        None
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle_session_model_with_credential(
    transition: &Arc<Mutex<SessionTransition>>,
    ui_tx: &mpsc::UnboundedSender<UiOutput>,
    config: &Config,
    runtime_builder: &TuiRuntimeBuilder,
    session_watch_tx: &watch::Sender<talos_session::Session>,
    sq_tx_watch_tx: &watch::Sender<mpsc::Sender<SessionOp>>,
    bridge_rx_update_tx: &mpsc::UnboundedSender<(
        talos_session::Session,
        mpsc::UnboundedReceiver<SessionEvent>,
    )>,
    session_watch_rx: &watch::Receiver<talos_session::Session>,
    cred: talos_conversation::CredentialResponseData,
) -> Option<Config> {
    let previous_model = config.model.clone();
    let previous_provider = config.provider.clone();
    let previous_variant = config.variant.clone();
    let credential_provider = cred.provider.clone();
    let credential_api_key = cred.api_key.clone();
    let mut model_config = match ConfigStore::default_store().update_config(|current| {
        current.set_provider_credential(&credential_provider, &credential_api_key);
        Ok(())
    }) {
        Ok(config) => config,
        Err(e) => {
            let text = format!("[Error] Failed to persist credentials: {e}\n");
            send_stream(ui_tx, MessageSource::Error, text);
            return None;
        }
    };

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

    let qualified_model_id = provider_qualified_model_reference(&cred.provider, &parsed_model_id);

    if let Err(e) = model_config.set_active_model(&qualified_model_id) {
        let text = format!("[Error] Unknown model '{parsed_model_id}': {e}\n");
        send_stream(ui_tx, MessageSource::Error, text);
        return None;
    }

    crate::model_lifecycle::apply_variant_change(&mut model_config, variant.as_deref());

    let provider_for_status = model_config.provider.clone();

    if rebuild_session_for_model(RebuildSessionParams {
        transition,
        ui_tx,
        model_config: &model_config,
        runtime_builder,
        session_watch_tx,
        sq_tx_watch_tx,
        bridge_rx_update_tx,
        session_watch_rx,
        previous_model,
        previous_provider,
        previous_variant,
        model_id: parsed_model_id.clone(),
        variant: model_config.variant.clone(),
        provider_for_status,
        success_message: format!(
            "[System] Credentials saved. Switched to model {parsed_model_id}.\n"
        ),
    })
    .await
    {
        match ConfigStore::default_store().update_config(|current| {
            current.set_provider_credential(&cred.provider, &cred.api_key);
            current.set_active_model(&qualified_model_id)?;
            crate::model_lifecycle::apply_variant_change(current, variant.as_deref());
            Ok(())
        }) {
            Ok(committed) => Some(committed),
            Err(e) => {
                let text = format!("[Error] Model switched, but failed to persist config: {e}\n");
                send_stream(ui_tx, MessageSource::Error, text);
                Some(model_config)
            }
        }
    } else {
        None
    }
}
