#[cfg(windows)]
#[test]
fn child_backlink_alias_requires_bound_root_and_marker_reads() {
    let root = fixture();
    let canonical = root.path().canonicalize().expect("canonical");
    let layout = super::super::receiver_state::Root::capture(root.path(), &canonical)
        .expect("layout");
    let conventional = host_path::directory_argument(&canonical).expect("Git spelling");
    let marker = conventional.join(".git");
    assert_ne!(marker, canonical.join(".git"));
    for omitted in [None, Some("root"), Some("marker")] {
        let mut plan = AddPlan::prepare_pull(
            root.path(),
            &serde_json::json!({}),
            ToolProvenance::Native,
        )
        .expect("plan");
        plan.facets.clear();
        for (kind, path) in [
            ("root", root.path().to_owned()),
            ("marker", root.path().join(".git")),
        ] {
            if omitted != Some(kind) {
                plan.facets.push(path_facet(ToolNature::Read, &path, kind).expect("facet"));
            }
        }
        let tokens = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("proof");
        let result = plan.child_backlink_matches(&layout, Some(&marker), &tokens);
        if omitted.is_some() {
            assert!(result.is_err(), "missing {omitted:?}");
        } else {
            assert!(result.expect("admitted alias"));
            let unrelated = conventional.join("unadmitted-missing/.git");
            assert!(!plan.child_backlink_matches(&layout, Some(&unrelated), &tokens)
                .expect("compare declaration without opening it"));
            assert!(!unrelated.exists());
        }
    }
}

#[tokio::test]
async fn pull_child_bootstrap_defers_worktree_and_retains_startup_evidence() {
    let root = fixture();
    let child = root.path().join("child");
    assert!(
        std::process::Command::new("git")
            .current_dir(root.path())
            .args(["init", "child"])
            .output()
            .expect("init")
            .status
            .success()
    );
    let child = child.canonicalize().expect("child");
    let gitdir = child.join(".git");
    let missing = root.path().join("unadmitted-worktree");
    let config_path = gitdir.join("config");
    let original = std::fs::read_to_string(&config_path).expect("config");
    assert!(
        std::process::Command::new("git")
            .current_dir(&child)
            .args(["config", "core.worktree"])
            .arg(&missing)
            .output()
            .expect("startup target")
            .status
            .success()
    );
    let layout = super::super::receiver_state::Root::capture(&child, &child).expect("root");
    let metadata =
        super::super::receiver_state::Metadata::capture(&gitdir, &gitdir).expect("metadata");
    let location =
        super::super::repository_state::ChildLocation::capture(&gitdir).expect("location");
    let mut plan = Box::new(
        AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
            .expect("prepare"),
    );
    plan.environment
        .insert("GIT_CONFIG_NOSYSTEM".into(), "1".into());
    plan.environment.insert(
        "GIT_CONFIG_GLOBAL".into(),
        root.path().join("absent-config").into_os_string(),
    );
    plan.phase = Phase::PullChildConfiguration;
    plan.pull_child_layouts.push((layout, Some(metadata)));
    plan.child_locations.push((child.clone(), location));
    plan.facets.push(ToolPermissionFacet::with_resource(
        ToolNature::Read,
        "git:configuration-chain",
        ToolResourceKind::Remote,
    ));
    for path in [&gitdir, &config_path, &gitdir.join("config.worktree")] {
        plan.facets
            .push(path_facet(ToolNature::Read, path, "startup").expect("facet"));
    }
    let tokens = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("proof");
    let mut plan = plan
        .discover_pull_child_configuration(&tokens)
        .await
        .expect("discovery");
    assert!(!missing.exists());
    assert!(plan.pull_child_startup[&child].targets.contains(&missing));
    assert!(plan.child_read(&missing, &tokens).is_err());
    assert!(
        plan.facets
            .iter()
            .any(|facet| facet.resource.as_deref() == missing.to_str())
    );
    assert!(plan.bind_pull_child_worktree(&child, &tokens).is_err());
    std::fs::create_dir(&missing).expect("create independently admitted target");
    let next_tokens = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("worktree proof");
    plan.bind_pull_child_worktree(&child, &next_tokens)
        .expect("bind admitted target");
    assert_eq!(plan.pull_child_worktrees[&child].0, missing);
    plan.environment = trace_suppressed_environment(execution_environment());
    plan.revalidate().expect("retained target identity");
    #[cfg(any(unix, windows))]
    {
        let previous_target = root.path().join("previous-worktree");
        std::fs::rename(&missing, &previous_target).expect("move admitted directory");
        std::fs::create_dir(&missing).expect("replace target");
        assert!(
            plan.revalidate().is_err(),
            "same spelling must not replace identity"
        );
        std::fs::remove_dir(&missing).expect("remove empty replacement");
        std::fs::rename(&previous_target, &missing).expect("restore identity");
        plan.revalidate().expect("restored target");
    }
    std::fs::write(&config_path, original).expect("change startup");
    assert!(plan.pull_child_startup[&child].revalidate().is_err());
}

