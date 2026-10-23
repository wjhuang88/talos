#[cfg(unix)]
#[tokio::test]
async fn pull_fetch_refuses_executable_reference_transaction_hook() {
    use std::os::unix::fs::PermissionsExt;
    let root = fixture();
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .current_dir(root.path())
            .args(args)
            .output()
            .expect("fixture Git");
        assert!(output.status.success());
    };
    git(&["config", "remote.origin.url", "git://example.invalid/repo"]);
    let hook = root.path().join(".git/hooks/reference-transaction");
    std::fs::write(&hook, "#!/bin/sh\ntouch hook-ran\n").expect("hook");
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).expect("hook mode");
    let mut plan = Box::new(
        prepare_pull_fixture(root.path(), &serde_json::json!({}), ToolProvenance::Native)
            .expect("Pull"),
    )
    .discover()
    .await
    .expect("configuration");
    for _ in 0..8 {
        if plan.phase == Phase::PullFetch {
            break;
        }
        let admission = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("stage proof");
        plan = plan
            .discover_pull_prefetch(&admission)
            .await
            .expect("Pull discovery");
    }
    assert!(plan.phase == Phase::PullFetch);
    let admission = proof(&plan);
    let output = match plan.execute_stage(Box::new(admission)).await {
        PreparedNormalStage::Complete(output) => output,
        PreparedNormalStage::Continue(_) => panic!("fetch stage unexpectedly continued"),
    };
    assert!(output.result.is_error);
    assert!(output.result.content.contains("Executable Git Pull hooks"));
    assert!(!root.path().join("hook-ran").exists());
    assert!(!root.path().join(".git/FETCH_HEAD").exists());
}

#[tokio::test]
async fn pull_network_fetch_continuation_requires_fresh_admission() {
    let remote = fixture();
    let git = |root: &Path, args: &[&str]| {
        let output = std::process::Command::new("git")
            .current_dir(root)
            .args(args)
            .output()
            .expect("Git");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .expect("UTF8")
            .trim()
            .to_owned()
    };
    git(
        remote.path(),
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "--allow-empty",
            "-m",
            "initial",
        ],
    );
    let expected = git(remote.path(), &["rev-parse", "HEAD"]);
    let remote_branch = git(remote.path(), &["symbolic-ref", "HEAD"]);
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("port");
    let port = listener.local_addr().expect("address").port();
    drop(listener);
    let daemon_log = tempfile::NamedTempFile::new().expect("daemon log");
    let mut daemon_command = tokio::process::Command::new("git");
    daemon_command
        .args([
            "daemon",
            "--reuseaddr",
            "--listen=127.0.0.1",
            "--export-all",
        ])
        .arg(format!("--port={port}"))
        .arg(format!(
            "--base-path={}",
            daemon_fixture_path(remote.path().parent().expect("parent"))
        ))
        .arg(daemon_fixture_path(remote.path()))
        .kill_on_drop(true)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(daemon_log.reopen().expect("daemon log writer"));
    let mut daemon = native_process::NativeProcess::spawn(&mut daemon_command, &daemon_log)
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
    .expect("ready");
    let destination = format!(
        "git://127.0.0.1:{port}/{}",
        remote.path().file_name().expect("name").to_string_lossy()
    );
    for mode in ["allow", "deny", "replay"] {
        let local = fixture();
        isolate_integration_fixture_filters(local.path());
        git(local.path(), &["remote", "add", "origin", &destination]);
        let branch = git(local.path(), &["symbolic-ref", "--short", "HEAD"]);
        git(
            local.path(),
            &["config", &format!("branch.{branch}.remote"), "origin"],
        );
        git(
            local.path(),
            &["config", &format!("branch.{branch}.merge"), &remote_branch],
        );
        let mut plan = Box::new(
            prepare_pull_fixture(local.path(), &serde_json::json!({}), ToolProvenance::Native)
                .expect("plan"),
        )
        .discover()
        .await
        .expect("configuration");
        for _ in 0..8 {
            if plan.phase == Phase::PullFetch {
                break;
            }
            let tokens = proof(&plan)
                .consume_for_prepared(&plan.request(), &plan.identity)
                .expect("proof");
            plan = plan
                .discover_pull_prefetch(&tokens)
                .await
                .expect("prefetch");
        }
        assert!(plan.phase == Phase::PullFetch);
        let stale = proof(&plan);
        let admission = proof(&plan);
        let PreparedNormalStage::Continue(next) = plan.execute_stage(Box::new(admission)).await
        else {
            panic!("fetch failed");
        };
        assert!(
            next.permission_profile()
                .iter()
                .all(|facet| !matches!(facet.nature, ToolNature::Write | ToolNature::Network))
        );
        let fetched = std::fs::read(local.path().join(".git/FETCH_HEAD")).expect("fetched");
        assert!(String::from_utf8_lossy(&fetched).contains(&expected));
        let result = if mode == "deny" {
            next.execute_stage(Box::new(())).await
        } else if mode == "replay" {
            next.execute_stage(Box::new(stale)).await
        } else {
            let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(
                local.path().to_owned(),
            ));
            let context = PermissionContext::new(
                PermissionMode::Interactive,
                InteractionCapability::Available,
            );
            let input = serde_json::json!({});
            let request = PermissionRequest::new(
                "git_pull",
                ToolProvenance::Native,
                next.permission_profile(),
                &input,
            );
            let proposal = state
                .propose(&request, &context, GrantScope::Once)
                .expect("proposal");
            let pending = state
                .approve_once(proposal, &request, &context)
                .expect("approve");
            let admission = state
                .try_admit_prepared_invocation(
                    pending,
                    &request,
                    &context,
                    next.plan_identity().expect("identity"),
                )
                .expect("admit");
            next.execute_stage(Box::new(admission)).await
        };
        match result {
            PreparedNormalStage::Continue(_) => assert_eq!(mode, "allow"),
            PreparedNormalStage::Complete(output) => {
                assert_ne!(mode, "allow", "{}", output.result.content);
                assert!(output.result.is_error);
                assert!(output.result.content.contains("proof"));
            }
        }
        assert_eq!(
            std::fs::read(local.path().join(".git/FETCH_HEAD")).expect("retained"),
            fetched
        );
        assert!(!local.path().join(".git/index").exists());
    }
    daemon.stop().await.expect("stop daemon process tree");
}

