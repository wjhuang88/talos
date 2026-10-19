#[cfg(unix)]
#[tokio::test]
async fn pull_prefetch_probe_uses_owned_runner_and_propagates_budget_failure() {
    let root = fixture();
    let plan = prepare(root.path());
    let run = |args: Vec<String>| {
        let mut command = tokio::process::Command::new(&plan.program);
        command
            .current_dir(root.path())
            .args(args)
            .env_clear()
            .envs(&plan.environment);
        plan.run_command_output(command, std::time::Duration::from_secs(5), 1024)
    };
    super::super::pull_state::BeforeFetch::probe(b"", None, 40, run)
        .await
        .expect("real unborn repository probes");
    let mut calls = 0;
    let changing = |args: Vec<String>| {
        calls += 1;
        if calls == 3 {
            let output = std::process::Command::new("git")
                .current_dir(root.path())
                .args(["symbolic-ref", "HEAD", "refs/heads/changed-during-probe"])
                .output()
                .expect("fixture branch change");
            assert!(output.status.success());
        }
        let mut command = tokio::process::Command::new(&plan.program);
        command
            .current_dir(root.path())
            .args(args)
            .env_clear()
            .envs(&plan.environment);
        plan.run_command_output(command, std::time::Duration::from_secs(5), 1024)
    };
    let changed = super::super::pull_state::BeforeFetch::probe(b"", None, 40, changing)
        .await
        .expect_err("changing HEAD cannot be retained");
    assert!(changed.to_string().contains("HEAD changed"));
    plan.command_budget.lock().expect("budget").commands = 0;
    let run = |args: Vec<String>| {
        let mut command = tokio::process::Command::new(&plan.program);
        command
            .current_dir(root.path())
            .args(args)
            .env_clear()
            .envs(&plan.environment);
        plan.run_command_output(command, std::time::Duration::from_secs(5), 1024)
    };
    let error = super::super::pull_state::BeforeFetch::probe(b"", None, 40, run)
        .await
        .expect_err("budget failure is not unborn HEAD");
    assert!(
        error
            .to_string()
            .contains("cumulative command/output budget")
    );
}

#[tokio::test]
async fn pull_configuration_stage_requires_fresh_admission_without_mutation() {
    let root = fixture();
    let input = serde_json::json!({});
    let prepare = || {
        AddPlan::prepare_pull(root.path(), &input, ToolProvenance::Native)
            .expect("Pull preparation")
    };
    let plan = prepare();
    assert_eq!(plan.operation.name(), "git_pull");
    assert!(
        plan.facets
            .iter()
            .all(|facet| facet.nature != ToolNature::Write)
    );
    assert!(
        plan.facets
            .iter()
            .any(|facet| { facet.resource.as_deref() == Some("git:configuration-chain") })
    );
    assert!(plan.facets.iter().any(|facet| {
        facet.nature == ToolNature::Read
            && facet.resource.as_deref() == Some("git:pull-repository-metadata")
    }));
    assert!(plan.facets.iter().all(|facet| {
        facet.resource.as_deref() != Some("git:pull-repository-metadata")
            || facet.nature == ToolNature::Read
    }));
    let before = RepositoryState::capture(root.path()).expect("before");
    let PreparedNormalStage::Complete(denied) = Box::new(plan).execute_stage(Box::new(())).await
    else {
        panic!("missing proof continued")
    };
    assert!(denied.result.is_error);
    let plan = prepare();
    let admitted = proof(&plan);
    let old_stage_proof = proof(&plan);
    let PreparedNormalStage::Continue(next) =
        Box::new(plan).execute_stage(Box::new(admitted)).await
    else {
        panic!("discovery failed")
    };
    assert!(
        next.permission_profile()
            .iter()
            .all(|facet| facet.nature != ToolNature::Write)
    );
    assert!(
        !next
            .permission_profile()
            .iter()
            .any(|facet| { facet.resource.as_deref() == Some("git:configuration-chain") })
    );
    let PreparedNormalStage::Complete(replayed) =
        next.execute_stage(Box::new(old_stage_proof)).await
    else {
        panic!("old proof continued")
    };
    assert!(replayed.result.is_error);
    assert_eq!(
        RepositoryState::capture(root.path()).expect("after"),
        before
    );
    assert!(!root.path().join(".git/FETCH_HEAD").exists());

    let next = Box::new(prepare()).discover().await.expect("configuration");
    let admitted = proof(&next);
    std::fs::OpenOptions::new()
        .append(true)
        .open(root.path().join(".git/config"))
        .and_then(|mut file| {
            use std::io::Write;
            file.write_all(b"\n[pull]\n\trebase = true\n")
        })
        .expect("change config");
    let PreparedNormalStage::Complete(changed) = next.execute_stage(Box::new(admitted)).await
    else {
        panic!("changed configuration continued")
    };
    assert!(changed.result.is_error);
    assert!(changed.result.content.contains("changed"));
    assert!(!root.path().join(".git/FETCH_HEAD").exists());
}

