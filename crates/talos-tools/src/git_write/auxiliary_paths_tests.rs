use super::*;
use talos_core::tool::ToolAuthorizationScope;

fn capability(tool: &str) -> ToolExecutionAuthorization {
    ToolExecutionAuthorization::for_git_auxiliary_path_resolution(
        tool,
        ToolAuthorizationScope::Once,
    )
}

fn read(tool: &str, workspace: &Path, target: &Path) -> ToolExecutionAuthorization {
    ToolExecutionAuthorization::for_path(
        tool,
        ToolNature::Read,
        workspace,
        target.to_str().expect("UTF-8 fixture path"),
        ToolAuthorizationScope::Once,
    )
    .expect("path authorization")
}

#[test]
fn resolution_requires_capability_before_traversal_and_target_read_before_content() {
    let workspace = tempfile::tempdir().expect("workspace");
    let root = workspace
        .path()
        .canonicalize()
        .expect("canonical workspace");
    let source = root.join("external/config");
    std::fs::create_dir_all(source.parent().expect("parent")).expect("directory");
    std::fs::write(&source, b"contents").expect("source");
    let anchor = root.join(".git");
    let anchor_read = read("git_add", &root, &anchor);
    assert!(AuxiliaryPaths::resolve("git_add", &root, &anchor, [source.clone()], &[]).is_err());

    let mut paths = AuxiliaryPaths::resolve(
        "git_add",
        &root,
        &anchor,
        [source.clone()],
        &[capability("git_add"), anchor_read.clone()],
    )
    .expect("metadata-only resolution");
    assert!(
        paths
            .capture_admitted(&[capability("git_add"), anchor_read.clone()], &[])
            .is_err()
    );
    let mut authority = vec![capability("git_add"), anchor_read];
    authority.extend(
        paths
            .related_paths()
            .map(|path| read("git_add", &root, path)),
    );
    paths
        .capture_admitted(&authority, &[])
        .expect("separate exact target Read");
    paths.revalidate(&authority).expect("retained source");
    assert!(
        paths
            .revalidate(&[capability("git_add"), read("git_add", &root, &anchor)])
            .is_err(),
        "metadata authority cannot re-read a previously captured file body"
    );
}

#[cfg(unix)]
#[test]
fn local_external_link_is_resolved_and_replacement_is_rejected() {
    use std::os::unix::fs::symlink;
    let workspace = tempfile::tempdir().expect("workspace");
    let outside = tempfile::tempdir().expect("external local directory");
    let root = workspace.path().canonicalize().expect("workspace root");
    let external = outside.path().canonicalize().expect("external root");
    let target = external.join("config");
    std::fs::write(&target, b"same bytes").expect("target");
    let alias = root.join("config-link");
    symlink(&target, &alias).expect("external local symlink");

    let anchor = root.join(".git");
    let anchor_read = read("git_add", &root, &anchor);
    let mut paths = AuxiliaryPaths::resolve(
        "git_add",
        &root,
        &anchor,
        [alias.clone()],
        &[capability("git_add"), anchor_read.clone()],
    )
    .expect("bounded local alias resolution");
    assert_eq!(paths.targets().next(), Some(target.as_path()));
    assert!(paths.original_leaf_is_link(&alias));
    let mut authority = vec![capability("git_add"), anchor_read];
    authority.extend(
        paths
            .related_paths()
            .map(|path| read("git_add", &root, path)),
    );
    paths
        .capture_admitted(&authority, &[])
        .expect("target Read");

    let retained = external.join("retained");
    std::fs::rename(&target, &retained).expect("retain old file identity");
    std::fs::write(&target, b"same bytes").expect("replace with equal bytes");
    assert!(paths.revalidate(&authority).is_err());

    symlink("//remote/share/config", root.join("network-link")).expect("network target link");
    assert!(
        AuxiliaryPaths::resolve(
            "git_add",
            &root,
            &anchor,
            [root.join("network-link")],
            &[capability("git_add"), read("git_add", &root, &anchor)]
        )
        .is_err()
    );
}

