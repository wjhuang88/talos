//! Exercise protocol configuration and real adapter selection through the shipped CLI.

use std::process::{Command, Output};

fn run(home: &std::path::Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_talos"))
        .env("HOME", home)
        .env("USERPROFILE", home)
        .args(args)
        .output()
        .expect("run talos")
}

fn success(home: &std::path::Path, args: &[&str]) -> String {
    let output = run(home, args);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("UTF-8 output")
}

#[test]
fn protocol_edits_persist_and_model_override_selects_responses_request() {
    let home = tempfile::tempdir().expect("isolated home");
    std::fs::create_dir_all(home.path().join(".talos")).expect("config directory");
    std::fs::write(
        home.path().join(".talos/config.toml"),
        "provider = \"openai\"\nmodel = \"o3\"\n[providers.openai]\napi_key = \"test-key\"\n",
    )
    .expect("fixture config");

    success(
        home.path(),
        &["config", "set", "providers.openai.protocol", "openai-chat"],
    );
    success(
        home.path(),
        &[
            "config",
            "set",
            "providers.openai.models.o3.protocol",
            "openai-responses",
        ],
    );
    assert!(
        success(home.path(), &["config", "get", "providers.openai.protocol"])
            .contains("openai-chat")
    );
    assert!(
        success(
            home.path(),
            &["config", "get", "providers.openai.models.o3.protocol"]
        )
        .contains("openai-responses")
    );

    success(
        home.path(),
        &[
            "config",
            "set",
            "providers.openai.models.gpt-4.1.protocol",
            "openai-responses",
        ],
    );
    assert!(
        success(
            home.path(),
            &["config", "get", "providers.openai.models.gpt-4.1.protocol"]
        )
        .contains("openai-responses")
    );

    success(
        home.path(),
        &[
            "config",
            "set",
            "providers.openai.models.o3.base_url",
            "https://model-endpoint.test/openai/v1",
        ],
    );
    assert!(
        success(
            home.path(),
            &["config", "get", "providers.openai.models.o3.base_url"]
        )
        .contains("https://model-endpoint.test/openai/v1")
    );
    success(
        home.path(),
        &[
            "config",
            "set",
            "providers.openai.models.gpt-4.1.base_url",
            "https://other-model.test/v1",
        ],
    );
    assert!(
        success(
            home.path(),
            &["config", "get", "providers.openai.models.gpt-4.1.base_url"]
        )
        .contains("https://other-model.test/v1")
    );

    // Mock mode still obtains its diagnostic body from the real selected adapter. No API call.
    let preview = success(home.path(), &["--mock", "-p", "/mock-request hello"]);
    assert!(
        preview.contains("https://model-endpoint.test/openai/v1/responses"),
        "{preview}"
    );
    assert!(preview.contains("max_output_tokens"), "{preview}");
    assert!(!preview.contains("test-key"), "{preview}");

    success(
        home.path(),
        &[
            "config",
            "set",
            "providers.openai.models.o3.protocol",
            "openai-chat",
        ],
    );
    let preview = success(home.path(), &["--mock", "-p", "/mock-request hello"]);
    assert!(preview.contains("/chat/completions"), "{preview}");
    success(
        home.path(),
        &["config", "set", "providers.openai.models.o3.base_url", ""],
    );
    let preview = success(home.path(), &["--mock", "-p", "/mock-request hello"]);
    assert!(
        preview.contains("https://api.openai.com/v1/chat/completions"),
        "{preview}"
    );
}

#[test]
fn invalid_protocol_reports_all_choices_without_mutating_config() {
    let home = tempfile::tempdir().expect("isolated home");
    std::fs::create_dir_all(home.path().join(".talos")).expect("config directory");
    let path = home.path().join(".talos/config.toml");
    std::fs::write(&path, "provider = \"anthropic\"\nmodel = \"test\"\n").expect("fixture config");
    let before = std::fs::read(&path).expect("original config");
    for key in [
        "providers.openai.protocol",
        "providers.openai.models.o3.protocol",
    ] {
        let output = run(home.path(), &["config", "set", key, "unspecified"]);
        assert!(!output.status.success());
        let error = String::from_utf8_lossy(&output.stderr);
        for protocol in ["openai-chat", "openai-responses", "anthropic-messages"] {
            assert!(error.contains(protocol), "{error}");
        }
        assert_eq!(std::fs::read(&path).expect("unchanged config"), before);
    }
}

