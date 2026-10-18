#[cfg(unix)]
#[tokio::test]
async fn prepared_local_push_runs_all_eight_admitted_stages() {
    for layout in [
        "bare",
        "normal",
        "linked",
        "file",
        "multiple",
        "partial",
        "cancel",
        "maintenance",
        "maintenance-cancel",
    ] {
        let sender = fixture();
        let receiver = tempfile::tempdir().expect("receiver");
        let destination = receiver.path().join("receiver ' quoted.git");
        let git = |cwd: &Path, args: &[&str]| {
            let output = std::process::Command::new("git")
                .current_dir(cwd)
                .args(args)
                .output()
                .expect("fixture Git");
            assert!(output.status.success(), "{output:?}");
            output.stdout
        };
        git(
            sender.path(),
            &[
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "-c",
                "commit.gpgSign=false",
                "commit",
                "--allow-empty",
                "-m",
                "source",
            ],
        );
        let common = receiver.path().join("main");
        if layout == "linked" {
            git(
                receiver.path(),
                &[
                    "clone",
                    "--no-hardlinks",
                    sender.path().to_str().expect("sender"),
                    common.to_str().expect("main"),
                ],
            );
            git(
                &common,
                &[
                    "worktree",
                    "add",
                    "-b",
                    "linked",
                    destination.to_str().expect("destination"),
                ],
            );
        } else {
            let mut args = vec!["init"];
            if layout != "normal" {
                args.push("--bare");
            }
            args.push(destination.to_str().expect("destination"));
            git(receiver.path(), &args);
        }
        let second = receiver.path().join("second.git");
        let mut second_original = None;
        let remote = if matches!(layout, "multiple" | "partial") {
            git(
                receiver.path(),
                &["init", "--bare", second.to_str().expect("second")],
            );
            if layout == "partial" {
                git(
                    &second,
                    &["fetch", sender.path().to_str().expect("sender"), "HEAD"],
                );
                let tree = String::from_utf8(git(&second, &["rev-parse", "FETCH_HEAD^{tree}"]))
                    .expect("tree");
                let commit = String::from_utf8(git(
                    &second,
                    &[
                        "-c",
                        "user.name=Fixture",
                        "-c",
                        "user.email=fixture@example.invalid",
                        "-c",
                        "commit.gpgSign=false",
                        "commit-tree",
                        tree.trim(),
                        "-m",
                        "divergent receiver",
                    ],
                ))
                .expect("commit");
                git(
                    &second,
                    &["update-ref", "refs/heads/approved", commit.trim()],
                );
                second_original = Some(git(&second, &["rev-parse", "refs/heads/approved"]));
            }
            git(
                sender.path(),
                &[
                    "config",
                    "remote.multiple.url",
                    destination.to_str().expect("destination"),
                ],
            );
            for target in [&destination, &second] {
                git(
                    sender.path(),
                    &[
                        "config",
                        "--add",
                        "remote.multiple.pushurl",
                        target.to_str().expect("target"),
                    ],
                );
            }
            "multiple".to_owned()
        } else if layout == "file" {
            format!("file://{}", destination.display())
        } else {
            destination.to_str().expect("remote").to_owned()
        };
        let metadata = if layout == "normal" {
            destination.join(".git")
        } else if layout == "linked" {
            common.join(".git")
        } else {
            destination.clone()
        };
        let receiver_lock = metadata.join("refs/heads/approved.lock");
        let packed_lock = metadata.join("packed-refs.lock");
        if layout == "maintenance-cancel" {
            git(
                &destination,
                &["fetch", sender.path().to_str().expect("sender"), "HEAD"],
            );
            let oid = String::from_utf8(git(sender.path(), &["rev-parse", "HEAD"])).expect("oid");
            for index in 0..17 {
                git(
                    &destination,
                    &[
                        "update-ref",
                        &format!("refs/heads/fixture-{index}"),
                        oid.trim(),
                    ],
                );
            }
            for (key, value) in [
                ("maintenance.auto", "true"),
                ("maintenance.gc.enabled", "false"),
                ("maintenance.pack-refs.enabled", "true"),
                ("core.packedRefsTimeout", "30000"),
            ] {
                git(&destination, &["config", key, value]);
            }
            std::fs::write(&packed_lock, b"fixture-owned packed lock\n").expect("packed lock");
        }
        if layout == "maintenance" {
            for (key, value) in [
                ("maintenance.auto", "true"),
                ("maintenance.gc.enabled", "false"),
                ("maintenance.loose-objects.enabled", "true"),
                ("maintenance.loose-objects.auto", "1"),
            ] {
                git(&destination, &["config", key, value]);
            }
        }
        if layout == "cancel" {
            git(
                &destination,
                &["config", "core.filesRefLockTimeout", "30000"],
            );
            std::fs::write(&receiver_lock, b"fixture-owned lock\n").expect("receiver lock");
        }
        let input = serde_json::json!({"remote": remote, "branch":"HEAD:refs/heads/approved"});
        let mut plan: Box<dyn PreparedNormalToolInvocation> = Box::new(
            AddPlan::prepare_push(sender.path(), &input, ToolProvenance::Native).expect("plan"),
        );
        let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(
            sender.path().into(),
        ));
        let context = PermissionContext::new(
            PermissionMode::Interactive,
            InteractionCapability::Available,
        );
        let mut stages = 0;
        loop {
            stages += 1;
            assert!(stages <= 8);
            let request = PermissionRequest::native("git_push", plan.permission_profile(), &input);
            let proposal = state
                .propose(&request, &context, GrantScope::Once)
                .expect("proposal");
            let pending = state
                .approve_once(proposal, &request, &context)
                .expect("approve stage");
            let admission = state
                .try_admit_prepared_invocation(
                    pending,
                    &request,
                    &context,
                    plan.plan_identity().expect("identity"),
                )
                .expect("admit stage");
            if layout == "maintenance-cancel" && stages == 8 {
                let receipt = plan.cleanup_receipt().expect("maintenance receipt");
                let task =
                    tokio::spawn(async move { plan.execute_stage(Box::new(admission)).await });
                let pack_pid = tokio::time::timeout(std::time::Duration::from_secs(10), async {
                    loop {
                        assert!(!task.is_finished(), "maintenance must wait on packed lock");
                        let output = std::process::Command::new("ps")
                            .args(["-axo", "pid=,ppid=,command="])
                            .output()
                            .expect("process observation");
                        assert!(output.status.success());
                        let rows: Vec<(u32, u32, String)> = String::from_utf8(output.stdout)
                            .expect("process list")
                            .lines()
                            .filter_map(|line| {
                                let mut fields = line.split_whitespace();
                                Some((
                                    fields.next()?.parse().ok()?,
                                    fields.next()?.parse().ok()?,
                                    fields.collect::<Vec<_>>().join(" "),
                                ))
                            })
                            .collect();
                        for (pid, parent, command) in &rows {
                            if !command.contains("pack-refs --all --prune --auto") {
                                continue;
                            }
                            let mut ancestor = *parent;
                            let mut maintenance = false;
                            for _ in 0..8 {
                                let Some((_, ppid, command)) =
                                    rows.iter().find(|(id, _, _)| *id == ancestor)
                                else {
                                    break;
                                };
                                maintenance |= command.contains("maintenance run");
                                if maintenance
                                    && command.contains("receive-pack")
                                    && command.contains(destination.to_str().expect("destination"))
                                {
                                    return *pid;
                                }
                                ancestor = *ppid;
                            }
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    }
                })
                .await
                .expect("actual receiver maintenance pack-refs descendant");
                assert!(
                    metadata.join("refs/heads/approved").exists(),
                    "ref update precedes maintenance"
                );
                task.abort();
                assert!(matches!(task.await, Err(error) if error.is_cancelled()));
                receipt
                    .wait(std::time::Duration::from_secs(3))
                    .await
                    .expect("maintenance cleanup confirmed");
                let observed = std::process::Command::new("ps")
                    .args(["-p", &pack_pid.to_string(), "-o", "stat="])
                    .output()
                    .expect("descendant status");
                assert!(
                    (observed.status.code() == Some(1)
                        && observed.stdout.is_empty()
                        && observed.stderr.is_empty())
                        || (observed.status.success()
                            && observed.stderr.is_empty()
                            && String::from_utf8_lossy(&observed.stdout)
                                .trim()
                                .starts_with('Z')),
                    "maintenance descendant remains live or observation failed: {observed:?}"
                );
                assert_eq!(
                    std::fs::read(&packed_lock).expect("original packed lock"),
                    b"fixture-owned packed lock\n"
                );
                std::fs::remove_file(&packed_lock).expect("release packed lock");
                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                assert!(
                    !metadata.join("packed-refs").exists(),
                    "no late maintenance write"
                );
                break;
            }
            if layout == "cancel" && stages == 8 {
                let receipt = plan.cleanup_receipt().expect("local push receipt");
                let task =
                    tokio::spawn(async move { plan.execute_stage(Box::new(admission)).await });
                let oid =
                    String::from_utf8(git(sender.path(), &["rev-parse", "HEAD"])).expect("oid");
                let oid = oid.trim();
                let received_object = metadata.join("objects").join(&oid[..2]).join(&oid[2..]);
                tokio::time::timeout(std::time::Duration::from_secs(10), async {
                    while !received_object.exists() {
                        assert!(!task.is_finished(), "push ended before receiver lock wait");
                        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                    }
                })
                .await
                .expect("receiver accepted objects");
                assert!(
                    !task.is_finished(),
                    "reference lock must hold execution open"
                );
                task.abort();
                assert!(matches!(task.await, Err(error) if error.is_cancelled()));
                receipt
                    .wait(std::time::Duration::from_secs(3))
                    .await
                    .expect("local receiver cleanup confirmed");
                assert!(!metadata.join("refs/heads/approved").exists());
                assert_eq!(
                    std::fs::read(&receiver_lock).expect("original lock"),
                    b"fixture-owned lock\n"
                );
                break;
            }
            match plan.execute_stage(Box::new(admission)).await {
                PreparedNormalStage::Continue(next) => {
                    assert!(
                        !metadata.join("refs/heads/approved").exists(),
                        "no early mutation"
                    );
                    if let Some(original) = &second_original {
                        assert_eq!(
                            &git(&second, &["rev-parse", "refs/heads/approved"]),
                            original
                        );
                    } else {
                        assert!(!second.join("refs/heads/approved").exists());
                    }
                    plan = next;
                }
                PreparedNormalStage::Complete(output) => {
                    assert_eq!(
                        output.result.is_error,
                        layout == "partial",
                        "{}",
                        output.result.content
                    );
                    if layout == "partial" {
                        assert!(
                            output
                                .result
                                .content
                                .contains("partially updated remote references"),
                            "{}",
                            output.result.content
                        );
                    }
                    break;
                }
            }
        }
        assert_eq!(stages, 8);
        if layout == "cancel" {
            // Releasing the competing lock after confirmed cleanup must not
            // let a surviving receive-pack finish the reference update.
            std::fs::remove_file(&receiver_lock).expect("release fixture lock");
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            assert!(!metadata.join("refs/heads/approved").exists());
            continue;
        }
        assert_eq!(
            git(sender.path(), &["rev-parse", "HEAD"]),
            git(&destination, &["rev-parse", "refs/heads/approved"])
        );
        if layout == "multiple" {
            assert_eq!(
                git(sender.path(), &["rev-parse", "HEAD"]),
                git(&second, &["rev-parse", "refs/heads/approved"])
            );
        }
        if let Some(original) = second_original {
            assert_eq!(
                git(&second, &["rev-parse", "refs/heads/approved"]),
                original
            );
        }
        if layout == "maintenance" {
            assert!(
                std::fs::read_dir(metadata.join("objects/pack"))
                    .expect("receiver packs")
                    .any(|entry| entry
                        .expect("pack entry")
                        .path()
                        .extension()
                        .is_some_and(|extension| extension == "pack")),
                "receiver automatic maintenance must finish before push completes"
            );
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[test]
fn receiver_system_shell_noninteractive_command_does_not_source_home_startup() {
    let home = tempfile::tempdir().expect("fixture home");
    for file in [
        ".profile",
        ".bash_profile",
        ".bash_login",
        ".bashrc",
        ".zshenv",
        ".zshrc",
    ] {
        std::fs::write(home.path().join(file), "printf unexpected > startup-ran\n")
            .expect("startup sentinel");
    }
    let canonical = Path::new("/bin/sh").canonicalize().expect("system shell");
    validate_receiver_shell_path(Path::new("/bin/sh"), &canonical).expect("system baseline");
    for shell in ["/bin/zsh", "/usr/local/bin/sh", "sh"] {
        assert!(validate_receiver_shell_path(Path::new(shell), &canonical).is_err());
    }
    assert!(validate_receiver_shell_path(Path::new("/bin/sh"), Path::new("/bin/ksh")).is_err());
    let output = std::process::Command::new("/bin/sh")
        .args(["-c", "printf '%s' \"$1\"", "fixed-command", "approved"])
        .current_dir(home.path())
        .env_clear()
        .env("HOME", home.path())
        .env("PATH", "/usr/bin:/bin")
        .output()
        .expect("system shell");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"approved");
    assert!(!home.path().join("startup-ran").exists());
}

#[test]
fn receiver_shell_startup_environment_refuses_without_mutating_parent_snapshot() {
    for name in [
        "BASH_ENV",
        "ENV",
        "SHELLOPTS",
        "BASHOPTS",
        "ZDOTDIR",
        "PS4",
        "SSH_CLIENT",
        "SSH2_CLIENT",
        "BASH_FUNC_git%%",
    ] {
        let environment = std::collections::BTreeMap::from([(
            std::ffi::OsString::from(name),
            std::ffi::OsString::from("unbound"),
        )]);
        let before = environment.clone();
        assert!(
            validate_receiver_shell_environment(&environment).is_err(),
            "{name}"
        );
        assert_eq!(before, environment);
    }
    validate_receiver_shell_environment(&std::collections::BTreeMap::from([(
        std::ffi::OsString::from("PATH"),
        std::ffi::OsString::from("/usr/bin:/bin"),
    )]))
    .expect("ordinary environment");
    #[cfg(unix)]
    validate_receiver_shell_environment(&std::collections::BTreeMap::from([(
        std::ffi::OsString::from("bash_env"),
        std::ffi::OsString::from("case-sensitive environment"),
    )]))
    .expect("Unix variable names are case sensitive");
}

#[cfg(unix)]
#[tokio::test]
async fn fixed_receiver_command_performs_real_local_push() {
    let sender = fixture();
    let target = tempfile::tempdir().expect("receiver");
    let receiver = target.path().join("receiver ' with spaces.git");
    let program = resolve_git(&execution_environment()).expect("Git");
    let commit = std::process::Command::new(&program)
        .current_dir(sender.path())
        .args([
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgSign=false",
            "commit",
            "--allow-empty",
            "-m",
            "fixture",
        ])
        .output()
        .expect("initial commit");
    assert!(commit.status.success());
    let init = std::process::Command::new(&program)
        .args(["init", "--bare"])
        .arg(&receiver)
        .output()
        .expect("bare init");
    assert!(init.status.success());
    let mut command = tokio::process::Command::new(&program);
    command
        .current_dir(sender.path())
        .arg("push")
        .arg(format!(
            "--receive-pack={}",
            receiver_command(&program).expect("receiver command")
        ))
        .arg("--")
        .arg(&receiver)
        .arg("HEAD:refs/heads/approved");
    super::super::run_host_command(command, std::time::Duration::from_secs(30), 16 * 1024)
        .await
        .expect("real local push");
    let head = std::process::Command::new(&program)
        .current_dir(sender.path())
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("sender head");
    let received = std::process::Command::new(&program)
        .arg("--git-dir")
        .arg(&receiver)
        .args(["rev-parse", "refs/heads/approved"])
        .output()
        .expect("receiver ref");
    assert!(head.status.success() && received.status.success());
    assert_eq!(head.stdout, received.stdout);
}

#[cfg(unix)]
#[tokio::test]
async fn receiver_command_quotes_executable_and_carries_foreground_settings() {
    use std::os::unix::fs::PermissionsExt;
    let directory = tempfile::tempdir().expect("fixture");
    let root = directory.path().canonicalize().expect("canonical");
    let program = root.join("git ' $(touch injected) ; executable");
    std::fs::write(&program, "#!/bin/sh\nprintf '%s\\n' \"$@\"\n").expect("fixture program");
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).expect("mode");
    let mut command = tokio::process::Command::new("/bin/sh");
    command
        .current_dir(&root)
        .arg("-c")
        .arg(receiver_command(&program).expect("fixed command"));
    let output = super::super::run_host_command(command, std::time::Duration::from_secs(10), 4096)
        .await
        .expect("execute");
    assert_eq!(
        output,
        b"-c\nmaintenance.autoDetach=false\n-c\ngc.autoDetach=false\nreceive-pack\n"
    );
    assert!(!root.join("injected").exists());
    assert!(receiver_command(Path::new("relative-git")).is_err());
}

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

#[cfg(windows)]
#[tokio::test]
async fn windows_program_guard_allows_real_git_startup() {
    let program = resolve_git(&execution_environment()).expect("Git on test PATH");
    let evidence = capture_program(&program).expect("retain Git guard");
    let mut command = tokio::process::Command::new(&program);
    command.arg("--version");
    let output =
        super::super::run_host_command(command, std::time::Duration::from_secs(10), 16 * 1024)
            .await
            .expect("Git must start while its evidence handle survives");
    assert!(output.starts_with(b"git version "));
    assert!(evidence == capture_program(&program).expect("revalidate Git"));
}

#[cfg(windows)]
#[test]
fn program_evidence_retains_windows_replacement_guard() {
    let root = tempfile::tempdir().expect("fixture");
    let path = root.path().join("program");
    std::fs::write(&path, "executable evidence").expect("program");
    let path = path.canonicalize().expect("canonical");
    let evidence = capture_program(&path).expect("capture");
    let second = capture_program(&path).expect("shared read capture");
    assert!(evidence == second);
    assert!(std::fs::OpenOptions::new().write(true).open(&path).is_err());
    assert!(std::fs::rename(&path, path.with_extension("old")).is_err());
    assert!(std::fs::remove_file(&path).is_err());
    drop(second);
    assert!(std::fs::remove_file(&path).is_err());
    drop(evidence);
    std::fs::rename(&path, path.with_extension("old")).expect("released guard");
}

#[cfg(unix)]
#[test]
fn receiver_child_executable_binds_link_target_and_identical_replacement() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let root = tempfile::tempdir().expect("fixture");
    let base = root.path().canonicalize().expect("canonical");
    let program = base.join("program");
    let alternate = base.join("alternate");
    for path in [&program, &alternate] {
        std::fs::write(path, "executable evidence").expect("program");
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).expect("mode");
    }
    let link = base.join("git-receive-pack");
    symlink(&program, &link).expect("link");
    let evidence = ReceiverChildExecutable::capture(&link, &program).expect("capture");
    evidence.revalidate(&program).expect("unchanged");
    std::fs::remove_file(&link).expect("unlink");
    symlink(&alternate, &link).expect("retarget");
    assert!(evidence.revalidate(&program).is_err());
    std::fs::remove_file(&link).expect("unlink");
    symlink(&program, &link).expect("restore");
    std::fs::rename(&program, base.join("old-inode")).expect("retain inode");
    std::fs::write(&program, "executable evidence").expect("replacement");
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).expect("mode");
    assert!(evidence.revalidate(&program).is_err());
    assert!(ReceiverChildExecutable::capture(&link, &alternate).is_err());
}

