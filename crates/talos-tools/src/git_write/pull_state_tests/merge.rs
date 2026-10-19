//! Pull merge regression coverage.

use super::*;

#[cfg(test)]
#[tokio::test]
async fn real_pull_obeys_branch_ff_and_autostash_policy() {
    for scenario in [
        "branch-merge",
        "branch-rebase",
        "rewritten-upstream",
        "configured-remote",
        "custom-tracking",
        "missing-tracking",
        "fast-forward",
        "merge-no-ff",
        "unspecified-policy",
        "ff-only",
        "stash",
        "no-stash",
    ] {
        let root = tempfile::tempdir().expect("fixture");
        let source = root.path().join("source");
        let local = root.path().join("local");
        std::fs::create_dir(&source).expect("source directory");
        let run = |cwd: &std::path::Path, args: &[&str]| {
            std::process::Command::new("git")
                .current_dir(cwd)
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_GLOBAL", root.path().join("absent-config"))
                .env("GIT_TERMINAL_PROMPT", "0")
                .env("GIT_EDITOR", "true")
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
        let git = |cwd: &std::path::Path, args: &[&str]| {
            let output = run(cwd, args);
            assert!(output.status.success(), "{scenario} {args:?}: {output:?}");
            String::from_utf8(output.stdout).expect("output")
        };
        git(&source, &["init", "-b", "Topic"]);
        std::fs::write(source.join("dirty"), "base\n").expect("base file");
        git(&source, &["add", "."]);
        git(&source, &["commit", "-m", "base"]);
        let base = git(&source, &["rev-parse", "HEAD"]);
        let old_upstream = if scenario == "rewritten-upstream" {
            git(
                &source,
                &["commit", "--allow-empty", "-m", "obsolete upstream"],
            );
            Some(git(&source, &["rev-parse", "HEAD"]))
        } else {
            None
        };
        git(
            root.path(),
            &[
                "clone",
                source.to_str().expect("source"),
                local.to_str().expect("local"),
            ],
        );
        if scenario != "fast-forward" {
            std::fs::write(local.join("local-only"), "local\n").expect("local change");
            git(&local, &["add", "."]);
            git(&local, &["commit", "-m", "local"]);
        }
        let before = git(&local, &["rev-parse", "HEAD"]);
        if let Some(old) = &old_upstream {
            assert_eq!(
                git(
                    &local,
                    &["merge-base", "--fork-point", "origin/Topic", "HEAD"]
                ),
                *old
            );
            git(&source, &["reset", "--hard", base.trim()]);
        }
        std::fs::write(source.join("remote-only"), "remote\n").expect("remote change");
        git(&source, &["add", "."]);
        git(&source, &["commit", "-m", "remote"]);
        let remote = git(&source, &["rev-parse", "HEAD"]);
        if scenario == "configured-remote" {
            git(&local, &["remote", "rename", "origin", "Up.stream"]);
        }
        if scenario == "custom-tracking" {
            git(
                &local,
                &[
                    "config",
                    "--replace-all",
                    "remote.origin.fetch",
                    "+refs/heads/*:refs/custom/*",
                ],
            );
            git(
                &local,
                &[
                    "update-ref",
                    "refs/custom/Topic",
                    "refs/remotes/origin/Topic",
                ],
            );
        }
        let merging = matches!(scenario, "branch-merge" | "merge-no-ff");
        git(
            &local,
            &[
                "config",
                "pull.rebase",
                if merging { "true" } else { "false" },
            ],
        );
        git(
            &local,
            &[
                "config",
                "branch.Topic.rebase",
                if merging { "false" } else { "true" },
            ],
        );
        if scenario == "unspecified-policy" {
            git(&local, &["config", "--unset", "pull.rebase"]);
            git(&local, &["config", "--unset", "branch.Topic.rebase"]);
        }
        if scenario == "ff-only" {
            git(&local, &["config", "pull.ff", "only"]);
        }
        if scenario == "merge-no-ff" {
            git(&local, &["config", "pull.ff", "false"]);
        }
        if matches!(scenario, "stash" | "no-stash") {
            git(
                &local,
                &[
                    "config",
                    "rebase.autostash",
                    if scenario == "stash" { "false" } else { "true" },
                ],
            );
            git(
                &local,
                &[
                    "config",
                    "pull.autostash",
                    if scenario == "stash" { "true" } else { "false" },
                ],
            );
            std::fs::write(local.join("dirty"), "uncommitted\n").expect("dirty worktree");
        }
        let config = run(&local, &["config", "--null", "--list"]);
        assert!(config.status.success());
        let policy = Policy::capture(&config.stdout, Some("Topic")).expect("policy");
        let selected = fetch_remote(&config.stdout, Some("Topic"), None).expect("remote selection");
        assert_eq!(
            selected,
            if scenario == "configured-remote" {
                "Up.stream"
            } else {
                "origin"
            }
        );
        assert_eq!(
            policy.rebase,
            if merging || scenario == "unspecified-policy" {
                Rebase::Off
            } else {
                Rebase::On
            }
        );
        assert_eq!(policy.autostash, scenario == "stash");
        let tracking =
            tracking_reference(&config.stdout, "Topic", &selected).expect("upstream mapping");
        let actual = git(
            &local,
            &["rev-parse", "--symbolic-full-name", "@{upstream}"],
        );
        assert_eq!(tracking.as_deref(), Some(actual.trim()));
        if scenario == "missing-tracking" {
            git(
                &local,
                &["update-ref", "-d", tracking.as_deref().expect("tracking")],
            );
        }
        let repository = super::super::repository_state::RepositoryState::capture(&local)
            .expect("repository evidence");
        let before_evidence = PullEvidence::capture(&repository, tracking.as_deref(), false)
            .expect("pre-fetch reference/reflog evidence");
        let retained = BeforeFetch::probe(&config.stdout, tracking.as_deref(), 40, |args| {
            let output = run(&local, &args.iter().map(String::as_str).collect::<Vec<_>>());
            async move { Ok(output) }
        })
        .await
        .expect("actual pre-fetch probes");
        before_evidence
            .revalidate()
            .expect("stable pre-fetch probes");
        if scenario == "branch-rebase" {
            let reflog = local
                .join(".git/logs")
                .join(tracking.as_deref().expect("tracking"));
            let saved = match std::fs::read(&reflog) {
                Ok(bytes) => Some(bytes),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                Err(error) => panic!("tracking reflog: {error}"),
            };
            std::fs::create_dir_all(reflog.parent().expect("reflog parent"))
                .expect("reflog directory");
            // Capture after constructing the missing parent hierarchy, so this
            // regression isolates leaf creation/content from parent movement.
            let before_evidence = PullEvidence::capture(&repository, tracking.as_deref(), false)
                .expect("reflog leaf evidence");
            let mut changed = saved.clone().unwrap_or_default();
            changed.extend_from_slice(b"changed evidence\n");
            std::fs::write(&reflog, changed).expect("mutate reflog only");
            assert!(
                before_evidence.revalidate().is_err(),
                "unchanged refs cannot hide reflog movement"
            );
            if let Some(saved) = saved {
                std::fs::write(&reflog, saved).expect("restore reflog");
            } else {
                std::fs::remove_file(&reflog).expect("restore missing reflog");
            }
            before_evidence.revalidate().expect("restored evidence");
        }
        if scenario == "missing-tracking" {
            assert_eq!(retained.fork_point, None);
        }
        // Compare the explicit fetch/integration boundary with native pull on
        // the same repository, including its reflogs and configuration.
        fn copy_tree(source: &std::path::Path, destination: &std::path::Path) {
            std::fs::create_dir(destination).expect("copy directory");
            for entry in std::fs::read_dir(source).expect("copy entries") {
                let entry = entry.expect("entry");
                let target = destination.join(entry.file_name());
                if entry.file_type().expect("entry type").is_dir() {
                    copy_tree(&entry.path(), &target);
                } else {
                    std::fs::copy(entry.path(), target).expect("copy fixture file");
                }
            }
        }
        let split = root.path().join("split");
        copy_tree(&local, &split);
        git(&split, &["fetch", &selected]);
        let fetched = std::fs::read(split.join(".git/FETCH_HEAD")).expect("fetched evidence");
        if scenario == "branch-rebase" {
            git(&split, &["update-ref", "HEAD", remote.trim()]);
            let moved = retained
                .integration(&fetched, 40, |args| {
                    let output = run(&split, &args.iter().map(String::as_str).collect::<Vec<_>>());
                    async move { Ok(output) }
                })
                .await
                .expect("post-fetch HEAD movement needs explicit reconciliation");
            assert!(
                matches!(moved, Integration::Reconcile { original, current, .. } if original == before.trim() && current == remote.trim())
            );
            git(&split, &["update-ref", "HEAD", before.trim()]);
        }
        let fetched_repository = super::super::repository_state::RepositoryState::capture(&split)
            .expect("fetched repository");
        let fetched_evidence =
            PullEvidence::capture(&fetched_repository, tracking.as_deref(), true)
                .expect("post-fetch evidence");
        assert_eq!(
            fetched_evidence.fetch_head().expect("bound FETCH_HEAD"),
            fetched
        );
        let failed_probe = retained
            .integration(&fetched, 40, |_| async {
                Err(GitToolError::Git("fixture collector failure".into()))
            })
            .await
            .expect_err("infrastructure failure must propagate");
        assert!(
            failed_probe
                .to_string()
                .contains("fixture collector failure")
        );
        let integration = retained
            .integration(
                fetched_evidence.fetch_head().expect("bound FETCH_HEAD"),
                40,
                |args| {
                    let output = run(&split, &args.iter().map(String::as_str).collect::<Vec<_>>());
                    async move { Ok(output) }
                },
            )
            .await;
        fetched_evidence
            .revalidate()
            .expect("stable integration probes");
        if scenario == "branch-rebase" {
            let path = split.join(".git/FETCH_HEAD");
            let mut changed = fetched.clone();
            changed.extend_from_slice(b"changed evidence\n");
            std::fs::write(&path, changed).expect("change FETCH_HEAD after inspection");
            assert!(fetched_evidence.revalidate().is_err());
            std::fs::write(&path, &fetched).expect("restore FETCH_HEAD");
            fetched_evidence.revalidate().expect("restored FETCH_HEAD");
        }
        let split_success = match integration {
            Ok(plan @ (Integration::Merge { .. } | Integration::Rebase { .. })) => {
                let commands = plan.commands().expect("integration commands");
                assert_eq!(commands.len(), 1);
                let args = &commands[0];
                run(&split, &args.iter().map(String::as_str).collect::<Vec<_>>())
                    .status
                    .success()
            }
            Ok(Integration::Unborn(_) | Integration::Reconcile { .. }) => {
                panic!("unchanged born fixture")
            }
            Err(error) => {
                assert!(
                    matches!(scenario, "ff-only" | "unspecified-policy"),
                    "unexpected integration refusal: {error}"
                );
                false
            }
        };
        let output = run(&local, &["pull"]);
        let refused = matches!(scenario, "ff-only" | "no-stash" | "unspecified-policy");
        assert_eq!(output.status.success(), !refused, "{scenario}: {output:?}");
        assert_eq!(
            split_success,
            output.status.success(),
            "split/native status: {scenario}"
        );
        assert_eq!(
            git(&split, &["rev-parse", "HEAD^{tree}"]),
            git(&local, &["rev-parse", "HEAD^{tree}"])
        );
        assert_eq!(
            git(&split, &["log", "--format=%s"]),
            git(&local, &["log", "--format=%s"])
        );
        if refused {
            assert_eq!(git(&local, &["rev-parse", "HEAD"]), before);
        } else {
            let parents = git(&local, &["rev-list", "--parents", "-n", "1", "HEAD"]);
            assert_eq!(
                parents.split_whitespace().count(),
                if merging { 3 } else { 2 }
            );
            git(
                &local,
                &["merge-base", "--is-ancestor", remote.trim(), "HEAD"],
            );
            if let Some(old) = &old_upstream {
                assert_eq!(retained.head.as_deref(), Some(before.trim()));
                assert_eq!(retained.fork_point.as_deref(), Some(old.trim()));
                let obsolete = run(&local, &["merge-base", "--is-ancestor", old.trim(), "HEAD"]);
                assert_eq!(
                    obsolete.status.code(),
                    Some(1),
                    "obsolete upstream replayed"
                );
                assert_eq!(git(&local, &["rev-list", "--count", "HEAD"]).trim(), "3");
            }
        }
        if matches!(scenario, "stash" | "no-stash") {
            assert_eq!(
                std::fs::read(local.join("dirty")).expect("preserved edits"),
                b"uncommitted\n"
            );
            assert_eq!(
                std::fs::read(split.join("dirty")).expect("split preserved edits"),
                b"uncommitted\n"
            );
            assert_eq!(
                git(&split, &["stash", "list"]),
                git(&local, &["stash", "list"])
            );
        }
    }
}

/// A closed stdin does not prevent an inherited auto-edit request from
/// executing an editor. Keep this native behavior visible to effect discovery.
#[cfg(unix)]
#[test]
fn pull_merge_editor_effect_depends_on_inherited_autoedit() {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir().expect("fixture");
    let marker = root.path().join("editor-ran");
    let editor = root.path().join("editor");
    std::fs::write(&editor, "#!/bin/sh\nprintf editor > editor-ran\n").expect("editor");
    std::fs::set_permissions(&editor, std::fs::Permissions::from_mode(0o700))
        .expect("executable editor");
    let run = |args: &[&str], autoedit: Option<&str>| {
        let mut command = std::process::Command::new("git");
        command
            .current_dir(root.path())
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", root.path().join("absent-config"))
            .env_remove("GIT_CONFIG_COUNT")
            .env_remove("GIT_CONFIG_PARAMETERS")
            .env_remove("GIT_MERGE_AUTOEDIT")
            .env("GIT_EDITOR", "./editor")
            .stdin(std::process::Stdio::null())
            .args([
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "-c",
                "commit.gpgsign=false",
                "-c",
                "maintenance.auto=false",
            ])
            .args(args);
        if let Some(value) = autoedit {
            command.env("GIT_MERGE_AUTOEDIT", value);
        }
        let output = command.output().expect("native Git");
        assert!(output.status.success(), "{args:?}: {output:?}");
        String::from_utf8(output.stdout).expect("Git output")
    };
    run(&["init", "-b", "main"], None);
    run(&["commit", "--allow-empty", "-m", "base"], None);
    run(&["branch", "other"], None);
    run(&["commit", "--allow-empty", "-m", "local"], None);
    let local = run(&["rev-parse", "HEAD"], None);
    run(&["checkout", "other"], None);
    run(&["commit", "--allow-empty", "-m", "remote"], None);
    run(&["checkout", "main"], None);
    for (autoedit, executes) in [(None, false), (Some("yes"), true), (Some("no"), false)] {
        run(&["reset", "--hard", local.trim()], None);
        if marker.exists() {
            std::fs::remove_file(&marker).expect("reset marker");
        }
        run(&["merge", "--no-autostash", "--ff", "other"], autoedit);
        assert_eq!(marker.exists(), executes, "autoedit={autoedit:?}");
    }
}
