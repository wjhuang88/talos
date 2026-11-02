//! Incremental host message restoration, separate from model search authority.

use std::path::PathBuf;
use std::time::Instant;

use talos_core::message::Message;

use crate::history_page::{HistoryEncoding, HistoryPageError, open_source, read_message_page};
use crate::jsonl::HistoryMessageDecoder;
use crate::recovery_stamp::FileStamp;

/// A paged restore of a Session's messages, including legacy JSONL envelopes.
///
/// Each step reads at most the history page byte/entry bounds. The final message
/// vector still occupies memory proportional to history. This is not a durable
/// provenance receipt and cannot authorize history search or model disclosure.
pub struct SessionMessageScan {
    path: PathBuf,
    encoding: HistoryEncoding,
    stamp: Option<FileStamp>,
    end: u64,
    offset: u64,
    synthetic_counter: u64,
    decoder: HistoryMessageDecoder,
    messages: Vec<Message>,
    complete: bool,
}

impl crate::Session {
    /// Start a bounded-page message restore using the configured store encoding.
    ///
    /// Custom stores must opt into a built-in encoding; otherwise this returns
    /// unavailable, never a whole-file fallback. Missing new-session files are
    /// empty. Malformed records are content-free errors, except legacy JSONL's
    /// skipped invalid lines and an incomplete final TLOG record. Oversized
    /// individual records are refused. No history is published before finish.
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
            encoding,
            stamp,
            end,
            offset: 0,
            synthetic_counter: 0,
            decoder: Default::default(),
            messages: Vec::new(),
            complete: false,
        })
    }
}

impl SessionMessageScan {
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
        if self.stamp.is_none() {
            self.complete = true;
            return Ok(true);
        }
        let mut counter = self.synthetic_counter;
        let page = read_message_page(
            &self.path,
            self.encoding,
            self.offset,
            self.end,
            deadline,
            &stopped,
            &mut counter,
        )?;
        let mut decoder = self.decoder.clone();
        let mut messages = Vec::new();
        for entry in page.entries {
            check()?;
            if let Some((_, message)) = decoder.decode(entry) {
                messages.push(message);
            }
        }
        self.check_current()?;
        check()?;
        self.messages.extend(messages);
        self.decoder = decoder;
        self.synthetic_counter = counter;
        self.offset = page.next_offset;
        self.complete = page.complete;
        Ok(self.complete)
    }

    /// Return only a complete restore after checking source identity again.
    /// Completing or dropping the scan releases its retained source lease.
    pub fn finish(self) -> Result<Vec<Message>, HistoryPageError> {
        if !self.complete {
            return Err(HistoryPageError::Stopped);
        }
        self.check_current()?;
        Ok(self.messages)
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
    use crate::{Session, SessionMetadata, SessionStore};
    use std::sync::Arc;
    use std::time::Duration;
    use talos_core::message::{MessageToolResult, ToolCall};

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
        let mut scan = owner.begin_message_scan().expect("scan");
        let calls = std::cell::Cell::new(0);
        assert!(matches!(
            scan.step(Instant::now() + Duration::from_secs(5), || {
                calls.set(calls.get() + 1);
                calls.get() >= 5
            }),
            Err(HistoryPageError::Stopped)
        ));
        assert_eq!(scan.offset, 0);
        assert_eq!(
            finish(scan),
            vec![Message::User {
                content: "kept".into()
            }]
        );
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
    fn incomplete_tlog_tail_is_not_a_message_and_oversized_record_is_not_silently_lost() {
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
        .expect("oversized");
        assert!(finish_error(owner.begin_message_scan().expect("scan")));
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
