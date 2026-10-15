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
fn absent_pause_creates_no_storage_and_later_admission_remains_durable() {
    let dir = tempfile::tempdir().expect("temporary directory");
    let session_file = dir.path().join("missing/deeper/session.tlog");
    let store = PendingSubmissionStore::for_session_file(&session_file, "session-1");
    assert_eq!(store.pause_unstarted().expect("empty pause"), 0);
    assert!(!dir.path().join("missing").exists());
    let payload = submission();
    let (receipt, disposition) = store.accept(&payload).expect("later admission");
    assert_eq!(disposition, SubmissionReceiptDisposition::AcceptedPending);
    drop(store);
    let reopened = PendingSubmissionStore::for_session_file(&session_file, "session-1");
    let records = reopened.recover_unstarted().expect("durable recovery");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].receipt_id, receipt);
    assert_eq!(records[0].submission, payload);
    assert_eq!(records[0].state, PendingSubmissionState::AcceptedPending);
    assert_eq!(reopened.pause_unstarted().expect("pause admitted work"), 1);
}

#[test]
fn pause_changes_only_existing_accepted_pending_and_preserves_receipts() {
    let dir = tempfile::tempdir().expect("temporary directory");
    let session_file = dir.path().join("session.tlog");
    let store = PendingSubmissionStore::for_session_file(&session_file, "session-1");
    let accepted = submission();
    let mut running = submission();
    running.id = "running".into();
    running.items[0].id = "running-item".into();
    let mut paused = submission();
    paused.id = "paused".into();
    paused.items[0].id = "paused-item".into();
    for payload in [&accepted, &running, &paused] {
        store.accept(payload).expect("accept");
    }
    store
        .mark_running(&running.id, "turn-running")
        .expect("start");
    store.mark_paused(&paused.id).expect("prior pause");
    let mut expected = store.get(&accepted.id).expect("read").expect("accepted");
    let running_before = store.get(&running.id).expect("read");
    let paused_before = store.get(&paused.id).expect("read");
    expected.state = PendingSubmissionState::PausedPending;
    assert_eq!(store.pause_unstarted().expect("pause"), 1);
    assert_eq!(store.pause_unstarted().expect("repeat pause"), 0);
    drop(store);
    let reopened = PendingSubmissionStore::for_session_file(&session_file, "session-1");
    assert_eq!(
        reopened.get(&accepted.id).expect("read"),
        Some(expected.clone())
    );
    assert_eq!(reopened.get(&running.id).expect("read"), running_before);
    assert_eq!(reopened.get(&paused.id).expect("read"), paused_before);
    assert_eq!(
        reopened.recover_unstarted().expect("FIFO"),
        vec![expected, paused_before.expect("paused")]
    );
    assert_eq!(
        reopened.recover_running().expect("running"),
        vec![running_before.expect("running")]
    );
}

#[test]
fn pause_and_cross_instance_admission_keep_acknowledged_work() {
    let dir = tempfile::tempdir().expect("temporary directory");
    let session_file = dir.path().join("missing/session.tlog");
    let reader = PendingSubmissionStore::for_session_file(&session_file, "session-1");
    assert_eq!(reader.pause_unstarted().expect("absent snapshot"), 0);
    assert!(!dir.path().join("missing").exists());
    let writer = PendingSubmissionStore::for_session_file(&session_file, "session-1");
    let barrier = Arc::new(std::sync::Barrier::new(2));
    let ready = Arc::clone(&barrier);
    let admission = std::thread::spawn(move || {
        ready.wait();
        writer.accept(&submission()).expect("concurrent admission")
    });
    barrier.wait();
    let changed = reader.pause_unstarted().expect("concurrent pause");
    let (receipt, disposition) = admission.join().expect("writer joins");
    assert_eq!(disposition, SubmissionReceiptDisposition::AcceptedPending);
    let reopened = PendingSubmissionStore::for_session_file(&session_file, "session-1");
    let records = reopened.recover_unstarted().expect("acknowledged custody");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].receipt_id, receipt);
    assert_eq!(records[0].submission, submission());
    assert_eq!(
        records[0].state,
        match changed {
            0 => PendingSubmissionState::AcceptedPending,
            1 => PendingSubmissionState::PausedPending,
            other => panic!("unexpected changed count: {other}"),
        }
    );
    assert_eq!(
        reopened.pause_unstarted().expect("post-ack pause"),
        1 - changed
    );
    assert_eq!(
        reopened
            .get(&submission().id)
            .expect("read")
            .expect("record")
            .state,
        PendingSubmissionState::PausedPending
    );
}