#[tokio::test]
async fn pull_prefetch_rebase_distinguishes_missing_empty_and_staged_index() {
    for state in ["missing", "empty", "staged"] {
        let root = fixture();
        let mut commands = vec![vec!["config", "pull.rebase", "true"]];
        if state != "missing" {
            commands.push(vec!["add", "file"]);
        }
        for args in commands {
            let output = std::process::Command::new("git")
                .current_dir(root.path())
                .args(args)
                .output()
                .expect("fixture Git");
            assert!(output.status.success());
        }
        if state == "empty" {
            let output = std::process::Command::new("git")
                .current_dir(root.path())
                .args(["read-tree", "--empty"])
                .output()
                .expect("empty index");
            assert!(output.status.success());
        }
        let plan =
            AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
                .expect("Pull");
        let plan = Box::new(plan).discover().await.expect("config");
        if state == "missing" {
            let plan = plan
                .inspect_pull_prefetch()
                .await
                .expect("unborn index accepted");
            assert!(plan.phase == Phase::PullFetch);
            assert!(!root.path().join(".git/index").exists());
            assert!(!root.path().join(".git/FETCH_HEAD").exists());
            continue;
        }
        let error = match plan.inspect_pull_prefetch().await {
            Ok(_) => panic!("staged unborn rebase accepted"),
            Err(error) => error,
        };
        assert!(
            error
                .to_string()
                .contains("unborn branch with staged changes")
        );
        assert!(!root.path().join(".git/FETCH_HEAD").exists());
    }
}

