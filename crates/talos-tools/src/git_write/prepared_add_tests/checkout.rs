#[tokio::test]
async fn checkout_linked_worktree_admits_actual_metadata_and_preserves_main_head() {
    let root = fixture();
    let linked = tempfile::tempdir().expect("linked worktree");
    let git = |path: &Path, args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(path)
            .output()
            .expect("git");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .expect("utf8")
            .trim()
            .to_owned()
    };
    git(root.path(), &["config", "user.name", "Fixture"]);
    git(
        root.path(),
        &["config", "user.email", "fixture@example.invalid"],
    );
    git(root.path(), &["config", "commit.gpgSign", "false"]);
    git(root.path(), &["add", "file"]);
    git(root.path(), &["commit", "-m", "initial"]);
    let main_head = git(root.path(), &["symbolic-ref", "HEAD"]);
    git(root.path(), &["branch", "target"]);
    git(
        root.path(),
        &[
            "worktree",
            "add",
            "-b",
            "linked",
            linked.path().to_str().expect("path"),
        ],
    );
    let plan = AddPlan::prepare_checkout(
        linked.path(),
        &serde_json::json!({"branch":"target"}),
        ToolProvenance::Native,
    )
    .expect("plan");
    let discovered = Box::new(plan).discover().await.expect("discover");
    for path in [
        &discovered.repository.git_dir,
        &discovered.repository.common_dir,
    ] {
        assert!(
            discovered
                .facets
                .iter()
                .any(|facet| facet.nature == ToolNature::Write
                    && facet.resource.as_deref() == path.to_str())
        );
    }
    let plan = AddPlan::prepare_checkout(
        linked.path(),
        &serde_json::json!({"branch":"target"}),
        ToolProvenance::Native,
    )
    .expect("plan");
    let output = execute_approved(plan).await;
    assert!(!output.result.is_error, "{}", output.result.content);
    assert_eq!(git(linked.path(), &["branch", "--show-current"]), "target");
    assert_eq!(git(root.path(), &["symbolic-ref", "HEAD"]), main_head);
}

#[tokio::test]
#[cfg(unix)]
async fn checkout_existing_local_branch_preserves_onbranch_configuration() {
    use std::os::unix::fs::PermissionsExt;
    let root = fixture();
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(root.path())
            .output()
            .expect("git");
        assert!(output.status.success(), "{output:?}");
        String::from_utf8(output.stdout)
            .expect("utf8")
            .trim()
            .to_owned()
    };
    git(&["config", "user.name", "Fixture"]);
    git(&["config", "user.email", "fixture@example.invalid"]);
    git(&["config", "commit.gpgSign", "false"]);
    git(&["add", "file"]);
    git(&["commit", "-m", "initial"]);
    git(&["branch", "-m", "source"]);
    git(&["branch", "target"]);
    for branch in ["source", "target"] {
        let path = root.path().join(format!("{branch}.config"));
        std::fs::write(&path, format!("[fixture]\n marker = {branch}\n"))
            .expect("conditional config");
        git(&[
            "config",
            &format!("includeIf.onbranch:{branch}.path"),
            path.to_str().expect("path"),
        ]);
    }
    let index = std::fs::read(root.path().join(".git/index")).expect("index");
    for branch in ["target", "source"] {
        let plan = AddPlan::prepare_checkout(
            root.path(),
            &serde_json::json!({"branch":branch}),
            ToolProvenance::Native,
        )
        .expect("plan");
        let output = execute_approved(plan).await;
        assert!(!output.result.is_error, "{}", output.result.content);
        assert_eq!(git(&["branch", "--show-current"]), branch);
        assert_eq!(git(&["config", "fixture.marker"]), branch);
    }
    for (input, branch) in [
        ("-", "target"),
        ("@{-1}", "source"),
        ("@{-1}", "target"),
        ("-", "source"),
    ] {
        let plan = AddPlan::prepare_checkout(
            root.path(),
            &serde_json::json!({"branch":input}),
            ToolProvenance::Native,
        )
        .expect("previous branch plan");
        let output = execute_approved(plan).await;
        assert!(!output.result.is_error, "{}", output.result.content);
        assert_eq!(git(&["branch", "--show-current"]), branch);
        assert_eq!(git(&["config", "fixture.marker"]), branch);
    }
    for input in ["HEAD", "@"] {
        let head_plan = AddPlan::prepare_checkout(
            root.path(),
            &serde_json::json!({"branch":input}),
            ToolProvenance::Native,
        )
        .expect("attached HEAD plan");
        let head_output = execute_approved(head_plan).await;
        assert!(
            !head_output.result.is_error,
            "{}",
            head_output.result.content
        );
        assert_eq!(git(&["branch", "--show-current"]), "source");
        assert_eq!(git(&["config", "fixture.marker"]), "source");
    }
    let plan = AddPlan::prepare_checkout(
        root.path(),
        &serde_json::json!({"branch":"target"}),
        ToolProvenance::Native,
    )
    .expect("stale config plan");
    let discovered = Box::new(plan).discover().await.expect("discovery");
    let admission = proof(&discovered);
    std::fs::write(
        root.path().join("source.config"),
        "[fixture]\n marker = changed\n",
    )
    .expect("change active conditional config");
    let PreparedNormalStage::Complete(refused) =
        discovered.execute_stage(Box::new(admission)).await
    else {
        panic!("stale conditional config must refuse before continuation");
    };
    assert!(
        refused.result.is_error && refused.result.content.contains("changed"),
        "{}",
        refused.result.content
    );
    assert_eq!(git(&["branch", "--show-current"]), "source");
    assert_eq!(
        std::fs::read(root.path().join(".git/index")).expect("index"),
        index
    );
    std::fs::write(
        root.path().join("source.config"),
        "[fixture]\n marker = source\n",
    )
    .expect("restore active conditional config");
    for event in ["post-checkout", "reference-transaction"] {
        std::fs::write(
            root.path().join("source.config"),
            format!(
                "[hook \"conditional\"]\n event = {event}\n command = touch command-hook-ran\n"
            ),
        )
        .expect("active command hook");
        let plan = AddPlan::prepare_checkout(
            root.path(),
            &serde_json::json!({"branch":"target"}),
            ToolProvenance::Native,
        )
        .expect("command hook plan");
        let refused = execute_approved(plan).await;
        assert!(
            refused.result.is_error && refused.result.content.contains("hook commands"),
            "{}",
            refused.result.content
        );
        assert_eq!(git(&["branch", "--show-current"]), "source");
        assert!(!root.path().join("command-hook-ran").exists());
        assert_eq!(
            std::fs::read(root.path().join(".git/index")).expect("index"),
            index
        );
    }
    std::fs::write(
        root.path().join("source.config"),
        "[fixture]\n marker = source\n",
    )
    .expect("restore conditional config after command hooks");
    let hooks = root.path().join("target-hooks");
    std::fs::create_dir(&hooks).expect("hooks");
    let hook = hooks.join("post-checkout");
    std::fs::write(&hook, "#!/bin/sh\ntouch hook-ran\n").expect("hook");
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).expect("mode");
    std::fs::write(
        root.path().join("target.config"),
        format!(
            "[core]\n hooksPath = {}\n[fixture]\n marker = target\n",
            hooks.display()
        ),
    )
    .expect("target config");
    let create = AddPlan::prepare_checkout(
        root.path(),
        &serde_json::json!({"branch":"created", "create":true}),
        ToolProvenance::Native,
    )
    .expect("create plan");
    let refused = execute_approved(create).await;
    assert!(
        refused.result.is_error && refused.result.content.contains("target-branch discovery"),
        "{}",
        refused.result.content
    );
    assert_eq!(git(&["branch", "--show-current"]), "source");
    let plan = AddPlan::prepare_checkout(
        root.path(),
        &serde_json::json!({"branch":"target"}),
        ToolProvenance::Native,
    )
    .expect("plan");
    let output = execute_approved(plan).await;
    assert!(!output.result.is_error, "{}", output.result.content);
    assert!(!root.path().join("hook-ran").exists());
    assert_eq!(git(&["branch", "--show-current"]), "target");
    let plan = AddPlan::prepare_checkout(
        root.path(),
        &serde_json::json!({"branch":"source"}),
        ToolProvenance::Native,
    )
    .expect("plan");
    let refused = execute_approved(plan).await;
    assert!(
        refused.result.is_error && refused.result.content.contains("post-checkout"),
        "{}",
        refused.result.content
    );
    assert_eq!(git(&["branch", "--show-current"]), "target");
    assert!(!root.path().join("hook-ran").exists());
    assert_eq!(
        std::fs::read(root.path().join(".git/index")).expect("index"),
        index
    );
}

#[tokio::test]
async fn checkout_ref_movement_after_discovery_invalidates_plan() {
    let root = fixture();
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(root.path())
            .output()
            .expect("git");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .expect("utf8")
            .trim()
            .to_owned()
    };
    git(&["config", "user.name", "Fixture"]);
    git(&["config", "user.email", "fixture@example.invalid"]);
    git(&["config", "commit.gpgSign", "false"]);
    git(&["add", "file"]);
    git(&["commit", "-m", "initial"]);
    let initial = git(&["rev-parse", "HEAD"]);
    git(&["branch", "target"]);
    std::fs::write(root.path().join("file"), "second").expect("file");
    git(&["commit", "-am", "second"]);
    let current = git(&["rev-parse", "HEAD"]);
    assert_ne!(initial, current);
    let plan = AddPlan::prepare_checkout(
        root.path(),
        &serde_json::json!({"branch":"target"}),
        ToolProvenance::Native,
    )
    .expect("plan");
    let mut next: Box<dyn PreparedNormalToolInvocation> = Box::new(plan);
    let engine =
        PermissionSessionState::new(PermissionEngine::with_workspace_root(root.path().into()));
    let context = PermissionContext::new(
        PermissionMode::Interactive,
        InteractionCapability::Available,
    );
    let input = serde_json::json!({"branch":"target"});
    let mut history = talos_core::tool::PreparedStageHistory::default();
    let proof = loop {
        assert!(history.record_profile(
            next.plan_identity().expect("identity"),
            next.permission_profile(),
        ), "discovery stage budget");
        let request = PermissionRequest::native("git_checkout", next.permission_profile(), &input);
        let proposal = engine
            .propose(&request, &context, GrantScope::Once)
            .expect("proposal");
        let pending = engine
            .approve_once(proposal, &request, &context)
            .expect("approve");
        let admission = engine
            .try_admit_prepared_invocation(
                pending,
                &request,
                &context,
                next.plan_identity().expect("identity"),
            )
            .expect("admit");
        // Configuration frontiers differ by platform. Reach the actual
        // mutation plan before changing its retained target reference.
        if next.permission_profile().iter().any(|facet| facet.nature == ToolNature::Write) {
            break admission;
        }
        match next.execute_stage(Box::new(admission)).await {
            PreparedNormalStage::Continue(continuation) => next = continuation,
            PreparedNormalStage::Complete(output) => panic!("discovery: {}", output.result.content),
        }
    };
    git(&["update-ref", "refs/heads/target", &current]);
    let output = next.execute(Box::new(proof)).await;
    assert!(
        output.result.is_error && output.result.content.contains("changed"),
        "{}",
        output.result.content
    );
    assert_eq!(git(&["rev-parse", "HEAD"]), current);
    assert_ne!(git(&["branch", "--show-current"]), "target");
    assert_eq!(
        std::fs::read_to_string(root.path().join("file")).expect("file"),
        "second"
    );
}

