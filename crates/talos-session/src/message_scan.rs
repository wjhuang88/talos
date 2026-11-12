//! Incremental host message restoration, separate from model search authority.

use std::path::PathBuf;
use std::time::Instant;

use talos_core::message::Message;

use crate::history_page::{HistoryPageError, MessagePageBuffer, open_source};
use crate::jsonl::HistoryMessageDecoder;
use crate::recovery_stamp::FileStamp;

/// A paged restore of a Session's messages, including legacy JSONL envelopes.
///
/// Each step reads at most the history page byte/entry bounds. The final message
/// vector still occupies memory proportional to history. This is not a durable
/// provenance receipt and cannot authorize history search or model disclosure.
pub struct SessionMessageScan {
    path: PathBuf,
    buffer: MessagePageBuffer,
    stamp: Option<FileStamp>,
    end: u64,
    offset: u64,
    copy_offset: u64,
    synthetic_counter: u64,
    decoder: HistoryMessageDecoder,
    messages: Vec<Message>,
    last_entry: Option<crate::SessionEntry>,
    complete: bool,
    archive_placeholder: bool,
    through_entry: Option<String>,
    selected_end: Option<u64>,
    last_entry_end: u64,
}

impl crate::Session {
    /// Capture an original-byte prefix ending at one unique decoded entry ID.
    ///
    /// The entire captured source is still validated, including duplicate target
    /// IDs and archive placeholders. Finish returns an error if the ID is absent.
    /// Copied pages must be truncated to the length returned by
    /// [`SessionMessageScan::finish_with_copy_length`] before publication.
    pub fn begin_message_scan_through(
        &self,
        entry_id: &str,
    ) -> Result<SessionMessageScan, HistoryPageError> {
        let mut scan = self.begin_message_scan()?;
        scan.through_entry = Some(entry_id.to_owned());
        Ok(scan)
    }

    /// Start a bounded-page message restore using the configured store encoding.
    ///
    /// Custom stores must opt into a built-in encoding; otherwise this returns
    /// unavailable, never a whole-file fallback. Missing new-session files are
    /// empty. Malformed records are content-free errors, except legacy JSONL's
    /// skipped invalid lines and an incomplete final TLOG record. Large records
    /// are accumulated across bounded reads. Memory remains proportional to the
    /// restored messages, not the I/O page size. Nothing is published before finish.
    pub fn begin_message_scan(&self) -> Result<SessionMessageScan, HistoryPageError> {
        let encoding = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.store.history_encoding()
        }))
        .map_err(|_| HistoryPageError::UnsupportedSource)?
        .ok_or(HistoryPageError::UnsupportedSource)?;
        let stamp = match open_source(&self.file_path) {
            Ok(file) => Some(crate::recovery_stamp::from_file(&file)?),
            Err(HistoryPageError::Io(error))
                if error.kind() == std::io::ErrorKind::NotFound && !self.persisted =>
            {
                None
            }
            Err(error) => return Err(error),
        };
        let end = match &stamp {
            Some(stamp) => {
                crate::recovery_stamp::length(stamp).ok_or(HistoryPageError::UnsupportedSource)?
            }
            None => 0,
        };
        Ok(SessionMessageScan {
            path: self.file_path.clone(),
            buffer: MessagePageBuffer::new(encoding, end),
            stamp,
            end,
            offset: 0,
            copy_offset: 0,
            synthetic_counter: 0,
            decoder: Default::default(),
            messages: Vec::new(),
            last_entry: None,
            complete: false,
            archive_placeholder: false,
            through_entry: None,
            selected_end: None,
            last_entry_end: 0,
        })
    }
}

