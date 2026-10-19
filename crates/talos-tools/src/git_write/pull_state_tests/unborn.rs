//! Pull unborn regression coverage.

use super::*;

#[cfg(test)]
#[tokio::test]
async fn real_pull_unborn_preserves_local_index_and_worktree() {
    for (scenario, object_format, oid_length) in [
        ("clean", "--object-format=sha1", 40),
        ("staged-disjoint", "--object-format=sha1", 40),
        ("untracked-conflict", "--object-format=sha1", 40),
        ("clean", "--object-format=sha256", 64),
        ("staged-disjoint", "--object-format=sha256", 64),
        ("untracked-conflict", "--object-format=sha256", 64),
    ] {
        let root = tempfile::tempdir().expect("fixture");
        let source = root.path().join("source");
        let native = root.path().join("native");
        let split = root.path().join("split");
        let run = |cwd: &std::path::Path, args: &[&str]| {
            std::process::Command::new("git")
                .current_dir(cwd)
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_GLOBAL", root.path().join("absent-config"))
                .env("GIT_TERMINAL_PROMPT", "0")
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
        std::fs::create_dir(&source).expect("source");
        git(&source, &["init", object_format, "-b", "Topic"]);
        std::fs::write(source.join("remote"), "remote content\n").expect("remote file");
        git(&source, &["add", "."]);
        git(&source, &["commit", "-m", "initial"]);
        for local in [&native, &split] {
            std::fs::create_dir(local).expect("local");
            git(local, &["init", object_format, "-b", "Topic"]);
            git(
                local,
                &["remote", "add", "origin", source.to_str().expect("source")],
            );
            git(local, &["config", "pull.rebase", "false"]);
            match scenario {
                "staged-disjoint" => {
                    std::fs::write(local.join("local"), "staged content\n").expect("local file");
                    git(local, &["add", "local"]);
                }
                "untracked-conflict" => {
                    std::fs::write(local.join("remote"), "local content\n").expect("conflict");
                }
                _ => {}
            }
        }
        let retained = BeforeFetch::capture(
            b"pull.rebase\nfalse\0",
            Some(b"refs/heads/Topic\n"),
            None,
            None,
            oid_length,
        )
        .expect("unborn state");
        let native_result = run(&native, &["pull", "origin", "Topic"]);
        git(&split, &["fetch", "--update-head-ok", "origin", "Topic"]);
        let fetched = std::fs::read(split.join(".git/FETCH_HEAD")).expect("FETCH_HEAD");
        let selected = retained
            .integration(&fetched, oid_length, |args| {
                let output = run(&split, &args.iter().map(String::as_str).collect::<Vec<_>>());
                async move { Ok(output) }
            })
            .await
            .expect("unborn selection");
        let Integration::Unborn(_) = &selected else {
            panic!("expected unborn plan")
        };
        let commands = selected.commands().expect("unborn commands");
        assert_eq!(commands.len(), 2);
        let split_success = commands.iter().all(|args| {
            run(&split, &args.iter().map(String::as_str).collect::<Vec<_>>())
                .status
                .success()
        });
        assert_eq!(split_success, native_result.status.success(), "{scenario}");
        assert_eq!(split_success, scenario != "untracked-conflict");
        for args in [
            vec!["rev-parse", "--verify", "--quiet", "HEAD"],
            vec!["ls-files", "--stage"],
            vec!["status", "--porcelain=v1"],
            vec!["diff", "--binary"],
            vec!["diff", "--cached", "--binary"],
        ] {
            let expected = run(&native, &args);
            let actual = run(&split, &args);
            assert_eq!(
                expected.status.code(),
                actual.status.code(),
                "{scenario} {args:?}"
            );
            assert_eq!(expected.stdout, actual.stdout, "{scenario} {args:?}");
        }
        for path in ["remote", "local"] {
            assert_eq!(
                std::fs::read(native.join(path)).ok(),
                std::fs::read(split.join(path)).ok(),
                "{scenario} {path}"
            );
        }
    }
}

#[test]
fn pull_integration_commands_refuse_invalid_or_mixed_object_identities() {
    for invalid in ["--help".to_owned(), "g".repeat(40), "1".repeat(39)] {
        assert!(Integration::Unborn(invalid).commands().is_err());
    }
    assert!(
        Integration::Reconcile {
            original: "1".repeat(40),
            current: "2".repeat(64),
        }
        .commands()
        .is_err()
    );
    assert!(
        Integration::Merge {
            heads: Vec::new(),
            fast_forward: FastForward::Allow,
            autostash: false,
        }
        .commands()
        .is_err()
    );
}
