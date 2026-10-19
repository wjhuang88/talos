#[test]
fn pull_fetch_command_binds_remote_foreground_maintenance_and_trace_policy() {
    use std::ffi::OsStr;
    let root = tempfile::tempdir().expect("root");
    let program = root.path().join("git executable");
    let workdir = root.path().join("worktree");
    std::fs::create_dir(&workdir).expect("worktree");
    let environment = trace_suppressed_environment(BTreeMap::from([
        (OsString::from("PATH"), OsString::from("/usr/bin")),
        (OsString::from("GIT_TRACE2"), OsString::from("/tmp/trace")),
    ]));
    let command = pull_fetch_command(&program, &workdir, "upstream name", &environment)
        .expect("fetch command");
    assert_eq!(command.as_std().get_program(), program.as_os_str());
    assert_eq!(command.as_std().get_current_dir(), Some(workdir.as_path()));
    assert_eq!(
        command
            .as_std()
            .get_args()
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        [
            "-c",
            "maintenance.autoDetach=false",
            "-c",
            "gc.autoDetach=false",
            "--no-pager",
            "fetch",
            "--no-recurse-submodules",
            "--update-head-ok",
            "--",
            "upstream name"
        ]
    );
    let actual_environment = command.as_std().get_envs().collect::<BTreeMap<_, _>>();
    assert_eq!(
        actual_environment.get(OsStr::new("GIT_TRACE2")),
        Some(&Some(OsStr::new("0")))
    );
    assert!(pull_fetch_command(&program, &workdir, "bad\nremote", &environment).is_err());
    assert!(pull_fetch_command(Path::new("git"), &workdir, "origin", &environment).is_err());
}

