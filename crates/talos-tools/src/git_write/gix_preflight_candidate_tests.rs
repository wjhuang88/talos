//! I324/T1a: characterize pure gix candidate primitives without changing
//! the accepted ADR-089 host-Git preflight execution boundary.
//!
//! No candidate in this file is called by production code. Native equivalence,
//! source installation defaults and authority review remain outstanding.

use super::target_config;
use gix::bstr::ByteSlice;
use std::collections::BTreeMap;
use std::ffi::OsString;

/// The Git --show-origin --null wire stream contains origin/record pairs,
/// whose cross-key order is security-relevant even when value maps match.
fn ordered_file_origin_records(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let payload = bytes.strip_suffix(b"\0").expect("terminated origin stream");
    let fields = payload.split(|byte| *byte == 0).collect::<Vec<_>>();
    assert_eq!(fields.len() % 2, 0, "each record needs a file origin");
    fields
        .chunks_exact(2)
        .map(|pair| {
            let origin = std::str::from_utf8(pair[0])
                .expect("ASCII test fixture origin")
                .strip_prefix("file:")
                .expect("file origin");
            // Native Git for Windows uses forward slashes; Rust PathBuf can
            // serialize the same admitted source using backslashes.
            #[cfg(windows)]
            let origin = origin.replace('\\', "/").trim_matches('"').to_ascii_lowercase();
            #[cfg(not(windows))]
            let origin = origin.to_owned();
            (origin, pair[1].to_vec())
        })
        .collect()
}

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
    let (parsed, origins) = super::config_state::parse_origins_for_tests(&serialized)
        .expect("origin-bearing host parser");
    assert!(origins.contains(&included), "include origin must be parsed");
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
    let (expected_values, _) = super::config_state::parse_origins_for_tests(&native.stdout)
        .expect("native origin-bearing config records");
    let (actual_values, _) = super::config_state::parse_origins_for_tests(&actual)
        .expect("candidate origin-bearing config records");
    assert_eq!(actual_values, expected_values, "configuration values diverged");
    assert_eq!(
        ordered_file_origin_records(&actual),
        ordered_file_origin_records(&native.stdout),
        "ordered file-origin/record pairs differ from native Git"
    );

    // A different-key order change is invisible to a BTreeMap of values,
    // but the actual production host verifier rejects the altered stream.
    let fields = actual[..actual.len() - 1]
        .split(|byte| *byte == 0)
        .collect::<Vec<_>>();
    let mut pairs = fields.chunks_exact(2)
        .map(|pair| (pair[0].to_vec(), pair[1].to_vec()))
        .collect::<Vec<_>>();
    let marker = pairs.iter()
        .position(|(_, record)| record.starts_with(b"fixture.marker"))
        .expect("marker fixture");
    let include = pairs.iter()
        .position(|(_, record)| record.starts_with(b"include.path"))
        .expect("include fixture");
    pairs.swap(marker, include);
    let mut reordered = Vec::new();
    for (origin, record) in pairs {
        reordered.extend_from_slice(&origin);
        reordered.push(0);
        reordered.extend_from_slice(&record);
        reordered.push(0);
    }
    let (reordered_values, reordered_origins) =
        super::config_state::parse_origins_for_tests(&reordered)
            .expect("reordered records still have valid framing");
    assert_eq!(reordered_values, actual_values, "fixture must only reorder keys");
    assert_eq!(
        reordered_origins,
        super::config_state::parse_origins_for_tests(&actual)
            .expect("original origins").1
    );
    let verifier = super::config_state::ChildConfig::capture(&actual, &fixture)
        .expect("prepare real host stream verifier");
    verifier.verify_host(&actual).expect("original order remains valid");
    assert!(verifier.verify_host(&reordered).is_err(),
        "the actual host verifier must reject cross-key reordering");

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
    let (records, origins) = super::config_state::parse_origins_for_tests(&output)
        .expect("origin-bearing host records");
    assert!(origins.contains(&present_global));
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
fn gix_gitdir_conditional_requires_admitted_repository_identity() {
    // Capturing the exact config/include bytes does NOT authorize reading or
    // resolving an unadmitted repository gitdir. The gitdir is a separate
    // metadata-only frontier, even though the source values are complete.
    let dir = tempfile::tempdir().expect("fixture");
    let fixture = {
        #[cfg(windows)]
        {
            // Keep Windows native paths free of the verbatim \\?\ prefix.
            dir.path().to_path_buf()
        }
        #[cfg(not(windows))]
        {
            dir.path().canonicalize().expect("fixture path")
        }
    };
    let root = fixture.join("root.conf");
    let conditional = fixture.join("conditional.conf");
    let gitdir = fixture.join("selected.git");
    std::fs::create_dir(&gitdir).expect("gitdir fixture");
    std::fs::write(
        &root,
        format!(
            "[includeIf \"gitdir:{}\"]\n path = conditional.conf\n",
            // Git's gitdir: patterns use '/' even on native Windows.
            gitdir.to_string_lossy().replace('\\', "/")
        ),
    )
    .expect("conditional fixture");
    std::fs::write(&conditional, b"[fixture]\n context = admitted\n").expect("included fixture");
    let environment = BTreeMap::<OsString, OsString>::new();
    let mut sources =
        target_config::Sources::new(None, [root.clone()]).expect("path-only source frontier");
    for _ in 0..3 {
        if sources.complete() {
            break;
        }
        let admitted = sources.paths();
        sources
            .advance(|path| admitted.contains(path), &environment)
            .expect("admit fixture config roots and includes");
    }
    assert!(sources.complete(), "config bytes are fully admitted");
    assert!(
        sources
            .ordered_host_output(std::slice::from_ref(&root), &environment, Some(&gitdir))
            .is_err(),
        "complete config snapshots cannot authorize a new gitdir identity"
    );

    sources
        .add_directory_context(&gitdir)
        .expect("plan separate metadata-only gitdir evidence");
    assert!(!sources.complete(), "new gitdir identity remains pending");
    let admitted = sources.paths();
    sources
        .advance(|path| admitted.contains(path), &environment)
        .expect("admit gitdir directory and ancestor identities");
    let records = sources
        .ordered_host_output(std::slice::from_ref(&root), &environment, Some(&gitdir))
        .and_then(|output| super::config_state::parse_origins_for_tests(&output)
            .map(|(values, _)| values))
        .expect("interpret config only after gitdir metadata admission");
    assert_eq!(
        records
            .get(b"fixture.context".as_slice())
            .and_then(|values| values.last()),
        Some(&Some(b"admitted".to_vec())),
        "gitdir-matched include must survive admitted source evaluation"
    );
    // A previously approved gitdir disappearing before the write invalidates
    // its captured directory identity, even with unchanged config bytes.
    std::fs::remove_dir(&gitdir).expect("remove gitdir after metadata admission");
    assert!(
        sources.revalidate().is_err(),
        "stale gitdir metadata must invalidate the prepared source graph"
    );
}