#[tokio::test]
async fn receiver_child_requires_read_admission_before_capture() {
    let sender = fixture();
    let receiver = fixture();
    let plan = AddPlan::prepare_push(
        sender.path(),
        &serde_json::json!({"remote": receiver.path()}),
        ToolProvenance::Native,
    )
    .expect("plan");
    let mut plan = Box::new(plan).discover().await.expect("discovery");
    // This absent child has no Read facet. Admission must fail before capture
    // could report that its executable is unavailable.
    plan.receiver_child_paths = vec![sender.path().join("unadmitted-child")];
    let admission = proof(&plan);
    let PreparedNormalStage::Complete(output) = plan.execute_stage(Box::new(admission)).await
    else {
        panic!("unadmitted child cannot advance discovery");
    };
    assert!(output.result.is_error);
    assert!(
        output
            .result
            .content
            .contains("Receiver Read authorization missing")
    );
}

#[cfg(unix)]
#[tokio::test]
async fn receiver_child_replacement_refuses_next_discovery_stage() {
    use std::os::unix::fs::PermissionsExt;
    let sender = fixture();
    let receiver = fixture();
    let child = sender
        .path()
        .canonicalize()
        .expect("canonical")
        .join("child");
    std::fs::write(&child, "fixture executable").expect("child");
    std::fs::set_permissions(&child, std::fs::Permissions::from_mode(0o755)).expect("mode");
    let plan = AddPlan::prepare_push(
        sender.path(),
        &serde_json::json!({"remote": receiver.path()}),
        ToolProvenance::Native,
    )
    .expect("plan");
    let mut plan = Box::new(plan).discover().await.expect("discovery");
    // Substitute only the shell fixture. Git children must retain their
    // identity relationship with the configuration-discovery executable.
    plan.receiver_child_paths[1] = child.clone();
    plan.facets
        .push(path_facet(ToolNature::Read, &child, "fixture child").expect("facet"));
    let admission = proof(&plan);
    let PreparedNormalStage::Continue(next) = plan.execute_stage(Box::new(admission)).await else {
        panic!("admitted child must allow root discovery");
    };
    std::fs::rename(&child, child.with_extension("old")).expect("retain old inode");
    std::fs::write(&child, "fixture executable").expect("replacement");
    std::fs::set_permissions(&child, std::fs::Permissions::from_mode(0o755)).expect("mode");
    // Re-admit the continuation through the production permission API.
    let state =
        PermissionSessionState::new(PermissionEngine::with_workspace_root(sender.path().into()));
    let context = PermissionContext::new(
        PermissionMode::Interactive,
        InteractionCapability::Available,
    );
    let input = serde_json::json!({"remote": receiver.path()});
    let request = PermissionRequest::native("git_push", next.permission_profile(), &input);
    let proposal = state
        .propose(&request, &context, GrantScope::Once)
        .expect("proposal");
    let pending = state
        .approve_once(proposal, &request, &context)
        .expect("approval");
    let admission = state
        .try_admit_prepared_invocation(
            pending,
            &request,
            &context,
            next.plan_identity().expect("identity"),
        )
        .expect("admission");
    let PreparedNormalStage::Complete(output) = next.execute_stage(Box::new(admission)).await
    else {
        panic!("replaced child cannot advance metadata discovery");
    };
    assert!(output.result.is_error);
    assert!(
        output
            .result
            .content
            .contains("Receiver child executable changed after discovery")
    );
}

#[tokio::test]
async fn discovery_proof_cannot_enter_mutation_dispatch() {
    let root = fixture();
    let plan = prepare(root.path());
    let admission = proof(&plan);
    let result = Box::new(plan).execute(Box::new(admission)).await;
    assert!(result.result.is_error);
    assert!(result.result.content.contains("completed discovery phase"));
    assert!(!root.path().join(".git/index").exists());
}

#[test]
fn command_budget_reserves_both_pipes_and_never_partially_charges() {
    let mut budget = CommandBudget {
        commands: 2,
        output_bytes: 8,
    };
    budget.reserve(2).expect("first child");
    assert_eq!((budget.commands, budget.output_bytes), (1, 4));
    assert!(budget.reserve(u64::MAX).is_err());
    assert_eq!((budget.commands, budget.output_bytes), (1, 4));
    assert!(budget.reserve(3).is_err());
    assert_eq!((budget.commands, budget.output_bytes), (1, 4));
    budget.reserve(2).expect("second child");
    assert!(budget.reserve(0).is_err());
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

#[cfg(unix)]
#[tokio::test]
async fn commit_target_probe_only_treats_quiet_missing_reference_as_absent() {
    let root = fixture();
    let plan = prepare(root.path());
    assert!(
        plan.inspect_commit_target("refs/heads/missing")
            .await
            .expect("missing reference")
            .is_none()
    );
    plan.command_budget.lock().expect("budget").commands = 0;
    let error = plan
        .inspect_commit_target("refs/heads/missing")
        .await
        .expect_err("exhaustion is not missing");
    assert!(
        error
            .to_string()
            .contains("cumulative command/output budget")
    );
}

#[cfg(unix)]
#[tokio::test]
async fn probe_exit_status_is_not_a_process_failure() {
    let root = fixture();
    let plan = prepare(root.path());
    let mut command = tokio::process::Command::new("git");
    command
        .current_dir(root.path())
        .args(["config", "--local", "--get", "fixture.absent"]);
    let output = plan
        .run_command_output(command, std::time::Duration::from_secs(5), 1024)
        .await
        .expect("completed missing-value probe");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
    let missing = tokio::process::Command::new(root.path().join("nonexistent-git"));
    assert!(
        plan.run_command_output(missing, std::time::Duration::from_secs(1), 1024)
            .await
            .is_err()
    );
    let mut slow = tokio::process::Command::new("/bin/sh");
    slow.args(["-c", "sleep 10"]);
    assert!(
        plan.run_command_output(slow, std::time::Duration::from_millis(100), 1024)
            .await
            .is_err()
    );
}

#[cfg(unix)]
#[tokio::test]
async fn exhausted_command_budget_refuses_before_spawn() {
    let root = fixture();
    let plan = prepare(root.path());
    plan.command_budget.lock().expect("budget").commands = 0;
    let marker = root.path().join("unexpected-child");
    let mut command = tokio::process::Command::new("/bin/sh");
    command
        .args(["-c", "printf spawned > \"$1\"", "fixture"])
        .arg(&marker);
    let error = plan
        .run_command(command, std::time::Duration::from_secs(1), 128)
        .await
        .expect_err("must refuse before spawn");
    assert!(
        error
            .to_string()
            .contains("cumulative command/output budget")
    );
    assert!(!marker.exists());
}

#[tokio::test]
async fn discovery_reports_budget_exhaustion_without_misdiagnosing_installation() {
    let root = fixture();
    let plan = prepare(root.path());
    plan.command_budget.lock().expect("budget").commands = 0;
    for error in [
        plan.inspect_config().await.expect_err("config inspection"),
        plan.inspect_config_paths("GIT_CONFIG_GLOBAL")
            .await
            .expect_err("config path inspection"),
        plan.inspect_attribute_path_at(root.path(), "GIT_ATTR_GLOBAL")
            .await
            .expect_err("attribute path inspection"),
    ] {
        assert!(
            error
                .to_string()
                .contains("cumulative command/output budget"),
            "{error}"
        );
    }
}

#[tokio::test]
async fn discovery_and_mutation_share_command_budget() {
    let root = fixture();
    let plan = prepare(root.path());
    let before = plan.command_budget.lock().expect("budget").commands;
    let next = Box::new(plan).discover().await.expect("discovery");
    assert!(next.command_budget.lock().expect("budget").commands < before);
    // A continuation keeps the same allowance; no mutation child may
    // start once prior discovery has exhausted it.
    next.command_budget.lock().expect("budget").commands = 0;
    let marker = root.path().join("unexpected-continuation-child");
    let mut command = tokio::process::Command::new("/bin/sh");
    command
        .args(["-c", "printf spawned > \"$1\"", "fixture"])
        .arg(&marker);
    let error = next
        .run_command(command, std::time::Duration::from_secs(1), 128)
        .await
        .expect_err("must refuse before spawn");
    assert!(
        error
            .to_string()
            .contains("cumulative command/output budget"),
        "{}",
        error
    );
    assert!(!marker.exists());
    assert!(!root.path().join(".git/index").exists());
}

#[cfg(unix)]
#[tokio::test]
async fn discovery_runner_does_not_connect_to_trace_socket() {
    let root = fixture();
    let home = tempfile::tempdir().expect("isolated home");
    let socket = home.path().join("trace.sock");
    let listener = std::os::unix::net::UnixListener::bind(&socket).expect("socket");
    listener.set_nonblocking(true).expect("nonblocking");
    std::fs::write(
        home.path().join(".gitconfig"),
        format!(
            "[trace2]\n eventTarget = af_unix:stream:{}\n",
            socket.display()
        ),
    )
    .expect("config");
    let plan = prepare(root.path());
    let mut environment = plan.environment.clone();
    environment.insert("HOME".into(), home.path().as_os_str().into());
    environment.insert("XDG_CONFIG_HOME".into(), home.path().as_os_str().into());
    let mut command = tokio::process::Command::new(&plan.program);
    command
        .current_dir(root.path())
        .args(["config", "--null", "--show-origin", "--list"])
        .env_clear()
        .envs(environment);
    plan.run_command(command, std::time::Duration::from_secs(10), 256 * 1024)
        .await
        .expect("discovery");
    assert_eq!(
        listener.accept().expect_err("no trace connection").kind(),
        std::io::ErrorKind::WouldBlock
    );
}

#[tokio::test]
async fn discovery_runner_suppresses_configured_trace_files() {
    let root = fixture();
    let home = tempfile::tempdir().expect("isolated home");
    let mut config = String::from("[trace2]\n");
    for (key, name) in [
        ("normalTarget", "normal"),
        ("eventTarget", "event"),
        ("perfTarget", "perf"),
    ] {
        config.push_str(&format!(" {key} = {}\n", home.path().join(name).display()));
    }
    std::fs::write(home.path().join(".gitconfig"), config).expect("config");
    let plan = prepare(root.path());
    let mut environment = plan.environment.clone();
    environment.insert("HOME".into(), home.path().as_os_str().into());
    environment.insert("XDG_CONFIG_HOME".into(), home.path().as_os_str().into());
    let mut command = tokio::process::Command::new(&plan.program);
    command
        .current_dir(root.path())
        .args(["config", "--null", "--show-origin", "--list"])
        .env_clear()
        .envs(environment);
    let output = plan
        .run_command(command, std::time::Duration::from_secs(10), 256 * 1024)
        .await
        .expect("discovery");
    assert!(
        output
            .windows(b"trace2.eventtarget".len())
            .any(|w| w == b"trace2.eventtarget")
    );
    for name in ["normal", "event", "perf"] {
        assert!(!home.path().join(name).exists());
    }
}

#[cfg(unix)]
#[tokio::test]
async fn cancelled_push_closes_transport_and_confirms_cleanup() {
    use std::io::{Read, Write};
    for during_pack in [false, true] {
        let root = fixture();
        if during_pack {
            let mut state = 0x12345678u32;
            let data: Vec<u8> = (0..8 * 1024 * 1024)
                .map(|_| {
                    state ^= state << 13;
                    state ^= state >> 17;
                    state ^= state << 5;
                    state as u8
                })
                .collect();
            std::fs::write(root.path().join("payload"), data).expect("payload");
            assert!(
                std::process::Command::new("git")
                    .current_dir(root.path())
                    .args(["add", "payload"])
                    .output()
                    .expect("add")
                    .status
                    .success()
            );
        }
        assert!(
            std::process::Command::new("git")
                .current_dir(root.path())
                .args([
                    "-c",
                    "user.name=Fixture",
                    "-c",
                    "user.email=fixture@example.invalid",
                    "-c",
                    "commit.gpgSign=false",
                    "commit",
                    "--allow-empty",
                    "-m",
                    "fixture"
                ])
                .output()
                .expect("commit")
                .status
                .success()
        );
        let index_before = std::fs::read(root.path().join(".git/index")).ok();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("listener");
        listener.set_nonblocking(true).expect("nonblocking");
        let destination = format!("git://{}/repo", listener.local_addr().expect("address"));
        let plan = AddPlan::prepare_push(
            root.path(),
            &serde_json::json!({"remote":destination,"branch":"HEAD:refs/heads/test"}),
            ToolProvenance::Native,
        )
        .expect("plan");
        let next = Box::new(plan).discover().await.expect("discovery");
        let receipt = next.cleanup_receipt().expect("receipt");
        let admission = proof(&next);
        let task = tokio::spawn(async move { next.execute(Box::new(admission)).await });
        let mut stream = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        tokio::time::sleep(std::time::Duration::from_millis(5)).await
                    }
                    Err(error) => panic!("accept: {error}"),
                }
            }
        })
        .await
        .expect("push connected");
        stream.set_nonblocking(true).expect("nonblocking");
        if during_pack {
            let advertisement = "0000000000000000000000000000000000000000 capabilities^{}\0report-status delete-refs ofs-delta\n";
            let packet = format!("{:04x}{advertisement}0000", advertisement.len() + 4);
            stream.write_all(packet.as_bytes()).expect("advertisement");
            tokio::time::timeout(std::time::Duration::from_secs(10), async {
                let mut observed = Vec::new();
                let mut bytes = [0; 256];
                loop {
                    match stream.read(&mut bytes) {
                        Ok(0) => panic!("closed before pack"),
                        Ok(count) => {
                            observed.extend_from_slice(&bytes[..count]);
                            if observed.windows(4).any(|window| window == b"PACK") {
                                break;
                            }
                            assert!(observed.len() < 64 * 1024, "pack header missing");
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            tokio::time::sleep(std::time::Duration::from_millis(5)).await
                        }
                        Err(error) => panic!("pack read: {error}"),
                    }
                }
            })
            .await
            .expect("pack started");
            assert!(!task.is_finished(), "push must still be active");
        }
        task.abort();
        assert!(task.await.expect_err("cancelled").is_cancelled());
        receipt
            .wait(std::time::Duration::from_secs(3))
            .await
            .expect("cleanup confirmed");
        tokio::time::timeout(std::time::Duration::from_secs(3), async {
            let mut bytes = [0; 4096];
            loop {
                match stream.read(&mut bytes) {
                    Ok(0) => break,
                    Ok(_) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        tokio::time::sleep(std::time::Duration::from_millis(5)).await
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::ConnectionReset => break,
                    Err(error) => panic!("read: {error}"),
                }
            }
        })
        .await
        .expect("transport closed");
        assert_eq!(
            std::fs::read(root.path().join(".git/index")).ok(),
            index_before
        );
    }
}

