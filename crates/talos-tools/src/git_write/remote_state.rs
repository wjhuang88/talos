//! Remote destination evidence for prepared discovery; transport execution is pending.
//! Never contacts a remote or prints credential-bearing configuration values.

use super::GitToolError;

fn invalid() -> GitToolError {
    GitToolError::InvalidInput("Git remote configuration cannot be resolved safely".into())
}

// Permission path normalization folds `..` lexically, while Git and the OS
// traverse a preceding symlink before applying it. Keep parent traversal only
// in the leading relative prefix, where both interpretations agree after the
// already-bound workdir is canonicalized.
pub(super) fn validate_local_path_syntax(path: &std::path::Path) -> Result<(), GitToolError> {
    let mut ordinary_component = false;
    for component in path.components() {
        match component {
            std::path::Component::Normal(_) => ordinary_component = true,
            std::path::Component::ParentDir if path.is_absolute() || ordinary_component => {
                return Err(GitToolError::InvalidInput(
                    "Git receiver path has ambiguous parent traversal".into(),
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

/// Built-in git transport connects directly; other transports need additional
/// receiver or helper evidence before mutation can be enabled.
pub(super) fn validate_direct_transport(destinations: &[String]) -> Result<(), GitToolError> {
    std::panic::catch_unwind(|| {
        if destinations.is_empty() || destinations.iter().any(|destination| {
            !gix::url::parse(destination).is_ok_and(|url| {
                url.scheme == gix::url::Scheme::Git && url.host.is_some()
                    && url.user.is_none() && url.password.is_none()
            })
        }) {
            return Err(GitToolError::InvalidInput("Push receiver or transport helper effects require additional validation (local receiver configuration/hooks, SSH configuration/helpers, or HTTP credentials/proxies/redirects); no mutation performed. Use a separately authorized Git invocation".into()));
        }
        Ok(())
    }).unwrap_or_else(|_| Err(invalid()))
}

/// Legacy remote files can override fallback destination resolution but are not
/// represented by config --show-origin. Do not silently resolve a different URL.
pub(super) fn reject_legacy_remotes(common_dir: &std::path::Path) -> Result<(), GitToolError> {
    for name in ["remotes", "branches"] {
        let path = common_dir.join(name);
        match std::fs::symlink_metadata(&path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
                if std::fs::read_dir(&path)
                    .map_err(|_| invalid())?
                    .next()
                    .is_some()
                {
                    return Err(GitToolError::InvalidInput("Legacy Git remote files require separately authorized destination discovery".into()));
                }
            }
            _ => return Err(invalid()),
        }
    }
    Ok(())
}

/// Classify a destination without reading a local receiver before admission.
/// A local receiver needs path authority, not merely network authority.
pub(super) fn local_destination(
    destination: &str,
    workdir: &std::path::Path,
) -> Result<Option<std::path::PathBuf>, GitToolError> {
    std::panic::catch_unwind(|| {
        let url = gix::url::parse(destination).map_err(|_| invalid())?;
        if url.user.is_some() || url.password.is_some() || destination.contains(['?', '#']) {
            return Err(GitToolError::InvalidInput("Credential-bearing or ambiguous remote destinations require a separately authorized invocation".into()));
        }
        match url.scheme {
            gix::url::Scheme::File => {
                if url.host.is_some() || destination.contains('%') {
                    return Err(invalid());
                }
                let raw: &[u8] = url.path.as_ref();
                let path = std::path::Path::new(std::str::from_utf8(raw).map_err(|_| invalid())?);
                if path.as_os_str().is_empty() || raw.starts_with(b"~") {
                    return Err(invalid());
                }
                validate_local_path_syntax(path)?;
                Ok(Some(if path.is_absolute() { path.to_path_buf() } else { workdir.join(path) }))
            }
            gix::url::Scheme::Git | gix::url::Scheme::Ssh
            | gix::url::Scheme::Http | gix::url::Scheme::Https => Ok(None),
            gix::url::Scheme::Ext | gix::url::Scheme::Helper(_) | gix::url::Scheme::HelperUrl(_) => Err(GitToolError::InvalidInput("Git remote helper execution requires separate authorization".into())),
        }
    }).unwrap_or_else(|_| Err(invalid()))
}

#[test]
fn local_receivers_are_paths_and_helpers_are_not_network_authority() {
    let root = tempfile::tempdir().expect("fixture");
    for (destination, expected) in [
        ("../receiver.git", Some(root.path().join("../receiver.git"))),
        ("git://example.invalid/repo", None),
        ("https://example.invalid/repo", None),
        ("git://[::1]/repo", None),
    ] {
        assert_eq!(
            local_destination(destination, root.path()).expect("destination"),
            expected
        );
    }
    let path = root.path().join("receiver.git");
    assert_eq!(
        local_destination(&format!("file://{}", path.display()), root.path()).expect("file"),
        Some(path)
    );
    for destination in ["foo/../receiver.git", "file:///tmp/foo/../receiver.git"] {
        assert!(
            local_destination(destination, root.path()).is_err(),
            "ambiguous parent traversal must refuse: {destination}"
        );
    }
    assert!(
        local_destination("../../receiver.git", root.path()).is_ok(),
        "leading parent traversal is a valid external receiver candidate"
    );
    for destination in [
        "ext::arbitrary-command",
        "ssh::address",
        "custom://address",
        "custom::https://example.invalid/repo",
        "https://token@example.invalid/repo",
        "~/receiver.git",
    ] {
        assert!(
            local_destination(destination, root.path()).is_err(),
            "{destination}"
        );
    }
}

/// Resolve ordered configuration records. Rewrite ties retain first base group,
/// empty URL values reset their list, and explicit pushurl overrides pushInsteadOf.
pub(super) fn destinations(
    records: &[(String, String)],
    remote: &str,
    push: bool,
) -> Result<Vec<String>, GitToolError> {
    super::validate_git_positional(remote)?;
    let list = |suffix: &str| {
        let key = format!("remote.{remote}.{suffix}");
        let mut urls = Vec::new();
        for (_, value) in records.iter().filter(|(name, _)| name == &key) {
            if value.is_empty() {
                urls.clear();
            } else {
                urls.push(value.clone());
            }
        }
        urls
    };
    let rewrite = |url: &str, suffix: &str| -> Option<String> {
        let mut groups: Vec<(&str, Vec<&str>)> = Vec::new();
        for (key, prefix) in records {
            let Some(base) = key
                .strip_prefix("url.")
                .and_then(|key| key.strip_suffix(suffix))
            else {
                continue;
            };
            if let Some((_, prefixes)) = groups.iter_mut().find(|(existing, _)| *existing == base) {
                prefixes.push(prefix);
            } else {
                groups.push((base, vec![prefix]));
            }
        }
        let mut longest: Option<(&str, &str)> = None;
        for (base, prefixes) in groups {
            for prefix in prefixes {
                if url.starts_with(prefix)
                    && longest.is_none_or(|(_, old)| old.len() < prefix.len())
                {
                    longest = Some((base, prefix));
                }
            }
        }
        longest.map(|(base, prefix)| format!("{base}{}", &url[prefix.len()..]))
    };
    let mut urls = list("url");
    let explicit = list("pushurl");
    if urls.is_empty() {
        urls.push(remote.to_owned());
    }
    let result = if push && !explicit.is_empty() {
        explicit
            .into_iter()
            .map(|url| rewrite(&url, ".insteadof").unwrap_or(url))
            .collect()
    } else if push {
        let mut rewritten = Vec::new();
        for url in &urls {
            if let Some(alias) = rewrite(url, ".pushinsteadof") {
                // Git feeds generated aliases through add_pushurl(): an
                // empty replacement clears earlier generated destinations.
                if alias.is_empty() {
                    rewritten.clear();
                } else {
                    rewritten.push(alias);
                }
            }
        }
        if rewritten.is_empty() {
            urls.into_iter()
                .map(|url| rewrite(&url, ".insteadof").unwrap_or(url))
                .collect()
        } else {
            rewritten
        }
    } else {
        urls.into_iter()
            .take(1)
            .map(|url| rewrite(&url, ".insteadof").unwrap_or(url))
            .collect()
    };
    if result.is_empty()
        || result.len() > 64
        || result.iter().any(|url| {
            url.len() > 16 * 1024 || url.bytes().any(|byte| matches!(byte, 0 | b'\n' | b'\r'))
        })
    {
        return Err(invalid());
    }
    Ok(result)
}

/// Extract only remote-related records while preserving host declaration order.
pub(super) fn records(output: &[u8]) -> Result<Vec<(String, String)>, GitToolError> {
    if output.is_empty() {
        return Ok(Vec::new());
    }
    let fields: Vec<_> = output
        .strip_suffix(&[0])
        .ok_or_else(invalid)?
        .split(|byte| *byte == 0)
        .collect();
    let (pairs, tail) = fields.as_chunks::<2>();
    if !tail.is_empty() {
        return Err(invalid());
    }
    let mut records = Vec::new();
    for [_, record] in pairs {
        let end = record
            .iter()
            .position(|byte| *byte == b'\n')
            .unwrap_or(record.len());
        let mut key = record[..end].to_vec();
        let first = key
            .iter()
            .position(|byte| *byte == b'.')
            .ok_or_else(invalid)?;
        let last = key
            .iter()
            .rposition(|byte| *byte == b'.')
            .ok_or_else(invalid)?;
        key[..first].make_ascii_lowercase();
        key[last + 1..].make_ascii_lowercase();
        if (key.starts_with(b"remote.") && matches!(&key[last + 1..], b"url" | b"pushurl"))
            || (key.starts_with(b"url.")
                && matches!(&key[last + 1..], b"insteadof" | b"pushinsteadof"))
        {
            let value = record.get(end + 1..).ok_or_else(invalid)?;
            records.push((
                String::from_utf8(key).map_err(|_| invalid())?,
                String::from_utf8(value.to_vec()).map_err(|_| invalid())?,
            ));
        }
    }
    Ok(records)
}

#[test]
fn legacy_remote_destinations_cannot_silently_bypass_config_resolution() {
    let root = tempfile::tempdir().expect("fixture");
    reject_legacy_remotes(root.path()).expect("no legacy files");
    std::fs::create_dir(root.path().join("remotes")).expect("directory");
    reject_legacy_remotes(root.path()).expect("empty legacy directory");
    std::fs::write(
        root.path().join("remotes/origin"),
        "URL: git://other.invalid/repo\n",
    )
    .expect("legacy remote");
    assert!(
        reject_legacy_remotes(root.path())
            .expect_err("undeclared destination")
            .to_string()
            .contains("Legacy Git remote")
    );
}

#[test]
fn destination_records_preserve_subsection_case_and_ignore_boolean_options() {
    let parsed = records(b"file:/config\0remote.Case.mirror\0file:/config\0Remote.Case.URL\ngit://example.invalid/repo\0").expect("records");
    assert_eq!(
        parsed,
        [(
            "remote.Case.url".into(),
            "git://example.invalid/repo".into()
        )]
    );
    assert_eq!(
        destinations(&parsed, "Case", true).expect("resolve"),
        ["git://example.invalid/repo"]
    );
    assert!(records(b"file:/config\0remote.Case.url\0").is_err());
}

#[test]
fn equal_length_rewrites_follow_base_group_order_in_host_git() {
    let root = tempfile::tempdir().expect("fixture");
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(root.path())
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .output()
            .expect("git");
        assert!(output.status.success());
        String::from_utf8(output.stdout).expect("utf8")
    };
    git(&["init"]);
    let config = "[remote \"origin\"]\n url = same:repo\n[url \"git://first.invalid/\"]\n pushInsteadOf = unrelated:\n[url \"git://second.invalid/\"]\n pushInsteadOf = same:\n[url \"git://first.invalid/\"]\n pushInsteadOf = same:\n";
    std::fs::write(root.path().join(".git/config"), config).expect("config");
    let output = git(&["config", "--null", "--show-origin", "--list"]);
    let resolved = destinations(
        &records(output.as_bytes()).expect("records"),
        "origin",
        true,
    )
    .expect("resolve");
    assert_eq!(
        resolved,
        git(&["remote", "get-url", "--push", "--all", "origin"])
            .lines()
            .collect::<Vec<_>>()
    );
    assert_eq!(resolved, ["git://first.invalid/repo"]);
}

#[test]
fn rewritten_push_destinations_match_host_without_network() {
    let root = tempfile::tempdir().expect("fixture");
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(root.path())
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .output()
            .expect("git");
        assert!(output.status.success(), "fixture Git failed");
        String::from_utf8(output.stdout).expect("utf8")
    };
    git(&["init"]);
    let mut records = Vec::new();
    for (key, value) in [
        ("remote.origin.url", "short:one"),
        ("remote.origin.url", "other:two"),
        ("url.git://read.invalid/.insteadof", "short:"),
        ("url.git://write.invalid/.pushinsteadof", "short:"),
        ("remote.origin.pushurl", "short:explicit"),
        ("remote.origin.pushurl", "short:second"),
        ("remote.origin.pushurl", ""),
        ("url.git://specific.invalid/.pushinsteadof", "short:o"),
    ] {
        git(&["config", "--add", key, value]);
        records.push((key.to_owned(), value.to_owned()));
        let expected = git(&["remote", "get-url", "--push", "--all", "origin"]);
        assert_eq!(
            destinations(&records, "origin", true).expect("resolve"),
            expected.lines().collect::<Vec<_>>()
        );
        let fetch = git(&["remote", "get-url", "origin"]);
        assert_eq!(
            destinations(&records, "origin", false).expect("fetch"),
            fetch.lines().collect::<Vec<_>>()
        );
    }
}

#[test]
fn empty_generated_push_alias_resets_previous_destinations_like_git() {
    let root = tempfile::tempdir().expect("fixture");
    assert!(
        std::process::Command::new("git")
            .args(["init", "-q"])
            .current_dir(root.path())
            .status()
            .expect("git")
            .success()
    );
    std::fs::write(root.path().join(".git/config"), "[remote \"origin\"]\n url = x:first\n url = reset\n url = keep:last\n[url \"git://example.invalid/\"]\n pushInsteadOf = x:\n[url \"\"]\n pushInsteadOf = reset\n").expect("config");
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(root.path())
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .output()
            .expect("git");
        assert!(output.status.success());
        output.stdout
    };
    let config = git(&["config", "--null", "--show-origin", "--list"]);
    let actual =
        destinations(&records(&config).expect("records"), "origin", true).expect("resolve");
    let expected =
        String::from_utf8(git(&["remote", "get-url", "--push", "--all", "origin"])).expect("utf8");
    assert_eq!(actual, expected.lines().collect::<Vec<_>>());
    assert_eq!(actual, ["x:first", "reset", "keep:last"]);
}

#[test]
fn direct_destinations_and_ambiguous_output() {
    let records = vec![(
        "url.git://write.invalid/.pushinsteadof".into(),
        "short:".into(),
    )];
    assert_eq!(
        destinations(&records, "short:repo", true).expect("direct rewrite"),
        ["git://write.invalid/repo"]
    );
    assert!(destinations(&[], "git://example.invalid/a\nb", true).is_err());
    assert!(destinations(&[], "--all", true).is_err());
}
