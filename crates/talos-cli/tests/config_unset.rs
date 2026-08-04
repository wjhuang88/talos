use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn unique_home(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "talos-cli-unset-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("test operation should succeed")
            .as_nanos()
    ));
    fs::create_dir_all(dir.join(".talos")).expect("test operation should succeed");
    dir
}

fn write_config(home: &std::path::Path, toml_content: &str) {
    fs::write(home.join(".talos/config.toml"), toml_content)
        .expect("test operation should succeed");
}

fn write_credentials(home: &std::path::Path, toml_content: &str) {
    fs::write(home.join(".talos/credentials.toml"), toml_content)
        .expect("test operation should succeed");
}

fn run_cmd(home: &std::path::Path, args: &[&str]) -> (bool, String, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_talos"));
    cmd.env("HOME", home);
    cmd.env("TALOS_INSTALL_DIR", home.join(".talos/bin"));
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

fn read_bytes(path: &std::path::Path) -> Vec<u8> {
    fs::read(path).unwrap_or_default()
}

fn assert_no_secret(stdout: &str, stderr: &str, markers: &[&str]) {
    for m in markers {
        assert!(!stdout.contains(m), "secret marker '{m}' found in stdout");
        assert!(!stderr.contains(m), "secret marker '{m}' found in stderr");
    }
}

fn assert_no_temp_residual(home: &std::path::Path) {
    let entries: Vec<String> = fs::read_dir(home.join(".talos"))
        .expect("test operation should succeed")
        .map(|e| {
            e.expect("test operation should succeed")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert!(
        !entries.iter().any(|n| n.contains(".tmp")),
        "no temp files must remain: {entries:?}"
    );
}

fn cleanup(home: &std::path::Path) {
    let _ = fs::remove_dir_all(home);
}

const SECRET_A: &str = "sk-MARKER-A";
const SECRET_B: &str = "sk-MARKER-B";
const SECRET_ANT: &str = "sk-MARKER-ANT";

#[test]
fn missing_confirm_both_files_byte_identical() {
    let home = unique_home("no-confirm");
    write_config(
        &home,
        &format!(
            r#"
provider = "custom-a"
model = "model-a"

[providers.custom-a]
api_key = "{SECRET_A}"
"#
        ),
    );
    write_credentials(&home, &format!("custom-a = \"{SECRET_A}\"\n"));

    let config_before = read_bytes(&home.join(".talos/config.toml"));
    let creds_before = read_bytes(&home.join(".talos/credentials.toml"));

    let (success, stdout, stderr) = run_cmd(&home, &["config", "unset", "providers.custom-a"]);
    assert!(!success);
    assert!(stderr.contains("--confirm"));

    assert_eq!(config_before, read_bytes(&home.join(".talos/config.toml")),);
    assert_eq!(
        creds_before,
        read_bytes(&home.join(".talos/credentials.toml")),
    );
    assert_no_secret(&stdout, &stderr, &[SECRET_A]);
    cleanup(&home);
}

#[test]
fn custom_provider_removed_from_both_files() {
    let home = unique_home("custom-rm");
    write_config(
        &home,
        &format!(
            r#"
provider = "custom-a"
model = "model-a"

[providers.custom-a]
api_key = "{SECRET_A}"

[providers.custom-b]
api_key = "{SECRET_B}"
"#
        ),
    );
    write_credentials(
        &home,
        &format!("custom-a = \"{SECRET_A}\"\ncustom-b = \"{SECRET_B}\"\n"),
    );

    let (success, stdout, stderr) = run_cmd(
        &home,
        &["config", "unset", "providers.custom-a", "--confirm"],
    );
    assert!(success);
    assert!(stdout.contains("removed"));
    assert_no_secret(&stdout, &stderr, &[SECRET_A]);

    let config_raw =
        fs::read_to_string(home.join(".talos/config.toml")).expect("test operation should succeed");
    let reloaded: talos_config::Config =
        toml::from_str(&config_raw).expect("test operation should succeed");
    assert!(!reloaded.providers.contains_key("custom-a"));
    assert!(reloaded.providers.contains_key("custom-b"));

    let creds_raw = fs::read_to_string(home.join(".talos/credentials.toml"))
        .expect("test operation should succeed");
    assert!(!creds_raw.contains("custom-a"));
    assert!(creds_raw.contains("custom-b"));
    assert_no_temp_residual(&home);
    cleanup(&home);
}

#[test]
fn builtin_provider_disconnected() {
    let home = unique_home("builtin-disc");
    write_config(
        &home,
        &format!(
            r#"
provider = "anthropic"
model = "claude-test"

[providers.anthropic]
api_key = "{SECRET_ANT}"
"#
        ),
    );
    write_credentials(&home, &format!("anthropic = \"{SECRET_ANT}\"\n"));

    let (success, stdout, stderr) = run_cmd(
        &home,
        &["config", "unset", "providers.anthropic", "--confirm"],
    );
    assert!(success);
    assert!(stdout.contains("disconnected"));
    assert!(!stdout.contains("destroyed"));
    assert_no_secret(&stdout, &stderr, &[SECRET_ANT]);

    let config_raw =
        fs::read_to_string(home.join(".talos/config.toml")).expect("test operation should succeed");
    let reloaded: talos_config::Config =
        toml::from_str(&config_raw).expect("test operation should succeed");
    assert!(!reloaded.providers.contains_key("anthropic"));
    assert_no_temp_residual(&home);
    cleanup(&home);
}

#[test]
fn api_key_cleared_preserves_other_fields() {
    let home = unique_home("apikey");
    write_config(
        &home,
        &format!(
            r#"
provider = "my-gw"
model = "test-model"

[providers.my-gw]
protocol = "openai-chat"
base_url = "https://gw.example.com/v1"
api_key = "{SECRET_A}"
api_key_env = "GW_API_KEY"
"#
        ),
    );

    let (success, stdout, stderr) = run_cmd(
        &home,
        &["config", "unset", "providers.my-gw.api_key", "--confirm"],
    );
    assert!(success);
    assert!(stdout.contains("cleared"));
    assert_no_secret(&stdout, &stderr, &[SECRET_A]);

    let config_raw =
        fs::read_to_string(home.join(".talos/config.toml")).expect("test operation should succeed");
    assert!(config_raw.contains("base_url"));
    assert!(config_raw.contains("api_key_env"));
    assert!(!config_raw.contains(SECRET_A));
    assert!(!config_raw.contains("api_key = \"\""));
    assert_no_temp_residual(&home);
    cleanup(&home);
}

#[test]
fn unknown_provider_both_files_unchanged() {
    let home = unique_home("unknown");
    write_config(&home, "provider = \"x\"\nmodel = \"y\"\n");
    write_credentials(&home, "x = \"key-x\"\n");

    let config_before = read_bytes(&home.join(".talos/config.toml"));
    let creds_before = read_bytes(&home.join(".talos/credentials.toml"));

    let (success, _stdout, stderr) = run_cmd(
        &home,
        &["config", "unset", "providers.nonexistent", "--confirm"],
    );
    assert!(!success);
    assert!(stderr.contains("not found"));

    assert_eq!(config_before, read_bytes(&home.join(".talos/config.toml")));
    assert_eq!(
        creds_before,
        read_bytes(&home.join(".talos/credentials.toml"))
    );
    cleanup(&home);
}

#[test]
fn invalid_dotted_key_both_files_unchanged() {
    let home = unique_home("invalid");
    write_config(&home, "provider = \"x\"\nmodel = \"y\"\n");

    let config_before = read_bytes(&home.join(".talos/config.toml"));

    let (success, _stdout, stderr) = run_cmd(&home, &["config", "unset", "model", "--confirm"]);
    assert!(!success);
    assert!(stderr.contains("unsupported"));

    assert_eq!(config_before, read_bytes(&home.join(".talos/config.toml")));
    cleanup(&home);
}

#[test]
fn credentials_only_custom_provider_removed() {
    let home = unique_home("creds-only-custom");
    write_config(&home, "provider = \"x\"\nmodel = \"y\"\n");
    write_credentials(&home, &format!("old-custom = \"{SECRET_A}\"\n"));

    let (success, stdout, stderr) = run_cmd(
        &home,
        &["config", "unset", "providers.old-custom", "--confirm"],
    );
    assert!(success);
    assert!(stdout.contains("removed"));
    assert_no_secret(&stdout, &stderr, &[SECRET_A]);

    let creds_path = home.join(".talos/credentials.toml");
    if creds_path.exists() {
        assert!(
            !fs::read_to_string(&creds_path)
                .expect("test operation should succeed")
                .contains("old-custom")
        );
    }
    cleanup(&home);
}

#[test]
fn credentials_only_builtin_provider_removed() {
    let home = unique_home("creds-only-builtin");
    write_config(&home, "provider = \"x\"\nmodel = \"y\"\n");
    write_credentials(&home, &format!("anthropic = \"{SECRET_ANT}\"\n"));

    let (success, stdout, stderr) = run_cmd(
        &home,
        &["config", "unset", "providers.anthropic", "--confirm"],
    );
    assert!(success);
    assert!(stdout.contains("disconnected"));
    assert_no_secret(&stdout, &stderr, &[SECRET_ANT]);

    let creds_path = home.join(".talos/credentials.toml");
    if creds_path.exists() {
        assert!(
            !fs::read_to_string(&creds_path)
                .expect("test operation should succeed")
                .contains("anthropic")
        );
    }
    cleanup(&home);
}

#[test]
fn active_custom_provider_unset_reaches_no_init_rejection() {
    let home = unique_home("active-custom-noinit");
    write_config(
        &home,
        &format!(
            r#"
provider = "active-gw"
model = "active-model"

[providers.active-gw]
api_key = "{SECRET_A}"
"#
        ),
    );

    let (success, stdout, stderr) = run_cmd(
        &home,
        &["config", "unset", "providers.active-gw", "--confirm"],
    );
    assert!(success);
    assert_no_secret(&stdout, &stderr, &[SECRET_A]);
    assert_no_temp_residual(&home);

    let config_raw =
        fs::read_to_string(home.join(".talos/config.toml")).expect("test operation should succeed");
    let reloaded: talos_config::Config =
        toml::from_str(&config_raw).expect("test operation should succeed");
    assert!(!reloaded.providers.contains_key("active-gw"));
    assert!(reloaded.api_key().is_err());

    let (init_success, _init_stdout, init_stderr) = run_cmd(&home, &["--no-init", "-p", "test"]);
    assert!(
        !init_success,
        "talos --no-init must reject when active provider credential is gone"
    );
    assert!(
        init_stderr.to_lowercase().contains("api key")
            || init_stderr.to_lowercase().contains("no model")
            || init_stderr.contains("--no-init"),
        "stderr must mention api key or model setup: {init_stderr}"
    );
    assert_no_secret(&_init_stdout, &init_stderr, &[SECRET_A]);

    cleanup(&home);
}

#[test]
fn active_builtin_provider_unset_reaches_no_init_rejection() {
    let home = unique_home("active-builtin-noinit");
    write_config(
        &home,
        &format!(
            r#"
provider = "anthropic"
model = "claude-test"

[providers.anthropic]
api_key = "{SECRET_ANT}"
"#
        ),
    );

    let (success, stdout, stderr) = run_cmd(
        &home,
        &["config", "unset", "providers.anthropic", "--confirm"],
    );
    assert!(success);
    assert_no_secret(&stdout, &stderr, &[SECRET_ANT]);

    let config_raw =
        fs::read_to_string(home.join(".talos/config.toml")).expect("test operation should succeed");
    let reloaded: talos_config::Config =
        toml::from_str(&config_raw).expect("test operation should succeed");
    assert!(!reloaded.providers.contains_key("anthropic"));
    assert!(reloaded.api_key().is_err());

    let (init_success, _init_stdout, init_stderr) = run_cmd(&home, &["--no-init", "-p", "test"]);
    assert!(!init_success);
    assert!(
        init_stderr.to_lowercase().contains("api key")
            || init_stderr.to_lowercase().contains("no model")
            || init_stderr.contains("--no-init"),
    );

    cleanup(&home);
}

#[test]
fn config_list_after_unset_no_secret() {
    let home = unique_home("list-after");
    write_config(
        &home,
        &format!(
            r#"
provider = "my-gw"
model = "test-model"

[providers.my-gw]
api_key = "{SECRET_A}"

[providers.keeper]
api_key = "{SECRET_B}"
"#
        ),
    );

    run_cmd(&home, &["config", "unset", "providers.my-gw", "--confirm"]);

    let (list_ok, list_out, list_err) = run_cmd(&home, &["config", "list"]);
    assert!(list_ok);
    assert_no_secret(&list_out, &list_err, &[SECRET_A]);

    let (get_ok, get_out, get_err) = run_cmd(&home, &["config", "get", "providers.my-gw.api_key"]);
    assert!(!get_ok || !get_out.contains(SECRET_A));
    assert_no_secret(&get_out, &get_err, &[SECRET_A]);

    let (keep_ok, keep_out, _keep_err) =
        run_cmd(&home, &["config", "get", "providers.keeper.api_key"]);
    assert!(keep_ok);
    assert_eq!(keep_out.trim(), "***");

    cleanup(&home);
}

#[test]
fn config_get_after_api_key_clear_no_secret() {
    let home = unique_home("get-after-clear");
    write_config(
        &home,
        &format!(
            r#"
provider = "gw"
model = "m"

[providers.gw]
api_key = "{SECRET_A}"
base_url = "https://gw.example.com"
"#
        ),
    );

    run_cmd(
        &home,
        &["config", "unset", "providers.gw.api_key", "--confirm"],
    );

    let (get_ok, get_out, get_err) = run_cmd(&home, &["config", "get", "providers.gw.api_key"]);
    assert!(get_ok);
    assert!(!get_out.contains(SECRET_A));
    assert_no_secret(&get_out, &get_err, &[SECRET_A]);

    let (list_ok, list_out, list_err) = run_cmd(&home, &["config", "list"]);
    assert!(list_ok);
    assert_no_secret(&list_out, &list_err, &[SECRET_A]);

    cleanup(&home);
}

#[test]
fn unrelated_credential_not_copied_to_config() {
    let home = unique_home("unrelated-creds");
    write_config(
        &home,
        &format!(
            r#"
provider = "target"
model = "m"

[providers.target]
api_key = "{SECRET_A}"

[providers.keeper]
api_key = "{SECRET_B}"
"#
        ),
    );
    write_credentials(
        &home,
        &format!("target = \"{SECRET_A}\"\nkeeper = \"{SECRET_B}\"\norphan = \"{SECRET_ANT}\"\n"),
    );

    run_cmd(&home, &["config", "unset", "providers.target", "--confirm"]);

    let config_raw =
        fs::read_to_string(home.join(".talos/config.toml")).expect("test operation should succeed");
    let reloaded: talos_config::Config =
        toml::from_str(&config_raw).expect("test operation should succeed");
    assert!(!reloaded.providers.contains_key("target"));
    assert!(reloaded.providers.contains_key("keeper"));
    assert!(
        !reloaded.providers.contains_key("orphan"),
        "orphan credential must not create a config entry"
    );

    assert_no_temp_residual(&home);
    cleanup(&home);
}

#[test]
fn crash_recovery_restores_before_state_on_config_load() {
    let home = unique_home("crash-recovery");
    write_config(
        &home,
        &format!(
            r#"provider = "my-gw"
model = "test-model"

[providers.my-gw]
api_key = "{SECRET_A}"
"#
        ),
    );

    let config_before = read_bytes(&home.join(".talos/config.toml"));
    let talos_dir = home.join(".talos");
    let txn_dir = talos_dir.join(".provider-unset-transaction");
    fs::create_dir_all(&txn_dir).expect("test operation should succeed");
    fs::write(
        txn_dir.join("manifest"),
        "version = 1\nphase = \"Prepared\"\ntransaction_id = \"test-1\"\nconfig_existed_before = true\nconfig_exists_after = true\ncredentials_existed_before = false\ncredentials_exist_after = false\n",
    )
    .expect("test operation should succeed");
    fs::write(txn_dir.join("config.before"), &config_before)
        .expect("test operation should succeed");

    let (list_ok, list_stdout, list_err) = run_cmd(&home, &["config", "list"]);
    assert!(list_ok, "config list must succeed after recovery");

    assert_eq!(config_before, read_bytes(&home.join(".talos/config.toml")),);
    assert!(!txn_dir.exists(), "journal must be cleaned up");
    assert_no_secret(&list_stdout, &list_err, &[SECRET_A]);
    cleanup(&home);
}

#[test]
fn committed_transaction_preserves_after_state_on_config_load() {
    let home = unique_home("committed-recovery");
    write_config(
        &home,
        r#"provider = "x"
model = "y"

[providers.x]
api_key = "key-x"
"#,
    );
    let new_config = "provider = \"x\"\nmodel = \"y\"\n";
    fs::write(home.join(".talos/config.toml"), new_config).expect("test operation should succeed");

    let txn_dir = home.join(".talos/.provider-unset-transaction");
    fs::create_dir_all(&txn_dir).expect("test operation should succeed");
    fs::write(txn_dir.join("config.after"), new_config.as_bytes())
        .expect("test operation should succeed");
    fs::write(
        txn_dir.join("manifest"),
        "version = 1\nphase = \"Committed\"\ntransaction_id = \"test-2\"\nconfig_existed_before = true\nconfig_exists_after = true\ncredentials_existed_before = false\ncredentials_exist_after = false\n",
    )
    .expect("test operation should succeed");

    let (list_ok, _stdout, _err) = run_cmd(&home, &["config", "list"]);
    assert!(list_ok);

    let after =
        fs::read_to_string(home.join(".talos/config.toml")).expect("test operation should succeed");
    assert_eq!(after, new_config);
    assert!(!txn_dir.exists());
    cleanup(&home);
}
