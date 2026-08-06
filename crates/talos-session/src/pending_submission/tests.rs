use std::sync::Arc;

use super::*;
use talos_core::submission::{SubmissionItem, SubmissionKind, SubmissionSource};

fn submission() -> StructuredSubmission {
    StructuredSubmission {
        id: "batch-1".into(),
        source: SubmissionSource::User,
        sender_generation: 0,
        items: vec![SubmissionItem {
            id: "item-1".into(),
            enqueue_sequence: 1,
            kind: SubmissionKind::UserTurn,
            text: "hello".into(),
            attachments: Vec::new(),
        }],
    }
}

#[test]
fn accept_is_durable_and_idempotent() {
    let dir = tempfile::tempdir().expect("operation should succeed");
    let store =
        PendingSubmissionStore::for_session_file(&dir.path().join("session.tlog"), "session-1");
    let payload = submission();
    let (receipt, first) = store.accept(&payload).expect("operation should succeed");
    assert!(!receipt.is_empty());
    assert_eq!(first, SubmissionReceiptDisposition::AcceptedPending);
    drop(store);

    let reopened =
        PendingSubmissionStore::for_session_file(&dir.path().join("session.tlog"), "session-1");
    let (same_receipt, second) = reopened.accept(&payload).expect("operation should succeed");
    assert_eq!(receipt, same_receipt);
    assert_eq!(
        second,
        SubmissionReceiptDisposition::AlreadyAccepted {
            state: PendingSubmissionState::AcceptedPending,
            turn_id: None,
        }
    );
}

#[test]
fn invalid_submission_does_not_create_custody() {
    let dir = tempfile::tempdir().expect("operation should succeed");
    let store =
        PendingSubmissionStore::for_session_file(&dir.path().join("session.tlog"), "session-1");
    let mut payload = submission();
    payload.items.clear();
    let (receipt, result) = store.accept(&payload).expect("operation should succeed");
    assert!(receipt.is_empty());
    assert_eq!(
        result,
        SubmissionReceiptDisposition::Rejected {
            reason: SubmissionRejectionReason::InvalidStructure,
        }
    );
    assert!(!store.path().exists());
}

#[test]
fn identity_conflict_fails_closed() {
    let dir = tempfile::tempdir().expect("operation should succeed");
    let store =
        PendingSubmissionStore::for_session_file(&dir.path().join("session.tlog"), "session-1");
    let payload = submission();
    store.accept(&payload).expect("operation should succeed");
    let mut conflict = payload.clone();
    conflict.items[0].text = "different".into();
    let (_, result) = store.accept(&conflict).expect("operation should succeed");
    assert_eq!(
        result,
        SubmissionReceiptDisposition::Rejected {
            reason: SubmissionRejectionReason::IdentityConflict,
        }
    );
}

#[test]
fn transcript_finalization_order_is_recoverable() {
    let dir = tempfile::tempdir().expect("operation should succeed");
    let store =
        PendingSubmissionStore::for_session_file(&dir.path().join("session.tlog"), "session-1");
    let payload = submission();
    store.accept(&payload).expect("operation should succeed");
    store
        .mark_running(&payload.id, "turn-1")
        .expect("operation should succeed");
    store
        .mark_committed(&payload.id, "turn-1")
        .expect("operation should succeed");
    let record = store
        .get(&payload.id)
        .expect("operation should succeed")
        .expect("operation should succeed");
    assert_eq!(record.state, PendingSubmissionState::Committed);
    assert_eq!(record.turn_id.as_deref(), Some("turn-1"));
    assert!(!record.payload_fingerprint.is_empty());
    assert!(
        store
            .recover_unstarted()
            .expect("operation should succeed")
            .is_empty()
    );
}

