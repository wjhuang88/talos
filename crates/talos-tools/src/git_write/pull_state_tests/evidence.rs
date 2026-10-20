//! Pull evidence regression coverage.

use super::*;

#[cfg(test)]
#[test]
fn pure_index_decodes_both_hash_formats_without_worktree_files() {
    for (format, length) in [("sha1", 40), ("sha256", 64)] {
        let root = tempfile::tempdir().expect("fixture");
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .current_dir(root.path())
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_GLOBAL", root.path().join("absent-config"))
                .args(args)
                .output()
                .expect("fixture Git");
            assert!(output.status.success(), "{output:?}");
        };
        git(&["init", &format!("--object-format={format}")]);
        std::fs::write(root.path().join("tracked"), b"content").expect("source");
        git(&["add", "tracked"]);
        std::fs::remove_file(root.path().join("tracked")).expect("remove worktree source");
        let (state, evidence) =
            decode_index(&root.path().join(".git/index"), length).expect("index");
        assert_eq!(state.entries().len(), 1);
        assert_eq!(state.entries()[0].path(&state), b"tracked");
        assert_eq!(state.entries()[0].id.to_string().len(), length);
        assert!(matches!(
            evidence.entry,
            super::super::snapshot::Entry::File(..)
        ));
        assert!(!root.path().join("tracked").exists());
        let path = root.path().join(".git/index");
        let mut bytes = std::fs::read(&path).expect("index bytes");
        let last = bytes.len() - 1;
        bytes[last] ^= 1;
        std::fs::write(&path, &bytes).expect("bad checksum");
        assert!(decode_index(&path, length).is_err());
        let trailer = bytes.len() - length / 2;
        bytes[trailer..].fill(0);
        std::fs::write(&path, &bytes).expect("skipHash checksum");
        assert!(decode_index(&path, length).is_ok());
        std::fs::write(root.path().join(".git/index"), b"truncated").expect("corrupt index");
        assert!(decode_index(&root.path().join(".git/index"), length).is_err());
    }
}

#[cfg(unix)]
#[cfg(test)]
#[test]
fn pull_sparse_patterns_are_exact_evidence_even_when_absent() {
    let root = tempfile::tempdir().expect("fixture");
    let output = std::process::Command::new("git")
        .arg("init")
        .arg(root.path())
        .output()
        .expect("Git fixture");
    assert!(output.status.success());
    let repository =
        super::super::repository_state::RepositoryState::capture(root.path()).expect("repository");
    let patterns = repository.git_dir.join("info/sparse-checkout");
    assert!(
        PullEvidence::required_paths(&repository, None, false)
            .expect("paths")
            .contains(&patterns)
    );
    let absent = PullEvidence::capture(&repository, None, false).expect("absent patterns");
    std::fs::write(&patterns, b"/*\n!/*/\n").expect("create patterns");
    assert!(absent.revalidate().is_err());
    let present = PullEvidence::capture(&repository, None, false).expect("present patterns");
    std::fs::write(&patterns, b"/*\n").expect("change patterns");
    assert!(present.revalidate().is_err());
    std::fs::remove_file(&patterns).expect("remove patterns");
    std::os::unix::fs::symlink(root.path().join("external"), &patterns).expect("redirect");
    assert!(PullEvidence::capture(&repository, None, false).is_err());
}