impl SessionMessageScan {
    /// Whether a decoded record is the legacy count-only archive placeholder.
    /// Hosts should restore its archived sources rather than treat counts as a
    /// semantic summary. This flag never authorizes an archive by itself.
    pub fn has_archive_placeholder(&self) -> bool {
        self.archive_placeholder
    }
    /// Read the next raw copy page from this same captured source version.
    ///
    /// Pages contain at most 2 MiB. Cancellation or source changes do not advance
    /// the copy cursor. The host must discard any staged output on error and must
    /// finish both copying and message decoding before publishing a fork. Raw
    /// bytes include compatibility suffixes and grant no model search authority.
    pub fn copy_next_page(
        &mut self,
        deadline: Instant,
        stopped: impl Fn() -> bool,
    ) -> Result<Option<Vec<u8>>, HistoryPageError> {
        use std::io::{Read, Seek, SeekFrom};

        let check = || {
            if stopped() || Instant::now() >= deadline {
                Err(HistoryPageError::Stopped)
            } else {
                Ok(())
            }
        };
        check()?;
        self.check_current()?;
        if self.copy_offset == self.end {
            return Ok(None);
        }
        let mut file = open_source(&self.path)?;
        let expected = self.stamp.as_ref().ok_or(HistoryPageError::Stale)?;
        let opened = crate::recovery_stamp::from_file(&file)?;
        if !crate::recovery_seal::same_file_version(expected, &opened)? {
            return Err(HistoryPageError::Stale);
        }
        file.seek(SeekFrom::Start(self.copy_offset))?;
        let amount = (self.end - self.copy_offset).min(2 * 1024 * 1024) as usize;
        let mut bytes = vec![0; amount];
        for chunk in bytes.chunks_mut(8192) {
            check()?;
            file.read_exact(chunk)?;
        }
        let after = crate::recovery_stamp::from_file(&file)?;
        if !crate::recovery_seal::same_file_version(expected, &after)? {
            return Err(HistoryPageError::Stale);
        }
        self.check_current()?;
        check()?;
        self.copy_offset += bytes.len() as u64;
        Ok(Some(bytes))
    }

    /// Restore one page, preserving cursor and tool-pairing state on cancellation.
    /// Deadlines are cooperative between bounded reads/records; kernel I/O cannot
    /// be forcibly interrupted. A stopped step can be retried on this scan.
    pub fn step(
        &mut self,
        deadline: Instant,
        stopped: impl Fn() -> bool,
    ) -> Result<bool, HistoryPageError> {
        let check = || {
            if stopped() || Instant::now() >= deadline {
                Err(HistoryPageError::Stopped)
            } else {
                Ok(())
            }
        };
        check()?;
        self.check_current()?;
        if self.complete {
            return Ok(true);
        }
        // SessionManager creates an empty file before the first append writes
        // the TLOG header. This is a valid empty persisted session.
        if self.end == 0 {
            self.complete = true;
            return Ok(true);
        }
        let mut counter = self.synthetic_counter;
        let page = self
            .buffer
            .read(&self.path, self.offset, deadline, &stopped, &mut counter)?;
        let mut last_entry = None;
        let mut last_entry_end = self.last_entry_end;
        let mut selected_end = self.selected_end;
        let mut decoder = self.decoder.clone();
        let mut messages = Vec::new();
        let mut archive_placeholder = self.archive_placeholder;
        for (entry, range) in page.entries.into_iter().zip(page.record_ranges) {
            check()?;
            archive_placeholder |= crate::archive_recovery::placeholder(&entry);
            let selected = self.through_entry.as_deref() == Some(entry.id.as_str());
            if selected && selected_end.is_some() {
                return Err(HistoryPageError::InvalidRecord);
            }
            if selected_end.is_none() {
                last_entry = Some(entry.clone());
                last_entry_end = range.end;
                if let Some((_, message)) = decoder.decode(entry) {
                    messages.push(message);
                }
            }
            if selected {
                selected_end = Some(range.end);
            }
        }
        self.check_current()?;
        check()?;
        self.messages.extend(messages);
        self.archive_placeholder = archive_placeholder;
        self.selected_end = selected_end;
        self.last_entry_end = last_entry_end;
        if last_entry.is_some() {
            self.last_entry = last_entry;
        }
        self.decoder = decoder;
        self.synthetic_counter = counter;
        self.buffer
            .commit((page.next_offset - self.offset) as usize);
        self.offset = page.next_offset;
        self.complete = page.complete;
        Ok(self.complete)
    }

    /// Return only a complete restore after checking source identity again.
    /// Completing or dropping the scan releases its retained source lease.
    pub fn finish(self) -> Result<Vec<Message>, HistoryPageError> {
        self.finish_with_tail().map(|(messages, _)| messages)
    }

    /// Return the messages and last successfully decoded entry from the same validated
    /// scan. The tail includes filtered diagnostic/control entries; invalid JSONL
    /// lines and incomplete TLOG tails are skipped, and legacy envelopes can have
    /// synthetic IDs. It is not proof of the final physical bytes. Neither value
    /// grants model search authority; callers must enforce their host transition
    /// fence before interpreting activation metadata or publishing a new Actor.
    pub fn finish_with_tail(
        self,
    ) -> Result<(Vec<Message>, Option<crate::SessionEntry>), HistoryPageError> {
        self.finish_with_copy_length()
            .map(|(messages, tail, _)| (messages, tail))
    }