#[tokio::test]
async fn checkout_protects_captured_target_only_config_from_tree_replacement() {
    let root = fixture();
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .current_dir(root.path())
            .args(args)
            .output()
            .expect("Git");
        assert!(output.status.success(), "{args:?}: {output:?}");
        String::from_utf8(output.stdout)
            .expect("UTF8")
            .trim()
            .to_owned()
    };
    git(&["config", "user.name", "Fixture"]);
    git(&["config", "user.email", "fixture@example.invalid"]);
    git(&["config", "commit.gpgSign", "false"]);
    git(&["add", "file"]);
    git(&["commit", "-qm", "base"]);
    let base = git(&["rev-parse", "HEAD"]);
    std::fs::write(root.path().join("target.conf"), b"[fixture]\nmarker=new\n").expect("config");
    git(&["add", "target.conf"]);
    git(&["commit", "-qm", "target"]);
    let target = git(&["rev-parse", "HEAD"]);
    git(&["checkout", "--detach", &base]);
    // The inactive include is absent now; the target tree would create it
    // before Git reloads the target configuration chain.
    let source = root.path().join("target.conf").canonicalize().err();
    assert!(source.is_some());
    let source = root
        .path()
        .canonicalize()
        .expect("root")
        .join("target.conf");
    let plan = AddPlan::prepare_checkout(
        root.path(),
        &serde_json::json!({"branch":target}),
        ToolProvenance::Native,
    )
    .expect("plan");
    let plan = Box::new(plan).discover().await.expect("discovery");
    let mut sources = super::super::target_config::Sources::new(Some(b"topic"), [source.clone()])
        .expect("sources");
    while !sources.complete() {
        sources
            .advance(|_| true, &plan.environment)
            .expect("capture absence");
    }
    let config = plan.config.as_ref().expect("config");
    let tree = plan.checkout_target.as_ref().expect("tree");
    config
        .validate_checkout_target(&plan.repository, tree, &[])
        .expect("ordinary added file");
    let protected = sources.captured_paths().cloned().collect::<Vec<_>>();
    assert!(
        config
            .validate_checkout_target(&plan.repository, tree, &protected)
            .is_err()
    );
    assert!(
        config
            .validate_checkout_deletions(&[source], &protected)
            .is_err()
    );
    assert_eq!(git(&["rev-parse", "HEAD"]), base);
    assert!(!root.path().join("target.conf").exists());
    // Exercise the same protection through actual target discovery and
    // fresh final admission, without injecting Sources into the plan.
    git(&["config", "remote.origin.url", "."]);
    git(&[
        "config",
        "remote.origin.fetch",
        "+refs/heads/*:refs/remotes/origin/*",
    ]);
    git(&["update-ref", "refs/remotes/origin/topic", &target]);
    git(&[
        "config",
        "includeIf.onbranch:topic.path",
        root.path()
            .canonicalize()
            .expect("canonical fixture")
            .join("target.conf")
            .to_str()
            .expect("source path"),
    ]);
    let before = std::fs::read(root.path().join(".git/index")).expect("index");
    let plan = AddPlan::prepare_checkout(
        root.path(),
        &serde_json::json!({"branch":"topic"}),
        ToolProvenance::Native,
    )
    .expect("guessed target plan");
    let output = execute_approved(plan).await;
    assert!(
        output.result.is_error,
        "target must not create its config source"
    );
    assert!(
        output
            .result
            .content
            .contains("Checkout rewrites an effective configuration or auxiliary source"),
        "{}",
        output.result.content
    );
    assert_eq!(git(&["rev-parse", "HEAD"]), base);
    assert!(!root.path().join("target.conf").exists());
    assert!(!root.path().join(".git/refs/heads/topic").exists());
    assert_eq!(
        std::fs::read(root.path().join(".git/index")).expect("index"),
        before
    );
}

#[tokio::test]
async fn checkout_target_tree_cannot_create_or_replace_late_hook() {
    for existing in [false, true] {
        let root = fixture();
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .current_dir(root.path())
                .args(args)
                .output()
                .expect("Git");
            assert!(output.status.success(), "{args:?}: {output:?}");
            String::from_utf8(output.stdout)
                .expect("UTF8")
                .trim()
                .to_owned()
        };
        git(&["config", "user.name", "Fixture"]);
        git(&["config", "user.email", "fixture@example.invalid"]);
        git(&["config", "commit.gpgSign", "false"]);
        let hook = root.path().join("late-hooks/post-checkout");
        if existing {
            std::fs::create_dir(root.path().join("late-hooks")).expect("parent");
            std::fs::write(&hook, b"retained non-executable hook\n").expect("source hook");
        }
        git(&["add", "."]);
        git(&["commit", "-qm", "base"]);
        let base = git(&["rev-parse", "HEAD"]);
        std::fs::create_dir_all(root.path().join("late-hooks")).expect("target parent");
        std::fs::write(&hook, b"#!/bin/sh\necho escaped > hook-marker\n").expect("target hook");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755))
                .expect("target executable mode");
        }
        git(&["add", "late-hooks/post-checkout"]);
        git(&["update-index", "--chmod=+x", "late-hooks/post-checkout"]);
        git(&["commit", "-qm", "target hook"]);
        let target = git(&["rev-parse", "HEAD"]);
        git(&["checkout", "--detach", &base]);
        let retained = std::fs::read(&hook).ok();
        git(&["config", "remote.origin.url", "."]);
        git(&[
            "config",
            "remote.origin.fetch",
            "+refs/heads/*:refs/remotes/origin/*",
        ]);
        git(&["update-ref", "refs/remotes/origin/topic", &target]);
        git(&["config", "includeIf.onbranch:topic.path", "target.conf"]);
        std::fs::write(
            root.path().join(".git/target.conf"),
            b"[core]\nhooksPath=late-hooks\n",
        )
        .expect("config");
        let index = std::fs::read(root.path().join(".git/index")).expect("index");
        let plan = AddPlan::prepare_checkout(
            root.path(),
            &serde_json::json!({"branch":"topic"}),
            ToolProvenance::Native,
        )
        .expect("plan");
        let output = execute_approved(plan).await;
        assert!(output.result.is_error, "target hook must be protected");
        assert!(
            output
                .result
                .content
                .contains("Checkout rewrites an effective configuration or auxiliary source"),
            "{}",
            output.result.content
        );
        assert_eq!(git(&["rev-parse", "HEAD"]), base);
        assert_eq!(
            std::fs::read(root.path().join(".git/index")).expect("index"),
            index
        );
        assert_eq!(std::fs::read(&hook).ok(), retained);
        assert!(!root.path().join(".git/refs/heads/topic").exists());
        assert!(!root.path().join("hook-marker").exists());
    }
}

#[tokio::test]
async fn checkout_target_configuration_limits_refuse_before_mutation() {
    for kind in ["bytes", "cumulative-bytes", "records", "paths", "expansion"] {
        let root = fixture();
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .current_dir(root.path())
                .args(args)
                .output()
                .expect("Git");
            assert!(output.status.success(), "{args:?}: {output:?}");
            output.stdout
        };
        git(&["config", "user.name", "Fixture"]);
        git(&["config", "user.email", "fixture@example.invalid"]);
        git(&["config", "commit.gpgSign", "false"]);
        git(&["add", "file"]);
        git(&["commit", "-qm", "initial"]);
        git(&["config", "includeIf.onbranch:topic.path", "target.conf"]);
        let payload = match kind {
            "bytes" => format!("#{}\n", "x".repeat(4 * 1024 * 1024)),
            "cumulative-bytes" => {
                std::fs::write(
                    root.path().join(".git/child.conf"),
                    format!("#{}\n", "x".repeat(2200 * 1024)),
                )
                .expect("second source below individual limit");
                format!("[include]\npath=child.conf\n#{}\n", "x".repeat(2200 * 1024))
            }
            "expansion" => {
                std::fs::write(
                    root.path().join(".git/child.conf"),
                    format!("[fixture]\nmarker={}\n", "x".repeat(200_000)),
                )
                .expect("shared child");
                format!("[include]\n{}", "path=child.conf\n".repeat(22))
            }
            "records" => format!("[fixture]\n{}", "marker=value\n".repeat(4097)),
            "paths" => format!(
                "[include]\n{}",
                (0..4096)
                    .map(|index| format!("path=missing-{index}\n"))
                    .collect::<String>()
            ),
            _ => unreachable!("fixture kind"),
        };
        let source = root.path().join(".git/target.conf");
        std::fs::write(&source, payload.as_bytes()).expect("target config");
        let head = std::fs::read(root.path().join(".git/HEAD")).expect("HEAD");
        let index = std::fs::read(root.path().join(".git/index")).expect("index");
        let worktree = std::fs::read(root.path().join("file")).expect("file");
        let plan = AddPlan::prepare_checkout(
            root.path(),
            &serde_json::json!({"branch":"topic", "create":true}),
            ToolProvenance::Native,
        )
        .expect("plan");
        let output = execute_approved(plan).await;
        assert!(output.result.is_error, "{kind}: {}", output.result.content);
        assert!(
            output.result.content.contains("target-branch discovery")
                && output
                    .result
                    .content
                    .contains("Target Git configuration cannot be parsed safely"),
            "{kind}: {}",
            output.result.content
        );
        assert_eq!(
            std::fs::read(&source).expect("config retained"),
            payload.as_bytes()
        );
        assert_eq!(
            std::fs::read(root.path().join(".git/HEAD")).expect("HEAD"),
            head
        );
        assert_eq!(
            std::fs::read(root.path().join(".git/index")).expect("index"),
            index
        );
        assert_eq!(
            std::fs::read(root.path().join("file")).expect("file"),
            worktree
        );
        assert!(!root.path().join(".git/refs/heads/topic").exists());
    }
}

#[tokio::test]
async fn checkout_target_only_configuration_drift_invalidates_retained_plan() {
    let root = fixture();
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .current_dir(root.path())
            .args(args)
            .output()
            .expect("Git");
        assert!(output.status.success(), "{args:?}: {output:?}");
        output.stdout
    };
    git(&["config", "user.name", "Fixture"]);
    git(&["config", "user.email", "fixture@example.invalid"]);
    git(&["config", "commit.gpgSign", "false"]);
    git(&["add", "file"]);
    git(&["commit", "-qm", "initial"]);
    git(&["config", "includeIf.onbranch:topic.path", "target.conf"]);
    let source = root.path().join(".git/target.conf");
    std::fs::write(&source, b"[fixture]\nmarker=before\n").expect("target source");
    let head = git(&["rev-parse", "HEAD"]);
    let index = std::fs::read(root.path().join(".git/index")).expect("index");
    let plan = AddPlan::prepare_checkout(
        root.path(),
        &serde_json::json!({"branch":"topic", "create":true}),
        ToolProvenance::Native,
    )
    .expect("plan");
    let mut plan = Box::new(plan).discover().await.expect("discovery");
    let sources = plan.target_config_sources.as_mut().expect("target chain");
    // Isolate the retained evidence guard from the unfinished final-effects
    // stage. This helper admits each exact frontier only inside the test.
    while !sources.complete() {
        sources
            .advance(|_| true, &plan.environment)
            .expect("capture");
    }
    plan.revalidate().expect("unchanged evidence");
    std::fs::write(&source, b"[fixture]\nmarker=after\n").expect("drift");
    let error = plan.revalidate().expect_err("target-only drift");
    assert!(
        error
            .to_string()
            .contains("Target Git configuration sources changed")
    );
    assert_eq!(git(&["rev-parse", "HEAD"]), head);
    assert_eq!(
        std::fs::read(root.path().join(".git/index")).expect("index"),
        index
    );
    assert!(!root.path().join(".git/refs/heads/topic").exists());
}