#[test]
fn gix_hasconfig_remote_url_lookahead_is_bounded_to_admitted_source_bytes() {
    // hasconfig:remote.*.url must see later root records but must never infer
    // authorization for an included source. Every root/child read remains in
    // the separately admitted, immutable exact-Path Read frontier.
    let dir = tempfile::tempdir().expect("fixture");
    // Git for Windows may refuse the canonical \\?\ path spelling even though
    // the Rust snapshot capture accepts it. Use the ordinary path in the
    // native Git oracle; keep the same physical fixture and admission checks.
    let fixture = {
        #[cfg(windows)]
        {
            dir.path().to_path_buf()
        }
        #[cfg(not(windows))]
        {
            dir.path().canonicalize().expect("canonical fixture")
        }
    };
    let root = fixture.join("root.conf");
    let conditional = fixture.join("conditional.conf");
    let matched_config = b"[includeIf \"hasconfig:remote.*.url:https://example.invalid/allowed\"]\n path = conditional.conf\n[remote \"origin\"]\n url = https://example.invalid/allowed\n";
    let unmatched_config = b"[includeIf \"hasconfig:remote.*.url:https://example.invalid/allowed\"]\n path = conditional.conf\n[remote \"origin\"]\n url = https://example.invalid/other\n";
    std::fs::write(&root, matched_config).expect("write root");
    std::fs::write(&conditional, b"[fixture]\n marker = conditional\n")
        .expect("write conditional source");
    let environment = BTreeMap::<OsString, OsString>::new();
    let capture = || {
        let mut sources =
            target_config::Sources::new(None, [root.clone()]).expect("root path-only source plan");
        for _ in 0..4 {
            if sources.complete() {
                break;
            }
            let admitted = sources.paths();
            sources
                .advance(|path| admitted.contains(path), &environment)
                .expect("separately admit all config and include paths");
        }
        assert!(
            sources.complete(),
            "all conditional sources must be captured"
        );
        assert!(
            sources.captured_paths().any(|path| path == &conditional),
            "includeIf destination remains a candidate Read even before the URL match"
        );
        sources
    };

    // Test-only native Git oracle: this invocation belongs exclusively to
    // a fixture after path admission, not to prepared-operation preflight.
    let native_fixture = || {
        let native = std::process::Command::new("git")
            .args(["config", "--file"])
            .arg(&root)
            .args(["--show-origin", "--null", "--list", "--includes"])
            .output()
            .expect("installed native Git fixture oracle");
        assert!(
            native.status.success(),
            "native Git rejected a valid admitted hasconfig fixture"
        );
        super::config_state::parse_origins_for_tests(&native.stdout)
            .expect("parse native origin-bearing config records").0
    };

    let sources = capture();
    let matched = sources
        .ordered_host_output(std::slice::from_ref(&root), &environment, None)
        .and_then(|output| super::config_state::parse_origins_for_tests(&output)
            .map(|(values, _)| values))
        .expect("evaluate admitted bytes without a Git process");
    assert_eq!(
        matched
            .get(b"fixture.marker".as_slice())
            .and_then(|values| values.last()),
        Some(&Some(b"conditional".to_vec())),
        "later remote URL must participate in admitted includeIf lookahead"
    );
    assert_eq!(
        matched,
        native_fixture(),
        "admitted hasconfig evaluation differs from native Git fixture values"
    );

    std::fs::write(&root, unmatched_config).expect("change root URL after admission");
    assert!(
        sources.revalidate().is_err(),
        "a changed remote URL cannot reuse the earlier admitted source graph"
    );
    let sources = capture();
    let unmatched = sources
        .ordered_host_output(std::slice::from_ref(&root), &environment, None)
        .and_then(|output| super::config_state::parse_origins_for_tests(&output)
            .map(|(values, _)| values))
        .expect("evaluate nonmatching admitted URLs");
    assert!(
        !unmatched.contains_key(b"fixture.marker".as_slice()),
        "a nonmatching remote URL must not activate an includeIf record"
    );
    assert_eq!(
        unmatched,
        native_fixture(),
        "nonmatching hasconfig evaluation differs from native Git fixture values"
    );

    // Git forbids remote URLs in hasconfig-conditional included files: they
    // would recursively influence the condition that selected the file.
    std::fs::write(&root, matched_config).expect("restore matching root");
    std::fs::write(
        &conditional,
        b"[remote \"injected\"]\n url = https://example.invalid/unapproved\n",
    )
    .expect("replace conditional source");
    let sources = capture();
    assert!(
        sources
            .ordered_host_output(std::slice::from_ref(&root), &environment, None)
            .is_err(),
        "remote URL in a hasconfig-conditional include must fail closed"
    );
}