#[tokio::test]
async fn pull_parent_fetch_defers_recursive_child_effects() {
    fn path(path: &Path) -> &str {
        path.to_str().expect("fixture path")
    }
    let root = tempfile::tempdir().expect("fixture");
    let child_remote = root.path().join("child.git");
    let child = root.path().join("child");
    let parent_remote = root.path().join("parent.git");
    let parent = root.path().join("parent");
    let native = root.path().join("native");
    let prepared = root.path().join("prepared");
    let child_policy_cases = [
        ("global-off-child-on", "false", "true", false),
        ("global-on-child-off", "true", "false", true),
        ("global-demand-child-on", "on-demand", "true", true),
        ("global-on-child-demand", "true", "on-demand", true),
        ("global-default-child-off", "", "false", false),
        ("global-default-child-on", "", "true", true),
    ];
    let policy_cases = [
        (
            "off-last",
            [
                ("submodule.recurse", "true"),
                ("fetch.recurseSubmodules", "false"),
            ],
            "off",
            false,
        ),
        (
            "on-last",
            [
                ("fetch.recurseSubmodules", "false"),
                ("submodule.recurse", "true"),
            ],
            "on",
            true,
        ),
        (
            "demand-last",
            [
                ("submodule.recurse", "false"),
                ("fetch.recurseSubmodules", "on-demand"),
            ],
            "on-demand",
            true,
        ),
        (
            "off-over-demand",
            [
                ("fetch.recurseSubmodules", "on-demand"),
                ("submodule.recurse", "false"),
            ],
            "off",
            false,
        ),
    ];
    let git = |cwd: &Path, args: &[&str]| {
        let output = std::process::Command::new("git")
            .current_dir(cwd)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", root.path().join("absent-config"))
            .args([
                "-c",
                "protocol.file.allow=always",
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
            .expect("fixture Git");
        assert!(output.status.success(), "{args:?}: {output:?}");
        output.stdout
    };
    git(
        root.path(),
        &["init", "--bare", "-b", "main", path(&child_remote)],
    );
    git(root.path(), &["init", "-b", "main", path(&child)]);
    std::fs::write(child.join("file"), b"one\n").expect("child source");
    git(&child, &["add", "file"]);
    git(&child, &["commit", "-m", "child one"]);
    git(&child, &["remote", "add", "origin", path(&child_remote)]);
    git(&child, &["push", "-u", "origin", "main"]);

    git(
        root.path(),
        &["init", "--bare", "-b", "main", path(&parent_remote)],
    );
    git(root.path(), &["init", "-b", "main", path(&parent)]);
    git(
        &parent,
        &["submodule", "add", path(&child_remote), "modules/child"],
    );
    git(&parent, &["commit", "-m", "parent one"]);
    git(&parent, &["remote", "add", "origin", path(&parent_remote)]);
    git(&parent, &["push", "-u", "origin", "main"]);
    for checkout in [&native, &prepared] {
        git(
            root.path(),
            &[
                "clone",
                "--recurse-submodules",
                path(&parent_remote),
                path(checkout),
            ],
        );
    }
    for (name, config, _, _) in &policy_cases {
        let checkout = root.path().join(name);
        git(
            root.path(),
            &[
                "clone",
                "--recurse-submodules",
                path(&parent_remote),
                path(&checkout),
            ],
        );
        // `git config --add` inserts into existing sections. Append fresh
        // sections explicitly so this fixture tests actual declaration order.
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(checkout.join(".git/config"))
            .expect("fixture configuration");
        for (key, value) in config {
            let (section, name) = key.split_once('.').expect("global key");
            writeln!(file, "\n[{section}]\n\t{name} = {value}").expect("ordered config");
        }
    }

    for (name, global, local, _) in child_policy_cases {
        let checkout = root.path().join(name);
        git(
            root.path(),
            &[
                "clone",
                "--recurse-submodules",
                path(&parent_remote),
                path(&checkout),
            ],
        );
        if !global.is_empty() {
            git(&checkout, &["config", "fetch.recurseSubmodules", global]);
        }
        git(
            &checkout,
            &[
                "config",
                "submodule.modules/child.fetchRecurseSubmodules",
                local,
            ],
        );
    }

    std::fs::write(child.join("file"), b"two\n").expect("new child source");
    git(&child, &["add", "file"]);
    git(&child, &["commit", "-m", "child two"]);
    let new_oid = String::from_utf8(git(&child, &["rev-parse", "HEAD"])).expect("object ID");
    let new_oid = new_oid.trim();
    git(&child, &["push", "origin", "main"]);
    let parent_child = parent.join("modules/child");
    git(&parent_child, &["fetch", "origin"]);
    git(&parent_child, &["checkout", new_oid]);
    git(&parent, &["add", "modules/child"]);
    git(&parent, &["commit", "-m", "parent two"]);
    git(&parent, &["push", "origin", "main"]);

    let has_child_object = |checkout: &Path| {
        std::process::Command::new("git")
            .current_dir(checkout.join("modules/child"))
            .args(["cat-file", "-e", new_oid])
            .output()
            .expect("child object lookup")
            .status
            .success()
    };
    assert!(!has_child_object(&native) && !has_child_object(&prepared));
    git(&native, &["fetch", "--update-head-ok", "origin"]);
    assert!(
        has_child_object(&native),
        "native fetch recursively imports child objects"
    );
    for (name, _, expected_mode, fetches_child) in &policy_cases {
        let checkout = root.path().join(name);
        assert!(!has_child_object(&checkout));
        let config = git(&checkout, &["config", "--null", "--list"]);
        let retained = super::super::pull_state::BeforeFetch::probe(&config, None, 40, |args| {
            let output = std::process::Command::new("git")
                .current_dir(&checkout)
                .args(args)
                .output()
                .expect("native policy probe");
            async move { Ok(output) }
        })
        .await
        .expect("retained recursive-fetch policy");
        assert_eq!(
            retained.fetch_recursion_description(),
            *expected_mode,
            "{name}"
        );
        git(&checkout, &["fetch", "--update-head-ok", "origin"]);
        assert_eq!(has_child_object(&checkout), *fetches_child, "{name}");
    }

    for (name, _, _, fetches_child) in child_policy_cases {
        let checkout = root.path().join(name);
        assert!(!has_child_object(&checkout));
        git(&checkout, &["fetch", "--update-head-ok", "origin"]);
        assert_eq!(has_child_object(&checkout), fetches_child, "{name}");
    }

    let program = resolve_git(&execution_environment()).expect("absolute Git");
    let mut environment = trace_suppressed_environment(execution_environment());
    environment.insert("GIT_CONFIG_NOSYSTEM".into(), "1".into());
    environment.insert(
        "GIT_CONFIG_GLOBAL".into(),
        root.path().join("absent-config").into_os_string(),
    );
    environment.insert("GIT_ALLOW_PROTOCOL".into(), "file".into());
    let output = pull_fetch_command(&program, &prepared, "origin", &environment)
        .expect("bound parent fetch")
        .output()
        .await
        .expect("parent fetch");
    assert!(output.status.success(), "{output:?}");
    assert!(
        !has_child_object(&prepared),
        "child effects await separate admission"
    );
    assert_eq!(
        git(&native, &["rev-parse", "refs/remotes/origin/main"]),
        git(&prepared, &["rev-parse", "refs/remotes/origin/main"]),
    );
}

#[tokio::test]
async fn pull_fetch_command_preserves_current_branch_update_for_later_reconciliation() {
    let root = tempfile::tempdir().expect("fixture");
    let source = root.path().join("source");
    let local = root.path().join("local");
    std::fs::create_dir(&source).expect("source");
    let git = |cwd: &Path, args: &[&str]| {
        let output = std::process::Command::new("git")
            .current_dir(cwd)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", root.path().join("absent-config"))
            .args(args)
            .output()
            .expect("fixture Git");
        assert!(output.status.success(), "{args:?}: {output:?}");
        output.stdout
    };
    git(&source, &["init", "-b", "Topic"]);
    std::fs::write(source.join("tracked"), b"before\n").expect("base file");
    git(&source, &["add", "tracked"]);
    git(
        &source,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgSign=false",
            "commit",
            "-m",
            "base",
        ],
    );
    git(
        root.path(),
        &[
            "clone",
            source.to_str().expect("source path"),
            local.to_str().expect("local path"),
        ],
    );
    git(
        &local,
        &[
            "config",
            "--replace-all",
            "remote.origin.fetch",
            "+refs/heads/Topic:refs/heads/Topic",
        ],
    );
    std::fs::write(source.join("tracked"), b"after\n").expect("remote update");
    git(&source, &["add", "tracked"]);
    git(
        &source,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgSign=false",
            "commit",
            "-m",
            "remote",
        ],
    );
    let expected = git(&source, &["rev-parse", "HEAD"]);
    let program = resolve_git(&execution_environment()).expect("absolute Git");
    let mut environment = trace_suppressed_environment(execution_environment());
    environment.insert("GIT_CONFIG_NOSYSTEM".into(), "1".into());
    environment.insert(
        "GIT_CONFIG_GLOBAL".into(),
        root.path().join("absent-config").into_os_string(),
    );
    environment.insert("GIT_TERMINAL_PROMPT".into(), "0".into());
    let mut command =
        pull_fetch_command(&program, &local, "origin", &environment).expect("bound fetch");
    let output = command.output().await.expect("execute fixed fetch");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(git(&local, &["rev-parse", "HEAD"]), expected);
    assert_eq!(
        std::fs::read(local.join("tracked")).expect("worktree"),
        b"before\n"
    );
}

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

#[test]
fn pull_attribute_evidence_detects_creation_and_replacement() {
    let root = tempfile::tempdir().expect("fixture");
    let path = root.path().canonicalize().expect("root").join("attributes");
    let budget = || snapshot::Budget {
        entries: 32,
        bytes: 4096,
    };
    let missing = PullAttribute::capture(&path, &mut budget()).expect("missing");
    missing.revalidate(&mut budget()).expect("unchanged");
    std::fs::write(&path, "* -filter\n").expect("create");
    assert!(missing.revalidate(&mut budget()).is_err());
    let present = PullAttribute::capture(&path, &mut budget()).expect("present");
    std::fs::write(&path, "* filter=external\n").expect("change");
    assert!(present.revalidate(&mut budget()).is_err());
}

#[cfg(unix)]
#[test]
fn pull_attribute_evidence_refuses_parent_and_leaf_links() {
    let root = tempfile::tempdir().expect("fixture");
    let root = root.path().canonicalize().expect("root");
    std::fs::create_dir(root.join("rules")).expect("directory");
    let path = root.join("rules/attributes");
    std::fs::write(&path, "* -filter\n").expect("rules");
    let budget = || snapshot::Budget {
        entries: 32,
        bytes: 4096,
    };
    let evidence = PullAttribute::capture(&path, &mut budget()).expect("capture");
    std::fs::rename(root.join("rules"), root.join("moved")).expect("move");
    std::os::unix::fs::symlink(root.join("moved"), root.join("rules")).expect("parent link");
    assert!(evidence.revalidate(&mut budget()).is_err());
    assert!(PullAttribute::capture(&path, &mut budget()).is_err());
    let leaf = root.join("leaf");
    std::os::unix::fs::symlink(root.join("moved/attributes"), &leaf).expect("leaf link");
    assert!(PullAttribute::capture(&leaf, &mut budget()).is_err());
}