#[tokio::test]
async fn checkout_target_hooks_wait_for_all_exact_read_tokens() {
    for (missing_read, fresh_mutation, drift) in [
        (true, false, None),
        (false, false, None),
        (false, true, None),
        (false, true, Some("config")),
        (false, true, Some("hook")),
        (false, true, Some("ancestor")),
    ] {
        let root = fixture();
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .current_dir(root.path())
                .args(args)
                .output()
                .expect("Git");
            assert!(output.status.success(), "{args:?}: {output:?}");
            output.stdout
        };
        git(&["config", "user.name", "Fixture"]);
        git(&["config", "user.email", "fixture@example.invalid"]);
        git(&["config", "commit.gpgSign", "false"]);
        git(&["add", "file"]);
        git(&["commit", "-qm", "initial"]);
        std::fs::create_dir(root.path().join(".git/traversal"))
            .expect("cancelled traversal directory");
        git(&[
            "config",
            "includeIf.onbranch:topic.path",
            "traversal/../target.conf",
        ]);
        std::fs::write(
            root.path().join(".git/target.conf"),
            b"[core]\nhooksPath=late-hooks\n",
        )
        .expect("target config");
        // Keep the parent present so the hook case exercises leaf evidence,
        // independently of the already-covered absent-parent guard.
        std::fs::create_dir(root.path().join("late-hooks")).expect("hook parent");
        let head = git(&["rev-parse", "HEAD"]);
        let symbolic_head = std::fs::read(root.path().join(".git/HEAD")).expect("HEAD");
        let worktree = std::fs::read(root.path().join("file")).expect("file");
        let index = std::fs::read(root.path().join(".git/index")).expect("index");
        let plan = AddPlan::prepare_checkout(
            root.path(),
            &serde_json::json!({"branch":"topic", "create":true}),
            ToolProvenance::Native,
        )
        .expect("plan");
        let mut history = talos_core::tool::PreparedStageHistory::default();
        assert!(history.record_profile(&plan.identity, &plan.facets));
        let mut plan = Box::new(plan).discover().await.expect("discovery");
        while plan.target_hook_paths.is_none() {
            assert!(history.record_profile(&plan.identity, &plan.facets));
            assert!(plan.phase == Phase::TargetConfiguration);
            let tokens = proof(&plan)
                .consume_for_prepared(&plan.request(), &plan.identity)
                .expect("source admission");
            plan = plan
                .discover_target_configuration(&tokens)
                .expect("frontier");
        }
        assert!(history.record_profile(&plan.identity, &plan.facets));
        let paths = plan.target_hook_paths.as_ref().expect("hook frontier");
        let omitted = paths[1].clone();
        // A directory would fail regular-file capture if inspected. Missing any
        // exact token must fail admission first, before inspecting either leaf.
        if missing_read {
            std::fs::create_dir_all(paths[0].parent().expect("parent")).expect("hooks directory");
            std::fs::create_dir(&paths[0]).expect("invalid hook leaf");
        }
        let mut tokens = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("hook admission");
        if missing_read {
            tokens.retain(|token| token.normalized_path() != omitted);
        }
        let stale = proof(&plan);
        let result = plan.discover_target_configuration(&tokens);
        if missing_read {
            assert!(
                result
                    .err()
                    .expect("missing read")
                    .to_string()
                    .contains("exact Read admission before capture")
            );
        } else {
            let final_plan = result.expect("final mutation plan");
            assert!(history.record_profile(&final_plan.identity, &final_plan.facets));
            assert!(final_plan.phase == Phase::Mutation);
            assert!(!final_plan.target_hook_evidence.is_empty());
            assert!(
                final_plan
                    .facets
                    .iter()
                    .any(|facet| facet.nature == ToolNature::Write)
            );
            if fresh_mutation {
                let admission = proof(&final_plan);
                match drift {
                    Some("config") => std::fs::write(
                        root.path().join(".git/target.conf"),
                        b"[core]\nhooksPath=other-hooks\n",
                    )
                    .expect("config drift"),
                    Some("hook") => std::fs::write(
                        root.path().join("late-hooks/post-checkout"),
                        b"new hook bytes\n",
                    )
                    .expect("hook drift"),
                    Some("ancestor") => {
                        std::fs::rename(
                            root.path().join(".git/traversal"),
                            root.path().join(".git/traversal-old"),
                        )
                        .expect("retain original ancestor");
                        std::fs::create_dir(root.path().join(".git/traversal"))
                            .expect("replacement ancestor");
                    }
                    None => {}
                    _ => unreachable!("fixture case"),
                }
                let output = final_plan.execute(Box::new(admission)).await;
                assert_eq!(
                    output.result.is_error,
                    drift.is_some(),
                    "{}",
                    output.result.content
                );
                if let Some(kind) = drift {
                    assert!(
                        output.result.content.contains(if kind != "hook" {
                            "Target Git configuration sources changed"
                        } else {
                            "Target checkout hook evidence changed"
                        }),
                        "{}",
                        output.result.content
                    );
                } else {
                    assert_eq!(git(&["branch", "--show-current"]), b"topic\n");
                }
            } else {
                let denied = final_plan.execute(Box::new(stale)).await;
                assert!(
                    denied.result.is_error,
                    "old discovery proof must not authorize mutation"
                );
            }
        }
        assert_eq!(git(&["rev-parse", "HEAD"]), head);
        assert_eq!(
            std::fs::read(root.path().join("file")).expect("file"),
            worktree
        );
        if !fresh_mutation || drift.is_some() {
            assert_eq!(
                std::fs::read(root.path().join(".git/HEAD")).expect("HEAD"),
                symbolic_head
            );
        }
        assert_eq!(
            std::fs::read(root.path().join(".git/index")).expect("index"),
            index
        );
        assert_eq!(
            root.path().join(".git/refs/heads/topic").exists(),
            fresh_mutation && drift.is_none()
        );
    }
}

#[tokio::test]
async fn checkout_target_hook_missing_evidence_refuses_later_creation() {
    let root = fixture();
    for args in [
        vec!["add", "file"],
        vec![
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgSign=false",
            "commit",
            "-qm",
            "initial",
        ],
    ] {
        assert!(
            std::process::Command::new("git")
                .current_dir(root.path())
                .args(args)
                .output()
                .expect("Git")
                .status
                .success()
        );
    }
    let plan = AddPlan::prepare_checkout(
        root.path(),
        &serde_json::json!({"branch":"topic", "create":true}),
        ToolProvenance::Native,
    )
    .expect("plan");
    let mut plan = Box::new(plan).discover().await.expect("discovery");
    let parent = plan.repository.workdir.join("late-hooks");
    std::fs::create_dir(&parent).expect("hook directory");
    let path = parent.join("post-checkout");
    let mut sources = super::super::target_config::Sources::new(None, []).expect("sources");
    sources
        .add_directory_context(path.parent().expect("hook parent"))
        .expect("ancestor frontier");
    while !sources.complete() {
        sources
            .advance(|_| true, &plan.environment)
            .expect("test-only ancestor admission");
    }
    plan.target_config_sources = Some(sources);
    // Isolate the evidence guard while final mutation remains disabled.
    plan.target_hook_evidence =
        super::super::config_state::ConfigState::capture_checkout_reload_hooks(
            std::slice::from_ref(&path),
        )
        .expect("admitted missing evidence");
    plan.revalidate().expect("unchanged absence");
    std::fs::write(&path, b"created after discovery").expect("late file");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).expect("mode");
    }
    assert!(plan.revalidate().is_err());
    #[cfg(unix)]
    {
        std::fs::remove_file(&path).expect("fixture hook cleanup");
        std::fs::remove_dir(&parent).expect("fixture parent cleanup");
        let outside = tempfile::tempdir().expect("outside");
        std::fs::create_dir(outside.path().join("post-checkout")).expect("invalid external leaf");
        std::os::unix::fs::symlink(outside.path(), &parent).expect("redirect parent");
        let error = plan.revalidate().expect_err("ancestor substitution");
        assert!(
            error
                .to_string()
                .contains("Target Git configuration sources changed")
        );
    }
    assert!(!root.path().join(".git/refs/heads/topic").exists());
}

#[tokio::test]
async fn checkout_tracking_prediction_from_detached_head_matches_native() {
    for mode in ["true", "always", "inherit"] {
        let root = fixture();
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .current_dir(root.path())
                .args(args)
                .output()
                .expect("git");
            assert!(output.status.success(), "{args:?}: {output:?}");
            output.stdout
        };
        git(&["config", "user.name", "Fixture"]);
        git(&["config", "user.email", "fixture@example.invalid"]);
        git(&["config", "commit.gpgSign", "false"]);
        git(&["add", "file"]);
        git(&["commit", "-m", "initial"]);
        git(&["checkout", "--detach", "HEAD"]);
        git(&["config", "branch.autoSetupMerge", mode]);
        let plan = AddPlan::prepare_checkout(
            root.path(),
            &serde_json::json!({"branch":"topic", "create":true}),
            ToolProvenance::Native,
        )
        .expect("plan");
        let plan = Box::new(plan).discover().await.expect("discovery");
        let predicted = plan.checkout_tracking_writes.clone();
        let output = execute_approved(*plan).await;
        assert!(
            !output.result.is_error,
            "mode={mode}: {}",
            output.result.content
        );
        let config = git(&["config", "--local", "--null", "--list"]);
        let actual = super::super::config_state::parse_host(&config)
            .expect("config")
            .into_iter()
            .filter(|(key, _)| key.starts_with(b"branch.topic."))
            .collect::<std::collections::BTreeMap<_, _>>();
        let mut expected = std::collections::BTreeMap::<_, Vec<_>>::new();
        for record in predicted {
            expected.entry(record.key).or_default().push(record.value);
        }
        assert_eq!(actual, expected, "mode={mode}");
    }
}

#[tokio::test]
async fn checkout_tracking_prediction_matches_native_mode_matrix() {
    for guessed in [false, true] {
        for mode in ["false", "true", "always", "inherit", "simple"] {
            for rebase in ["never", "local", "remote", "always"] {
                let root = fixture();
                let git = |args: &[&str]| {
                    let output = std::process::Command::new("git")
                        .current_dir(root.path())
                        .args(args)
                        .output()
                        .expect("git");
                    assert!(output.status.success(), "{args:?}: {output:?}");
                    output.stdout
                };
                git(&["config", "user.name", "Fixture"]);
                git(&["config", "user.email", "fixture@example.invalid"]);
                git(&["config", "commit.gpgSign", "false"]);
                git(&["add", "file"]);
                git(&["commit", "-m", "initial"]);
                git(&["branch", "-m", "parent"]);
                git(&["config", "branch.autoSetupMerge", mode]);
                git(&["config", "branch.autoSetupRebase", rebase]);
                git(&["config", "branch.parent.remote", "."]);
                git(&["config", "branch.parent.merge", "refs/heads/base"]);
                if guessed {
                    git(&["config", "remote.origin.url", "/unused"]);
                    git(&[
                        "config",
                        "remote.origin.fetch",
                        "+refs/heads/*:refs/remotes/origin/*",
                    ]);
                    git(&["update-ref", "refs/remotes/origin/topic", "HEAD"]);
                }
                let plan = AddPlan::prepare_checkout(
                    root.path(),
                    &serde_json::json!({"branch":"topic", "create":!guessed}),
                    ToolProvenance::Native,
                )
                .expect("plan");
                let plan = Box::new(plan).discover().await.expect("discovery");
                let predicted = plan.checkout_tracking_writes.clone();
                let output = execute_approved(*plan).await;
                assert!(
                    !output.result.is_error,
                    "guessed={guessed}, mode={mode}, rebase={rebase}: {}",
                    output.result.content
                );
                let config = git(&["config", "--local", "--null", "--list"]);
                let actual = super::super::config_state::parse_host(&config)
                    .expect("config")
                    .into_iter()
                    .filter(|(key, _)| key.starts_with(b"branch.topic."))
                    .collect::<std::collections::BTreeMap<_, _>>();
                let mut expected = std::collections::BTreeMap::<_, Vec<_>>::new();
                for record in predicted {
                    expected.entry(record.key).or_default().push(record.value);
                }
                assert_eq!(
                    actual, expected,
                    "guessed={guessed}, mode={mode}, rebase={rebase}"
                );
            }
        }
    }
}

#[tokio::test]
async fn checkout_guess_binds_remote_and_reference_even_for_equal_commits() {
    let root = fixture();
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .current_dir(root.path())
            .args(args)
            .output()
            .expect("git");
        assert!(output.status.success(), "{output:?}");
        String::from_utf8(output.stdout)
            .expect("text")
            .trim()
            .to_owned()
    };
    git(&["config", "user.name", "Fixture"]);
    git(&["config", "user.email", "fixture@example.invalid"]);
    git(&["config", "commit.gpgSign", "false"]);
    git(&["add", "file"]);
    git(&["commit", "-m", "initial"]);
    let oid = git(&["rev-parse", "HEAD"]);
    for remote in ["first", "second"] {
        git(&["config", &format!("remote.{remote}.url"), "/unused"]);
        git(&[
            "config",
            &format!("remote.{remote}.fetch"),
            &format!("+refs/heads/*:refs/remotes/{remote}/*"),
        ]);
        git(&["update-ref", &format!("refs/remotes/{remote}/topic"), &oid]);
    }
    let mut retained = Vec::new();
    for remote in ["first", "second"] {
        git(&["config", "checkout.defaultRemote", remote]);
        let plan = AddPlan::prepare_checkout(
            root.path(),
            &serde_json::json!({"branch":"topic"}),
            ToolProvenance::Native,
        )
        .expect("plan");
        let plan = Box::new(plan).discover().await.expect("discovery");
        assert!(plan.checkout_guessed);
        assert_eq!(plan.checkout_new_branch.as_deref(), Some("topic"));
        assert_eq!(
            plan.checkout_tracking_source,
            Some((remote.to_owned(), format!("refs/remotes/{remote}/topic")))
        );
        assert_eq!(plan.checkout_target.as_ref().expect("target").commit, oid);
        retained.push(plan.checkout_tracking_source.clone());
    }
    assert_ne!(retained[0], retained[1]);
    assert!(git(&["branch", "--list", "topic"]).is_empty());

    git(&["config", "checkout.defaultRemote", "first"]);
    let plan = AddPlan::prepare_checkout(
        root.path(),
        &serde_json::json!({"branch":"topic"}),
        ToolProvenance::Native,
    )
    .expect("stale-source plan");
    let plan = Box::new(plan).discover().await.expect("discovery");
    let admission = proof(&plan);
    let branch = git(&["branch", "--show-current"]);
    let index = std::fs::read(root.path().join(".git/index")).expect("index");
    let head = std::fs::read(root.path().join(".git/HEAD")).expect("HEAD");
    git(&["config", "checkout.defaultRemote", "second"]);
    assert_eq!(git(&["rev-parse", "refs/remotes/first/topic"]), oid);
    assert_eq!(git(&["rev-parse", "refs/remotes/second/topic"]), oid);
    let PreparedNormalStage::Complete(output) = plan.execute_stage(Box::new(admission)).await
    else {
        panic!("changed remote selection must refuse before continuation");
    };
    assert!(
        output.result.is_error && output.result.content.contains("changed"),
        "{}",
        output.result.content
    );
    assert_eq!(git(&["branch", "--show-current"]), branch);
    assert!(git(&["branch", "--list", "topic"]).is_empty());
    assert_eq!(
        std::fs::read(root.path().join(".git/index")).expect("index"),
        index
    );
    assert_eq!(
        std::fs::read(root.path().join(".git/HEAD")).expect("HEAD"),
        head
    );
}