#[tokio::test]
async fn pull_failed_parent_fetch_reports_possible_partial_effects() {
    let root = fixture();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("listener");
    let remote = format!("git://{}/repo", listener.local_addr().expect("address"));
    let output = std::process::Command::new("git")
        .current_dir(root.path())
        .args(["config", "remote.origin.url", &remote])
        .output()
        .expect("configure remote");
    assert!(output.status.success());
    let mut plan = Box::new(
        prepare_pull_fixture(root.path(), &serde_json::json!({}), ToolProvenance::Native)
            .expect("Pull"),
    )
    .discover()
    .await
    .expect("configuration");
    while plan.phase != Phase::PullFetch {
        let admission = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("proof");
        plan = plan
            .discover_pull_prefetch(&admission)
            .await
            .expect("prefetch");
    }
    // A local fake peer closes the connection without advertising refs.
    // No external service or host Git daemon is required.
    listener.set_nonblocking(true).expect("nonblocking");
    let peer = tokio::spawn(async move {
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
        loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    drop(stream);
                    break;
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(
                        tokio::time::Instant::now() < deadline,
                        "fetch never connected"
                    );
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
                Err(error) => panic!("peer accept: {error}"),
            }
        }
    });
    let admission = proof(&plan);
    let PreparedNormalStage::Complete(output) = plan.execute_stage(Box::new(admission)).await
    else {
        panic!("failed fetch continued");
    };
    peer.await.expect("peer");
    assert!(output.result.is_error);
    assert!(output.result.content.contains("Pull parent fetch failed"));
    assert!(output.result.content.contains("effects may be partial"));
}

#[tokio::test]
async fn pull_unsupported_transport_is_shown_but_never_dispatched() {
    let root = fixture();
    let output = std::process::Command::new("git")
        .current_dir(root.path())
        .args([
            "config",
            "remote.origin.url",
            "https://example.invalid/repo",
        ])
        .output()
        .expect("fixture Git");
    assert!(output.status.success());
    let mut plan = Box::new(
        prepare_pull_fixture(root.path(), &serde_json::json!({}), ToolProvenance::Native)
            .expect("Pull"),
    )
    .discover()
    .await
    .expect("configuration");
    for _ in 0..8 {
        if plan.phase == Phase::PullFetch {
            break;
        }
        let admission = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("stage proof");
        plan = plan
            .discover_pull_prefetch(&admission)
            .await
            .expect("Pull discovery");
    }
    assert!(plan.phase == Phase::PullFetch);
    assert!(plan.facets.iter().any(|facet| {
        facet.nature == ToolNature::Network
            && facet.resource.as_deref() == Some("https://example.invalid/repo")
    }));
    let admission = proof(&plan);
    let output = match plan.execute_stage(Box::new(admission)).await {
        PreparedNormalStage::Complete(output) => output,
        PreparedNormalStage::Continue(_) => panic!("fetch stage unexpectedly continued"),
    };
    assert!(output.result.is_error);
    assert!(output.result.content.contains("transport"));
    assert!(!root.path().join(".git/FETCH_HEAD").exists());
}