#[tokio::test]
async fn pull_initialized_current_child_suppresses_same_historical_name() {
    for historical_name in ["renamed", "Renamed"] {
        let root = fixture();
        assert!(
            std::process::Command::new("git")
                .current_dir(root.path())
                .args(["init", "current"])
                .status()
                .expect("child init")
                .success()
        );
        let mut plan = Box::new(
            AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
                .expect("plan"),
        )
        .discover()
        .await
        .expect("config");
        let mut modules = super::super::pull_children::Tree::parse(b"", 40).expect("modules");
        modules
            .module_names
            .insert("current".into(), "renamed".into());
        plan.pull_current_modules = Some(modules);
        plan.pull_current_fetch_children.insert(
            "current".into(),
            super::super::pull_children::Recursion::OnDemand,
        );
        let oid = "a".repeat(40);
        plan.pull_changed_children.insert(
            historical_name.into(),
            super::super::pull_children::Changed {
                commit: "b".repeat(40),
                path: "historical/missing".into(),
                objects: [oid.clone()].into_iter().collect(),
                historical_recursion: None,
                effective_recursion: Some(super::super::pull_children::Recursion::OnDemand),
                activity: Some(super::super::pull_children::Activity::Enabled),
            },
        );
        let current = plan.repository.workdir.join("current");
        assert_eq!(
            plan.pull_child_objects(&current),
            if historical_name == "renamed" {
                vec![oid]
            } else {
                vec![]
            }
        );
        plan.select_pull_child_identity().expect("select");
        let tokens = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("proof");
        let result = plan.discover_pull_child_identity(&tokens);
        if historical_name == "renamed" {
            let plan =
                result.expect("same name is suppressed before opening historical missing path");
            assert_eq!(plan.pull_child_layouts.len(), 1);
            assert_eq!(plan.pull_child_layouts[0].0.spelling, current);
        } else {
            assert!(result.is_err(), "module names remain case sensitive");
        }
    }
}

#[cfg(unix)]
#[test]
fn native_child_fetch_uses_gitdir_for_relative_hooks_and_remote() {
    use std::os::unix::fs::PermissionsExt;
    let source = fixture();
    let git = |cwd: &Path, args: &[&str]| {
        let output = std::process::Command::new("git")
            .current_dir(cwd)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .args(args)
            .output()
            .expect("Git");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(
        source.path(),
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.com",
            "commit",
            "--allow-empty",
            "-m",
            "source",
        ],
    );
    for mode in ["native", "configured", "stale", "historical", "prepared"] {
        let child = fixture();
        let root = child.path().canonicalize().expect("root");
        let gitdir = root.join(".git");
        let worktree = root.join("configured-worktree");
        if mode != "stale" {
            std::fs::create_dir(&worktree).expect("worktree");
        }
        if mode != "native" {
            git(
                &root,
                &["config", "core.worktree", worktree.to_str().expect("path")],
            );
        }
        git(
            &gitdir,
            &[
                "clone",
                "--bare",
                source.path().to_str().expect("source"),
                "relative-source",
            ],
        );
        git(&root, &["config", "remote.origin.url", "relative-source"]);
        git(&root, &["config", "core.hooksPath", "relative-hooks"]);
        let hooks = gitdir.join("relative-hooks");
        std::fs::create_dir(&hooks).expect("hooks");
        let hook = hooks.join("reference-transaction");
        std::fs::write(&hook, "#!/bin/sh\npwd > hook-ran\n").expect("hook");
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).expect("mode");
        let mut command = std::process::Command::new("git");
        command
            .current_dir(&gitdir)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_DIR", ".");
        if mode == "historical" {
            command.arg("--work-tree=.");
        } else if mode == "prepared" {
            command
                .arg("--git-dir")
                .arg(&gitdir)
                .arg("--work-tree")
                .arg(&worktree);
        }
        let output = command
            .args([
                "-c",
                "maintenance.autoDetach=false",
                "-c",
                "gc.autoDetach=false",
                "fetch",
                "--no-recurse-submodules",
                "origin",
                "HEAD:refs/heads/fetched",
            ])
            .output()
            .expect("fetch");
        assert!(
            output.status.success(),
            "{mode}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let receipt = std::fs::read_to_string(gitdir.join("hook-ran")).expect("hook ran in gitdir");
        assert_eq!(
            Path::new(receipt.trim()).canonicalize().expect("hook cwd"),
            gitdir
        );
        assert!(!root.join("hook-ran").exists());
        assert!(!worktree.join("hook-ran").exists());
    }
}

