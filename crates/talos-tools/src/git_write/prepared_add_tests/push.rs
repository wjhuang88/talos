// Local receiver probes use the same fresh internal-program admission as the
// production ReceiverRoots dispatcher before capturing child executables.
async fn discover_receiver_sender_approved(plan: AddPlan) -> Box<AddPlan> {
    let mut plan = Box::new(plan).discover().await.expect("sender discovery");
    assert!(plan.phase == Phase::ReceiverRoots);
    if cfg!(windows) {
        let tokens = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("receiver sender program admission");
        plan = plan.rebind_push_program(&tokens).await.expect("receiver sender binding");
    }
    plan
}

// Network Push reaches its mutation plan only after Windows internal Git has
// received fresh admission. Local receivers retain their separate root stage.
async fn finish_push_program_approved(mut plan: Box<AddPlan>) -> Box<AddPlan> {
    if plan.phase == Phase::PushProgram {
        let tokens = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("Push program admission");
        plan = plan.rebind_push_program(&tokens).await.expect("Push program binding");
        plan.facets.append(&mut plan.sender_mutation_facets);
        plan.phase = Phase::Mutation;
        plan.identity = PreparedPlanIdentity::new();
    }
    plan
}

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
    // Exercise builtin dispatch with an empty GIT_EXEC_PATH.
    // This is a command-level fixture; structured preparation intentionally
    // rejects GIT_EXEC_PATH overrides without separate authorization.
    let exec_path = tempfile::tempdir().expect("empty exec-path");
    let mut command = tokio::process::Command::new(&program);
    command
        .current_dir(sender.path())
        .env("GIT_EXEC_PATH", exec_path.path())
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
    let mut plan = discover_receiver_sender_approved(plan).await;
    // This absent child has no Read facet. Admission must fail before capture
    // could report that its executable is unavailable.
    plan.receiver_child_paths = vec![sender.path().join("unadmitted-child")];
    let tokens = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("root admission");
    let Err(error) = plan.discover_receiver_roots(&tokens) else {
        panic!("unadmitted child cannot advance discovery");
    };
    assert!(error.to_string().contains("Receiver Read authorization missing"));
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
    let mut plan = discover_receiver_sender_approved(plan).await;
    // Substitute only the shell fixture. Git children must retain their
    // identity relationship with the configuration-discovery executable.
    plan.receiver_child_paths[0] = child.clone();
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
async fn receiver_reference_tree_refuses_exact_paths_without_closed_capability() {
    let sender = fixture();
    let receiver = fixture();
    let plan = AddPlan::prepare_push(
        sender.path(),
        &serde_json::json!({"remote": receiver.path()}),
        ToolProvenance::Native,
    )
    .expect("plan");
    let mut plan = discover_receiver_sender_approved(plan).await;
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
    let daemon_log = tempfile::NamedTempFile::new().expect("daemon log");
    let mut daemon = tokio::process::Command::new("git")
        .args([
            "daemon",
            "--verbose",
            "--reuseaddr",
            "--listen=127.0.0.1",
            "--export-all",
            "--enable=receive-pack",
        ])
        .arg(format!("--port={port}"))
        .arg(format!(
            "--base-path={}",
            daemon_fixture_path(receiver.path().parent().expect("parent"))
        ))
        .arg(daemon_fixture_path(receiver.path()))
        .kill_on_drop(true)
        .stdout(std::process::Stdio::null())
        .stderr(daemon_log.reopen().expect("daemon log writer"))
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
    let next = finish_push_program_approved(next).await;
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
        let next = finish_push_program_approved(next).await;
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
    let next = finish_push_program_approved(next).await;
    let admission = proof(&next);
    let output = next.execute(Box::new(admission)).await;
    assert!(
        !output.result.is_error,
        "{}; daemon_state={:?}; receiver_ref_written={}; daemon_log={}",
        output.result.content,
        daemon.try_wait().expect("daemon state"),
        receiver.path().join("refs/heads/approved").exists(),
        std::fs::read_to_string(daemon_log.path()).expect("daemon diagnostics")
    );
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
    let next = finish_push_program_approved(next).await;
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
    let next = finish_push_program_approved(next).await;
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
        let next = finish_push_program_approved(next).await;
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
        let next = finish_push_program_approved(next).await;
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
        let next = finish_push_program_approved(next).await;
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
        ToolNature::Network | ToolNature::Write
    )));
    assert_eq!(next.permission_profile().iter().any(|facet|
        facet.nature == ToolNature::Execute), cfg!(windows));
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
    let plan = discover_receiver_sender_approved(plan).await;
    assert!(plan.phase == Phase::ReceiverRoots);
    let wrong = Box::new(plan).execute_stage(Box::new(stale)).await;
    assert!(matches!(wrong, PreparedNormalStage::Complete(output) if output.result.is_error));

    let plan = AddPlan::prepare_push(
        root.path(),
        &serde_json::json!({"remote":receiver.path()}),
        ToolProvenance::Native,
    )
    .expect("sender plan");
    let plan = discover_receiver_sender_approved(plan).await;
    let prior_facets = plan.permission_profile().to_vec();
    let authorizations = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("root admission");
    // The helper already performed the Windows program-rebinding half of the
    // ReceiverRoots dispatcher; finish its root-discovery half exactly once.
    let next = plan
        .discover_receiver_roots(&authorizations)
        .expect("admitted root must produce metadata continuation");
    assert!(next.phase == Phase::ReceiverMetadata);
    assert!(
        next.permission_profile()
            .iter()
            .any(|facet| facet.nature == ToolNature::Read
                && facet.resource.as_deref() == external.to_str())
    );
    assert!(
        next.permission_profile()
            .iter()
            .all(|facet| facet.nature == ToolNature::Read || prior_facets.contains(facet))
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
        let mut plan = discover_receiver_sender_approved(plan).await;
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
    let plan = discover_receiver_sender_approved(plan).await;
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
    let plan = discover_receiver_sender_approved(plan).await;
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
        let plan = discover_receiver_sender_approved(plan).await;
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
        let plan = discover_receiver_sender_approved(plan).await;
        let authorization = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("root proof");
        let plan = plan.discover_receiver_roots(&authorization).expect("roots");
        let prior_facets = plan.facets.clone();
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
                .all(|facet| facet.nature == ToolNature::Read || prior_facets.contains(facet))
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
    let plan = discover_receiver_sender_approved(plan).await;
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
    let next = Box::new(plan).discover().await.expect("sender discovery");
    let next = finish_push_program_approved(next).await;
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
