//! Native whole-operation baselines for replacing the staged Pull scheduler.
//! These exercise the owned runner, not prepared Pull authorization/activation.

use super::*;

fn git(cwd: &Path, args: &[&str]) -> Vec<u8> {
    let output = std::process::Command::new("git")
        .current_dir(cwd)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", cwd.join("absent-global-config"))
        .env("GIT_AUTHOR_NAME", "Native Pull fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Native Pull fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .args(["-c", "core.hooksPath=", "-c", "commit.gpgSign=false"])
        .args(args)
        .output()
        .expect("native Git fixture");
    assert!(output.status.success(), "{args:?}: {output:?}");
    output.stdout
}

async fn native_pull_preserves_configured_integration(conflict: bool) {
    let root = tempfile::tempdir().expect("fixture");
    let source = root.path().join("source");
    let local = root.path().join("local");
    std::fs::create_dir(&source).expect("source");
    git(&source, &["init", "-b", "main"]);
    std::fs::write(source.join("file"), "base\n").expect("base");
    git(&source, &["add", "file"]);
    git(&source, &["commit", "-m", "base"]);
    git(
        root.path(),
        &[
            "clone",
            "--origin",
            "upstream",
            source.to_str().expect("source path"),
            local.to_str().expect("local path"),
        ],
    );
    git(
        &local,
        &[
            "config",
            "pull.rebase",
            if conflict { "false" } else { "true" },
        ],
    );
    let local_path = if conflict { "file" } else { "local-file" };
    std::fs::write(local.join(local_path), "local\n").expect("local change");
    git(&local, &["add", local_path]);
    git(&local, &["commit", "-m", "local"]);
    let before = git(&local, &["rev-parse", "HEAD"]);
    std::fs::write(source.join("file"), "remote\n").expect("remote change");
    git(&source, &["commit", "-am", "remote"]);
    let remote = git(&source, &["rev-parse", "HEAD"]);

    let plan = prepare(&local);
    let mut environment = plan.environment.clone();
    environment.insert("GIT_CONFIG_NOSYSTEM".into(), "1".into());
    environment.insert(
        "GIT_CONFIG_GLOBAL".into(),
        root.path().join("absent-global-config").into_os_string(),
    );
    environment.insert("GIT_COMMITTER_NAME".into(), "Native Pull fixture".into());
    environment.insert(
        "GIT_COMMITTER_EMAIL".into(),
        "fixture@example.invalid".into(),
    );
    let arguments = [
        "-c",
        "maintenance.autoDetach=false",
        "-c",
        "gc.autoDetach=false",
        "-c",
        "core.hooksPath=",
        "-c",
        "commit.gpgSign=false",
        "--no-pager",
        "pull",
    ];
    let frozen = command::FrozenCommand::new(&plan.program, &local, arguments, &environment)
        .expect("freeze native Pull");
    let output = plan
        .run_command_output(
            frozen.command(),
            std::time::Duration::from_secs(20),
            1024 * 1024,
        )
        .await
        .expect("supervised native Pull");
    assert_eq!(git(&local, &["rev-parse", "upstream/main"]), remote);
    if conflict {
        assert!(!output.status.success(), "{output:?}");
        assert_eq!(git(&local, &["rev-parse", "HEAD"]), before);
        assert_eq!(git(&local, &["rev-parse", "MERGE_HEAD"]), remote);
        assert!(!git(&local, &["ls-files", "--unmerged"]).is_empty());
        assert!(
            std::fs::read_to_string(local.join("file"))
                .expect("conflict")
                .contains("<<<<<<<")
        );
    } else {
        assert!(output.status.success(), "{output:?}");
        assert_ne!(git(&local, &["rev-parse", "HEAD"]), before);
        assert_eq!(git(&local, &["rev-parse", "HEAD^"]), remote);
        assert_eq!(
            std::fs::read(local.join("local-file")).expect("local file"),
            b"local\n"
        );
        assert_eq!(
            std::fs::read(local.join("file")).expect("remote file"),
            b"remote\n"
        );
    }
}

#[tokio::test]
async fn native_pull_owned_runner_uses_upstream_and_configured_rebase() {
    native_pull_preserves_configured_integration(false).await;
}

#[tokio::test]
async fn native_pull_owned_runner_retains_fetch_and_conflict_effects() {
    native_pull_preserves_configured_integration(true).await;
}

#[tokio::test]
async fn native_pull_owned_runner_preserves_configured_submodule_recursion() {
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
    let arguments = [
        "-c",
        "maintenance.autoDetach=false",
        "-c",
        "gc.autoDetach=false",
        "-c",
        "core.hooksPath=",
        "-c",
        "protocol.file.allow=always",
        "--no-pager",
        "pull",
    ];
    let frozen = command::FrozenCommand::new(&plan.program, &local, arguments, &environment)
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
