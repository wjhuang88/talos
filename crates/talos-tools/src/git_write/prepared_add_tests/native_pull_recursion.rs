//! Native recursion scope includes stored repositories without a populated worktree.

use super::*;

#[cfg(unix)]
#[tokio::test]
async fn native_pull_local_source_can_execute_upload_pack_configuration() {
    local_source_upload_pack_configuration(true).await;
    local_source_upload_pack_configuration(false).await;
}

#[cfg(unix)]
async fn local_source_upload_pack_configuration(protected_global: bool) {
    let root = tempfile::tempdir().expect("fixture");
    let source = root.path().join("source");
    let local = root.path().join("local");
    std::fs::create_dir(&source).expect("source");
    git(&source, &["init", "-b", "main"]);
    std::fs::write(source.join("file"), "before\n").expect("file");
    git(&source, &["add", "file"]);
    git(&source, &["commit", "-m", "base"]);
    git(
        root.path(),
        &[
            "clone",
            source.to_str().expect("path"),
            local.to_str().expect("path"),
        ],
    );
    std::fs::write(source.join("file"), "after\n").expect("update");
    git(&source, &["commit", "-am", "update"]);

    let plan = prepare(&local);
    let global = root.path().join("global-config");
    std::fs::write(
        &global,
        if protected_global {
            "[uploadpack]\npackObjectsHook = printf observed > upload-pack-effect; exit 73;\n"
        } else {
            ""
        },
    )
    .expect("global configuration");
    if !protected_global {
        git(
            &source,
            &[
                "config",
                "uploadpack.packObjectsHook",
                "printf observed > upload-pack-effect; exit 73;",
            ],
        );
    }
    let mut environment = plan.environment.clone();
    environment.insert("GIT_CONFIG_NOSYSTEM".into(), "1".into());
    environment.insert("GIT_CONFIG_GLOBAL".into(), global.into_os_string());
    // Protected global configuration reaches the local upload-pack.
    // Deliberately fail after a marker: this is an effect baseline, not an
    // authorized production Pull or a successful integration claim.
    fixture_config(&mut environment, &[("core.hooksPath", "")]);
    let frozen = command::FrozenCommand::pull(
        &plan.program,
        &local,
        &GitPullInput { remote: None },
        &environment,
    )
    .expect("command");
    let output = plan
        .run_command_output(
            frozen.command(),
            std::time::Duration::from_secs(20),
            1024 * 1024,
        )
        .await
        .expect("supervised native Pull");
    if !protected_global {
        assert!(output.status.success(), "{output:?}");
        assert!(!source.join(".git/upload-pack-effect").exists());
        assert_eq!(
            std::fs::read(local.join("file")).expect("updated file"),
            b"after\n"
        );
        return;
    }
    assert!(!output.status.success());
    assert_eq!(
        std::fs::read(source.join(".git/upload-pack-effect")).expect("source-side effect"),
        b"observed"
    );
    assert_eq!(
        std::fs::read(local.join("file")).expect("unchanged local file"),
        b"before\n"
    );
}

#[tokio::test]
async fn native_pull_owned_runner_preserves_configured_submodule_recursion() {
    native_pull_recursion(false).await;
}

#[tokio::test]
async fn native_pull_fetches_deinitialized_stored_submodule() {
    native_pull_recursion(true).await;
}

async fn native_pull_recursion(deinitialized: bool) {
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
    if deinitialized {
        git(&local, &["submodule", "deinit", "-f", "nested"]);
        assert!(!local.join("nested/.git").exists());
    }
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
    // Git fetches the retained module even after deinit removed its .git marker
    // and submodule.<name>.url from parent config. Current worktree gitlinks or
    // initialized markers alone cannot describe the native operation's effects.
    assert_eq!(
        git(
            &local,
            &["--git-dir=.git/modules/nested", "rev-parse", "origin/main"],
        ),
        expected
    );
    // Native Pull also repopulates the deinitialized worktree when the fetched
    // parent advances its gitlink. Its Write scope must include that worktree,
    // rather than authorizing only the stored repository's fetch metadata.
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