#[tokio::test]
async fn pull_prefetch_rejects_unmerged_index_without_fetch() {
    use std::io::Write;
    let root = fixture();
    let blob = std::process::Command::new("git")
        .current_dir(root.path())
        .args(["hash-object", "-w", "file"])
        .output()
        .expect("fixture blob");
    assert!(blob.status.success());
    let oid = String::from_utf8(blob.stdout).expect("oid");
    let mut index = std::process::Command::new("git")
        .current_dir(root.path())
        .args(["update-index", "--index-info"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("fixture index");
    index
        .stdin
        .take()
        .expect("index input")
        .write_all(format!("100644 {} 1\tfile\n", oid.trim()).as_bytes())
        .expect("conflict entry");
    assert!(index.wait().expect("index result").success());
    let plan = AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
        .expect("Pull");
    let plan = Box::new(plan).discover().await.expect("config");
    // Exercise production inspection with a real conflict-stage index;
    // exact frontier admission is separately covered by runner tests.
    let error = match plan.inspect_pull_prefetch().await {
        Ok(_) => panic!("unmerged index accepted"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("index conflicts"));
    assert!(!root.path().join(".git/FETCH_HEAD").exists());
}

#[tokio::test]
async fn pull_prefetch_retains_configured_remote_and_rejects_changed_reflog() {
    let root = fixture();
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .current_dir(root.path())
            .args(args)
            .output()
            .expect("fixture Git");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(&["symbolic-ref", "HEAD", "refs/heads/Topic"]);
    git(&["config", "user.name", "Pull Fixture"]);
    git(&["config", "user.email", "fixture@example.invalid"]);
    git(&["config", "commit.gpgSign", "false"]);
    git(&["add", "file"]);
    git(&["commit", "-m", "original"]);
    for (key, value) in [
        (
            "remote.Up.stream.url",
            "git://example.invalid/prefetch-must-not-contact",
        ),
        ("remote.Up.stream.fetch", "+refs/heads/*:refs/custom/*"),
        ("branch.Topic.remote", "Up.stream"),
        ("branch.Topic.merge", "refs/heads/main"),
        ("branch.Topic.rebase", "true"),
        ("fetch.recurseSubmodules", "on-demand"),
    ] {
        git(&["config", key, value]);
    }
    let plan = AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
        .expect("Pull");
    let plan = Box::new(plan).discover().await.expect("config");
    let admitted = proof(&plan);
    admitted
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("prefetch admission");
    let before = RepositoryState::capture(root.path()).expect("before");
    let plan = plan.inspect_pull_prefetch().await.expect("prefetch");
    assert!(plan.phase == Phase::PullFetch);
    assert_eq!(plan.pull_remote.as_deref(), Some("Up.stream"));
    assert!(plan.facets.iter().any(|facet| {
        facet.nature == ToolNature::Write
            && facet.resource.as_deref() == Some("git:pull-repository-metadata")
            && facet.description.as_deref().is_some_and(|description| {
                description.contains("policy: on-demand")
                    && description.contains("child effects require fresh admission")
            })
    }));
    assert_eq!(
        plan.pull_destinations,
        ["git://example.invalid/prefetch-must-not-contact"]
    );
    assert!(plan.facets.iter().any(|facet| {
        facet.nature == ToolNature::Network
            && facet.resource_kind == Some(ToolResourceKind::Remote)
            && facet.resource.as_deref() == Some("git://example.invalid/prefetch-must-not-contact")
    }));
    assert_eq!(
        plan.pull_before.as_ref().expect("retained").branch(),
        Some("Topic")
    );
    assert_eq!(
        plan.pull_destinations,
        ["git://example.invalid/prefetch-must-not-contact"]
    );
    assert!(plan.facets.iter().any(|facet| {
        facet.nature == ToolNature::Network
            && facet.resource_kind == Some(ToolResourceKind::Remote)
            && facet.resource.as_deref() == Some("git://example.invalid/prefetch-must-not-contact")
    }));
    assert!(plan.facets.iter().all(|facet| {
        facet.nature != ToolNature::Write
            || facet.resource.as_deref() == Some("git:pull-repository-metadata")
            || facet.resource.as_deref() == plan.repository.git_dir.to_str()
            || facet.resource.as_deref() == plan.repository.common_dir.to_str()
    }));
    assert_eq!(
        RepositoryState::capture(root.path()).expect("after"),
        before
    );
    assert!(!root.path().join(".git/FETCH_HEAD").exists());
    let admitted = proof(&plan);
    std::fs::write(root.path().join(".git/logs/HEAD"), b"changed reflog\n").expect("reflog");
    let PreparedNormalStage::Complete(changed) = plan.execute_stage(Box::new(admitted)).await
    else {
        panic!("changed reflog continued")
    };
    assert!(changed.result.is_error && changed.result.content.contains("evidence"));
    assert!(!root.path().join(".git/FETCH_HEAD").exists());
}

#[cfg(unix)]
#[tokio::test]
async fn pull_bundle_source_refuses_before_native_prefetch_side_effects() {
    let source = fixture();
    let native = fixture();
    let prepared = fixture();
    let run = |cwd: &Path, args: &[&str]| {
        std::process::Command::new("git")
            .current_dir(cwd)
            .args([
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "-c",
                "commit.gpgSign=false",
                "-c",
                "maintenance.auto=false",
            ])
            .args(args)
            .output()
            .expect("fixture Git")
    };
    let git = |cwd: &Path, args: &[&str]| {
        let output = run(cwd, args);
        assert!(output.status.success(), "{args:?}: {output:?}");
        output.stdout
    };
    git(source.path(), &["add", "file"]);
    git(source.path(), &["commit", "-m", "bundle-only object"]);
    let oid = String::from_utf8(git(source.path(), &["rev-parse", "HEAD"])).expect("object ID");
    let oid = oid.trim();
    let bundle = source.path().join("outside.bundle");
    let bundle_path = bundle.to_str().expect("bundle path");
    git(source.path(), &["bundle", "create", bundle_path, "--all"]);
    let bundle_bytes = std::fs::read(&bundle).expect("bundle");
    for root in [&native, &prepared] {
        git(root.path(), &["config", "fetch.bundleURI", bundle_path]);
        assert!(!run(root.path(), &["cat-file", "-e", oid]).status.success());
    }
    // Native fetch imports this extra source even when the selected remote
    // subsequently fails. Remote destination approval cannot cover it.
    let missing = source.path().join("absent-remote");
    assert!(
        !run(
            native.path(),
            &[
                "fetch",
                "--update-head-ok",
                "--",
                missing.to_str().expect("remote")
            ],
        )
        .status
        .success()
    );
    assert!(
        run(native.path(), &["cat-file", "-e", oid])
            .status
            .success()
    );

    let before = RepositoryState::capture(prepared.path()).expect("before");
    let plan = Box::new(
        AddPlan::prepare_pull(
            prepared.path(),
            &serde_json::json!({"remote": "origin"}),
            ToolProvenance::Native,
        )
        .expect("prepare"),
    );
    let admission = proof(&plan);
    let PreparedNormalStage::Complete(output) = plan.execute_stage(Box::new(admission)).await
    else {
        panic!("unadmitted bundle source continued to fetch planning");
    };
    assert!(output.result.is_error);
    assert!(output.result.content.contains("bundle URI"), "{output:?}");
    assert_eq!(
        RepositoryState::capture(prepared.path()).expect("after"),
        before
    );
    assert!(
        !run(prepared.path(), &["cat-file", "-e", oid])
            .status
            .success()
    );
    assert!(!prepared.path().join(".git/FETCH_HEAD").exists());
    assert_eq!(
        std::fs::read(&bundle).expect("source retained"),
        bundle_bytes
    );
}

#[tokio::test]
async fn pull_promisor_refuses_before_any_prefetch_probe() {
    let root = fixture();
    assert!(
        std::process::Command::new("git")
            .current_dir(root.path())
            .args(["config", "remote.origin.promisor", "true"])
            .status()
            .expect("config")
            .success()
    );
    let plan = AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
        .expect("Pull");
    let plan = Box::new(plan).discover().await.expect("config");
    assert!(
        plan.config
            .as_ref()
            .expect("config")
            .prepare_pull_inspection()
            .is_err()
    );
    let error = plan
        .inspect_pull_prefetch()
        .await
        .err()
        .expect("promisor refused");
    assert!(error.to_string().contains("promisor remote"));
    assert!(!root.path().join(".git/FETCH_HEAD").exists());
}
