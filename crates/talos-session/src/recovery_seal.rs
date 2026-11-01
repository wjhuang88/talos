//! Incremental immutable-prefix verification for storage-issued recovery sources.

use crate::{DurableRecoverySource, history_page::HistoryPageError};
use sha2::{Digest, Sha256};
use std::time::Instant;

/// Incremental source scan. Each step reads at most one bounded history page.
/// This is content consistency evidence, not branch or permission authority.
pub struct RecoveryPrefixScan {
    source: DurableRecoverySource,
    stamp: std::fs::Metadata,
    offset: u64,
    digest: [u8; 32],
    expected: Option<[u8; 32]>,
}

/// Storage-issued rolling digest for all raw bytes of one captured prefix. Later suffix appends
/// are compatible; replacing any covered record invalidates verification.
#[derive(Clone)]
pub struct RecoveryPrefixSeal {
    source: DurableRecoverySource,
    digest: [u8; 32],
}

impl DurableRecoverySource {
    /// Begin incrementally sealing this source's captured prefix.
    /// No source content is read until the first step.
    pub fn begin_prefix_seal(&self) -> Result<RecoveryPrefixScan, HistoryPageError> {
        Ok(RecoveryPrefixScan {
            source: self.clone(),
            stamp: self.recovery_metadata()?,
            offset: 0,
            digest: [0; 32],
            expected: None,
        })
    }
}

impl RecoveryPrefixSeal {
    /// Begin a bounded-page verification of the entire captured prefix.
    /// The scan must finish before this evidence can authorize a later read.
    pub fn begin_verification(&self) -> Result<RecoveryPrefixScan, HistoryPageError> {
        let mut scan = self.source.begin_prefix_seal()?;
        scan.expected = Some(self.digest);
        Ok(scan)
    }

    /// Captured source cutoff, unaffected by later append-only writes.
    pub fn source_end(&self) -> u64 {
        self.source.source_end()
    }
}

impl RecoveryPrefixScan {
    /// Read at most 2 MiB of raw bytes, extending a constant-size rolling digest.
    /// Verification compares the digest only at completion, never certifies a page.
    /// Cancellation, deadline, or concurrent source changes do not advance state.
    /// A failed scan should be discarded; restart against a stable source.
    pub fn step(
        &mut self,
        deadline: Instant,
        stopped: impl Fn() -> bool,
    ) -> Result<bool, HistoryPageError> {
        self.check_stamp()?;
        if stopped() || Instant::now() >= deadline {
            return Err(HistoryPageError::Stopped);
        }
        if self.offset == self.source.source_end() {
            return Ok(true);
        }
        let bytes =
            self.source
                .read_recovery_bytes(self.offset, &self.stamp, deadline, &stopped)?;
        let next_offset = self.offset + bytes.len() as u64;
        let mut hash = Sha256::new();
        hash.update(self.digest);
        hash.update(self.offset.to_le_bytes());
        hash.update(next_offset.to_le_bytes());
        hash.update(bytes);
        let digest: [u8; 32] = hash.finalize().into();
        let complete = next_offset == self.source.source_end();
        if complete && self.expected.is_some_and(|expected| expected != digest) {
            return Err(HistoryPageError::Stale);
        }
        self.check_stamp()?;
        if stopped() || Instant::now() >= deadline {
            return Err(HistoryPageError::Stopped);
        }
        self.digest = digest;
        self.offset = next_offset;
        Ok(complete)
    }

    /// Finish only after every covered page has been read successfully.
    /// Source stability is checked again; no partial digest grants evidence.
    pub fn finish(self) -> Result<RecoveryPrefixSeal, HistoryPageError> {
        self.check_stamp()?;
        if self.offset != self.source.source_end()
            || self
                .expected
                .as_ref()
                .is_some_and(|expected| expected != &self.digest)
        {
            return Err(HistoryPageError::Stale);
        }
        Ok(RecoveryPrefixSeal {
            source: self.source,
            digest: self.digest,
        })
    }

    fn check_stamp(&self) -> Result<(), HistoryPageError> {
        let current = self.source.recovery_metadata()?;
        if !same_file_version(&current, &self.stamp)? {
            return Err(HistoryPageError::Stale);
        }
        Ok(())
    }
}