    /// Finish with the raw-copy cutoff at the last retained decoded entry.
    ///
    /// Unlike ordinary physical-EOF copying, this excludes skipped compatibility
    /// suffixes. The entire captured source is still validated. Empty scans return
    /// zero; callers must reject empty histories before publishing a fork.
    /// This grants no provenance or fork authority.
    pub fn finish_with_record_copy_length(
        self,
    ) -> Result<(Vec<Message>, Option<crate::SessionEntry>, u64), HistoryPageError> {
        let cutoff = self.last_entry_end;
        self.finish_with_copy_length()
            .map(|(messages, tail, _)| (messages, tail, cutoff))
    }

    /// Finish a validated scan and return the exact raw-copy cutoff.
    ///
    /// A selected prefix ends immediately after its original record, excluding
    /// later bytes. An ordinary scan retains the full captured file, including
    /// compatibility tails. This length grants no provenance or fork authority.
    pub fn finish_with_copy_length(
        self,
    ) -> Result<(Vec<Message>, Option<crate::SessionEntry>, u64), HistoryPageError> {
        if !self.complete {
            return Err(HistoryPageError::Stopped);
        }
        if self.through_entry.is_some() && self.selected_end.is_none() {
            return Err(HistoryPageError::InvalidRecord);
        }
        self.check_current()?;
        Ok((
            self.messages,
            self.last_entry,
            self.selected_end.unwrap_or(self.end),
        ))
    }