#[tokio::test]
async fn pull_child_fetch_has_own_write_stage_and_rebuilds_graph() {
    let remote = fixture();
    let git = |cwd: &Path, args: &[&str]| {
        let output = std::process::Command::new("git")
            .current_dir(cwd)
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
            "user.email=fixture@example.com",
            "commit",
            "--allow-empty",
            "-m",
            "source",
        ],
    );
    let expected = git(remote.path(), &["rev-parse", "HEAD"]);
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("port");
    let port = listener.local_addr().expect("address").port();
    drop(listener);
    let mut daemon = tokio::process::Command::new("git")
        .args([
            "daemon",
            "--reuseaddr",
            "--listen=127.0.0.1",
            "--export-all",
        ])
        .arg(format!("--port={port}"))
        .arg(format!("--base-path={}", daemon_fixture_path(remote.path())))
        .arg(daemon_fixture_path(remote.path()))
        .kill_on_drop(true)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("daemon");
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
                break;
            }
            assert!(daemon.try_wait().expect("state").is_none());
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("ready");
    'cases: for mode in [
        "allow",
        "relative-hook",
        "relative-local",
        "missing-write",
        "skip",
        "linked",
        "linked-tamper",
        "renamed-on",
        "historical-on",
    ] {
        let root = fixture();
        git(root.path(), &["init", "child"]);
        let child = root.path().join("child").canonicalize().expect("child");
        if mode == "relative-hook" {
            git(&child, &["config", "core.hooksPath", "relative-hooks"]);
            let hooks = child.join(".git/relative-hooks");
            std::fs::create_dir(&hooks).expect("hooks");
            let hook = hooks.join("reference-transaction");
            std::fs::write(&hook, "#!/bin/sh\necho ran > hook-ran\n").expect("hook");
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755))
                    .expect("executable hook");
            }
        }
        git(
            &child,
            &[
                "remote",
                "add",
                "origin",
                &format!("git://127.0.0.1:{port}/"),
            ],
        );
        if mode == "relative-local" {
            git(&child, &["remote", "set-url", "origin", "relative-source"]);
        }
        if matches!(mode, "skip" | "renamed-on" | "historical-on") {
            git(
                &child,
                &[
                    "fetch",
                    remote.path().to_str().expect("remote"),
                    "HEAD:refs/heads/present",
                ],
            );
            std::fs::remove_file(child.join(".git/FETCH_HEAD"))
                .expect("clear fixture fetch marker");
        }
        if mode.starts_with("linked") {
            git(
                &child,
                &[
                    "-c",
                    "user.name=Fixture",
                    "-c",
                    "user.email=fixture@example.com",
                    "commit",
                    "--allow-empty",
                    "-m",
                    "child",
                ],
            );
            git(&child, &["worktree", "add", "--detach", "../sibling"]);
        }
        let stale = root.path().join("missing-historical-worktree");
        if mode == "historical-on" {
            git(
                &child,
                &["config", "core.worktree", stale.to_str().expect("stale")],
            );
        }
        let mut plan = Box::new(
            AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
                .expect("plan"),
        )
        .discover()
        .await
        .expect("config");
        if mode != "historical-on" {
            plan.pull_current_fetch_children.insert(
                "child".into(),
                if mode == "skip" || mode == "renamed-on" {
                    super::super::pull_children::Recursion::OnDemand
                } else {
                    super::super::pull_children::Recursion::On
                },
            );
        }
        if matches!(mode, "skip" | "renamed-on" | "historical-on") {
            plan.pull_changed_children.insert(
                "child".into(),
                super::super::pull_children::Changed {
                    commit: expected.clone(),
                    path: "child".into(),
                    objects: [expected.clone()].into_iter().collect(),
                    historical_recursion: None,
                    effective_recursion: Some(if mode == "historical-on" {
                        super::super::pull_children::Recursion::On
                    } else {
                        super::super::pull_children::Recursion::OnDemand
                    }),
                    activity: Some(super::super::pull_children::Activity::Enabled),
                },
            );
        }
        if mode.starts_with("linked") {
            plan.pull_current_fetch_children
                .insert("sibling".into(), super::super::pull_children::Recursion::On);
        }
        if mode == "renamed-on" {
            let mut modules = super::super::pull_children::Tree::parse(b"", 40).expect("modules");
            modules.module_names.insert("child".into(), "child".into());
            plan.pull_current_modules = Some(modules);
            let changed = plan
                .pull_changed_children
                .get_mut("child")
                .expect("historical");
            changed.path = "old/missing".into();
            changed.effective_recursion = Some(super::super::pull_children::Recursion::On);
        }
        plan.select_pull_child_identity().expect("select");
        let fetch_command = plan
            .pull_child_fetch_command(&child, &child.join(".git"))
            .expect("identity-preserving fetch command");
        let fetch_arguments: Vec<_> = fetch_command.as_std().get_args().collect();
        assert_eq!(
            fetch_arguments.iter().any(|arg| *arg == "--work-tree=."),
            mode == "historical-on",
            "only the historical task overrides worktree setup: {mode}"
        );
        assert!(!fetch_arguments.iter().any(|arg| *arg == "--work-tree"));
        for _ in 0..8 {
            if plan.phase != Phase::PullChildIdentity {
                break;
            }
            let tokens = proof(&plan)
                .consume_for_prepared(&plan.request(), &plan.identity)
                .expect("identity proof");
            plan = plan
                .discover_pull_child_identity(&tokens)
                .unwrap_or_else(|error| panic!("{mode}: child identity: {error}"));
        }
        for _ in 0..8 {
            if plan.phase != Phase::PullChildConfiguration {
                break;
            }
            let tokens = proof(&plan)
                .consume_for_prepared(&plan.request(), &plan.identity)
                .expect("config proof");
            let result = plan.discover_pull_child_configuration(&tokens).await;
            if mode == "relative-hook" && result.is_err() {
                let error = result.err().expect("hook refusal");
                assert!(
                    error.to_string().contains("Executable Git Pull hooks"),
                    "{error}"
                );
                assert!(!child.join(".git/FETCH_HEAD").exists());
                assert!(!child.join(".git/hook-ran").exists());
                continue 'cases;
            }
            plan = result.expect("child config");
            if mode == "relative-local" {
                let expected = child.join(".git/relative-source");
                assert!(plan.facets.iter().any(|facet| {
                    facet.nature == ToolNature::Read
                        && facet.resource.as_deref() == expected.to_str()
                }));
                assert!(!child.join(".git/FETCH_HEAD").exists());
                continue 'cases;
            }
        }
        assert_ne!(
            mode, "relative-hook",
            "executable hook must be refused before fetch"
        );
        if mode == "historical-on" {
            assert!(plan.pull_child_startup[&child].targets.contains(&stale));
            assert_eq!(plan.pull_child_worktrees[&child].0, child.join(".git"));
            assert!(
                !plan
                    .facets
                    .iter()
                    .any(|facet| facet.resource.as_deref() == stale.to_str())
            );
            assert!(!stale.exists());
        }
        assert!(plan.phase == Phase::PullChildFetch);
        assert!(!child.join(".git/FETCH_HEAD").exists());
        let mut tokens = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("fetch proof");
        if mode == "missing-write" {
            tokens.retain(|token| {
                !token.authorizes_path(
                    "git_pull",
                    ToolNature::Write,
                    &plan.workspace,
                    child.join(".git").to_str().expect("path"),
                )
            });
            assert!(
                plan.execute_pull_child_fetch(&tokens)
                    .await
                    .err()
                    .expect("refusal")
                    .to_string()
                    .contains("Write admission")
            );
            assert!(!child.join(".git/FETCH_HEAD").exists());
            continue;
        }
        let mut plan = plan.execute_pull_child_fetch(&tokens).await.expect("fetch");
        if mode.starts_with("linked") {
            for _ in 0..8 {
                if plan.phase != Phase::PullChildConfiguration {
                    break;
                }
                let tokens = proof(&plan)
                    .consume_for_prepared(&plan.request(), &plan.identity)
                    .expect("shared graph proof");
                plan = plan
                    .discover_pull_child_configuration(&tokens)
                    .await
                    .expect("shared graph refresh");
            }
            assert!(plan.phase == Phase::PullChildFetch);
            assert_eq!(plan.pull_children_fetched.len(), 1);
            plan.revalidate()
                .expect("shared graph refreshed before sibling fetch");
            if mode == "linked-tamper" {
                git(&child, &["update-ref", "refs/heads/tampered", &expected]);
                assert!(
                    plan.revalidate().is_err(),
                    "first child's graph stays bound between fetches"
                );
                continue;
            }
            let tokens = proof(&plan)
                .consume_for_prepared(&plan.request(), &plan.identity)
                .expect("sibling fetch proof");
            plan = plan
                .execute_pull_child_fetch(&tokens)
                .await
                .expect("sibling fetch");
            assert_eq!(plan.pull_children_fetched.len(), 2);
        }
        assert!(matches!(
            plan.phase,
            Phase::PullChildConfiguration | Phase::PullIntegration
        ));
        assert!(
            !plan
                .facets
                .iter()
                .any(|facet| matches!(facet.nature, ToolNature::Write | ToolNature::Network))
        );
        if mode == "allow" || mode.starts_with("linked") || mode == "historical-on" {
            assert!(
                std::fs::read_to_string(child.join(".git/FETCH_HEAD"))
                    .expect("fetch head")
                    .contains(&expected)
            );
        } else {
            assert!(!child.join(".git/FETCH_HEAD").exists());
        }
        for _ in 0..8 {
            if plan.phase != Phase::PullChildConfiguration {
                break;
            }
            let tokens = proof(&plan)
                .consume_for_prepared(&plan.request(), &plan.identity)
                .expect("postfetch proof");
            plan = plan
                .discover_pull_child_configuration(&tokens)
                .await
                .expect("fresh graph");
        }
        assert!(plan.phase == Phase::PullIntegration);
    }
    daemon.kill().await.expect("stop");
    daemon.wait().await.expect("reap");
}

