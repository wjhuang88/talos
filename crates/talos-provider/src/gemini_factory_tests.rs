//! Isolated-process acceptance of the public factory, lazy ADC and real Agent tool loop.
//! Endpoint redirection exists only in the unit-test binary, never in library builds.

use crate::gemini_auth::VertexTokenSource;
use serde_json::json;
use talos_core::provider::{ProviderError, ProviderResult};

const CHILD: &str = "TALOS_GEMINI_POSITIVE_FACTORY_CHILD";

pub(crate) fn redirect_transport(
    source: VertexTokenSource,
    endpoint: String,
) -> ProviderResult<(VertexTokenSource, String)> {
    if std::env::var_os(CHILD).is_none() {
        return Ok((source, endpoint));
    }
    assert_eq!(
        endpoint,
        "https://aiplatform.googleapis.com/v1/projects/fixture-project/locations/global/publishers/google/models/fixture-model:streamGenerateContent?alt=sse"
    );
    let local = |key| -> ProviderResult<String> {
        let text = std::env::var(key)
            .map_err(|_| ProviderError::InvalidResponse("missing factory test endpoint".into()))?;
        let url = reqwest::Url::parse(&text)
            .map_err(|_| ProviderError::InvalidResponse("invalid factory test endpoint".into()))?;
        if url.scheme() != "http"
            || url.host_str() != Some("127.0.0.1")
            || url.port().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
            || url.path() != "/"
        {
            return Err(ProviderError::InvalidResponse(
                "nonlocal factory fixture".into(),
            ));
        }
        Ok(text)
    };
    Ok((
        source.with_test_endpoint(local("TALOS_GEMINI_FACTORY_TOKEN")?),
        local("TALOS_GEMINI_FACTORY_GENERATE")?,
    ))
}