#[test]
fn git_var_legacy_path_query_support_is_classified_per_command() {
    // Test-only native oracle. Git 2.40 documentation does not enumerate
    // these variables; Git 2.42 does. An older writer may reject some of
    // them with usage, while a supported query may have no value. Neither
    // outcome authorizes a production fallback or any new filesystem reads.
    let dir = tempfile::tempdir().expect("isolated Git variable fixture");
    for variable in [
        "GIT_ATTR_SYSTEM",
        "GIT_ATTR_GLOBAL",
        "GIT_CONFIG_SYSTEM",
        "GIT_CONFIG_GLOBAL",
        "GIT_SHELL_PATH",
    ] {
        let native = std::process::Command::new("git")
            .current_dir(dir.path())
            .args(["--no-pager", "var", variable])
            .output()
            .expect("selected test Git variable oracle");
        let outcome = if native.status.success() {
            "supported"
        } else if native.status.code() == Some(1)
            && native.stdout.is_empty()
            && native.stderr.is_empty()
        {
            "supported-but-unset"
        } else if native
            .stderr
            .windows(b"usage: git var".len())
            .any(|part| part.eq_ignore_ascii_case(b"usage: git var"))
        {
            "unsupported-variable"
        } else {
            panic!(
                "Git var {variable} failed for another reason: code={:?}, stdout_bytes={}, stderr_bytes={}",
                native.status.code(),
                native.stdout.len(),
                native.stderr.len()
            );
        };
        // Preserve a *value-free* native compatibility receipt. Do not log
        // paths or configuration values returned by any Git variable.
        eprintln!("I324_GIT_VAR_NATIVE variable={variable} outcome={outcome}");
    }
}