#[tokio::test]
async fn pull_child_metadata_scope_batches_depth_and_requires_own_root() {
    let mut modes = vec![
        "allow",
        "missing-capability",
        "missing-root",
        "missing-hook",
    ];
    if cfg!(unix) {
        modes.extend([
            "missing-object",
            "noncommit",
            "executable-hook",
            "hook-change",
            "linked-hook",
        ]);
    }
    for mode in modes {
        let root = fixture();
        assert!(
            std::process::Command::new("git")
                .current_dir(root.path())
                .args([
                    "init",
                    if mode == "linked-hook" {
                        "seed"
                    } else {
                        "child"
                    }
                ])
                .output()
                .expect("child init")
                .status
                .success()
        );
        if mode == "linked-hook" {
            let seed = root.path().join("seed");
            assert!(
                std::process::Command::new("git")
                    .current_dir(&seed)
                    .args([
                        "-c",
                        "user.name=Fixture",
                        "-c",
                        "user.email=fixture@example.com",
                        "commit",
                        "--allow-empty",
                        "-m",
                        "seed"
                    ])
                    .output()
                    .expect("seed commit")
                    .status
                    .success()
            );
            assert!(
                std::process::Command::new("git")
                    .current_dir(&seed)
                    .args(["worktree", "add", "--detach", "../child"])
                    .output()
                    .expect("linked child")
                    .status
                    .success()
            );
        }
        let child = root.path().join("child").canonicalize().expect("child");
        let gitdir = if mode == "linked-hook" {
            root.path().join("seed/.git/worktrees/child")
        } else {
            child.join(".git")
        };
        let common = if mode == "linked-hook" {
            root.path().join("seed/.git")
        } else {
            gitdir.clone()
        };
        let hook = common.join("hooks/reference-transaction");
        #[cfg(unix)]
        if matches!(mode, "executable-hook" | "hook-change" | "linked-hook") {
            use std::os::unix::fs::PermissionsExt;
            std::fs::write(&hook, "#!/bin/sh\nexit 0\n").expect("hook");
            std::fs::set_permissions(
                &hook,
                std::fs::Permissions::from_mode(if mode != "hook-change" { 0o755 } else { 0o644 }),
            )
            .expect("hook permissions");
        }
        let deep = gitdir
            .join("refs/heads")
            .join("nested/".repeat(24))
            .join("tip");
        std::fs::create_dir_all(deep.parent().expect("parent")).expect("deep refs");
        std::fs::write(&deep, format!("{}\n", "a".repeat(40))).expect("ref");
        let mut plan = Box::new(
            AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
                .expect("prepare"),
        )
        .discover()
        .await
        .expect("parent config");
        plan.pull_current_fetch_children
            .insert("child".into(), super::super::pull_children::Recursion::On);
        if matches!(mode, "missing-object" | "noncommit") {
            let object = if mode == "noncommit" {
                let output = std::process::Command::new("git")
                    .current_dir(&child)
                    .args(["hash-object", "-w", "../file"])
                    .output()
                    .expect("blob");
                assert!(output.status.success());
                String::from_utf8(output.stdout)
                    .expect("oid")
                    .trim()
                    .to_owned()
            } else {
                "b".repeat(40)
            };
            plan.pull_changed_children.insert(
                "child".into(),
                super::super::pull_children::Changed {
                    commit: "c".repeat(40),
                    path: "child".into(),
                    objects: [object].into_iter().collect(),
                    historical_recursion: None,
                    effective_recursion: Some(super::super::pull_children::Recursion::OnDemand),
                    activity: Some(super::super::pull_children::Activity::Enabled),
                },
            );
        }
        plan.select_pull_child_identity().expect("select");
        while plan.phase == Phase::PullChildIdentity {
            let tokens = proof(&plan)
                .consume_for_prepared(&plan.request(), &plan.identity)
                .expect("identity proof");
            plan = plan
                .discover_pull_child_identity(&tokens)
                .expect("child identity");
        }
        assert!(plan.phase == Phase::PullChildConfiguration);
        let tokens = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("config proof");
        plan = plan
            .discover_pull_child_configuration(&tokens)
            .await
            .expect("config discovery");
        assert!(plan.pull_config_sources_discovered);
        assert!(!plan.pull_child_inventories[&child].complete());
        assert!(
            !plan
                .facets
                .iter()
                .any(|facet| facet.resource.as_deref() == deep.to_str())
        );
        if mode == "missing-capability" {
            plan.facets
                .retain(|facet| facet.resource.as_deref() != Some("git:pull-repository-metadata"));
        }
        let mut tokens = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("graph proof");
        if mode == "missing-root" {
            tokens.retain(|token| token.normalized_path() != gitdir);
        }
        if mode == "missing-hook" {
            tokens.retain(|token| token.normalized_path() != hook);
        }
        let next = plan.discover_pull_child_configuration(&tokens).await;
        if matches!(mode, "executable-hook" | "linked-hook") {
            assert!(
                next.err()
                    .expect("hook refusal")
                    .to_string()
                    .contains("Executable Git Pull hooks")
            );
            continue;
        }
        if mode == "noncommit" {
            assert!(
                next.err()
                    .expect("noncommit")
                    .to_string()
                    .contains("not a commit")
            );
            continue;
        }
        if mode == "missing-hook" {
            let plan = next.expect("deferred hook admission");
            assert!(plan.phase == Phase::PullChildConfiguration);
            assert!(plan.pull_child_hook_evidence.is_empty());
            continue;
        }
        if !matches!(mode, "allow" | "missing-object" | "hook-change") {
            assert!(next.is_err(), "{mode} must refuse graph discovery");
            continue;
        }
        let plan = next.expect("single admitted graph stage");
        assert!(plan.phase == Phase::PullChildFetch);
        assert!(
            plan.pull_child_hook_evidence
                .iter()
                .any(|(path, _)| path == &hook)
        );
        assert!(plan.pull_child_inventories[&child].complete());
        assert!(
            !plan
                .facets
                .iter()
                .any(|facet| facet.resource.as_deref() == deep.to_str())
        );
        if mode == "hook-change" {
            std::fs::write(&hook, "changed\n").expect("edit hook");
        } else {
            std::fs::write(&deep, format!("{}\n", "b".repeat(40))).expect("edit ref");
        }
        assert!(plan.revalidate().is_err());
    }
}

