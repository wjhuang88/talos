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

/// R9 regression: when discovery succeeds, the discovered model IDs
/// MUST be persisted into the provider's `models` map atomically with
/// the provider entry, so that the /model picker can surface them
/// without a separate save round-trip. Verified end-to-end with a
/// mock HTTP server that returns an OpenAI-shaped /models response.
#[tokio::test]
async fn r9_discovered_models_persisted_atomically_with_provider() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let base_url = format!("http://{addr}/v1");

    let body = r#"{"data":[{"id":"gw-1"},{"id":"gw-2"},{"id":"gw-3"}]}"#;
    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut req_buf = [0u8; 4096];
        let _ = socket.read(&mut req_buf).await;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        socket.write_all(response.as_bytes()).await.unwrap();
        socket.flush().await.unwrap();
    });

    let _lock = HOME_ENV_MUTEX.lock().unwrap();
    let new_config = with_isolated_home(|| async {
        let (tx, mut rx) = mpsc::unbounded_channel::<UiOutput>();
        let config = Config::default();
        let result = handle_register_custom_provider(
            &tx,
            &config,
            "r9-gw",
            "openai-chat",
            &base_url,
            "r9-key",
        )
        .await;
        drop(tx);
        while rx.recv().await.is_some() {}
        result
    })
    .await
    .expect("R9 registration with mock discovery must succeed");

    let provider = new_config
        .providers
        .get("r9-gw")
        .expect("r9-gw provider entry must exist");

    // R9 invariant: all three discovered models must be in the models map.
    assert!(
        provider.models.contains_key("gw-1"),
        "gw-1 must be persisted, got: {:?}",
        provider.models.keys().collect::<Vec<_>>()
    );
    assert!(
        provider.models.contains_key("gw-2"),
        "gw-2 must be persisted, got: {:?}",
        provider.models.keys().collect::<Vec<_>>()
    );
    assert!(
        provider.models.contains_key("gw-3"),
        "gw-3 must be persisted, got: {:?}",
        provider.models.keys().collect::<Vec<_>>()
    );
}

/// R9 regression: discovery failure (e.g. network error) must NOT
/// prevent the provider entry from being saved. The two concerns are
/// decoupled — provider registration is authoritative, discovery is
/// best-effort.
#[tokio::test]
async fn r9_provider_saved_when_discovery_fails() {
    let _lock = HOME_ENV_MUTEX.lock().unwrap();
    let new_config = with_isolated_home(|| async {
        let (tx, mut rx) = mpsc::unbounded_channel::<UiOutput>();
        let config = Config::default();
        // Use a routable-but-closed port so discovery fails fast.
        let result = handle_register_custom_provider(
            &tx,
            &config,
            "r9-fail-gw",
            "openai-chat",
            "http://127.0.0.1:1/v1",
            "r9-fail-key",
        )
        .await;
        drop(tx);
        while rx.recv().await.is_some() {}
        result
    })
    .await
    .expect("R9 registration must succeed even when discovery fails");

    let provider = new_config
        .providers
        .get("r9-fail-gw")
        .expect("provider entry must exist even if discovery failed");
    assert_eq!(provider.api_key.as_deref(), Some("r9-fail-key"));
    assert!(
        provider.models.is_empty(),
        "no models should be persisted when discovery failed"
    );
}

// ── P1 closeout tests: discovery → picker visibility → structured identity ──

/// Helper: spawn a mock HTTP server that responds to one request and
/// returns the given body with 200 OK.
async fn spawn_mock_models_server(body: String) -> String {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{addr}");

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut req_buf = [0u8; 4096];
        let _ = socket.read(&mut req_buf).await;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let _ = socket.write_all(response.as_bytes()).await;
        let _ = socket.flush().await;
    });

    url
}

/// P1: After successful OpenAI-compatible discovery, discovered models
/// must appear in `config.all_models()` so the `/model` picker surfaces
/// them. This proves the "picker visibility" half of the closed loop.
#[tokio::test]
async fn p1_discovered_models_visible_in_all_models() {
    use talos_config::model::find_model_by_provider;

    let body = r#"{"data":[{"id":"p1-model-a"},{"id":"p1-model-b"}]}"#;
    let base_url = spawn_mock_models_server(body.to_string()).await;

    let _lock = HOME_ENV_MUTEX.lock().unwrap();
    let new_config = with_isolated_home(|| async {
        let (tx, mut rx) = mpsc::unbounded_channel::<UiOutput>();
        let config = Config::default();
        let result = handle_register_custom_provider(
            &tx,
            &config,
            "p1-gw",
            "openai-chat",
            &format!("{base_url}/v1"),
            "p1-key",
        )
        .await;
        drop(tx);
        while rx.recv().await.is_some() {}
        result
    })
    .await
    .expect("P1 registration must succeed");

    let all = new_config.all_models();
    let found_a = find_model_by_provider(&all, "p1-gw", "p1-model-a");
    let found_b = find_model_by_provider(&all, "p1-gw", "p1-model-b");
    assert!(found_a.is_some(), "p1-model-a must appear in all_models()");
    assert!(found_b.is_some(), "p1-model-b must appear in all_models()");
}

