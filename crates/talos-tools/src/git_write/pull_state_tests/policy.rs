//! Pull policy regression coverage.

use super::*;

#[cfg(test)]
#[test]
fn tracking_mapping_preserves_first_merge_and_positive_fetch_order() {
    let config = b"branch.Topic.remote\nother\0branch.Topic.merge\nrefs/heads/main\0branch.Topic.merge\nrefs/heads/second\0remote.other.fetch\n^refs/heads/main\0remote.other.fetch\n+refs/heads/*:refs/custom/*\0remote.other.fetch\nrefs/heads/main:refs/alternate/main\0";
    assert_eq!(
        tracking_reference(config, "Topic", "other")
            .expect("mapping")
            .as_deref(),
        Some("refs/custom/main")
    );
    assert!(
        tracking_reference(config, "Topic", "origin")
            .expect("different remote")
            .is_none()
    );
    assert!(
        tracking_reference(config, "topic", "other")
            .expect("case")
            .is_none()
    );
    assert_eq!(
        tracking_reference(
            b"branch.Topic.remote\n.\0branch.Topic.merge\nrefs/heads/main\0",
            "Topic",
            "."
        )
        .expect("local")
        .as_deref(),
        Some("refs/heads/main")
    );
    assert!(tracking_reference(b"branch.Topic.remote\nother\0branch.Topic.merge\nrefs/heads/main\0remote.other.fetch\nrefs/heads/**:refs/x/*\0", "Topic", "other").is_err());
}

#[cfg(test)]
#[test]
fn fetch_remote_preserves_branch_and_single_remote_defaults() {
    let config = b"remote.Up.stream.url\n/path\0branch.Topic.remote\n.\0branch.topic.remote\nother\0remote.pushdefault\npush-only\0";
    assert_eq!(
        fetch_remote(config, Some("Topic"), None).expect("local"),
        "."
    );
    assert_eq!(
        fetch_remote(config, Some("topic"), None).expect("branch"),
        "other"
    );
    assert_eq!(fetch_remote(config, None, None).expect("sole"), "Up.stream");
    assert_eq!(
        fetch_remote(config, Some("Topic"), Some("override")).expect("explicit"),
        "override"
    );
    let mut multiple = config.to_vec();
    multiple.extend_from_slice(b"remote.second.fetch\n+refs/heads/*:refs/remotes/second/*\0");
    assert_eq!(
        fetch_remote(&multiple, None, None).expect("fallback"),
        "origin"
    );
    assert!(fetch_remote(b"branch.Topic.remote\0", Some("Topic"), None).is_err());
}

#[cfg(test)]
#[test]
fn pull_policy_preserves_branch_override_and_autostash_precedence() {
    let config = b"pull.rebase\ntrue\0branch.Topic.rebase\nfalse\0branch.topic.rebase\nmerges\0pull.ff\nonly\0rebase.autostash\ntrue\0merge.autostash\nfalse\0";
    let topic = Policy::capture(config, Some("Topic")).expect("branch policy");
    assert_eq!(topic.rebase, Rebase::Off);
    assert!(!topic.autostash);
    assert_eq!(topic.fast_forward, FastForward::Only);
    let lower = Policy::capture(config, Some("topic")).expect("case-sensitive branch");
    assert_eq!(lower.rebase, Rebase::Merges);
    assert!(lower.autostash);
    let detached = Policy::capture(config, None).expect("detached policy");
    assert_eq!(detached.rebase, Rebase::On);
    assert!(detached.autostash);
    let mut override_config = config.to_vec();
    override_config.extend_from_slice(b"pull.autostash\nfalse\0");
    assert!(
        !Policy::capture(&override_config, None)
            .expect("override")
            .autostash
    );
    let default = Policy::capture(b"", None).expect("defaults");
    assert!(default.rebase_unspecified);
    assert_eq!(default.fast_forward, FastForward::Unspecified);
    assert!(!default.autostash);
    for invalid in [
        b"pull.rebase\npreserve\0".as_slice(),
        b"pull.ff\ninvalid\0",
        b"pull.autostash\ninvalid\0",
    ] {
        assert!(Policy::capture(invalid, None).is_err());
    }
}