#[test]
fn unstarted_work_can_pause_and_recover_in_fifo_order() {
    let dir = tempfile::tempdir().expect("operation should succeed");
    let store =
        PendingSubmissionStore::for_session_file(&dir.path().join("session.tlog"), "session-1");
    let first = submission();
    let mut second = submission();
    second.id = "batch-2".into();
    second.items[0].id = "item-2".into();
    second.items[0].enqueue_sequence = 2;
    store.accept(&first).expect("operation should succeed");
    store.accept(&second).expect("operation should succeed");
    assert_eq!(
        store.pause_unstarted().expect("operation should succeed"),
        2
    );
    let recovered = store.recover_unstarted().expect("operation should succeed");
    assert_eq!(recovered.len(), 2);
    assert_eq!(recovered[0].submission.id, "batch-1");
    assert_eq!(recovered[1].submission.id, "batch-2");
    assert!(
        recovered
            .iter()
            .all(|record| record.state == PendingSubmissionState::PausedPending)
    );
}

#[test]
fn pruned_terminal_payload_retains_permanent_idempotency_identity() {
    let dir = tempfile::tempdir().expect("operation should succeed");
    let store =
        PendingSubmissionStore::for_session_file(&dir.path().join("session.tlog"), "session-1");

    let mut oldest = submission();
    oldest.id = "oldest-batch".into();
    oldest.items[0].id = "oldest-item".into();
    let (oldest_receipt, accepted) = store.accept(&oldest).expect("operation should succeed");
    assert_eq!(accepted, SubmissionReceiptDisposition::AcceptedPending);
    store
        .mark_running(&oldest.id, "turn-oldest")
        .expect("operation should succeed");
    store
        .mark_committed(&oldest.id, "turn-oldest")
        .expect("operation should succeed");

    for index in 0..MAX_TOMBSTONES {
        let mut payload = submission();
        payload.id = format!("later-batch-{index}");
        payload.items[0].id = format!("later-item-{index}");
        payload.items[0].enqueue_sequence = index as u64 + 2;
        let (_, disposition) = store.accept(&payload).expect("operation should succeed");
        assert_eq!(disposition, SubmissionReceiptDisposition::AcceptedPending);
        let turn_id = format!("later-turn-{index}");
        store
            .mark_running(&payload.id, &turn_id)
            .expect("operation should succeed");
        store
            .mark_committed(&payload.id, &turn_id)
            .expect("operation should succeed");
    }

    assert!(
        store
            .get(&oldest.id)
            .expect("operation should succeed")
            .is_none(),
        "the oldest large terminal payload must be pruned after the bound is exceeded"
    );

    let (replay_receipt, replay) = store.accept(&oldest).expect("operation should succeed");
    assert_eq!(replay_receipt, oldest_receipt);
    assert_eq!(
        replay,
        SubmissionReceiptDisposition::AlreadyAccepted {
            state: PendingSubmissionState::Committed,
            turn_id: Some("turn-oldest".into()),
        }
    );
    assert!(
        store
            .get(&oldest.id)
            .expect("operation should succeed")
            .is_none(),
        "an idempotent replay must not recreate a pruned payload row"
    );

    let mut conflict = oldest.clone();
    conflict.items[0].text = "conflicting delayed retry".into();
    let (conflict_receipt, conflict_result) =
        store.accept(&conflict).expect("operation should succeed");
    assert_eq!(conflict_receipt, oldest_receipt);
    assert_eq!(
        conflict_result,
        SubmissionReceiptDisposition::Rejected {
            reason: SubmissionRejectionReason::IdentityConflict,
        }
    );
}

#[test]
fn runtime_generation_survives_store_reopen_and_advances_atomically() {
    let dir = tempfile::tempdir().expect("operation should succeed");
    let session_file = dir.path().join("session.tlog");
    let store = PendingSubmissionStore::for_session_file(&session_file, "session-1");
    assert_eq!(
        store
            .runtime_generation()
            .expect("operation should succeed"),
        0
    );
    assert_eq!(
        store
            .advance_runtime_generation(0)
            .expect("operation should succeed"),
        1
    );
    drop(store);

    let reopened = PendingSubmissionStore::for_session_file(&session_file, "session-1");
    assert_eq!(
        reopened
            .runtime_generation()
            .expect("operation should succeed"),
        1
    );
    assert!(matches!(
        reopened.advance_runtime_generation(0),
        Err(PendingSubmissionError::GenerationConflict {
            expected: 0,
            actual: 1,
        })
    ));
    assert_eq!(
        reopened
            .advance_runtime_generation(1)
            .expect("operation should succeed"),
        2
    );
}