#[test]
fn lazy_factory_adc_completes_real_agent_round_for_user_and_service_account() {
    for mode in [
        "explicit-user",
        "well-known-user",
        "explicit-service-account",
    ] {
        let temp = tempfile::tempdir().expect("workspace");
        let root = temp.path().canonicalize().expect("canonical root");
        let path = if mode == "well-known-user" {
            let mut path = root.clone();
            if !cfg!(windows) {
                path.push(".config");
            }
            path.join("gcloud/application_default_credentials.json")
        } else {
            root.join("adc.json")
        };
        std::fs::create_dir_all(path.parent().expect("parent")).expect("ADC directory");
        std::fs::write(root.join("fixture.txt"), "LAZY_FACTORY_READ_OK").expect("read fixture");
        let mut token_server = mockito::Server::new();
        let mut generation_server = mockito::Server::new();
        let removed_path = path.clone();
        let service_account = mode.ends_with("service-account");
        let token = token_server
            .mock("POST", "/")
            .match_request(move |request| {
                let value: serde_json::Value =
                    serde_json::from_slice(request.body().expect("body")).expect("JSON");
                let valid = if service_account {
                    value["grant_type"] == "urn:ietf:params:oauth:grant-type:jwt-bearer"
                        && value["assertion"]
                            .as_str()
                            .is_some_and(|assertion| assertion.split('.').count() == 3)
                } else {
                    value["grant_type"] == "refresh_token"
                        && value["client_id"] == "fixture-client"
                        && value["refresh_token"] == "fixture-refresh"
                };
                if valid {
                    // Subsequent tool-turn dispatch must reuse initialized credentials/cache.
                    std::fs::remove_file(&removed_path).expect("remove initialized ADC");
                }
                valid
            })
            .with_body(
                r#"{"access_token":"fixture-access","token_type":"Bearer","expires_in":3600}"#,
            )
            .expect(1)
            .create();
        let signed_part = json!({"functionCall":{"name":"read","args":{"path":"fixture.txt"}},"thoughtSignature":"factory-private-signature"});
        let first = generation_server.mock("POST", "/")
            .match_header("authorization", "Bearer fixture-access")
            .match_header("x-goog-api-key", mockito::Matcher::Missing)
            .match_request(|request| !String::from_utf8_lossy(request.body().expect("body")).contains("functionResponse"))
            .with_body(format!("data: {}\n\n", json!({"candidates":[{"content":{"parts":[signed_part.clone()]},"finishReason":"STOP"}]})))
            .expect(1).create();
        let second = generation_server.mock("POST", "/")
            .match_header("authorization", "Bearer fixture-access")
            .match_header("x-goog-api-key", mockito::Matcher::Missing)
            .match_request(move |request| {
                let body: serde_json::Value = serde_json::from_slice(request.body().expect("body")).expect("JSON");
                let contents = body["contents"].as_array().expect("contents");
                contents.iter().any(|entry| entry["role"] == "model" && entry["parts"] == json!([signed_part.clone()]))
                    && contents.iter().flat_map(|entry| entry["parts"].as_array().expect("parts")).any(|part| {
                        let response = &part["functionResponse"];
                        response["name"] == "read" && response.get("id").is_none()
                            && response["response"]["output"].as_str().is_some_and(|text| text.contains("LAZY_FACTORY_READ_OK"))
                    })
            })
            .with_body(format!("data: {}\n\n", json!({"candidates":[{"content":{"parts":[{"text":"factory ok"}]},"finishReason":"STOP"}]})))
            .expect(1).create();
        let mut command =
            std::process::Command::new(std::env::current_exe().expect("test executable"));
        command
            .args([
                "--exact",
                "gemini_factory_tests::positive_factory_child",
                "--nocapture",
            ])
            .env(CHILD, mode)
            .env("TALOS_GEMINI_FACTORY_ROOT", &root)
            .env("TALOS_GEMINI_FACTORY_ADC_PATH", &path)
            .env("TALOS_GEMINI_FACTORY_TOKEN", token_server.url())
            .env("TALOS_GEMINI_FACTORY_GENERATE", generation_server.url())
            .env("GCE_METADATA_HOST", "127.0.0.1:9")
            .env_remove("GOOGLE_APPLICATION_CREDENTIALS")
            .env_remove("HOME")
            .env_remove("APPDATA");
        for name in [
            "HTTP_PROXY",
            "HTTPS_PROXY",
            "ALL_PROXY",
            "http_proxy",
            "https_proxy",
            "all_proxy",
        ] {
            command.env_remove(name);
        }
        command
            .env("NO_PROXY", "127.0.0.1")
            .env("no_proxy", "127.0.0.1");
        if mode == "well-known-user" {
            command.env(if cfg!(windows) { "APPDATA" } else { "HOME" }, &root);
        } else {
            command.env("GOOGLE_APPLICATION_CREDENTIALS", &path);
        }
        let output = command.output().expect("isolated factory child");
        assert!(
            output.status.success(),
            "factory {mode} failed: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
        first.assert();
        second.assert();
        token.assert();
        assert!(!path.exists());
    }
}

#[tokio::test]
async fn positive_factory_child() {
    let Ok(mode) = std::env::var(CHILD) else {
        return;
    };
    let root =
        std::path::PathBuf::from(std::env::var_os("TALOS_GEMINI_FACTORY_ROOT").expect("root"));
    let path =
        std::path::PathBuf::from(std::env::var_os("TALOS_GEMINI_FACTORY_ADC_PATH").expect("ADC"));
    let config = talos_config::Config {
        provider: "fixture-vertex".into(),
        model: "fixture-model".into(),
        providers: std::collections::HashMap::from([(
            "fixture-vertex".into(),
            talos_config::ProviderConfig {
                protocol: talos_config::ProviderProtocol::GoogleGemini,
                gemini: Some(talos_config::GeminiTransportConfig::Vertex {
                    project: "fixture-project".into(),
                    location: "global".into(),
                    allow_metadata: false,
                }),
                timeout: talos_config::ProviderTimeoutConfig {
                    dispatch_timeout_secs: 5,
                    max_attempts: 0,
                    ..Default::default()
                },
                ..Default::default()
            },
        )]),
        ..Default::default()
    };
    config.validate().expect("Vertex config");
    let key = config.api_key().expect("keyless Vertex host admission");
    assert!(key.is_empty());
    assert!(!path.exists());
    let provider = crate::openai_responses::factory::build_configured_provider(&config, &key)
        .expect("lazy factory with absent ADC");
    let credentials = if mode.ends_with("service-account") {
        json!({"type":"service_account","client_email":"fixture@example.invalid","private_key_id":"fixture-key","project_id":"fixture-project","private_key":include_str!("../tests/fixtures/gemini/test-only-service-account.pem")})
    } else {
        json!({"type":"authorized_user","client_id":"fixture-client","client_secret":"fixture-client-secret","refresh_token":"fixture-refresh"})
    };
    std::fs::write(&path, serde_json::to_vec(&credentials).expect("JSON"))
        .expect("ADC appears after factory construction");
    let mut tools = talos_core::tool::ToolRegistry::new();
    tools.register(std::sync::Arc::new(talos_tools::file_tools::ReadTool::new(
        root.clone(),
    )));
    let engine = talos_permission::PermissionEngine::with_workspace_root(root.clone());
    let agent = talos_agent::Agent::with_security(
        provider,
        tools,
        Some(std::sync::Arc::new(engine)),
        None,
        root,
    )
    .with_reasoning_identity(
        Some("fixture-vertex".into()),
        Some("fixture-model".into()),
        true,
    );
    let (events, _receiver) = tokio::sync::mpsc::unbounded_channel();
    let (answer, history) = tokio::time::timeout(
        std::time::Duration::from_secs(20),
        agent.run_streaming("Read fixture.txt".into(), vec![], events),
    )
    .await
    .expect("bounded Agent round")
    .expect("factory Agent round");
    assert_eq!(answer, "factory ok");
    assert!(history.iter().any(|message| matches!(message, talos_core::message::Message::Tool { result } if !result.is_error && result.content.contains("LAZY_FACTORY_READ_OK"))));
    assert!(!format!("{history:?}").contains("factory-private-signature"));
    let durable = serde_json::to_string(&history).expect("history");
    for secret in [
        "fixture-access",
        "fixture-client-secret",
        "fixture-refresh",
        "BEGIN PRIVATE KEY",
    ] {
        assert!(!durable.contains(secret));
    }
}
