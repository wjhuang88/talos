//! Ordinary transcript capture for an Actor-owned fork exclusion window.

use std::time::Instant;

use crate::{
    PendingSubmissionStore, Session, SessionEntry, SessionError, SessionRuntimeIdentity,
    SessionRuntimeLease,
};

/// Exact ordinary transcript prefix and runtime identity captured under source ownership.
///
/// This is not publication authority for archived or receipt-bound history. The host
/// must publish to a fresh child and retain the captured identity, including absence.
/// The caller must exclude its own turns and model transitions while capturing;
/// a runtime lease excludes competing cooperating runtimes, not its owner's writes.
pub struct OrdinaryForkSnapshot {
    /// Original bytes through the selected or last decodable record.
    pub bytes: Vec<u8>,
    /// Original selected tail; IDs and parent links are not normalized.
    pub tail: SessionEntry,
    /// Publication source identity, or explicit absence, at capture time.
    pub runtime_identity: Option<SessionRuntimeIdentity>,
}

fn unavailable() -> SessionError {
    SessionError::DurableTurn("ordinary fork snapshot unavailable".into())
}

impl Session {
    /// Capture ordinary TLOG/JSONL history and committed identity under source ownership.
    ///
    /// Raw copying is limited to 64 MiB and uses the caller's cooperative deadline.
    /// Both source version and runtime state are checked before returning. Incomplete
    /// compatibility tails are excluded exactly as in ordinary startup forks. Archive
    /// placeholders and receipt envelopes require the provenance-aware publisher.
    pub fn capture_ordinary_fork(
        &self,
        lease: &SessionRuntimeLease,
        through_entry: Option<&str>,
        deadline: Instant,
        stopped: impl Fn() -> bool,
    ) -> Result<OrdinaryForkSnapshot, SessionError> {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let check = || {
                if stopped() || Instant::now() >= deadline {
                    Err(unavailable())
                } else {
                    Ok(())
                }
            };
            check()?;
            lease.check_session(self)?;
            let store = PendingSubmissionStore::for_session(self);
            let (identity, _) = store
                .fork_runtime_with_budget(deadline, None)
                .map_err(|_| unavailable())?;
            if identity.as_ref().is_some_and(|state| {
                state.status != crate::SessionRuntimeActivationStatus::Committed
            }) {
                return Err(unavailable());
            }
            let mut scan = match through_entry {
                Some(id) => self.begin_message_scan_through(id),
                None => self.begin_message_scan(),
            }
            .map_err(|_| unavailable())?;
            let mut bytes = Vec::new();
            loop {
                check()?;
                let page = scan
                    .copy_next_page(deadline, &stopped)
                    .map_err(|_| unavailable())?;
                let copied = page.is_none();
                if let Some(page) = page {
                    if bytes.len().saturating_add(page.len()) > 64 * 1024 * 1024 {
                        return Err(unavailable());
                    }
                    bytes.extend_from_slice(&page);
                }
                let decoded = scan.step(deadline, &stopped).map_err(|_| unavailable())?;
                if copied && decoded {
                    break;
                }
            }
            if scan.has_archive_placeholder() {
                return Err(unavailable());
            }
            let (_, tail, length) = scan
                .finish_with_record_copy_length_cancellable(deadline, &stopped)
                .map_err(|_| unavailable())?;
            check()?;
            let (current, _) = store
                .fork_runtime_with_budget(deadline, None)
                .map_err(|_| unavailable())?;
            if current != identity {
                return Err(unavailable());
            }
            let length = usize::try_from(length).map_err(|_| unavailable())?;
            if length > bytes.len() {
                return Err(unavailable());
            }
            bytes.truncate(length);
            check()?;
            Ok(OrdinaryForkSnapshot {
                bytes,
                tail: tail.ok_or_else(unavailable)?,
                runtime_identity: identity.map(|state| state.activation.target),
            })
        }))
        .unwrap_or_else(|_| Err(unavailable()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use talos_core::message::Message;
    use uuid::Uuid;

    #[test]
    fn ordinary_capture_preserves_both_formats_and_excludes_incomplete_tail() {
        for extension in ["tlog", "jsonl"] {
            let root = tempfile::tempdir().expect("root");
            let id = Uuid::new_v4();
            let source = Session::new(
                id,
                "source".into(),
                "".into(),
                root.path().join(format!("{id}.{extension}")),
            );
            source
                .append(&Message::User {
                    content: "original".into(),
                })
                .expect("append");
            let original = std::fs::read(&source.file_path).expect("bytes");
            let mut physical = original.clone();
            physical.extend_from_slice(if extension == "tlog" {
                b"E\t0\t1000\tincomplete\t-\t2097153:x"
            } else {
                b"{invalid trailing record\n"
            });
            std::fs::write(&source.file_path, &physical).expect("partial tail");
            let lease = SessionRuntimeLease::acquire(&source.file_path).expect("lease");
            let snapshot = source
                .capture_ordinary_fork(
                    &lease,
                    None,
                    Instant::now() + Duration::from_secs(5),
                    || false,
                )
                .expect("capture");
            assert_eq!(snapshot.bytes, original);
            assert_eq!(snapshot.tail.content, "original");
            assert!(snapshot.runtime_identity.is_none());
            PendingSubmissionStore::for_session(&source)
                .initialize_runtime_identity(SessionRuntimeIdentity::new("later", "model", None))
                .expect("later identity");
            assert!(
                snapshot.runtime_identity.is_none(),
                "capture cannot inherit later identity"
            );
            assert_eq!(
                std::fs::read(&source.file_path).expect("source unchanged"),
                physical
            );
            assert!(
                source
                    .capture_ordinary_fork(
                        &lease,
                        None,
                        Instant::now() + Duration::from_secs(5),
                        || true
                    )
                    .is_err()
            );
        }
    }
    #[test]
    fn ordinary_capture_rejects_identity_change_during_copy() {
        let root = tempfile::tempdir().expect("root");
        let manager = crate::SessionManager::with_dir(root.path().join("sessions"));
        let source = manager.create_session("source", "").expect("source");
        source
            .append(&Message::User {
                content: "original".into(),
            })
            .expect("append");
        let lease = SessionRuntimeLease::acquire(&source.file_path).expect("lease");
        let polls = std::cell::Cell::new(0);
        let result = source.capture_ordinary_fork(
            &lease,
            None,
            Instant::now() + Duration::from_secs(5),
            || {
                polls.set(polls.get() + 1);
                // First poll is admission, second is the first copy iteration after identity capture.
                if polls.get() == 2 {
                    PendingSubmissionStore::for_session(&source)
                        .initialize_runtime_identity(SessionRuntimeIdentity::new(
                            "changed", "model", None,
                        ))
                        .expect("identity mutation");
                }
                false
            },
        );
        assert!(
            result.is_err(),
            "mixed history/identity capture must not escape"
        );
        let current = source
            .capture_ordinary_fork(
                &lease,
                None,
                Instant::now() + Duration::from_secs(5),
                || false,
            )
            .expect("stable retry");
        assert_eq!(
            current.runtime_identity,
            Some(SessionRuntimeIdentity::new("changed", "model", None))
        );
    }

    #[test]
    fn ordinary_capture_rejects_wrong_owner_and_archived_head() {
        let root = tempfile::tempdir().expect("root");
        let manager = crate::SessionManager::with_dir(root.path().join("sessions"));
        let source = manager.create_session("source", "").expect("source");
        let other = manager.create_session("other", "").expect("other");
        source
            .append(&Message::User {
                content: "original".into(),
            })
            .expect("append");
        let other_lease = SessionRuntimeLease::acquire(&other.file_path).expect("other lease");
        assert!(
            source
                .capture_ordinary_fork(
                    &other_lease,
                    None,
                    Instant::now() + Duration::from_secs(5),
                    || false
                )
                .is_err()
        );
        source.compact_archived(0).expect("archive");
        let source_lease = SessionRuntimeLease::acquire(&source.file_path).expect("source lease");
        assert!(
            source
                .capture_ordinary_fork(
                    &source_lease,
                    None,
                    Instant::now() + Duration::from_secs(5),
                    || false
                )
                .is_err()
        );
    }
    #[test]
    fn ordinary_capture_rejects_staged_uncommitted_activation() {
        let root = tempfile::tempdir().expect("root");
        let manager = crate::SessionManager::with_dir(root.path().join("sessions"));
        let source = manager.create_session("source", "").expect("source");
        source
            .append(&Message::User {
                content: "original".into(),
            })
            .expect("append");
        let store = PendingSubmissionStore::for_session(&source);
        let previous = SessionRuntimeIdentity::new("provider", "old", None);
        store
            .initialize_runtime_identity(previous.clone())
            .expect("initial identity");
        let activation = crate::SessionRuntimeActivation::new(
            1,
            previous,
            SessionRuntimeIdentity::new("provider", "new", None),
        );
        store
            .stage_runtime_activation(0, &activation)
            .expect("stage without marker");
        let lease = SessionRuntimeLease::acquire(&source.file_path).expect("lease");
        assert!(
            source
                .capture_ordinary_fork(
                    &lease,
                    None,
                    Instant::now() + Duration::from_secs(5),
                    || false
                )
                .is_err()
        );
        assert_eq!(
            store
                .runtime_state()
                .expect("state")
                .expect("pending")
                .status,
            crate::SessionRuntimeActivationStatus::PendingMarker
        );
    }
}
