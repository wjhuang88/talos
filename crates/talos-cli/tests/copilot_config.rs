//! Public Copilot transport selection through the shipped CLI; mock previews never call a vendor.

use std::path::Path;
use std::process::Command;

const KEY: &str = "copilot-cli-controlled-test-key";

fn run(fixture_home: &Path, args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_talos"))
        .current_dir(fixture_home)
        .env("HOME", fixture_home)
        .env("USERPROFILE", fixture_home)
        .env("COPILOT_GITHUB_TOKEN", KEY)
        .args(args)
        .output()
        .expect("run CLI");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 output");
    assert!(!stdout.contains(KEY));
    assert!(!String::from_utf8_lossy(&output.stderr).contains(KEY));
    stdout
}

fn preview(fixture_home: &Path) -> serde_json::Value {
    let stdout = run(fixture_home, &["--mock", "--print", "/mock-request hello"]);
    let json = stdout.find('{').expect("request JSON");
    serde_json::Deserializer::from_str(&stdout[json..])
        .into_iter::<serde_json::Value>()
        .next()
        .expect("snapshot")
        .expect("request JSON")
}

fn setup() -> tempfile::TempDir {
    let fixture = tempfile::tempdir().expect("isolated profile");
    std::fs::create_dir_all(fixture.path().join(".talos")).expect("profile");
    std::fs::write(
        fixture.path().join(".talos/config.toml"),
        concat!(
            "provider = \"github-copilot\"\nmodel = \"claude-sonnet-4.6\"\n",
            "[providers.github-copilot]\napi_key_env = \"COPILOT_GITHUB_TOKEN\"\n"
        ),
    )
    .expect("config");
    fixture
}

fn auth(snapshot: &serde_json::Value) {
    assert_eq!(snapshot["headers"]["Authorization"], "Bearer ***");
    assert_eq!(snapshot["headers"]["Copilot-Integration-Id"], "talos");
    assert!(snapshot["headers"].get("x-api-key").is_none());
    uuid::Uuid::parse_str(
        snapshot["headers"]["X-Interaction-Id"]
            .as_str()
            .expect("identity"),
    )
    .expect("process identity");
}

#[test]
fn explicit_copilot_messages_and_wire_rollback_keep_the_selected_vendor_and_auth() {
    let fixture = setup();
    for (protocol, path) in [
        ("anthropic-messages", "/v1/messages"),
        ("openai-chat", "/chat/completions"),
        ("openai-responses", "/responses"),
    ] {
        run(
            fixture.path(),
            &[
                "config",
                "set",
                "providers.github-copilot.models.claude-sonnet-4.6.protocol",
                protocol,
            ],
        );
        let snapshot = preview(fixture.path());
        assert_eq!(
            snapshot["url"],
            format!("https://api.githubcopilot.com{path}")
        );
        auth(&snapshot);
        if protocol == "anthropic-messages" {
            assert_eq!(snapshot["headers"]["anthropic-version"], "2023-06-01");
            assert!(snapshot["body"].get("stream_options").is_none());
        }
    }
    let config =
        std::fs::read_to_string(fixture.path().join(".talos/config.toml")).expect("saved config");
    assert!(config.contains("COPILOT_GITHUB_TOKEN"));
    assert!(!config.contains(KEY));
}

#[test]
fn unreviewed_copilot_catalog_row_keeps_chat_default_until_data_admission() {
    let fixture = setup();
    let snapshot = preview(fixture.path());
    assert_eq!(
        snapshot["url"],
        "https://api.githubcopilot.com/chat/completions"
    );
    auth(&snapshot);
}

#[test]
fn full_wire_endpoint_template_cannot_bypass_model_override_validation() {
    let fixture = setup();
    let config_path = fixture.path().join(".talos/config.toml");
    let config = r#"provider = "github-copilot"
model = "claude-sonnet-4.6"
[providers.github-copilot]
api_key_env = "COPILOT_GITHUB_TOKEN"
base_url = "${COPILOT_ENDPOINT}"
[providers.github-copilot.models."claude-sonnet-4.6"]
protocol = "anthropic-messages"
"#;
    std::fs::write(&config_path, config).expect("config");
    let output = Command::new(env!("CARGO_BIN_EXE_talos"))
        .current_dir(fixture.path())
        .env("HOME", fixture.path())
        .env("USERPROFILE", fixture.path())
        .env("COPILOT_GITHUB_TOKEN", KEY)
        .env(
            "COPILOT_ENDPOINT",
            "https://api.githubcopilot.com/chat/completions",
        )
        .args(["--print", "hello"])
        .output()
        .expect("real startup");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("public Copilot URL does not match the selected wire protocol"),
        "{stderr}"
    );
    assert!(!stderr.contains(KEY));
    assert!(!String::from_utf8_lossy(&output.stdout).contains(KEY));
    assert_eq!(
        std::fs::read_to_string(config_path).expect("config preserved"),
        config
    );
}