#[test]
fn pull_source_evidence_detects_content_and_missing_path_changes() {
    let root = tempfile::tempdir().expect("fixture");
    std::fs::write(root.path().join("file"), "before").expect("source");
    let evidence =
        PullSources::capture(root.path(), vec!["file".into(), "missing".into()]).expect("capture");
    evidence.revalidate(root.path()).expect("unchanged");
    std::fs::write(root.path().join("file"), "after").expect("change");
    assert!(evidence.revalidate(root.path()).is_err());
    std::fs::write(root.path().join("file"), "before").expect("restore");
    std::fs::write(root.path().join("missing"), "created").expect("create");
    assert!(evidence.revalidate(root.path()).is_err());
}

#[cfg(unix)]
#[test]
fn pull_source_evidence_refuses_redirected_parent() {
    let root = tempfile::tempdir().expect("fixture");
    let outside = tempfile::tempdir().expect("outside");
    std::fs::create_dir(root.path().join("dir")).expect("directory");
    std::fs::write(root.path().join("dir/file"), "same").expect("source");
    std::fs::write(outside.path().join("file"), "same").expect("outside source");
    let evidence = PullSources::capture(root.path(), vec!["dir/file".into()]).expect("capture");
    std::fs::rename(root.path().join("dir"), root.path().join("old")).expect("move");
    std::os::unix::fs::symlink(outside.path(), root.path().join("dir")).expect("redirect");
    assert!(evidence.revalidate(root.path()).is_err());
    assert!(PullSources::capture(root.path(), vec!["dir/file".into()]).is_err());
}

#[test]
fn pull_sparse_tree_listing_rejects_malformed_and_oversize_records() {
    let prefix = b"dir/";
    let good = b"100644 blob aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\tfile\0";
    assert_eq!(
        pull_sparse_names(prefix, good).expect("good"),
        b"dir/file\0"
    );
    for bad in [
        b"100644 blob zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz\tfile\0".as_slice(),
        b"100644 blob 0000000000000000000000000000000000000000\tfile\0",
        b"100644 tree aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\tdir\0",
        b"100644 blob aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\t../file\0",
        b"100644 blob aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\tfile",
    ] {
        assert!(pull_sparse_names(prefix, bad).is_err());
    }
    let large = vec![b'x'; 16 * 1024 * 1024 + 1];
    assert!(pull_sparse_names(prefix, &large).is_err());
}

#[test]
fn pull_tracked_inventory_only_constructs_literal_paths() {
    let root = tempfile::tempdir().expect("fixture");
    let names = pull_tracked_read_paths(root.path(), b"missing/leaf\0-leading\0").expect("names");
    assert!(names.contains(&root.path().join("missing/leaf")));
    assert!(!root.path().join("missing").exists());
    for bad in [
        b"../outside\0".as_slice(),
        b"/absolute\0",
        b"a//b\0",
        b"a/./b\0",
        b"unterminated",
    ] {
        assert!(pull_tracked_read_paths(root.path(), bad).is_err());
    }
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
        AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
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
    let mut daemon = tokio::process::Command::new("git")
        .args([
            "daemon",
            "--reuseaddr",
            "--listen=127.0.0.1",
            "--export-all",
        ])
        .arg(format!("--port={port}"))
        .arg(format!(
            "--base-path={}",
            remote.path().parent().expect("parent").display()
        ))
        .arg(remote.path())
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
            assert!(daemon.try_wait().expect("daemon state").is_none());
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
            AddPlan::prepare_pull(local.path(), &serde_json::json!({}), ToolProvenance::Native)
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
    daemon.kill().await.expect("stop daemon");
    daemon.wait().await.expect("reap daemon");
}

