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

    // Mock mode still obtains its diagnostic body from the real selected adapter. No API call.
    let preview = success(home.path(), &["--mock", "-p", "/mock-request hello"]);
    assert!(preview.contains("/responses"), "{preview}");
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
