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
    let config_path = home.path().join(".gitconfig");
    for (key, name) in [
        ("normalTarget", "normal"),
        ("eventTarget", "event"),
        ("perfTarget", "perf"),
    ] {
        let output = std::process::Command::new("git")
            .args(["config", "--file"])
            .arg(&config_path)
            .arg(format!("trace2.{key}"))
            .arg(home.path().join(name))
            .output()
            .expect("configure trace target");
        assert!(output.status.success(), "{output:?}");
    }
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
                    "maintenance.autoDetach=false",
                    "-c",
                    "gc.autoDetach=false",
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
            .contains("configuration inspection runner failed; configuration values withheld"),
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
            .contains("configuration inspection runner failed; configuration values withheld")
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
    let Err(error) = discover_sender_approved(Box::new(plan)).await else {
        panic!("redirect was allowed")
    };
    assert!(error.to_string().contains("gc.repackFilterTo"));
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
    // nevertheless complete: metadata resolution is admitted, but directory
    // contents are not. The next frontier names the resolved target.
    let status = std::process::Command::new("git")
        .args(["config", "core.attributesFile"])
        .arg(outside.path())
        .current_dir(root.path())
        .status()
        .expect("fixture config");
    assert!(status.success());
    let plan = prepare(root.path());
    let identity = plan.identity.clone();
    // Windows may first require admission of Git's distinct internal binary.
    // Admit only sender discovery frontiers, never the auxiliary content Read.
    let next = discover_sender_approved(Box::new(plan))
        .await
        .expect("discovery must not read auxiliary directory");
    assert!(
        !next
            .plan_identity()
            .expect("new identity")
            .same_plan(&identity)
    );
    let resolved = outside.path().canonicalize().expect("resolved outside target");
    assert!(next.permission_profile().iter().any(|facet| {
        facet.nature == ToolNature::Read
            && facet.resource_kind == Some(ToolResourceKind::Path)
            // The resolver preserves admitted Windows short-name spelling;
            // verify the frontier names this directory rather than requiring
            // canonicalize's different long-name/verbatim representation.
            && facet.resource.as_deref().is_some_and(|resource| {
                Path::new(resource).canonicalize().ok().as_ref() == Some(&resolved)
            })
    }));
    drop(next); // Refuse the second stage: no mutation occurs.
    assert!(!root.path().join(".git/index").exists());
}

#[tokio::test]
async fn mutation_refuses_auxiliary_replacement_after_discovery() {
    for existed in [false, true] {
        let root = fixture();
        let outside = tempfile::tempdir().expect("external attributes");
        let source = outside.path().join("attributes");
        if existed {
            std::fs::write(&source, b"file -filter\n").expect("original attributes");
        }
        assert!(std::process::Command::new("git")
            .args(["config", "core.attributesFile"])
            .arg(&source)
            .current_dir(root.path())
            .status().expect("configure attributes").success());
        let plan = Box::new(prepare(root.path())).discover().await.expect("discovery");
        let admission = proof(&plan);
        if existed {
            std::fs::rename(&source, outside.path().join("retained"))
                .expect("retain original inode");
        }
        std::fs::write(&source, b"file -filter\n").expect("new attributes inode");
        let output = plan.execute(Box::new(admission)).await;
        assert!(output.result.is_error, "existed={existed}");
        assert!(!root.path().join(".git/index").exists());
    }
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
