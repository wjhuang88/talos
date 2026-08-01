use super::*;
use talos_core::submission::{SubmissionItem, SubmissionKind, SubmissionSource};

fn submission() -> StructuredSubmission {
    StructuredSubmission {
        batch_id: "batch-1".into(),
        reservation_id: "reservation-1".into(),
        transfer_attempt_id: "attempt-1".into(),
        session_id: "session-1".into(),
        session_generation: 3,
        source: SubmissionSource::User,
        items: vec![SubmissionItem {
            item_id: "item-1".into(),
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
    let store = PendingSubmissionStore::for_session_file(
        &dir.path().join("session.tlog"),
        "session-1",
    );
    let payload = submission();
    let (receipt, first) = store.accept(&payload).unwrap();
    assert!(!receipt.is_empty());
    assert_eq!(first, SubmissionReceiptDisposition::AcceptedPending);
    drop(store);

    let reopened = PendingSubmissionStore::for_session_file(
        &dir.path().join("session.tlog"),
        "session-1",
    );
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
    let store = PendingSubmissionStore::for_session_file(
        &dir.path().join("session.tlog"),
        "session-1",
    );
    let mut payload = submission();
    payload.session_generation = 0;
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
    let store = PendingSubmissionStore::for_session_file(
        &dir.path().join("session.tlog"),
        "session-1",
    );
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
    let store = PendingSubmissionStore::for_session_file(
        &dir.path().join("session.tlog"),
        "session-1",
    );
    let payload = submission();
    store.accept(&payload).unwrap();
    store.mark_running(&payload.batch_id, "turn-1").unwrap();
    store.mark_committed(&payload.batch_id, "turn-1").unwrap();
    let record = store.get(&payload.batch_id).unwrap().unwrap();
    assert_eq!(record.state, PendingSubmissionState::Committed);
    assert_eq!(record.turn_id.as_deref(), Some("turn-1"));
    assert!(!record.payload_fingerprint.is_empty());
    assert!(store.recover_unstarted().unwrap().is_empty());
}

#[test]
fn unstarted_work_can_pause_and_recover_in_fifo_order() {
    let dir = tempfile::tempdir().unwrap();
    let store = PendingSubmissionStore::for_session_file(
        &dir.path().join("session.tlog"),
        "session-1",
    );
    let first = submission();
    let mut second = submission();
    second.batch_id = "batch-2".into();
    second.reservation_id = "reservation-2".into();
    second.transfer_attempt_id = "attempt-2".into();
    second.items[0].item_id = "item-2".into();
    second.items[0].enqueue_sequence = 2;
    store.accept(&first).unwrap();
    store.accept(&second).unwrap();
    assert_eq!(store.pause_unstarted().unwrap(), 2);
    let recovered = store.recover_unstarted().unwrap();
    assert_eq!(recovered.len(), 2);
    assert_eq!(recovered[0].submission.batch_id, "batch-1");
    assert_eq!(recovered[1].submission.batch_id, "batch-2");
    assert!(recovered
        .iter()
        .all(|record| record.state == PendingSubmissionState::PausedPending));
}
