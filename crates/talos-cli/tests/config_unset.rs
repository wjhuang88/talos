use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn unique_home(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "talos-cli-unset-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(dir.join(".talos")).unwrap();
    dir
}

fn write_config(home: &std::path::Path, toml_content: &str) {
    fs::write(home.join(".talos/config.toml"), toml_content).unwrap();
}

fn write_credentials(home: &std::path::Path, toml_content: &str) {
    fs::write(home.join(".talos/credentials.toml"), toml_content).unwrap();
}

fn run_unset(home: &std::path::Path, args: &[&str]) -> (bool, String, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_talos"));
    cmd.env("HOME", home);
    for arg in args {
        cmd.arg(arg);
    }
    let output = cmd.output().expect("failed to execute talos binary");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn file_hash(path: &std::path::Path) -> String {
    if !path.exists() {
        return "NONEXISTENT".to_string();
    }
    let bytes = fs::read(path).unwrap();
    format!("{}bytes", bytes.len())
}

fn cleanup(home: &std::path::Path) {
    let _ = fs::remove_dir_all(home);
}

#[test]
fn missing_confirm_exits_nonzero_and_leaves_files_unchanged() {
    let home = unique_home("no-confirm");
    let config_toml = r#"
provider = "custom-a"
model = "model-a"

[providers.custom-a]
api_key = "sk-test-marker-a"
"#;
    write_config(&home, config_toml);

    let config_before = file_hash(&home.join(".talos/config.toml"));

    let (success, stdout, stderr) = run_unset(&home, &["config", "unset", "providers.custom-a"]);
    assert!(!success, "must exit non-zero without --confirm");
    assert!(
        stderr.contains("--confirm"),
        "stderr must mention --confirm: {stderr}"
    );

    let config_after = file_hash(&home.join(".talos/config.toml"));
    assert_eq!(
        config_before, config_after,
        "config.toml must be byte-identical without --confirm"
    );
    assert!(
        !stdout.contains("sk-test-marker-a"),
        "credential must not appear in stdout"
    );
    cleanup(&home);
}

#[test]
fn custom_provider_removed_from_config_and_credentials() {
    let home = unique_home("custom-remove");
    write_config(
        &home,
        r#"
provider = "custom-a"
model = "model-a"

[providers.custom-a]
api_key = "sk-inline-a"
base_url = "https://a.example.com/v1"

[providers.custom-b]
api_key = "sk-inline-b"
"#,
    );
    write_credentials(
        &home,
        "custom-a = \"sk-creds-a\"\ncustom-b = \"sk-creds-b\"\n",
    );

    let (success, stdout, _stderr) = run_unset(
        &home,
        &["config", "unset", "providers.custom-a", "--confirm"],
    );
    assert!(success, "must succeed with --confirm");
    assert!(stdout.contains("removed"));

    let config_raw = fs::read_to_string(home.join(".talos/config.toml")).unwrap();
    let reloaded: talos_config::Config = toml::from_str(&config_raw).unwrap();
    assert!(!reloaded.providers.contains_key("custom-a"));
    assert!(reloaded.providers.contains_key("custom-b"));

    let creds_path = home.join(".talos/credentials.toml");
    let creds_raw = fs::read_to_string(&creds_path).unwrap();
    assert!(!creds_raw.contains("custom-a"));
    assert!(creds_raw.contains("custom-b"));
    cleanup(&home);
}

#[test]
fn builtin_provider_disconnected() {
    let home = unique_home("builtin-disconnect");
    write_config(
        &home,
        r#"
provider = "anthropic"
model = "claude-test"

[providers.anthropic]
api_key = "sk-ant-inline"
"#,
    );
    write_credentials(&home, "anthropic = \"sk-ant-creds\"\n");

    let (success, stdout, _stderr) = run_unset(
        &home,
        &["config", "unset", "providers.anthropic", "--confirm"],
    );
    assert!(success);
    assert!(
        stdout.contains("disconnected"),
        "output must use disconnected semantics: {stdout}"
    );
    assert!(
        !stdout.contains("destroyed"),
        "must not claim builtin was destroyed"
    );

    let config_raw = fs::read_to_string(home.join(".talos/config.toml")).unwrap();
    let reloaded: talos_config::Config = toml::from_str(&config_raw).unwrap();
    assert!(!reloaded.providers.contains_key("anthropic"));
    cleanup(&home);
}

#[test]
fn api_key_cleared_preserves_other_fields() {
    let home = unique_home("apikey-clear");
    write_config(
        &home,
        r#"
provider = "my-gw"
model = "test-model"

[providers.my-gw]
protocol = "openai-chat"
base_url = "https://gw.example.com/v1"
api_key = "sk-gw-secret"
api_key_env = "GW_API_KEY"
"#,
    );

    let (success, stdout, _stderr) = run_unset(
        &home,
        &["config", "unset", "providers.my-gw.api_key", "--confirm"],
    );
    assert!(success);
    assert!(stdout.contains("cleared"));

    let config_raw = fs::read_to_string(home.join(".talos/config.toml")).unwrap();
    assert!(config_raw.contains("base_url"));
    assert!(config_raw.contains("api_key_env"));
    assert!(
        !config_raw.contains("sk-gw-secret"),
        "cleared credential must not appear in config"
    );
    assert!(
        !config_raw.contains("api_key = \"\""),
        "must not write empty api_key"
    );
    cleanup(&home);
}

#[test]
fn unknown_provider_exits_nonzero() {
    let home = unique_home("unknown");
    write_config(&home, "provider = \"x\"\nmodel = \"y\"\n");

    let config_before = file_hash(&home.join(".talos/config.toml"));

    let (success, _stdout, stderr) = run_unset(
        &home,
        &["config", "unset", "providers.nonexistent", "--confirm"],
    );
    assert!(!success);
    assert!(stderr.contains("not found"));

    let config_after = file_hash(&home.join(".talos/config.toml"));
    assert_eq!(config_before, config_after);
    cleanup(&home);
}

#[test]
fn invalid_dotted_key_exits_nonzero() {
    let home = unique_home("invalid-key");
    write_config(&home, "provider = \"x\"\nmodel = \"y\"\n");

    let (success, _stdout, stderr) = run_unset(&home, &["config", "unset", "model", "--confirm"]);
    assert!(!success);
    assert!(stderr.contains("unsupported"));
    cleanup(&home);
}

#[test]
fn credentials_only_custom_provider_removed() {
    let home = unique_home("creds-only-custom");
    write_config(&home, "provider = \"x\"\nmodel = \"y\"\n");
    write_credentials(&home, "old-custom = \"sk-old\"\n");

    let (success, stdout, _stderr) = run_unset(
        &home,
        &["config", "unset", "providers.old-custom", "--confirm"],
    );
    assert!(success);
    assert!(stdout.contains("removed"));

    let creds_path = home.join(".talos/credentials.toml");
    if creds_path.exists() {
        let creds_raw = fs::read_to_string(&creds_path).unwrap();
        assert!(!creds_raw.contains("old-custom"));
    }
    cleanup(&home);
}

#[test]
fn credentials_only_builtin_provider_removed() {
    let home = unique_home("creds-only-builtin");
    write_config(&home, "provider = \"x\"\nmodel = \"y\"\n");
    write_credentials(&home, "anthropic = \"sk-legacy\"\n");

    let (success, stdout, _stderr) = run_unset(
        &home,
        &["config", "unset", "providers.anthropic", "--confirm"],
    );
    assert!(success);
    assert!(
        stdout.contains("disconnected"),
        "credentials-only builtin must report disconnected"
    );

    let creds_path = home.join(".talos/credentials.toml");
    if creds_path.exists() {
        let creds_raw = fs::read_to_string(&creds_path).unwrap();
        assert!(!creds_raw.contains("anthropic"));
    }
    cleanup(&home);
}

#[test]
fn active_custom_provider_unset_is_picker_recoverable() {
    let home = unique_home("active-custom");
    write_config(
        &home,
        r#"
provider = "active-gw"
model = "active-model"

[providers.active-gw]
api_key = "sk-active"
"#,
    );

    let (success, stdout, _stderr) = run_unset(
        &home,
        &["config", "unset", "providers.active-gw", "--confirm"],
    );
    assert!(success);

    let config_raw = fs::read_to_string(home.join(".talos/config.toml")).unwrap();
    let reloaded: talos_config::Config = toml::from_str(&config_raw).unwrap();
    assert!(!reloaded.providers.contains_key("active-gw"));
    assert!(reloaded.api_key().is_err());
    let pc = reloaded.active_provider_config();
    assert!(pc.api_key.is_none());

    assert!(
        !stdout.contains("sk-active"),
        "credential must not appear in stdout"
    );
    cleanup(&home);
}

#[test]
fn active_builtin_provider_unset_is_picker_recoverable() {
    let home = unique_home("active-builtin");
    write_config(
        &home,
        r#"
provider = "anthropic"
model = "claude-test"

[providers.anthropic]
api_key = "sk-ant-active"
"#,
    );

    let (success, _stdout, _stderr) = run_unset(
        &home,
        &["config", "unset", "providers.anthropic", "--confirm"],
    );
    assert!(success);

    let config_raw = fs::read_to_string(home.join(".talos/config.toml")).unwrap();
    let reloaded: talos_config::Config = toml::from_str(&config_raw).unwrap();
    assert!(!reloaded.providers.contains_key("anthropic"));
    assert!(
        reloaded.api_key().is_err(),
        "active builtin removal must leave api_key() in error state for picker recovery"
    );
    cleanup(&home);
}

#[test]
fn no_temp_residual_after_success() {
    let home = unique_home("no-residual");
    write_config(
        &home,
        r#"
provider = "gw"
model = "m"

[providers.gw]
api_key = "sk-gw"
"#,
    );

    run_unset(&home, &["config", "unset", "providers.gw", "--confirm"]);

    let entries: Vec<String> = fs::read_dir(home.join(".talos"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        !entries
            .iter()
            .any(|n| n.contains(".atomic-tmp") || n.ends_with(".tmp")),
        "no temp files must remain: {entries:?}"
    );
    cleanup(&home);
}