#[tokio::test]
async fn checkout_created_branch_binds_native_previous_name_expansion() {
    let root = fixture();
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .current_dir(root.path())
            .args(args)
            .output()
            .expect("git");
        assert!(output.status.success(), "{output:?}");
        String::from_utf8(output.stdout)
            .expect("text")
            .trim()
            .to_owned()
    };
    git(&["config", "user.name", "Fixture"]);
    git(&["config", "user.email", "fixture@example.invalid"]);
    git(&["config", "commit.gpgSign", "false"]);
    git(&["add", "file"]);
    git(&["commit", "-m", "initial"]);
    git(&["branch", "-m", "previous"]);
    git(&["checkout", "-b", "current"]);
    git(&["branch", "-D", "previous"]);
    let plan = AddPlan::prepare_checkout(
        root.path(),
        &serde_json::json!({"branch":"@{-1}", "create":true}),
        ToolProvenance::Native,
    )
    .expect("plan");
    let plan = Box::new(plan).discover().await.expect("discovery");
    assert_eq!(plan.checkout_new_branch.as_deref(), Some("previous"));
    assert!(!plan.checkout_guessed);
    let output = execute_approved(*plan).await;
    assert!(!output.result.is_error, "{}", output.result.content);
    assert_eq!(git(&["branch", "--show-current"]), "previous");
    let plan = AddPlan::prepare_checkout(
        root.path(),
        &serde_json::json!({"branch":"bad..name", "create":true}),
        ToolProvenance::Native,
    )
    .expect("lexical plan");
    assert!(Box::new(plan).discover().await.is_err());
    assert_eq!(git(&["branch", "--show-current"]), "previous");
}

#[tokio::test]
async fn checkout_same_oid_previous_branch_reflog_change_invalidates_plan() {
    let root = fixture();
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(root.path())
            .output()
            .expect("git");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .expect("utf8")
            .trim()
            .to_owned()
    };
    git(&["config", "user.name", "Fixture"]);
    git(&["config", "user.email", "fixture@example.invalid"]);
    git(&["config", "commit.gpgSign", "false"]);
    git(&["add", "file"]);
    git(&["commit", "-m", "initial"]);
    git(&["checkout", "-b", "first"]);
    git(&["branch", "other"]);
    git(&["checkout", "-b", "current"]);
    let conditional = root.path().join("current.config");
    std::fs::write(&conditional, "[fixture]\n marker = current\n").expect("config");
    git(&[
        "config",
        "includeIf.onbranch:current.path",
        conditional.to_str().expect("path"),
    ]);
    let input = serde_json::json!({"branch":"@{-1}"});
    let plan =
        AddPlan::prepare_checkout(root.path(), &input, ToolProvenance::Native).expect("plan");
    let plan = Box::new(plan).discover().await.expect("discovery");
    let oid = git(&["rev-parse", "@{-1}"]);
    let log_path = root.path().join(".git/logs/HEAD");
    let log = std::fs::read_to_string(&log_path).expect("reflog");
    assert!(log.contains("moving from first to current"));
    std::fs::write(
        &log_path,
        log.replace(
            "moving from first to current",
            "moving from other to current",
        ),
    )
    .expect("change previous branch");
    assert_eq!(
        git(&["rev-parse", "@{-1}"]),
        oid,
        "OID alone cannot detect this change"
    );
    let output = execute_approved(*plan).await;
    assert!(
        output.result.is_error && output.result.content.contains("changed"),
        "{}",
        output.result.content
    );
    assert_eq!(git(&["branch", "--show-current"]), "current");
}

#[tokio::test]
async fn checkout_target_executable_hook_refuses_before_mutation() {
    for hook_name in [".hooks/post-checkout", ".hooks/POST-CHECKOUT"] {
        let root = fixture();
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .args(args)
                .current_dir(root.path())
                .output()
                .expect("git");
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            String::from_utf8(output.stdout)
                .expect("utf8")
                .trim()
                .to_owned()
        };
        git(&["config", "user.name", "Fixture"]);
        git(&["config", "user.email", "fixture@example.invalid"]);
        git(&["config", "commit.gpgSign", "false"]);
        git(&["config", "core.hooksPath", ".hooks"]);
        git(&["add", "file"]);
        git(&["commit", "-m", "base"]);
        git(&["branch", "target"]);
        git(&["checkout", "-b", "hook-target"]);
        std::fs::create_dir(root.path().join(".hooks")).expect("hooks");
        std::fs::write(root.path().join(hook_name), "#!/bin/sh\ntouch marker\n").expect("hook");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                root.path().join(hook_name),
                std::fs::Permissions::from_mode(0o755),
            )
            .expect("mode");
        }
        git(&["add", hook_name]);
        git(&["commit", "-m", "hook"]);
        git(&["checkout", "target"]);
        let index_before = std::fs::read(root.path().join(".git/index")).expect("index");
        let plan = AddPlan::prepare_checkout(
            root.path(),
            &serde_json::json!({"branch":"hook-target"}),
            ToolProvenance::Native,
        )
        .expect("plan");
        let output = execute_approved(plan).await;
        assert!(
            output.result.is_error && output.result.content.contains("executable hook"),
            "{}",
            output.result.content
        );
        assert_eq!(git(&["branch", "--show-current"]), "target");
        assert!(!root.path().join("marker").exists());
        assert!(!root.path().join(hook_name).exists());
        assert_eq!(
            std::fs::read(root.path().join(".git/index")).expect("index"),
            index_before
        );
    }
}

#[tokio::test]
#[cfg(unix)]
async fn checkout_cannot_replace_effective_configuration_symlink_ancestor() {
    for source_path in ["../alias/config", "../alias/../config"] {
        let root = fixture();
        let external = tempfile::tempdir().expect("external configuration");
        let link_target = external.path().join("directory");
        std::fs::create_dir(&link_target).expect("target directory");
        std::fs::write(link_target.join("config"), "[core]\n filemode = true\n")
            .expect("nested config");
        std::fs::write(external.path().join("config"), "[core]\n filemode = true\n")
            .expect("config");
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .args(args)
                .current_dir(root.path())
                .output()
                .expect("git");
            assert!(output.status.success(), "{output:?}");
            String::from_utf8(output.stdout)
                .expect("utf8")
                .trim()
                .to_owned()
        };
        git(&["config", "user.name", "Fixture"]);
        git(&["config", "user.email", "fixture@example.invalid"]);
        git(&["config", "commit.gpgSign", "false"]);
        std::os::unix::fs::symlink(&link_target, root.path().join("alias")).expect("alias");
        git(&["add", "."]);
        git(&["commit", "-m", "alias"]);
        git(&["branch", "base"]);
        git(&["checkout", "-b", "replacement"]);
        std::fs::remove_file(root.path().join("alias")).expect("unlink fixture alias");
        std::fs::write(root.path().join("alias"), "replacement").expect("replacement");
        git(&["add", "alias"]);
        git(&["commit", "-m", "replace alias"]);
        git(&["checkout", "base"]);
        git(&["config", "include.path", source_path]);
        let index = std::fs::read(root.path().join(".git/index")).expect("index");
        let plan = AddPlan::prepare_checkout(
            root.path(),
            &serde_json::json!({"branch":"replacement"}),
            ToolProvenance::Native,
        )
        .expect("plan");
        let output = execute_approved(plan).await;
        assert!(
            output.result.is_error && output.result.content.contains("rewrites an effective"),
            "{}",
            output.result.content
        );
        assert_eq!(git(&["branch", "--show-current"]), "base");
        assert_eq!(
            std::fs::read_link(root.path().join("alias")).expect("alias retained"),
            link_target
        );
        assert_eq!(
            std::fs::read(root.path().join(".git/index")).expect("index"),
            index
        );
    }
}

#[tokio::test]
async fn checkout_refuses_to_rewrite_effective_configuration_and_attributes() {
    for (kind, empty_include) in [
        ("include.path", false),
        ("include.path", true),
        ("includeIf.onbranch:changed.path", false),
        ("includeIf.onbranch:changed.path", true),
        ("core.attributesFile", false),
    ] {
        let root = fixture();
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .args(args)
                .current_dir(root.path())
                .output()
                .expect("git");
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            String::from_utf8(output.stdout)
                .expect("utf8")
                .trim()
                .to_owned()
        };
        git(&["config", "user.name", "Fixture"]);
        git(&["config", "user.email", "fixture@example.invalid"]);
        git(&["config", "commit.gpgSign", "false"]);
        let (initial, changed) = if empty_include {
            ("", "[core]\n fsmonitor = touch marker\n")
        } else if kind != "core.attributesFile" {
            (
                "[core]\n filemode = true\n",
                "[core]\n fsmonitor = touch marker\n",
            )
        } else {
            ("* -filter\n", "* filter=evil\n")
        };
        std::fs::create_dir(root.path().join("settings")).expect("settings");
        let source = root.path().join("settings/rules");
        std::fs::write(&source, initial).expect("initial");
        git(&["add", "."]);
        git(&["commit", "-m", "base"]);
        git(&["branch", "base"]);
        git(&["checkout", "-b", "changed"]);
        std::fs::write(&source, changed).expect("changed");
        git(&["commit", "-am", "changed"]);
        git(&["checkout", "base"]);
        if kind.starts_with("includeIf.") {
            git(&["branch", "unchanged"]);
        }
        git(&["config", kind, source.to_str().expect("source path")]);
        git(&["config", "filter.evil.clean", "touch marker"]);
        let index = std::fs::read(root.path().join(".git/index")).expect("index");
        let plan = AddPlan::prepare_checkout(
            root.path(),
            &serde_json::json!({"branch":"changed"}),
            ToolProvenance::Native,
        )
        .expect("plan");
        let output = execute_approved(plan).await;
        assert!(
            output.result.is_error && output.result.content.contains("rewrites an effective"),
            "{kind}: {}",
            output.result.content
        );
        assert_eq!(git(&["branch", "--show-current"]), "base");
        assert_eq!(std::fs::read_to_string(&source).expect("source"), initial);
        assert_eq!(
            std::fs::read(root.path().join(".git/index")).expect("index"),
            index
        );
        assert!(!root.path().join("marker").exists());
        // An unchanged effective source in a tracked directory is safe;
        // tree directory entries alone must not be mistaken for rewrites.
        let plan = AddPlan::prepare_checkout(
            root.path(),
            &serde_json::json!({"branch":"unchanged", "create":!kind.starts_with("includeIf.")}),
            ToolProvenance::Native,
        )
        .expect("unchanged plan");
        let output = execute_approved(plan).await;
        assert!(!output.result.is_error, "{kind}: {}", output.result.content);
    }
}

