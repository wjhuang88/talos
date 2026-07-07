//! Regression guard for MC-002: the Talos CLI must not open, create, or seed
//! `~/.talos/catalog.db` at runtime. Runtime model/provider metadata is
//! sourced exclusively from the packaged `crates/talos-config/src/models.toml`
//! and user `~/.talos/config.toml` overlays.
//!
//! This test drives every non-interactive CLI entry point that touches
//! model/provider metadata (`--import-models`, `--available-models`,
//! `--available-models --available-models-filter`, `--available-models-all`,
//! and `config list`) inside an isolated `HOME` directory, then asserts that
//! no `catalog.db` file was created under `~/.talos/`.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

/// Returns the absolute path to the freshly built `talos` binary, as Cargo
/// exposes via the `CARGO_BIN_EXE_<name>` environment variable for `[bin]`
/// crate integration tests.
fn talos_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_talos"))
}

/// A RUNNING_DIR-style fixture with an isolated HOME inside the cargo target
/// tree, so a stray `catalog.db` is detectable without touching the real user
/// home.
struct IsolatedHome {
    home: PathBuf,
    talos_dir: PathBuf,
    _tmp: tempfile::TempDir,
}

impl IsolatedHome {
    fn new() -> Self {
        let tmp = tempfile::TempDir::new().expect("create temp dir for isolated HOME");
        let home = tmp.path().join("home").to_path_buf();
        let talos_dir = home.join(".talos");
        fs::create_dir_all(&talos_dir).expect("create ~/.talos");
        fs::write(
            talos_dir.join("config.toml"),
            "# isolated HOME; intentionally empty config for the no-catalog.db guard test\n",
        )
        .expect("write empty config.toml");
        Self {
            home,
            talos_dir,
            _tmp: tmp,
        }
    }

    fn catalog_db_path(&self) -> PathBuf {
        self.talos_dir.join("catalog.db")
    }

    fn run_success(&self, args: &[&str]) -> String {
        let output = Command::new(talos_bin())
            .args(args)
            .env("HOME", &self.home)
            // Windows ignores HOME; consult USERPROFILE there too.
            .env("USERPROFILE", &self.home)
            .env("XDG_CONFIG_HOME", "")
            .env("BUILD_MODELS", "")
            .output()
            .expect("run talos binary");
        assert_success(args, &output);
        format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    }
}

fn assert_success(args: &[&str], output: &Output) {
    assert!(
        output.status.success(),
        "talos {:?} failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
        args,
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn import_models_does_not_create_catalog_db() {
    let home = IsolatedHome::new();
    let stdout = home.run_success(&["--import-models", "/nonexistent/path.toml"]);
    assert!(stdout.contains("--import-models is deprecated"));
    assert!(stdout.contains("Talos does not create or read ~/.talos/catalog.db"));
    assert!(
        !home.catalog_db_path().exists(),
        "MC-002 regression: --import-models created ~/.talos/catalog.db"
    );
    let wal = home.talos_dir.join("catalog.db-wal");
    let shm = home.talos_dir.join("catalog.db-shm");
    assert!(!wal.exists(), "MC-002 regression: catalog.db-wal created");
    assert!(!shm.exists(), "MC-002 regression: catalog.db-shm created");
}

#[test]
fn available_models_does_not_create_catalog_db() {
    let home = IsolatedHome::new();
    let stdout = home.run_success(&["--available-models"]);
    assert!(stdout.contains("Built-in model catalog:"));
    assert!(stdout.contains("Showing first"));
    assert!(
        !home.catalog_db_path().exists(),
        "MC-002 regression: --available-models created ~/.talos/catalog.db"
    );
}

#[test]
fn available_models_filter_does_not_create_catalog_db() {
    let home = IsolatedHome::new();
    let stdout = home.run_success(&[
        "--available-models",
        "--available-models-filter",
        "anthropic/claude-sonnet-4",
    ]);
    assert!(stdout.contains("matching models across"));
    assert!(stdout.contains("anthropic/claude-sonnet-4"));
    assert!(
        !home.catalog_db_path().exists(),
        "MC-002 regression: --available-models-filter created ~/.talos/catalog.db"
    );
}

#[test]
fn available_models_all_does_not_create_catalog_db() {
    let home = IsolatedHome::new();
    let stdout = home.run_success(&["--available-models", "--available-models-all"]);
    assert!(stdout.contains("Built-in model catalog:"));
    assert!(stdout.contains('/'));
    assert!(
        !home.catalog_db_path().exists(),
        "MC-002 regression: --available-models-all created ~/.talos/catalog.db"
    );
}

#[test]
fn config_list_does_not_create_catalog_db() {
    let home = IsolatedHome::new();
    let output = home.run_success(&["config", "list"]);
    assert!(output.contains("provider ="));
    assert!(output.contains("model ="));
    assert!(
        !home.catalog_db_path().exists(),
        "MC-002 regression: `config list` created ~/.talos/catalog.db"
    );
}