#[cfg(unix)]
#[tokio::test]
async fn cancelled_plan_receipt_prevents_descendant_delayed_write() {
    let root = fixture();
    let plan = prepare(root.path());
    let receipt = plan.cleanup_receipt().expect("local receipt");
    let mut command = tokio::process::Command::new("sh");
    command.current_dir(root.path()).args([
        "-c",
        "(sleep 1; printf forbidden > delayed) & printf ready > ready; wait",
    ]);
    let task = tokio::spawn(async move {
        plan.run_command(command, std::time::Duration::from_secs(10), 1024)
            .await
    });
    tokio::time::timeout(std::time::Duration::from_secs(3), async {
        while !root.path().join("ready").exists() {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("started");
    task.abort();
    assert!(task.await.expect_err("cancelled task").is_cancelled());
    receipt
        .wait(std::time::Duration::from_secs(3))
        .await
        .expect("confirmed cleanup");
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    assert!(!root.path().join("delayed").exists());
}

#[cfg(unix)]
#[tokio::test]
async fn sealed_owner_refuses_discovery_and_prepared_continuation() {
    use talos_core::tool::AgentTool;
    let root = fixture();
    let tool = super::super::GitAddTool::new(root.path().to_owned());
    let owner = tool.cleanup_receipt().expect("owner");
    let plan = prepare(root.path()).bind_operation(tool.executor.operation());
    let receipt = plan.cleanup_receipt().expect("operation");
    let continuation = Box::new(plan).discover().await.expect("discovery");
    receipt
        .wait(std::time::Duration::from_secs(1))
        .await
        .expect("discovery cleanup");
    owner.seal();
    let output = execute_approved(*continuation).await;
    assert!(output.result.is_error);
    assert!(
        output
            .result
            .content
            .contains("configuration inspection failed"),
        "{}",
        output.result.content
    );
    let fresh = prepare(root.path()).bind_operation(tool.executor.operation());
    let error = match Box::new(fresh).discover().await {
        Ok(_) => panic!("sealed discovery accepted"),
        Err(error) => error,
    };
    assert!(
        error
            .to_string()
            .contains("configuration inspection failed")
    );
    assert!(!root.path().join(".git/index").exists());
    receipt
        .wait(std::time::Duration::from_secs(1))
        .await
        .expect("local cleanup");
    owner
        .wait(std::time::Duration::from_secs(1))
        .await
        .expect("owner cleanup");
}

#[cfg(unix)]
#[tokio::test]
async fn parent_configuration_identity_changes_invalidate_discovered_plan() {
    for retarget in [false, true] {
        let root = fixture();
        for directory in ["first", "second"] {
            std::fs::create_dir(root.path().join(directory)).expect("directory");
            std::fs::write(
                root.path().join(directory).join("config"),
                "[user]\n name = Fixture\n",
            )
            .expect("config");
        }
        let link = root.path().join("current");
        std::os::unix::fs::symlink(root.path().join("first"), &link).expect("link");
        assert!(
            std::process::Command::new("git")
                .args(["config", "include.path"])
                .arg(link.join("config"))
                .current_dir(root.path())
                .status()
                .expect("include")
                .success()
        );
        let plan = Box::new(prepare(root.path()))
            .discover()
            .await
            .expect("discovery");
        plan.revalidate().expect("unchanged");
        if retarget {
            std::fs::remove_file(&link).expect("unlink");
            std::os::unix::fs::symlink(root.path().join("second"), &link).expect("retarget");
        } else {
            std::fs::rename(
                root.path().join("second/config"),
                root.path().join("first/config"),
            )
            .expect("replace with identical bytes");
        }
        assert!(
            plan.revalidate()
                .expect_err("changed identity")
                .to_string()
                .contains("path identity")
        );
    }
}

#[tokio::test]
async fn promisor_remote_maintenance_refuses_before_mutation() {
    for (key, value) in [
        ("remote.origin.promisor", "true"),
        ("remote.origin.partialCloneFilter", "blob:none"),
        ("extensions.partialClone", "origin"),
    ] {
        let root = fixture();
        assert!(
            std::process::Command::new("git")
                .args(["config", "maintenance.auto", "false"])
                .current_dir(root.path())
                .status()
                .expect("maintenance")
                .success()
        );
        let status = std::process::Command::new("git")
            .args(["config", key, value])
            .current_dir(root.path())
            .status()
            .expect("config");
        assert!(status.success());
        let plan = AddPlan::prepare_commit(
            root.path(),
            &serde_json::json!({"message":"promisor"}),
            ToolProvenance::Native,
        )
        .expect("plan");
        let output = execute_approved(plan).await;
        assert!(output.result.is_error && output.result.content.contains("promisor remote"));
        assert!(!root.path().join(".git/COMMIT_EDITMSG").exists());
    }
}

#[cfg(unix)]
#[test]
fn maintenance_entry_retarget_is_refused_after_initial_validation() {
    let root = fixture();
    let mut plan = prepare(root.path());
    let entry = root.path().join("maintenance-git");
    std::os::unix::fs::symlink(&plan.program, &entry).expect("entry");
    plan.maintenance_program = Some(entry.clone());
    plan.revalidate_maintenance_program()
        .expect("initial binding");
    let other = root.path().join("other-program");
    std::fs::write(&other, "not Git").expect("other program");
    std::fs::remove_file(&entry).expect("remove fixture link");
    std::os::unix::fs::symlink(other, &entry).expect("retarget");
    assert!(
        plan.revalidate_maintenance_program()
            .expect_err("late replacement")
            .to_string()
            .contains("maintenance executable differs")
    );
}

#[tokio::test]
async fn commit_maintenance_recent_objects_hook_requires_execution_authorization() {
    for maintenance in [None, Some("true"), Some("false")] {
        let root = fixture();
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .args(args)
                .current_dir(root.path())
                .output()
                .expect("fixture Git");
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            output.stdout
        };
        git(&["add", "file"]);
        git(&["commit", "-m", "initial"]);
        std::fs::write(root.path().join("file"), "changed").expect("change");
        git(&["add", "file"]);
        git(&[
            "config",
            "gc.recentObjectsHook",
            "touch recent-objects-hook-ran",
        ]);
        if let Some(value) = maintenance {
            git(&["config", "maintenance.auto", value]);
        }
        let head = git(&["rev-parse", "HEAD"]);
        let index = std::fs::read(root.path().join(".git/index")).expect("index");
        let plan = AddPlan::prepare_commit(
            root.path(),
            &serde_json::json!({"message":"prepared"}),
            ToolProvenance::Native,
        )
        .expect("plan");
        let output = execute_approved(plan).await;
        if maintenance == Some("false") {
            assert!(!output.result.is_error, "{}", output.result.content);
            assert_ne!(git(&["rev-parse", "HEAD"]), head);
        } else {
            assert!(
                output.result.is_error && output.result.content.contains("gc.recentObjectsHook")
            );
            assert_eq!(git(&["rev-parse", "HEAD"]), head);
            assert_eq!(
                std::fs::read(root.path().join(".git/index")).expect("index"),
                index
            );
        }
        assert!(!root.path().join("recent-objects-hook-ran").exists());
    }
}

#[tokio::test]
async fn maintenance_pack_redirection_requires_separate_effect_authorization() {
    let root = fixture();
    let outside = tempfile::tempdir().expect("outside object destination");
    let git = |args: &[&str]| {
        assert!(
            std::process::Command::new("git")
                .args(args)
                .current_dir(root.path())
                .status()
                .expect("config")
                .success()
        );
    };
    git(&[
        "config",
        "gc.repackFilterTo",
        outside.path().to_str().expect("path"),
    ]);
    let input = serde_json::json!({"message":"parent"});
    let plan = AddPlan::prepare_commit(root.path(), &input, ToolProvenance::Native).expect("plan");
    let admission = proof(&plan);
    let PreparedNormalStage::Complete(output) =
        Box::new(plan).execute_stage(Box::new(admission)).await
    else {
        panic!("redirect was allowed")
    };
    assert!(output.result.is_error && output.result.content.contains("gc.repackFilterTo"));
    assert!(
        std::fs::read_dir(outside.path())
            .expect("outside unchanged")
            .next()
            .is_none()
    );
    git(&["config", "maintenance.auto", "false"]);
    let plan = AddPlan::prepare_commit(root.path(), &input, ToolProvenance::Native)
        .expect("disabled maintenance");
    let admission = proof(&plan);
    assert!(matches!(
        Box::new(plan).execute_stage(Box::new(admission)).await,
        PreparedNormalStage::Continue(_)
    ));
    assert!(!root.path().join(".git/COMMIT_EDITMSG").exists());
}

#[test]
fn retained_child_evidence_rejects_changes_after_inspection_returns() {
    let root = fixture();
    let output = std::process::Command::new("git")
        .args(["add", "file"])
        .current_dir(root.path())
        .output()
        .expect("stage");
    assert!(output.status.success());
    let paths = vec!["file".to_owned()];
    let evidence = ChildInspectionEvidence {
        attributes: vec![(root.path().join(".gitattributes"), snapshot::Entry::Missing)],
        status: Vec::new(),
        sources: vec![(
            root.path().to_owned(),
            root.path().to_owned(),
            paths.clone(),
            capture_tracked_sources(root.path(), &paths).expect("sources"),
        )],
    };
    evidence.revalidate().expect("unchanged");
    std::fs::write(root.path().join("file"), "after!").expect("late edit");
    assert!(
        evidence
            .revalidate()
            .expect_err("late source change")
            .to_string()
            .contains("tracked sources")
    );
    std::fs::write(root.path().join("file"), "before").expect("restore");
    std::fs::write(root.path().join(".gitattributes"), "file filter=late\n")
        .expect("late attribute");
    assert!(
        evidence
            .revalidate()
            .expect_err("late attributes")
            .to_string()
            .contains("attributes changed")
    );
}

#[test]
fn tracked_child_sources_detect_edits_without_reading_untracked_replacement_tree() {
    let root = tempfile::tempdir().expect("fixture");
    let paths = vec!["tracked".to_owned()];
    std::fs::write(root.path().join("tracked"), "first").expect("tracked");
    let before = capture_tracked_sources(root.path(), &paths).expect("before");
    std::fs::write(root.path().join("tracked"), "other").expect("same length edit");
    assert_ne!(
        before,
        capture_tracked_sources(root.path(), &paths).expect("after")
    );
    std::fs::remove_file(root.path().join("tracked")).expect("remove file");
    std::fs::create_dir_all(root.path().join("tracked/untracked/.git"))
        .expect("untracked nested repository");
    assert_eq!(
        capture_tracked_sources(root.path(), &paths).expect("directory kind only"),
        vec![snapshot::Entry::Directory(Vec::new())]
    );
}

#[tokio::test]
async fn child_status_filter_is_refused_without_executing_helper() {
    let root = fixture();
    let child = root.path().join("module");
    std::fs::create_dir(&child).expect("child");
    let git = |cwd: &Path, args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("fixture Git");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(&child, &["init"]);
    std::fs::write(child.join("tracked"), "first\n").expect("tracked");
    git(&child, &["add", "tracked"]);
    git(
        root.path(),
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            "160000",
            "1111111111111111111111111111111111111111",
            "module",
        ],
    );
    let parent_attributes = root.path().join("parent-attributes");
    std::fs::write(&parent_attributes, "").expect("parent attributes");
    git(
        root.path(),
        &[
            "config",
            "core.attributesFile",
            parent_attributes.to_str().expect("path"),
        ],
    );
    let plan = Box::new(
        AddPlan::prepare_commit(
            root.path(),
            &serde_json::json!({"message":"parent"}),
            ToolProvenance::Native,
        )
        .expect("plan"),
    )
    .discover()
    .await
    .expect("child attribute discovery");
    let child_global = plan
        .inspect_attribute_path_at(&child, "GIT_ATTR_GLOBAL")
        .await
        .expect("child global");
    assert_ne!(child_global, parent_attributes);
    assert!(plan.child_attributes.contains(&child_global));
    assert!(
        plan.facets
            .iter()
            .any(|facet| facet.nature == ToolNature::Read
                && facet.resource.as_deref() == child_global.to_str())
    );
    std::fs::write(child.join(".gitattributes"), "tracked filter=probe\n").expect("attributes");
    git(
        &child,
        &["config", "filter.probe.clean", "touch helper-ran; cat"],
    );
    std::fs::write(child.join("tracked"), "other\n").expect("same length edit");
    for all in [false, true] {
        let input = serde_json::json!({"message":"parent", "all":all});
        let result = execute_approved(
            AddPlan::prepare_commit(root.path(), &input, ToolProvenance::Native).expect("plan"),
        )
        .await;
        assert!(
            result.result.is_error
                && result
                    .result
                    .content
                    .contains("Submodule status uses a configured filter"),
            "{}",
            result.result.content
        );
        assert!(!child.join("helper-ran").exists());
        assert!(!root.path().join(".git/COMMIT_EDITMSG").exists());
    }
}

#[tokio::test]
async fn child_invalid_index_is_read_only_after_continuation_admission() {
    let root = fixture();
    let child = root.path().join("module");
    let external = tempfile::tempdir().expect("external metadata");
    std::fs::create_dir(&child).expect("child");
    let git = |cwd: &Path, args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("Git fixture");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(
        &child,
        &[
            "init",
            "--separate-git-dir",
            external
                .path()
                .join("metadata")
                .to_str()
                .expect("metadata path"),
        ],
    );
    git(
        root.path(),
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            "160000",
            "1111111111111111111111111111111111111111",
            "module",
        ],
    );
    std::fs::write(
        external.path().join("metadata/index"),
        "invalid child index",
    )
    .expect("invalid index");
    let input = serde_json::json!({"message":"parent", "all":true});
    let plan = AddPlan::prepare_commit(root.path(), &input, ToolProvenance::Native)
        .expect("prepare without reading child index");
    let admission = proof(&plan);
    let PreparedNormalStage::Continue(next) =
        Box::new(plan).execute_stage(Box::new(admission)).await
    else {
        panic!("discovery must not read child index")
    };
    assert!(next.permission_profile().iter().any(|facet| {
        facet.nature == ToolNature::Read
            && facet.resource.as_deref()
                == external
                    .path()
                    .join("metadata")
                    .canonicalize()
                    .expect("canonical metadata")
                    .to_str()
    }));
    drop(next);
    assert!(!root.path().join(".git/COMMIT_EDITMSG").exists());
    let result = execute_approved(
        AddPlan::prepare_commit(root.path(), &input, ToolProvenance::Native).expect("second plan"),
    )
    .await;
    assert!(result.result.is_error);
    assert!(!root.path().join(".git/COMMIT_EDITMSG").exists());
    assert_eq!(
        std::fs::read(external.path().join("metadata/index")).expect("preserved index"),
        b"invalid child index"
    );
}