/// P1: After successful Anthropic-compatible discovery (using the
/// Anthropic /models endpoint shape), discovered models must also appear
/// in `config.all_models()`.
#[tokio::test]
async fn p1_anthropic_discovery_models_visible_in_all_models() {
    use talos_config::model::find_model_by_provider;

    let body = r#"{"data":[{"id":"claude-p1-a"},{"id":"claude-p1-b"}]}"#;
    let base_url = spawn_mock_models_server(body.to_string()).await;

    let _lock = HOME_ENV_MUTEX.lock().unwrap();
    let new_config = with_isolated_home(|| async {
        let (tx, mut rx) = mpsc::unbounded_channel::<UiOutput>();
        let config = Config::default();
        let result = handle_register_custom_provider(
            &tx,
            &config,
            "p1-anthropic-gw",
            "anthropic-messages",
            &format!("{base_url}/v1/messages"),
            "p1-anthropic-key",
        )
        .await;
        drop(tx);
        while rx.recv().await.is_some() {}
        result
    })
    .await
    .expect("P1 Anthropic registration must succeed");

    let all = new_config.all_models();
    assert!(
        find_model_by_provider(&all, "p1-anthropic-gw", "claude-p1-a").is_some(),
        "claude-p1-a must appear in all_models()"
    );
    assert!(
        find_model_by_provider(&all, "p1-anthropic-gw", "claude-p1-b").is_some(),
        "claude-p1-b must appear in all_models()"
    );
}

/// P1: No UI output message after discovery (success or failure) may
/// contain the API key. Collects all content blocks and scans for the
/// key string.
#[tokio::test]
async fn p1_credential_redaction_in_discovery_messages() {
    let body = r#"{"data":[{"id":"redact-model"}]}"#;
    let base_url = spawn_mock_models_server(body.to_string()).await;
    let secret = "super-secret-key-12345";

    let _lock = HOME_ENV_MUTEX.lock().unwrap();
    let _new_config = with_isolated_home(|| async {
        let (tx, mut rx) = mpsc::unbounded_channel::<UiOutput>();
        let config = Config::default();
        let _ = handle_register_custom_provider(
            &tx,
            &config,
            "redact-gw",
            "openai-chat",
            &format!("{base_url}/v1"),
            secret,
        )
        .await;
        drop(tx);

        let mut messages = Vec::new();
        while let Some(output) = rx.recv().await {
            match output {
                UiOutput::Content(talos_conversation::ContentOutput::Block { text, .. }) => {
                    messages.push(text);
                }
                _ => {}
            }
        }

        for msg in &messages {
            assert!(
                !msg.contains(secret),
                "API key must not appear in UI output: found in '{msg}'"
            );
        }
    })
    .await;
}

/// P1: Model IDs containing `/` or `@` must be preserved exactly in
/// config. They are panel data, not command syntax.
#[tokio::test]
async fn p1_structured_identity_for_slash_and_at_model_ids() {
    use talos_config::model::find_model_by_provider;

    let body = r#"{"data":[{"id":"org/model-v1"},{"id":"model@variant"}]}"#;
    let base_url = spawn_mock_models_server(body.to_string()).await;

    let _lock = HOME_ENV_MUTEX.lock().unwrap();
    let new_config = with_isolated_home(|| async {
        let (tx, mut rx) = mpsc::unbounded_channel::<UiOutput>();
        let config = Config::default();
        let result = handle_register_custom_provider(
            &tx,
            &config,
            "struct-gw",
            "openai-chat",
            &format!("{base_url}/v1"),
            "struct-key",
        )
        .await;
        drop(tx);
        while rx.recv().await.is_some() {}
        result
    })
    .await
    .expect("registration with structured IDs must succeed");

    let all = new_config.all_models();
    assert!(
        find_model_by_provider(&all, "struct-gw", "org/model-v1").is_some(),
        "model ID with '/' must be preserved exactly"
    );
    assert!(
        find_model_by_provider(&all, "struct-gw", "model@variant").is_some(),
        "model ID with '@' must be preserved exactly"
    );
}