#[tokio::test]
async fn pull_legacy_destinations_require_command_authority_or_strict_route_refusal() {
    for directory in ["remotes", "branches"] {
        for after_planning in [false, true] {
            let root = fixture();
            let input = serde_json::json!({"remote":"origin"});
            let mut plan = Box::new(
                prepare_pull_fixture(root.path(), &input, ToolProvenance::Native)
                    .expect("Pull plan"),
            );
            if after_planning {
                plan = plan.discover().await.expect("configuration");
                while plan.phase != Phase::PullFetch {
                    let admission = proof(&plan)
                        .consume_for_prepared(&plan.request(), &plan.identity)
                        .expect("discovery admission");
                    plan = plan
                        .discover_pull_prefetch(&admission)
                        .await
                        .expect("fetch planning");
                }
            }
            let admission = proof(&plan);
            let legacy = root.path().join(".git").join(directory);
            std::fs::create_dir_all(&legacy).expect("legacy directory");
            std::fs::write(legacy.join("origin"), "URL: ext::unapproved-command\n")
                .expect("legacy destination");
            let stage = plan.execute_stage(Box::new(admission)).await;
            if !after_planning {
                let PreparedNormalStage::Continue(plan) = stage else {
                    panic!("bounded configuration discovery should publish command authority")
                };
                assert!(plan.permission_profile().iter().any(|facet| facet.resource.as_deref() == Some("git:pull-command")));
                let PreparedNormalStage::Complete(output) = plan.execute_stage(Box::new(())).await else {
                    panic!("unapproved command must refuse")
                };
                assert!(output.result.is_error);
                assert!(!root.path().join(".git/FETCH_HEAD").exists());
                continue;
            }
            let PreparedNormalStage::Complete(output) = stage else {
                panic!("strict legacy destination continued");
            };
            assert!(output.result.is_error, "{}", output.result.content);
            assert!(
                output.result.content.contains("Legacy Git remote"),
                "{}",
                output.result.content
            );
            assert!(!root.path().join(".git/FETCH_HEAD").exists());
            assert!(!root.path().join(".git/index").exists());
        }
    }
}

#[tokio::test]
async fn pull_fetch_requires_every_metadata_write_admission_before_dispatch() {
    for missing in ["gitdir", "common"] {
        let root = fixture();
        let input = serde_json::json!({});
        let mut plan = Box::new(
            prepare_pull_fixture(root.path(), &input, ToolProvenance::Native).expect("Pull"),
        )
        .discover()
        .await
        .expect("configuration");
        for _ in 0..8 {
            if plan.phase == Phase::PullFetch {
                break;
            }
            let admission = proof(&plan)
                .consume_for_prepared(&plan.request(), &plan.identity)
                .expect("stage proof");
            plan = plan
                .discover_pull_prefetch(&admission)
                .await
                .expect("Pull discovery");
        }
        assert!(plan.phase == Phase::PullFetch);
        let missing_resource = match missing {
            "gitdir" => plan.repository.git_dir.to_str().expect("gitdir"),
            "common" => plan.repository.common_dir.to_str().expect("common dir"),
            _ => "git:pull-repository-metadata",
        };
        plan.facets.retain(|facet| {
            !(facet.nature == ToolNature::Write
                && facet.resource.as_deref() == Some(missing_resource))
        });
        let before = RepositoryState::capture(root.path()).expect("before");
        let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(
            plan.workspace.clone(),
        ));
        let context = PermissionContext::new(
            PermissionMode::Interactive,
            InteractionCapability::Available,
        );
        let request = plan.request();
        assert!(
            state.propose(&request, &context, GrantScope::Once).is_err(),
            "missing {missing} must fail permission compilation"
        );
        assert_eq!(
            RepositoryState::capture(root.path()).expect("after"),
            before
        );
        assert!(!root.path().join(".git/FETCH_HEAD").exists());
    }
}