#[test]
fn pull_child_identity_defers_external_gitdir_and_common_targets() {
    let root = fixture();
    let external = tempfile::tempdir().expect("external fixture");
    let child = root.path().join("child");
    let gitdir = external.path().join("metadata");
    let common = external.path().join("common");
    std::fs::create_dir(&child).expect("child");
    std::fs::write(
        child.join(".git"),
        format!("gitdir: {}\n", gitdir.display()),
    )
    .expect("marker");
    let mut plan =
        AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
            .expect("prepare");
    plan.phase = Phase::PullChildIdentity;
    plan.pull_changed_children.insert(
        "child".into(),
        super::super::pull_children::Changed {
            commit: "0".repeat(40),
            path: "child".into(),
            objects: Default::default(),
            historical_recursion: None,
            effective_recursion: None,
            activity: Some(super::super::pull_children::Activity::Enabled),
        },
    );
    for path in [&child, &child.join(".git")] {
        plan.facets
            .push(path_facet(ToolNature::Read, path, "child identity").expect("facet"));
    }
    let mut plan = Box::new(plan);
    let tokens = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("marker proof");
    plan = plan
        .discover_pull_child_identity(&tokens)
        .expect("marker wave");
    assert!(!gitdir.exists(), "marker wave must not resolve its target");
    assert!(plan.child_locations.is_empty());
    assert!(
        plan.facets
            .iter()
            .any(|facet| facet.resource.as_deref() == gitdir.to_str())
    );
    // Creating the newly named target after discovery is legal: it was
    // intentionally unopened until the next independently admitted wave.
    std::fs::create_dir(&gitdir).expect("gitdir");
    std::fs::write(gitdir.join("commondir"), b"../common\n").expect("pointer");
    std::fs::write(gitdir.join("HEAD"), b"ref: refs/heads/main\n").expect("HEAD");
    assert!(plan.child_read(&gitdir, &tokens).is_err());
    let tokens = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("gitdir proof");
    plan = plan
        .discover_pull_child_identity(&tokens)
        .expect("metadata wave");
    assert!(
        !common.exists(),
        "metadata wave must not resolve common target"
    );
    assert!(plan.child_locations.is_empty());
    assert!(plan.child_read(&common, &tokens).is_err());
    std::fs::create_dir(&common).expect("common");
    let tokens = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("common proof");
    plan = plan
        .discover_pull_child_identity(&tokens)
        .expect("common wave");
    assert!(matches!(plan.phase, Phase::PullChildConfiguration));
    assert_eq!(plan.child_locations.len(), 1);
    assert_eq!(
        plan.child_locations[0].1.common_dir,
        common.canonicalize().expect("common identity")
    );
    plan.revalidate().expect("stable layout");
    std::fs::write(child.join(".git"), "gitdir: /unadmitted/new/target\n")
        .expect("redirect marker");
    assert!(
        plan.revalidate().is_err(),
        "retained marker evidence must reject redirection"
    );
    std::fs::write(
        child.join(".git"),
        format!("gitdir: {}\n", gitdir.display()),
    )
    .expect("restore marker");
    plan.revalidate().expect("restored marker");
    std::fs::write(gitdir.join("commondir"), b"/unadmitted/common\n")
        .expect("redirect common pointer");
    assert!(
        plan.revalidate().is_err(),
        "retained metadata must reject common-directory redirection"
    );
}

