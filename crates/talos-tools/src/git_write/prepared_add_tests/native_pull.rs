//! Native whole-operation baselines for replacing the staged Pull scheduler.
//! Bounded discovery, final-stage admission and runner baselines, not activation.

use super::*;

#[path = "native_pull_recursion.rs"]
mod recursion;

#[test]
fn native_pull_plan_rejects_changed_operation_before_discovery() {
    let root = tempfile::tempdir().expect("fixture");
    git(root.path(), &["init", "-b", "main"]);
    let mut plan = prepare_pull_fixture(
        root.path(),
        &serde_json::json!({"remote": "upstream"}),
        ToolProvenance::Native,
    )
    .expect("Pull plan");
    plan.revalidate().expect("unchanged plan");
    let command = plan
        .native_pull_command
        .as_ref()
        .expect("retained command")
        .command();
    let arguments = command.as_std().get_args().collect::<Vec<_>>();
    assert_eq!(arguments[5..], ["pull", "--", "upstream"]);
    plan.operation = Operation::Pull(GitPullInput {
        remote: Some("other".into()),
    });
    assert!(plan.revalidate().is_err());
    // Discovery never receives the worktree/metadata mutation or network scope.
    assert!(
        !plan
            .facets
            .iter()
            .any(|facet| matches!(facet.nature, ToolNature::Write | ToolNature::Network))
    );
}

#[tokio::test]
async fn native_pull_command_refuses_missing_stale_or_incomplete_authority() {
    for scenario in ["missing-proof", "stale-proof", "missing-command-authority"] {
        let root = tempfile::tempdir().expect("fixture");
        git(root.path(), &["init", "-b", "main"]);
        let plan =
            prepare_pull_fixture(root.path(), &serde_json::json!({}), ToolProvenance::Native)
                .expect("plan");
        let mut plan = Box::new(plan)
            .discover_pull_command()
            .await
            .expect("discovery");
        if scenario == "missing-command-authority" {
            plan.facets
                .retain(|facet| facet.resource.as_deref() != Some("git:pull-command"));
        }
        let admission: Box<dyn std::any::Any + Send> = if scenario == "missing-proof" {
            Box::new(())
        } else {
            let admission = proof(&plan);
            if scenario == "stale-proof" {
                plan.identity = PreparedPlanIdentity::new();
            }
            Box::new(admission)
        };
        let PreparedNormalStage::Complete(output) = plan.execute_stage(admission).await else {
            panic!("refusal must complete without further discovery")
        };
        assert!(output.result.is_error, "{scenario}");
        let expected = match scenario {
            "missing-proof" => "requires its admitted proof",
            "stale-proof" => "does not authorize this plan",
            _ => "command authority missing",
        };
        assert!(
            output.result.content.contains(expected),
            "{scenario}: {:?}",
            output.result
        );
        assert!(!root.path().join(".git/FETCH_HEAD").exists());
    }
}