#[tokio::test]
async fn child_config_discovery_detects_included_fsmonitor_before_status() {
    let root = fixture();
    let child = root.path().join("module");
    std::fs::create_dir(&child).expect("child");
    let git = |cwd: &Path, args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("fixture Git");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(&child, &["init"]);
    git(
        root.path(),
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            "160000",
            "1111111111111111111111111111111111111111",
            "module",
        ],
    );
    let outside = tempfile::tempdir().expect("external config");
    let included = outside.path().join("config");
    std::fs::write(&included, "[core]\n fsmonitor = touch helper-ran\n").expect("helper config");
    git(
        &child,
        &["config", "include.path", included.to_str().expect("path")],
    );
    let input = serde_json::json!({"message":"parent", "all":true});
    let plan = AddPlan::prepare_commit(root.path(), &input, ToolProvenance::Native)
        .expect("no child config before admission");
    let admission = proof(&plan);
    let PreparedNormalStage::Complete(output) =
        Box::new(plan).execute_stage(Box::new(admission)).await
    else {
        panic!("child fsmonitor allowed")
    };
    assert!(output.result.is_error && output.result.content.contains("submodule fsmonitor"));
    assert!(!child.join("helper-ran").exists());
    assert!(!root.path().join(".git/COMMIT_EDITMSG").exists());
    std::fs::write(&included, "[core]\n fsmonitor = false\n").expect("disabled helper");
    let plan = AddPlan::prepare_commit(root.path(), &input, ToolProvenance::Native).expect("plan");
    let admission = proof(&plan);
    let PreparedNormalStage::Continue(next) =
        Box::new(plan).execute_stage(Box::new(admission)).await
    else {
        panic!("disabled fsmonitor should discover")
    };
    assert!(
        next.permission_profile()
            .iter()
            .any(|facet| facet.nature == ToolNature::Read
                && facet.resource.as_deref() == included.to_str())
    );
    drop(next); // Declining continuation has no mutation effects.
    assert!(!child.join("helper-ran").exists());
}

#[test]
fn external_gitlink_marker_is_planned_without_reading_destination() {
    let root = fixture();
    let outside = tempfile::tempdir().expect("external metadata");
    let child = root.path().join("module");
    std::fs::create_dir(&child).expect("checkout");
    let marker = child.join(".git");
    std::fs::write(&marker, format!("gitdir: {}\n", outside.path().display())).expect("marker");
    // Not a valid repository: preparation must still succeed, because the
    // destination's contents have no authorization yet.
    std::fs::write(outside.path().join("config"), "invalid config").expect("external config");
    let output = std::process::Command::new("git")
        .args([
            "update-index",
            "--add",
            "--cacheinfo",
            "160000",
            "1111111111111111111111111111111111111111",
            "module",
        ])
        .current_dir(root.path())
        .output()
        .expect("index");
    assert!(output.status.success());
    let plan = AddPlan::prepare_commit(
        root.path(),
        &serde_json::json!({"message":"parent"}),
        ToolProvenance::Native,
    )
    .expect("deferred external metadata");
    assert!(
        plan.facets
            .iter()
            .any(|facet| facet.nature == ToolNature::Read
                && facet.resource.as_deref() == outside.path().to_str())
    );
    std::fs::write(outside.path().join("config"), "different invalid config")
        .expect("external change");
    plan.revalidate()
        .expect("external content not read before admission");
    std::fs::write(&marker, "gitdir: other\n").expect("retarget marker");
    assert!(
        plan.revalidate().is_err(),
        "marker retarget must invalidate preparation"
    );
}

#[test]
fn commit_gitlink_preparation_defers_child_head_and_worktree_reads() {
    let root = fixture();
    let child = root.path().join("module");
    std::fs::create_dir(&child).expect("module");
    let git = |cwd: &Path, args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("fixture Git");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(&child, &["init"]);
    git(
        &child,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "-c",
            "commit.gpgSign=false",
            "commit",
            "--allow-empty",
            "-m",
            "first",
        ],
    );
    git(root.path(), &["add", "module", "file"]);
    let (paths, before) = tracked_commit_sources(root.path()).expect("sources");
    assert_eq!(paths, ["file"]);
    assert_eq!(before.len(), 1);
    assert!(before[0].metadata_path.is_some());
    std::fs::create_dir(child.join("untracked")).expect("untracked");
    std::fs::create_dir(child.join("untracked/.git")).expect("nested untracked marker");
    assert_eq!(
        before,
        tracked_commit_sources(root.path())
            .expect("unchanged metadata")
            .1
    );
    git(
        &child,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "-c",
            "commit.gpgSign=false",
            "commit",
            "--allow-empty",
            "-m",
            "second",
        ],
    );
    let after = tracked_commit_sources(root.path()).expect("new HEAD").1;
    assert_eq!(before[0].index_oid, after[0].index_oid);
    assert_eq!(
        before, after,
        "preparation must defer child HEAD reads until admission"
    );
    let plan = AddPlan::prepare_commit(
        root.path(),
        &serde_json::json!({"message":"parent", "all":true}),
        ToolProvenance::Native,
    )
    .expect("plan with gitlink");
    assert_eq!(plan.gitlinks.len(), 1);
    assert!(plan.facets.iter().any(|facet| {
        facet.nature == ToolNature::Read
            && facet.resource.as_deref()
                == child
                    .join(".git")
                    .canonicalize()
                    .expect("canonical child metadata")
                    .to_str()
    }));
}

#[test]
fn commit_gitlink_preserves_missing_and_uninitialized_checkout_states() {
    let root = fixture();
    let output = std::process::Command::new("git")
        .args([
            "update-index",
            "--add",
            "--cacheinfo",
            "160000",
            "1111111111111111111111111111111111111111",
            "module",
        ])
        .current_dir(root.path())
        .output()
        .expect("gitlink");
    assert!(output.status.success());
    let missing = tracked_commit_sources(root.path()).expect("missing").1;
    assert_eq!(missing[0].entry, snapshot::Entry::Missing);
    std::fs::create_dir(root.path().join("module")).expect("uninitialized");
    let empty = tracked_commit_sources(root.path()).expect("empty").1;
    assert!(matches!(empty[0].entry, snapshot::Entry::Directory(_)));
    assert!(empty[0].metadata_path.is_none());
    assert_ne!(missing, empty);
}