#[test]
fn git_check_attr_source_is_not_interchangeable_with_cached() {
    // Git 2.40 introduced check-attr --source. Git 2.36 still supports
    // --cached, but cached reads the CURRENT index, not an arbitrary
    // Checkout target tree. This test-only native oracle must never cause
    // production to substitute one for the other.
    let dir = tempfile::tempdir().expect("isolated Git attribute fixture");
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .current_dir(dir.path())
            .args(args)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_ATTR_NOSYSTEM", "1")
            .env("HOME", dir.path())
            .env("XDG_CONFIG_HOME", dir.path().join("xdg"))
            .output()
            .expect("Git fixture command");
        assert!(
            output.status.success(),
            "Git attribute fixture setup failed: command={:?}, exit={:?}, stdout_bytes={}, stderr_bytes={}",
            args,
            output.status.code(),
            output.stdout.len(),
            output.stderr.len()
        );
        output
    };
    git(&["init", "-q"]);
    std::fs::write(
        dir.path().join(".gitattributes"),
        b"*.txt filter=committed\n",
    )
    .expect("original attributes");
    std::fs::write(dir.path().join("tracked.txt"), b"tracked").expect("tracked fixture");
    git(&["add", "--", ".gitattributes", "tracked.txt"]);
    git(&[
        "-c",
        "user.name=I324 Fixture",
        "-c",
        "user.email=i324@example.invalid",
        "commit",
        "-q",
        "-m",
        "fixture",
    ]);

    std::fs::write(dir.path().join(".gitattributes"), b"*.txt filter=index\n")
        .expect("index attributes");
    git(&["add", "--", ".gitattributes"]);
    std::fs::write(
        dir.path().join(".gitattributes"),
        b"*.txt filter=worktree\n",
    )
    .expect("worktree attributes");

    let current = git(&["check-attr", "-z", "filter", "--", "tracked.txt"]);
    let cached = git(&[
        "check-attr",
        "-z",
        "--cached",
        "filter",
        "--",
        "tracked.txt",
    ]);
    assert_eq!(current.stdout, b"tracked.txt\0filter\0worktree\0");
    assert_eq!(cached.stdout, b"tracked.txt\0filter\0index\0");
    assert_ne!(
        cached.stdout, current.stdout,
        "cached and worktree attributes cannot be conflated"
    );

    // A bounded gix-native primitive can evaluate TARGET attributes from
    // externally supplied, already admitted bytes instead of asking old
    // Git to interpret --source. The real adapter must still prove tree
    // reads, ordering, macros, includes and filter effects separately.
    let mut search = gix_attributes::Search::default();
    let mut metadata = gix_attributes::search::MetadataCollection::default();
    search
        .add_patterns_buffer(
            b"*.txt filter=committed\n",
            ".gitattributes".into(),
            None,
            &mut metadata,
            true,
        )
        .expect("in-memory target-tree attributes");
    let mut outcome = gix_attributes::search::Outcome::default();
    outcome.initialize_with_selection(&metadata, ["filter"]);
    assert!(search.pattern_matching_relative_path(
        "tracked.txt".into(),
        gix_attributes::glob::pattern::Case::Sensitive,
        Some(false),
        &mut outcome,
    ));
    let selected = outcome
        .iter_selected()
        .map(|value| value.assignment.to_string())
        .collect::<Vec<_>>();
    assert_eq!(selected, vec!["filter=committed".to_owned()]);

    let source = std::process::Command::new("git")
        .current_dir(dir.path())
        .args([
            "check-attr",
            "-z",
            "--source=HEAD",
            "filter",
            "--",
            "tracked.txt",
        ])
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_ATTR_NOSYSTEM", "1")
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", dir.path().join("xdg"))
        .output()
        .expect("native target-tree attribute oracle");
    let outcome = if source.status.success() {
        assert_eq!(source.stdout, b"tracked.txt\0filter\0committed\0");
        assert_ne!(
            source.stdout, cached.stdout,
            "HEAD attributes must not be replaced by the current index"
        );
        "supported"
    } else {
        let stderr = String::from_utf8_lossy(&source.stderr).to_ascii_lowercase();
        assert!(
            source.status.code() == Some(129)
                && (stderr.contains("usage: git check-attr") || stderr.contains("unknown option")),
            "unexpected native check-attr failure: exit={:?}, stdout_bytes={}, stderr_bytes={}",
            source.status.code(),
            source.stdout.len(),
            source.stderr.len()
        );
        "unsupported-option"
    };
    // Do not print returned configuration, path values, or attributes.
    eprintln!("I324_CHECK_ATTR_SOURCE_NATIVE outcome={outcome}");
}