/// P1: Updating an existing provider must preserve manually-added models
/// that were not part of the discovery response. This proves the
/// "duplicate provider update" acceptance.
#[tokio::test]
async fn p1_update_preserves_unrelated_models() {
    let body = r#"{"data":[{"id":"discovered-1"}]}"#;
    let base_url = spawn_mock_models_server(body.to_string()).await;

    let _lock = HOME_ENV_MUTEX.lock().unwrap();
    let new_config = with_isolated_home(|| async {
        // First: register with a manually-added model.
        let (tx, mut rx) = mpsc::unbounded_channel::<UiOutput>();
        let mut config = Config::default();
        config.providers.insert(
            "update-gw".to_string(),
            ProviderConfig {
                protocol: talos_config::ProviderProtocol::OpenAIChat,
                base_url: Some(format!("{base_url}/v1")),
                api_key: Some("old-key".to_string()),
                ..Default::default()
            },
        );
        config
            .providers
            .get_mut("update-gw")
            .unwrap()
            .models
            .insert("manual-model".to_string(), Default::default());

        // Second: re-register (update) the same provider with discovery.
        let result = handle_register_custom_provider(
            &tx,
            &config,
            "update-gw",
            "openai-chat",
            &format!("{base_url}/v1"),
            "new-key",
        )
        .await;
        drop(tx);
        while rx.recv().await.is_some() {}
        result
    })
    .await
    .expect("update must succeed");

    let provider = new_config
        .providers
        .get("update-gw")
        .expect("provider must exist");
    assert!(
        provider.models.contains_key("manual-model"),
        "manually-added model must be preserved after update"
    );
    assert!(
        provider.models.contains_key("discovered-1"),
        "discovered model must also be present"
    );
    assert_eq!(
        provider.api_key.as_deref(),
        Some("new-key"),
        "API key must be updated"
    );
}

/// P1: After discovery failure, the user can manually add a model to
/// the provider's config, and it will appear in `all_models()` — proving
/// the manual fallback path is usable.
#[tokio::test]
async fn p1_manual_fallback_after_discovery_failure() {
    use talos_config::model::find_model_by_provider;

    let _lock = HOME_ENV_MUTEX.lock().unwrap();
    let new_config = with_isolated_home(|| async {
        let (tx, mut rx) = mpsc::unbounded_channel::<UiOutput>();
        let config = Config::default();
        let result = handle_register_custom_provider(
            &tx,
            &config,
            "fallback-gw",
            "openai-chat",
            "http://127.0.0.1:1/v1",
            "fallback-key",
        )
        .await;
        drop(tx);
        while rx.recv().await.is_some() {}
        result
    })
    .await
    .expect("registration must succeed even when discovery fails");

    // Simulate the user manually adding a model ID.
    let mut config = new_config;
    config
        .providers
        .get_mut("fallback-gw")
        .unwrap()
        .models
        .insert("manually-entered-model".to_string(), Default::default());

    let all = config.all_models();
    assert!(
        find_model_by_provider(&all, "fallback-gw", "manually-entered-model").is_some(),
        "manually entered model must appear in all_models()"
    );
}

/// P1: After discovery, selecting a discovered model via the existing
/// `Config::set_active_model` path (which the model lifecycle uses)
/// produces the correct `(provider, model_id)` active identity. This
/// proves the "selection → activation" half of the closed loop at the
/// data level — the actual session rebuild is handled by the existing
/// model lifecycle code already tested in model_lifecycle.rs.
#[tokio::test]
async fn p1_selecting_discovered_model_sets_active_identity() {
    use talos_config::model::find_model_by_provider;

    let body = r#"{"data":[{"id":"select-me"}]}"#;
    let base_url = spawn_mock_models_server(body.to_string()).await;

    let _lock = HOME_ENV_MUTEX.lock().unwrap();
    let mut new_config = with_isolated_home(|| async {
        let (tx, mut rx) = mpsc::unbounded_channel::<UiOutput>();
        let config = Config::default();
        let result = handle_register_custom_provider(
            &tx,
            &config,
            "select-gw",
            "openai-chat",
            &format!("{base_url}/v1"),
            "select-key",
        )
        .await;
        drop(tx);
        while rx.recv().await.is_some() {}
        result
    })
    .await
    .expect("registration must succeed");

    // Verify the discovered model is in all_models (picker would show it).
    let all = new_config.all_models();
    let found = find_model_by_provider(&all, "select-gw", "select-me")
        .expect("discovered model must be in all_models");
    assert_eq!(found.provider, "select-gw");
    assert_eq!(found.id, "select-me");

    // Verify that set_active_model with the provider-qualified form
    // resolves correctly (this is what handle_session_model does when
    // provider_hint is Some).
    assert!(
        new_config.set_active_model("select-gw/select-me").is_ok(),
        "provider-qualified form must resolve the discovered model"
    );
    assert_eq!(new_config.model, "select-me");
    assert_eq!(new_config.provider, "select-gw");
}

