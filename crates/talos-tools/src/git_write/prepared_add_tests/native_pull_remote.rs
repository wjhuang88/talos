//! Legacy remote content uses the same frozen auxiliary frontier as child hooks.

#![cfg(unix)]

use super::*;

#[tokio::test]
async fn native_pull_linked_legacy_source_requires_actual_target_admission() {
    use std::os::unix::fs::symlink;
    let root = tempfile::tempdir().expect("fixture");
    git(root.path(), &["init", "-b", "main"]);
    git(root.path(), &["config", "core.hooksPath", ""]);
    git(
        root.path(),
        &["remote", "add", "origin", "git://example.invalid/parent"],
    );
    let module = root
        .path()
        .canonicalize()
        .expect("root")
        .join(".git/modules/child");
    std::fs::create_dir_all(&module).expect("module");
    git(&module, &["init", "--bare", "-b", "main"]);
    git(&module, &["config", "core.hooksPath", ""]);
    git(&module, &["config", "branch.main.remote", "upstream"]);
    let outside = tempfile::tempdir().expect("outside");
    let source = outside
        .path()
        .canonicalize()
        .expect("outside root")
        .join("remote");
    std::fs::write(&source, "URL: git://example.invalid/child\n").expect("legacy source");
    std::fs::create_dir(module.join("remotes")).expect("legacy directory");
    let alias = module.join("remotes/upstream");
    symlink(&source, &alias).expect("external source link");
    let mut plan = Box::new(
        prepare_pull_fixture(root.path(), &serde_json::json!({}), ToolProvenance::Native)
            .expect("plan"),
    )
    .discover()
    .await
    .expect("sender");
    plan.phase = Phase::PullNativePreparation;
    for _ in 0..10 {
        if plan
            .facets
            .iter()
            .any(|facet| facet.resource.as_deref() == source.to_str())
        {
            break;
        }
        let tokens = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("proof");
        plan = plan
            .discover_native_pull_scope(&tokens)
            .await
            .expect("scope");
    }
    assert!(
        plan.facets
            .iter()
            .any(|facet| facet.resource.as_deref() == source.to_str())
    );
    assert!(plan.native_pull_remotes.is_empty());
    assert!(
        plan.auxiliary_paths
            .as_ref()
            .expect("auxiliary")
            .leaf_evidence(&alias)
            .is_none()
    );
    let tokens = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("proof");
    let mut denied = tokens.clone();
    denied.retain(|token| {
        !token.authorizes_path(
            "git_pull",
            ToolNature::Read,
            &plan.workspace,
            source.to_str().expect("source"),
        )
    });
    assert!(
        !plan
            .discover_native_pull_remotes(&denied)
            .await
            .expect("unapproved target frontier")
    );
    assert!(plan.native_pull_remotes.is_empty());
    assert!(
        plan.auxiliary_paths
            .as_ref()
            .expect("auxiliary")
            .leaf_evidence(&alias)
            .is_none()
    );
    assert!(
        plan.discover_native_pull_remotes(&tokens)
            .await
            .expect("fresh target read")
    );
    assert_eq!(
        plan.native_pull_remotes[&module].destination,
        "git://example.invalid/child"
    );
    std::fs::write(&source, "URL: git://example.invalid/changed\n").expect("source drift");
    assert!(
        plan.validate_native_pull_remote(&module, "upstream", &tokens)
            .await
            .is_err()
    );
    assert!(!root.path().join(".git/FETCH_HEAD").exists());
    assert!(!module.join("FETCH_HEAD").exists());
}