    fn check_current(&self) -> Result<(), HistoryPageError> {
        match (&self.stamp, open_source(&self.path)) {
            (None, Err(HistoryPageError::Io(error)))
                if error.kind() == std::io::ErrorKind::NotFound =>
            {
                Ok(())
            }
            (Some(stamp), Ok(file)) => {
                let current = crate::recovery_stamp::from_file(&file)?;
                if crate::recovery_seal::same_file_version(stamp, &current)? {
                    Ok(())
                } else {
                    Err(HistoryPageError::Stale)
                }
            }
            (_, Err(error)) => Err(error),
            _ => Err(HistoryPageError::Stale),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history_page::{HISTORY_PAGE_BYTES, HistoryEncoding};
    use crate::{Session, SessionMetadata, SessionStore};
    use std::sync::Arc;
    use std::time::Duration;
    use talos_core::message::{MessageToolResult, ToolCall};

    #[test]
    fn selected_prefix_tracks_large_original_record_ranges_in_both_formats() {
        for extension in ["tlog", "jsonl"] {
            let root = tempfile::tempdir().expect("root");
            let source = session(root.path().join(format!("source.{extension}")));
            source
                .append(&Message::User {
                    content: "中".repeat(HISTORY_PAGE_BYTES / 3 + 100),
                })
                .expect("large first");
            let prefix = source.snapshot_bytes().expect("prefix");
            let id = source.read_entries().expect("entries")[0].id.clone();
            source
                .append(&Message::User {
                    content: "excluded suffix".into(),
                })
                .expect("suffix");
            let mut scan = source.begin_message_scan_through(&id).expect("capture");
            let deadline = Instant::now() + Duration::from_secs(10);
            while !scan.step(deadline, || false).expect("page") {}
            let (messages, tail, cutoff) = scan.finish_with_copy_length().expect("finish");
            assert_eq!(messages.len(), 1);
            assert_eq!(tail.expect("tail").id, id);
            assert_eq!(cutoff, prefix.len() as u64);

            let mut missing = source
                .begin_message_scan_through("missing")
                .expect("capture");
            while !missing.step(deadline, || false).expect("page") {}
            assert!(missing.finish_with_copy_length().is_err());
            let duplicate = source.read_entries().expect("entries")[0].clone();
            let store: Arc<dyn SessionStore> = if extension == "tlog" {
                Arc::new(crate::CompactTextSessionStore)
            } else {
                Arc::new(crate::JsonlSessionStore)
            };
            store
                .append_entry(&source.file_path, &duplicate)
                .expect("duplicate target fixture");
            let mut duplicate = source.begin_message_scan_through(&id).expect("capture");
            loop {
                match duplicate.step(deadline, || false) {
                    Ok(false) => continue,
                    Err(HistoryPageError::InvalidRecord) => break,
                    other => panic!("duplicate target must fail: {other:?}"),
                }
            }
        }
    }

    fn session(path: PathBuf) -> Session {
        Session::new(uuid::Uuid::new_v4(), "test".into(), "test".into(), path)
    }

    fn finish(mut scan: SessionMessageScan) -> Vec<Message> {
        while !scan
            .step(Instant::now() + Duration::from_secs(5), || false)
            .expect("page")
        {}
        scan.finish().expect("complete history")
    }

    #[test]
    fn both_encodings_preserve_pairing_across_entry_and_byte_pages() {
        for extension in ["jsonl", "tlog"] {
            let root = tempfile::tempdir().expect("root");
            let owner = session(root.path().join(format!("history.{extension}")));
            let mut messages: Vec<_> = (0..8191)
                .map(|_| Message::User {
                    content: "x".into(),
                })
                .collect();
            messages.push(Message::Assistant {
                content: String::new(),
                reasoning: None,
                tool_calls: vec![ToolCall {
                    id: "cross-page".into(),
                    name: "read".into(),
                    input: serde_json::json!({}),
                }],
            });
            messages.push(Message::Tool {
                result: MessageToolResult {
                    tool_use_id: "cross-page".into(),
                    content: "original result".into(),
                    is_error: false,
                },
            });
            messages.extend((0..9000).map(|_| Message::User {
                content: "next ".repeat(100),
            }));
            let entries: Vec<_> = messages
                .iter()
                .enumerate()
                .map(|(i, message)| {
                    let (role, content) = crate::jsonl::message_parts(message);
                    crate::SessionEntry {
                        id: i.to_string(),
                        parent_id: None,
                        timestamp: chrono::Utc::now(),
                        role,
                        content,
                        metadata: SessionMetadata::default(),
                    }
                })
                .collect();
            owner
                .store
                .replace_entries_atomically(&owner.file_path, &entries)
                .expect("fixture");
            assert!(std::fs::metadata(&owner.file_path).expect("length").len() > 2 * 1024 * 1024);
            let expected = owner.read_messages().expect("existing decoder");
            let mut scan = owner.begin_message_scan().expect("begin");
            assert!(
                !scan
                    .step(Instant::now() + Duration::from_secs(5), || false)
                    .expect("first page")
            );
            assert_eq!(scan.messages.len(), 8192, "pair straddles first entry page");
            assert!(matches!(
                scan.step(Instant::now(), || false),
                Err(HistoryPageError::Stopped)
            ));
            assert_eq!(
                scan.messages.len(),
                8192,
                "stopped step cannot publish partial state"
            );
            assert_eq!(finish(scan), expected);
            assert_eq!(expected, messages);
        }
    }

    #[test]
    fn raw_copy_pages_preserve_bytes_and_retry_cancelled_cursor() {
        let root = tempfile::tempdir().expect("root");
        let owner = session(root.path().join("copy.jsonl"));
        let bytes = b"not-json\n".repeat(300_000);
        std::fs::write(&owner.file_path, &bytes).expect("source");
        let mut scan = owner.begin_message_scan().expect("capture");
        let calls = std::cell::Cell::new(0);
        assert!(matches!(
            scan.copy_next_page(Instant::now() + Duration::from_secs(5), || {
                calls.set(calls.get() + 1);
                calls.get() > 5
            }),
            Err(HistoryPageError::Stopped)
        ));
        assert_eq!(scan.copy_offset, 0);
        let mut copied = Vec::new();
        let mut pages = 0;
        while let Some(page) = scan
            .copy_next_page(Instant::now() + Duration::from_secs(5), || false)
            .expect("copy")
        {
            assert!(page.len() <= 2 * 1024 * 1024);
            copied.extend(page);
            pages += 1;
        }
        assert_eq!(pages, 2);
        assert_eq!(copied, bytes);
        assert!(
            finish(scan).is_empty(),
            "copy and decoding use independent cursors"
        );
    }

    #[cfg(unix)]
    #[test]
    fn raw_copy_rejects_source_change_between_pages() {
        let root = tempfile::tempdir().expect("root");
        let owner = session(root.path().join("copy.jsonl"));
        std::fs::write(&owner.file_path, vec![b'\n'; 3 * 1024 * 1024]).expect("source");
        let mut scan = owner.begin_message_scan().expect("capture");
        assert!(
            scan.copy_next_page(Instant::now() + Duration::from_secs(5), || false)
                .expect("first")
                .is_some()
        );
        std::fs::write(&owner.file_path, vec![b' '; 3 * 1024 * 1024]).expect("replace bytes");
        assert!(matches!(
            scan.copy_next_page(Instant::now() + Duration::from_secs(5), || false),
            Err(HistoryPageError::Stale)
        ));
        assert_eq!(scan.copy_offset, 2 * 1024 * 1024);
    }

    #[test]
    fn headerless_legacy_tlog_restores_messages_without_granting_authority() {
        let root = tempfile::tempdir().expect("root");
        let owner = session(root.path().join("legacy.tlog"));
        let entry = crate::SessionEntry {
            id: "original".into(),
            parent_id: None,
            timestamp: chrono::Utc::now(),
            role: "user".into(),
            content: "legacy\nmessage".into(),
            metadata: Default::default(),
        };
        let bytes = crate::compact_text::encode_entry(&entry).into_bytes();
        std::fs::write(&owner.file_path, &bytes).expect("legacy fixture");
        assert_eq!(
            finish(owner.begin_message_scan().expect("scan")),
            vec![Message::User {
                content: entry.content
            }]
        );
        assert_eq!(std::fs::read(&owner.file_path).expect("unchanged"), bytes);
        assert!(
            crate::history_page::read_history_page(
                &owner.file_path,
                HistoryEncoding::Tlog,
                0,
                bytes.len() as u64,
                Instant::now() + Duration::from_secs(5),
                || false,
            )
            .is_err(),
            "compatibility must not grant strict recovery authority"
        );
    }

    #[test]
    fn newly_created_empty_persisted_tlog_restores_without_a_header() {
        let root = tempfile::tempdir().expect("root");
        let manager = crate::SessionManager::with_dir(root.path().join("sessions"));
        let owner = manager
            .create_session("empty", "workspace")
            .expect("session");
        assert_eq!(std::fs::metadata(&owner.file_path).expect("file").len(), 0);
        assert!(finish(owner.begin_message_scan().expect("scan")).is_empty());
        owner
            .append(&Message::User {
                content: "first append".into(),
            })
            .expect("append");
        assert_eq!(
            finish(owner.begin_message_scan().expect("scan after append")),
            vec![Message::User {
                content: "first append".into()
            }]
        );
    }

    #[test]
    fn snapshot_tail_survives_empty_trailing_pages_and_releases_append_lease() {
        use std::io::Write;

        for extension in ["jsonl", "tlog"] {
            let root = tempfile::tempdir().expect("root");
            let owner = session(root.path().join(format!("history.{extension}")));
            let mut entries: Vec<_> = (0..8200)
                .map(|i| crate::SessionEntry {
                    id: format!("entry-{i}"),
                    parent_id: None,
                    timestamp: chrono::Utc::now(),
                    role: "user".into(),
                    content: format!("message-{i}"),
                    metadata: Default::default(),
                })
                .collect();
            entries.push(crate::SessionEntry {
                id: "activation-tail".into(),
                parent_id: None,
                timestamp: chrono::Utc::now(),
                role: "diagnostic".into(),
                content: "transition marker".into(),
                metadata: Default::default(),
            });
            owner
                .store
                .replace_entries_atomically(&owner.file_path, &entries)
                .expect("fixture");
            let valid_length = std::fs::metadata(&owner.file_path)
                .expect("valid length")
                .len();
            {
                let mut file = std::fs::OpenOptions::new()
                    .append(true)
                    .open(&owner.file_path)
                    .expect("append fixture");
                if extension == "jsonl" {
                    // Force pages with no decoded entries after the logical tail.
                    file.write_all("not-json\n".repeat(300_000).as_bytes())
                        .expect("invalid suffix");
                } else {
                    file.write_all(b"E\t0\t1000\tid\t-\t4:tail")
                        .expect("incomplete suffix");
                }
            }
            let original = std::fs::read(&owner.file_path).expect("original bytes");
            let mut scan = owner.begin_message_scan().expect("capture");
            let mut pages = 0;
            loop {
                pages += 1;
                if scan
                    .step(Instant::now() + Duration::from_secs(10), || false)
                    .expect("page")
                {
                    break;
                }
            }
            assert!(pages > 1);
            let (messages, tail, cutoff) = scan.finish_with_record_copy_length().expect("snapshot");
            assert_eq!(cutoff, valid_length);
            let mut physical = owner.begin_message_scan().expect("physical capture");
            while !physical
                .step(Instant::now() + Duration::from_secs(10), || false)
                .expect("physical page")
            {}
            assert_eq!(
                physical
                    .finish_with_copy_length()
                    .expect("physical snapshot")
                    .2,
                original.len() as u64
            );
            assert_eq!(messages.len(), 8200);
            assert_eq!(tail.expect("diagnostic tail").id, "activation-tail");
            assert_eq!(
                std::fs::read(&owner.file_path).expect("unchanged"),
                original
            );
            // In particular, Windows must release its retained read lease before
            // the transition can append the next activation marker.
            std::fs::OpenOptions::new()
                .append(true)
                .open(&owner.file_path)
                .expect("snapshot finish releases writer exclusion")
                .write_all(b"\n")
                .expect("append after finish");
        }
    }

    #[test]
    fn legacy_envelopes_and_invalid_lines_match_existing_decoder_without_authority() {
        let root = tempfile::tempdir().expect("root");
        let owner = session(root.path().join("legacy.jsonl"));
        let mut text = String::new();
        for i in 0..8200 {
            let message = match i {
                8191 => Message::Assistant {
                    content: String::new(),
                    reasoning: None,
                    tool_calls: vec![ToolCall {
                        id: "legacy-call".into(),
                        name: "read".into(),
                        input: serde_json::json!({}),
                    }],
                },
                8192 => Message::Tool {
                    result: MessageToolResult {
                        tool_use_id: "legacy-call".into(),
                        content: "legacy result".into(),
                        is_error: false,
                    },
                },
                _ => Message::User {
                    content: format!("legacy {i}"),
                },
            };
            let value = serde_json::json!({"type":"message", "data":message});
            text.push_str(&value.to_string());
            text.push('\n');
            text.push_str("not-json\n");
        }
        text.push_str("{\"type\":");
        std::fs::write(&owner.file_path, text).expect("fixture");
        assert_eq!(
            finish(owner.begin_message_scan().expect("scan")),
            owner.read_messages().expect("legacy parity")
        );
        // The strict recovery reader still refuses compatibility records.
        assert!(
            crate::history_page::read_history_page(
                &owner.file_path,
                HistoryEncoding::Jsonl,
                0,
                std::fs::metadata(&owner.file_path).expect("length").len(),
                Instant::now() + Duration::from_secs(5),
                || false
            )
            .is_err()
        );
    }

    #[test]
    fn cancellation_mid_page_can_retry_without_duplicate_messages() {
        let root = tempfile::tempdir().expect("root");
        let owner = session(root.path().join("history.jsonl"));
        let entry = crate::SessionEntry {
            id: "one".into(),
            parent_id: None,
            timestamp: chrono::Utc::now(),
            role: "user".into(),
            content: "kept".into(),
            metadata: Default::default(),
        };
        crate::JsonlSessionStore
            .replace_entries_atomically(&owner.file_path, &[entry])
            .expect("fixture");
        let valid_end = std::fs::metadata(&owner.file_path).expect("length").len();
        // Exercise cancellation at each callback boundary, including after local
        // decoding but before the page transaction commits its record cutoff.
        for stop_at in 1..=12 {
            let mut scan = owner.begin_message_scan_through("one").expect("scan");
            let calls = std::cell::Cell::new(0);
            match scan.step(Instant::now() + Duration::from_secs(5), || {
                calls.set(calls.get() + 1);
                calls.get() >= stop_at
            }) {
                Err(HistoryPageError::Stopped) => {
                    assert_eq!(scan.offset, 0);
                    assert_eq!(scan.last_entry_end, 0);
                    assert!(scan.selected_end.is_none());
                    assert!(scan.messages.is_empty());
                }
                Ok(true) => {}
                other => panic!("unexpected step: {other:?}"),
            }
            while !scan
                .step(Instant::now() + Duration::from_secs(5), || false)
                .expect("retry")
            {}
            let (messages, tail, cutoff) = scan.finish_with_record_copy_length().expect("finish");
            assert_eq!(
                messages,
                vec![Message::User {
                    content: "kept".into()
                }]
            );
            assert_eq!(tail.expect("tail").id, "one");
            assert_eq!(cutoff, valid_end);
        }
    }

    #[test]
    fn missing_deferred_session_is_empty_but_missing_persisted_history_is_error() {
        let root = tempfile::tempdir().expect("root");
        let mut owner = session(root.path().join("new.tlog"));
        assert!(owner.begin_message_scan().is_err());
        owner.persisted = false;
        assert!(finish(owner.begin_message_scan().expect("deferred")).is_empty());
        let scan = owner.begin_message_scan().expect("absent snapshot");
        std::fs::write(&owner.file_path, b"new source").expect("concurrent creation");
        assert!(finish_error(scan));
    }

    fn finish_error(mut scan: SessionMessageScan) -> bool {
        scan.step(Instant::now() + Duration::from_secs(5), || false)
            .is_err()
    }

    #[test]
    fn incomplete_tlog_tail_never_allocates_from_its_declared_length() {
        let root = tempfile::tempdir().expect("root");
        let owner = session(root.path().join("history.tlog"));
        std::fs::write(
            &owner.file_path,
            b"TALOS\t1\t1000\nE\t0\t1000\tid\t-\t4:tail",
        )
        .expect("partial tail");
        assert_eq!(
            finish(owner.begin_message_scan().expect("scan")),
            owner.read_messages().expect("legacy")
        );
        std::fs::write(
            &owner.file_path,
            b"TALOS\t1\t1000\nE\t0\t1000\tid\t-\t2097153:x",
        )
        .expect("incomplete large record");
        assert!(finish(owner.begin_message_scan().expect("scan")).is_empty());
    }

    #[test]
    fn large_writer_records_restore_across_pages_without_expanding_model_authority() {
        for extension in ["tlog", "jsonl"] {
            let root = tempfile::tempdir().expect("root");
            let owner = session(root.path().join(format!("history.{extension}")));
            let content = "large 界\n\t".repeat(HISTORY_PAGE_BYTES / 4);
            let entries: Vec<_> = ["before", content.as_str(), "after"]
                .into_iter()
                .enumerate()
                .map(|(i, content)| crate::SessionEntry {
                    id: i.to_string(),
                    parent_id: None,
                    timestamp: chrono::Utc::now(),
                    role: "user".into(),
                    content: content.into(),
                    metadata: Default::default(),
                })
                .collect();
            owner
                .store
                .replace_entries_atomically(&owner.file_path, &entries)
                .expect("writer");
            let original = std::fs::read(&owner.file_path).expect("bytes");
            let expected = owner.read_messages().expect("legacy reader");
            let mut scan = owner.begin_message_scan().expect("scan");
            let mut steps = 0;
            loop {
                let before = scan.offset + scan.buffer.bytes_len() as u64;
                let complete = scan
                    .step(Instant::now() + Duration::from_secs(5), || false)
                    .expect("step");
                let after = scan.offset + scan.buffer.bytes_len() as u64;
                assert!(after - before <= HISTORY_PAGE_BYTES as u64);
                steps += 1;
                if complete {
                    break;
                }
                // Retrying a cancelled step cannot duplicate or lose a record.
                let position = scan.offset;
                assert!(matches!(
                    scan.step(Instant::now() + Duration::from_secs(5), || true),
                    Err(HistoryPageError::Stopped)
                ));
                assert_eq!(scan.offset, position);
            }
            assert!(steps >= 3);
            assert_eq!(scan.finish().expect("complete"), expected);
            assert_eq!(
                std::fs::read(&owner.file_path).expect("unchanged"),
                original
            );
            // The stricter model index still refuses the same large record.
            let mut offset = 0;
            loop {
                match crate::history_page::read_history_page(
                    &owner.file_path,
                    owner.store.history_encoding().expect("encoding"),
                    offset,
                    original.len() as u64,
                    Instant::now() + Duration::from_secs(5),
                    || false,
                ) {
                    Err(HistoryPageError::RecordTooLarge) => break,
                    Ok(page) if !page.complete => offset = page.next_offset,
                    other => panic!("large record must not acquire model authority: {other:?}"),
                }
            }
        }
    }

    #[test]
    fn cancellation_during_large_record_accumulation_retries_exactly() {
        let root = tempfile::tempdir().expect("root");
        let owner = session(root.path().join("history.tlog"));
        let entry = crate::SessionEntry {
            id: "large".into(),
            parent_id: None,
            timestamp: chrono::Utc::now(),
            role: "user".into(),
            content: "x".repeat(3 * HISTORY_PAGE_BYTES),
            metadata: Default::default(),
        };
        owner
            .store
            .append_entry(&owner.file_path, &entry)
            .expect("writer");
        let mut scan = owner.begin_message_scan().expect("scan");
        for _ in 0..2 {
            assert!(
                !scan
                    .step(Instant::now() + Duration::from_secs(5), || false)
                    .expect("partial")
            );
        }
        assert!(scan.buffer.bytes_len() > HISTORY_PAGE_BYTES);
        let before = (scan.offset, scan.buffer.bytes_len());
        let checks = std::cell::Cell::new(0);
        let stopped = || {
            checks.set(checks.get() + 1);
            checks.get() == 8
        };
        assert!(matches!(
            scan.step(Instant::now() + Duration::from_secs(5), stopped),
            Err(HistoryPageError::Stopped)
        ));
        assert_eq!((scan.offset, scan.buffer.bytes_len()), before);
        assert_eq!(finish(scan), owner.read_messages().expect("legacy parity"));
    }

    #[cfg(unix)]
    #[test]
    fn large_record_partial_read_cannot_publish_a_changed_source() {
        let root = tempfile::tempdir().expect("root");
        let owner = session(root.path().join("history.tlog"));
        let entry = crate::SessionEntry {
            id: "large".into(),
            parent_id: None,
            timestamp: chrono::Utc::now(),
            role: "user".into(),
            content: "x".repeat(3 * HISTORY_PAGE_BYTES),
            metadata: Default::default(),
        };
        owner
            .store
            .append_entry(&owner.file_path, &entry)
            .expect("writer");
        let mut scan = owner.begin_message_scan().expect("scan");
        assert!(
            !scan
                .step(Instant::now() + Duration::from_secs(5), || false)
                .expect("partial")
        );
        assert!(matches!(
            scan.finish_with_tail(),
            Err(HistoryPageError::Stopped)
        ));
        let mut scan = owner.begin_message_scan().expect("scan");
        assert!(
            !scan
                .step(Instant::now() + Duration::from_secs(5), || false)
                .expect("partial")
        );
        owner
            .store
            .append_entry(&owner.file_path, &entry)
            .expect("concurrent append");
        assert!(matches!(
            scan.step(Instant::now() + Duration::from_secs(5), || false),
            Err(HistoryPageError::Stale)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn changed_source_cannot_publish_partial_history() {
        let root = tempfile::tempdir().expect("root");
        let owner = session(root.path().join("history.jsonl"));
        std::fs::write(&owner.file_path, b"\n").expect("source");
        let mut scan = owner.begin_message_scan().expect("scan");
        assert!(
            scan.step(Instant::now() + Duration::from_secs(5), || false)
                .expect("page")
        );
        std::fs::write(&owner.file_path, b"\n\n").expect("mutation");
        assert!(matches!(scan.finish(), Err(HistoryPageError::Stale)));
    }

    #[test]
    fn store_encoding_is_explicit_not_inferred_from_path_extension() {
        let root = tempfile::tempdir().expect("root");
        let owner = Session::with_store(
            uuid::Uuid::new_v4(),
            "test".into(),
            "test".into(),
            root.path().join("mismatched.tlog"),
            Arc::new(crate::JsonlSessionStore),
        );
        std::fs::write(&owner.file_path, b"\n").expect("empty jsonl");
        assert!(finish(owner.begin_message_scan().expect("explicit store")).is_empty());
    }
    #[test]
    fn final_jsonl_without_newline_is_restored_but_invalid_utf8_and_tlog_header_fail() {
        let root = tempfile::tempdir().expect("root");
        let owner = session(root.path().join("history.jsonl"));
        let raw = serde_json::json!({"type":"message", "data": Message::User { content: "last message".into() }}).to_string();
        std::fs::write(&owner.file_path, raw).expect("no newline");
        assert_eq!(
            finish(owner.begin_message_scan().expect("scan")),
            owner.read_messages().expect("parity")
        );
        std::fs::write(&owner.file_path, [0xff, b'\n']).expect("invalid utf8");
        assert!(finish_error(owner.begin_message_scan().expect("scan")));
        let malformed = session(root.path().join("bad.tlog"));
        std::fs::write(&malformed.file_path, b"private invalid header\n").expect("header");
        assert!(finish_error(malformed.begin_message_scan().expect("scan")));
    }

    #[cfg(unix)]
    #[test]
    fn same_length_replacement_is_not_the_captured_source() {
        let root = tempfile::tempdir().expect("root");
        let owner = session(root.path().join("history.jsonl"));
        std::fs::write(&owner.file_path, b"\n").expect("source");
        let scan = owner.begin_message_scan().expect("scan");
        let replacement = root.path().join("replacement");
        std::fs::write(&replacement, b"\n").expect("replacement");
        std::fs::rename(replacement, &owner.file_path).expect("replace");
        assert!(finish_error(scan));
    }

    #[cfg(windows)]
    #[test]
    fn message_scan_releases_writer_exclusion_after_finish_and_drop() {
        let root = tempfile::tempdir().expect("root");
        let owner = session(root.path().join("history.jsonl"));
        std::fs::write(&owner.file_path, b"\n").expect("source");
        for finish_scan in [false, true] {
            let scan = owner.begin_message_scan().expect("lease");
            assert!(
                std::fs::OpenOptions::new()
                    .write(true)
                    .open(&owner.file_path)
                    .is_err()
            );
            if finish_scan {
                assert!(finish(scan).is_empty());
            } else {
                drop(scan);
            }
            assert!(
                std::fs::OpenOptions::new()
                    .write(true)
                    .open(&owner.file_path)
                    .is_ok()
            );
        }
    }
}