#[tokio::test]
async fn checkout_file_directory_transitions_and_conflicting_edits() {
    let root = fixture();
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(root.path())
            .output()
            .expect("git");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .expect("utf8")
            .trim()
            .to_owned()
    };
    git(&["config", "user.name", "Fixture"]);
    git(&["config", "user.email", "fixture@example.invalid"]);
    git(&["config", "commit.gpgSign", "false"]);
    git(&["add", "file"]);
    git(&["commit", "-m", "file"]);
    git(&["branch", "plain"]);
    git(&["checkout", "-b", "directory"]);
    std::fs::remove_file(root.path().join("file")).expect("remove fixture file");
    std::fs::create_dir(root.path().join("file")).expect("directory");
    std::fs::write(root.path().join("file/nested"), "nested").expect("nested");
    git(&["add", "-A"]);
    git(&["commit", "-m", "directory"]);
    git(&["checkout", "plain"]);
    for branch in ["directory", "plain"] {
        let plan = AddPlan::prepare_checkout(
            root.path(),
            &serde_json::json!({"branch":branch}),
            ToolProvenance::Native,
        )
        .expect("plan");
        let output = execute_approved(plan).await;
        assert!(
            !output.result.is_error,
            "{branch}: {}",
            output.result.content
        );
        assert_eq!(git(&["branch", "--show-current"]), branch);
    }
    std::fs::write(root.path().join("file"), "uncommitted").expect("dirty file");
    let before = std::fs::read(root.path().join(".git/index")).expect("index");
    let plan = AddPlan::prepare_checkout(
        root.path(),
        &serde_json::json!({"branch":"directory"}),
        ToolProvenance::Native,
    )
    .expect("conflict plan");
    let output = execute_approved(plan).await;
    assert!(
        output.result.is_error,
        "conflicting edit must not be overwritten"
    );
    assert_eq!(git(&["branch", "--show-current"]), "plain");
    assert_eq!(
        std::fs::read_to_string(root.path().join("file")).expect("dirty file"),
        "uncommitted"
    );
    assert_eq!(
        std::fs::read(root.path().join(".git/index")).expect("index"),
        before
    );
}

#[cfg(unix)]
#[tokio::test]
async fn checkout_tracked_symlink_to_gitlink_preserves_referent_and_dirty_link() {
    for dangling in [false, true] {
        let root = fixture();
        let external = tempfile::tempdir().expect("external fixture");
        let referent = external.path().join("sentinel");
        if !dangling {
            std::fs::write(&referent, b"external sentinel").expect("sentinel");
        }
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .current_dir(root.path())
                .args(args)
                .output()
                .expect("git");
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            String::from_utf8(output.stdout)
                .expect("utf8")
                .trim()
                .to_owned()
        };
        git(&["config", "user.name", "Fixture"]);
        git(&["config", "user.email", "fixture@example.invalid"]);
        git(&["config", "submodule.recurse", "false"]);
        let link = root.path().join("link");
        std::os::unix::fs::symlink(&referent, &link).expect("tracked link");
        git(&["add", "link"]);
        git(&["commit", "-m", "base link"]);
        git(&["branch", "base"]);
        let oid = git(&["rev-parse", "HEAD"]);
        git(&["checkout", "-b", "linked"]);
        git(&["update-index", "--cacheinfo", "160000", &oid, "link"]);
        git(&["commit", "-m", "gitlink"]);
        git(&["checkout", "base"]);
        let index = std::fs::read(root.path().join(".git/index")).expect("index");
        let dirty = external.path().join("dirty-target");

        git(&["config", "core.excludesFile", "link"]);
        let plan = AddPlan::prepare_checkout(
            root.path(),
            &serde_json::json!({"branch":"linked"}),
            ToolProvenance::Native,
        )
        .expect("protected symlink plan");
        let result = execute_approved(plan).await;
        assert!(result.result.is_error, "protected auxiliary source");
        assert_eq!(std::fs::read_link(&link).expect("protected link"), referent);
        assert_eq!(git(&["branch", "--show-current"]), "base");
        assert_eq!(
            std::fs::read(root.path().join(".git/index")).expect("protected index"),
            index
        );
        git(&["config", "--unset", "core.excludesFile"]);

        // Both already-dirty and approval-time link changes must preserve
        // the link entry rather than follow or remove its external target.
        for late in [false, true] {
            if !late {
                std::fs::remove_file(&link).expect("remove fixture link");
                std::os::unix::fs::symlink(&dirty, &link).expect("dirty link");
            }
            let plan = AddPlan::prepare_checkout(
                root.path(),
                &serde_json::json!({"branch":"linked"}),
                ToolProvenance::Native,
            )
            .expect("symlink checkout plan");
            assert!(
                !plan.facets.iter().any(|facet| {
                    facet.resource_kind == Some(ToolResourceKind::Path)
                        && facet.resource.as_deref() == referent.to_str()
                }),
                "link target must not become a source permission"
            );
            let plan = Box::new(plan).discover().await.expect("symlink discovery");
            if late {
                std::fs::remove_file(&link).expect("remove fixture link");
                std::os::unix::fs::symlink(&dirty, &link).expect("late link change");
            }
            let admission = proof(&plan);
            let result = plan.execute(Box::new(admission)).await;
            assert!(result.result.is_error, "{}", result.result.content);
            assert_eq!(std::fs::read_link(&link).expect("retained link"), dirty);
            assert_eq!(git(&["branch", "--show-current"]), "base");
            assert_eq!(
                std::fs::read(root.path().join(".git/index")).expect("retained index"),
                index
            );
            std::fs::remove_file(&link).expect("remove fixture dirty link");
            std::os::unix::fs::symlink(&referent, &link).expect("restore fixture link");
        }

        let plan = AddPlan::prepare_checkout(
            root.path(),
            &serde_json::json!({"branch":"linked"}),
            ToolProvenance::Native,
        )
        .expect("clean link plan");
        let result = execute_approved(plan).await;
        assert!(!result.result.is_error, "{}", result.result.content);
        assert_eq!(git(&["branch", "--show-current"]), "linked");
        assert!(
            std::fs::symlink_metadata(&link)
                .expect("gitlink directory")
                .is_dir()
        );
        assert!(
            std::fs::read_dir(&link)
                .expect("uninitialized gitlink")
                .next()
                .is_none()
        );
        assert!(git(&["ls-files", "--stage", "link"]).starts_with("160000 "));
        if dangling {
            assert!(!referent.exists());
        } else {
            assert_eq!(
                std::fs::read(&referent).expect("external preserved"),
                b"external sentinel"
            );
        }
        assert!(!dirty.exists());
    }
}

#[test]
fn new_gitlink_directory_binds_names_without_reading_descendants() {
    let root = tempfile::tempdir().expect("fixture");
    let child = root.path().join("child");
    std::fs::create_dir(&child).expect("directory");
    std::fs::write(child.join("sentinel"), b"preserve").expect("sentinel");
    std::fs::create_dir_all(child.join("deep/nested")).expect("descendants");
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(root.path().join("absent"), child.join("external"))
            .expect("broken external link");
        let _socket = std::os::unix::net::UnixListener::bind(child.join("socket")).expect("socket");
    }
    let before = capture_new_gitlink_directory(&child, false).expect("ordinary descendants");
    std::fs::write(child.join("deep/nested/late"), b"not inspected").expect("deep change");
    assert_eq!(
        capture_new_gitlink_directory(&child, false).expect("names only"),
        before
    );
    for marker in [".git", ".GIT", "git~1"] {
        std::fs::write(child.join(marker), b"must not be opened").expect("marker");
        assert!(
            capture_new_gitlink_directory(&child, false).is_err(),
            "{marker}"
        );
        std::fs::remove_file(child.join(marker)).expect("remove marker");
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(root.path().join("absent"), child.join(".git"))
            .expect("broken marker");
        assert!(capture_new_gitlink_directory(&child, false).is_err());
        std::fs::remove_file(child.join(".git")).expect("remove marker");
    }
    std::fs::create_dir(child.join(".git")).expect("metadata directory");
    assert!(capture_new_gitlink_directory(&child, false).is_err());
    std::fs::remove_dir(child.join(".git")).expect("remove metadata");
    for index in 0..4096 {
        std::fs::write(child.join(format!("entry-{index}")), []).expect("budget entry");
    }
    assert!(capture_new_gitlink_directory(&child, false).is_err());

    let windows_component_is_valid = |name: &str| {
        std::panic::catch_unwind(|| {
            gix::validate::path::component(
                name.as_bytes().into(),
                None,
                gix::validate::path::component::Options {
                    protect_windows: true,
                    protect_hfs: true,
                    protect_ntfs: true,
                },
            )
            .is_ok()
        })
        .expect("path validation")
    };
    assert!(windows_component_is_valid("ordinary-name"));
    for unsafe_name in ["CON", "NUL.txt", "bad:name", "trailing.", "trailing "] {
        assert!(!windows_component_is_valid(unsafe_name), "{unsafe_name}");
    }
}

