//! Opt-in independently resolved Cargo-root SDK contract for local storage.
//!
//! This crate imports no Talos internals: the only direct Talos dependency
//! remains talos-runtime. The default and coding fixture modes stay unchanged.

use std::sync::Arc;

use anyhow::Result;
use talos_runtime::{
    RuntimeBuilder, RuntimeTurnCompletionStatus, SessionError, collect_until_turn_completed,
    local_storage::LocalStorageProfile,
};

use crate::provider::FixtureProvider;

pub(crate) async fn verify() -> Result<()> {
    let invalid_home = tempfile::tempdir()?;
    let invalid_profile = LocalStorageProfile::under_home_dir(invalid_home.path());
    let invalid = RuntimeBuilder::new()
        .durable_session_from_custom_local_profile(&invalid_profile, "../outside");
    assert!(matches!(invalid, Err(SessionError::InvalidExternalId(_))));
    assert!(
        !invalid_profile.talos_root().exists(),
        "invalid host identity must not create any local storage"
    );

    let home = tempfile::tempdir()?;
    let profile = LocalStorageProfile::under_home_dir(home.path());
    let manager = profile.custom_session_manager();
    assert!(!profile.talos_root().exists());
    let selected = manager.create_or_open_session("external-e3a-sdk-host")?;
    let session_id = selected.id();
    let tlog_path = selected.file_path().to_path_buf();

    // The prior public durable-session signature remains source compatible.
    let _legacy_builder = RuntimeBuilder::new().durable_session(selected);

    let mut runtime = RuntimeBuilder::new()
        .provider(Arc::new(
            FixtureProvider::new().with_response("external local storage turn"),
        ))
        .durable_session_from_custom_local_profile(&profile, "external-e3a-sdk-host")?
        .build_async()
        .await?;
    runtime.submit("persist this external SDK turn").await?;
    let status = collect_until_turn_completed(&mut runtime)
        .await
        .ok_or_else(|| anyhow::anyhow!("SDK local-storage turn ended before completion"))?;
    assert!(matches!(status, RuntimeTurnCompletionStatus::Success { .. }));
    runtime.shutdown().await?;

    let reopened = manager.create_or_open_session("external-e3a-sdk-host")?;
    assert_eq!(reopened.id(), session_id);
    assert_eq!(reopened.file_path(), tlog_path.as_path());
    let messages = reopened.read_messages()?;
    assert!(
        serde_json::to_string(&messages)?.contains("external local storage turn"),
        "public feature-on SDK selected an existing durable TLOG transcript"
    );
    assert!(profile.sessions_dir().join("durable-bindings.sqlite").is_file());
    assert!(
        !profile.memory_db().exists()
            && !profile.exploration_db().exists()
            && !profile.sessions_dir().join("todos.sqlite").exists(),
        "SDK Session selection must not eagerly open optional databases"
    );
    Ok(())
}