#[cfg(test)]
#[test]
fn fetched_heads_preserve_merge_order_and_ignore_description_bytes() {
    for width in [40, 64] {
        let first = "a".repeat(width);
        let second = "b".repeat(width);
        let ignored = "c".repeat(width);
        let mut bytes =
            format!("{first}\t\tfirst\n{ignored}\tnot-for-merge\tignored\n{second}\t\t")
                .into_bytes();
        bytes.extend_from_slice(b"non-utf8 \xff\tcomment\n");
        assert_eq!(
            merge_candidates(&bytes, width).expect("heads"),
            [first.clone(), second.clone()]
        );
        assert_eq!(
            fetched_tips(&bytes, width, false).expect("all fetched tips"),
            [first, ignored, second]
        );
    }
    assert!(merge_candidates(b"", 40).expect("empty").is_empty());
}

#[cfg(test)]
#[test]
fn pull_graph_listings_accept_only_complete_object_ids() {
    for width in [40, 64] {
        let oid = "a".repeat(width);
        let newer = "b".repeat(width);
        assert_eq!(
            graph_oids(format!("{newer}\n{oid}\n{newer}\n").as_bytes(), width).expect("walk order"),
            [newer, oid.clone()]
        );
        assert_eq!(
            graph_oids(format!("{oid}\n{}\n", oid.to_uppercase()).as_bytes(), width)
                .expect("normalized tips"),
            std::slice::from_ref(&oid)
        );
        for bad in [
            oid.clone(),
            format!("{}\n", "0".repeat(width)),
            format!("{}\n", "g".repeat(width)),
            format!("{oid}\n--all\n"),
            format!("{oid}\0\n"),
        ] {
            assert!(graph_oids(bad.as_bytes(), width).is_err());
        }
    }
    assert!(graph_oids(b"", 40).expect("empty refs").is_empty());
    assert!(graph_oids(b"", 32).is_err());
    assert!(graph_oids(&vec![b'a'; 1024 * 1024 + 1], 40).is_err());
}

#[cfg(test)]
#[test]
fn real_fetch_selects_upstream_only_from_all_fetched_branches() {
    let source = tempfile::tempdir().expect("source");
    let destination = tempfile::tempdir().expect("destination");
    let git = |cwd: &std::path::Path, args: &[&str]| {
        let output = std::process::Command::new("git")
            .current_dir(cwd)
            .args(args)
            .output()
            .expect("fixture Git");
        assert!(output.status.success(), "{output:?}");
        String::from_utf8(output.stdout).expect("Git output")
    };
    git(source.path(), &["init", "-b", "upstream"]);
    git(
        source.path(),
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
            "base",
        ],
    );
    let expected = git(source.path(), &["rev-parse", "HEAD"]).trim().to_owned();
    git(source.path(), &["branch", "unrelated"]);
    git(destination.path(), &["init", "-b", "local"]);
    git(
        destination.path(),
        &[
            "remote",
            "add",
            "origin",
            source.path().to_str().expect("source path"),
        ],
    );
    git(
        destination.path(),
        &["config", "branch.local.remote", "origin"],
    );
    git(
        destination.path(),
        &["config", "branch.local.merge", "refs/heads/upstream"],
    );
    git(
        destination.path(),
        &["-c", "maintenance.auto=false", "fetch", "origin"],
    );
    let evidence =
        std::fs::read(destination.path().join(".git/FETCH_HEAD")).expect("fetch evidence");
    assert!(
        evidence
            .windows(b"not-for-merge".len())
            .any(|bytes| bytes == b"not-for-merge")
    );
    assert_eq!(
        merge_candidates(&evidence, expected.len()).expect("upstream candidate"),
        [expected]
    );
}