#[test]
fn unsupported_model_endpoint_edit_preserves_the_config() {
    let home = tempfile::tempdir().expect("home");
    let dir = home.path().join(".talos");
    std::fs::create_dir_all(&dir).expect("config directory");
    let path = dir.join("config.toml");
    std::fs::write(&path, "provider = \"openai\"\nmodel = \"o3\"\n").expect("config");
    let before = std::fs::read(&path).expect("original config");
    for endpoint in [
        "wss://example.test/realtime",
        "http://remote.test/v1",
        "http://localhost:80@remote.test/v1",
        "http://[::1]@remote.test/v1",
    ] {
        let output = run(
            home.path(),
            &[
                "config",
                "set",
                "providers.openai.models.o3.base_url",
                endpoint,
            ],
        );
        assert!(!output.status.success());
        assert_eq!(std::fs::read(&path).expect("unchanged config"), before);
    }
}

#[test]
fn model_endpoint_environment_template_is_preserved_by_cli_edit() {
    let home = tempfile::tempdir().expect("home");
    let dir = home.path().join(".talos");
    std::fs::create_dir_all(&dir).expect("config directory");
    let path = dir.join("config.toml");
    std::fs::write(&path, "provider = \"openai\"\nmodel = \"o3\"\n").expect("config");
    let output = Command::new(env!("CARGO_BIN_EXE_talos"))
        .env("HOME", home.path())
        .env("USERPROFILE", home.path())
        .env("TALOS_CONFIG_MODEL_ROOT", "https://model-env.test")
        .args([
            "config",
            "set",
            "providers.openai.models.o3.base_url",
            "${TALOS_CONFIG_MODEL_ROOT}/v1",
        ])
        .output()
        .expect("CLI edit");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let raw = std::fs::read_to_string(path).expect("persisted template");
    assert!(raw.contains("${TALOS_CONFIG_MODEL_ROOT}/v1"));
    assert!(!raw.contains("model-env.test"));
}

#[test]
fn invalid_manual_model_endpoint_returns_an_error_without_panicking() {
    for endpoint in [
        "wss://model.test/realtime",
        "${TALOS_MISSING_MODEL_ROOT_748}/v1",
        "http://localhost:80@remote.test/v1",
    ] {
        let home = tempfile::tempdir().expect("home");
        let dir = home.path().join(".talos");
        std::fs::create_dir_all(&dir).expect("config directory");
        let path = dir.join("config.toml");
        std::fs::write(&path, format!("provider = \"openai\"\nmodel = \"o3\"\n[providers.openai]\napi_key = \"endpoint-test-secret\"\n[providers.openai.models.o3]\nprotocol = \"openai-responses\"\nbase_url = {endpoint:?}\n")).expect("manual config");
        let output = run(home.path(), &["--print", "--no-context", "hello"]);
        assert!(!output.status.success());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("model endpoint"), "{error}");
        assert!(!error.contains("panicked"), "{error}");
        assert!(!error.contains("endpoint-test-secret"), "{error}");
    }
}

#[test]
fn neon_catalog_responses_rollback_restores_the_matching_provider_root() {
    let home = tempfile::tempdir().expect("home");
    let dir = home.path().join(".talos");
    std::fs::create_dir_all(&dir).expect("config directory");
    let path = dir.join("config.toml");
    let base = "provider = \"neon\"\nmodel = \"gpt-5-2\"\n[providers.neon]\nbase_url = \"${NEON_AI_GATEWAY_BASE_URL}/v1\"\napi_key_env = \"NEON_AI_GATEWAY_TOKEN\"\n";
    for persisted_provider_root in [true, false] {
        let base = if persisted_provider_root {
            base.to_string()
        } else {
            base.replace("base_url = \"${NEON_AI_GATEWAY_BASE_URL}/v1\"\n", "")
        };
        for (explicit, expected) in [
            (false, "https://neon-branch.test/openai/v1/responses"),
            (true, "https://neon-branch.test/v1/chat/completions"),
        ] {
            let config = if explicit {
                format!("{base}protocol = \"openai-chat\"\n")
            } else {
                base.clone()
            };
            std::fs::write(&path, config).expect("config");
            let output = Command::new(env!("CARGO_BIN_EXE_talos"))
                .env("HOME", home.path())
                .env("USERPROFILE", home.path())
                .env("NEON_AI_GATEWAY_BASE_URL", "https://neon-branch.test")
                .env("NEON_AI_GATEWAY_TOKEN", "endpoint-test-secret")
                .args(["--mock", "--print", "/mock-request hello"])
                .output()
                .expect("CLI preview");
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let preview = String::from_utf8_lossy(&output.stdout);
            assert!(preview.contains(expected), "{preview}");
            assert!(!preview.contains("endpoint-test-secret"), "{preview}");
            if explicit {
                assert!(!preview.contains("/openai/v1/responses"), "{preview}");
            }
        }
    }
}