#[cfg(unix)]
#[cfg(test)]
#[test]
fn pull_evidence_refuses_reflog_escape_oversize_and_missing_leaf_creation() {
    let root = tempfile::tempdir().expect("fixture");
    let work = root.path().join("work");
    std::fs::create_dir(&work).expect("worktree");
    let output = std::process::Command::new("git")
        .args(["init", "-b", "Topic"])
        .current_dir(&work)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", root.path().join("absent-config"))
        .output()
        .expect("fixture Git");
    assert!(output.status.success());
    let repository =
        super::super::repository_state::RepositoryState::capture(&work).expect("repository");
    let clean = PullEvidence::capture(&repository, None, false).expect("clean merge state");
    clean.require_no_pending_merge().expect("no pending merge");
    let index_path = repository.git_dir.join("index");
    std::fs::write(work.join("tracked"), b"timestamp evidence").expect("index source");
    let add = std::process::Command::new("git")
        .args(["add", "tracked"])
        .current_dir(&work)
        .output()
        .expect("index fixture");
    assert!(add.status.success());
    let indexed_repository =
        super::super::repository_state::RepositoryState::capture(&work).expect("index repository");
    let indexed = PullEvidence::capture(&indexed_repository, None, false).expect("index evidence");
    std::fs::File::options()
        .write(true)
        .open(&index_path)
        .expect("index handle")
        .set_modified(std::time::UNIX_EPOCH)
        .expect("change only timestamp");
    assert!(
        indexed.revalidate().is_err(),
        "index timestamp drift invalidates evidence"
    );
    std::fs::remove_file(&index_path).expect("remove index fixture");
    let repository = super::super::repository_state::RepositoryState::capture(&work)
        .expect("repository after index");
    std::fs::write(repository.git_dir.join("MERGE_HEAD"), b"").expect("pending merge marker");
    assert!(
        clean.revalidate().is_err(),
        "new merge invalidates admission evidence"
    );
    let pending = PullEvidence::capture(&repository, None, false).expect("pending merge evidence");
    assert!(
        pending.require_no_pending_merge().is_err(),
        "even an empty marker blocks fetch"
    );
    std::fs::remove_file(repository.git_dir.join("MERGE_HEAD")).expect("remove merge marker");
    assert!(PullEvidence::capture(&repository, Some("refs/../outside"), false).is_err());
    let parent = repository.git_dir.join("logs/refs/remotes");
    std::fs::create_dir_all(&parent).expect("logs parent");
    let outside = root.path().join("outside");
    std::fs::create_dir(&outside).expect("outside");
    std::os::unix::fs::symlink(&outside, parent.join("origin")).expect("escaped parent");
    assert!(PullEvidence::capture(&repository, Some("refs/remotes/origin/Topic"), false).is_err());
    std::fs::remove_file(parent.join("origin")).expect("remove fixture link");
    let internal = repository.git_dir.join("private");
    std::fs::create_dir(&internal).expect("internal target");
    std::os::unix::fs::symlink(&internal, parent.join("origin")).expect("internal parent link");
    assert!(
        PullEvidence::capture(&repository, Some("refs/remotes/origin/Topic"), false).is_err(),
        "root-internal retarget still needs its own Read admission"
    );
    std::fs::remove_file(parent.join("origin")).expect("remove internal fixture link");
    std::fs::create_dir(parent.join("origin")).expect("real parent");
    let before = PullEvidence::capture(&repository, Some("refs/remotes/origin/Topic"), false)
        .expect("missing leaf evidence");
    let path = parent.join("origin/Topic");
    std::fs::write(&path, "created after inspection").expect("create leaf");
    assert!(before.revalidate().is_err());
    std::fs::File::create(&path)
        .expect("large leaf")
        .set_len(16 * 1024 * 1024 + 1)
        .expect("oversize");
    assert!(PullEvidence::capture(&repository, Some("refs/remotes/origin/Topic"), false).is_err());
    std::fs::write(&path, "").expect("restore bounded leaf");
    let bound = PullEvidence::capture(&repository, Some("refs/remotes/origin/Topic"), false)
        .expect("bound location");
    std::fs::write(
        repository.git_dir.join("commondir"),
        outside.to_str().expect("outside path"),
    )
    .expect("redirect common root");
    let error = bound
        .revalidate()
        .expect_err("redirect must fail before inspecting new root");
    assert!(
        error.to_string().contains("location pointer changed"),
        "{error}"
    );
}