#[test]
fn gix_supplied_target_tree_nested_attributes_respect_git_precedence() {
    // Test-only source oracle. A future replacement for the unsupported old
    // Git check-attr --source option must evaluate *every* admitted ancestor
    // from the target tree; a root-only pattern can miss nested filter effects.
    // No gix API in this test opens a Git file, executes a child or discovers
    // an ambient attribute location. Git CLI here is a TEST-ONLY oracle.
    let dir = tempfile::tempdir().expect("isolated target attributes fixture");
    let root = dir.path();
    std::fs::create_dir_all(root.join("src/deep")).expect("nested fixture");
    std::fs::write(root.join(".gitattributes"), b"*.txt filter=root\n")
        .expect("root target attributes");
    std::fs::write(root.join("src/.gitattributes"), b"*.txt filter=nested\n")
        .expect("nested target attributes");
    std::fs::write(root.join("src/deep/.gitattributes"), b"*.txt -filter\n")
        .expect("deep target attributes");
    for path in ["outside.txt", "src/inside.txt", "src/deep/inside.txt"] {
        std::fs::write(root.join(path), b"fixture\n").expect("tracked target");
    }
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .current_dir(root)
            .args(args)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_ATTR_NOSYSTEM", "1")
            .env("HOME", root)
            .env("XDG_CONFIG_HOME", root.join("xdg"))
            .output()
            .expect("native test-only Git command");
        assert!(
            output.status.success(),
            "target fixture command failed: exit={:?}, stdout_bytes={}, stderr_bytes={}",
            output.status.code(),
            output.stdout.len(),
            output.stderr.len()
        );
        output
    };
    git(&["init", "-q"]);
    git(&[
        "add",
        "--",
        ".gitattributes",
        "src/.gitattributes",
        "src/deep/.gitattributes",
        "outside.txt",
        "src/inside.txt",
        "src/deep/inside.txt",
    ]);
    git(&[
        "-c",
        "user.name=I324 Fixture",
        "-c",
        "user.email=i324@example.invalid",
        "commit",
        "-q",
        "-m",
        "frozen target",
    ]);

    // Capture actual committed tree attribute blob bytes through the existing
    // checkout target snapshot, not a duplicated hand-authored rule list.
    // This is TEST-ONLY object discovery, not a new production Read grant.
    let head = git(&["rev-parse", "HEAD"]);
    let commit_id = std::str::from_utf8(&head.stdout)
        .expect("fixture commit oid")
        .trim();
    let target = super::checkout_state::TargetTree::capture(root, commit_id)
        .expect("test-only committed target tree capture");
    assert_eq!(target.attributes.len(), 3);

    // Mutate the current worktree after committing: target-tree evaluation
    // must continue to match the recorded source bytes, not these new rules.
    std::fs::write(
        root.join("src/deep/.gitattributes"),
        b"*.txt filter=worktree\n",
    )
    .expect("mutated worktree attributes");

    // Even well-formed replacement rules cannot be substituted for the
    // commit's original .gitattributes bytes: a complete source-name set
    // alone is not a trustworthy Git tree snapshot.
    let mut forged = super::checkout_state::TargetTree {
        commit: target.commit.clone(),
        tree: target.tree.clone(),
        entries: target.entries.clone(),
        attributes: target.attributes.clone(),
    };
    forged.attributes.insert(
        "src/.gitattributes".to_owned(),
        b"*.txt filter=forged\n".to_vec(),
    );
    assert!(
        super::checkout_state::select_captured_target_filter(
            &forged,
            "src/inside.txt",
            gix_attributes::glob::pattern::Case::Sensitive,
        )
        .is_err(),
        "well-formed but forged attribute bytes must fail blob OID binding"
    );
    forged.attributes = target.attributes.clone();
    forged
        .entries
        .get_mut("src/.gitattributes")
        .expect("captured source entry")
        .1 = "0000000000000000000000000000000000000000".to_owned();
    assert!(
        super::checkout_state::select_captured_target_filter(
            &forged,
            "src/inside.txt",
            gix_attributes::glob::pattern::Case::Sensitive,
        )
        .is_err(),
        "changed target entry OID must not authenticate valid captured bytes"
    );

    // A pure evaluator must fail closed on corrupt supplied bytes or an
    // untrusted source spelling, without probing the filesystem to recover.
    let mut corrupt = super::checkout_state::TargetTree {
        commit: target.commit.clone(),
        tree: target.tree.clone(),
        entries: target.entries.clone(),
        attributes: target.attributes.clone(),
    };
    corrupt.attributes.insert(
        "src/.gitattributes".to_owned(),
        b"*.txt filter=lfs invalid$name\n".to_vec(),
    );
    assert!(
        super::checkout_state::select_captured_target_filter(
            &corrupt,
            "src/inside.txt",
            gix_attributes::glob::pattern::Case::Sensitive,
        )
        .is_err()
    );
    corrupt.attributes.remove("src/.gitattributes");
    assert!(
        super::checkout_state::select_captured_target_filter(
            &corrupt,
            "src/inside.txt",
            gix_attributes::glob::pattern::Case::Sensitive,
        )
        .is_err(),
        "missing committed nested attribute bytes must be rejected"
    );
    corrupt.attributes.insert(
        "../outside/.gitattributes".to_owned(),
        b"*.txt filter=untrusted\n".to_vec(),
    );
    assert!(
        super::checkout_state::select_captured_target_filter(
            &corrupt,
            "src/inside.txt",
            gix_attributes::glob::pattern::Case::Sensitive,
        )
        .is_err()
    );
    assert!(
        super::checkout_state::select_captured_target_filter(
            &target,
            "../escape.txt",
            gix_attributes::glob::pattern::Case::Sensitive,
        )
        .is_err()
    );

    // An invented source absent from the tree entry index cannot override
    // committed rules, even if its text is syntactically well formed.
    corrupt.attributes = target.attributes.clone();
    corrupt.attributes.insert(
        "src/other/.gitattributes".to_owned(),
        b"*.txt filter=invented\n".to_vec(),
    );
    assert!(
        super::checkout_state::select_captured_target_filter(
            &corrupt,
            "src/inside.txt",
            gix_attributes::glob::pattern::Case::Sensitive,
        )
        .is_err(),
        "unrecorded attribute sources must be refused"
    );

    let paths = [
        ("outside.txt", "filter=root", "root"),
        ("src/inside.txt", "filter=nested", "nested"),
        ("src/deep/inside.txt", "-filter", "unset"),
    ];
    for (path, expected_gix, expected_native) in paths {
        let selection = super::checkout_state::select_captured_target_filter(
            &target,
            path,
            gix_attributes::glob::pattern::Case::Sensitive,
        )
        .expect("only admitted target tree bytes are interpreted");
        assert_eq!(
            selection.as_deref(),
            Some(expected_gix),
            "target-tree rule component disagrees at {path}"
        );

        let native = std::process::Command::new("git")
            .current_dir(root)
            .args(["check-attr", "-z", "--source=HEAD", "filter", "--", path])
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_ATTR_NOSYSTEM", "1")
            .env("HOME", root)
            .env("XDG_CONFIG_HOME", root.join("xdg"))
            .output()
            .expect("native committed-target oracle");
        if native.status.success() {
            let expected = format!("{path}\0filter\0{expected_native}\0");
            assert_eq!(
                native.stdout,
                expected.as_bytes(),
                "newer Git target-tree precedence differs at {path}"
            );
        } else {
            // Git for Windows 2.36 has no --source option; this must not
            // be treated as evidence that --cached answers the same query.
            let stderr = String::from_utf8_lossy(&native.stderr).to_ascii_lowercase();
            assert!(
                native.status.code() == Some(129)
                    && (stderr.contains("usage: git check-attr")
                        || stderr.contains("unknown option")),
                "unexpected old-Git target query: exit={:?}, stdout_bytes={}, stderr_bytes={}",
                native.status.code(),
                native.stdout.len(),
                native.stderr.len()
            );
        }
    }
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
fn gix_admitted_filter_attributes_match_native_git_fixture() {
    // Native Git is a test-only differential oracle, never a preflight probe.
    // Search consumes only the already validated in-memory .gitattributes
    // source and must not discover or read ambient Git attribute files.
    const ATTRIBUTES: &[u8] =
        b"[attr]myfilter filter=lfs\n*.bin myfilter\n*.txt -filter\n*.jpg filter=photo\n*.md !filter\nspecial.bin filter=direct\n*.override filter=first\n*.override filter=second\n";
    assert!(
        gix_attributes::parse(ATTRIBUTES).all(|line| {
            let Ok((_, assignments, _)) = line else {
                return false;
            };
            assignments.into_iter().all(|entry| entry.is_ok())
        }),
        "gix search may not silently ignore malformed filter assignments"
    );

    let dir = tempfile::tempdir().expect("native filter fixture");
    let attributes_file = dir.path().join(".gitattributes");
    std::fs::write(&attributes_file, ATTRIBUTES).expect("attribute fixture");
    let init = std::process::Command::new("git")
        .arg("-C")
        .arg(dir.path())
        .args(["init", "--quiet"])
        .output()
        .expect("native Git test-only init");
    assert!(
        init.status.success(),
        "native fixture initialization failed"
    );

    let mut search = gix_attributes::Search::default();
    let mut metadata = gix_attributes::search::MetadataCollection::default();
    search
        .add_patterns_buffer(
            ATTRIBUTES,
            ".gitattributes".into(),
            None,
            &mut metadata,
            true,
        )
        .expect("gix in-memory pattern search");
    for (path, expected) in [
        ("sample.bin", "filter=lfs"),
        ("sample.txt", "-filter"),
        ("sample.jpg", "filter=photo"),
        ("sample.md", "!filter"),
        // Later specific/duplicate patterns override an earlier matching
        // macro or assignment; effects must not be inferred from first match.
        ("special.bin", "filter=direct"),
        ("sample.override", "filter=second"),
    ] {
        let native = std::process::Command::new("git")
            .arg("-C")
            .arg(dir.path())
            .args(["check-attr", "-z", "filter", "--", path])
            .env("HOME", dir.path())
            .env("XDG_CONFIG_HOME", dir.path().join("xdg"))
            .env("GIT_ATTR_NOSYSTEM", "1")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", dir.path().join("empty-global-config"))
            .output()
            .expect("native attribute fixture oracle");
        assert!(native.status.success(), "native check-attr failed: {path}");
        let fields = native.stdout.split(|b| *b == 0).collect::<Vec<_>>();
        assert_eq!(fields.len(), 4, "expected native NUL attribute triple");
        assert_eq!(fields[0], path.as_bytes());
        assert_eq!(fields[1], b"filter");
        assert!(fields[3].is_empty());
        let native_value = std::str::from_utf8(fields[2]).expect("ASCII native filter state");
        let native_assignment = match native_value {
            "unset" => "-filter".to_owned(),
            "unspecified" => "!filter".to_owned(),
            "set" => "filter".to_owned(),
            value => format!("filter={value}"),
        };
        assert_eq!(
            native_assignment, expected,
            "unexpected native fixture: {path}"
        );

        let mut outcome = gix_attributes::search::Outcome::default();
        outcome.initialize_with_selection(&metadata, ["filter"]);
        assert!(
            search.pattern_matching_relative_path(
                path.into(),
                gix_attributes::glob::pattern::Case::Sensitive,
                Some(false),
                &mut outcome,
            ),
            "gix did not match the admitted attribute pattern: {path}"
        );
        let gix_assignments = outcome
            .iter_selected()
            .map(|item| item.assignment.to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            gix_assignments,
            vec![native_assignment],
            "filter effect differs between native Git and gix: {path}"
        );
    }
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

#[cfg(unix)]
#[test]
fn gix_pure_candidate_records_zero_child_git_invocations() {
    use std::os::unix::fs::PermissionsExt;

    // Run the gix-only fixture in its own test-harness process, so a hostile
    // PATH cannot interfere with other native Git fixture tests in parallel.
    // The test-child launcher is NOT a Git preflight operation.
    let dir = tempfile::tempdir().expect("isolated command sentinel");
    let fake_git = dir.path().join("git");
    let log = dir.path().join("git-invocations");
    std::fs::write(
        &fake_git,
        b"#!/bin/sh\nprintf 'git invoked\n' >> \"$TALOS_I324_FAKE_GIT_LOG\"\nexit 97\n",
    )
    .expect("install fake git");
    std::fs::set_permissions(&fake_git, std::fs::Permissions::from_mode(0o700))
        .expect("make fake git executable");

    // Prove the sentinel itself works before measuring pure-gix behavior.
    // This intentionally spawned Git is a TEST-ONLY positive control, not
    // part of preflight; clear its log before launching the candidate child.
    let control = std::process::Command::new("git")
        .env("PATH", dir.path())
        .env("TALOS_I324_FAKE_GIT_LOG", &log)
        .status()
        .expect("positive-control fake Git");
    assert_eq!(control.code(), Some(97), "fake Git must fail on purpose");
    assert_eq!(
        std::fs::read(&log).expect("positive-control Git invocation log"),
        b"git invoked\n"
    );
    std::fs::remove_file(&log).expect("reset the sentinel before candidate");

    let output = std::process::Command::new(std::env::current_exe().expect("test binary"))
        .args([
            "gix_pure_candidate_no_child_fixture",
            "--ignored",
            "--nocapture",
        ])
        .env("PATH", dir.path())
        .env("TALOS_I324_RUN_ISOLATED_CHILD", "1")
        .env("TALOS_I324_FAKE_GIT_LOG", &log)
        .output()
        .expect("isolated gix-only test process");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success() && stdout.contains("I324_PURE_CANDIDATE_CHILD_OK"),
        "isolated pure-gix fixture failed or did not execute: stdout={stdout}, stderr={stderr}"
    );
    assert!(
        !log.exists(),
        "pure gix byte/lexical candidates launched a Git executable via PATH"
    );
}