#[tokio::test]
async fn commit_all_deletes_replaced_tracked_file_and_refuses_nested_repository() {
    for nested in [false, true] {
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
            String::from_utf8(output.stdout).expect("utf8")
        };
        git(&["config", "user.name", "Fixture"]);
        git(&["config", "user.email", "fixture@example.invalid"]);
        git(&["config", "commit.gpgSign", "false"]);
        git(&["add", "file"]);
        git(&["commit", "-m", "initial"]);
        std::fs::remove_file(root.path().join("file")).expect("remove tracked file");
        std::fs::create_dir(root.path().join("file")).expect("replacement directory");
        if nested {
            git(&["init", "file"]);
        }
        std::fs::write(root.path().join("file/untracked"), "not a commit source")
            .expect("nested untracked");
        let plan = AddPlan::prepare_commit(
            root.path(),
            &serde_json::json!({"message":"delete replaced file", "all":true}),
            ToolProvenance::Native,
        )
        .expect("tracked-only plan");
        let head = git(&["rev-parse", "HEAD"]);
        let output = execute_approved(plan).await;
        if nested {
            assert!(
                output.result.is_error && output.result.content.contains("nested repository"),
                "{}",
                output.result.content
            );
            assert_eq!(git(&["rev-parse", "HEAD"]), head);
        } else {
            assert!(!output.result.is_error, "{}", output.result.content);
            assert!(git(&["ls-tree", "--name-only", "HEAD"]).is_empty());
        }
        assert_eq!(
            std::fs::read_to_string(root.path().join("file/untracked")).expect("preserved"),
            "not a commit source"
        );
    }
}

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
    let admission = proof(&plan);
    let PreparedNormalStage::Continue(next) =
        Box::new(plan).execute_stage(Box::new(admission)).await
    else {
        panic!("discovery");
    };
    let engine =
        PermissionSessionState::new(PermissionEngine::with_workspace_root(root.path().into()));
    let context = PermissionContext::new(
        PermissionMode::Interactive,
        InteractionCapability::Available,
    );
    let input = serde_json::json!({"branch":"target"});
    let request = PermissionRequest::native("git_checkout", next.permission_profile(), &input);
    let proposal = engine
        .propose(&request, &context, GrantScope::Once)
        .expect("proposal");
    let pending = engine
        .approve_once(proposal, &request, &context)
        .expect("approve");
    let proof = engine
        .try_admit_prepared_invocation(
            pending,
            &request,
            &context,
            next.plan_identity().expect("identity"),
        )
        .expect("admit");
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
async fn git_configuration_path_discovery_includes_absent_global_candidates() {
    let root = fixture();
    let mut plan = prepare(root.path());
    let home = root.path().join("home");
    let xdg = root.path().join("xdg");
    // Override only the child command environment, never the test process.
    plan.environment
        .insert("HOME".into(), home.clone().into_os_string());
    plan.environment
        .insert("XDG_CONFIG_HOME".into(), xdg.clone().into_os_string());
    let paths = plan
        .inspect_config_paths("GIT_CONFIG_GLOBAL")
        .await
        .expect("path query");
    assert!(paths.contains(&home.join(".gitconfig")));
    assert!(paths.contains(&xdg.join("git/config")));
    assert!(paths.iter().all(|path| !path.exists()));
    assert!(!home.exists());
    assert!(!xdg.exists());
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
    if cfg!(windows) {
        assert!(capture_new_gitlink_directory(&child, false).is_err());
        return;
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
}

#[tokio::test]
async fn child_filter_command_uses_effective_root_without_discovery_prefix() {
    let root = fixture();
    let discovery = root.path().join("nested");
    std::fs::create_dir(&discovery).expect("discovery");
    assert!(
        std::process::Command::new("git")
            .current_dir(&discovery)
            .arg("init")
            .output()
            .expect("child init")
            .status
            .success()
    );
    std::fs::write(
        root.path().join(".gitattributes"),
        "tracked filter=root-filter\n",
    )
    .expect("root attributes");
    std::fs::write(discovery.join(".gitattributes"), "tracked -filter\n")
        .expect("discovery attributes");
    let plan = prepare(root.path());
    let command =
        plan.child_filter_command(&discovery.join(".git"), root.path(), &["tracked".into()]);
    let output = plan
        .run_command(command, std::time::Duration::from_secs(10), 4096)
        .await
        .expect("real child filter probe");
    assert_eq!(output, b"tracked\0filter\0root-filter\0");
}

#[tokio::test]
async fn child_bound_config_discovery_does_not_resolve_configured_worktree() {
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
    let missing = root.path().join("unapproved-missing-worktree");
    assert!(
        std::process::Command::new("git")
            .current_dir(&child)
            .args(["config", "core.worktree"])
            .arg(&missing)
            .output()
            .expect("config")
            .status
            .success()
    );
    let plan = AddPlan::prepare_checkout(
        root.path(),
        &serde_json::json!({"branch":"HEAD"}),
        ToolProvenance::Native,
    )
    .expect("plan");
    let output = plan
        .inspect_child_bootstrap_config(&child.join(".git"), &child)
        .await
        .expect("bound probe");
    assert!(
        output
            .windows(missing.as_os_str().len())
            .any(|value| { value == missing.to_string_lossy().as_bytes() })
    );
    assert!(!missing.exists());
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
                assert_eq!(execute_approved(plan).await.result.is_error, cfg!(windows));
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
        if matches!(change, "native" | "native-nonempty" | "none")
            || (change == "nonempty" && !cfg!(windows))
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

#[tokio::test]
async fn real_commit_runs_configured_automatic_maintenance() {
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
    };
    git(&["config", "user.name", "Fixture"]);
    git(&["config", "user.email", "fixture@example.invalid"]);
    git(&["config", "commit.gpgSign", "false"]);
    git(&["config", "maintenance.auto", "true"]);
    git(&["config", "maintenance.gc.enabled", "false"]);
    git(&["config", "maintenance.loose-objects.enabled", "true"]);
    git(&["config", "maintenance.loose-objects.auto", "1"]);
    git(&["add", "file"]);
    let pack_dir = root.path().join(".git/objects/pack");
    assert_eq!(
        std::fs::read_dir(&pack_dir)
            .expect("pack directory")
            .count(),
        0
    );
    let plan = AddPlan::prepare_commit(
        root.path(),
        &serde_json::json!({"message":"maintenance"}),
        ToolProvenance::Native,
    )
    .expect("plan");
    let output = execute_approved(plan).await;
    assert!(!output.result.is_error, "{}", output.result.content);
    assert!(
        std::fs::read_dir(&pack_dir)
            .expect("packs")
            .any(|entry| entry
                .expect("entry")
                .path()
                .extension()
                .is_some_and(|extension| extension == "pack")),
        "automatic loose-object maintenance must produce a pack before completion"
    );
}

#[tokio::test]
async fn real_commit_preserves_index_semantics_and_all_excludes_untracked() {
    let root = fixture();
    let git = |args: &[&str]| {
        let result = std::process::Command::new("git")
            .args(args)
            .current_dir(root.path())
            .output()
            .expect("git");
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        String::from_utf8(result.stdout).expect("utf8")
    };
    git(&["config", "user.name", "Fixture"]);
    git(&["config", "user.email", "fixture@example.invalid"]);
    git(&["config", "commit.gpgSign", "false"]);
    git(&["add", "file"]);
    std::fs::write(root.path().join("file"), "unstaged").expect("edit");
    std::fs::write(root.path().join("untracked"), "private").expect("untracked");
    let plan = AddPlan::prepare_commit(
        root.path(),
        &serde_json::json!({"message":"index only"}),
        ToolProvenance::Native,
    )
    .expect("plan");
    let output = execute_approved(plan).await;
    assert!(!output.result.is_error, "{}", output.result.content);
    assert_eq!(git(&["show", "HEAD:file"]), "before");
    assert_eq!(
        std::fs::read_to_string(root.path().join("file")).expect("worktree"),
        "unstaged"
    );
    let plan = AddPlan::prepare_commit(
        root.path(),
        &serde_json::json!({"message":"tracked", "all":true}),
        ToolProvenance::Native,
    )
    .expect("all plan");
    let output = execute_approved(plan).await;
    assert!(!output.result.is_error, "{}", output.result.content);
    assert_eq!(git(&["show", "HEAD:file"]), "unstaged");
    assert_eq!(git(&["ls-tree", "--name-only", "HEAD"]), "file\n");
    let head = git(&["rev-parse", "HEAD"]);
    let plan = AddPlan::prepare_commit(
        root.path(),
        &serde_json::json!({"message":"no changes"}),
        ToolProvenance::Native,
    )
    .expect("no-op plan");
    assert!(execute_approved(plan).await.result.is_error);
    assert_eq!(git(&["rev-parse", "HEAD"]), head);
}

#[tokio::test]
async fn commit_plan_binds_tracked_sources_and_rejects_add_proof() {
    let root = fixture();
    let output = std::process::Command::new("git")
        .args(["add", "file"])
        .current_dir(root.path())
        .output()
        .expect("stage");
    assert!(output.status.success());
    std::fs::write(root.path().join("untracked"), "not selected").expect("untracked");
    let commit = AddPlan::prepare_commit(
        root.path(),
        &serde_json::json!({"message":"first"}),
        ToolProvenance::Native,
    )
    .expect("commit plan");
    assert_eq!(commit.paths, ["file"]);
    assert_eq!(commit.request().tool_name(), "git_commit");
    assert!(commit.facets.iter().any(|facet| {
        facet
            .description
            .as_deref()
            .is_some_and(|description| description.contains("maintenance.autoDetach=false"))
    }));
    let add = prepare(root.path());
    let result = Box::new(commit).execute_stage(Box::new(proof(&add))).await;
    let PreparedNormalStage::Complete(result) = result else {
        panic!("exchanged proof accepted")
    };
    assert!(result.result.is_error);
    let tool = super::super::GitCommitTool::new(root.path().to_owned());
    assert!(
        tool.execute(serde_json::json!({"message":"first"}))
            .await
            .is_error
    );
}

#[tokio::test]
async fn commit_inspects_parent_filters_before_mutation_in_both_modes() {
    for all in [false, true] {
        let root = fixture();
        assert!(
            std::process::Command::new("git")
                .args(["add", "file"])
                .current_dir(root.path())
                .status()
                .expect("stage")
                .success()
        );
        std::fs::write(root.path().join(".gitattributes"), "file filter=probe\n")
            .expect("attributes");
        assert!(
            std::process::Command::new("git")
                .args(["config", "filter.probe.clean", "touch helper-ran; cat"])
                .current_dir(root.path())
                .status()
                .expect("filter")
                .success()
        );
        let index = std::fs::read(root.path().join(".git/index")).expect("index");
        let output = execute_approved(
            AddPlan::prepare_commit(
                root.path(),
                &serde_json::json!({"message":"filtered", "all":all}),
                ToolProvenance::Native,
            )
            .expect("plan"),
        )
        .await;
        assert!(
            output.result.is_error && output.result.content.contains("configured filter"),
            "{}",
            output.result.content
        );
        assert!(!root.path().join("helper-ran").exists());
        assert!(!root.path().join(".git/COMMIT_EDITMSG").exists());
        assert_eq!(
            index,
            std::fs::read(root.path().join(".git/index")).expect("preserved index")
        );
    }
}

#[test]
fn directory_rules_include_empty_and_symlink_only_subdirectories() {
    let entry = super::snapshot::Entry::Directory(vec![
        ("empty".into(), super::snapshot::Entry::Directory(vec![])),
        (
            "links".into(),
            super::snapshot::Entry::Directory(vec![(
                "link".into(),
                super::snapshot::Entry::Link("missing".into()),
            )]),
        ),
    ]);
    let root = std::path::Path::new("selected");
    let mut paths = std::collections::BTreeSet::new();
    super::selected_directory_rules(root, &entry, &mut paths);
    for directory in [root.to_path_buf(), root.join("empty"), root.join("links")] {
        assert!(paths.contains(&directory.join(".gitattributes")));
        assert!(paths.contains(&directory.join(".gitignore")));
    }
    assert_eq!(paths.len(), 6);
}
use super::*;
use talos_core::tool::AgentTool;
use talos_permission::{
    GrantScope, InteractionCapability, PermissionContext, PermissionEngine, PermissionMode,
    PermissionSessionState,
};

fn proof(plan: &AddPlan) -> AdmittedInvocation {
    let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(
        plan.workspace.clone(),
    ));
    let context = PermissionContext::new(
        PermissionMode::Interactive,
        InteractionCapability::Available,
    );
    let request = plan.request();
    let proposal = state
        .propose(&request, &context, GrantScope::Once)
        .expect("proposal");
    let pending = state
        .approve_once(proposal, &request, &context)
        .expect("approval");
    state
        .try_admit_prepared_invocation(pending, &request, &context, &plan.identity)
        .expect("admission")
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

// Match native recursive fetch's cwd/GIT_DIR setup and the prepared command's
// explicit worktree override. Neither changes relative fetch resource lookup.
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
        .arg(format!("--base-path={}", remote.path().display()))
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
        let fetch_command = plan.pull_child_fetch_command(&child, &child.join(".git"));
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
                .expect("identity");
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
                    error.to_string().contains("Executable Git fetch hooks"),
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
                    .contains("Executable Git fetch hooks")
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
    assert!(output.result.content.contains("Executable Git fetch hooks"));
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
    for path in [&child, &child.join(".git"), &fallback] {
        assert!(plan.facets.iter().any(|facet| {
            facet.nature == ToolNature::Read && facet.resource.as_deref() == path.to_str()
        }));
        assert!(plan.child_read(path, &stale).is_err());
    }
    let tokens = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("fresh child tokens");
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
            let deep = external.path().join("refs/heads/a/b/c/d/e/f/g/h/i/j/topic");
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
                    .expect("frontier");
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
        assert!(matches!(
            next.pull_integration,
            Some(super::super::pull_state::Integration::Command(_))
        ));
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
    assert!(inventory.exact_paths(&common, &common).contains(&external));
    assert!(!inventory.paths().contains(&reference));
    assert!(
        !plan
            .advance_pull_parent_inventory(&mut inventory, &old)
            .expect("old grants")
    );
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

#[tokio::test]
async fn receiver_reference_tree_refuses_exact_paths_without_closed_capability() {
    let sender = fixture();
    let receiver = fixture();
    let plan = AddPlan::prepare_push(
        sender.path(),
        &serde_json::json!({"remote": receiver.path()}),
        ToolProvenance::Native,
    )
    .expect("plan");
    let mut plan = Box::new(plan).discover().await.expect("sender discovery");
    let authorization = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("roots admission");
    plan = plan.discover_receiver_roots(&authorization).expect("roots");
    let authorization = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("metadata admission");
    plan = plan
        .discover_receiver_metadata(&authorization)
        .expect("metadata");
    assert!(plan.require_receiver_reference_capability().is_ok());
    let stale = proof(&plan);
    plan.facets
        .retain(|facet| facet.resource.as_deref() != Some("git:receiver-reference-tree"));
    assert!(
        stale
            .consume_for_prepared(&plan.request(), &plan.identity)
            .is_err()
    );
    let admission = proof(&plan);
    let PreparedNormalStage::Complete(output) = plan.execute_stage(Box::new(admission)).await
    else {
        panic!("exact directory tokens cannot authorize descendant reads");
    };
    assert!(output.result.is_error);
    assert!(
        output
            .result
            .content
            .contains("reference tree discovery capability missing")
    );
}

#[tokio::test]
async fn prepared_push_transfers_objects_to_admitted_git_daemon() {
    let root = fixture();
    let receiver = tempfile::tempdir().expect("receiver");
    let git = |cwd: &Path, args: &[&str]| {
        let output = std::process::Command::new("git")
            .current_dir(cwd)
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
    git(receiver.path(), &["init", "--bare"]);
    std::fs::write(root.path().join("payload"), "approved object transfer\n").expect("payload");
    git(root.path(), &["add", "payload"]);
    git(
        root.path(),
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "commit",
            "-m",
            "fixture",
        ],
    );
    let expected = git(root.path(), &["rev-parse", "HEAD"]);
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("port");
    let port = listener.local_addr().expect("address").port();
    drop(listener);
    let mut daemon = tokio::process::Command::new("git")
        .args([
            "daemon",
            "--reuseaddr",
            "--listen=127.0.0.1",
            "--export-all",
            "--enable=receive-pack",
        ])
        .arg(format!("--port={port}"))
        .arg(format!(
            "--base-path={}",
            receiver.path().parent().expect("parent").display()
        ))
        .arg(receiver.path())
        .kill_on_drop(true)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("daemon");
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            if std::net::TcpStream::connect_timeout(
                &std::net::SocketAddr::from(([127, 0, 0, 1], port)),
                std::time::Duration::from_millis(20),
            )
            .is_ok()
            {
                break;
            }
            assert!(daemon.try_wait().expect("daemon state").is_none());
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("daemon ready");
    let destination = format!(
        "git://127.0.0.1:{port}/{}",
        receiver.path().file_name().expect("name").to_string_lossy()
    );
    let input = serde_json::json!({"remote":destination,"branch":"HEAD:refs/heads/approved"});
    // Both stages reject absent proof before contacting the receiver.
    let plan = AddPlan::prepare_push(root.path(), &input, ToolProvenance::Native).expect("plan");
    let PreparedNormalStage::Complete(denied) = Box::new(plan).execute_stage(Box::new(())).await
    else {
        panic!("unapproved discovery continued");
    };
    assert!(denied.result.is_error);
    let plan = AddPlan::prepare_push(root.path(), &input, ToolProvenance::Native).expect("plan");
    let next = Box::new(plan).discover().await.expect("discovery");
    assert!(next.execute(Box::new(())).await.result.is_error);
    assert!(git(receiver.path(), &["for-each-ref"]).is_empty());

    // Discovery approval cannot authorize the generated mutation plan.
    let plan = AddPlan::prepare_push(root.path(), &input, ToolProvenance::Native).expect("plan");
    let stale = proof(&plan);
    let admission = proof(&plan);
    let PreparedNormalStage::Continue(next) =
        Box::new(plan).execute_stage(Box::new(admission)).await
    else {
        panic!("discovery failed");
    };
    let denied = next.execute(Box::new(stale)).await;
    assert!(denied.result.is_error && denied.result.content.contains("does not authorize"));
    assert!(git(receiver.path(), &["for-each-ref"]).is_empty());

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let hook = root.path().join(".git/hooks/pre-push");
        let plan =
            AddPlan::prepare_push(root.path(), &input, ToolProvenance::Native).expect("plan");
        let next = Box::new(plan).discover().await.expect("discovery");
        let admission = proof(&next);
        // A hook appearing after approval must never run under Git's
        // executable permission, even though its inspection was admitted.
        std::fs::write(&hook, "#!/bin/sh\nprintf forbidden > hook-ran\n").expect("hook");
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).expect("mode");
        let denied = next.execute(Box::new(admission)).await;
        assert!(denied.result.is_error, "{}", denied.result.content);
        assert!(!root.path().join("hook-ran").exists());
        assert!(git(receiver.path(), &["for-each-ref"]).is_empty());
        std::fs::remove_file(hook).expect("remove fixture hook");
    }
    let plan = AddPlan::prepare_push(root.path(), &input, ToolProvenance::Native).expect("plan");
    let next = Box::new(plan).discover().await.expect("discovery");
    let admission = proof(&next);
    let output = next.execute(Box::new(admission)).await;
    assert!(!output.result.is_error, "{}", output.result.content);
    assert_eq!(
        git(receiver.path(), &["rev-parse", "refs/heads/approved"]),
        expected
    );
    assert_eq!(
        git(receiver.path(), &["show", "approved:payload"]),
        "approved object transfer"
    );
    // The receive side owns its own policy. A denied deletion must report
    // failure without claiming success or removing the existing ref.
    git(receiver.path(), &["config", "receive.denyDeletes", "true"]);
    let deletion = serde_json::json!({"remote":destination,"branch":":refs/heads/approved"});
    let plan =
        AddPlan::prepare_push(root.path(), &deletion, ToolProvenance::Native).expect("delete plan");
    let next = Box::new(plan).discover().await.expect("discovery");
    let admission = proof(&next);
    let output = next.execute(Box::new(admission)).await;
    assert!(output.result.is_error, "{}", output.result.content);
    assert!(
        output.result.content.contains("denied") || output.result.content.contains("rejected"),
        "{}",
        output.result.content
    );
    assert!(
        output
            .result
            .content
            .contains("inspect every destination before retrying")
    );
    assert_eq!(
        git(receiver.path(), &["rev-parse", "refs/heads/approved"]),
        expected
    );
    git(
        root.path(),
        &["config", "remote.multiple.url", &destination],
    );
    git(
        root.path(),
        &["config", "--add", "remote.multiple.pushurl", &destination],
    );
    let missing = format!("git://127.0.0.1:{port}/missing-receiver.git");
    git(
        root.path(),
        &["config", "--add", "remote.multiple.pushurl", &missing],
    );
    let input = serde_json::json!({"remote":"multiple", "branch":"HEAD:refs/heads/partial"});
    let plan = AddPlan::prepare_push(root.path(), &input, ToolProvenance::Native)
        .expect("multi-target plan");
    let next = Box::new(plan).discover().await.expect("discovery");
    assert_eq!(next.push_destinations, [destination, missing]);
    let admission = proof(&next);
    let output = next.execute(Box::new(admission)).await;
    assert!(
        output.result.is_error
            && output
                .result
                .content
                .contains("partially updated remote references"),
        "{}",
        output.result.content
    );
    assert_eq!(
        git(receiver.path(), &["rev-parse", "refs/heads/partial"]),
        expected
    );
    // A global local receive-pack command is not a valid git:// service.
    // Refuse both URL orders before updating either receiver.
    let local = tempfile::tempdir().expect("local mixed receiver");
    git(local.path(), &["init", "--bare"]);
    for local_first in [false, true] {
        let remote = if local_first {
            "mixed-local-first"
        } else {
            "mixed-network-first"
        };
        let local_url = local.path().to_str().expect("local path");
        let network_url = format!(
            "git://127.0.0.1:{port}/{}",
            receiver.path().file_name().expect("name").to_string_lossy()
        );
        let urls = if local_first {
            [local_url, network_url.as_str()]
        } else {
            [network_url.as_str(), local_url]
        };
        for url in urls {
            git(
                root.path(),
                &["config", "--add", &format!("remote.{remote}.pushurl"), url],
            );
        }
        let input = serde_json::json!({"remote":remote,"branch":"HEAD:refs/heads/mixed"});
        let mut plan: Box<dyn PreparedNormalToolInvocation> = Box::new(
            AddPlan::prepare_push(root.path(), &input, ToolProvenance::Native).expect("mixed plan"),
        );
        let state =
            PermissionSessionState::new(PermissionEngine::with_workspace_root(root.path().into()));
        let context = PermissionContext::new(
            PermissionMode::Interactive,
            InteractionCapability::Available,
        );
        let mut stages = 0;
        loop {
            stages += 1;
            assert!(stages <= 8);
            let request = PermissionRequest::native("git_push", plan.permission_profile(), &input);
            let proposal = state
                .propose(&request, &context, GrantScope::Once)
                .expect("proposal");
            let pending = state
                .approve_once(proposal, &request, &context)
                .expect("approval");
            let admission = state
                .try_admit_prepared_invocation(
                    pending,
                    &request,
                    &context,
                    plan.plan_identity().expect("identity"),
                )
                .expect("admission");
            match plan.execute_stage(Box::new(admission)).await {
                PreparedNormalStage::Continue(next) => plan = next,
                PreparedNormalStage::Complete(output) => {
                    assert!(output.result.is_error);
                    assert!(
                        output.result.content.contains("Mixed local and network"),
                        "{}",
                        output.result.content
                    );
                    break;
                }
            }
        }
        for destination in [local.path(), receiver.path()] {
            assert!(
                !std::process::Command::new("git")
                    .current_dir(destination)
                    .args(["rev-parse", "--verify", "refs/heads/mixed"])
                    .output()
                    .expect("reference probe")
                    .status
                    .success()
            );
        }
    }
    daemon.kill().await.expect("stop daemon");
    daemon.wait().await.expect("join daemon");
}

