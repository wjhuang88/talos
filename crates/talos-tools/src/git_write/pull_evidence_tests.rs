//! Retained Pull metadata must reject parent and leaf identity drift.
use super::PullEvidence;

#[test]
fn missing_leaf_revalidation_binds_closest_parent_and_streams_regular_contents() {
    let root = tempfile::tempdir().expect("fixture");
    let output = std::process::Command::new("git")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", root.path().join("absent-config"))
        .args(["init", "--quiet"])
        .arg(root.path())
        .output()
        .expect("Git fixture");
    assert!(output.status.success(), "{output:?}");
    let repository =
        super::super::repository_state::RepositoryState::capture(root.path()).expect("repository");
    let tracking = "refs/remotes/origin/main";
    let absent = PullEvidence::capture(&repository, Some(tracking), false).expect("absent");
    absent.revalidate().expect("unchanged missing evidence");
    let parent = repository.git_dir.join("logs/refs/remotes/origin");
    std::fs::create_dir_all(&parent).expect("new parent");
    assert!(
        absent.revalidate().is_err(),
        "new parent must invalidate absent leaf"
    );
    let path = parent.join("main");
    let bytes = vec![b'x'; 8192 * 2 + 3];
    std::fs::write(&path, &bytes).expect("regular reflog");
    let present = PullEvidence::capture(&repository, Some(tracking), false).expect("present");
    present.revalidate().expect("unchanged multi-chunk body");
    let mut changed = bytes.clone();
    changed[8192 + 1] = b'y';
    std::fs::write(&path, changed).expect("changed middle chunk");
    assert!(present.revalidate().is_err());
    std::fs::write(&path, &bytes).expect("restore same inode");
    present.revalidate().expect("restored body");
    let retained = parent.join("retained");
    std::fs::rename(&path, &retained).expect("retain old inode");
    std::fs::write(&path, &bytes).expect("identical replacement");
    assert!(
        present.revalidate().is_err(),
        "identical replacement is drift"
    );
}