#[test]
fn pull_historical_recursion_off_does_not_admit_child_paths() {
    use super::super::pull_children::{Activity, Changed, Recursion};
    let root = fixture();
    let mut plan =
        AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
            .expect("Pull");
    let child = plan.repository.workdir.join("historical");
    plan.pull_changed_children.insert(
        "module".into(),
        Changed {
            commit: "0".repeat(40),
            path: "historical".into(),
            objects: Default::default(),
            historical_recursion: Some(Recursion::On),
            effective_recursion: Some(Recursion::Off),
            activity: Some(Activity::Enabled),
        },
    );
    plan.select_pull_child_identity().expect("selection");
    assert!(plan.phase == Phase::PullIntegration);
    assert!(!plan.facets.iter().any(|facet| {
        facet.resource.as_deref() == child.to_str()
            || facet.resource.as_deref() == child.join(".git").to_str()
    }));
    for mode in [Recursion::On, Recursion::OnDemand] {
        plan.pull_changed_children
            .get_mut("module")
            .expect("child")
            .effective_recursion = Some(mode);
        assert_eq!(plan.selected_pull_child_paths(), vec![child.clone()]);
    }
    plan.pull_changed_children
        .get_mut("module")
        .expect("child")
        .effective_recursion = Some(Recursion::Off);
    plan.pull_current_fetch_children
        .insert("historical".into(), Recursion::On);
    assert_eq!(plan.selected_pull_child_paths(), vec![child]);
}

