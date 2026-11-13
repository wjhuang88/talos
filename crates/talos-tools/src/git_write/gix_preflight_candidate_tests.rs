//! I324/T1a: characterize pure gix candidate primitives without changing
//! the accepted ADR-089 host-Git preflight execution boundary.
//!
//! No candidate in this file is called by production code. Native equivalence,
//! source installation defaults and authority review remain outstanding.

use super::target_config;
use gix::bstr::ByteSlice;
use std::collections::BTreeMap;
use std::ffi::OsString;

#[test]
fn gix_branch_name_validation_requires_no_git_child() {
    // The gix-validate API takes a *complete* Git ref, not a short
    // checkout branch name: `branch_name("topic")` correctly fails.
    // Map the user-facing spelling into refs/heads/ before validating.
    // This still isn't equivalent to Git's `check-ref-format --branch`:
    // leading '-' and reflog shorthand need their own lexical/semantic gates.
    assert!(gix::validate::reference::branch_name(b"topic".as_bstr()).is_err());

    for name in ["topic", "feature/topic", "release-1.2"] {
        let full_ref = format!("refs/heads/{name}");
        assert!(
            gix::validate::reference::branch_name(full_ref.as_bytes().as_bstr()).is_ok(),
            "ordinary branch should be valid: {name}"
        );
    }

    for name in [
        "",
        "topic..bad",
        "topic.lock",
        "bad name",
        "topic/",
        "topic@{bad",
        "HEAD",
    ] {
        let full_ref = format!("refs/heads/{name}");
        assert!(
            gix::validate::reference::branch_name(full_ref.as_bytes().as_bstr()).is_err(),
            "invalid or reserved branch must be refused: {name}"
        );
    }
}

#[test]
fn gix_config_include_frontier_is_lexical_and_preserves_missing_files() {
    // The origin need not exist: discovery must not read or stat an
    // unadmitted include target merely to find its name.
    let origin = std::env::temp_dir()
        .join("talos-i324-no-io-preflight-test")
        .join("config");
    let environment = BTreeMap::<OsString, OsString>::new();
    let bytes = b"[include]\n    path = never-created.conf\n[core]\n    bare = false\n";
    let parsed = target_config::parse_records(bytes).expect("gix config byte parsing");
    assert!(parsed.iter().any(|record| record.key == b"include.path"));
    let named = target_config::declared_paths(bytes, &origin, &environment, None)
        .expect("pure gix-backed include path analysis");
    assert_eq!(
        named,
        vec![origin.parent().expect("parent").join("never-created.conf")]
    );
}

#[test]
fn staged_config_roots_start_as_path_only_requests() {
    // Existing target_config::Sources is the approved exact Read frontier to
    // reuse. Root registration only names paths/ancestors; content capture
    // remains locked behind Sources::advance(admitted, environment).
    let root = std::env::temp_dir().join("talos-i324-frontier-not-read");
    let missing_system = root.join("system-config");
    let missing_global = root.join("global-config");
    let roots = [missing_system.clone(), missing_global.clone()];
    let sources = target_config::Sources::new(None, roots).expect("path-only source registration");
    assert!(
        !sources.complete(),
        "unread/absent candidates must not be silently elided"
    );
    let requests = sources.paths();
    assert!(requests.contains(&missing_system));
    assert!(requests.contains(&missing_global));
    assert!(sources.captured_paths().next().is_none());
}

#[test]
fn gix_admitted_config_graph_can_emit_host_records_without_git() {
    // The test fixture acts as a separately admitted source; neither
    // gix nor the adapter chooses roots, reads arbitrary files or runs Git.
    let dir = tempfile::tempdir().expect("fixture");
    let fixture = dir.path().canonicalize().expect("canonical fixture");
    let root = fixture.join("root.conf");
    let included = fixture.join("included.conf");
    std::fs::write(
        &root,
        b"[core]\n    bare = false\n[include]\n    path = included.conf\n",
    )
    .expect("root fixture");
    std::fs::write(&included, b"[user]\n    name = Test User\n").expect("include fixture");

    let env = BTreeMap::<OsString, OsString>::new();
    let mut sources = target_config::Sources::new(None, [root.clone()])
        .expect("path-only initial plan");
    for _ in 0..3 {
        if sources.complete() {
            break;
        }
        let admitted = sources.paths();
        sources
            .advance(|path| admitted.contains(path), &env)
            .expect("test-only exact path admission");
    }
    assert!(sources.complete(), "every declared include must be admitted");
    let serialized = sources
        .ordered_host_output(&[root], &env, None)
        .expect("pure gix-backed serialization");
    let parsed = super::config_state::parse_host(&serialized).expect("existing host parser");
    assert_eq!(
        parsed.get(b"core.bare".as_slice()).and_then(|values| values.last()),
        Some(&Some(b"false".to_vec()))
    );
    assert_eq!(
        parsed.get(b"user.name".as_slice()).and_then(|values| values.last()),
        Some(&Some(b"Test User".to_vec()))
    );
    let origin = format!("file:{}", included.display());
    assert!(
        serialized.windows(origin.len()).any(|bytes| bytes == origin.as_bytes()),
        "included records must retain their physical source"
    );
}

#[test]
fn gix_preflight_refuses_unadmitted_config_sources() {
    let dir = tempfile::tempdir().expect("fixture");
    let fixture = dir.path().canonicalize().expect("canonical fixture");
    let root = fixture.join("not-admitted.conf");
    let env = BTreeMap::<OsString, OsString>::new();
    let sources = target_config::Sources::new(None, [root.clone()]).expect("source plan");
    assert!(
        sources.ordered_host_output(&[root], &env, None).is_err(),
        "pure adapter cannot turn incomplete frontiers into evidence"
    );
}