#[tokio::test]
async fn pull_postfetch_rebuilds_evidence_before_selecting_integration() {
    for source in [
        "worktree",
        "index",
        "head",
        "absent",
        "linked-tamper",
        "linked-budget",
    ] {
        let root = fixture();
        let current = b"[submodule \"current\"]\npath = old\npath = current/child\nfetchRecurseSubmodules = true\nfetchRecurseSubmodules = on-demand\n";
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .current_dir(root.path())
                .args(["-c", "maintenance.autoDetach=false", "-c", "gc.autoDetach=false"])
                .args(args)
                .output()
                .expect("Git");
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            output.stdout
        };
        if source == "head" {
            std::fs::write(root.path().join(".gitmodules"), current).expect("HEAD modules");
            git(&["add", ".gitmodules"]);
        }
        git(&[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "--allow-empty",
            "-m",
            "initial",
        ]);
        git(&["config", "remote.origin.url", "git://example.invalid/repo"]);
        std::fs::write(root.path().join("diff-order"), "missing/*\n").expect("order file");
        git(&["config", "diff.orderFile", "diff-order"]);
        // A dangling commit is already stored but is not reachable from any
        // pre-fetch ref. Fetching it must include it in child-graph discovery
        // while excluding its already-reachable parent.
        let child_oid = git(&["rev-parse", "HEAD"]);
        let child_oid = std::str::from_utf8(&child_oid).expect("child OID").trim();
        git(&[
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("160000,{child_oid},missing/child"),
        ]);
        std::fs::write(
            root.path().join(".gitmodules"),
            "[submodule \"child\"]\npath = missing/child\nurl = git://example.invalid/child\n",
        )
        .expect("module declaration");
        git(&["add", ".gitmodules"]);
        let tree_oid = git(&["write-tree"]);
        let tree_oid = std::str::from_utf8(&tree_oid).expect("tree OID").trim();
        let target = git(&[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit-tree",
            tree_oid,
            "-p",
            "HEAD",
            "-m",
            "new fetched history",
        ]);
        git(&["reset", "--mixed", "HEAD"]);
        std::fs::remove_file(root.path().join(".gitmodules")).expect("remove fixture config");
        git(&[
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("160000,{child_oid},current/child"),
        ]);
        match source {
            "worktree" => {
                std::fs::write(root.path().join(".gitmodules"), current).expect("worktree modules")
            }
            "index" => {
                std::fs::write(root.path().join(".gitmodules"), current).expect("index modules");
                git(&["add", ".gitmodules"]);
                std::fs::remove_file(root.path().join(".gitmodules"))
                    .expect("missing worktree modules");
            }
            "head" => {
                git(&["rm", "--cached", ".gitmodules"]);
            }
            _ => {}
        }
        let target = std::str::from_utf8(&target).expect("object ID").trim();
        git(&["config", "submodule.child.active", "true"]);
        let mut plan = Box::new(
            AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
                .expect("Pull"),
        );
        let mut history = talos_core::tool::PreparedStageHistory::default();
        let mut recorded_stages = 0;
        assert!(history.record_profile(&plan.identity, &plan.facets));
        recorded_stages += 1;
        plan = plan.discover().await.expect("configuration");
        for _ in 0..8 {
            if plan.phase == Phase::PullFetch {
                break;
            }
            assert!(history.record_profile(&plan.identity, &plan.facets));
            recorded_stages += 1;
            let admission = proof(&plan)
                .consume_for_prepared(&plan.request(), &plan.identity)
                .expect("proof");
            plan = plan
                .discover_pull_prefetch(&admission)
                .await
                .expect("prefetch");
        }
        assert!(plan.phase == Phase::PullFetch);
        assert!(history.record_profile(&plan.identity, &plan.facets));
        recorded_stages += 1;
        assert!(
            plan.pull_attributes
                .iter()
                .any(|attribute| attribute.path == plan.repository.workdir.join("diff-order"))
        );

        // Exercise the post-fetch boundary with actual Git metadata mutation,
        // independently of a network daemon or successful transport fixture.
        git(&["fetch", "--no-recurse-submodules", ".", target]);
        assert!(
            plan.revalidate().is_err(),
            "old FETCH_HEAD evidence must expire"
        );
        plan.facets
            .retain(|facet| matches!(facet.nature, ToolNature::Read | ToolNature::Execute));
        plan.pull_inventory = None;
        plan.identity = PreparedPlanIdentity::new();
        plan.phase = Phase::PullPostFetch;
        let external = tempfile::tempdir().expect("postfetch external common");
        let pointer = root.path().join(".git/worktrees/linked/commondir");
        if source.starts_with("linked-") {
            std::fs::create_dir_all(pointer.parent().expect("registration")).expect("registration");
            std::fs::write(
                &pointer,
                format!(
                    "{}\n",
                    external.path().canonicalize().expect("external").display()
                ),
            )
            .expect("external common pointer");
        }
        if source == "linked-budget" {
            let deep = external
                .path()
                .canonicalize()
                .expect("canonical external root")
                .join("refs/heads/a/b/c/d/e/f/g/h/i/j/topic");
            std::fs::create_dir_all(deep.parent().expect("parents")).expect("deep refs");
            std::fs::write(&deep, format!("{target}\n")).expect("external ref");
            let fetched =
                std::fs::read(root.path().join(".git/FETCH_HEAD")).expect("fetch effects");
            while history.record_profile(&plan.identity, &plan.facets) {
                recorded_stages += 1;
                let tokens = proof(&plan)
                    .consume_for_prepared(&plan.request(), &plan.identity)
                    .expect("postfetch frontier admission");
                plan = plan
                    .discover_pull_postfetch(&tokens)
                    .await
                    .unwrap_or_else(|error| {
                        panic!("{source} frontier at stage {recorded_stages}: {error}")
                    });
                assert!(plan.phase == Phase::PullPostFetch);
            }
            assert_eq!(recorded_stages, 8);
            assert!(
                !plan
                    .pull_inventory
                    .as_ref()
                    .expect("retained frontier")
                    .paths()
                    .contains(&deep)
            );
            assert_eq!(
                std::fs::read(root.path().join(".git/FETCH_HEAD")).expect("retained fetch"),
                fetched
            );
            continue;
        }
        let admission = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("fresh proof");
        assert!(!admission.is_empty());
        let next = plan
            .discover_pull_postfetch(&admission)
            .await
            .expect("postfetch");
        if source == "linked-tamper" {
            assert!(next.phase == Phase::PullPostFetch);
            assert!(next.pull_inventory.is_some());
            let tokens = proof(&next)
                .consume_for_prepared(&next.request(), &next.identity)
                .expect("target admission");
            std::fs::write(&pointer, "../..\n").expect("change retained pointer");
            assert!(next.discover_pull_postfetch(&tokens).await.is_err());
            assert!(root.path().join(".git/FETCH_HEAD").exists());
            continue;
        }
        assert!(
            next.phase == Phase::PullChildIdentity,
            "source={source}, activity={:?}",
            next.pull_changed_children
                .values()
                .map(|child| &child.activity)
                .collect::<Vec<_>>()
        );
        assert!(next.pull_activity_globals.is_empty());
        for path in [
            next.repository.workdir.join("missing/child"),
            next.repository.workdir.join("missing/child/.git"),
        ] {
            assert!(next.facets.iter().any(|facet| {
                facet.nature == ToolNature::Read && facet.resource.as_deref() == path.to_str()
            }));
            assert!(next.child_read(&path, &admission).is_err());
        }
        assert!(next.child_locations.is_empty());
        assert_eq!(next.pull_new_commits, [target]);
        assert!(next.pull_child_trees.contains_key(target));
        assert_eq!(
            next.pull_index_children
                .get("current/child")
                .map(String::as_str),
            Some(child_oid)
        );
        assert!(!root.path().join("current").exists());
        let child_tree = &next.pull_child_trees[target];
        assert_eq!(
            child_tree.gitlinks.get("missing/child").map(String::as_str),
            Some(child_oid)
        );
        assert_eq!(child_tree.changed_gitlinks, child_tree.gitlinks);
        let changed = &next.pull_changed_children["child"];
        assert_eq!(changed.commit, target);
        assert_eq!(changed.path, "missing/child");
        assert!(changed.objects.contains(child_oid));
        assert_eq!(
            changed.effective_recursion,
            Some(super::super::pull_children::Recursion::OnDemand)
        );
        assert!(child_tree.modules_blob.is_some());
        assert_eq!(
            child_tree
                .module_names
                .get("missing/child")
                .map(String::as_str),
            Some("child")
        );
        assert!(!root.path().join("missing").exists());
        let modules = next
            .pull_current_modules
            .as_ref()
            .expect("current declarations");
        if source == "absent" {
            assert!(modules.module_names.is_empty());
        } else {
            assert_eq!(modules.module_names["current/child"], "current", "{source}");
            assert_eq!(
                modules.module_recursion["current"],
                super::super::pull_children::Recursion::OnDemand
            );
        }
        assert_eq!(
            root.path().join(".gitmodules").exists(),
            source == "worktree"
        );
        match next.pull_integration.as_ref().expect("integration") {
            super::super::pull_state::Integration::Merge { heads, .. } => {
                let fetched = git(&["rev-parse", "FETCH_HEAD^{commit}"]);
                assert_eq!(heads, &[String::from_utf8(fetched).expect("OID").trim().to_owned()]);
            }
            other => panic!("expected merge plan, got {other:?}"),
        }
        next.revalidate().expect("fresh evidence");
        std::fs::write(root.path().join(".gitmodules"), b"changed after admission")
            .expect("tamper modules");
        assert!(
            next.revalidate().is_err(),
            "{source} module evidence must expire"
        );
        if source == "worktree" {
            std::fs::write(root.path().join(".gitmodules"), current).expect("restore modules");
        } else {
            std::fs::remove_file(root.path().join(".gitmodules")).expect("restore absence");
        }
        next.revalidate().expect("restored module evidence");
        std::fs::write(root.path().join(".git/FETCH_HEAD"), "tampered").expect("tamper");
        assert!(next.revalidate().is_err());
    }
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
        AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
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
        AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
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
async fn pull_rejects_legacy_destinations_before_configuration_and_after_fetch_planning() {
    for directory in ["remotes", "branches"] {
        for after_planning in [false, true] {
            let root = fixture();
            let input = serde_json::json!({"remote":"origin"});
            let mut plan = Box::new(
                AddPlan::prepare_pull(root.path(), &input, ToolProvenance::Native)
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
            let PreparedNormalStage::Complete(output) =
                plan.execute_stage(Box::new(admission)).await
            else {
                panic!("legacy destination continued");
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
            AddPlan::prepare_pull(root.path(), &input, ToolProvenance::Native).expect("Pull"),
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

#[tokio::test]
async fn pull_config_admission_cannot_authorize_graph_probes() {
    let root = fixture();
    let input = serde_json::json!({});
    let tool = super::super::GitPullTool::new(root.path().to_path_buf());
    let prepared = tool
        .prepare_normalized_with_provenance(&input, ToolProvenance::Native)
        .expect("prepare")
        .expect("prepared Pull");
    #[cfg(unix)]
    assert!(prepared.cleanup_receipt().is_some());
    let PreparedNormalStage::Complete(denied) = prepared.execute_stage(Box::new(())).await else {
        panic!("missing admission continued")
    };
    assert!(denied.result.is_error);

    let plan = AddPlan::prepare_pull(root.path(), &input, ToolProvenance::Native).expect("Pull");
    let old_proof = proof(&plan);
    let plan = Box::new(plan).discover().await.expect("configuration");
    let PreparedNormalStage::Complete(output) = plan.execute_stage(Box::new(old_proof)).await
    else {
        panic!("configuration proof authorized graph inspection")
    };
    assert!(output.result.is_error && output.result.content.contains("does not authorize"));

    let plan = AddPlan::prepare_pull(root.path(), &input, ToolProvenance::Native).expect("Pull");
    let mut history = talos_core::tool::PreparedStageHistory::default();
    assert!(history.record_profile(&plan.identity, &plan.facets));
    let mut plan = Box::new(plan).discover().await.expect("config");
    assert!(plan.pull_evidence.is_none());
    assert!(plan.facets.iter().any(|facet| {
        facet.nature == ToolNature::Read
            && facet.resource.as_deref() == plan.repository.common_dir.join("info/grafts").to_str()
    }));
    for _ in 0..5 {
        assert!(history.record_profile(&plan.identity, &plan.facets));
        let admitted = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("fresh exact inventory admission");
        plan = plan
            .discover_pull_prefetch(&admitted)
            .await
            .expect("inventory/prefetch");
        if plan.phase == Phase::PullFetch {
            break;
        }
        if plan
            .pull_graph_paths
            .as_ref()
            .is_some_and(|paths| paths.contains(&plan.repository.common_dir.join("info/grafts")))
        {
            assert!(plan.pull_evidence.is_none());
            assert!(plan.facets.iter().any(|facet| {
                facet.nature == ToolNature::Read
                    && facet.resource.as_deref()
                        == plan.repository.common_dir.join("info/grafts").to_str()
            }));
        }
        assert!(plan.pull_before.is_none(), "no probes during inventory");
    }
    assert!(plan.phase == Phase::PullFetch);
    assert!(history.record_profile(&plan.identity, &plan.facets));
    assert!(!root.path().join(".git/FETCH_HEAD").exists());
}

#[tokio::test]
async fn pull_sparse_index_requires_object_discovery_before_source_capture() {
    let root = fixture();
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .current_dir(root.path())
            .args(args)
            .output()
            .expect("Git fixture");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(&["config", "user.name", "Fixture"]);
    git(&["config", "user.email", "fixture@example.invalid"]);
    git(&["add", "file"]);
    git(&["commit", "-m", "base"]);
    std::fs::create_dir(root.path().join("kept")).expect("kept");
    std::fs::write(root.path().join("kept/child"), "child").expect("child");
    std::fs::create_dir(root.path().join("excluded")).expect("excluded");
    std::fs::write(root.path().join("excluded/child"), "outside cone").expect("excluded child");
    // More than 16 KiB of ls-tree output; identical blobs keep the object
    // inventory small so this exercises listing capacity, not object count.
    for number in 0..300 {
        std::fs::write(
            root.path().join(format!("excluded/entry-{number:04}")),
            "outside cone",
        )
        .expect("large sparse tree entry");
    }
    git(&["add", "."]);
    git(&["commit", "-m", "child"]);
    git(&["sparse-checkout", "init", "--cone"]);
    git(&["sparse-checkout", "set", "--sparse-index", "kept"]);
    git(&["config", "index.sparse", "true"]);
    git(&["config", "pull.rebase", "true"]);
    git(&["branch", "-m", "Topic"]);
    git(&["config", "remote.origin.url", "/missing-fixture-remote"]);
    git(&[
        "config",
        "remote.origin.fetch",
        "+refs/heads/*:refs/remotes/origin/*",
    ]);
    git(&["config", "branch.Topic.remote", "origin"]);
    git(&["config", "branch.Topic.merge", "refs/heads/Topic"]);
    git(&["update-ref", "refs/remotes/origin/Topic", "HEAD"]);
    let index = super::super::pull_state::decode_index(&root.path().join(".git/index"), 40)
        .expect("sparse index")
        .0;
    assert!(
        index
            .entries()
            .iter()
            .any(|entry| entry.mode.bits() == 0o040000)
    );
    let plan = AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
        .expect("Pull");
    let mut history = talos_core::tool::PreparedStageHistory::default();
    assert!(history.record_profile(&plan.identity, &plan.facets));
    let mut plan = Box::new(plan).discover().await.expect("configuration");
    let original_index = std::fs::read(root.path().join(".git/index")).expect("index bytes");
    let mut saw_descendant_continuation = false;
    let mut stages = 1;
    let mut stage_trace = Vec::new();
    for _ in 0..8 {
        assert!(history.record_profile(&plan.identity, &plan.facets));
        stages += 1;
        let admission = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("admission");
        match plan.discover_pull_prefetch(&admission).await {
            Ok(next) => {
                stage_trace.push((
                    stages,
                    next.pull_graph_paths.is_some(),
                    next.pull_inventory
                        .as_ref()
                        .is_some_and(|inventory| inventory.complete()),
                    next.facets.len(),
                ));
                let descendant = next.repository.workdir.join("excluded/child");
                if next
                    .facets
                    .iter()
                    .any(|facet| facet.resource.as_deref() == descendant.to_str())
                    && !admission.iter().any(|token| {
                        token.authorizes_path(
                            "git_pull",
                            ToolNature::Read,
                            &next.workspace,
                            descendant.to_str().expect("path"),
                        )
                    })
                {
                    assert!(next.pull_sources.is_none());
                    assert!(next.phase == Phase::PullPreparation);
                    saw_descendant_continuation = true;
                }
                if next.phase == Phase::PullFetch {
                    assert!(
                        stages <= 6,
                        "sparse preparation must reserve fetch and integration: {stages}; trace {stage_trace:?}"
                    );
                    assert!(saw_descendant_continuation);
                    assert!(next.facets.iter().any(|facet| {
                        facet.nature == ToolNature::Write
                            && facet.resource.as_deref() == Some("git:pull-repository-metadata")
                    }));
                    assert!(next.facets.iter().any(|facet| {
                        facet.nature == ToolNature::Write
                            && facet.resource.as_deref() == next.repository.git_dir.to_str()
                    }));
                    assert!(!next.facets.iter().any(|facet| {
                        facet.nature == ToolNature::Write
                            && facet.resource.as_deref() == next.repository.workdir.to_str()
                    }));
                    assert!(history.record_profile(&next.identity, &next.facets));
                    assert!(
                        history.record_profile(&PreparedPlanIdentity::new(), &next.facets),
                        "integration must fit actual shared history after fetch reservation"
                    );
                    assert_eq!(
                        std::fs::read(root.path().join(".git/index")).expect("index after"),
                        original_index
                    );
                    assert!(next.pull_graph_paths.as_ref().is_some_and(|paths| {
                        paths.iter().any(|path| path.ends_with("excluded/child"))
                    }));
                    return;
                }
                plan = next
            }
            Err(error) => {
                panic!("sparse expansion failed: {error}");
            }
        }
    }
    panic!("sparse index did not reach prepared fetch");
}

#[tokio::test]
async fn pull_rebase_clean_probes_preserve_index_and_touch_semantics() {
    for state in [
        "clean",
        "touch",
        "dirty",
        "staged",
        "autostash",
        "pull-autostash",
        "pull-no-autostash",
        "symbolic-policy",
        "external-orderfile",
        "binary",
        "deleted",
        "mode",
        "gitlink",
    ] {
        let root = fixture();
        let order_root = tempfile::tempdir().expect("external order fixture");
        let order_path = order_root
            .path()
            .canonicalize()
            .expect("order root")
            .join("order");
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
        git(&["config", "user.name", "Fixture"]);
        git(&["config", "user.email", "fixture@example.invalid"]);
        git(&["config", "commit.gpgSign", "false"]);
        git(&["add", "file"]);
        git(&["commit", "-m", "base"]);
        git(&["config", "pull.rebase", "true"]);
        if state == "external-orderfile" {
            std::fs::write(&order_path, "file\n").expect("order file");
            git(&[
                "config",
                "diff.orderFile",
                order_path.to_str().expect("path"),
            ]);
        }
        if state == "symbolic-policy" {
            git(&["branch", "-m", "Topic"]);
            git(&["symbolic-ref", "refs/heads/Alias", "refs/heads/Topic"]);
            git(&["symbolic-ref", "HEAD", "refs/heads/Alias"]);
            git(&["config", "branch.Alias.rebase", "false"]);
            git(&["config", "branch.Topic.rebase", "true"]);
        }
        if state == "gitlink" {
            let head = std::process::Command::new("git")
                .current_dir(root.path())
                .args(["rev-parse", "HEAD"])
                .output()
                .expect("head");
            assert!(head.status.success());
            let oid = String::from_utf8(head.stdout).expect("oid");
            git(&[
                "update-index",
                "--add",
                "--cacheinfo",
                &format!("160000,{},child", oid.trim()),
            ]);
        }
        if state == "binary" {
            std::fs::write(root.path().join("file"), b"changed\0binary").expect("binary");
        } else if state == "deleted" {
            std::fs::remove_file(root.path().join("file")).expect("delete");
        } else if state == "mode" {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                git(&["config", "core.fileMode", "true"]);
                std::fs::set_permissions(
                    root.path().join("file"),
                    std::fs::Permissions::from_mode(0o755),
                )
                .expect("mode");
            }
            #[cfg(not(unix))]
            continue;
        }
        if state == "touch" {
            let file = std::fs::OpenOptions::new()
                .write(true)
                .open(root.path().join("file"))
                .expect("file");
            file.set_modified(
                std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1234567890),
            )
            .expect("mtime");
        } else if matches!(
            state,
            "dirty" | "staged" | "autostash" | "pull-autostash" | "pull-no-autostash"
        ) {
            std::fs::write(root.path().join("file"), "changed\n").expect("change");
            if state == "staged" {
                git(&["add", "file"]);
            }
            if state == "autostash" {
                git(&["config", "rebase.autostash", "true"]);
            }
            if state == "pull-autostash" {
                git(&["config", "rebase.autostash", "false"]);
                git(&["config", "pull.autostash", "true"]);
            }
            if state == "pull-no-autostash" {
                git(&["config", "rebase.autostash", "true"]);
                git(&["config", "pull.autostash", "false"]);
            }
        }
        let index = std::fs::read(root.path().join(".git/index")).expect("index");
        let plan =
            AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
                .expect("prepare");
        let mut plan = Box::new(plan).discover().await.expect("configuration");
        let mut outcome = None;
        let mut late_source_admission = false;
        for _ in 0..8 {
            let admission = proof(&plan)
                .consume_for_prepared(&plan.request(), &plan.identity)
                .expect("admission");
            match plan.discover_pull_prefetch(&admission).await {
                Ok(next) if next.phase == Phase::PullFetch => {
                    if state == "external-orderfile" {
                        assert!(
                            next.facets
                                .iter()
                                .any(|facet| facet.nature == ToolNature::Read
                                    && facet.resource.as_deref() == order_path.to_str())
                        );
                        assert!(
                            next.pull_attributes
                                .iter()
                                .any(|entry| entry.path == order_path)
                        );
                        std::fs::write(&order_path, "changed\n").expect("change order file");
                        assert!(
                            next.revalidate().is_err(),
                            "external rule change must invalidate plan"
                        );
                    }
                    if state == "symbolic-policy" {
                        assert!(
                            late_source_admission,
                            "alias policy must require fresh source admission"
                        );
                        assert!(!next.pull_attribute_paths.is_empty());
                        assert_eq!(next.pull_attributes.len(), next.pull_attribute_paths.len());
                    }
                    if matches!(state, "autostash" | "pull-autostash") {
                        let source = next.repository.workdir.join("file");
                        assert!(
                            !next.facets.iter().any(|facet| {
                                facet.nature == ToolNature::Read
                                    && facet.resource.as_deref() == source.to_str()
                            }),
                            "autostash must not request prefetch clean-source Read"
                        );
                        assert!(next.pull_sources.is_none());
                        assert!(next.pull_attributes.is_empty());
                    }
                    outcome = Some(Ok(()));
                    break;
                }
                Ok(next) => {
                    if state == "symbolic-policy" {
                        let source = next.repository.workdir.join("file");
                        if next
                            .facets
                            .iter()
                            .any(|facet| facet.resource.as_deref() == source.to_str())
                            && !admission.iter().any(|token| {
                                token.authorizes_path(
                                    "git_pull",
                                    ToolNature::Read,
                                    &next.workspace,
                                    source.to_str().expect("path"),
                                )
                            })
                        {
                            assert!(next.pull_sources.is_none());
                            assert!(next.pull_attributes.is_empty());
                            late_source_admission = true;
                        }
                    }
                    plan = next;
                }
                Err(error) => {
                    outcome = Some(Err(error));
                    break;
                }
            }
        }
        let outcome = outcome.expect("bounded preparation");
        if matches!(
            state,
            "dirty" | "staged" | "binary" | "deleted" | "mode" | "pull-no-autostash"
        ) {
            let error = outcome.expect_err("dirty source must refuse").to_string();
            assert!(
                error.contains(if state == "staged" {
                    "clean index"
                } else {
                    "clean worktree"
                }),
                "{state}: {error}"
            );
        } else {
            outcome.unwrap_or_else(|error| panic!("{state}: {error}"));
        }
        assert_eq!(
            std::fs::read(root.path().join(".git/index")).expect("index after"),
            index,
            "{state}"
        );
        assert!(!root.path().join(".git/index.lock").exists());
        assert!(!root.path().join(".git/FETCH_HEAD").exists());
    }
}

#[tokio::test]
async fn pull_rebase_filter_refuses_without_running_helper() {
    let root = fixture();
    for args in [
        vec!["config", "user.name", "Fixture"],
        vec!["config", "user.email", "fixture@example.invalid"],
        vec!["config", "commit.gpgSign", "false"],
        vec!["add", "file"],
        vec!["commit", "-m", "base"],
        vec!["config", "pull.rebase", "true"],
        vec!["config", "filter.probe.clean", "touch helper-ran; cat"],
    ] {
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
    }
    std::fs::write(root.path().join(".gitattributes"), "file filter=probe\n").expect("rules");
    let plan = AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
        .expect("prepare");
    let mut plan = Box::new(plan).discover().await.expect("configuration");
    let mut refused = false;
    for _ in 0..8 {
        let admission = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("fresh admission");
        match plan.discover_pull_prefetch(&admission).await {
            Ok(next) => {
                assert!(
                    next.phase != Phase::PullFetch,
                    "filtered source reached fetch"
                );
                plan = next;
            }
            Err(error) => {
                assert!(error.to_string().contains("configured filter"), "{error}");
                refused = true;
                break;
            }
        }
    }
    assert!(refused);
    assert!(!root.path().join("helper-ran").exists());
    assert!(!root.path().join(".git/FETCH_HEAD").exists());
}

#[tokio::test]
async fn pull_graph_read_requires_closed_capability_after_configuration() {
    let root = fixture();
    let plan = AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
        .expect("Pull");
    let mut plan = Box::new(plan).discover().await.expect("configuration");
    plan.facets
        .retain(|facet| facet.resource.as_deref() != Some("git:pull-repository-metadata"));
    let admission = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("exact-path admission without closed capability");
    let Err(error) = plan.discover_pull_prefetch(&admission).await else {
        panic!("missing metadata capability continued");
    };
    assert!(
        error
            .to_string()
            .contains("dedicated metadata Read admission")
    );
    assert!(!root.path().join(".git/FETCH_HEAD").exists());
}

#[tokio::test]
async fn pull_unapproved_resolution_does_not_inspect_missing_external_target() {
    let root = fixture();
    let mut plan = Box::new(
        AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
            .expect("Pull"),
    )
    .discover()
    .await
    .expect("configuration");
    let common = plan.repository.common_dir.clone();
    let registration = common.join("worktrees/linked");
    std::fs::create_dir_all(&registration).expect("registration");
    let missing = root.path().join("missing-external-common");
    std::fs::write(
        registration.join("commondir"),
        format!("{}\n", missing.display()),
    )
    .expect("pointer");
    let mut inventory = super::super::pull_inventory::Inventory::new(&common, &common);
    while !inventory.complete() {
        inventory.advance(|_| true).expect("approved metadata");
    }
    inventory.bind_references(&common, &common).expect("source evidence");
    for _ in 0..2 {
        assert!(AddPlan::advance_pull_worktree_targets(
            &mut inventory,
            &common,
            &mut plan.facets,
            &plan.workspace,
            &[],
            false,
        ).expect("unapproved capability must defer without inspecting missing target"));
        assert!(inventory.exact_paths(&common, &common).is_empty());
    }
    assert!(AddPlan::advance_pull_worktree_targets(
        &mut inventory,
        &common,
        &mut plan.facets,
        &plan.workspace,
        &[],
        true,
    ).is_err(), "approved resolution detects the missing common directory");
}

#[tokio::test]
async fn pull_parent_linked_targets_require_fresh_exact_admission() {
    let root = fixture();
    let mut plan = Box::new(
        AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
            .expect("Pull"),
    )
    .discover()
    .await
    .expect("configuration");
    let common = plan.repository.common_dir.clone();
    let registration = common.join("worktrees/linked");
    std::fs::create_dir_all(&registration).expect("registration");
    std::fs::write(registration.join("gitdir"), "/absent-checkout/.git\n")
        .expect("absolute backlink remains text");
    let external = tempfile::tempdir().expect("external common");
    let external = external.path().canonicalize().expect("canonical external");
    std::fs::create_dir_all(external.join("refs/heads")).expect("external refs");
    let reference = external.join("refs/heads/topic");
    std::fs::write(&reference, format!("{}\n", "1".repeat(40))).expect("reference");
    std::fs::write(
        registration.join("commondir"),
        format!("{}\n", external.display()),
    )
    .expect("common pointer");
    let mut inventory = super::super::pull_inventory::Inventory::new(&common, &common);
    let old = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("original grants");
    assert!(
        !plan
            .advance_pull_parent_inventory(&mut inventory, &old)
            .expect("targets")
    );
    assert!(AddPlan::pull_resolution_admitted(&plan.facets));
    assert!(inventory.exact_paths(&common, &common).is_empty());
    assert!(!inventory.paths().contains(&reference));
    // Only the newly approved dedicated capability permits chain inspection.
    let resolution = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("resolution capability");
    assert!(
        !plan
            .advance_pull_parent_inventory(&mut inventory, &resolution)
            .expect("resolved target plan")
    );
    assert!(inventory.exact_paths(&common, &common).contains(&external));
    // Capability approval does not grant the newly resolved target's contents.
    assert!(
        !plan
            .advance_pull_parent_inventory(&mut inventory, &resolution)
            .expect("target still lacks exact admission")
    );
    assert!(!inventory.paths().contains(&reference));
    for _ in 0..8 {
        let tokens = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("fresh frontier grants");
        if plan
            .advance_pull_parent_inventory(&mut inventory, &tokens)
            .expect("advance")
        {
            break;
        }
    }
    assert!(inventory.complete());
    assert!(inventory.paths().contains(&reference));
    inventory.revalidate().expect("bound contents");
    std::fs::write(&reference, format!("{}\n", "2".repeat(40))).expect("change contents");
    assert!(inventory.revalidate().is_err());
}

#[tokio::test]
async fn pull_tracking_inventory_uses_shared_stage_budget() {
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
    git(&["config", "user.name", "Fixture"]);
    git(&["config", "user.email", "fixture@example.invalid"]);
    git(&["config", "commit.gpgSign", "false"]);
    git(&["add", "file"]);
    git(&["commit", "-m", "base"]);
    git(&["config", "remote.origin.url", "/missing-fixture-remote"]);
    git(&[
        "config",
        "remote.origin.fetch",
        "+refs/heads/*:refs/remotes/origin/*",
    ]);
    git(&["config", "branch.Topic.remote", "origin"]);
    git(&["config", "branch.Topic.merge", "refs/heads/Topic"]);
    git(&["config", "pull.rebase", "true"]);
    git(&["update-ref", "refs/remotes/origin/Topic", "HEAD"]);
    // Namespace depth must not consume prepared-runner stages now that
    // graph inspection uses the dedicated metadata Read capability.
    git(&["update-ref", "refs/heads/a/b/c/d/e/f/g/h/i/j/deep", "HEAD"]);
    let absent_tracking_log = root.path().join(".git/logs/refs/remotes/origin/Topic");
    std::fs::remove_file(&absent_tracking_log).expect("absent tracking reflog");
    let plan = AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
        .expect("Pull");
    let mut history = talos_core::tool::PreparedStageHistory::default();
    assert!(history.record_profile(&plan.identity, &plan.facets));
    let mut plan = Box::new(plan).discover().await.expect("configuration");
    let mut stages = 1;
    while plan.phase != Phase::PullFetch {
        assert!(
            history.record_profile(&plan.identity, &plan.facets),
            "inventory exhausted stage budget"
        );
        stages += 1;
        let admission = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("fresh admission");
        plan = plan
            .discover_pull_prefetch(&admission)
            .await
            .expect("inventory");
    }
    assert!(history.record_profile(&plan.identity, &plan.facets));
    assert!(
        stages <= 3,
        "post-fetch discovery, reconciliation and integration stages must remain available"
    );
    for path in [
        plan.repository.workdir.join(".gitattributes"),
        plan.repository.common_dir.join("info/attributes"),
    ] {
        assert!(
            plan.facets
                .iter()
                .any(|facet| facet.nature == ToolNature::Read
                    && facet.resource_kind == Some(ToolResourceKind::Path)
                    && facet.resource.as_deref() == path.to_str())
        );
    }
    assert!(history.record_profile(&PreparedPlanIdentity::new(), &plan.facets));
    assert!(!root.path().join(".git/FETCH_HEAD").exists());
    let reference = plan.repository.common_dir.join("refs/heads/Topic");
    let original_reference = std::fs::read(&reference).expect("reference");
    std::fs::write(&reference, format!("{}\n", "1".repeat(40))).expect("change reference in place");
    assert!(
        plan.revalidate().is_err(),
        "reference contents must remain bound"
    );
    std::fs::write(&reference, original_reference).expect("restore reference");
    plan.revalidate().expect("restored reference evidence");
    std::fs::write(&absent_tracking_log, b"new evidence").expect("create reflog");
    assert!(
        plan.revalidate().is_err(),
        "absent reflog must remain bound"
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
