//! Cooperative cross-process exclusion for Session execution and recovery.

use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

use crate::SessionError;

/// Exclusive execution/recovery ownership of a Session.
///
/// The OS releases ownership when the last handle closes, including on process
/// death. Lock files are permanent identity anchors: never unlink them, including
/// during Session deletion. All cooperating writers must use this protocol;
/// older binaries and external filesystem modification are outside its boundary.
#[derive(Debug)]
pub struct SessionRuntimeLease {
    _file: File,
    identity: PathBuf,
    transcript: PathBuf,
}

impl SessionRuntimeLease {
    /// Stable durable and model-facing explanation for one interrupted turn.
    pub fn interruption_message(turn_id: &str) -> String {
        format!(
            "Turn {turn_id} was interrupted before its terminal result was saved. Tool side effects may have occurred; the interrupted request was not replayed. You can submit new work."
        )
    }
    /// Acquire ownership without waiting or interrupting the current owner.
    /// Parent aliases are canonicalized; symlink lock directories/files are refused.
    pub fn acquire(transcript: &Path) -> Result<Self, SessionError> {
        std::panic::catch_unwind(|| Self::acquire_inner(transcript)).unwrap_or_else(|_| {
            Err(SessionError::DurableTurn(
                "Session runtime lock panicked".into(),
            ))
        })
    }