#[cfg(unix)]
#[test]
fn prior_link_read_does_not_authorize_a_redirected_target() {
    use std::os::unix::fs::symlink;
    let workspace = tempfile::tempdir().expect("workspace");
    let outside = tempfile::tempdir().expect("external directory");
    let root = workspace.path().canonicalize().expect("root");
    let external = outside.path().canonicalize().expect("external root");
    let first = external.join("first");
    let second = external.join("second");
    std::fs::write(&first, b"same").expect("first");
    std::fs::write(&second, b"same").expect("second");
    let alias = root.join("attributes");
    symlink(&first, &alias).expect("initial link");
    let anchor = root.join(".git");
    // Path grants bind the canonical target at approval time.
    let mut authority = vec![
        capability("git_checkout"),
        read("git_checkout", &root, &anchor),
        read("git_checkout", &root, &alias),
    ];
    std::fs::remove_file(&alias).expect("remove link");
    symlink(&second, &alias).expect("redirect link");
    let mut paths =
        AuxiliaryPaths::resolve("git_checkout", &root, &anchor, [alias.clone()], &authority)
            .expect("metadata discovery");
    assert_eq!(paths.targets().next(), Some(second.as_path()));
    assert!(paths.capture_admitted(&authority, &[]).is_err());
    assert!(paths.leaf_evidence(&alias).is_none());
    authority.push(read("git_checkout", &root, &second));
    paths
        .capture_admitted(&authority, &[])
        .expect("fresh target approval");
}

#[cfg(unix)]
#[test]
fn double_root_network_namespace_is_rejected() {
    assert!(
        super::pull_resolution::Resolution::capture_file(
            Path::new("//server/share/config"),
            &mut super::pull_resolution::Budget::default(),
        )
        .is_err()
    );
}

#[test]
fn continuation_keeps_budget_and_requires_new_target_read() {
    let workspace = tempfile::tempdir().expect("workspace");
    let root = workspace.path().canonicalize().expect("root");
    let anchor = root.join(".git");
    let first = root.join("first");
    let second = root.join("second");
    std::fs::write(&first, b"first").expect("first");
    std::fs::write(&second, b"second").expect("second");
    let authority = vec![
        capability("git_checkout"),
        read("git_checkout", &root, &anchor),
    ];
    let mut paths =
        AuxiliaryPaths::resolve("git_checkout", &root, &anchor, [first.clone()], &authority)
            .expect("first resolution");
    let remaining = paths.budget.entries;
    paths
        .extend("git_checkout", [first.clone()], &authority)
        .expect("existing source");
    assert_eq!(paths.budget.entries, remaining);
    paths
        .extend("git_checkout", [second.clone()], &authority)
        .expect("new source");
    assert!(paths.budget.entries < remaining);
    let mut admitted = authority.clone();
    admitted.push(read("git_checkout", &root, &first));
    assert!(paths.capture_admitted(&admitted, &[]).is_err());
    assert!(paths.sources.iter().all(|source| source.leaf.is_none()));
    admitted.push(read("git_checkout", &root, &second));
    paths
        .capture_admitted(&admitted, &[])
        .expect("complete frontier");
    paths.budget.entries = 0;
    assert!(
        paths
            .extend("git_checkout", [root.join("third")], &authority)
            .is_err()
    );
    assert_eq!(paths.budget.entries, 0);
}

#[test]
fn continuation_refuses_replaced_prior_source_before_extending() {
    let workspace = tempfile::tempdir().expect("workspace");
    let root = workspace.path().canonicalize().expect("root");
    let source = root.join("source");
    std::fs::write(&source, b"unchanged").expect("source");
    let anchor = root.join(".git");
    let authority = vec![
        capability("git_checkout"),
        read("git_checkout", &root, &anchor),
    ];
    let mut paths =
        AuxiliaryPaths::resolve("git_checkout", &root, &anchor, [source.clone()], &authority)
            .expect("initial resolution");
    let remaining = paths.budget.entries;
    std::fs::rename(&source, root.join("retained")).expect("retain old inode");
    std::fs::write(&source, b"unchanged").expect("replacement");
    assert!(
        paths
            .extend("git_checkout", [root.join("new")], &authority)
            .is_err()
    );
    assert_eq!(paths.sources.len(), 1);
    assert_eq!(paths.budget.entries, remaining);
}