#[test]
fn pause_preserves_invalid_parent_malformed_journal_and_schema_errors() {
    let dir = tempfile::tempdir().expect("temporary directory");
    let parent = dir.path().join("file");
    std::fs::write(&parent, "sentinel").expect("file fixture");
    let invalid =
        PendingSubmissionStore::for_session_file(&parent.join("session.tlog"), "session-1");
    assert!(matches!(
        invalid.pause_unstarted(),
        Err(PendingSubmissionError::Io(_))
    ));
    assert_eq!(
        std::fs::read_to_string(&parent).expect("unchanged"),
        "sentinel"
    );
    let malformed =
        PendingSubmissionStore::for_session_file(&dir.path().join("session.tlog"), "malformed");
    std::fs::write(malformed.path(), "not SQLite").expect("malformed fixture");
    assert!(matches!(
        malformed.pause_unstarted(),
        Err(PendingSubmissionError::Storage(_))
    ));
    let unsupported =
        PendingSubmissionStore::for_session_file(&dir.path().join("session.tlog"), "unsupported");
    let connection = unsupported.connection().expect("connection");
    ensure_schema(&connection).expect("schema");
    connection
        .execute(
            "UPDATE pending_journal_meta SET value = 999 WHERE key = 'schema_version'",
            [],
        )
        .expect("unsupported fixture");
    drop(connection);
    assert!(matches!(
        unsupported.pause_unstarted(),
        Err(PendingSubmissionError::UnsupportedSchema(999))
    ));
}

#[test]
fn pause_parentdir_path_cannot_hide_previously_acknowledged_custody() {
    let dir = tempfile::tempdir().expect("temporary directory");
    let store =
        PendingSubmissionStore::for_session_file(&dir.path().join("session.tlog"), "session-1");
    let (receipt, _) = store.accept(&submission()).expect("canonical admission");
    let missing = dir.path().join("missing");
    let alternate =
        PendingSubmissionStore::for_session_file(&missing.join("../session.tlog"), "session-1");
    assert_eq!(alternate.pause_unstarted().expect("conservative pause"), 1);
    let records = store.recover_unstarted().expect("recovery");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].receipt_id, receipt);
    assert_eq!(records[0].submission, submission());
    assert_eq!(records[0].state, PendingSubmissionState::PausedPending);
}

#[test]
fn pause_parentdir_empty_path_retains_sqlite_initialization() {
    let dir = tempfile::tempdir().expect("temporary directory");
    let missing = dir.path().join("missing");
    let store =
        PendingSubmissionStore::for_session_file(&missing.join("../session.tlog"), "session-1");
    assert_eq!(
        store.pause_unstarted().expect("conservative empty pause"),
        0
    );
    assert!(dir.path().join("session-1.pending.sqlite").is_file());
    #[cfg(unix)]
    assert!(missing.is_dir());
}

#[test]
fn absent_pause_preserves_poisoned_guard_error() {
    let dir = tempfile::tempdir().expect("temporary directory");
    let store = PendingSubmissionStore::for_session_file(
        &dir.path().join("missing/session.tlog"),
        "session-1",
    );
    let lock = Arc::clone(&store.lock);
    assert!(
        std::thread::spawn(move || {
            let _guard = lock.lock().expect("guard");
            panic!("poison fixture");
        })
        .join()
        .is_err()
    );
    assert!(matches!(
        store.pause_unstarted(),
        Err(PendingSubmissionError::LockPoisoned)
    ));
    assert!(!dir.path().join("missing").exists());
}