#[test]
fn generation_advance_requires_quiescent_custody_and_rejects_fresh_stale_work() {
    let dir = tempfile::tempdir().expect("operation should succeed");
    let store =
        PendingSubmissionStore::for_session_file(&dir.path().join("session.tlog"), "session-1");
    let retained = submission();
    assert_eq!(
        store.accept(&retained).expect("operation should succeed").1,
        SubmissionReceiptDisposition::AcceptedPending
    );
    assert!(matches!(
        store.advance_runtime_generation(0),
        Err(PendingSubmissionError::GenerationBusy {
            generation: 0,
            pending: 1,
        })
    ));
    assert_eq!(
        store
            .runtime_generation()
            .expect("operation should succeed"),
        0
    );

    store
        .cancel_unstarted(&retained.id)
        .expect("operation should succeed");
    assert_eq!(
        store
            .advance_runtime_generation(0)
            .expect("operation should succeed"),
        1
    );

    let mut stale = submission();
    stale.id = "fresh-stale-batch".into();
    stale.items[0].id = "fresh-stale-item".into();
    assert_eq!(
        store.accept(&stale).expect("operation should succeed").1,
        SubmissionReceiptDisposition::Rejected {
            reason: SubmissionRejectionReason::WrongGeneration,
        }
    );
    assert!(
        store
            .get(&stale.id)
            .expect("operation should succeed")
            .is_none()
    );

    let mut current = stale;
    current.id = "current-batch".into();
    current.items[0].id = "current-item".into();
    current.sender_generation = 1;
    assert_eq!(
        store.accept(&current).expect("operation should succeed").1,
        SubmissionReceiptDisposition::AcceptedPending
    );
}

#[test]
fn generation_fence_and_accept_are_serialized_across_store_instances() {
    let dir = tempfile::tempdir().expect("operation should succeed");
    let session_file = dir.path().join("session.tlog");
    let accept_store = PendingSubmissionStore::for_session_file(&session_file, "session-1");
    let fence_store = PendingSubmissionStore::for_session_file(&session_file, "session-1");
    let barrier = Arc::new(std::sync::Barrier::new(3));

    let accept_barrier = barrier.clone();
    let accept = std::thread::spawn(move || {
        let payload = submission();
        accept_barrier.wait();
        accept_store
            .accept(&payload)
            .expect("operation should succeed")
            .1
    });
    let fence_barrier = barrier.clone();
    let fence = std::thread::spawn(move || {
        fence_barrier.wait();
        fence_store.advance_runtime_generation(0)
    });
    barrier.wait();

    let disposition = accept.join().expect("operation should succeed");
    let advanced = fence.join().expect("operation should succeed");
    match (disposition, advanced) {
        (
            SubmissionReceiptDisposition::AcceptedPending,
            Err(PendingSubmissionError::GenerationBusy {
                generation: 0,
                pending: 1,
            }),
        )
        | (
            SubmissionReceiptDisposition::Rejected {
                reason: SubmissionRejectionReason::WrongGeneration,
            },
            Ok(1),
        ) => {}
        unexpected => panic!("accept/fence transaction escaped serialization: {unexpected:?}"),
    }
}

#[test]
fn prestart_cancel_terminalizes_identity_without_provider_turn() {
    let dir = tempfile::tempdir().expect("operation should succeed");
    let store =
        PendingSubmissionStore::for_session_file(&dir.path().join("session.tlog"), "session-1");
    let payload = submission();
    let (receipt_id, accepted) = store.accept(&payload).expect("operation should succeed");
    assert_eq!(accepted, SubmissionReceiptDisposition::AcceptedPending);
    store
        .mark_paused(&payload.id)
        .expect("operation should succeed");
    store
        .cancel_unstarted(&payload.id)
        .expect("operation should succeed");

    let record = store
        .get(&payload.id)
        .expect("operation should succeed")
        .expect("operation should succeed");
    assert_eq!(record.receipt_id, receipt_id);
    assert_eq!(record.state, PendingSubmissionState::TerminalCancelled);
    assert_eq!(record.turn_id, None);
    assert!(
        store
            .recover_unstarted()
            .expect("operation should succeed")
            .is_empty()
    );

    let (same_receipt, replay) = store.accept(&payload).expect("operation should succeed");
    assert_eq!(same_receipt, receipt_id);
    assert_eq!(
        replay,
        SubmissionReceiptDisposition::AlreadyAccepted {
            state: PendingSubmissionState::TerminalCancelled,
            turn_id: None,
        }
    );
}

