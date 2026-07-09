//! Provider and connection handlers for the Talos TUI.
//!
//! Extracted from `mode_runners.rs` to reduce file size and improve
//! maintainability. All functions are behavior-preserving.

use std::collections::BTreeMap;

use talos_config::Config;
use talos_conversation::{
    ConnectPickerData, ConnectPickerItem, CredentialRequestData, CredentialResponseData,
    MessageSource, StreamMessage, UiOutput,
};
use tokio::sync::mpsc;

use crate::model_lifecycle::build_model_picker_data;
use crate::session_handlers::send_stream;


#[allow(clippy::too_many_arguments)]
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


pub(crate) async fn handle_connect(ui_tx: &mpsc::UnboundedSender<UiOutput>, config: &Config, provider: &str) {
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

#[cfg(test)]
mod connect_tests {
    use super::*;
    use crate::test_support::HOME_ENV_MUTEX;
    use talos_config::ProviderConfig;

    /// Runs `f` with `HOME` redirected to a fresh temp directory, restoring
    /// the original `HOME` afterward. Must be called under `HOME_ENV_MUTEX`
    /// (shared crate-wide — see `crate::test_support` for why a private
    /// per-module mutex is not sufficient).
    async fn with_isolated_home<F, Fut, T>(f: F) -> T
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let dir = tempfile::tempdir().expect("tempdir");
        let original = std::env::var("HOME").ok();
        unsafe { std::env::set_var("HOME", dir.path()) };
        let result = f().await;
        match original {
            Some(value) => unsafe { std::env::set_var("HOME", value) },
            None => unsafe { std::env::remove_var("HOME") },
        }
        result
    }

    #[test]
    fn build_connect_picker_data_none_falls_back_without_blocking() {
        let config = Config::default();
        let data = build_connect_picker_data(&config);
        assert!(data.connected.len() + data.available.len() > 0);
    }

    #[tokio::test]
    async fn handle_connect_with_credential_writes_new_provider_api_key_and_base_url() {
        let _lock = HOME_ENV_MUTEX.lock().unwrap();
        let new_config = with_isolated_home(|| async {
            let (tx, mut rx) = mpsc::unbounded_channel::<UiOutput>();
            let config = Config::default();
            let cred = talos_conversation::CredentialResponseData {
                provider: "groq".to_string(),
                api_key: "gsk-secret".to_string(),
                model_id: None,
                connect_mode: true,
                base_url: Some("https://api.groq.com/openai/v1".to_string()),
            };

            let result = handle_connect_with_credential(&tx, &config, cred).await;
            drop(tx);
            while rx.recv().await.is_some() {}
            result
        })
        .await
        .expect("new provider connect must succeed");

        let groq = new_config
            .providers
            .get("groq")
            .expect("groq entry created");
        assert_eq!(groq.api_key.as_deref(), Some("gsk-secret"));
        assert_eq!(groq.api_key_env.as_deref(), Some("GROQ_API_KEY"));
        assert_eq!(
            groq.base_url.as_deref(),
            Some("https://api.groq.com/openai/v1")
        );
    }

    #[tokio::test]
    async fn handle_connect_with_credential_preserves_unrelated_provider_fields() {
        let _lock = HOME_ENV_MUTEX.lock().unwrap();
        let new_config = with_isolated_home(|| async {
            let (tx, mut rx) = mpsc::unbounded_channel::<UiOutput>();
            let mut config = Config::default();
            config.providers.insert(
                "groq".to_string(),
                ProviderConfig {
                    base_url: Some("https://old.groq.example/v1".to_string()),
                    models: std::collections::HashMap::from([(
                        "llama-3".to_string(),
                        talos_config::ModelConfig {
                            context_limit: Some(999_000),
                            output_limit: None,
                            reasoning: None,
                        },
                    )]),
                    ..Default::default()
                },
            );
            // An unrelated provider that must remain completely untouched.
            config.providers.insert(
                "anthropic".to_string(),
                ProviderConfig {
                    api_key: Some("sk-ant-untouched".to_string()),
                    ..Default::default()
                },
            );

            let cred = talos_conversation::CredentialResponseData {
                provider: "groq".to_string(),
                api_key: "gsk-updated".to_string(),
                model_id: None,
                connect_mode: true,
                base_url: None, // user left it blank; resolved default was also None
            };

            let result = handle_connect_with_credential(&tx, &config, cred).await;
            drop(tx);
            while rx.recv().await.is_some() {}
            result
        })
        .await
        .expect("existing provider reconnect must succeed");

        let groq = new_config.providers.get("groq").expect("groq entry exists");
        assert_eq!(groq.api_key.as_deref(), Some("gsk-updated"));
        assert_eq!(
            groq.base_url.as_deref(),
            Some("https://old.groq.example/v1"),
            "existing base_url must be preserved when cred.base_url is None"
        );
        assert_eq!(
            groq.models.get("llama-3").and_then(|m| m.context_limit),
            Some(999_000),
            "existing model overrides must be preserved"
        );

        let anthropic = new_config
            .providers
            .get("anthropic")
            .expect("anthropic entry exists");
        assert_eq!(
            anthropic.api_key.as_deref(),
            Some("sk-ant-untouched"),
            "unrelated provider must not be touched by a groq connect"
        );
    }

    #[tokio::test]
    async fn handle_connect_with_credential_updates_base_url_when_provided() {
        let _lock = HOME_ENV_MUTEX.lock().unwrap();
        let new_config = with_isolated_home(|| async {
            let (tx, mut rx) = mpsc::unbounded_channel::<UiOutput>();
            let mut config = Config::default();
            config.providers.insert(
                "groq".to_string(),
                ProviderConfig {
                    base_url: Some("https://old.groq.example/v1".to_string()),
                    ..Default::default()
                },
            );

            let cred = talos_conversation::CredentialResponseData {
                provider: "groq".to_string(),
                api_key: "gsk-updated".to_string(),
                model_id: None,
                connect_mode: true,
                base_url: Some("https://new.groq.example/v1".to_string()),
            };

            let result = handle_connect_with_credential(&tx, &config, cred).await;
            drop(tx);
            while rx.recv().await.is_some() {}
            result
        })
        .await
        .expect("update must succeed");

        let groq = new_config.providers.get("groq").unwrap();
        assert_eq!(
            groq.base_url.as_deref(),
            Some("https://new.groq.example/v1"),
            "explicit user input must update the base_url"
        );
    }

    #[tokio::test]
    async fn handle_connect_default_base_url_falls_back_to_builtin_provider_config() {
        let (tx, mut rx) = mpsc::unbounded_channel::<UiOutput>();
        let config = Config::default();

        // groq has an api_base_url in the models.toml [[providers]] section
        // (from BUILD_MODELS=1) AND a hardcoded base_url in builtin_provider_config.
        // Both return "https://api.groq.com/openai/v1".
        handle_connect(&tx, &config, "groq").await;
        drop(tx);

        let mut default_base_url = None;
        while let Some(output) = rx.recv().await {
            if let UiOutput::CredentialRequest(req) = output {
                default_base_url = req.default_base_url;
            }
        }

        assert_eq!(
            default_base_url.as_deref(),
            Some("https://api.groq.com/openai/v1")
        );
    }

    #[tokio::test]
    async fn handle_connect_minimax_coding_plan_uses_anthropic_messages_endpoint() {
        let (tx, mut rx) = mpsc::unbounded_channel::<UiOutput>();
        let config = Config::default();

        handle_connect(&tx, &config, "minimax-coding-plan").await;
        drop(tx);

        let mut default_base_url = None;
        while let Some(output) = rx.recv().await {
            if let UiOutput::CredentialRequest(req) = output {
                default_base_url = req.default_base_url;
            }
        }

        assert_eq!(
            default_base_url.as_deref(),
            Some("https://api.minimax.io/anthropic/v1/messages")
        );
    }

    #[tokio::test]
    async fn handle_connect_with_credential_sets_anthropic_protocol_for_minimax_endpoint() {
        let _lock = HOME_ENV_MUTEX.lock().unwrap();
        let new_config = with_isolated_home(|| async {
            let (tx, mut rx) = mpsc::unbounded_channel::<UiOutput>();
            let config = Config::default();
            let cred = talos_conversation::CredentialResponseData {
                provider: "minimax-coding-plan".to_string(),
                api_key: "minimax-secret".to_string(),
                model_id: None,
                connect_mode: true,
                base_url: Some("https://api.minimax.io/anthropic/v1/messages".to_string()),
            };

            let result = handle_connect_with_credential(&tx, &config, cred).await;
            drop(tx);
            while rx.recv().await.is_some() {}
            result
        })
        .await
        .expect("minimax coding plan connect must succeed");

        let minimax = new_config
            .providers
            .get("minimax-coding-plan")
            .expect("minimax coding plan entry created");
        assert_eq!(
            minimax.protocol,
            talos_config::ProviderProtocol::AnthropicMessages
        );
        assert_eq!(
            minimax.base_url.as_deref(),
            Some("https://api.minimax.io/anthropic/v1/messages")
        );
    }

    #[tokio::test]
    async fn handle_connect_already_authenticated_does_not_request_credential() {
        let (tx, mut rx) = mpsc::unbounded_channel::<UiOutput>();
        let mut config = Config::default();
        config.providers.insert(
            "groq".to_string(),
            ProviderConfig {
                api_key: Some("gsk-existing".to_string()),
                ..Default::default()
            },
        );

        handle_connect(&tx, &config, "groq").await;
        drop(tx);

        let mut saw_credential_request = false;
        while let Some(output) = rx.recv().await {
            if matches!(output, UiOutput::CredentialRequest(_)) {
                saw_credential_request = true;
            }
        }
        assert!(
            !saw_credential_request,
            "already-connected provider must not prompt for credentials again"
        );
    }
}