#[cfg(unix)]
#[test]
fn pause_preserves_symlink_routes_and_dangling_ancestor_errors() {
    use std::os::unix::fs::symlink;
    let dir = tempfile::tempdir().expect("temporary directory");
    let store =
        PendingSubmissionStore::for_session_file(&dir.path().join("session.tlog"), "linked");
    let target = dir.path().join("target.sqlite");
    symlink(&target, store.path()).expect("final dangling link");
    assert_eq!(
        store.pause_unstarted().expect("SQLite follows final link"),
        0
    );
    assert!(target.is_file());
    let invalid =
        PendingSubmissionStore::for_session_file(&dir.path().join("session.tlog"), "invalid-link");
    symlink(dir.path().join("absent/target.sqlite"), invalid.path()).expect("invalid final link");
    assert!(matches!(
        invalid.pause_unstarted(),
        Err(PendingSubmissionError::Storage(_))
    ));
    let dangling = dir.path().join("dangling");
    symlink(dir.path().join("absent"), &dangling).expect("dangling ancestor");
    let store = PendingSubmissionStore::for_session_file(
        &dangling.join("deeper/session.tlog"),
        "session-1",
    );
    assert!(matches!(
        store.pause_unstarted(),
        Err(PendingSubmissionError::Io(_))
    ));
    let directory = dir.path().join("directory");
    std::fs::create_dir(&directory).expect("directory");
    let link = dir.path().join("directory-link");
    symlink(&directory, &link).expect("directory link");
    let store =
        PendingSubmissionStore::for_session_file(&link.join("missing/session.tlog"), "session-1");
    assert_eq!(store.pause_unstarted().expect("absent linked child"), 0);
    assert!(!directory.join("missing").exists());
    let looping = dir.path().join("looping");
    symlink(&looping, &looping).expect("symlink loop");
    let store =
        PendingSubmissionStore::for_session_file(&looping.join("session.tlog"), "session-1");
    assert!(matches!(
        store.pause_unstarted(),
        Err(PendingSubmissionError::Io(_))
    ));
}