#[test]
fn runtime_identity_survives_reopen_and_isolated_session_sidecars() {
    let dir = tempfile::tempdir().expect("operation should succeed");
    let first_path = dir.path().join("first.tlog");
    let second_path = dir.path().join("second.tlog");
    let first = PendingSubmissionStore::for_session_file(&first_path, "first");
    let second = PendingSubmissionStore::for_session_file(&second_path, "second");
    let high = SessionRuntimeIdentity::new("openai", "o3", Some("high-reasoning"));
    let low = SessionRuntimeIdentity::new("openai", "o3", Some("low-reasoning"));
    first
        .initialize_runtime_identity(high.clone())
        .expect("operation should succeed");
    second
        .initialize_runtime_identity(low.clone())
        .expect("operation should succeed");
    drop(first);
    drop(second);

    assert_eq!(
        PendingSubmissionStore::for_session_file(&first_path, "first")
            .runtime_state()
            .expect("operation should succeed")
            .expect("operation should succeed")
            .activation
            .target,
        high
    );
    assert_eq!(
        PendingSubmissionStore::for_session_file(&second_path, "second")
            .runtime_state()
            .expect("operation should succeed")
            .expect("operation should succeed")
            .activation
            .target,
        low
    );
}

#[test]
fn runtime_activation_stage_is_generation_atomic_and_restart_recoverable() {
    let dir = tempfile::tempdir().expect("operation should succeed");
    let path = dir.path().join("session.tlog");
    let store = PendingSubmissionStore::for_session_file(&path, "session-1");
    let low = SessionRuntimeIdentity::new("openai", "o3", Some("low-reasoning"));
    let high = SessionRuntimeIdentity::new("openai", "o3", Some("high-reasoning"));
    store
        .initialize_runtime_identity(low.clone())
        .expect("operation should succeed");
    let activation = SessionRuntimeActivation::new(1, low, high.clone());
    assert_eq!(
        store
            .stage_runtime_activation(0, &activation)
            .expect("operation should succeed"),
        1
    );
    drop(store);

    let reopened = PendingSubmissionStore::for_session_file(&path, "session-1");
    let pending = reopened
        .runtime_state()
        .expect("operation should succeed")
        .expect("operation should succeed");
    assert_eq!(
        pending.status,
        SessionRuntimeActivationStatus::PendingMarker
    );
    assert_eq!(pending.activation, activation);
    assert_eq!(
        reopened
            .runtime_generation()
            .expect("operation should succeed"),
        1
    );
    let committed = reopened
        .commit_runtime_activation(&pending.activation.activation_id)
        .expect("operation should succeed");
    assert_eq!(committed.status, SessionRuntimeActivationStatus::Committed);
    assert_eq!(committed.activation.target, high);
}

#[test]
fn default_identity_initialization_is_idempotent_but_conflicts_fail_closed() {
    let dir = tempfile::tempdir().expect("operation should succeed");
    let store =
        PendingSubmissionStore::for_session_file(&dir.path().join("session.tlog"), "session-1");
    let baseline = SessionRuntimeIdentity::new("openai", "o3", None);
    let default = SessionRuntimeIdentity::new("openai", "o3", Some(" DEFAULT "));
    let first = store
        .initialize_runtime_identity(baseline)
        .expect("operation should succeed");
    let replay = store
        .initialize_runtime_identity(default)
        .expect("operation should succeed");
    assert_eq!(first, replay);
    assert!(matches!(
        store.initialize_runtime_identity(SessionRuntimeIdentity::new(
            "openai",
            "o3",
            Some("high-reasoning")
        )),
        Err(PendingSubmissionError::RuntimeActivationConflict { .. })
    ));
}