#[tokio::test]
async fn checkout_new_gitlink_preserves_initialized_child() {
    if cfg!(windows) {
        return;
    }
    for mode in [
        "native",
        "prepared",
        "native-external",
        "prepared-external",
        "filter-external",
        "native-linked",
        "prepared-linked",
        "ancestor-refused",
        "raw-denied",
    ] {
        let root = fixture();
        let git = |cwd: &Path, args: &[&str]| {
            let output = std::process::Command::new("git")
                .current_dir(cwd)
                .args(args)
                .output()
                .expect("Git fixture");
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
        git(root.path(), &["config", "user.name", "Fixture"]);
        git(
            root.path(),
            &["config", "user.email", "fixture@example.invalid"],
        );
        git(root.path(), &["config", "submodule.recurse", "false"]);
        git(root.path(), &["commit", "--allow-empty", "-m", "base"]);
        git(root.path(), &["branch", "base"]);
        let oid = git(root.path(), &["rev-parse", "HEAD"]);
        git(root.path(), &["checkout", "-b", "linked"]);
        git(
            root.path(),
            &[
                "update-index",
                "--add",
                "--cacheinfo",
                "160000",
                &oid,
                "child",
            ],
        );
        git(root.path(), &["commit", "-m", "gitlink"]);
        git(root.path(), &["checkout", "base"]);
        let linked_common = tempfile::tempdir().expect("linked common repository");
        if mode.ends_with("-linked") {
            git(linked_common.path(), &["init"]);
            git(linked_common.path(), &["config", "user.name", "Child"]);
            git(
                linked_common.path(),
                &["config", "user.email", "child@example.invalid"],
            );
            git(
                linked_common.path(),
                &["commit", "--allow-empty", "-m", "common"],
            );
            git(
                linked_common.path(),
                &[
                    "worktree",
                    "add",
                    "-b",
                    "child",
                    root.path().join("child").to_str().expect("child path"),
                ],
            );
        } else {
            git(root.path(), &["init", "child"]);
        }
        let child = root.path().join("child");
        git(&child, &["config", "user.name", "Child"]);
        git(&child, &["config", "user.email", "child@example.invalid"]);
        std::fs::write(child.join("tracked"), b"tracked-preserve").expect("tracked source");
        git(&child, &["add", "tracked"]);
        git(&child, &["commit", "--allow-empty", "-m", "child"]);
        std::fs::write(child.join("sentinel"), b"preserve").expect("sentinel");
        let child_gitdir = PathBuf::from(git(&child, &["rev-parse", "--absolute-git-dir"]));
        let head = std::fs::read(child_gitdir.join("HEAD")).expect("child HEAD");
        let external = tempfile::tempdir().expect("external worktree");
        if mode == "ancestor-refused" {
            git(
                &child,
                &[
                    "config",
                    "core.worktree",
                    root.path().to_str().expect("ancestor"),
                ],
            );
        }
        if mode.ends_with("-external") {
            std::fs::rename(child.join("tracked"), external.path().join("tracked"))
                .expect("relocate tracked worktree source");
            git(
                &child,
                &[
                    "config",
                    "core.worktree",
                    external.path().to_str().expect("path"),
                ],
            );
            std::fs::write(
                external.path().join("external-sentinel"),
                b"external-preserve",
            )
            .expect("external sentinel");
            if mode == "filter-external" {
                std::fs::write(
                    external.path().join(".gitattributes"),
                    "tracked filter=sentinel\n",
                )
                .expect("external attributes");
                git(
                    &child,
                    &[
                        "config",
                        "filter.sentinel.clean",
                        "touch filter-must-not-run",
                    ],
                );
            }
        }
        if mode == "raw-denied" {
            let outside = tempfile::tempdir().expect("raw external worktree");
            let external = outside.path().canonicalize().expect("canonical");
            git(
                &child,
                &["config", "core.worktree", external.to_str().expect("path")],
            );
            // An include hides the raw startup target in merged output.
            std::fs::write(
                child.join(".git/override.conf"),
                format!("[core]\nworktree={}\n", child.display()),
            )
            .expect("override");
            git(&child, &["config", "include.path", "override.conf"]);
            let plan = AddPlan::prepare_checkout(
                root.path(),
                &serde_json::json!({"branch":"linked"}),
                ToolProvenance::Native,
            )
            .expect("plan");
            let mut plan = Box::new(plan).discover().await.expect("parent discovery");
            let tokens = proof(&plan)
                .consume_for_prepared(&plan.request(), &plan.identity)
                .expect("names proof");
            plan = plan.discover_new_gitlinks(&tokens).expect("names");
            let tokens = proof(&plan)
                .consume_for_prepared(&plan.request(), &plan.identity)
                .expect("marker proof");
            plan = plan
                .discover_checkout_child_markers(&tokens)
                .expect("marker");
            let tokens = proof(&plan)
                .consume_for_prepared(&plan.request(), &plan.identity)
                .expect("metadata proof");
            plan = plan
                .discover_checkout_child_metadata(&tokens)
                .expect("metadata");
            let tokens = proof(&plan)
                .consume_for_prepared(&plan.request(), &plan.identity)
                .expect("config proof");
            plan = plan
                .discover_checkout_child_configuration(&tokens)
                .await
                .expect("bound bootstrap");
            assert!(
                plan.facets
                    .iter()
                    .any(|facet| facet.nature == ToolNature::Read
                        && facet.resource.as_deref() == external.to_str())
            );
            let mut tokens = proof(&plan)
                .consume_for_prepared(&plan.request(), &plan.identity)
                .expect("worktree proof");
            tokens.retain(|token| token.normalized_path() != external);
            let error = plan
                .discover_checkout_child_worktree(&tokens)
                .await
                .err()
                .expect("raw target denied");
            assert!(error.to_string().contains("Read"), "{error}");
            assert_eq!(git(root.path(), &["branch", "--show-current"]), "base");
            assert_eq!(
                std::fs::read(child.join("sentinel")).expect("sentinel"),
                b"preserve"
            );
            continue;
        } else if mode.starts_with("native") {
            git(root.path(), &["checkout", "linked"]);
        } else {
            let plan = AddPlan::prepare_checkout(
                root.path(),
                &serde_json::json!({"branch":"linked"}),
                ToolProvenance::Native,
            )
            .expect("plan");
            let output = execute_approved(plan).await;
            if mode == "ancestor-refused" {
                assert!(
                    output.result.is_error
                        && output
                            .result
                            .content
                            .contains("source snapshot limit exceeded"),
                    "{:?}",
                    output.result
                );
                assert_eq!(git(root.path(), &["branch", "--show-current"]), "base");
                assert_eq!(
                    std::fs::read(child.join("tracked")).expect("child source"),
                    b"tracked-preserve"
                );
                assert_eq!(
                    std::fs::read(child_gitdir.join("HEAD")).expect("HEAD"),
                    head
                );
                continue;
            }
            if mode == "filter-external" {
                assert!(
                    output.result.is_error && output.result.content.contains("configured filter"),
                    "{:?}",
                    output.result
                );
                assert_eq!(git(root.path(), &["branch", "--show-current"]), "base");
                assert!(!external.path().join("filter-must-not-run").exists());
                assert!(!child.join("filter-must-not-run").exists());
                assert_eq!(
                    std::fs::read(external.path().join("tracked")).expect("tracked source"),
                    b"tracked-preserve"
                );
                continue;
            }
            assert!(!output.result.is_error, "{:?}", output.result);
        }
        assert_eq!(git(root.path(), &["branch", "--show-current"]), "linked");
        assert_eq!(
            std::fs::read(child.join("sentinel")).expect("sentinel"),
            b"preserve"
        );
        assert_eq!(
            std::fs::read(child_gitdir.join("HEAD")).expect("HEAD"),
            head
        );
        if mode.ends_with("-external") {
            assert_eq!(
                std::fs::read(external.path().join("tracked")).expect("tracked source"),
                b"tracked-preserve"
            );
            assert_eq!(
                std::fs::read(external.path().join("external-sentinel"))
                    .expect("external sentinel"),
                b"external-preserve"
            );
        }
    }
}

#[tokio::test]
async fn checkout_new_gitlink_accepts_empty_directory_and_rejects_late_changes() {
    for change in [
        "native",
        "native-nonempty",
        "none",
        "late-marker",
        "child",
        "identity",
        "nonempty",
        "initialized",
        "missing-read",
        "old-proof",
        "symlink",
    ] {
        if change == "symlink" && !cfg!(unix) {
            continue;
        }
        let root = fixture();
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .current_dir(root.path())
                .args(args)
                .output()
                .expect("git");
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            String::from_utf8(output.stdout)
                .expect("utf8")
                .trim()
                .to_owned()
        };
        git(&["config", "user.name", "Fixture"]);
        git(&["config", "user.email", "fixture@example.invalid"]);
        git(&["config", "submodule.recurse", "false"]);
        git(&["commit", "--allow-empty", "-m", "base"]);
        git(&["branch", "base"]);
        let oid = git(&["rev-parse", "HEAD"]);
        git(&["checkout", "-b", "linked"]);
        git(&[
            "update-index",
            "--add",
            "--cacheinfo",
            "160000",
            &oid,
            "child",
        ]);
        git(&["commit", "-m", "gitlink"]);
        git(&["checkout", "base"]);
        let child = root.path().join("child");
        std::fs::create_dir_all(&child).expect("empty directory");
        let index = std::fs::read(root.path().join(".git/index")).expect("index");
        #[cfg(unix)]
        let mut preserved_socket = None;
        if matches!(change, "nonempty" | "native-nonempty") {
            for directory in ["objects", "refs", "deep/nested"] {
                std::fs::create_dir_all(child.join(directory)).expect("bare-shaped directory");
            }
            std::fs::write(child.join("HEAD"), b"ref: refs/heads/main\n").expect("bare HEAD");
            std::fs::write(
                child.join("config"),
                b"[core]\n bare = true\n fsmonitor = touch EXECUTED\n",
            )
            .expect("inert child config");
            #[cfg(unix)]
            {
                std::os::unix::fs::symlink("/unread-external-target", child.join("external"))
                    .expect("preserved link");
                preserved_socket = Some(
                    std::os::unix::net::UnixListener::bind(child.join("socket"))
                        .expect("preserved socket"),
                );
                assert!(
                    std::process::Command::new("mkfifo")
                        .arg(child.join("pipe"))
                        .status()
                        .expect("FIFO fixture")
                        .success()
                );
            }
        }
        if matches!(change, "native" | "native-nonempty") {
            if change == "native-nonempty" {
                std::fs::write(child.join("sentinel"), b"preserve").expect("native sentinel");
            }
            git(&["checkout", "linked"]);
        } else {
            if change == "nonempty" {
                std::fs::write(child.join("sentinel"), b"preserve").expect("child");
            } else if change == "initialized" {
                std::fs::create_dir(child.join(".git")).expect("child metadata");
            }
            let plan = AddPlan::prepare_checkout(
                root.path(),
                &serde_json::json!({"branch":"linked"}),
                ToolProvenance::Native,
            )
            .expect("plan");
            if change == "nonempty" {
                assert!(!execute_approved(plan).await.result.is_error);
            } else if change == "initialized" {
                assert!(execute_approved(plan).await.result.is_error);
            } else {
                let plan = Box::new(plan).discover().await.expect("discovery");
                assert!(plan.phase == Phase::CheckoutNewGitlinks);
                let old_proof = proof(&plan);
                let mut tokens = proof(&plan)
                    .consume_for_prepared(&plan.request(), &plan.identity)
                    .expect("admission");
                if change == "missing-read" {
                    tokens.retain(|token| {
                        !token.authorizes_path(
                            "git_checkout",
                            ToolNature::Read,
                            &plan.workspace,
                            child.to_str().expect("child path"),
                        )
                    });
                    std::fs::write(child.join("sentinel"), b"preserve").expect("unread child");
                    let error = match plan.discover_new_gitlinks(&tokens) {
                        Ok(_) => panic!("missing Read must refuse"),
                        Err(error) => error,
                    };
                    assert!(
                        error.to_string().contains("exact Read admission"),
                        "{error}"
                    );
                } else {
                    let plan = plan
                        .discover_new_gitlinks(&tokens)
                        .expect("empty directory");
                    assert!(plan.phase == Phase::Mutation);
                    assert!(plan.facets.iter().any(|facet| {
                        facet.nature == ToolNature::Read
                            && facet.resource.as_deref()
                                == plan.repository.workdir.join("child").to_str()
                    }));
                    if change == "old-proof" {
                        let PreparedNormalStage::Complete(output) =
                            plan.execute_stage(Box::new(old_proof)).await
                        else {
                            panic!("old discovery proof must not continue")
                        };
                        assert!(output.result.is_error);
                    } else {
                        let tokens = proof(&plan)
                            .consume_for_prepared(&plan.request(), &plan.identity)
                            .expect("mutation admission");
                        if change == "late-marker" {
                            std::fs::write(child.join(".git"), b"must not be read")
                                .expect("late marker");
                        } else if change == "child" {
                            std::fs::write(child.join("sentinel"), b"preserve")
                                .expect("late child");
                        } else if change == "identity" {
                            std::fs::rename(&child, root.path().join("retained-child"))
                                .expect("retain inode");
                            std::fs::create_dir(&child).expect("replacement");
                        } else if change == "symlink" {
                            #[cfg(unix)]
                            {
                                let retained = root.path().join("retained-child");
                                std::fs::rename(&child, &retained).expect("retain directory");
                                std::fs::write(retained.join("sentinel"), b"preserve")
                                    .expect("link referent");
                                std::os::unix::fs::symlink(&retained, &child)
                                    .expect("replacement link");
                            }
                        }
                        let result = plan.execute_admitted(&tokens).await;
                        assert_eq!(result.is_ok(), change == "none", "{change}: {result:?}");
                    }
                }
            }
        }
        if matches!(change, "native" | "native-nonempty" | "none" | "nonempty")
        {
            assert_eq!(git(&["branch", "--show-current"]), "linked");
            assert!(git(&["ls-files", "--stage", "child"]).starts_with("160000 "));
            if matches!(change, "nonempty" | "native-nonempty") {
                assert_eq!(
                    std::fs::read(child.join("sentinel")).expect("preserved"),
                    b"preserve"
                );
            } else {
                assert_eq!(std::fs::read_dir(&child).expect("child").count(), 0);
            }
        } else {
            assert_eq!(git(&["branch", "--show-current"]), "base");
            assert_eq!(
                std::fs::read(root.path().join(".git/index")).expect("index retained"),
                index
            );
            if matches!(change, "child" | "nonempty" | "symlink") {
                assert_eq!(
                    std::fs::read(child.join("sentinel")).expect("retained child"),
                    b"preserve"
                );
            }
        }
        if matches!(change, "nonempty" | "native-nonempty") {
            assert_eq!(
                std::fs::read(child.join("HEAD")).expect("preserved HEAD"),
                b"ref: refs/heads/main\n"
            );
            assert_eq!(
                std::fs::read(child.join("config")).expect("preserved config"),
                b"[core]\n bare = true\n fsmonitor = touch EXECUTED\n"
            );
            assert!(child.join("deep/nested").is_dir());
            assert!(!child.join("EXECUTED").exists());
            assert!(!root.path().join("EXECUTED").exists());
            #[cfg(unix)]
            {
                use std::os::unix::fs::FileTypeExt;
                assert_eq!(
                    std::fs::read_link(child.join("external")).expect("preserved link"),
                    PathBuf::from("/unread-external-target")
                );
                assert!(
                    std::fs::symlink_metadata(child.join("pipe"))
                        .expect("preserved FIFO")
                        .file_type()
                        .is_fifo()
                );
                let socket = preserved_socket.as_ref().expect("socket listener");
                socket.set_nonblocking(true).expect("nonblocking socket");
                assert_eq!(
                    socket.accept().expect_err("no socket connection").kind(),
                    std::io::ErrorKind::WouldBlock
                );
            }
        }
    }
}

