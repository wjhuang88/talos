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
