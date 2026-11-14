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
fn gix_short_branch_validation_matches_native_git_fixture_matrix() {
    // The Git subprocess is a test oracle ONLY: no CLI discovery is added to
    // production. Native --branch mode rejects an initial '-'; validation
    // of a full gix ref alone does not enforce that extra lexical rule.
    // Reflog shorthand such as @{-1} is context-dependent and belongs to
    // a separate admitted semantic resolution gate, not this lexical test.
    for short in [
        "topic",
        "feature/topic",
        "release-1.2",
        "@",
        "topic/-bad",
        "topic./child",
        "foo.lock/child",
        "",
        "HEAD",
        "-bad",
        ".hidden",
        "topic/.hidden",
        "topic/",
        "topic.",
        "topic.lock",
        "topic/child.lock",
        "topic..bad",
        "topic//bad",
        "bad name",
        "bad\tname",
        "topic\\bad",
        "topic:bad",
        "topic^bad",
        "topic~bad",
        "topic?bad",
        "topic*bad",
        "topic[bad",
        "topic@{bad",
    ] {
        let native = std::process::Command::new("git")
            .args(["check-ref-format", "--branch", short])
            .output()
            .expect("installed native Git fixture oracle");
        let full_ref = format!("refs/heads/{short}");
        let candidate = !short.starts_with('-')
            && gix::validate::reference::branch_name(full_ref.as_bytes().as_bstr()).is_ok();
        assert_eq!(
            candidate,
            native.status.success(),
            "gix branch lexical parity differs for {short:?}"
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
    let mut sources =
        target_config::Sources::new(None, [root.clone()]).expect("path-only initial plan");
    for _ in 0..3 {
        if sources.complete() {
            break;
        }
        let admitted = sources.paths();
        sources
            .advance(|path| admitted.contains(path), &env)
            .expect("test-only exact path admission");
    }
    assert!(
        sources.complete(),
        "every declared include must be admitted"
    );
    let serialized = sources
        .ordered_host_output(&[root], &env, None)
        .expect("pure gix-backed serialization");
    let parsed = super::config_state::parse_host(&serialized).expect("existing host parser");
    assert_eq!(
        parsed
            .get(b"core.bare".as_slice())
            .and_then(|values| values.last()),
        Some(&Some(b"false".to_vec()))
    );
    assert_eq!(
        parsed
            .get(b"user.name".as_slice())
            .and_then(|values| values.last()),
        Some(&Some(b"Test User".to_vec()))
    );
    let origin = format!("file:{}", included.display());
    assert!(
        serialized
            .windows(origin.len())
            .any(|bytes| bytes == origin.as_bytes()),
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

#[test]
fn gix_admitted_host_output_matches_native_config_values_and_stales() {
    // Git CLI is a fixture oracle ONLY. Runtime preflight must not call it.
    // `--file` selects exactly the fixture source, not ambient installation
    // roots. Compare parsed values, not textual origins: Git for Windows may
    // quote/backslash physical paths differently.
    let dir = tempfile::tempdir().expect("fixture");
    let fixture = {
        #[cfg(windows)]
        {
            // Git CLI does not reliably accept canonical Windows \\?\ paths.
            dir.path().to_path_buf()
        }
        #[cfg(not(windows))]
        {
            dir.path().canonicalize().expect("canonical fixture")
        }
    };
    let root = fixture.join("root.conf");
    let included = fixture.join("child.conf");
    std::fs::write(
        &root,
        b"[fixture]\n marker=before\n[include]\n path=child.conf\n[fixture]\n marker=after\n",
    )
    .expect("root fixture");
    std::fs::write(&included, b"[fixture]\n marker=child\n implicit\n empty=\n")
        .expect("child fixture");

    let native = std::process::Command::new("git")
        .args(["config", "--file"])
        .arg(&root)
        .args(["--show-origin", "--null", "--list", "--includes"])
        .output()
        .expect("installed Git fixture oracle");
    assert!(native.status.success(), "Git fixture oracle failed");

    let env = BTreeMap::<OsString, OsString>::new();
    let mut sources = target_config::Sources::new(None, [root.clone()]).expect("root path");
    for _ in 0..4 {
        if sources.complete() {
            break;
        }
        let admitted = sources.paths();
        sources
            .advance(|path| admitted.contains(path), &env)
            .expect("explicit fixture Read admission");
    }
    assert!(
        sources.complete(),
        "include frontier should be fully admitted"
    );
    let actual = sources
        .ordered_host_output(std::slice::from_ref(&root), &env, None)
        .expect("serialize only admitted gix records");
    let expected_values =
        super::config_state::parse_host(&native.stdout).expect("native host records");
    let actual_values = super::config_state::parse_host(&actual).expect("gix host records");
    assert_eq!(
        actual_values, expected_values,
        "configuration values diverged"
    );

    // Mutating an included source after the snapshot must fail revalidation.
    std::fs::write(&included, b"[fixture]\n marker=changed\n").expect("change included source");
    assert!(sources.revalidate().is_err(), "stale config was accepted");
}

#[test]
fn gix_host_candidate_retains_missing_root_and_rejects_later_appearance() {
    // An absent, separately admitted system root is still security-relevant.
    // It cannot disappear from evidence just because a later global root has
    // usable values. If the missing root appears after review, refuse.
    let dir = tempfile::tempdir().expect("fixture");
    let fixture = dir.path().canonicalize().expect("canonical fixture");
    let missing_system = fixture.join("system.conf");
    let present_global = fixture.join("global.conf");
    std::fs::write(&present_global, b"[fixture]\n marker=global\n").expect("global fixture");

    let env = BTreeMap::<OsString, OsString>::new();
    let roots = [missing_system.clone(), present_global.clone()];
    let mut sources = target_config::Sources::new(None, roots.clone()).expect("path-only roots");
    let admitted = sources.paths();
    sources
        .advance(|path| admitted.contains(path), &env)
        .expect("all missing and present roots admitted");
    assert!(sources.complete(), "no unread roots remain");
    assert!(
        sources.captured_paths().any(|path| path == &missing_system),
        "negative root identity must survive capture"
    );

    let output = sources
        .ordered_host_output(&roots, &env, None)
        .expect("serialize admitted root graph only");
    let records = super::config_state::parse_host(&output).expect("host records");
    assert_eq!(
        records
            .get(b"fixture.marker".as_slice())
            .and_then(|values| values.last()),
        Some(&Some(b"global".to_vec()))
    );
    std::fs::write(&missing_system, b"[fixture]\n marker=system\n")
        .expect("root appears after admission");
    assert!(
        sources.revalidate().is_err(),
        "a previously absent system root appeared after approval"
    );
}

#[test]
fn gix_attributes_strict_parse_refuses_malformed_filter_sources() {
    // The locked gix-attributes parser operates solely on admitted bytes.
    // A future filter-effect resolver must not use Search's lenient parsing
    // alone: invalid records may be discarded. Parse every line and
    // assignment with an explicit error branch before any effect inference.
    let strict = |bytes: &[u8]| {
        gix_attributes::parse(bytes).all(|line| {
            let Ok((_, attributes, _)) = line else {
                return false;
            };
            attributes.into_iter().all(|attribute| attribute.is_ok())
        })
    };
    assert!(strict(
        b"[attr]binary -diff -merge -text\n*.bin binary\n*.txt -filter\n"
    ));
    assert!(
        !strict(b"*.bin filter=lfs\n!*.bad filter=untrusted\n"),
        "negative attribute patterns must not be silently skipped"
    );
    assert!(
        !strict(b"*.bin filter=lfs invalid$name\n"),
        "invalid assignment names must not be silently skipped"
    );
}

#[test]
fn gix_config_frontier_denial_prevents_all_content_capture() {
    let dir = tempfile::tempdir().expect("fixture");
    let root = dir.path().join("denied.conf");
    std::fs::write(&root, b"[fixture]\n marker=secret\n").expect("fixture");
    let mut sources = target_config::Sources::new(None, [root]).expect("path only");
    assert!(
        sources
            .advance(|_| false, &BTreeMap::<OsString, OsString>::new())
            .is_err(),
        "denied paths must not be read"
    );
    assert!(
        sources.captured_paths().next().is_none(),
        "denied data must never enter a prepared source snapshot"
    );
}
