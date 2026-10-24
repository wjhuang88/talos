#![cfg(all(target_os = "linux", feature = "bwrap_installed"))]

use std::{fs, path::Path};
use talos_sandbox::{BubblewrapSandbox, SandboxConfig, SandboxProvider};

fn quote(path: &Path) -> String {
    format!(
        "'{}'",
        path.to_str()
            .expect("UTF-8 fixture path")
            .replace('\'', "'\\''")
    )
}

#[tokio::test]
async fn temporary_workspace_is_visible_without_exposing_host_siblings() {
    let root = tempfile::tempdir_in("/tmp").expect("fixture root");
    let workspace = root.path().join("workspace");
    fs::create_dir(&workspace).expect("workspace");
    fs::write(workspace.join("marker"), "marker").expect("marker");
    let sibling = root.path().join("host-only");
    fs::write(&sibling, "host sentinel").expect("sentinel");
    let config = SandboxConfig {
        workspace_root: workspace.clone(),
        allow_network: false,
        extra_read_paths: vec![],
    };
    let command = format!(
        "set -eu; test \"$PWD\" = {workspace}; test \"$(pwd -P)\" = {workspace}; \
         test \"$(cat marker)\" = marker; printf written > created; \
         test ! -e {sibling}; printf private > {sibling}",
        workspace = quote(&workspace),
        sibling = quote(&sibling),
    );
    let result = BubblewrapSandbox::new()
        .execute(&command, &config)
        .await
        .expect("native Bubblewrap must execute; namespace failures are test failures");
    assert_eq!(result.exit_code, 0, "{}", result.stderr);
    assert_eq!(
        fs::read_to_string(workspace.join("created")).expect("created file"),
        "written"
    );
    assert_eq!(
        fs::read_to_string(&sibling).expect("host sentinel"),
        "host sentinel"
    );
}

#[tokio::test]
async fn extra_read_paths_override_workspace_writes_and_host_root_stays_read_only() {
    let root = tempfile::tempdir_in("/tmp").expect("fixture root");
    let workspace = root.path().join("workspace");
    fs::create_dir(&workspace).expect("workspace");
    let extra = root.path().join("extra-read");
    let nested = workspace.join("nested-read");
    fs::write(&extra, "extra").expect("extra read fixture");
    fs::write(&nested, "nested").expect("nested read fixture");
    // /var/tmp is outside the private /tmp mount and the authorized workspace.
    let outside = tempfile::NamedTempFile::new_in("/var/tmp").expect("outside fixture");
    fs::write(outside.path(), "outside").expect("outside contents");
    let config = SandboxConfig {
        workspace_root: workspace,
        allow_network: false,
        extra_read_paths: vec![extra.clone(), nested.clone()],
    };
    let command = format!(
        "set -eu; test \"$(cat {extra})\" = extra; test \"$(cat {nested})\" = nested; \
         test \"$(cat {outside})\" = outside; \
         if (printf changed > {extra}) 2>/dev/null; then exit 11; fi; \
         if (printf changed > {nested}) 2>/dev/null; then exit 12; fi; \
         if (printf changed > {outside}) 2>/dev/null; then exit 13; fi",
        extra = quote(&extra),
        nested = quote(&nested),
        outside = quote(outside.path()),
    );
    let result = BubblewrapSandbox::new()
        .execute(&command, &config)
        .await
        .expect("native Bubblewrap must execute");
    assert_eq!(result.exit_code, 0, "{}", result.stderr);
    assert_eq!(fs::read_to_string(extra).expect("extra"), "extra");
    assert_eq!(fs::read_to_string(nested).expect("nested"), "nested");
    assert_eq!(
        fs::read_to_string(outside.path()).expect("outside"),
        "outside"
    );
}

#[tokio::test]
async fn missing_workspace_never_runs_the_payload_in_a_fallback_directory() {
    let root = tempfile::tempdir_in("/tmp").expect("fixture root");
    let witness = root.path().join("executed");
    let config = SandboxConfig {
        workspace_root: root.path().join("missing"),
        allow_network: false,
        extra_read_paths: vec![root.path().to_path_buf()],
    };
    let result = BubblewrapSandbox::new()
        .execute(
            &format!("printf EXECUTED; touch {}", quote(&witness)),
            &config,
        )
        .await;
    if let Ok(result) = result {
        assert_ne!(result.exit_code, 0);
        assert!(!result.stdout.contains("EXECUTED"));
    }
    assert!(!witness.exists(), "payload must never execute");
}
