use super::*;
use talos_core::submission::{SubmissionItem, SubmissionKind, SubmissionSource};

fn submission() -> StructuredSubmission {
    StructuredSubmission {
        id: "batch-1".into(),
        source: SubmissionSource::User,
        sender_generation: 3,
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
    let dir = tempfile::tempdir().unwrap();
    let store =
        PendingSubmissionStore::for_session_file(&dir.path().join("session.tlog"), "session-1");
    let payload = submission();
    let (receipt, first) = store.accept(&payload).unwrap();
    assert!(!receipt.is_empty());
    assert_eq!(first, SubmissionReceiptDisposition::AcceptedPending);
    drop(store);

    let reopened =
        PendingSubmissionStore::for_session_file(&dir.path().join("session.tlog"), "session-1");
    let (same_receipt, second) = reopened.accept(&payload).unwrap();
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
    let dir = tempfile::tempdir().unwrap();
    let store =
        PendingSubmissionStore::for_session_file(&dir.path().join("session.tlog"), "session-1");
    let mut payload = submission();
    payload.items.clear();
    let (receipt, result) = store.accept(&payload).unwrap();
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
    let dir = tempfile::tempdir().unwrap();
    let store =
        PendingSubmissionStore::for_session_file(&dir.path().join("session.tlog"), "session-1");
    let payload = submission();
    store.accept(&payload).unwrap();
    let mut conflict = payload.clone();
    conflict.items[0].text = "different".into();
    let (_, result) = store.accept(&conflict).unwrap();
    assert_eq!(
        result,
        SubmissionReceiptDisposition::Rejected {
            reason: SubmissionRejectionReason::IdentityConflict,
        }
    );
}

#[test]
fn transcript_finalization_order_is_recoverable() {
    let dir = tempfile::tempdir().unwrap();
    let store =
        PendingSubmissionStore::for_session_file(&dir.path().join("session.tlog"), "session-1");
    let payload = submission();
    store.accept(&payload).unwrap();
    store.mark_running(&payload.id, "turn-1").unwrap();
    store.mark_committed(&payload.id, "turn-1").unwrap();
    let record = store.get(&payload.id).unwrap().unwrap();
    assert_eq!(record.state, PendingSubmissionState::Committed);
    assert_eq!(record.turn_id.as_deref(), Some("turn-1"));
    assert!(!record.payload_fingerprint.is_empty());
    assert!(store.recover_unstarted().unwrap().is_empty());
}

#[test]
fn unstarted_work_can_pause_and_recover_in_fifo_order() {
    let dir = tempfile::tempdir().unwrap();
    let store =
        PendingSubmissionStore::for_session_file(&dir.path().join("session.tlog"), "session-1");
    let first = submission();
    let mut second = submission();
    second.id = "batch-2".into();
    second.items[0].id = "item-2".into();
    second.items[0].enqueue_sequence = 2;
    store.accept(&first).unwrap();
    store.accept(&second).unwrap();
    assert_eq!(store.pause_unstarted().unwrap(), 2);
    let recovered = store.recover_unstarted().unwrap();
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
    let dir = tempfile::tempdir().unwrap();
    let store =
        PendingSubmissionStore::for_session_file(&dir.path().join("session.tlog"), "session-1");

    let mut oldest = submission();
    oldest.id = "oldest-batch".into();
    oldest.items[0].id = "oldest-item".into();
    let (oldest_receipt, accepted) = store.accept(&oldest).unwrap();
    assert_eq!(accepted, SubmissionReceiptDisposition::AcceptedPending);
    store.mark_running(&oldest.id, "turn-oldest").unwrap();
    store.mark_committed(&oldest.id, "turn-oldest").unwrap();

    for index in 0..MAX_TOMBSTONES {
        let mut payload = submission();
        payload.id = format!("later-batch-{index}");
        payload.items[0].id = format!("later-item-{index}");
        payload.items[0].enqueue_sequence = index as u64 + 2;
        let (_, disposition) = store.accept(&payload).unwrap();
        assert_eq!(disposition, SubmissionReceiptDisposition::AcceptedPending);
        let turn_id = format!("later-turn-{index}");
        store.mark_running(&payload.id, &turn_id).unwrap();
        store.mark_committed(&payload.id, &turn_id).unwrap();
    }

    assert!(
        store.get(&oldest.id).unwrap().is_none(),
        "the oldest large terminal payload must be pruned after the bound is exceeded"
    );

    let (replay_receipt, replay) = store.accept(&oldest).unwrap();
    assert_eq!(replay_receipt, oldest_receipt);
    assert_eq!(
        replay,
        SubmissionReceiptDisposition::AlreadyAccepted {
            state: PendingSubmissionState::Committed,
            turn_id: Some("turn-oldest".into()),
        }
    );
    assert!(
        store.get(&oldest.id).unwrap().is_none(),
        "an idempotent replay must not recreate a pruned payload row"
    );

    let mut conflict = oldest.clone();
    conflict.items[0].text = "conflicting delayed retry".into();
    let (conflict_receipt, conflict_result) = store.accept(&conflict).unwrap();
    assert_eq!(conflict_receipt, oldest_receipt);
    assert_eq!(
        conflict_result,
        SubmissionReceiptDisposition::Rejected {
            reason: SubmissionRejectionReason::IdentityConflict,
        }
    );
}