#[tokio::test]
async fn push_rejects_legacy_destination_inserted_after_discovery() {
    for directory in ["remotes", "branches"] {
        let root = fixture();
        let plan = AddPlan::prepare_push(
            root.path(),
            &serde_json::json!({"remote":"git://example.invalid/repo"}),
            ToolProvenance::Native,
        )
        .expect("plan");
        let next = Box::new(plan).discover().await.expect("discovery");
        let admission = proof(&next);
        let directory = root.path().join(".git").join(directory);
        std::fs::create_dir_all(&directory).expect("legacy directory");
        std::fs::write(directory.join("origin"), "URL: ext::unapproved-command\n")
            .expect("legacy destination");
        let result = next.execute(Box::new(admission)).await;
        assert!(
            result.result.is_error && result.result.content.contains("Legacy Git remote"),
            "{}",
            result.result.content
        );
    }
}

#[tokio::test]
async fn push_rejects_symbolic_and_detached_head_changes_after_discovery() {
    for replacement in [
        "ref: refs/heads/different\n",
        "1111111111111111111111111111111111111111\n",
    ] {
        let root = fixture();
        let plan = AddPlan::prepare_push(
            root.path(),
            &serde_json::json!({"remote":"git://example.invalid/repository", "branch":"HEAD:refs/heads/main"}),
            ToolProvenance::Native,
        ).expect("plan");
        let next = Box::new(plan).discover().await.expect("discovery");
        let admission = proof(&next);
        std::fs::write(root.path().join(".git/HEAD"), replacement).expect("replace HEAD");
        let output = next.execute(Box::new(admission)).await;
        assert!(
            output.result.is_error && output.result.content.contains("changed after preparation"),
            "{}",
            output.result.content
        );
    }
}

#[tokio::test]
async fn push_rejects_sender_ref_changes_after_discovery() {
    for packed in [false, true] {
        let root = fixture();
        let plan =
            AddPlan::prepare_push(root.path(), &serde_json::json!({}), ToolProvenance::Native)
                .expect("plan");
        let next = Box::new(plan).discover().await.expect("discovery");
        assert!(next.revalidate().is_ok());
        let path = if packed {
            root.path().join(".git/packed-refs")
        } else {
            root.path().join(".git/refs/heads/other")
        };
        std::fs::write(
            path,
            if packed {
                "1111111111111111111111111111111111111111 refs/heads/other\n"
            } else {
                "1111111111111111111111111111111111111111\n"
            },
        )
        .expect("ref change");
        assert!(
            next.revalidate()
                .expect_err("stale refs")
                .to_string()
                .contains("changed after preparation")
        );
    }
}

#[tokio::test]
async fn local_push_receiver_requires_path_admission_without_receiver_reads() {
    let root = fixture();
    let receiver = root.path().join("not-created-receiver.git");
    let plan = AddPlan::prepare_push(
        root.path(),
        &serde_json::json!({"remote": receiver}),
        ToolProvenance::Native,
    )
    .expect("plan");
    let admission = proof(&plan);
    let PreparedNormalStage::Continue(next) =
        Box::new(plan).execute_stage(Box::new(admission)).await
    else {
        panic!("discovery failed");
    };
    assert!(next.permission_profile().iter().any(|facet| {
        facet.nature == ToolNature::Read && facet.resource.as_deref() == receiver.to_str()
    }));
    assert!(!next.permission_profile().iter().any(|facet| matches!(
        facet.nature,
        ToolNature::Network | ToolNature::Write | ToolNature::Execute
    )));
    assert!(!receiver.exists());
    assert!(!root.path().join(".git/index").exists());
}

#[tokio::test]
async fn receiver_root_discovery_defers_external_gitdir_and_requires_fresh_proof() {
    let root = fixture();
    let receiver = tempfile::tempdir().expect("receiver");
    // A nonexistent external target must only become a next-stage resource.
    let external = root.path().join("unopened-metadata");
    std::fs::write(
        receiver.path().join(".git"),
        format!("gitdir: {}\n", external.display()),
    )
    .expect("gitfile");
    let plan = AddPlan::prepare_push(
        root.path(),
        &serde_json::json!({"remote":receiver.path()}),
        ToolProvenance::Native,
    )
    .expect("sender plan");
    let stale = proof(&plan);
    let plan = Box::new(plan).discover().await.expect("sender discovery");
    assert!(plan.phase == Phase::ReceiverRoots);
    let wrong = Box::new(plan).execute_stage(Box::new(stale)).await;
    assert!(matches!(wrong, PreparedNormalStage::Complete(output) if output.result.is_error));

    let plan = AddPlan::prepare_push(
        root.path(),
        &serde_json::json!({"remote":receiver.path()}),
        ToolProvenance::Native,
    )
    .expect("sender plan");
    let plan = Box::new(plan).discover().await.expect("sender discovery");
    let admission = proof(&plan);
    let PreparedNormalStage::Continue(next) = plan.execute_stage(Box::new(admission)).await else {
        panic!("admitted root must produce metadata continuation");
    };
    assert!(
        next.permission_profile()
            .iter()
            .any(|facet| facet.nature == ToolNature::Read
                && facet.resource.as_deref() == external.to_str())
    );
    assert!(
        next.permission_profile()
            .iter()
            .all(|facet| facet.nature == ToolNature::Read)
    );
    assert!(!external.exists());
    assert!(!root.path().join(".git/index").exists());
}

#[tokio::test]
async fn receiver_home_includes_refuse_ambiguous_interpolation() {
    for (home, include) in [("/alias/../home", "~/config"), ("/home", "~//absolute")] {
        let sender = fixture();
        let receiver = fixture();
        std::fs::write(
            receiver.path().join(".git/config"),
            format!("[include]\npath={include}\n"),
        )
        .expect("receiver config");
        let plan = AddPlan::prepare_push(
            sender.path(),
            &serde_json::json!({"remote":receiver.path()}),
            ToolProvenance::Native,
        )
        .expect("plan");
        let mut plan = Box::new(plan).discover().await.expect("sender discovery");
        plan.environment.insert("HOME".into(), home.into());
        let authorization = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("roots admission");
        let plan = plan.discover_receiver_roots(&authorization).expect("roots");
        let authorization = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("metadata admission");
        let plan = plan
            .discover_receiver_metadata(&authorization)
            .expect("metadata");
        let authorization = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("common admission");
        assert!(plan.require_receiver_reference_capability().is_ok());
        assert!(plan.discover_receiver_common(&authorization).is_err());
    }
}

#[cfg(unix)]
#[tokio::test]
async fn receiver_external_hook_is_deferred_to_fresh_read_admission() {
    use std::os::unix::fs::PermissionsExt;
    let sender = fixture();
    let receiver = fixture();
    let external = tempfile::tempdir().expect("external hooks");
    let hook = external
        .path()
        .canonicalize()
        .expect("canonical")
        .join("pre-receive");
    std::fs::write(
        receiver.path().join(".git/config"),
        format!(
            "[core]\nrepositoryFormatVersion=0\nbare=false\nhooksPath={}\n",
            hook.parent().expect("parent").display()
        ),
    )
    .expect("config");
    std::fs::write(&hook, "#!/bin/sh\nexit 99\n").expect("hook");
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).expect("executable");
    let plan = AddPlan::prepare_push(
        sender.path(),
        &serde_json::json!({"remote":receiver.path()}),
        ToolProvenance::Native,
    )
    .expect("plan");
    let plan = Box::new(plan).discover().await.expect("sender");
    let authorization = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("root admission");
    let plan = plan.discover_receiver_roots(&authorization).expect("roots");
    let authorization = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("metadata admission");
    let plan = plan
        .discover_receiver_metadata(&authorization)
        .expect("metadata");
    let authorization = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("common admission");
    let plan = plan
        .discover_receiver_common(&authorization)
        .expect("common");
    let stale = proof(&plan);
    let authorization = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("configuration admission");
    let plan = plan
        .discover_receiver_configuration(&authorization)
        .await
        .expect("configuration does not inspect executable hook");
    assert!(plan.facets.iter().any(
        |facet| facet.nature == ToolNature::Read && facet.resource.as_deref() == hook.to_str()
    ));
    assert!(
        stale
            .consume_for_prepared(&plan.request(), &plan.identity)
            .is_err()
    );
    let admission = proof(&plan);
    let PreparedNormalStage::Complete(output) = plan.execute_stage(Box::new(admission)).await
    else {
        panic!("refuse executable hook")
    };
    assert!(output.result.is_error && output.result.content.contains("Executable receiver hook"));
    assert!(!receiver.path().join(".git/index").exists());
    assert!(!sender.path().join(".git/index").exists());
}

#[tokio::test]
async fn receiver_multiple_destinations_keep_sibling_bisect_evidence_separate() {
    let sender = fixture();
    let first = fixture();
    let second = fixture();
    let linked = tempfile::tempdir().expect("linked roots");
    for (index, receiver) in [first.path(), second.path()].into_iter().enumerate() {
        let output = std::process::Command::new("git")
            .current_dir(receiver)
            .args(["worktree", "add", "-b", "sibling"])
            .arg(linked.path().join(index.to_string()))
            .output()
            .expect("worktree");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let output = std::process::Command::new("git")
            .current_dir(sender.path())
            .args(["config", "--add", "remote.multiple.pushurl"])
            .arg(receiver)
            .output()
            .expect("push destination");
        assert!(output.status.success());
    }
    let plan = AddPlan::prepare_push(
        sender.path(),
        &serde_json::json!({"remote":"multiple"}),
        ToolProvenance::Native,
    )
    .expect("plan");
    let plan = Box::new(plan).discover().await.expect("sender discovery");
    let authorization = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("roots admission");
    let plan = plan.discover_receiver_roots(&authorization).expect("roots");
    let authorization = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("metadata admission");
    let plan = plan
        .discover_receiver_metadata(&authorization)
        .expect("metadata");
    let authorization = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("common admission");
    let plan = plan
        .discover_receiver_common(&authorization)
        .expect("common");
    let authorization = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("configuration admission");
    let plan = plan
        .discover_receiver_configuration(&authorization)
        .await
        .expect("configuration");
    assert_eq!(plan.receiver_effects.len(), 2);
    assert_eq!(plan.receiver_siblings.len(), 2);
    for (metadata, effects) in plan.receiver_metadata.iter().zip(&plan.receiver_effects) {
        for sibling in &plan.receiver_siblings {
            let belongs =
                sibling.spelling.parent() == Some(metadata.common_path.join("worktrees").as_path());
            for leaf in ["BISECT_LOG", "BISECT_START"] {
                assert_eq!(
                    effects.paths.contains(&sibling.canonical.join(leaf)),
                    belongs
                );
            }
        }
    }
}