#[tokio::test]
async fn checkout_tracked_file_to_gitlink_preserves_dirty_source() {
    let root = fixture();
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .current_dir(root.path())
            .args(args)
            .output()
            .expect("git");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .expect("utf8")
            .trim()
            .to_owned()
    };
    git(&["config", "user.name", "Fixture"]);
    git(&["config", "user.email", "fixture@example.invalid"]);
    git(&["config", "submodule.recurse", "false"]);
    std::fs::write(root.path().join("file"), b"[fixture]\n value = retained\n")
        .expect("config-compatible source");
    git(&["add", "file"]);
    git(&["commit", "-m", "base"]);
    git(&["branch", "base"]);
    let oid = git(&["rev-parse", "HEAD"]);
    let original = std::fs::read(root.path().join("file")).expect("original");
    git(&["checkout", "-b", "linked"]);
    git(&["update-index", "--cacheinfo", "160000", &oid, "file"]);
    git(&["commit", "-m", "link"]);
    git(&["checkout", "base"]);
    git(&["config", "include.path", "../file"]);
    let index_before = std::fs::read(root.path().join(".git/index")).expect("index");
    let plan = AddPlan::prepare_checkout(
        root.path(),
        &serde_json::json!({"branch":"linked"}),
        ToolProvenance::Native,
    )
    .expect("configuration-source plan");
    let output = execute_approved(plan).await;
    assert!(
        output.result.is_error,
        "configuration source must not be replaced"
    );
    assert!(
        output.result.content.contains("configuration")
            || output.result.content.contains("auxiliary"),
        "{}",
        output.result.content
    );
    assert_eq!(git(&["branch", "--show-current"]), "base");
    assert_eq!(
        std::fs::read(root.path().join("file")).expect("config preserved"),
        original
    );
    assert_eq!(
        std::fs::read(root.path().join(".git/index")).expect("index preserved"),
        index_before
    );
    git(&["config", "--unset", "include.path"]);
    for dirty in [true, false] {
        std::fs::write(
            root.path().join("file"),
            if dirty {
                b"dirty".as_slice()
            } else {
                &original
            },
        )
        .expect("source");
        let index = std::fs::read(root.path().join(".git/index")).expect("index");
        let plan = AddPlan::prepare_checkout(
            root.path(),
            &serde_json::json!({"branch":"linked"}),
            ToolProvenance::Native,
        )
        .expect("plan");
        let output = execute_approved(plan).await;
        assert_eq!(output.result.is_error, dirty, "{}", output.result.content);
        if dirty {
            assert_eq!(git(&["branch", "--show-current"]), "base");
            assert_eq!(
                std::fs::read(root.path().join("file")).expect("dirty preserved"),
                b"dirty"
            );
            assert_eq!(
                std::fs::read(root.path().join(".git/index")).expect("index preserved"),
                index
            );
        } else {
            assert_eq!(git(&["branch", "--show-current"]), "linked");
            assert!(root.path().join("file").is_dir());
            assert!(!root.path().join("file/.git").exists());
            assert!(git(&["ls-files", "--stage", "file"]).starts_with("160000 "));
        }
    }
    git(&["-C", "file", "init"]);
    git(&["-C", "file", "config", "user.name", "Fixture"]);
    git(&[
        "-C",
        "file",
        "config",
        "user.email",
        "fixture@example.invalid",
    ]);
    std::fs::write(root.path().join("file/local"), b"child content").expect("child file");
    git(&["-C", "file", "add", "local"]);
    git(&["-C", "file", "commit", "-m", "child"]);
    std::fs::write(root.path().join("file/local"), b"dirty child").expect("dirty child");
    let child_head = git(&["-C", "file", "rev-parse", "HEAD"]);
    let index_before = std::fs::read(root.path().join(".git/index")).expect("parent index");
    let plan = AddPlan::prepare_checkout(
        root.path(),
        &serde_json::json!({"branch":"base"}),
        ToolProvenance::Native,
    )
    .expect("initialized reverse plan");
    let output = execute_approved(plan).await;
    assert!(
        output.result.is_error,
        "initialized child deletion is not authorized"
    );
    assert!(
        output.result.content.contains("effective configuration"),
        "{}",
        output.result.content
    );
    assert_eq!(git(&["branch", "--show-current"]), "linked");
    assert_eq!(git(&["-C", "file", "rev-parse", "HEAD"]), child_head);
    assert_eq!(
        std::fs::read(root.path().join("file/local")).expect("preserved dirty child"),
        b"dirty child"
    );
    assert_eq!(
        std::fs::read(root.path().join(".git/index")).expect("preserved index"),
        index_before
    );
    std::fs::remove_dir_all(root.path().join("file")).expect("remove isolated fixture child");
    std::fs::create_dir(root.path().join("file")).expect("empty uninitialized directory");
    std::fs::write(root.path().join("file/untracked"), b"explicit deletion")
        .expect("uninitialized child content");
    // A guessed branch combines target-conditional discovery with the
    // deletion frontier. No Write may appear until both are complete.
    git(&["config", "remote.origin.url", "."]);
    git(&[
        "config",
        "remote.origin.fetch",
        "+refs/heads/*:refs/remotes/origin/*",
    ]);
    let base_oid = git(&["rev-parse", "base"]);
    git(&["update-ref", "refs/remotes/origin/topic", &base_oid]);
    git(&["config", "includeIf.onbranch:topic.path", "target.conf"]);
    std::fs::write(
        root.path().join(".git/target.conf"),
        b"[fixture]\nmarker=target\n",
    )
    .expect("target config");
    let plan = AddPlan::prepare_checkout(
        root.path(),
        &serde_json::json!({"branch":"topic"}),
        ToolProvenance::Native,
    )
    .expect("guessed deletion plan");
    let mut plan = Box::new(plan).discover().await.expect("guessed discovery");
    assert!(plan.phase == Phase::TargetConfiguration);
    while plan.phase == Phase::TargetConfiguration {
        assert!(
            !plan
                .facets
                .iter()
                .any(|facet| facet.nature == ToolNature::Write)
        );
        let tokens = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("target proof");
        plan = plan
            .discover_target_configuration(&tokens)
            .expect("target frontier");
    }
    assert!(plan.phase == Phase::CheckoutDeletion);
    while plan.phase == Phase::CheckoutDeletion {
        assert!(
            !plan
                .facets
                .iter()
                .any(|facet| facet.nature == ToolNature::Write)
        );
        let tokens = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("deletion proof");
        plan = plan
            .discover_checkout_deletion(&tokens)
            .expect("deletion frontier");
    }
    assert!(plan.phase == Phase::Mutation);
    let mut tokens = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("mutation proof");
    let child = plan.repository.workdir.join("file/untracked");
    tokens.retain(|token| {
        !token.authorizes_path(
            "git_checkout",
            ToolNature::Write,
            &plan.workspace,
            child.to_str().expect("child"),
        )
    });
    assert!(
        plan.execute_admitted(&tokens)
            .await
            .expect_err("missing child Write")
            .to_string()
            .contains("exact Write admission")
    );
    assert_eq!(git(&["branch", "--show-current"]), "linked");
    assert!(!root.path().join(".git/refs/heads/topic").exists());
    assert_eq!(
        std::fs::read(root.path().join("file/untracked")).expect("preserved"),
        b"explicit deletion"
    );
    #[cfg(unix)]
    let _socket = std::os::unix::net::UnixListener::bind(root.path().join("file/socket"))
        .expect("socket fixture");
    for denied_entry in [None, Some("untracked"), Some("pipe"), Some("socket")] {
        if !cfg!(unix) && matches!(denied_entry, Some("pipe" | "socket")) {
            continue;
        }
        let missing_write = denied_entry.is_some();
        #[cfg(unix)]
        if !root.path().join("file/pipe").exists() {
            assert!(
                std::process::Command::new("mkfifo")
                    .arg(root.path().join("file/pipe"))
                    .status()
                    .expect("FIFO fixture")
                    .success()
            );
        }
        let plan = AddPlan::prepare_checkout(
            root.path(),
            &serde_json::json!({"branch":"base"}),
            ToolProvenance::Native,
        )
        .expect("deletion refusal plan");
        let mut plan = Box::new(plan).discover().await.expect("deletion discovery");
        while plan.phase == Phase::CheckoutDeletion {
            let tokens = proof(&plan)
                .consume_for_prepared(&plan.request(), &plan.identity)
                .expect("frontier admission");
            plan = plan.discover_checkout_deletion(&tokens).expect("frontier");
        }
        let mut tokens = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("final admission");
        if missing_write {
            let child = plan
                .repository
                .workdir
                .join("file")
                .join(denied_entry.expect("denied child"));
            tokens.retain(|token| {
                !token.authorizes_path(
                    "git_checkout",
                    ToolNature::Write,
                    &plan.workspace,
                    child.to_str().expect("child path"),
                )
            });
        } else {
            std::fs::write(root.path().join("file/late"), b"preserve late child")
                .expect("late child");
        }
        let error = plan
            .execute_admitted(&tokens)
            .await
            .expect_err("denied deletion");
        if missing_write {
            assert!(
                error.to_string().contains("exact Write admission"),
                "{error}"
            );
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt;
            assert!(
                std::fs::symlink_metadata(root.path().join("file/pipe"))
                    .expect("FIFO retained")
                    .file_type()
                    .is_fifo()
            );
            assert!(
                std::fs::symlink_metadata(root.path().join("file/socket"))
                    .expect("socket retained")
                    .file_type()
                    .is_socket()
            );
        }
        assert_eq!(git(&["branch", "--show-current"]), "linked");
        assert_eq!(
            std::fs::read(root.path().join("file/untracked")).expect("preserved child"),
            b"explicit deletion"
        );
        assert_eq!(
            std::fs::read(root.path().join(".git/index")).expect("preserved index"),
            index_before
        );
        if !missing_write {
            assert_eq!(
                std::fs::read(root.path().join("file/late")).expect("preserved late child"),
                b"preserve late child"
            );
            std::fs::remove_file(root.path().join("file/late")).expect("remove fixture late child");
        }
    }
    let plan = AddPlan::prepare_checkout(
        root.path(),
        &serde_json::json!({"branch":"base"}),
        ToolProvenance::Native,
    )
    .expect("nonempty reverse plan");
    let mut plan = Box::new(plan).discover().await.expect("config discovery");
    while plan.phase == Phase::CheckoutDeletion {
        assert!(
            !plan
                .facets
                .iter()
                .any(|facet| facet.nature == ToolNature::Write)
        );
        let tokens = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("inventory proof");
        plan = plan
            .discover_checkout_deletion(&tokens)
            .expect("admitted frontier");
    }
    assert!(
        plan.facets
            .iter()
            .any(|facet| facet.nature == ToolNature::Write
                && facet.resource.as_deref()
                    == plan.repository.workdir.join("file/untracked").to_str())
    );
    let admission = proof(&plan);
    let result = plan.execute(Box::new(admission)).await;
    assert!(!result.result.is_error, "{}", result.result.content);
    assert_eq!(
        std::fs::read(root.path().join("file")).expect("restored file"),
        original
    );
    git(&["checkout", "linked"]);
    let plan = AddPlan::prepare_checkout(
        root.path(),
        &serde_json::json!({"branch":"base"}),
        ToolProvenance::Native,
    )
    .expect("empty reverse plan");
    let plan = Box::new(plan).discover().await.expect("empty discovery");
    std::fs::write(root.path().join("file/late"), b"preserve").expect("late child");
    let admission = proof(&plan);
    let output = plan.execute(Box::new(admission)).await;
    assert!(output.result.is_error);
    assert_eq!(
        std::fs::read(root.path().join("file/late")).expect("late preserved"),
        b"preserve"
    );
    assert_eq!(git(&["branch", "--show-current"]), "linked");
    std::fs::remove_file(root.path().join("file/late")).expect("remove fixture late child");

    #[cfg(unix)]
    {
        let plan = AddPlan::prepare_checkout(
            root.path(),
            &serde_json::json!({"branch":"base"}),
            ToolProvenance::Native,
        )
        .expect("replacement plan");
        let plan = Box::new(plan)
            .discover()
            .await
            .expect("replacement discovery");
        std::fs::rename(root.path().join("file"), root.path().join("saved-empty"))
            .expect("retain original directory inode");
        std::fs::create_dir(root.path().join("file")).expect("replacement empty directory");
        let admission = proof(&plan);
        let output = plan.execute(Box::new(admission)).await;
        assert!(output.result.is_error);
        assert!(
            output
                .result
                .content
                .contains("Empty submodule directory changed"),
            "{}",
            output.result.content
        );
        assert_eq!(git(&["branch", "--show-current"]), "linked");
        std::fs::remove_dir(root.path().join("file")).expect("remove fixture replacement");
        std::fs::rename(root.path().join("saved-empty"), root.path().join("file"))
            .expect("restore fixture directory");
    }

    let plan = AddPlan::prepare_checkout(
        root.path(),
        &serde_json::json!({"branch":"base"}),
        ToolProvenance::Native,
    )
    .expect("reverse plan");
    let output = execute_approved(plan).await;
    assert!(!output.result.is_error, "{}", output.result.content);
    assert_eq!(git(&["branch", "--show-current"]), "base");
    assert_eq!(
        std::fs::read(root.path().join("file")).expect("restored source"),
        original
    );
    git(&["checkout", "-b", "directory-target"]);
    git(&["rm", "file"]);
    std::fs::create_dir(root.path().join("file")).expect("target directory");
    std::fs::write(root.path().join("file/child"), b"directory child").expect("target child");
    git(&["add", "file/child"]);
    git(&["commit", "-m", "directory target"]);
    git(&["checkout", "linked"]);
    std::fs::remove_dir(root.path().join("file")).expect("empty gitlink directory");
    let plan = AddPlan::prepare_checkout(
        root.path(),
        &serde_json::json!({"branch":"directory-target"}),
        ToolProvenance::Native,
    )
    .expect("directory reverse plan");
    let output = execute_approved(plan).await;
    assert!(!output.result.is_error, "{}", output.result.content);
    assert_eq!(git(&["branch", "--show-current"]), "directory-target");
    assert_eq!(
        std::fs::read(root.path().join("file/child")).expect("restored child"),
        b"directory child"
    );
    assert!(!root.path().join("file/.git").exists());
}

