//! Pull reconcile regression coverage.

use super::*;

#[cfg(test)]
#[tokio::test]
async fn real_pull_current_branch_fetch_reconciles_index_and_worktree() {
    for scenario in ["clean", "disjoint-edit", "conflicting-edit"] {
        let root = tempfile::tempdir().expect("fixture");
        let source = root.path().join("source");
        let native = root.path().join("native");
        let split = root.path().join("split");
        std::fs::create_dir(&source).expect("source");
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
        std::fs::write(source.join("changed"), "base\n").expect("base");
        std::fs::write(source.join("preserved"), "base\n").expect("base");
        git(&source, &["add", "."]);
        git(&source, &["commit", "-m", "base"]);
        for local in [&native, &split] {
            git(
                root.path(),
                &[
                    "clone",
                    source.to_str().expect("source"),
                    local.to_str().expect("local"),
                ],
            );
            git(
                local,
                &[
                    "config",
                    "remote.origin.fetch",
                    "+refs/heads/Topic:refs/heads/Topic",
                ],
            );
            git(local, &["config", "pull.rebase", "false"]);
            if scenario != "clean" {
                let path = if scenario == "disjoint-edit" {
                    "preserved"
                } else {
                    "changed"
                };
                std::fs::write(local.join(path), "local edit\n").expect("dirty edit");
            }
        }
        let original = git(&split, &["rev-parse", "HEAD"]);
        std::fs::write(source.join("changed"), "remote edit\n").expect("remote");
        git(&source, &["add", "."]);
        git(&source, &["commit", "-m", "remote"]);
        let expected = git(&source, &["rev-parse", "HEAD"]);
        let retained = BeforeFetch::capture(
            b"pull.rebase\nfalse\0",
            Some(b"refs/heads/Topic\n"),
            Some(original.as_bytes()),
            None,
            40,
        )
        .expect("before fetch");
        let native_result = run(&native, &["pull", "origin"]);
        git(&split, &["fetch", "--update-head-ok", "origin"]);
        assert_eq!(git(&split, &["rev-parse", "HEAD"]), expected);
        assert_eq!(
            std::fs::read(split.join("changed")).expect("pre-reconciliation"),
            if scenario == "conflicting-edit" {
                b"local edit\n".as_slice()
            } else {
                b"base\n".as_slice()
            }
        );
        let fetched = std::fs::read(split.join(".git/FETCH_HEAD")).expect("FETCH_HEAD");
        // Native pull reconciles a moved checked-out HEAD before rejecting
        // missing merge candidates. Candidate parsing must not skip this step.
        let without_candidates = retained
            .integration(b"", 40, |args| {
                let output = run(&split, &args.iter().map(String::as_str).collect::<Vec<_>>());
                async move { Ok(output) }
            })
            .await
            .expect("reconciliation before candidate validation");
        assert!(matches!(without_candidates, Integration::Reconcile { .. }));
        let integration = retained
            .integration(&fetched, 40, |args| {
                let output = run(&split, &args.iter().map(String::as_str).collect::<Vec<_>>());
                async move { Ok(output) }
            })
            .await
            .expect("selected integration");
        let commands = integration.commands().expect("reconciliation commands");
        let Integration::Reconcile {
            original: old,
            current,
        } = integration
        else {
            panic!("missing explicit reconciliation");
        };
        assert_eq!(old, original.trim());
        assert_eq!(current, expected.trim());
        assert_eq!(commands.len(), 1);
        let reconciled = run(
            &split,
            &commands[0].iter().map(String::as_str).collect::<Vec<_>>(),
        );
        let split_success = if reconciled.status.success() {
            // A completed read-tree is not permission to accept another HEAD
            // movement while selecting the next independently approved step.
            git(&split, &["update-ref", "HEAD", original.trim()]);
            let drift = retained
                .integration_stage(&fetched, 40, Some(&current), |args| {
                    let output = run(&split, &args.iter().map(String::as_str).collect::<Vec<_>>());
                    async move { Ok(output) }
                })
                .await
                .expect_err("reconciled HEAD must remain bound");
            assert!(
                drift
                    .to_string()
                    .contains("HEAD changed after worktree reconciliation")
            );
            git(&split, &["update-ref", "HEAD", &current]);
            let next = retained
                .integration_stage(&fetched, 40, Some(&current), |args| {
                    let output = run(&split, &args.iter().map(String::as_str).collect::<Vec<_>>());
                    async move { Ok(output) }
                })
                .await
                .expect("integration after reconciliation");
            let commands = next.commands().expect("expected merge");
            assert_eq!(commands.len(), 1);
            let args = &commands[0];
            run(&split, &args.iter().map(String::as_str).collect::<Vec<_>>())
                .status
                .success()
        } else {
            false
        };
        assert_eq!(split_success, native_result.status.success(), "{scenario}");
        assert_eq!(split_success, scenario != "conflicting-edit");
        assert_eq!(git(&native, &["rev-parse", "HEAD"]), expected);
        for path in ["changed", "preserved"] {
            assert_eq!(
                std::fs::read(native.join(path)).expect("native bytes"),
                std::fs::read(split.join(path)).expect("split bytes"),
                "{scenario} {path}"
            );
        }
        for args in [
            vec!["ls-files", "--stage"],
            vec!["status", "--porcelain=v1"],
            vec!["diff", "--binary"],
            vec!["diff", "--cached", "--binary"],
        ] {
            assert_eq!(
                git(&native, &args),
                git(&split, &args),
                "{scenario} {args:?}"
            );
        }
    }
}