#[cfg(unix)]
#[test]
#[ignore = "only invoked by the isolated no-Git-child sentinel parent"]
fn gix_pure_candidate_no_child_fixture() {
    if std::env::var("TALOS_I324_RUN_ISOLATED_CHILD").as_deref() != Ok("1") {
        return;
    }

    let config = b"[core]\n bare = false\n[include]\n path = never-authorized.conf\n";
    let parsed = target_config::parse_records(config).expect("parse admitted bytes only");
    assert!(
        parsed.iter().any(|record| record.key == b"include.path"),
        "include path remains a lexical name, not an implicit read"
    );
    let named = target_config::declared_paths(
        config,
        std::path::Path::new("/i324-unadmitted/root.conf"),
        &BTreeMap::<OsString, OsString>::new(),
        None,
    )
    .expect("compute include without ambient discovery");
    assert_eq!(
        named,
        vec![std::path::PathBuf::from(
            "/i324-unadmitted/never-authorized.conf"
        )]
    );

    let _file = gix::config::File::from_bytes_no_includes(
        config,
        gix::config::file::Metadata::default(),
        Default::default(),
    )
    .expect("gix config must consume only supplied bytes");
    assert!(
        gix::validate::reference::branch_name(b"refs/heads/topic".as_bstr()).is_ok(),
        "gix branch lexical validation must not invoke native Git"
    );
    let attrs = b"*.txt filter=lfs\n";
    assert!(
        gix_attributes::parse(attrs).all(|line| {
            let Ok((_, assignments, _)) = line else {
                return false;
            };
            assignments.into_iter().all(|assignment| assignment.is_ok())
        }),
        "attribute parsing refuses malformed assignments"
    );
    let mut search = gix_attributes::Search::default();
    let mut metadata = gix_attributes::search::MetadataCollection::default();
    search
        .add_patterns_buffer(attrs, ".gitattributes".into(), None, &mut metadata, true)
        .expect("in-memory attribute search");
    let mut result = gix_attributes::search::Outcome::default();
    result.initialize_with_selection(&metadata, ["filter"]);
    assert!(
        search.pattern_matching_relative_path(
            "sample.txt".into(),
            gix_attributes::glob::pattern::Case::Sensitive,
            Some(false),
            &mut result,
        ),
        "supplied attribute bytes should match"
    );
    let values = result
        .iter_selected()
        .map(|value| value.assignment.to_string())
        .collect::<Vec<_>>();
    assert_eq!(values, vec!["filter=lfs".to_owned()]);

    // Exercise the actual I324 supplied-tree candidate under the hostile
    // PATH sentinel, not only its underlying gix matcher. These are
    // synthetic TEST-ONLY bytes; neither repository-object Read admission
    // nor target-tree authenticity is implied by a matching blob hash.
    // Both Git object hash families must bind the captured rule bytes.
    for commit in ["0".repeat(40), "0".repeat(64)] {
        let commit_id =
            gix::ObjectId::from_hex(commit.as_bytes()).expect("synthetic hash-kind fixture");
        let blob_id = gix::objs::compute_hash(commit_id.kind(), gix::objs::Kind::Blob, attrs)
            .expect("pure in-memory Git blob hashing");
        let target = super::checkout_state::TargetTree {
            commit,
            tree: String::new(),
            entries: BTreeMap::from([(
                ".gitattributes".to_owned(),
                (0o100644, blob_id.to_string()),
            )]),
            attributes: BTreeMap::from([(".gitattributes".to_owned(), attrs.to_vec())]),
        };
        assert_eq!(
            super::checkout_state::select_captured_target_filter(
                &target,
                "sample.txt",
                gix_attributes::glob::pattern::Case::Sensitive,
            )
            .expect("test-only supplied target tree lookup")
            .as_deref(),
            Some("filter=lfs")
        );
    }
    println!("I324_PURE_CANDIDATE_CHILD_OK");
}