// Platforms without stable file identity/change markers cannot mint this evidence.
#[cfg(unix)]
pub(crate) fn same_file_version(
    a: &std::fs::Metadata,
    b: &std::fs::Metadata,
) -> Result<bool, HistoryPageError> {
    use std::os::unix::fs::MetadataExt;
    Ok(a.dev() == b.dev()
        && a.ino() == b.ino()
        && a.len() == b.len()
        && a.mtime() == b.mtime()
        && a.mtime_nsec() == b.mtime_nsec()
        && a.ctime() == b.ctime()
        && a.ctime_nsec() == b.ctime_nsec())
}
#[cfg(not(unix))]
pub(crate) fn same_file_version(
    _: &std::fs::Metadata,
    _: &std::fs::Metadata,
) -> Result<bool, HistoryPageError> {
    Err(HistoryPageError::UnsupportedSource)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::{DurableSession, PersistencePolicy, TurnTranscriptOutcome};
    use std::time::Duration;
    use talos_core::message::Message;

    #[test]
    fn multi_page_prefix_verifies_after_append_and_refuses_replacement_or_partial_scan() {
        let dir = tempfile::tempdir().expect("directory");
        let session = DurableSession::open_or_create(dir.path(), "owner").expect("session");
        let messages: Vec<_> = (0..600)
            .map(|i| Message::User {
                content: format!("source {i} {}", "bounded record ".repeat(300)),
            })
            .collect();
        session
            .finalize_turn_with_recovery(
                "first",
                &messages,
                &PersistencePolicy::default(),
                TurnTranscriptOutcome::Success,
                &[],
            )
            .expect("commit");
        let source = session.capture_recovery_source().expect("source");
        assert!(source.source_end() > crate::history_page::HISTORY_PAGE_BYTES as u64);
        let mut interrupted = source.begin_prefix_seal().expect("interrupted scan");
        assert!(
            !interrupted
                .step(Instant::now() + Duration::from_secs(10), || false)
                .expect("first chunk")
        );
        let original_meta = std::fs::metadata(session.file_path()).expect("metadata");
        let replacement = dir.path().join("replacement");
        std::fs::copy(session.file_path(), &replacement).expect("same length clone");
        let file = std::fs::OpenOptions::new()
            .write(true)
            .open(&replacement)
            .expect("replacement");
        file.set_times(
            std::fs::FileTimes::new().set_modified(original_meta.modified().expect("mtime")),
        )
        .expect("preserve mtime");
        std::fs::rename(replacement, session.file_path()).expect("atomic replacement");
        assert!(
            matches!(
                source.read_recovery_bytes(
                    0,
                    &original_meta,
                    Instant::now() + Duration::from_secs(2),
                    || false
                ),
                Err(HistoryPageError::Stale)
            ),
            "opened handle must match expected inode even if length/mtime match"
        );
        assert!(matches!(
            interrupted.step(Instant::now() + Duration::from_secs(10), || false),
            Err(HistoryPageError::Stale)
        ));
        let deadline = Instant::now() + Duration::from_secs(10);
        assert!(source.begin_prefix_seal().expect("scan").finish().is_err());
        let mut scan = source.begin_prefix_seal().expect("scan");
        assert!(matches!(
            scan.step(deadline, || true),
            Err(HistoryPageError::Stopped)
        ));
        let mut steps = 1;
        while !scan.step(deadline, || false).expect("page") {
            steps += 1;
        }
        assert!(steps > 1);
        let seal = scan.finish().expect("seal");
        let original_bytes = std::fs::read(session.file_path()).expect("original bytes");
        session
            .finalize_turn_with_recovery(
                "second",
                &[Message::User {
                    content: "later suffix".into(),
                }],
                &PersistencePolicy::default(),
                TurnTranscriptOutcome::Success,
                &[],
            )
            .expect("durable append");
        assert!(
            std::fs::read(session.file_path())
                .expect("committed bytes")
                .starts_with(&original_bytes)
        );
        let mut verify = seal.begin_verification().expect("verify appended source");
        while !verify.step(deadline, || false).expect("same prefix") {}
        assert_eq!(
            verify.finish().expect("verified").source_end(),
            source.source_end()
        );
        // Actual same-length replacement retains IDs and ranges, but not content.
        let mut bytes = std::fs::read(session.file_path()).expect("source bytes");
        let at = bytes
            .windows(b"bounded record".len())
            .position(|b| b == b"bounded record")
            .expect("record text");
        bytes[at] = b'X';
        std::fs::write(session.file_path(), bytes).expect("replace prefix");
        let mut verify = seal.begin_verification().expect("scan changed source");
        loop {
            match verify.step(deadline, || false) {
                Ok(false) => continue,
                Err(HistoryPageError::Stale) => break,
                other => panic!("changed prefix must fail: {other:?}"),
            }
        }
    }

    #[test]
    fn mutation_between_pages_invalidates_scan_instead_of_sealing_mixed_versions() {
        let dir = tempfile::tempdir().expect("directory");
        let session = DurableSession::open_or_create(dir.path(), "owner").expect("session");
        session
            .finalize_turn_with_recovery(
                "turn",
                &[Message::User {
                    content: "original".into(),
                }],
                &PersistencePolicy::default(),
                TurnTranscriptOutcome::Success,
                &[],
            )
            .expect("commit");
        let source = session.capture_recovery_source().expect("source");
        let mut scan = source.begin_prefix_seal().expect("scan");
        assert!(
            scan.step(Instant::now() + Duration::from_secs(2), || false)
                .expect("page")
        );
        session
            .finalize_turn_with_recovery(
                "append",
                &[Message::User {
                    content: "suffix".into(),
                }],
                &PersistencePolicy::default(),
                TurnTranscriptOutcome::Success,
                &[],
            )
            .expect("append");
        assert!(matches!(scan.finish(), Err(HistoryPageError::Stale)));
    }
}