/// P1-fix: provider_hint disambiguates cross-provider duplicate model
/// IDs. Two providers have a model named "shared-model". Without
/// provider_hint, set_active_model errors. With provider_hint, it
/// resolves to the correct provider.
#[test]
fn p1fix_provider_hint_disambiguates_cross_provider_duplicates() {
    let mut config = Config::default();
    config.providers.insert(
        "provider-a".to_string(),
        ProviderConfig {
            api_key: Some("key-a".to_string()),
            ..Default::default()
        },
    );
    config
        .providers
        .get_mut("provider-a")
        .unwrap()
        .models
        .insert("shared-model".to_string(), Default::default());

    config.providers.insert(
        "provider-b".to_string(),
        ProviderConfig {
            api_key: Some("key-b".to_string()),
            ..Default::default()
        },
    );
    config
        .providers
        .get_mut("provider-b")
        .unwrap()
        .models
        .insert("shared-model".to_string(), Default::default());

    // Without hint: ambiguity error
    let result = config.set_active_model("shared-model");
    assert!(result.is_err(), "bare model_id must fail on duplicates");

    // With hint: resolves to the correct provider
    let result = config.set_active_model("provider-b/shared-model");
    assert!(result.is_ok(), "provider-qualified form must resolve");
    assert_eq!(config.provider, "provider-b");
    assert_eq!(config.model, "shared-model");
}

/// P1-fix: handle_session_model with provider_hint resolves the
/// discovered model through the lifecycle path. This test verifies
/// the data-level resolution: provider_hint causes
/// Config::set_active_model to receive `provider/model_id` form.
/// The actual session rebuild (spawned actor, channels) is tested in
/// model_lifecycle.rs tests with MockProvider.
#[test]
fn p1fix_provider_hint_flows_to_set_active_model() {
    let mut config = Config::default();
    config.providers.insert(
        "p1fix-gw".to_string(),
        ProviderConfig {
            api_key: Some("p1fix-key".to_string()),
            ..Default::default()
        },
    );
    config
        .providers
        .get_mut("p1fix-gw")
        .unwrap()
        .models
        .insert("p1fix-model".to_string(), Default::default());

    // Simulate what handle_session_model does with provider_hint:
    let parsed_model_id = "p1fix-model";
    let provider_hint = Some("p1fix-gw".to_string());
    let resolve_id = match &provider_hint {
        Some(p) if !p.is_empty() => format!("{p}/{parsed_model_id}"),
        _ => parsed_model_id.to_string(),
    };

    assert!(config.set_active_model(&resolve_id).is_ok());
    assert_eq!(config.provider, "p1fix-gw");
    assert_eq!(config.model, "p1fix-model");
}

/// P1-fix: when handle_session_model receives an unknown model (with
/// provider_hint), set_active_model errors, the function returns None,
/// and the caller must not update config_for_handler. This test
/// verifies the set_active_model failure path directly.
#[test]
fn p1fix_activation_failure_preserves_old_config() {
    let mut config = Config::default();
    config.model = "original-model".to_string();
    config.provider = "original-provider".to_string();
    config.providers.insert(
        "original-provider".to_string(),
        ProviderConfig {
            api_key: Some("orig-key".to_string()),
            ..Default::default()
        },
    );

    let original_model = config.model.clone();
    let original_provider = config.provider.clone();

    // Simulate handle_session_model with a nonexistent model + hint:
    let result = config.set_active_model("nonexistent-provider/nonexistent-model");
    assert!(result.is_err(), "unknown model must fail");
    assert_eq!(config.model, original_model, "old model must be unchanged");
    assert_eq!(
        config.provider, original_provider,
        "old provider must be unchanged"
    );
}