#[test]
fn prestart_error_is_idempotent_but_cannot_erase_a_started_turn() {
    let dir = tempfile::tempdir().expect("temporary directory");
    let store =
        PendingSubmissionStore::for_session_file(&dir.path().join("session.tlog"), "session-1");
    let payload = submission();
    store.accept(&payload).expect("accept");
    store.error_unstarted(&payload.id).expect("terminalize");
    store
        .error_unstarted(&payload.id)
        .expect("duplicate terminalization");
    let record = store.get(&payload.id).expect("read").expect("record");
    assert_eq!(record.state, PendingSubmissionState::TerminalError);
    assert_eq!(record.turn_id, None);

    let mut started = submission();
    started.id = "started-batch".into();
    started.items[0].id = "started-item".into();
    store.accept(&started).expect("accept started");
    store.mark_running(&started.id, "turn-1").expect("start");
    store
        .mark_terminal(&started.id, PendingSubmissionState::TerminalError, "turn-1")
        .expect("terminalize started");
    assert!(matches!(
        store.error_unstarted(&started.id),
        Err(PendingSubmissionError::InvalidTransition)
    ));
    assert_eq!(
        store
            .get(&started.id)
            .expect("read")
            .expect("record")
            .turn_id
            .as_deref(),
        Some("turn-1")
    );
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
        PendingSubmissionStore::for_session_file(&dir.path().join("session.tlog"), "session-1")
            .with_tombstone_limit(2);

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

    for index in 0..3 {
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

#[test]
fn missing_sidecar_recovery_is_read_only_and_admission_stays_durable() {
    let dir = tempfile::tempdir().expect("temporary directory");
    let parent = dir.path().join("missing/runtime");
    let file = parent.join("session.tlog");
    let store = PendingSubmissionStore::for_session_file(&file, "session-1");
    assert!(store.recover_running().expect("empty running").is_empty());
    assert!(store.recover_unstarted().expect("empty pending").is_empty());
    assert!(!parent.exists(), "recovery must not create directories");
    let (receipt, disposition) = store.accept(&submission()).expect("durable admission");
    assert_eq!(disposition, SubmissionReceiptDisposition::AcceptedPending);
    assert!(store.path().is_file());
    drop(store);
    let reopened = PendingSubmissionStore::for_session_file(&file, "session-1");
    let recovered = reopened.recover_unstarted().expect("recover durable work");
    assert_eq!(recovered.len(), 1);
    assert_eq!(recovered[0].receipt_id, receipt);
    assert_eq!(recovered[0].submission, submission());
}

#[test]
fn existing_sidecar_recovery_preserves_running_pending_and_schema_errors() {
    let dir = tempfile::tempdir().expect("temporary directory");
    let file = dir.path().join("session.tlog");
    let store = PendingSubmissionStore::for_session_file(&file, "session-1");
    let running = submission();
    store.accept(&running).expect("admit running");
    store
        .mark_running(&running.id, "turn-1")
        .expect("start running");
    let mut pending = submission();
    pending.id = "batch-2".into();
    pending.items[0].id = "item-2".into();
    store.accept(&pending).expect("admit pending");
    let reopened = PendingSubmissionStore::for_session_file(&file, "session-1");
    let recovered = reopened.recover_running().expect("running recovery");
    assert_eq!(recovered.len(), 1);
    assert_eq!(recovered[0].submission, running);
    assert_eq!(recovered[0].turn_id.as_deref(), Some("turn-1"));
    assert_eq!(
        reopened.recover_unstarted().expect("pending recovery")[0].submission,
        pending
    );
    let connection = Connection::open(store.path()).expect("existing database");
    connection
        .execute(
            "UPDATE pending_journal_meta SET value = 999 WHERE key = 'schema_version'",
            [],
        )
        .expect("unsupported schema fixture");
    assert!(matches!(
        reopened.recover_running(),
        Err(PendingSubmissionError::UnsupportedSchema(999))
    ));
    assert!(matches!(
        reopened.recover_unstarted(),
        Err(PendingSubmissionError::UnsupportedSchema(999))
    ));
}

#[test]
fn absent_read_then_cross_instance_ack_is_recoverable() {
    let dir = tempfile::tempdir().expect("temporary directory");
    let file = dir.path().join("runtime/session.tlog");
    let reader = PendingSubmissionStore::for_session_file(&file, "session-1");
    let writer = PendingSubmissionStore::for_session_file(&file, "session-1");
    let barrier = Arc::new(std::sync::Barrier::new(2));
    let writer_barrier = barrier.clone();
    let writer_task = std::thread::spawn(move || {
        writer_barrier.wait();
        writer
            .accept(&submission())
            .expect("durable acknowledgement")
    });
    assert!(
        reader
            .recover_unstarted()
            .expect("absent snapshot")
            .is_empty()
    );
    assert!(!reader.path().exists());
    barrier.wait();
    let concurrent_snapshot = reader
        .recover_unstarted()
        .expect("read racing with admission");
    assert!(concurrent_snapshot.len() <= 1);
    let (receipt, disposition) = writer_task.join().expect("writer completes");
    if let Some(record) = concurrent_snapshot.first() {
        assert_eq!(record.receipt_id, receipt);
        assert_eq!(record.submission, submission());
    }
    assert_eq!(disposition, SubmissionReceiptDisposition::AcceptedPending);
    let recovered = reader
        .recover_unstarted()
        .expect("read after acknowledgement");
    assert_eq!(recovered.len(), 1);
    assert_eq!(recovered[0].receipt_id, receipt);
}

#[test]
fn recovery_rejects_a_file_instead_of_a_parent_directory() {
    let dir = tempfile::tempdir().expect("temporary directory");
    let parent = dir.path().join("not-a-directory");
    std::fs::write(&parent, "sentinel").expect("file fixture");
    let store = PendingSubmissionStore::for_session_file(&parent.join("session.tlog"), "session-1");
    assert!(store.recover_running().is_err());
    assert!(store.recover_unstarted().is_err());
    assert_eq!(
        std::fs::read_to_string(parent).expect("unchanged parent"),
        "sentinel"
    );
}

#[cfg(unix)]
#[test]
fn recovery_preserves_dangling_ancestor_and_symlink_loop_errors() {
    use std::os::unix::fs::symlink;
    let dir = tempfile::tempdir().expect("temporary directory");
    let dangling = dir.path().join("dangling");
    symlink(dir.path().join("missing-target"), &dangling).expect("dangling ancestor");
    let store = PendingSubmissionStore::for_session_file(
        &dangling.join("deeper/session.tlog"),
        "session-1",
    );
    assert!(matches!(
        store.recover_running(),
        Err(PendingSubmissionError::Io(_))
    ));
    assert!(matches!(
        store.recover_unstarted(),
        Err(PendingSubmissionError::Io(_))
    ));
    let looping = dir.path().join("looping");
    symlink(&looping, &looping).expect("loop fixture");
    let store =
        PendingSubmissionStore::for_session_file(&looping.join("session.tlog"), "session-1");
    assert!(matches!(
        store.recover_running(),
        Err(PendingSubmissionError::Io(_))
    ));
}

#[cfg(unix)]
#[test]
fn recovery_keeps_final_symlink_sqlite_behavior() {
    use std::os::unix::fs::symlink;
    let dir = tempfile::tempdir().expect("temporary directory");
    let store =
        PendingSubmissionStore::for_session_file(&dir.path().join("session.tlog"), "session-1");
    let target = dir.path().join("target.sqlite");
    symlink(&target, store.path()).expect("final dangling link");
    assert!(
        store
            .recover_running()
            .expect("SQLite follows existing link")
            .is_empty()
    );
    assert!(
        target.is_file(),
        "existing link must not be treated as absent"
    );
    let invalid =
        PendingSubmissionStore::for_session_file(&dir.path().join("session.tlog"), "invalid");
    symlink(dir.path().join("absent/target.sqlite"), invalid.path()).expect("invalid final target");
    assert!(matches!(
        invalid.recover_running(),
        Err(PendingSubmissionError::Storage(_))
    ));
}

#[cfg(unix)]
#[test]
fn absent_recovery_below_directory_link_creates_nothing() {
    use std::os::unix::fs::symlink;
    let dir = tempfile::tempdir().expect("temporary directory");
    let target = dir.path().join("target");
    std::fs::create_dir(&target).expect("directory target");
    let link = dir.path().join("link");
    symlink(&target, &link).expect("directory link");
    let store =
        PendingSubmissionStore::for_session_file(&link.join("missing/session.tlog"), "session-1");
    assert!(
        store
            .recover_unstarted()
            .expect("absent linked child")
            .is_empty()
    );
    assert!(!target.join("missing").exists());
}

#[test]
fn relative_missing_sidecar_recovery_does_not_create_storage() {
    let session_id = format!("missing-{}", uuid::Uuid::new_v4());
    let store = PendingSubmissionStore::for_session_file(Path::new("session.tlog"), &session_id);
    assert!(
        store
            .recover_running()
            .expect("relative empty running")
            .is_empty()
    );
    assert!(
        store
            .recover_unstarted()
            .expect("relative empty pending")
            .is_empty()
    );
    assert!(!store.path().exists());
}

#[cfg(unix)]
#[test]
fn recovery_preserves_filesystem_permission_errors() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().expect("temporary directory");
    let parent = dir.path().join("restricted");
    std::fs::create_dir(&parent).expect("restricted directory");
    let store = PendingSubmissionStore::for_session_file(&parent.join("session.tlog"), "session-1");
    std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o000))
        .expect("restrict directory");
    let observation = std::fs::symlink_metadata(store.path());
    let running = store.recover_running();
    let pending = store.recover_unstarted();
    let paused = store.pause_unstarted();
    std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o700))
        .expect("restore directory");
    let kind = observation.expect_err("sidecar remains absent").kind();
    eprintln!("pending-journal permission fixture filesystem_error={kind:?}");
    match kind {
        std::io::ErrorKind::PermissionDenied => {
            assert!(
                matches!(paused, Err(PendingSubmissionError::Io(e)) if e.kind() == std::io::ErrorKind::PermissionDenied)
            );
            assert!(
                matches!(running, Err(PendingSubmissionError::Io(e)) if e.kind() == std::io::ErrorKind::PermissionDenied)
            );
            assert!(
                matches!(pending, Err(PendingSubmissionError::Io(e)) if e.kind() == std::io::ErrorKind::PermissionDenied)
            );
        }
        std::io::ErrorKind::NotFound => {
            // A privileged process can inspect this directory despite its mode.
            assert_eq!(paused.expect("privileged empty pause"), 0);
            assert!(running.expect("privileged empty running").is_empty());
            assert!(pending.expect("privileged empty pending").is_empty());
        }
        other => panic!("unexpected filesystem error: {other:?}"),
    }
    assert!(
        !store.path().exists(),
        "neither branch may create a journal"
    );
}