#[tokio::test]
async fn pull_current_child_without_changes_requires_fresh_identity_admission() {
    let root = fixture();
    let output = std::process::Command::new("git")
        .current_dir(root.path())
        .args(["init", "child"])
        .output()
        .expect("initialize child");
    assert!(output.status.success(), "{output:?}");
    let mut plan = Box::new(
        AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
            .expect("Pull"),
    )
    .discover()
    .await
    .expect("configuration");
    let child = plan.repository.workdir.join("child");
    let mut modules = super::super::pull_children::Tree::parse(b"", 40).expect("modules");
    modules
        .module_names
        .insert("child".into(), "group/named-child".into());
    plan.pull_current_modules = Some(modules);
    let fallback = plan.repository.common_dir.join("modules/group/named-child");
    let stale = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("old tokens");
    assert!(plan.pull_changed_children.is_empty());
    plan.pull_current_fetch_children
        .insert("child".into(), super::super::pull_children::Recursion::On);
    plan.select_pull_child_identity()
        .expect("select current child");
    assert!(plan.phase == Phase::PullChildIdentity);
    let prefix = plan.repository.common_dir.join("modules/group");
    let frontier = [
        child.clone(),
        child.join(".git"),
        fallback.clone(),
        plan.repository.common_dir.join("modules"),
        prefix.clone(),
        prefix.join("HEAD"),
        prefix.join("objects"),
        prefix.join("refs"),
    ];
    for path in &frontier {
        assert!(plan.facets.iter().any(|facet| {
            facet.nature == ToolNature::Read && facet.resource.as_deref() == path.to_str()
        }));
        assert!(plan.child_read(path, &stale).is_err());
    }
    let tokens = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("fresh child tokens");
    for path in &frontier {
        assert!(plan.child_read(path, &tokens).is_ok());
        let mut denied = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("independent denial proof");
        denied.retain(|token| token.normalized_path() != path);
        assert!(plan.child_read(path, &denied).is_err());
    }
    assert!(plan.child_read(&fallback, &tokens).is_ok());
    assert!(
        !fallback.exists(),
        "candidate admission must not create metadata"
    );
    assert!(plan.child_locations.is_empty());
    let plan = plan
        .discover_pull_child_identity(&tokens)
        .expect("admitted current child");
    assert_eq!(plan.pull_child_layouts.len(), 1);
    assert_eq!(plan.pull_child_layouts[0].0.spelling, child);
    assert!(
        plan.child_locations.is_empty(),
        "metadata requires another admission"
    );
    let tokens = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("metadata tokens");
    let plan = plan
        .discover_pull_child_identity(&tokens)
        .expect("already admitted common directory");
    assert!(plan.phase == Phase::PullChildConfiguration);
    assert_eq!(plan.child_locations.len(), 1);
    assert!(
        !plan.pull_config_sources_discovered,
        "config still waits for admission"
    );
}

#[tokio::test]
async fn pull_child_attributes_require_fresh_exact_admission() {
    for denied in [true, false] {
        let root = fixture();
        let mut plan = Box::new(
            AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
                .expect("Pull"),
        )
        .discover()
        .await
        .expect("configuration");
        let stale = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("old tokens");
        let path = plan.repository.workdir.join(".gitattributes");
        std::fs::write(&path, "* selected\n").expect("rules");
        plan.pull_attribute_paths.push(path.clone());
        plan.facets
            .push(path_facet(ToolNature::Read, &path, "child attributes").expect("facet"));
        plan.phase = Phase::PullChildAttributes;
        plan.identity = PreparedPlanIdentity::new();
        assert!(plan.pull_attributes.is_empty());
        let admission = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("fresh tokens");
        // The previous frontier has no token for this new worktree file.
        assert!(!stale.iter().any(|token| token.authorizes_path(
            "git_pull",
            ToolNature::Read,
            &plan.workspace,
            path.to_str().expect("path"),
        )));
        if denied {
            assert!(plan.discover_pull_child_attributes(&stale).await.is_err());
            continue;
        }
        let next = plan
            .discover_pull_child_attributes(&admission)
            .await
            .expect("capture");
        assert!(next.phase == Phase::PullIntegration);
        assert_eq!(next.pull_attributes.len(), 1);
        next.revalidate().expect("retained evidence");
        std::fs::write(path, "* -selected\n").expect("tamper");
        assert!(next.revalidate().is_err());
    }
}

