use talos_core::message::Message;
use talos_session::{PendingSubmissionStore, SessionManager, SessionRuntimeIdentity};

#[test]
fn session_runtime_identity_survives_production_scale_compaction() {
    let temp_dir = tempfile::tempdir().expect("operation should succeed");
    let manager = SessionManager::with_dir(temp_dir.path().to_path_buf());
    let session = manager
        .create_session("runtime-identity", "")
        .expect("operation should succeed");
    let high = SessionRuntimeIdentity::new("openai", "o3", Some("high-reasoning"));
    PendingSubmissionStore::for_session(&session)
        .initialize_runtime_identity(high.clone())
        .expect("operation should succeed");

    for index in 0..205 {
        session
            .append(&Message::User {
                content: format!("turn-{index}"),
            })
            .expect("operation should succeed");
    }
    session
        .compact_archived(50)
        .expect("operation should succeed");
    session
        .compact_archived(50)
        .expect("operation should succeed");

    let reopened = PendingSubmissionStore::for_session(&session)
        .runtime_state()
        .expect("operation should succeed")
        .expect("operation should succeed");
    assert_eq!(reopened.activation.target, high);
    assert_eq!(
        reopened.status,
        talos_session::SessionRuntimeActivationStatus::Committed
    );
}
