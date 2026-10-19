//! Pull unborn regression coverage.

use super::*;

#[cfg(test)]
#[tokio::test]
async fn real_pull_unborn_preserves_local_index_and_worktree() {
    for scenario in ["clean", "staged-disjoint", "untracked-conflict"] {
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
        git(&source, &["init", "-b", "Topic"]);
        std::fs::write(source.join("remote"), "remote content\n").expect("remote file");
        git(&source, &["add", "."]);
        git(&source, &["commit", "-m", "initial"]);
        for local in [&native, &split] {
            std::fs::create_dir(local).expect("local");
            git(local, &["init", "-b", "Topic"]);
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
            40,
        )
        .expect("unborn state");
        let native_result = run(&native, &["pull", "origin", "Topic"]);
        git(&split, &["fetch", "--update-head-ok", "origin", "Topic"]);
        let fetched = std::fs::read(split.join(".git/FETCH_HEAD")).expect("FETCH_HEAD");
        let selected = retained
            .integration(&fetched, 40, |args| {
                let output = run(&split, &args.iter().map(String::as_str).collect::<Vec<_>>());
                async move { Ok(output) }
            })
            .await
            .expect("unborn selection");
        let Integration::Unborn(target) = selected else {
            panic!("expected unborn plan")
        };
        let empty = git(&split, &["hash-object", "-t", "tree", "--stdin"]);
        let checkout = run(&split, &["read-tree", "-u", "-m", empty.trim(), &target]);
        let split_success = checkout.status.success()
            && run(
                &split,
                &[
                    "update-ref",
                    "-m",
                    "initial pull",
                    "HEAD",
                    &target,
                    "0000000000000000000000000000000000000000",
                ],
            )
            .status
            .success();
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