#[tokio::test]
async fn pull_current_modules_require_exact_read_before_capture() {
    let root = fixture();
    std::fs::write(root.path().join(".gitmodules"), b"not config").expect("unadmitted source");
    let mut plan = Box::new(
        AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
            .expect("Pull"),
    )
    .discover()
    .await
    .expect("configuration");
    for _ in 0..8 {
        if plan.pull_graph_paths.is_some() {
            break;
        }
        let admission = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("proof");
        plan = plan
            .discover_pull_prefetch(&admission)
            .await
            .expect("inventory");
    }
    assert!(plan.pull_graph_paths.is_some());
    assert!(plan.pull_modules_evidence.is_none());
    let path = plan.repository.workdir.join(".gitmodules");
    assert!(
        plan.facets
            .iter()
            .any(|facet| facet.resource.as_deref() == path.to_str())
    );
    plan.facets
        .retain(|facet| facet.resource.as_deref() != path.to_str());
    let admission = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("proof without module path");
    let Err(error) = plan.discover_pull_prefetch(&admission).await else {
        panic!("module source opened without exact Read");
    };
    assert!(
        error.to_string().contains("exact Read admission"),
        "{error}"
    );
    assert!(!root.path().join(".git/FETCH_HEAD").exists());
}

#[cfg(unix)]
#[tokio::test]
async fn pull_child_batch_uses_owned_input_and_distinguishes_execution_failure() {
    let root = fixture();
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .current_dir(root.path())
            .args(args)
            .output()
            .expect("git");
        assert!(output.status.success(), "{:?}", output.stderr);
        String::from_utf8(output.stdout)
            .expect("UTF8")
            .trim()
            .to_owned()
    };
    git(&[
        "-c",
        "user.name=test",
        "-c",
        "user.email=test@example.com",
        "commit",
        "--allow-empty",
        "-m",
        "initial",
    ]);
    let commit = git(&["rev-parse", "HEAD"]);
    let tree = git(&["rev-parse", "HEAD^{tree}"]);
    let orphan = git(&[
        "-c",
        "user.name=test",
        "-c",
        "user.email=test@example.com",
        "commit-tree",
        &tree,
        "-m",
        "unreferenced",
    ]);
    let mut plan =
        AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
            .expect("plan");
    let path = root.path().canonicalize().expect("root");
    let gitdir = path.join(".git");
    assert!(
        plan.pull_child_has_commits(&path, &gitdir, &path, std::slice::from_ref(&commit))
            .await
            .expect("reachable")
    );
    assert!(
        !plan
            .pull_child_has_commits(&path, &gitdir, &path, std::slice::from_ref(&orphan))
            .await
            .expect("unreachable")
    );
    assert!(
        !plan
            .pull_child_has_commits(&path, &gitdir, &path, &[commit.clone(), orphan.clone()])
            .await
            .expect("mixed")
    );
    assert!(
        !plan
            .pull_child_has_commits(&path, &gitdir, &path, &["a".repeat(40)])
            .await
            .expect("missing")
    );
    assert!(
        plan.pull_child_commit_objects(&path, &gitdir, &path, std::slice::from_ref(&commit))
            .await
            .expect("commit")
    );
    assert!(
        !plan
            .pull_child_commit_objects(&path, &gitdir, &path, &["a".repeat(40)])
            .await
            .expect("missing")
    );
    assert!(
        plan.pull_child_commit_objects(&path, &gitdir, &path, std::slice::from_ref(&tree))
            .await
            .expect_err("tree")
            .to_string()
            .contains("not a commit")
    );
    // --all includes detached HEADs of other registered worktrees; no
    // branch ref is needed to make this previously orphaned commit reachable.
    git(&["worktree", "add", "--detach", "linked", &orphan]);
    plan = AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
        .expect("linked worktree plan");
    assert!(
        plan.pull_child_has_commits(&path, &gitdir, &path, std::slice::from_ref(&orphan))
            .await
            .expect("linked detached HEAD is reachable")
    );
    git(&["replace", "-f", &commit, &tree]);
    plan = AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
        .expect("replacement plan");
    let error = plan
        .pull_child_commit_objects(&path, &gitdir, &path, std::slice::from_ref(&commit))
        .await
        .expect_err("replacement type");
    assert!(error.to_string().contains("not a commit"), "{error}");
    git(&["replace", "-d", &commit]);
    git(&[
        "-c",
        "user.name=test",
        "-c",
        "user.email=test@example.com",
        "tag",
        "-a",
        "tagged",
        "-m",
        "tag",
    ]);
    let tag = git(&["rev-parse", "tagged"]);
    plan = AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
        .expect("tag plan");
    assert!(
        plan.pull_child_commit_objects(&path, &gitdir, &path, &[tag])
            .await
            .expect_err("annotated tag must not peel")
            .to_string()
            .contains("not a commit")
    );
    // A setup failure must not be converted to an object-missing result.
    plan.program = std::path::PathBuf::from("/usr/bin/false");
    assert!(
        plan.pull_child_commit_objects(&path, &gitdir, &path, &[commit])
            .await
            .is_err()
    );
}
