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

use super::*;

#[test]
fn parse_dashboard_board_section_extracts_items() {
    let board = "# Board

## Now

| Item | State | Owner Doc | Gate |
|---|---|---|---|
| T57 Tool sweep | Active | [x](x.md) | Tests |

## Blocked / Paused

| Item | State | Owner Doc | Gate |
|---|---|---|---|
| T58 Dashboard review | Blocked | [x](x.md) | Security |

## Next

| Item | State | Owner Doc | Gate |
|---|---|---|---|
| T61 Rehearsal | Planned | [x](x.md) | Evidence |
";

    assert_eq!(
        crate::dashboard_helpers::parse_dashboard_board_section(board, "Blocked / Paused"),
        vec![("T58 Dashboard review".to_string(), "Blocked".to_string())]
    );
    assert_eq!(
        crate::dashboard_helpers::parse_dashboard_board_section(board, "Next"),
        vec![("T61 Rehearsal".to_string(), "Planned".to_string())]
    );
}

#[test]
fn dashboard_notifications_are_transient_and_never_include_tokens() {
    let loopback = dashboard_available_tip("http://127.0.0.1:61205/", true);
    assert!(matches!(
        loopback,
        UiOutput::Tip {
            kind: TipKind::Info,
            ref text
        } if text == "Dashboard ready: http://127.0.0.1:61205/ (loopback-only)"
    ));

    let restricted = dashboard_available_tip("http://127.0.0.1:61205/", false);
    assert!(matches!(
        restricted,
        UiOutput::Tip {
            kind: TipKind::Info,
            ref text
        } if text.contains("token required, see terminal output") && !text.contains("secret-token")
    ));

    assert!(matches!(
        dashboard_failure_tip("address in use"),
        UiOutput::Tip {
            kind: TipKind::Error,
            ref text
        } if text == "Dashboard failed to start: address in use"
    ));
}

#[tokio::test]
async fn handle_register_custom_provider_openai_chat_succeeds() {
    let _lock = HOME_ENV_MUTEX.lock().unwrap();
    let new_config = with_isolated_home(|| async {
        let (tx, mut rx) = mpsc::unbounded_channel::<UiOutput>();
        let config = Config::default();
        let result = handle_register_custom_provider(
            &tx,
            &config,
            "my-gateway",
            "openai-chat",
            "https://api.example.com/v1",
            "gw-secret-key",
        )
        .await;
        drop(tx);
        while rx.recv().await.is_some() {}
        result
    })
    .await
    .expect("registration must succeed");

    let provider = new_config
        .providers
        .get("my-gateway")
        .expect("my-gateway entry created");
    assert_eq!(provider.api_key.as_deref(), Some("gw-secret-key"));
    assert_eq!(provider.api_key_env.as_deref(), Some("MY-GATEWAY_API_KEY"));
    assert_eq!(
        provider.base_url.as_deref(),
        Some("https://api.example.com/v1")
    );
}

#[tokio::test]
async fn handle_register_custom_provider_anthropic_messages_succeeds() {
    let _lock = HOME_ENV_MUTEX.lock().unwrap();
    let new_config = with_isolated_home(|| async {
        let (tx, mut rx) = mpsc::unbounded_channel::<UiOutput>();
        let config = Config::default();
        let result = handle_register_custom_provider(
            &tx,
            &config,
            "anthropic-gw",
            "anthropic-messages",
            "https://api.example.com/anthropic/v1",
            "sk-ant-secret",
        )
        .await;
        drop(tx);
        while rx.recv().await.is_some() {}
        result
    })
    .await
    .expect("registration must succeed");

    let provider = new_config
        .providers
        .get("anthropic-gw")
        .expect("anthropic-gw entry created");
    assert_eq!(provider.api_key.as_deref(), Some("sk-ant-secret"));
    assert_eq!(
        provider.base_url.as_deref(),
        Some("https://api.example.com/anthropic/v1/messages")
    );
}

#[tokio::test]
async fn handle_register_custom_provider_invalid_name_no_partial_write() {
    let _lock = HOME_ENV_MUTEX.lock().unwrap();
    let result = with_isolated_home(|| async {
        let (tx, _rx) = mpsc::unbounded_channel::<UiOutput>();
        let config = Config::default();
        handle_register_custom_provider(
            &tx,
            &config,
            "Invalid_Name",
            "openai-chat",
            "https://api.example.com/v1",
            "key",
        )
        .await
    })
    .await;
    assert!(result.is_none(), "invalid name must return None");
}

