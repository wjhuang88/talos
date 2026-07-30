use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const SECRET: &str = "sk-I157-CLI-SECRET";

fn unique_home(label: &str) -> PathBuf {
    let home = std::env::temp_dir().join(format!(
        "talos-cli-i157-finalization-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(home.join(".talos")).unwrap();
    home
}

fn run_cmd(home: &Path, args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_talos"))
        .env("HOME", home)
        .env("TALOS_INSTALL_DIR", home.join(".talos/bin"))
        .args(args)
        .output()
        .expect("failed to run talos");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn assert_no_marker(stdout: &str, stderr: &str) {
    assert!(!stdout.contains(SECRET), "secret marker leaked to stdout");
    assert!(!stderr.contains(SECRET), "secret marker leaked to stderr");
}

fn write_config_and_credentials(home: &Path) -> (Vec<u8>, Vec<u8>) {
    let config = format!(
        "provider = \"custom-a\"\nmodel = \"model-a\"\n\n\
         [providers.custom-a]\napi_key = \"{SECRET}\"\n"
    )
    .into_bytes();
    let credentials = format!("custom-a = \"{SECRET}\"\n").into_bytes();
    fs::write(home.join(".talos/config.toml"), &config).unwrap();
    fs::write(home.join(".talos/credentials.toml"), &credentials).unwrap();
    (config, credentials)
}

fn write_manifest(
    dir: &Path,
    phase: &str,
    transaction_id: &str,
    config_before: bool,
    config_after: bool,
    credentials_before: bool,
    credentials_after: bool,
) {
    fs::write(
        dir.join("manifest"),
        format!(
            "version = 1\nphase = \"{phase}\"\ntransaction_id = \"{transaction_id}\"\n\
             config_existed_before = {config_before}\nconfig_exists_after = {config_after}\n\
             credentials_existed_before = {credentials_before}\n\
             credentials_exist_after = {credentials_after}\n"
        ),
    )
    .unwrap();
}

#[test]
fn real_cli_verifies_and_cleans_valid_finalize_residue() {
    let home = unique_home("valid-residue");
    let (config, credentials) = write_config_and_credentials(&home);
    let transaction_id = "cli-finalize-1";
    let finalize = home.join(format!(
        ".talos/.provider-unset-transaction.finalize.{transaction_id}"
    ));
    fs::create_dir_all(&finalize).unwrap();
    fs::write(finalize.join("config.after"), &config).unwrap();
    fs::write(finalize.join("credentials.after"), &credentials).unwrap();
    write_manifest(
        &finalize,
        "Committed",
        transaction_id,
        true,
        true,
        true,
        true,
    );

    let (success, stdout, stderr) = run_cmd(&home, &["config", "list"]);
    assert!(success, "config list failed: {stderr}");
    assert_no_marker(&stdout, &stderr);
    assert!(!finalize.exists());

    let _ = fs::remove_dir_all(home);
}

#[test]
fn real_cli_fails_closed_on_ambiguous_active_and_finalize_evidence() {
    let home = unique_home("ambiguous");
    let (config, credentials) = write_config_and_credentials(&home);
    let transaction_id = "cli-ambiguous-1";
    let active = home.join(".talos/.provider-unset-transaction");
    let finalize = home.join(format!(
        ".talos/.provider-unset-transaction.finalize.{transaction_id}"
    ));
    fs::create_dir_all(&active).unwrap();
    fs::create_dir_all(&finalize).unwrap();

    for journal in [&active, &finalize] {
        fs::write(journal.join("config.after"), &config).unwrap();
        fs::write(journal.join("credentials.after"), &credentials).unwrap();
        write_manifest(
            journal,
            "Committed",
            transaction_id,
            true,
            true,
            true,
            true,
        );
    }

    let (success, stdout, stderr) = run_cmd(&home, &["config", "list"]);
    assert!(!success);
    assert!(
        stderr.to_lowercase().contains("ambiguous")
            || stdout.to_lowercase().contains("ambiguous")
    );
    assert_no_marker(&stdout, &stderr);
    assert!(active.exists());
    assert!(finalize.exists());

    let _ = fs::remove_dir_all(home);
}

#[test]
fn real_cli_corrupt_credentials_error_is_redacted() {
    let home = unique_home("corrupt-credentials");
    let _ = write_config_and_credentials(&home);
    fs::write(
        home.join(".talos/credentials.toml"),
        format!("leaked = \"{SECRET}\"\nbroken = ["),
    )
    .unwrap();

    let (success, stdout, stderr) = run_cmd(&home, &["config", "list"]);
    assert!(!success);
    assert_no_marker(&stdout, &stderr);
    assert!(
        stderr.contains("credentials.toml is not valid TOML")
            || stdout.contains("credentials.toml is not valid TOML")
    );

    let _ = fs::remove_dir_all(home);
}

#[test]
fn real_cli_non_utf8_config_fails_closed() {
    let home = unique_home("non-utf8-config");
    fs::write(
        home.join(".talos/config.toml"),
        b"provider = \"x\"\nmodel = \"y\"\n\xff\xfe",
    )
    .unwrap();

    let (success, stdout, stderr) = run_cmd(&home, &["config", "list"]);
    assert!(!success);
    assert_no_marker(&stdout, &stderr);
    assert!(
        stderr.contains("config.toml is not valid UTF-8")
            || stdout.contains("config.toml is not valid UTF-8")
    );

    let _ = fs::remove_dir_all(home);
}

#[test]
fn real_cli_missing_committed_image_fails_closed_and_keeps_journal() {
    let home = unique_home("missing-image");
    let _ = write_config_and_credentials(&home);
    let active = home.join(".talos/.provider-unset-transaction");
    fs::create_dir_all(&active).unwrap();
    write_manifest(
        &active,
        "Committed",
        "cli-missing-image",
        true,
        true,
        true,
        true,
    );

    let (success, stdout, stderr) = run_cmd(&home, &["config", "list"]);
    assert!(!success);
    assert_no_marker(&stdout, &stderr);
    assert!(active.exists());

    let _ = fs::remove_dir_all(home);
}
