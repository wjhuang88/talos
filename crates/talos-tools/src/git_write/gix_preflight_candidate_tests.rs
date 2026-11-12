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
    // The API is a pure, in-memory validation primitive. It is only a
    // candidate to replace `git check-ref-format --branch`: native Git may
    // first expand `@{-n}` using reflog history, which must not be guessed.
    for name in ["topic", "feature/topic", "release-1.2"] {
        assert!(
            gix::validate::reference::branch_name(name.as_bytes().as_bstr()).is_ok(),
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
    ] {
        assert!(
            gix::validate::reference::branch_name(name.as_bytes().as_bstr()).is_err(),
            "unsupported branch must not be admitted: {name}"
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