    fn acquire_inner(transcript: &Path) -> Result<Self, SessionError> {
        let parent = transcript
            .parent()
            .ok_or_else(|| SessionError::DurableTurn("Session transcript has no parent".into()))?;
        std::fs::create_dir_all(parent)?;
        let directory = parent.canonicalize()?.join(".runtime-locks");
        match std::fs::create_dir(&directory) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.into()),
        }
        if !std::fs::symlink_metadata(&directory)?.file_type().is_dir() {
            return Err(SessionError::DurableTurn(
                "invalid Session lock directory".into(),
            ));
        }
        let stem = transcript.file_stem().ok_or_else(|| {
            SessionError::DurableTurn("Session transcript has no identity".into())
        })?;
        let path = directory.join(stem);
        let file = match OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                if !std::fs::symlink_metadata(&path)?.file_type().is_file() {
                    return Err(SessionError::DurableTurn(
                        "invalid Session lock file".into(),
                    ));
                }
                OpenOptions::new().read(true).write(true).open(&path)?
            }
            Err(error) => return Err(error.into()),
        };
        file.try_lock().map_err(|error| {
            SessionError::DurableTurn(format!(
                "Session is in use or runtime lock unavailable: {error}"
            ))
        })?;
        Ok(Self {
            _file: file,
            identity: path,
            transcript: parent
                .canonicalize()?
                .join(transcript.file_name().ok_or_else(|| {
                    SessionError::DurableTurn("Session transcript has no filename".into())
                })?),
        })
    }

    /// Verify all native transcript records before interpreting missing outcomes.
    /// Corrupt or truncated tails are refused, never silently skipped or rewritten.
    pub fn validate_transcript(&self) -> Result<(), SessionError> {
        crate::history_page::validate_host_transcript(&self.transcript).map_err(|error| {
            SessionError::DurableTurn(format!("cannot validate interrupted transcript: {error}"))
        })
    }

    /// Record an abandoned turn as an error, never success or resumable work.
    ///
    /// The caller must own this Session's execution lifetime and verify the
    /// journal generation/turn before calling. Existing terminal proof wins.
    /// The explanation and terminal marker are synced before journal finalization;
    /// retries reuse the explanation if a previous attempt stopped between writes.
    pub fn interrupt_turn(
        &self,
        session: &crate::Session,
        turn_id: &str,
    ) -> Result<bool, SessionError> {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let parent = session.file_path.parent().ok_or_else(|| {
                SessionError::DurableTurn("Session transcript has no parent".into())
            })?;
            let identity = parent.canonicalize()?.join(".runtime-locks").join(
                session.file_path.file_stem().ok_or_else(|| {
                    SessionError::DurableTurn("Session transcript has no identity".into())
                })?,
            );
            let actual = parent
                .canonicalize()?
                .join(session.file_path.file_name().ok_or_else(|| {
                    SessionError::DurableTurn("Session transcript has no filename".into())
                })?);
            if identity != self.identity || actual != self.transcript || turn_id.is_empty() {
                return Err(SessionError::DurableTurn(
                    "recovery lease identity mismatch".into(),
                ));
            }
            self.validate_transcript()?;
            if session
                .read_turn_transcript_outcomes()?
                .iter()
                .any(|record| record.turn_id == turn_id)
            {
                return Ok(false);
            }
            let explanation = Self::interruption_message(turn_id);
            if !session
                .read_entries()?
                .iter()
                .any(|entry| entry.role == "assistant" && entry.content == explanation)
            {
                session.append_with_metadata(
                    &talos_core::message::Message::Assistant {
                        content: explanation,
                        tool_calls: Vec::new(),
                        reasoning: None,
                    },
                    crate::SessionMetadata::default(),
                )?;
            }
            session.append_turn_transcript_outcome(&crate::TurnTranscriptOutcomeRecord::new(
                turn_id,
                crate::TurnTranscriptOutcome::Error,
            ))?;
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(&session.file_path)?
                .sync_all()?;
            Ok(true)
        }))
        .unwrap_or_else(|_| {
            Err(SessionError::DurableTurn(
                "Session interruption recovery panicked".into(),
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_lease_is_released_after_process_death() {
        const CHILD: &str = "TALOS_RUNTIME_LEASE_CHILD";
        if let Some(root) = std::env::var_os(CHILD) {
            let root = PathBuf::from(root);
            let _lease =
                SessionRuntimeLease::acquire(&root.join("crashed.tlog")).expect("child lease");
            std::fs::write(root.join("ready"), b"ready").expect("ready");
            loop {
                std::thread::park();
            }
        }
        struct ChildGuard(std::process::Child);
        impl Drop for ChildGuard {
            fn drop(&mut self) {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }
        let temp = tempfile::tempdir().expect("temp");
        let mut child = ChildGuard(
            std::process::Command::new(std::env::current_exe().expect("exe"))
                .args([
                    "--exact",
                    "runtime_lease::tests::runtime_lease_is_released_after_process_death",
                    "--nocapture",
                ])
                .env(CHILD, temp.path())
                .stdout(std::process::Stdio::null())
                .spawn()
                .expect("child"),
        );
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while !temp.path().join("ready").exists() {
            assert!(
                std::time::Instant::now() < deadline,
                "child readiness timeout"
            );
            assert!(child.0.try_wait().expect("child status").is_none());
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let transcript = temp.path().join("crashed.tlog");
        assert!(SessionRuntimeLease::acquire(&transcript).is_err());
        child.0.kill().expect("simulate crash");
        child.0.wait().expect("reap");
        let _recovered = SessionRuntimeLease::acquire(&transcript).expect("recover after death");
    }

    #[test]
    fn runtime_lease_excludes_second_owner_and_survives_anchor_reuse() {
        let temp = tempfile::tempdir().expect("temp");
        let transcript = temp.path().join("session.tlog");
        let first = SessionRuntimeLease::acquire(&transcript).expect("first");
        assert!(SessionRuntimeLease::acquire(&transcript).is_err());
        assert!(SessionRuntimeLease::acquire(&temp.path().join("session.jsonl")).is_err());
        drop(first);
        let second = SessionRuntimeLease::acquire(&transcript).expect("reacquire");
        assert!(temp.path().join(".runtime-locks/session").exists());
        drop(second);
    }

    #[test]
    fn interruption_is_idempotent_and_does_not_overwrite_terminal_proof() {
        let temp = tempfile::tempdir().expect("temp");
        let manager = crate::SessionManager::with_dir(temp.path().to_path_buf());
        let durable = manager
            .create_or_open_session("lease-recovery")
            .expect("session");
        let session = durable.session();
        let lease = SessionRuntimeLease::acquire(&session.file_path).expect("lease");
        assert!(
            lease
                .interrupt_turn(session, "interrupted")
                .expect("recover")
        );
        assert!(!lease.interrupt_turn(session, "interrupted").expect("retry"));
        session
            .append_turn_transcript_outcome(&crate::TurnTranscriptOutcomeRecord::new(
                "finished",
                crate::TurnTranscriptOutcome::Success,
            ))
            .expect("success");
        assert!(!lease.interrupt_turn(session, "finished").expect("preserve"));
        let outcomes = session.read_turn_transcript_outcomes().expect("outcomes");
        assert_eq!(outcomes.len(), 2);
        assert_eq!(outcomes[0].outcome, crate::TurnTranscriptOutcome::Error);
        assert_eq!(outcomes[1].outcome, crate::TurnTranscriptOutcome::Success);
        let other = manager
            .create_or_open_session("other-session")
            .expect("other");
        assert!(lease.interrupt_turn(other.session(), "wrong").is_err());
    }

    #[test]
    fn corrupt_or_truncated_transcript_refuses_recovery_without_mutation() {
        for extension in ["tlog", "jsonl"] {
            let temp = tempfile::tempdir().expect("temp");
            let path = temp.path().join(format!("session.{extension}"));
            let original = if extension == "tlog" {
                b"TALOS\t1\t0\nE\ttruncated".as_slice()
            } else {
                b"{broken json}\n".as_slice()
            };
            std::fs::write(&path, original).expect("fixture");
            let lease = SessionRuntimeLease::acquire(&path).expect("lease");
            assert!(lease.validate_transcript().is_err());
            assert_eq!(std::fs::read(path).expect("read"), original);
        }
    }

    #[test]
    fn strict_recovery_accepts_large_native_and_legacy_records() {
        use crate::SessionStore;
        let temp = tempfile::tempdir().expect("temp");
        let manager = crate::SessionManager::with_dir(temp.path().to_path_buf());
        let session = manager.create_session("large", "").expect("session");
        session
            .append(&talos_core::message::Message::User {
                content: "x".repeat(3 * 1024 * 1024),
            })
            .expect("large write");
        let lease = SessionRuntimeLease::acquire(&session.file_path).expect("lease");
        lease.validate_transcript().expect("large native record");
        lease
            .interrupt_turn(&session, "large-turn")
            .expect("recover");
        let bytes = std::fs::read(&session.file_path).expect("native bytes");
        let header_end = bytes.iter().position(|b| *b == b'\n').expect("header");
        let legacy = temp.path().join("legacy.tlog");
        std::fs::write(&legacy, &bytes[header_end + 1..]).expect("legacy headerless");
        SessionRuntimeLease::acquire(&legacy)
            .expect("legacy lease")
            .validate_transcript()
            .expect("headerless native");
        let jsonl = temp.path().join("legacy.jsonl");
        let envelope = serde_json::json!({"type": "message", "data": talos_core::message::Message::User { content: "legacy".into() }});
        std::fs::write(&jsonl, format!("{envelope}\n")).expect("envelope");
        {
            let legacy_lease = SessionRuntimeLease::acquire(&jsonl).expect("legacy jsonl lease");
            legacy_lease.validate_transcript().expect("legacy envelope");
        }
        let entries = session.read_entries().expect("entries");
        crate::JsonlSessionStore
            .replace_entries_atomically(&jsonl, &entries)
            .expect("jsonl");
        SessionRuntimeLease::acquire(&jsonl)
            .expect("jsonl lease")
            .validate_transcript()
            .expect("large jsonl");
    }

    #[test]
    fn live_runtime_prevents_artifact_deletion() {
        let temp = tempfile::tempdir().expect("temp");
        let manager = crate::SessionManager::with_dir(temp.path().to_path_buf());
        let session = manager.create_session("live-delete", "").expect("session");
        let lease = SessionRuntimeLease::acquire(&session.file_path).expect("lease");
        assert!(manager.delete_session(&session.id).is_err());
        assert!(session.file_path.exists());
        drop(lease);
        manager
            .delete_session(&session.id)
            .expect("delete after exit");
    }

    #[test]
    fn complete_cleanup_holds_ownership_through_index_removal() {
        let temp = tempfile::tempdir().expect("temp");
        let manager = crate::SessionManager::with_dir(temp.path().to_path_buf());
        let session = manager.create_session("index-race", "").expect("session");
        let store = crate::PendingSubmissionStore::for_session(&session);
        store.runtime_generation().expect("sidecar");
        crate::artifacts::cleanup_session_with_index(&session.file_path, || {
            assert!(!store.path().exists(), "sidecars removed first");
            assert!(
                session.file_path.exists(),
                "transcript retained until index cleanup"
            );
            assert!(
                SessionRuntimeLease::acquire(&session.file_path).is_err(),
                "actor cannot start in cleanup gap"
            );
            Ok(())
        })
        .expect("cleanup");
        assert!(!session.file_path.exists());
    }

    #[cfg(unix)]
    #[test]
    fn readable_terminal_marker_cannot_release_custody_when_sync_open_is_denied() {
        use std::os::unix::fs::PermissionsExt;
        let temp = tempfile::tempdir().expect("temp");
        let manager = crate::SessionManager::with_dir(temp.path().to_path_buf());
        let session = manager.create_session("sync-denied", "").expect("session");
        let store = crate::PendingSubmissionStore::for_session(&session);
        let submission = talos_core::session::StructuredSubmission {
            id: "sync-failure".into(),
            source: talos_core::session::SubmissionSource::User,
            sender_generation: 0,
            items: vec![talos_core::session::SubmissionItem {
                id: "item".into(),
                enqueue_sequence: 1,
                kind: talos_core::session::SubmissionKind::UserTurn,
                text: "work".into(),
                attachments: Vec::new(),
            }],
        };
        store.accept(&submission).expect("accept");
        store
            .mark_running(&submission.id, "sync-turn")
            .expect("running");
        session
            .append_turn_transcript_outcome(&crate::TurnTranscriptOutcomeRecord::new(
                "sync-turn",
                crate::TurnTranscriptOutcome::Error,
            ))
            .expect("append");
        let original = std::fs::metadata(&session.file_path)
            .expect("metadata")
            .permissions();
        std::fs::set_permissions(&session.file_path, std::fs::Permissions::from_mode(0o400))
            .expect("read only");
        let result = store.mark_committed(&submission.id, "sync-turn");
        std::fs::set_permissions(&session.file_path, original).expect("restore");
        assert!(
            result.is_err(),
            "readable bytes do not replace successful sync"
        );
        assert_eq!(
            store
                .get(&submission.id)
                .expect("get")
                .expect("record")
                .state,
            talos_core::session::PendingSubmissionState::Running
        );
        store
            .mark_committed(&submission.id, "sync-turn")
            .expect("retry");
        assert_eq!(
            store
                .get(&submission.id)
                .expect("get")
                .expect("record")
                .state,
            talos_core::session::PendingSubmissionState::TerminalError
        );
    }

    #[cfg(unix)]
    #[test]
    fn runtime_lease_refuses_symlink_anchors_and_resolves_parent_aliases() {
        let temp = tempfile::tempdir().expect("temp");
        let root = temp.path().join("real");
        std::fs::create_dir(&root).expect("root");
        let alias = temp.path().join("alias");
        std::os::unix::fs::symlink(&root, &alias).expect("alias");
        let first = SessionRuntimeLease::acquire(&root.join("session.tlog")).expect("first");
        assert!(SessionRuntimeLease::acquire(&alias.join("session.tlog")).is_err());
        let target = temp.path().join("target");
        std::fs::write(&target, "untouched").expect("target");
        std::os::unix::fs::symlink(&target, root.join(".runtime-locks/other")).expect("link");
        assert!(SessionRuntimeLease::acquire(&root.join("other.tlog")).is_err());
        assert_eq!(
            std::fs::read_to_string(target).expect("target"),
            "untouched"
        );
        drop(first);
    }
}