#[cfg(test)]
#[test]
fn postfetch_evidence_rebaselines_changed_metadata_only_under_retained_location() {
    let root = tempfile::tempdir().expect("fixture");
    let source = root.path().join("source");
    let local = root.path().join("local");
    let git = |cwd: &std::path::Path, args: &[&str]| {
        let output = std::process::Command::new("git")
            .current_dir(cwd)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", root.path().join("absent-config"))
            .args([
                "-c",
                "maintenance.autoDetach=false",
                "-c",
                "gc.autoDetach=false",
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
            ])
            .args(args)
            .output()
            .expect("fixture Git");
        assert!(output.status.success(), "{args:?}: {output:?}");
        output.stdout
    };
    std::fs::create_dir(&source).expect("source");
    git(&source, &["init", "-b", "main"]);
    std::fs::write(source.join("file"), b"one\n").expect("source file");
    git(&source, &["add", "file"]);
    git(&source, &["commit", "-m", "one"]);
    git(
        root.path(),
        &[
            "clone",
            source.to_str().expect("source"),
            local.to_str().expect("local"),
        ],
    );
    let original =
        super::super::repository_state::RepositoryState::capture(&local).expect("before");
    let prefetch = PullEvidence::capture(&original, Some("refs/remotes/origin/main"), false)
        .expect("prefetch evidence");
    std::fs::write(source.join("file"), b"two\n").expect("source change");
    git(&source, &["add", "file"]);
    git(&source, &["commit", "-m", "two"]);
    let expected = git(&source, &["rev-parse", "HEAD"]);
    git(&local, &["fetch", "--no-recurse-submodules", "origin"]);
    original
        .revalidate_location()
        .expect("old location retained");
    assert_ne!(
        original,
        super::super::repository_state::RepositoryState::capture(&local).expect("changed graph")
    );
    assert!(
        prefetch.revalidate().is_err(),
        "old content evidence is stale"
    );
    let fetched = PullEvidence::capture(&original, Some("refs/remotes/origin/main"), true)
        .expect("new content under old roots");
    assert_eq!(
        merge_candidates(fetched.fetch_head().expect("FETCH_HEAD"), 40).expect("candidate"),
        [String::from_utf8(expected).expect("OID").trim().to_owned()]
    );
    fetched.revalidate().expect("stable post-fetch evidence");
    let path = original.git_dir.join("FETCH_HEAD");
    let saved = std::fs::read(&path).expect("fetched bytes");
    std::fs::write(&path, b"replacement\n").expect("drift");
    assert!(fetched.revalidate().is_err());
    std::fs::write(&path, saved).expect("restore");
    let redirected = root.path().join("unapproved");
    std::fs::create_dir(&redirected).expect("external root");
    std::fs::write(
        original.git_dir.join("commondir"),
        redirected.to_str().expect("path"),
    )
    .expect("redirect pointer");
    let error = PullEvidence::capture(&original, None, true)
        .expect_err("old admission must not read the new location");
    assert!(
        error.to_string().contains("location pointer changed"),
        "{error}"
    );
    std::fs::remove_file(original.git_dir.join("commondir")).expect("restore pointer");
    // Content evidence retains Windows leaf handles. Release the completed
    // content probes before exercising the independent directory identity
    // guard; keep `original` and all of its location handles alive.
    drop(prefetch);
    drop(fetched);
    std::fs::rename(&original.git_dir, local.join("saved-gitdir")).expect("retain original");
    std::fs::create_dir(&original.git_dir).expect("replace directory");
    let error = PullEvidence::capture(&original, None, true)
        .expect_err("replacement identity must fail before repository rediscovery");
    assert!(
        error.to_string().contains("root identity changed"),
        "{error}"
    );
}

