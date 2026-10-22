//! Native whole-operation baselines for replacing the staged Pull scheduler.
//! Final-stage admission and owned-runner baselines, not discovery/activation.

use super::*;

#[test]
fn native_pull_plan_rejects_changed_operation_before_discovery() {
    let root = tempfile::tempdir().expect("fixture");
    git(root.path(), &["init", "-b", "main"]);
    let mut plan = AddPlan::prepare_pull(
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
async fn native_pull_stage_refuses_missing_or_stale_admission_and_missing_scope() {
    for scenario in [
        "missing-proof",
        "stale-proof",
        "missing-scope",
        "missing-root",
        "missing-network",
        "missing-execute",
    ] {
        let root = tempfile::tempdir().expect("fixture");
        git(root.path(), &["init", "-b", "main"]);
        let mut plan =
            AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
                .expect("plan");
        plan.phase = Phase::PullNativeMutation;
        if matches!(
            scenario,
            "missing-root" | "missing-network" | "missing-execute"
        ) {
            plan.pull_destinations = vec!["git://example.invalid/source".into()];
            plan.facets
                .extend(plan.native_pull_parent_facets().expect("parent scope"));
            if scenario == "missing-execute" {
                plan.facets
                    .retain(|facet| facet.nature != ToolNature::Execute);
            } else {
                let removed = if scenario == "missing-root" {
                    plan.facets.iter().position(|facet| {
                        facet.nature == ToolNature::Write
                            && facet.resource_kind == Some(ToolResourceKind::Path)
                            && facet.resource.as_deref() == plan.repository.workdir.to_str()
                    })
                } else {
                    plan.facets
                        .iter()
                        .position(|facet| facet.nature == ToolNature::Network)
                }
                .expect("required facet");
                plan.facets.remove(removed);
            }
        }
        if scenario == "missing-execute" {
            // Admission already refuses command-less profiles. Keep the
            // executor's own scope invariant defensive as well, before probes.
            let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(
                plan.workspace.clone(),
            ));
            let context = PermissionContext::new(
                PermissionMode::Interactive,
                InteractionCapability::Available,
            );
            assert!(
                state
                    .propose(&plan.request(), &context, GrantScope::Once)
                    .is_err()
            );
            let error = plan
                .execute_native_pull(&[])
                .await
                .expect_err("missing Execute");
            assert!(error.to_string().contains("scope capability missing"));
            assert!(!root.path().join(".git/FETCH_HEAD").exists());
            continue;
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
        let PreparedNormalStage::Complete(output) = Box::new(plan).execute_stage(admission).await
        else {
            panic!("denial must complete without discovery")
        };
        assert!(output.result.is_error, "{scenario}");
        let expected = match scenario {
            "missing-proof" => "requires its admitted proof",
            "stale-proof" => "does not authorize this plan",
            _ => "scope capability missing",
        };
        assert!(
            output.result.content.contains(expected),
            "{scenario}: {:?}",
            output.result
        );
        assert!(!root.path().join(".git/FETCH_HEAD").exists());
    }
}

fn git(cwd: &Path, args: &[&str]) -> Vec<u8> {
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
    let mut plan = AddPlan::prepare_pull(&local, &serde_json::json!({}), ToolProvenance::Native)
        .expect("native plan");
    let config_output = plan
        .inspect_config()
        .await
        .expect("fixture config inspection");
    plan.config = Some(
        super::super::super::config_state::ConfigState::capture_host(
            &config_output,
            &plan.repository,
            &plan.environment,
        )
        .expect("fixture configuration evidence"),
    );
    let (resolved_remote, resolved_destinations) = plan
        .resolve_native_pull_destination()
        .await
        .expect("native destination discovery");
    assert_eq!(resolved_remote, "upstream");
    assert_eq!(resolved_destinations, std::slice::from_ref(&destination));
    assert!(!local.join(".git/FETCH_HEAD").exists());
    plan.pull_remote = Some(resolved_remote);
    // Final-stage fixture: discovery is tested separately and remains a gate.
    plan.facets
        .retain(|facet| facet.nature == ToolNature::Execute);
    plan.pull_destinations = resolved_destinations.clone();
    plan.facets
        .extend(plan.native_pull_parent_facets().expect("parent scope"));
    // Even a separately approved-looking destination cannot substitute for
    // the remote resolved by native Pull's retained config and current branch.
    plan.pull_destinations = vec!["git://different.invalid/source".into()];
    assert!(plan.validate_native_pull_destination().await.is_err());
    assert!(!local.join(".git/FETCH_HEAD").exists());
    plan.pull_destinations = resolved_destinations;
    plan.pull_transport_supported = true;
    plan.phase = Phase::PullNativeMutation;
    plan.identity = PreparedPlanIdentity::new();
    let stale = proof(&plan);
    plan.identity = PreparedPlanIdentity::new();
    // A proof from an identical-looking earlier plan cannot authorize execution.
    let admitted = proof(&plan);
    assert!(
        stale
            .consume_for_prepared(&plan.request(), &plan.identity)
            .is_err()
    );
    let PreparedNormalStage::Complete(output) =
        Box::new(plan).execute_stage(Box::new(admitted)).await
    else {
        panic!("native Pull must complete in one execution stage")
    };
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

#[tokio::test]
async fn native_pull_owned_runner_preserves_configured_submodule_recursion() {
    let root = tempfile::tempdir().expect("fixture");
    let child = root.path().join("child");
    let parent = root.path().join("parent");
    let local = root.path().join("local");
    for path in [&child, &parent] {
        std::fs::create_dir(path).expect("repository");
        git(path, &["init", "-b", "main"]);
    }
    std::fs::write(child.join("file"), "before\n").expect("child file");
    git(&child, &["add", "file"]);
    git(&child, &["commit", "-m", "child base"]);
    git(
        &parent,
        &[
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            child.to_str().expect("child path"),
            "nested",
        ],
    );
    git(&parent, &["commit", "-am", "parent base"]);
    git(
        root.path(),
        &[
            "-c",
            "protocol.file.allow=always",
            "clone",
            "--recurse-submodules",
            parent.to_str().expect("parent path"),
            local.to_str().expect("local path"),
        ],
    );
    git(&local, &["config", "submodule.recurse", "true"]);
    std::fs::write(child.join("file"), "after\n").expect("child change");
    git(&child, &["commit", "-am", "child update"]);
    let expected = git(&child, &["rev-parse", "HEAD"]);
    git(&parent.join("nested"), &["fetch"]);
    git(&parent.join("nested"), &["checkout", "origin/main"]);
    git(&parent, &["commit", "-am", "parent update"]);

    let plan = prepare(&local);
    let mut environment = plan.environment.clone();
    environment.insert("GIT_CONFIG_NOSYSTEM".into(), "1".into());
    environment.insert(
        "GIT_CONFIG_GLOBAL".into(),
        root.path().join("absent-global-config").into_os_string(),
    );
    fixture_config(
        &mut environment,
        &[("core.hooksPath", ""), ("protocol.file.allow", "always")],
    );
    let frozen = command::FrozenCommand::pull(
        &plan.program,
        &local,
        &GitPullInput { remote: None },
        &environment,
    )
    .expect("freeze native Pull");
    let output = plan
        .run_command_output(
            frozen.command(),
            std::time::Duration::from_secs(20),
            1024 * 1024,
        )
        .await
        .expect("supervised recursive native Pull");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(git(&local.join("nested"), &["rev-parse", "HEAD"]), expected);
    assert_eq!(
        std::fs::read(local.join("nested/file")).expect("updated child"),
        b"after\n"
    );
    assert_eq!(
        git(&local, &["rev-parse", "HEAD"]),
        git(&parent, &["rev-parse", "HEAD"])
    );
}