#[tokio::test]
async fn receiver_layout_stages_accept_real_bare_normal_and_linked_repositories() {
    for kind in ["bare", "normal", "linked"] {
        let sender = fixture();
        let target = tempfile::tempdir().expect("receiver fixture");
        let repository = target.path().join("repository");
        let mut init = std::process::Command::new("git");
        init.arg("init");
        if kind == "bare" {
            init.arg("--bare");
        }
        assert!(
            init.arg(&repository)
                .output()
                .expect("init")
                .status
                .success()
        );
        let receiver = if kind == "linked" {
            let output = std::process::Command::new("git")
                .current_dir(&repository)
                .args([
                    "-c",
                    "user.name=Fixture",
                    "-c",
                    "user.email=fixture@example.invalid",
                    "-c",
                    "commit.gpgSign=false",
                    "commit",
                    "--allow-empty",
                    "-m",
                    "fixture",
                ])
                .output()
                .expect("commit");
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let linked = target.path().join("linked");
            let output = std::process::Command::new("git")
                .current_dir(&repository)
                .args(["worktree", "add", "-b", "linked"])
                .arg(&linked)
                .output()
                .expect("worktree");
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let output = std::process::Command::new("git")
                .current_dir(&repository)
                .args(["worktree", "add", "-b", "other-linked"])
                .arg(target.path().join("other-linked"))
                .output()
                .expect("second worktree");
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            linked
        } else {
            repository
        };
        let include = target.path().join("not-opened-include");
        assert!(
            std::process::Command::new("git")
                .current_dir(&receiver)
                .args(["config", "--local", "include.path"])
                .arg(&include)
                .output()
                .expect("declare include")
                .status
                .success()
        );
        let plan = AddPlan::prepare_push(
            sender.path(),
            &serde_json::json!({"remote":receiver}),
            ToolProvenance::Native,
        )
        .expect("plan");
        let plan = Box::new(plan).discover().await.expect("sender discovery");
        let authorization = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("root admission");
        let plan = plan.discover_receiver_roots(&authorization).expect("roots");
        let authorization = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("metadata admission");
        let plan = plan
            .discover_receiver_metadata(&authorization)
            .expect("metadata");
        let authorization = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("common admission");
        let references = plan.receiver_metadata[0].common_path.join("refs");
        let without_reference_read: Vec<_> = authorization
            .iter()
            .filter(|entry| {
                !entry.authorizes_path(
                    plan.operation.name(),
                    ToolNature::Read,
                    &plan.workspace,
                    references.to_str().expect("reference path"),
                )
            })
            .cloned()
            .collect();
        assert!(
            plan.receiver_read_path(&references, &without_reference_read)
                .is_err()
        );
        let plan = plan
            .discover_receiver_common(&authorization)
            .expect("common");
        let authorization = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("backlink admission");
        assert!(plan.require_receiver_reference_capability().is_ok());
        plan.revalidate_receiver_common(&authorization)
            .expect("consistent real layout");
        assert!(
            plan.facets
                .iter()
                .any(|facet| facet.nature == ToolNature::Read
                    && facet.resource.as_deref() == include.to_str())
        );
        assert!(!include.exists());
        assert!(plan.phase == Phase::ReceiverConfiguration);
        assert_eq!(plan.receiver_common.len(), 1);
        assert_eq!(
            plan.receiver_common[0].backlink_target.is_some(),
            kind == "linked"
        );
        assert!(
            plan.facets
                .iter()
                .any(|facet| facet.nature == ToolNature::Execute)
        );
        assert!(
            !plan
                .facets
                .iter()
                .any(|facet| matches!(facet.nature, ToolNature::Write | ToolNature::Network))
        );
        let stale = proof(&plan);
        let authorization = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("configuration admission");
        let plan = plan
            .discover_receiver_configuration(&authorization)
            .await
            .expect("configuration");
        assert!(plan.phase == Phase::ReceiverAuxiliary);
        assert!(plan.require_receiver_reference_capability().is_ok());
        assert_eq!(plan.receiver_configs.len(), 1);
        assert!(
            plan.facets
                .iter()
                .all(|facet| facet.nature == ToolNature::Read)
        );
        assert!(
            stale
                .consume_for_prepared(&plan.request(), &plan.identity)
                .is_err()
        );
        let admission = proof(&plan);
        let authorization = admission
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("auxiliary admission");
        let plan = plan
            .discover_receiver_auxiliary(&authorization)
            .expect("auxiliary evidence");
        assert!(plan.phase == Phase::ReceiverMutation);
        assert!(plan.require_receiver_reference_capability().is_ok());
        assert_eq!(plan.receiver_auxiliary.len(), 1);
        let authorization = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("revalidation admission");
        plan.revalidate_receiver_auxiliary(&authorization)
            .expect("unchanged auxiliaries");
        let mut plan = plan;
        let metadata_facet = plan
            .facets
            .iter()
            .position(|facet| {
                facet.nature == ToolNature::Read
                    && facet.resource_kind == Some(ToolResourceKind::Remote)
                    && facet.resource.as_deref() == Some("git:receiver-metadata")
            })
            .expect("dedicated metadata capability");
        let facet = plan.facets.remove(metadata_facet);
        assert!(
            plan.inspect_receiver_storage(&authorization)
                .expect_err("exact paths alone do not authorize metadata descendants")
                .to_string()
                .contains("Receiver metadata capability missing")
        );
        plan.facets.insert(metadata_facet, facet);
        let inspection = plan
            .inspect_receiver_storage(&authorization)
            .expect("admitted storage");
        assert_eq!(
            inspection.len(),
            plan.receiver_metadata.len() + plan.receiver_common.len()
        );
        for root in plan
            .receiver_metadata
            .iter()
            .map(|metadata| &metadata.canonical)
            .chain(plan.receiver_common.iter().map(|common| &common.canonical))
        {
            let without_root: Vec<_> = authorization
                .iter()
                .filter(|token| {
                    !token.authorizes_path(
                        "git_push",
                        ToolNature::Read,
                        &plan.workspace,
                        root.to_str().expect("receiver root"),
                    )
                })
                .cloned()
                .collect();
            let failure = plan
                .inspect_receiver_storage(&without_root)
                .expect_err("exact root admission required");
            assert!(
                !failure
                    .to_string()
                    .contains("Receiver mutation execution effects are not yet fully bound")
            );
        }
        for leaf in ["HEAD", "config.worktree"] {
            let path = plan.receiver_common[0].canonical.join(leaf);
            let before = std::fs::read(&path).ok();
            let without_read: Vec<_> = authorization
                .iter()
                .filter(|token| {
                    !token.authorizes_path(
                        "git_push",
                        ToolNature::Read,
                        &plan.workspace,
                        path.to_str().expect("main worktree evidence"),
                    )
                })
                .cloned()
                .collect();
            assert!(plan.revalidate_receiver_auxiliary(&without_read).is_err());
            std::fs::write(&path, "changed\n").expect("main worktree change");
            assert!(plan.revalidate_receiver_auxiliary(&authorization).is_err());
            if let Some(bytes) = before {
                std::fs::write(&path, bytes).expect("restore main worktree evidence");
            } else {
                std::fs::remove_file(&path).expect("restore absence");
            }
            plan.revalidate_receiver_auxiliary(&authorization)
                .expect("restored main worktree");
        }
        for (path, merge, _) in &plan.receiver_rebase {
            for read in std::iter::once(path.clone()).chain(
                super::super::receiver_state::RebaseState::leaves(*merge)
                    .iter()
                    .map(|leaf| path.join(leaf)),
            ) {
                let without_read: Vec<_> = authorization
                    .iter()
                    .filter(|token| {
                        !token.authorizes_path(
                            "git_push",
                            ToolNature::Read,
                            &plan.workspace,
                            read.to_str().expect("rebase path"),
                        )
                    })
                    .cloned()
                    .collect();
                assert!(plan.revalidate_receiver_auxiliary(&without_read).is_err());
            }
            std::fs::create_dir(path).expect("begin rebase");
            assert!(plan.revalidate_receiver_auxiliary(&authorization).is_err());
            std::fs::remove_dir(path).expect("restore absent rebase");
            plan.revalidate_receiver_auxiliary(&authorization)
                .expect("restored rebase");
        }
        let bisect_roots: std::collections::BTreeSet<_> =
            std::iter::once(plan.receiver_metadata[0].canonical.clone())
                .chain(std::iter::once(plan.receiver_common[0].canonical.clone()))
                .chain(
                    plan.receiver_siblings
                        .iter()
                        .map(|sibling| sibling.canonical.clone()),
                )
                .collect();
        if kind == "linked" {
            assert_eq!(bisect_roots.len(), 3, "main, receiver and distinct sibling");
        }
        for path in bisect_roots
            .iter()
            .flat_map(|root| ["BISECT_LOG", "BISECT_START"].map(|leaf| root.join(leaf)))
        {
            assert!(
                plan.facets
                    .iter()
                    .any(|facet| facet.nature == ToolNature::Read
                        && facet.resource.as_deref() == path.to_str())
            );
            let without_bisect: Vec<_> = authorization
                .iter()
                .filter(|token| {
                    !token.authorizes_path(
                        "git_push",
                        ToolNature::Read,
                        &plan.workspace,
                        path.to_str().expect("bisect path"),
                    )
                })
                .cloned()
                .collect();
            assert!(plan.revalidate_receiver_auxiliary(&without_bisect).is_err());
            std::fs::write(&path, "refs/heads/bisected\n").expect("bisect change");
            assert!(plan.revalidate_receiver_auxiliary(&authorization).is_err());
            std::fs::remove_file(&path).expect("restore absent bisect state");
            plan.revalidate_receiver_auxiliary(&authorization)
                .expect("restored state");
        }
        for sibling in &plan.receiver_siblings {
            let references = sibling.canonical.join("refs");
            let without_refs: Vec<_> = authorization
                .iter()
                .filter(|token| {
                    !token.authorizes_path(
                        "git_push",
                        ToolNature::Read,
                        &plan.workspace,
                        references.to_str().expect("refs path"),
                    )
                })
                .cloned()
                .collect();
            assert!(plan.revalidate_receiver_auxiliary(&without_refs).is_err());
            let private = references.join("worktree");
            let had_references = references.is_dir();
            let had_private = private.is_dir();
            std::fs::create_dir_all(&private).expect("private refs");
            std::fs::write(
                private.join("occupied"),
                "0000000000000000000000000000000000000000\n",
            )
            .expect("private ref");
            assert!(plan.revalidate_receiver_auxiliary(&authorization).is_err());
            std::fs::remove_file(private.join("occupied")).expect("remove private ref");
            if !had_private {
                std::fs::remove_dir(&private).expect("remove private namespace");
            }
            if !had_references {
                std::fs::remove_dir(&references).expect("restore absent refs");
            }
            plan.revalidate_receiver_auxiliary(&authorization)
                .expect("restored private refs");
            let spelling = sibling.spelling.join("HEAD");
            let without_head: Vec<_> = authorization
                .iter()
                .filter(|token| {
                    !token.authorizes_path(
                        "git_push",
                        ToolNature::Read,
                        &plan.workspace,
                        spelling.to_str().expect("HEAD path"),
                    )
                })
                .cloned()
                .collect();
            assert!(plan.revalidate_receiver_siblings(&without_head).is_err());
            let head = sibling.canonical.join("HEAD");
            let saved = sibling.canonical.join("HEAD.saved-test");
            std::fs::rename(&head, &saved).expect("retain sibling HEAD inode");
            std::fs::write(&head, "ref: refs/heads/new-sibling-branch\n").expect("changed sibling");
            assert!(plan.revalidate_receiver_auxiliary(&authorization).is_err());
            std::fs::remove_file(&head).expect("remove changed HEAD");
            std::fs::rename(&saved, &head).expect("restore sibling");
        }
        let changed_ref = plan.receiver_common[0]
            .canonical
            .join("refs/heads/approval-change");
        std::fs::write(&changed_ref, "ref: refs/heads/changed-target\n")
            .expect("changed reference");
        assert!(plan.revalidate_receiver_auxiliary(&authorization).is_err());
        std::fs::remove_file(changed_ref).expect("restore missing reference");
        let hook = plan.receiver_common[0].canonical.join("hooks/pre-receive");
        std::fs::write(&hook, "nonexecutable replacement").expect("changed auxiliary");
        assert!(plan.revalidate_receiver_auxiliary(&authorization).is_err());
        std::fs::remove_file(&hook).expect("restore missing auxiliary");
        let stale = proof(&plan);
        let plan = plan
            .prepare_receiver_mutation(&authorization)
            .expect("final plan");
        assert!(plan.phase == Phase::Mutation);
        assert!(!plan.receiver_storage.is_empty());
        assert!(
            stale
                .consume_for_prepared(&plan.request(), &plan.identity)
                .is_err()
        );
        let authorization = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("fresh mutation admission");
        let root = &plan.receiver_common[0].canonical;
        let without_write: Vec<_> = authorization
            .iter()
            .filter(|token| {
                !token.authorizes_path(
                    "git_push",
                    ToolNature::Write,
                    &plan.workspace,
                    root.to_str().expect("root"),
                )
            })
            .cloned()
            .collect();
        assert!(
            plan.execute_admitted(&without_write)
                .await
                .expect_err("write required")
                .to_string()
                .contains("lacks Write admission")
        );
        let object = root.join("objects/unapproved-object");
        std::fs::write(&object, "changed storage").expect("object change");
        assert!(
            plan.execute_admitted(&authorization)
                .await
                .expect_err("changed storage")
                .to_string()
                .contains("Receiver metadata changed after approval")
        );
        std::fs::remove_file(object).expect("remove changed storage");
        assert!(!sender.path().join(".git/index").exists());
    }
}

#[tokio::test]
async fn receiver_metadata_wave_defers_pointer_targets_and_binds_changes() {
    for linked in [false, true] {
        let root = fixture();
        let receiver = tempfile::tempdir().expect("receiver");
        let metadata = if linked {
            let path = receiver.path().join("metadata");
            std::fs::create_dir(&path).expect("metadata");
            std::fs::write(receiver.path().join(".git"), "gitdir: metadata\n").expect("gitfile");
            path
        } else {
            receiver.path().to_owned()
        };
        // Nonexistent targets prove discovery does not require their contents.
        let common = receiver.path().join("unopened-common");
        let marker = receiver.path().join("unopened-worktree/.git");
        std::fs::write(
            metadata.join("commondir"),
            format!("{}\n", common.display()),
        )
        .expect("common pointer");
        std::fs::write(metadata.join("gitdir"), format!("{}\n", marker.display()))
            .expect("backpointer");
        let plan = AddPlan::prepare_push(
            root.path(),
            &serde_json::json!({"remote":receiver.path()}),
            ToolProvenance::Native,
        )
        .expect("plan");
        let plan = Box::new(plan).discover().await.expect("sender");
        let authorization = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("root proof");
        let plan = plan.discover_receiver_roots(&authorization).expect("roots");
        let stale = proof(&plan);
        let authorization = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("metadata proof");
        let plan = plan
            .discover_receiver_metadata(&authorization)
            .expect("metadata wave");
        assert!(plan.phase == Phase::ReceiverCommon);
        assert_eq!(plan.receiver_metadata.len(), 1);
        for path in [&common, &marker] {
            assert!(
                plan.facets
                    .iter()
                    .any(|facet| facet.nature == ToolNature::Read
                        && facet.resource.as_deref() == path.to_str())
            );
        }
        assert!(
            plan.facets
                .iter()
                .all(|facet| facet.nature == ToolNature::Read)
        );
        assert!(
            stale
                .consume_for_prepared(&plan.request(), &plan.identity)
                .is_err()
        );
        let admission = proof(&plan);
        std::fs::write(metadata.join("commondir"), "changed\n").expect("change pointer");
        let PreparedNormalStage::Complete(output) = plan.execute_stage(Box::new(admission)).await
        else {
            panic!("stale layout must refuse");
        };
        assert!(
            output.result.is_error && output.result.content.contains("Receiver layout evidence")
        );
        assert!(!common.exists());
        assert!(!marker.exists());
        assert!(!root.path().join(".git/index").exists());
    }
}

#[tokio::test]
async fn receiver_metadata_stage_rejects_changed_root_marker() {
    let root = fixture();
    let receiver = tempfile::tempdir().expect("receiver");
    std::fs::write(receiver.path().join(".git"), "gitdir: unopened\n").expect("marker");
    let plan = AddPlan::prepare_push(
        root.path(),
        &serde_json::json!({"remote":receiver.path()}),
        ToolProvenance::Native,
    )
    .expect("plan");
    let plan = Box::new(plan).discover().await.expect("sender discovery");
    let admission = proof(&plan);
    let authorizations = admission
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("proof");
    let plan = plan
        .discover_receiver_roots(&authorizations)
        .expect("root discovery");
    assert_eq!(plan.receiver_layouts.len(), 1);
    let admission = proof(&plan);
    std::fs::write(receiver.path().join(".git"), "gitdir: replacement\n").expect("retarget");
    let PreparedNormalStage::Complete(output) = plan.execute_stage(Box::new(admission)).await
    else {
        panic!("changed marker must refuse");
    };
    assert!(output.result.is_error);
    assert!(
        output.result.content.contains("Receiver layout evidence"),
        "{}",
        output.result.content
    );
    assert!(!receiver.path().join("unopened").exists());
    assert!(!root.path().join(".git/index").exists());
}

