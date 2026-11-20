//! I334 E3a — externally compiled public SDK selector contract (workspace Cargo root).
//!
//! Unlike the independent Cargo.lock consumer fixture, this integration test
//! compiles as a separate downstream Rust crate within the workspace feature
//! resolution. It must not be cited as independent-Cargo dependency proof.
#![cfg(feature = "local-storage")]

use talos_runtime::{RuntimeBuilder, SessionError, local_storage::LocalStorageProfile};

#[test]
fn explicit_public_sdk_local_host_selector_reuses_domain_session_and_legacy_api() {
    let home = tempfile::tempdir().expect("isolated external SDK home");
    let profile = LocalStorageProfile::under_home_dir(home.path());
    let root = profile.talos_root();
    let _legacy_unbound = RuntimeBuilder::new();
    assert!(!root.exists(), "unbound SDK must not create local storage");

    let _selected = RuntimeBuilder::new()
        .durable_session_from_custom_local_profile(&profile, "external-e3a-host")
        .expect("public feature-on local Session selection");
    let manager = profile.custom_session_manager();
    let direct = manager
        .create_or_open_session("external-e3a-host")
        .expect("existing direct Session binding");
    let reopened = manager
        .create_or_open_session("external-e3a-host")
        .expect("same host identity");
    assert_eq!(direct.id(), reopened.id());
    assert_eq!(direct.file_path(), reopened.file_path());
    let durable_root = profile.sessions_dir().join("durable");
    assert_eq!(reopened.file_path().parent(), Some(durable_root.as_path()));
    assert!(profile.sessions_dir().join("durable-bindings.sqlite").is_file());
    assert!(
        !profile.memory_db().exists()
            && !profile.exploration_db().exists()
            && !profile.sessions_dir().join("todos.sqlite").exists(),
        "explicit durable binding must not eagerly open optional domain databases"
    );

    // This public legacy signature is still available in the same downstream crate.
    let _legacy_direct = RuntimeBuilder::new().durable_session(reopened);
}

#[test]
fn public_sdk_selector_invalid_id_does_not_create_a_directory() {
    let home = tempfile::tempdir().expect("isolated invalid-id home");
    let profile = LocalStorageProfile::under_home_dir(home.path());
    let selected = RuntimeBuilder::new()
        .durable_session_from_custom_local_profile(&profile, "../escape");
    assert!(matches!(selected, Err(SessionError::InvalidExternalId(_))));
    assert!(
        !profile.talos_root().exists(),
        "invalid external identity must fail before creating any selected storage"
    );
}