pub(super) fn git(cwd: &Path, args: &[&str]) -> Vec<u8> {
    let output = std::process::Command::new("git")
        .current_dir(cwd)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", cwd.join("absent-global-config"))
        .env("GIT_AUTHOR_NAME", "Native Pull fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Native Pull fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .args(["-c", "core.hooksPath=", "-c", "commit.gpgSign=false"])
        .args(args)
        .output()
        .expect("native Git fixture");
    assert!(output.status.success(), "{args:?}: {output:?}");
    output.stdout
}

// Fixture-only config overrides travel through the retained environment,
// including to recursive Git children; the production argv stays unchanged.
fn fixture_config(environment: &mut BTreeMap<OsString, OsString>, values: &[(&str, &str)]) {
    environment.insert("GIT_CONFIG_COUNT".into(), values.len().to_string().into());
    for (index, (key, value)) in values.iter().enumerate() {
        environment.insert(format!("GIT_CONFIG_KEY_{index}").into(), (*key).into());
        environment.insert(format!("GIT_CONFIG_VALUE_{index}").into(), (*value).into());
    }
}

async fn native_pull_preserves_configured_integration(conflict: bool) {
    let root = tempfile::tempdir().expect("fixture");
    let source = root.path().join("source");
    let local = root.path().join("local");
    std::fs::create_dir(&source).expect("source");
    git(&source, &["init", "-b", "main"]);
    std::fs::write(source.join("file"), "base\n").expect("base");
    git(&source, &["add", "file"]);
    git(&source, &["commit", "-m", "base"]);
    git(
        root.path(),
        &[
            "clone",
            "--origin",
            "upstream",
            source.to_str().expect("source path"),
            local.to_str().expect("local path"),
        ],
    );
    git(
        &local,
        &[
            "config",
            "pull.rebase",
            if conflict { "false" } else { "true" },
        ],
    );
    let local_path = if conflict { "file" } else { "local-file" };
    std::fs::write(local.join(local_path), "local\n").expect("local change");
    git(&local, &["add", local_path]);
    git(&local, &["commit", "-m", "local"]);
    let before = git(&local, &["rev-parse", "HEAD"]);
    std::fs::write(source.join("file"), "remote\n").expect("remote change");
    git(&source, &["commit", "-am", "remote"]);
    let remote = git(&source, &["rev-parse", "HEAD"]);

    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("port");
    let port = listener.local_addr().expect("address").port();
    drop(listener);
    let log = tempfile::NamedTempFile::new().expect("daemon log");
    let mut daemon_command = tokio::process::Command::new("git");
    daemon_command
        .args([
            "daemon",
            "--reuseaddr",
            "--listen=127.0.0.1",
            "--export-all",
        ])
        .arg(format!("--port={port}"))
        .arg(format!("--base-path={}", daemon_fixture_path(root.path())))
        .arg(daemon_fixture_path(&source))
        .kill_on_drop(true)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(log.reopen().expect("log"));
    let mut daemon = native_process::NativeProcess::spawn(&mut daemon_command, &log)
        .await
        .expect("daemon");
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
                break;
            }
            assert!(daemon.is_running().expect("daemon state"));
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("daemon ready");
    let destination = format!("git://127.0.0.1:{port}/source");
    git(&local, &["remote", "set-url", "upstream", &destination]);
    // Keep the exact process environment captured by production preparation.
    // Fixture isolation belongs in repository config, before identity capture.
    for (key, value) in [
        ("user.name", "Native Pull fixture"),
        ("user.email", "fixture@example.invalid"),
        ("core.hooksPath", ""),
        ("commit.gpgSign", "false"),
    ] {
        git(&local, &["config", key, value]);
    }
    super::isolate_integration_fixture_filters(&local);
    let plan = prepare_pull_fixture(&local, &serde_json::json!({}), ToolProvenance::Native)
        .expect("native plan");
    assert!(!local.join(".git/FETCH_HEAD").exists());
    let output = execute_approved(plan).await;
    daemon.stop().await.expect("daemon cleanup");
    assert_eq!(output.result.is_error, conflict, "{:?}", output.result);
    assert_eq!(git(&local, &["rev-parse", "upstream/main"]), remote);
    if conflict {
        assert!(output.result.is_error, "{:?}", output.result);
        assert!(output.result.content.contains("partial effects may remain"));
        assert_eq!(git(&local, &["rev-parse", "HEAD"]), before);
        assert_eq!(git(&local, &["rev-parse", "MERGE_HEAD"]), remote);
        assert!(!git(&local, &["ls-files", "--unmerged"]).is_empty());
        assert!(
            std::fs::read_to_string(local.join("file"))
                .expect("conflict")
                .contains("<<<<<<<")
        );
    } else {
        assert!(!output.result.is_error, "{:?}", output.result);
        assert_ne!(git(&local, &["rev-parse", "HEAD"]), before);
        assert_eq!(git(&local, &["rev-parse", "HEAD^"]), remote);
        assert_eq!(
            std::fs::read(local.join("local-file")).expect("local file"),
            b"local\n"
        );
        assert_eq!(
            std::fs::read(local.join("file")).expect("remote file"),
            b"remote\n"
        );
    }
}

#[tokio::test]
async fn native_pull_owned_runner_uses_upstream_and_configured_rebase() {
    native_pull_preserves_configured_integration(false).await;
}

#[tokio::test]
async fn native_pull_owned_runner_retains_fetch_and_conflict_effects() {
    native_pull_preserves_configured_integration(true).await;
}
