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
    let mut command = tokio::process::Command::new(&plan.program);
    command
        .current_dir(&local)
        .env_clear()
        .envs(&plan.environment)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env(
            "GIT_CONFIG_GLOBAL",
            root.path().join("absent-global-config"),
        )
        .env("GIT_COMMITTER_NAME", "Native Pull fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .args([
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
        ]);
    let output = plan
        .run_command_output(command, std::time::Duration::from_secs(20), 1024 * 1024)
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