#[cfg(test)]
#[test]
fn prefetch_evidence_distinguishes_detached_unborn_and_rejects_invalid_probes() {
    let oid = format!("{}\n", "a".repeat(40));
    let detached =
        BeforeFetch::capture(b"", None, Some(oid.as_bytes()), None, 40).expect("detached");
    assert!(detached.branch.is_none() && detached.head.is_some());
    let unborn =
        BeforeFetch::capture(b"", Some(b"refs/heads/Topic\n"), None, None, 40).expect("unborn");
    assert_eq!(unborn.branch.as_deref(), Some("Topic"));
    assert!(unborn.head.is_none());
    let born = BeforeFetch::capture(
        b"",
        Some(b"refs/heads/Topic\n"),
        Some(oid.as_bytes()),
        None,
        40,
    )
    .expect("born branch");
    assert!(!unborn.same_checkout(&born));
    assert!(!born.same_checkout(&detached));
    let changed_oid = format!("{}\n", "b".repeat(40));
    let moved = BeforeFetch::capture(
        b"",
        Some(b"refs/heads/Topic\n"),
        Some(changed_oid.as_bytes()),
        None,
        40,
    )
    .expect("moved branch");
    assert!(!born.same_checkout(&moved));
    let with_fork = BeforeFetch::capture(
        b"",
        Some(b"refs/heads/Topic\n"),
        Some(oid.as_bytes()),
        Some(changed_oid.as_bytes()),
        40,
    )
    .expect("fork point discovered");
    assert!(born.same_checkout(&with_fork));
    assert!(
        BeforeFetch::capture(b"", None, Some(oid.as_bytes()), Some(oid.as_bytes()), 40).is_err()
    );
    for bad in [
        b"refs/tags/tag\n".as_slice(),
        b"refs/heads/\n",
        b"refs/heads/Topic\nextra\n",
    ] {
        assert!(BeforeFetch::capture(b"", Some(bad), None, None, 40).is_err());
    }
    for bad in ["0".repeat(40), "g".repeat(40), "a".repeat(39)] {
        assert!(
            BeforeFetch::capture(b"", None, Some(format!("{bad}\n").as_bytes()), None, 40).is_err()
        );
    }
}

#[cfg(test)]
#[test]
fn malformed_fetch_evidence_cannot_become_an_approved_target() {
    let oid = "a".repeat(40);
    for bytes in [
        format!("{oid}\t\tunterminated"),
        format!("{oid}\tunknown\tdescription\n"),
        format!("{oid}\t\n"),
        format!("{}\t\tzero\n", "0".repeat(40)),
        format!("{}\t\tinvalid\n", "g".repeat(40)),
    ] {
        assert!(merge_candidates(bytes.as_bytes(), 40).is_err());
    }
    assert!(merge_candidates(&vec![b'x'; 4 * 1024 * 1024 + 1], 40).is_err());
    assert!(merge_candidates(format!("{oid}\t\tline\n").repeat(65_537).as_bytes(), 40).is_err());
}

#[cfg(test)]
#[cfg(unix)]
#[test]
fn pull_prefetch_binds_fetch_head_destination_before_mutation() {
    let root = tempfile::tempdir().expect("fixture");
    let work = root.path().join("work");
    std::fs::create_dir(&work).expect("worktree");
    let output = std::process::Command::new("git")
        .args(["init", "-b", "Topic"])
        .current_dir(&work)
        .output()
        .expect("fixture Git");
    assert!(output.status.success());
    let repository =
        super::super::repository_state::RepositoryState::capture(&work).expect("repository");
    let path = repository.git_dir.join("FETCH_HEAD");
    let absent = PullEvidence::capture(&repository, None, false).expect("absent destination");
    std::fs::write(&path, b"previous fetch\n").expect("create destination");
    assert!(absent.revalidate().is_err());
    let present = PullEvidence::capture(&repository, None, false).expect("regular destination");
    present.revalidate().expect("unchanged destination");
    std::fs::write(&path, b"changed fetch\n").expect("change destination");
    assert!(present.revalidate().is_err());
    std::fs::remove_file(&path).expect("remove destination");
    let outside = root.path().join("sentinel");
    std::fs::write(&outside, b"preserved\n").expect("sentinel");
    std::os::unix::fs::symlink(&outside, &path).expect("redirect destination");
    assert!(PullEvidence::capture(&repository, None, false).is_err());
    assert!(absent.revalidate().is_err());
    assert_eq!(
        std::fs::read(outside).expect("sentinel retained"),
        b"preserved\n"
    );
}