#[tokio::test]
async fn checkout_adds_uninitialized_gitlink_and_removes_dirty_initialized_gitlink() {
    let root = fixture();
    let source = tempfile::tempdir().expect("child source");
    let git = |path: &Path, args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(path)
            .output()
            .expect("git");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .expect("utf8")
            .trim()
            .to_owned()
    };
    git(source.path(), &["init"]);
    for path in [root.path(), source.path()] {
        git(path, &["config", "user.name", "Fixture"]);
        git(path, &["config", "user.email", "fixture@example.invalid"]);
        git(path, &["config", "commit.gpgSign", "false"]);
    }
    std::fs::write(source.path().join("child-file"), "original").expect("child file");
    git(source.path(), &["add", "."]);
    git(source.path(), &["commit", "-m", "child"]);
    let child_head = git(source.path(), &["rev-parse", "HEAD"]);
    git(root.path(), &["add", "file"]);
    git(root.path(), &["commit", "-m", "base"]);
    git(root.path(), &["branch", "base"]);
    git(root.path(), &["checkout", "-b", "with-child"]);
    git(
        root.path(),
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            "160000",
            &child_head,
            "child",
        ],
    );
    git(root.path(), &["commit", "-m", "gitlink"]);
    git(root.path(), &["checkout", "base"]);
    assert!(!root.path().join("child").exists());
    let plan = AddPlan::prepare_checkout(
        root.path(),
        &serde_json::json!({"branch":"with-child"}),
        ToolProvenance::Native,
    )
    .expect("addition plan");
    let output = execute_approved(plan).await;
    assert!(!output.result.is_error, "{}", output.result.content);
    assert_eq!(
        git(root.path(), &["branch", "--show-current"]),
        "with-child"
    );
    assert!(!root.path().join("child/.git").exists());
    git(
        root.path(),
        &[
            "clone",
            source.path().to_str().expect("source path"),
            "child",
        ],
    );
    let child = root.path().join("child");
    std::fs::write(child.join("child-file"), "dirty").expect("dirty child");
    let plan = AddPlan::prepare_checkout(
        root.path(),
        &serde_json::json!({"branch":"base"}),
        ToolProvenance::Native,
    )
    .expect("removal plan");
    let output = execute_approved(plan).await;
    assert!(!output.result.is_error, "{}", output.result.content);
    assert_eq!(git(root.path(), &["branch", "--show-current"]), "base");
    assert!(git(root.path(), &["ls-files", "--", "child"]).is_empty());
    assert_eq!(git(&child, &["rev-parse", "HEAD"]), child_head);
    assert_eq!(
        std::fs::read_to_string(child.join("child-file")).expect("child preserved"),
        "dirty"
    );
}

#[tokio::test]
async fn checkout_nonrecursive_gitlink_preserves_dirty_child_worktree() {
    let root = fixture();
    let child = root.path().join("child");
    std::fs::create_dir(&child).expect("child");
    let git = |path: &Path, args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(path)
            .output()
            .expect("git");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .expect("utf8")
            .trim()
            .to_owned()
    };
    git(&child, &["init"]);
    for path in [root.path(), child.as_path()] {
        git(path, &["config", "user.name", "Fixture"]);
        git(path, &["config", "user.email", "fixture@example.invalid"]);
        git(path, &["config", "commit.gpgSign", "false"]);
    }
    std::fs::write(child.join("child-file"), "first").expect("child file");
    git(&child, &["add", "."]);
    git(&child, &["commit", "-m", "first"]);
    let first = git(&child, &["rev-parse", "HEAD"]);
    git(root.path(), &["add", "file"]);
    git(
        root.path(),
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            "160000",
            &first,
            "child",
        ],
    );
    git(root.path(), &["commit", "-m", "base"]);
    git(root.path(), &["branch", "base"]);
    git(root.path(), &["checkout", "-b", "target"]);
    std::fs::write(child.join("child-file"), "second").expect("child file");
    git(&child, &["commit", "-am", "second"]);
    let second = git(&child, &["rev-parse", "HEAD"]);
    git(
        root.path(),
        &["update-index", "--cacheinfo", "160000", &second, "child"],
    );
    git(root.path(), &["commit", "-m", "new gitlink"]);
    git(root.path(), &["checkout", "base"]);
    std::fs::write(child.join("child-file"), "dirty child").expect("dirty");
    let plan = AddPlan::prepare_checkout(
        root.path(),
        &serde_json::json!({"branch":"target"}),
        ToolProvenance::Native,
    )
    .expect("plan");
    let discovered = Box::new(
        AddPlan::prepare_checkout(
            root.path(),
            &serde_json::json!({"branch":"target"}),
            ToolProvenance::Native,
        )
        .expect("permission plan"),
    )
    .discover()
    .await
    .expect("discovery");
    assert!(discovered.facets.iter().any(|facet| {
        facet.nature == ToolNature::Write
            && facet.resource.as_deref()
                == child
                    .join(".git")
                    .canonicalize()
                    .expect("canonical child metadata")
                    .to_str()
    }));
    let output = execute_approved(plan).await;
    assert!(!output.result.is_error, "{}", output.result.content);
    assert_eq!(git(root.path(), &["branch", "--show-current"]), "target");
    assert_eq!(git(&child, &["rev-parse", "HEAD"]), second);
    assert_eq!(
        std::fs::read_to_string(child.join("child-file")).expect("dirty preserved"),
        "dirty child"
    );
    git(&child, &["config", "core.fsmonitor", "touch helper-ran"]);
    let index_before = std::fs::read(root.path().join(".git/index")).expect("index");
    let plan = AddPlan::prepare_checkout(
        root.path(),
        &serde_json::json!({"branch":"base"}),
        ToolProvenance::Native,
    )
    .expect("helper plan");
    let output = execute_approved(plan).await;
    assert!(
        output.result.is_error && output.result.content.contains("submodule fsmonitor"),
        "{}",
        output.result.content
    );
    assert_eq!(git(root.path(), &["branch", "--show-current"]), "target");
    assert_eq!(
        std::fs::read(root.path().join(".git/index")).expect("index"),
        index_before
    );
    assert!(!child.join("helper-ran").exists());
}

#[cfg(unix)]
#[tokio::test]
async fn checkout_post_hook_refuses_before_branch_change() {
    use std::os::unix::fs::PermissionsExt;
    let root = fixture();
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(root.path())
            .output()
            .expect("git");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        output.stdout
    };
    git(&["config", "user.name", "Fixture"]);
    git(&["config", "user.email", "fixture@example.invalid"]);
    git(&["config", "commit.gpgSign", "false"]);
    git(&["add", "file"]);
    git(&["commit", "-m", "base"]);
    git(&["branch", "target"]);
    let hook = root.path().join(".git/hooks/post-checkout");
    std::fs::write(&hook, "#!/bin/sh\nprintf forbidden > marker\n").expect("hook");
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).expect("executable");
    let branch = git(&["branch", "--show-current"]);
    let index = std::fs::read(root.path().join(".git/index")).expect("index");
    let plan = AddPlan::prepare_checkout(
        root.path(),
        &serde_json::json!({"branch":"target"}),
        ToolProvenance::Native,
    )
    .expect("plan");
    let output = execute_approved(plan).await;
    assert!(
        output.result.is_error && output.result.content.contains("post-checkout"),
        "{}",
        output.result.content
    );
    assert_eq!(git(&["branch", "--show-current"]), branch);
    assert_eq!(
        std::fs::read(root.path().join(".git/index")).expect("index"),
        index
    );
    assert!(!root.path().join("marker").exists());
}

#[tokio::test]
async fn checkout_index_only_filter_refuses_before_mutation() {
    let root = fixture();
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(root.path())
            .output()
            .expect("git");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        output.stdout
    };
    git(&["config", "user.name", "Fixture"]);
    git(&["config", "user.email", "fixture@example.invalid"]);
    git(&["config", "commit.gpgSign", "false"]);
    std::fs::write(root.path().join(".gitattributes"), "file -filter\n").expect("base rules");
    git(&["add", "."]);
    git(&["commit", "-m", "base"]);
    git(&["branch", "base"]);
    git(&["checkout", "-b", "target"]);
    std::fs::write(root.path().join("file"), "target").expect("file");
    git(&["commit", "-am", "target"]);
    git(&["checkout", "base"]);
    std::fs::write(root.path().join(".gitattributes"), "file filter=custom\n")
        .expect("index rules");
    git(&["add", ".gitattributes"]);
    std::fs::write(root.path().join(".gitattributes"), "file -filter\n").expect("worktree rules");
    git(&[
        "config",
        "filter.custom.smudge",
        "sh -c 'printf forbidden > marker; cat'",
    ]);
    assert!(
        String::from_utf8(git(&["check-attr", "--cached", "filter", "--", "file"]))
            .expect("cached")
            .contains("custom")
    );
    assert!(
        String::from_utf8(git(&["check-attr", "filter", "--", "file"]))
            .expect("current")
            .contains("unset")
    );
    let head = git(&["rev-parse", "HEAD"]);
    let index = std::fs::read(root.path().join(".git/index")).expect("index");
    let plan = AddPlan::prepare_checkout(
        root.path(),
        &serde_json::json!({"branch":"target"}),
        ToolProvenance::Native,
    )
    .expect("plan");
    let output = execute_approved(plan).await;
    assert!(
        output.result.is_error && output.result.content.contains("filter"),
        "{}",
        output.result.content
    );
    assert_eq!(git(&["rev-parse", "HEAD"]), head);
    assert_eq!(
        std::fs::read(root.path().join(".git/index")).expect("index"),
        index
    );
    assert_eq!(
        std::fs::read(root.path().join("file")).expect("file"),
        b"before"
    );
    assert!(!root.path().join("marker").exists());
}

#[tokio::test]
async fn checkout_target_filter_refuses_before_worktree_mutation() {
    let root = fixture();
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(root.path())
            .output()
            .expect("git");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        output.stdout
    };
    git(&["config", "user.name", "Fixture"]);
    git(&["config", "user.email", "fixture@example.invalid"]);
    git(&["config", "commit.gpgSign", "false"]);
    git(&["add", "file"]);
    git(&["commit", "-m", "base"]);
    git(&["branch", "base"]);
    git(&["checkout", "-b", "filtered"]);
    std::fs::write(root.path().join(".gitattributes"), "file filter=custom\n").expect("attributes");
    std::fs::write(root.path().join("file"), "target").expect("target");
    git(&["add", "."]);
    git(&["commit", "-m", "filtered target"]);
    git(&["checkout", "base"]);
    git(&[
        "config",
        "filter.custom.smudge",
        "sh -c 'printf forbidden > marker; cat'",
    ]);
    let head = git(&["rev-parse", "HEAD"]);
    let index = std::fs::read(root.path().join(".git/index")).expect("index");
    let plan = AddPlan::prepare_checkout(
        root.path(),
        &serde_json::json!({"branch":"filtered"}),
        ToolProvenance::Native,
    )
    .expect("plan");
    let output = execute_approved(plan).await;
    assert!(
        output.result.is_error && output.result.content.contains("filter"),
        "{}",
        output.result.content
    );
    assert_eq!(git(&["rev-parse", "HEAD"]), head);
    assert_eq!(
        std::fs::read(root.path().join(".git/index")).expect("index"),
        index
    );
    assert_eq!(
        std::fs::read(root.path().join("file")).expect("file"),
        b"before"
    );
    assert!(!root.path().join("marker").exists());
}