#[test]
fn missing_parent_before_dotdot_still_recovers_existing_custody() {
    let dir = tempfile::tempdir().expect("temporary directory");
    let store =
        PendingSubmissionStore::for_session_file(&dir.path().join("session.tlog"), "session-1");
    let (receipt, _) = store
        .accept(&submission())
        .expect("durable canonical admission");
    let missing = dir.path().join("missing");
    let alternate =
        PendingSubmissionStore::for_session_file(&missing.join("../session.tlog"), "session-1");
    assert!(!missing.exists());
    let recovered = alternate
        .recover_unstarted()
        .expect("original SQLite path recovery");
    assert_eq!(recovered.len(), 1);
    assert_eq!(recovered[0].receipt_id, receipt);
    assert_eq!(recovered[0].submission, submission());
    // Windows may normalize the cancelled component before filesystem access.
    #[cfg(unix)]
    assert!(
        missing.is_dir(),
        "ParentDir paths retain original directory creation"
    );
}

#[test]
fn absent_parentdir_path_retains_original_sqlite_creation() {
    let dir = tempfile::tempdir().expect("temporary directory");
    let missing = dir.path().join("missing");
    let store =
        PendingSubmissionStore::for_session_file(&missing.join("../session.tlog"), "session-1");
    assert!(
        store
            .recover_running()
            .expect("original empty SQLite recovery")
            .is_empty()
    );
    // Creating this intermediate component is the original Unix path behavior.
    #[cfg(unix)]
    assert!(missing.is_dir());
    assert!(dir.path().join("session-1.pending.sqlite").is_file());
}