#[tokio::test]
async fn push_discovery_binds_rewritten_destinations_and_requires_fresh_admission() {
    let root = fixture();
    for (key, value) in [
        ("remote.origin.url", "short:repo"),
        ("url.git://example.invalid/.pushInsteadOf", "short:"),
    ] {
        assert!(
            std::process::Command::new("git")
                .args(["config", key, value])
                .current_dir(root.path())
                .output()
                .expect("config")
                .status
                .success()
        );
    }
    let plan = AddPlan::prepare_push(root.path(), &serde_json::json!({}), ToolProvenance::Native)
        .expect("plan");
    assert!(
        !plan
            .facets
            .iter()
            .any(|facet| matches!(facet.nature, ToolNature::Network | ToolNature::Write))
    );
    let stale = proof(&plan);
    let admission = proof(&plan);
    let PreparedNormalStage::Continue(next) =
        Box::new(plan).execute_stage(Box::new(admission)).await
    else {
        panic!("discovery failed")
    };
    assert!(
        next.permission_profile()
            .iter()
            .any(|facet| facet.nature == ToolNature::Network
                && facet.resource.as_deref() == Some("git://example.invalid/repo"))
    );
    assert!(next.permission_profile().iter().any(|facet| {
        facet.nature == ToolNature::Execute
            && facet
                .description
                .as_deref()
                .is_some_and(|text| text.contains("object transfer"))
    }));
    let output = next.execute(Box::new(stale)).await;
    assert!(output.result.is_error && output.result.content.contains("does not authorize"));
    assert!(!root.path().join(".git/index").exists());
}

#[tokio::test]
async fn rerere_refuses_active_records_but_preserves_inert_and_disabled_configurations() {
    for (enabled, cache, records, refused) in [
        (Some("true"), false, false, false),
        (Some("true"), true, true, true),
        (Some("false"), true, true, false),
        (None, true, true, true),
        (None, false, true, false),
    ] {
        let root = fixture();
        if let Some(value) = enabled {
            assert!(
                std::process::Command::new("git")
                    .args(["config", "rerere.enabled", value])
                    .current_dir(root.path())
                    .status()
                    .expect("config")
                    .success()
            );
        }
        if cache {
            std::fs::create_dir(root.path().join(".git/rr-cache")).expect("cache");
        }
        if records {
            std::fs::write(root.path().join(".git/MERGE_RR"), "active record").expect("record");
        }
        let plan = AddPlan::prepare_commit(
            root.path(),
            &serde_json::json!({"message":"first"}),
            ToolProvenance::Native,
        )
        .expect("plan");
        let admission = proof(&plan);
        match Box::new(plan).execute_stage(Box::new(admission)).await {
            PreparedNormalStage::Continue(_) => assert!(!refused),
            PreparedNormalStage::Complete(output) => {
                assert!(refused, "{}", output.result.content);
                assert!(output.result.is_error && output.result.content.contains("rerere replay"));
            }
        }
        assert!(!root.path().join(".git/COMMIT_EDITMSG").exists());
    }
}

#[test]
fn commit_refuses_unmerged_index_before_discovery() {
    let root = fixture();
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(root.path())
            .output()
            .expect("fixture Git");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).expect("UTF8")
    };
    let oid = git(&["hash-object", "-w", "file"]);
    use std::io::Write;
    let mut child = std::process::Command::new("git")
        .args(["update-index", "--index-info"])
        .current_dir(root.path())
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("index fixture");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(format!("100644 {} 2\tfile\n", oid.trim()).as_bytes())
        .expect("unmerged entry");
    assert!(child.wait().expect("index").success());
    let error = AddPlan::prepare_commit(
        root.path(),
        &serde_json::json!({"message":"conflicted"}),
        ToolProvenance::Native,
    )
    .err()
    .expect("unmerged refused");
    assert!(error.to_string().contains("unmerged"));
}

#[tokio::test]
async fn commit_discovery_defers_hooks_and_ignores_inactive_template_and_tag_signing() {
    let root = fixture();
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(root.path())
            .output()
            .expect("Git fixture");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(&["config", "commit.template", "/unavailable/unused-template"]);
    git(&["config", "tag.gpgSign", "true"]);
    git(&["config", "commit.gpgSign", "false"]);
    let plan = AddPlan::prepare_commit(
        root.path(),
        &serde_json::json!({"message":"first"}),
        ToolProvenance::Native,
    )
    .expect("commit");
    let proof = proof(&plan);
    let PreparedNormalStage::Continue(next) = Box::new(plan).execute_stage(Box::new(proof)).await
    else {
        panic!("ordinary config discovery refused")
    };
    for name in [
        "pre-commit",
        "prepare-commit-msg",
        "commit-msg",
        "post-commit",
        "reference-transaction",
        "pre-auto-gc",
    ] {
        assert!(next.permission_profile().iter().any(|facet| {
            facet.nature == ToolNature::Read
                && facet
                    .resource
                    .as_deref()
                    .is_some_and(|resource| resource.ends_with(name))
        }));
    }
    assert!(
        !next
            .permission_profile()
            .iter()
            .any(|facet| facet.resource.as_deref() == Some("/unavailable/unused-template"))
    );
    drop(next);
    assert!(!root.path().join(".git/COMMIT_EDITMSG").exists());
}

pub(crate) async fn execute_approved(plan: AddPlan) -> ToolExecutionOutput {
    let name = plan.operation.name();
    let root = plan.workspace.clone();
    let input = plan.input.clone();
    let provenance = plan.provenance.clone();
    let mut operation: Box<dyn PreparedNormalToolInvocation> = Box::new(plan);
    let mut history = talos_core::tool::PreparedStageHistory::default();
    loop {
        if !history.record_profile(
            operation.plan_identity().expect("identity"),
            operation.permission_profile(),
        ) {
            return ToolExecutionOutput::error("stage limit");
        }
        let state =
            PermissionSessionState::new(PermissionEngine::with_workspace_root(root.clone()));
        let context = PermissionContext::new(
            PermissionMode::Interactive,
            InteractionCapability::Available,
        );
        let request = PermissionRequest::new(
            name,
            provenance.clone(),
            operation.permission_profile(),
            &input,
        );
        let proposal = state
            .propose(&request, &context, GrantScope::Once)
            .expect("proposal");
        let pending = state
            .approve_once(proposal, &request, &context)
            .expect("approval");
        let proof = state
            .try_admit_prepared_invocation(
                pending,
                &request,
                &context,
                operation.plan_identity().expect("identity"),
            )
            .expect("admit");
        match operation.execute_stage(Box::new(proof)).await {
            PreparedNormalStage::Complete(output) => return output,
            PreparedNormalStage::Continue(next) => operation = next,
        }
    }
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

fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("fixture");
    let output = std::process::Command::new("git")
        .arg("init")
        .arg(root.path())
        .output()
        .expect("Git fixture");
    assert!(output.status.success());
    std::fs::write(root.path().join("file"), "before").expect("file");
    root
}

#[test]
fn sender_location_redirection_refuses_before_repository_rediscovery() {
    let root = fixture();
    let plan = AddPlan::prepare(
        root.path(),
        &serde_json::json!({"paths": ["file"]}),
        ToolProvenance::Native,
    )
    .expect("plan");
    plan.revalidate().expect("original location");
    // An invalid new root would fail discovery with a different error if
    // the retained location pointer were not checked first.
    std::fs::write(
        plan.repository.git_dir.join("commondir"),
        root.path().join("unadmitted-root").to_str().expect("path"),
    )
    .expect("redirect pointer");
    let error = plan
        .revalidate()
        .expect_err("old admission must not follow the pointer");
    assert!(
        error.to_string().contains("location pointer changed"),
        "{error}"
    );
    std::fs::remove_file(plan.repository.git_dir.join("commondir")).expect("restore pointer");
    let replacement = fixture();
    std::fs::rename(&plan.repository.git_dir, root.path().join("old-gitdir"))
        .expect("retain original directory");
    std::fs::rename(replacement.path().join(".git"), &plan.repository.git_dir)
        .expect("replace directory");
    let error = plan
        .revalidate()
        .expect_err("replacement root must refuse before content reads");
    assert!(
        error.to_string().contains("root identity changed"),
        "{error}"
    );
}

#[test]
fn inherited_git_redirection_is_refused_without_echoing_values() {
    for name in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_CONFIG_COUNT",
        "GIT_CONFIG_PARAMETERS",
        "GIT_TRACE",
    ] {
        let environment = BTreeMap::from([(OsString::from(name), OsString::from("private-value"))]);
        let error = validate_environment(&environment).expect_err("unsupported redirection");
        assert!(!error.to_string().contains("private-value"));
    }
    assert!(
        validate_environment(&BTreeMap::from([(
            OsString::from("PATH"),
            OsString::from("/usr/bin")
        )]))
        .is_ok()
    );
}

#[cfg(unix)]
#[test]
fn executable_replacement_with_identical_bytes_invalidates_evidence() {
    let root = tempfile::tempdir().expect("directory");
    let program = root.path().join("git");
    std::fs::write(&program, b"identical program bytes").expect("program");
    let program = program.canonicalize().expect("canonical program");
    let before = capture_program(&program).expect("evidence");
    let replacement = root.path().join("replacement");
    std::fs::write(&replacement, b"identical program bytes").expect("replacement");
    std::fs::rename(&replacement, &program).expect("replace");
    let after = capture_program(&program).expect("new evidence");
    assert!(before != after);
}

fn prepare(root: &Path) -> AddPlan {
    AddPlan::prepare(
        root,
        &serde_json::json!({"paths":["file"]}),
        ToolProvenance::Native,
    )
    .expect("plan")
}

#[tokio::test]
async fn direct_and_forged_or_exchanged_proofs_cannot_stage() {
    let root = fixture();
    let tool = super::super::GitAddTool::new(root.path().to_owned());
    let input = serde_json::json!({"paths":["file"]});
    assert!(tool.execute(input.clone()).await.is_error);
    assert!(tool.execute_authorized(input, &[]).await.is_error);
    assert!(
        Box::new(prepare(root.path()))
            .execute(Box::new(()))
            .await
            .result
            .is_error
    );
    let first = prepare(root.path());
    let other = prepare(root.path());
    assert!(
        Box::new(other)
            .execute(Box::new(proof(&first)))
            .await
            .result
            .is_error
    );
    assert!(!root.path().join(".git/index").exists());
}

#[tokio::test]
async fn approval_is_bound_to_prepared_source_and_index() {
    let root = fixture();
    let plan = prepare(root.path());
    let admission = proof(&plan);
    std::fs::write(root.path().join("file"), "after!").expect("edit after approval");
    let PreparedNormalStage::Complete(result) =
        Box::new(plan).execute_stage(Box::new(admission)).await
    else {
        panic!("changed evidence must refuse discovery");
    };
    assert!(result.result.is_error);
    assert!(result.result.content.contains("changed after preparation"));
    assert!(!root.path().join(".git/index").exists());

    let plan = prepare(root.path());
    let admission = proof(&plan);
    std::fs::write(root.path().join(".git/index"), "changed index").expect("index change");
    let PreparedNormalStage::Complete(result) =
        Box::new(plan).execute_stage(Box::new(admission)).await
    else {
        panic!("changed evidence must refuse discovery");
    };
    assert!(result.result.is_error);
    assert_eq!(
        std::fs::read(root.path().join(".git/index")).expect("index"),
        b"changed index"
    );
}

#[tokio::test]
async fn exact_approved_plan_stages_selected_file() {
    let root = fixture();
    let result = execute_approved(prepare(root.path())).await;
    assert!(!result.result.is_error, "{}", result.result.content);
    let output = std::process::Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(root.path())
        .output()
        .expect("index inspection");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"file\0");
}

#[tokio::test]
async fn discovery_defers_auxiliary_reads_until_separate_admission() {
    let root = fixture();
    let outside = tempfile::tempdir().expect("outside workspace");
    // A directory cannot be captured as an auxiliary file. Discovery must
    // nevertheless complete because it has no authority to inspect it yet.
    let status = std::process::Command::new("git")
        .args(["config", "core.attributesFile"])
        .arg(outside.path())
        .current_dir(root.path())
        .status()
        .expect("fixture config");
    assert!(status.success());
    let plan = prepare(root.path());
    let identity = plan.identity.clone();
    let admission = proof(&plan);
    let PreparedNormalStage::Continue(next) =
        Box::new(plan).execute_stage(Box::new(admission)).await
    else {
        panic!("discovery must not read auxiliary directory");
    };
    assert!(
        !next
            .plan_identity()
            .expect("new identity")
            .same_plan(&identity)
    );
    assert!(next.permission_profile().iter().any(|facet| {
        facet.nature == ToolNature::Read
            && facet.resource_kind == Some(ToolResourceKind::Path)
            && facet.resource.as_deref() == outside.path().to_str()
    }));
    drop(next); // Refuse the second stage: no mutation occurs.
    assert!(!root.path().join(".git/index").exists());
}

#[tokio::test]
async fn included_config_preserves_mixed_duplicate_values() {
    let root = fixture();
    let included = root.path().join("included.config");
    std::fs::write(&included, "[custom]\n value\n value =\n value = final\n").expect("include");
    let status = std::process::Command::new("git")
        .args(["config", "include.path"])
        .arg(&included)
        .current_dir(root.path())
        .status()
        .expect("config");
    assert!(status.success());
    let result = execute_approved(prepare(root.path())).await;
    assert!(!result.result.is_error, "{}", result.result.content);
}

#[cfg(unix)]
#[tokio::test]
async fn configured_filter_hook_and_fsmonitor_refuse_before_effects() {
    use std::os::unix::fs::PermissionsExt;
    for helper in ["filter", "hook", "fsmonitor"] {
        let root = fixture();
        match helper {
            "filter" => {
                std::fs::write(root.path().join(".gitattributes"), "file filter=blocked\n")
                    .expect("attributes");
                let status = std::process::Command::new("git")
                    .args(["config", "filter.blocked.clean", "touch helper-ran; cat"])
                    .current_dir(root.path())
                    .status()
                    .expect("filter config");
                assert!(status.success());
            }
            "hook" => {
                let hook = root.path().join(".git/hooks/post-index-change");
                std::fs::write(&hook, "#!/bin/sh\ntouch helper-ran\n").expect("hook");
                std::fs::set_permissions(hook, std::fs::Permissions::from_mode(0o755))
                    .expect("executable");
            }
            _ => {
                let status = std::process::Command::new("git")
                    .args(["config", "core.fsmonitor", "touch helper-ran"])
                    .current_dir(root.path())
                    .status()
                    .expect("fsmonitor config");
                assert!(status.success());
            }
        }
        let result = execute_approved(prepare(root.path())).await;
        assert!(result.result.is_error, "helper {helper} must be refused");
        assert!(
            result.result.content.contains(helper),
            "{}",
            result.result.content
        );
        assert!(!root.path().join("helper-ran").exists());
        assert!(!root.path().join(".git/index").exists());
    }
}

#[tokio::test]
async fn disabled_fsmonitor_spellings_match_host_git() {
    for value in [
        "false", "0", "off", "no", "FALSE", "Off", "", "00", "0k", "0M", "-0g",
    ] {
        let root = fixture();
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .args(args)
                .current_dir(root.path())
                .output()
                .expect("host Git");
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            output.stdout
        };
        git(&["config", "core.fsmonitor", value]);
        assert_eq!(git(&["config", "--bool", "core.fsmonitor"]), b"false\n");
        let result = execute_approved(prepare(root.path())).await;
        assert!(
            !result.result.is_error,
            "{value:?}: {}",
            result.result.content
        );
        assert_eq!(
            git(&["show", ":file"]),
            std::fs::read(root.path().join("file")).expect("source")
        );
    }
}