#[tokio::test]
async fn handle_register_custom_provider_invalid_protocol_no_partial_write() {
    let _lock = HOME_ENV_MUTEX.lock().unwrap();
    let result = with_isolated_home(|| async {
        let (tx, _rx) = mpsc::unbounded_channel::<UiOutput>();
        let config = Config::default();
        handle_register_custom_provider(
            &tx,
            &config,
            "valid-name",
            "custom-protocol",
            "https://api.example.com/v1",
            "key",
        )
        .await
    })
    .await;
    assert!(result.is_none(), "invalid protocol must return None");
}

#[tokio::test]
async fn handle_register_custom_provider_non_https_url_no_partial_write() {
    let _lock = HOME_ENV_MUTEX.lock().unwrap();
    let result = with_isolated_home(|| async {
        let (tx, _rx) = mpsc::unbounded_channel::<UiOutput>();
        let config = Config::default();
        handle_register_custom_provider(
            &tx,
            &config,
            "valid-name",
            "openai-chat",
            "ftp://bad.example.com",
            "key",
        )
        .await
    })
    .await;
    assert!(result.is_none(), "non-HTTPS URL must return None");
}

#[tokio::test]
async fn handle_register_custom_provider_empty_key_no_partial_write() {
    let _lock = HOME_ENV_MUTEX.lock().unwrap();
    let result = with_isolated_home(|| async {
        let (tx, _rx) = mpsc::unbounded_channel::<UiOutput>();
        let config = Config::default();
        handle_register_custom_provider(
            &tx,
            &config,
            "valid-name",
            "openai-chat",
            "https://api.example.com/v1",
            "   ",
        )
        .await
    })
    .await;
    assert!(result.is_none(), "empty key must return None");
}

#[tokio::test]
async fn handle_register_custom_provider_update_preserves_unrelated_providers() {
    let _lock = HOME_ENV_MUTEX.lock().unwrap();
    let new_config = with_isolated_home(|| async {
        let (tx, mut rx) = mpsc::unbounded_channel::<UiOutput>();
        let mut config = Config::default();
        config.providers.insert(
            "existing-gw".to_string(),
            ProviderConfig {
                api_key: Some("old-key".to_string()),
                base_url: Some("https://old.example.com/v1".to_string()),
                ..Default::default()
            },
        );
        config.providers.insert(
            "other-provider".to_string(),
            ProviderConfig {
                api_key: Some("other-key".to_string()),
                ..Default::default()
            },
        );
        let result = handle_register_custom_provider(
            &tx,
            &config,
            "existing-gw",
            "openai-chat",
            "https://new.example.com/v1",
            "new-key",
        )
        .await;
        drop(tx);
        while rx.recv().await.is_some() {}
        result
    })
    .await
    .expect("update must succeed");

    let updated = new_config
        .providers
        .get("existing-gw")
        .expect("existing-gw still present");
    assert_eq!(updated.api_key.as_deref(), Some("new-key"));
    assert_eq!(
        updated.base_url.as_deref(),
        Some("https://new.example.com/v1")
    );

    let other = new_config
        .providers
        .get("other-provider")
        .expect("other-provider preserved");
    assert_eq!(other.api_key.as_deref(), Some("other-key"));
}

#[tokio::test]
async fn handle_register_custom_provider_loopback_http_allowed() {
    let _lock = HOME_ENV_MUTEX.lock().unwrap();
    let new_config = with_isolated_home(|| async {
        let (tx, mut rx) = mpsc::unbounded_channel::<UiOutput>();
        let config = Config::default();
        let result = handle_register_custom_provider(
            &tx,
            &config,
            "local-gw",
            "openai-chat",
            "http://127.0.0.1:8080/v1",
            "local-key",
        )
        .await;
        drop(tx);
        while rx.recv().await.is_some() {}
        result
    })
    .await
    .expect("loopback HTTP must succeed");

    let provider = new_config
        .providers
        .get("local-gw")
        .expect("local-gw entry created");
    assert_eq!(
        provider.base_url.as_deref(),
        Some("http://127.0.0.1:8080/v1")
    );
}
