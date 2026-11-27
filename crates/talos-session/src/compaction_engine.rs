#![allow(dead_code)]
//! Session compaction engine — freezes segments, applies rules, archives (ADR-037 Mechanism B).

#[cfg(not(feature = "archive-compress-zstd"))]
use crate::compression::NoCompressor;
use crate::compression::SegmentCompressor;
#[cfg(feature = "archive-compress-zstd")]
use crate::compression::ZstdCompressor;
use crate::segment_chain::{ChainMetadata, SegmentMeta, SegmentStatus, chain_path};
use crate::store::SessionStore;
use crate::{SessionEntry, SessionError};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct CompactionEngine {
    store: Arc<dyn SessionStore>,
    compressor: Box<dyn SegmentCompressor>,
    rules: CompactionRules,
    #[cfg(test)]
    publication_byte_limit: Option<usize>,
}

#[derive(Clone)]
pub struct CompactionRules {
    pub tool_result_threshold_turns: usize,
    pub max_tool_result_chars: usize,
    pub remove_old_thinking: bool,
}

impl Default for CompactionRules {
    fn default() -> Self {
        Self {
            tool_result_threshold_turns: 20,
            max_tool_result_chars: 4000,
            remove_old_thinking: true,
        }
    }
}

impl CompactionEngine {
    pub fn new(store: Arc<dyn SessionStore>) -> Self {
        Self {
            store,
            #[cfg(feature = "archive-compress-zstd")]
            compressor: Box::new(ZstdCompressor::default()),
            #[cfg(not(feature = "archive-compress-zstd"))]
            compressor: Box::new(NoCompressor),
            rules: CompactionRules::default(),
            #[cfg(test)]
            publication_byte_limit: None,
        }
    }

    pub fn with_compressor(mut self, compressor: Box<dyn SegmentCompressor>) -> Self {
        self.compressor = compressor;
        self
    }

    #[cfg(test)]
    pub(crate) fn with_publication_byte_limit(mut self, limit: usize) -> Self {
        assert!(limit > 0 && limit <= 64 * 1024 * 1024);
        self.publication_byte_limit = Some(limit);
        self
    }

    pub fn with_rules(mut self, rules: CompactionRules) -> Self {
        self.rules = rules;
        self
    }

    pub fn should_compact(&self, head_path: &Path, max_entries: usize) -> bool {
        if let Ok(entries) = self.store.read_entries(head_path) {
            entries.len() > max_entries
        } else {
            false
        }
    }

    pub fn compact_segment(
        &self,
        head_path: &Path,
        session_dir: &Path,
        max_entries: usize,
    ) -> Result<CompactionResult, SessionError> {
        self.compact_bound(head_path, session_dir, max_entries, None)
    }

    pub(crate) fn compact_bound(
        &self,
        head_path: &Path,
        session_dir: &Path,
        max_entries: usize,
        checkpoint: Option<(&[SessionEntry], &str, &str)>,
    ) -> Result<CompactionResult, SessionError> {
        // Keep retirement excluded between store calls and archive publication.
        // Nested store acquisitions are nonblocking; no waiter is installed.
        let _identity = crate::identity_custody::require_not_retired(head_path)?;
        let entries = self.store.read_entries(head_path)?;
        if let Some((expected, _, _)) = checkpoint
            && (entries.len() != expected.len()
                || entries.iter().zip(expected).any(|(a, b)| {
                    crate::compact_text::encode_entry(a) != crate::compact_text::encode_entry(b)
                }))
        {
            return Err(SessionError::DurableTurn(
                "checkpoint source changed".into(),
            ));
        }
        if entries.len() <= max_entries {
            return Ok(CompactionResult::Skipped);
        }

        let mut receipt_bound = false;
        if matches!(
            self.store.history_encoding(),
            Some(crate::history_page::HistoryEncoding::Tlog)
        ) {
            use std::io::Read;
            let file = crate::history_page::open_source_under_identity(head_path, &_identity)
                .map_err(|_| SessionError::ParseError("compaction source unavailable".into()))?;
            let mut marker = Vec::new();
            file.take(crate::fork_receipt::ENVELOPE.len() as u64)
                .read_to_end(&mut marker)?;
            receipt_bound = marker == crate::fork_receipt::ENVELOPE;
        }

        let owned = matches!(
            self.store.history_encoding(),
            Some(crate::history_page::HistoryEncoding::Tlog)
        ) && head_path
            .file_stem()
            .and_then(|s| s.to_str())
            .is_some_and(|s| uuid::Uuid::parse_str(s).is_ok());
        let segment_id = format!(
            "{}{}",
            if owned { "s-owned-" } else { "s-" },
            uuid::Uuid::new_v4()
        );
        let archive_path = session_dir.join(format!("{segment_id}.tlog"));
        let compressed_path = session_dir.join(format!("{segment_id}.tlog.zst"));

        let original_bytes = std::fs::metadata(head_path).map(|m| m.len()).unwrap_or(0);

        let mut compacted = self.apply_rules(&entries, max_entries);
        if let Some((_, summary, through)) = checkpoint {
            let name = if self.compressor.format_tag() == "none" {
                format!("{segment_id}.tlog")
            } else {
                format!("{segment_id}.tlog.zst")
            };
            compacted[0].content.push_str(&format!(
                "\nArchive: {name}\nCovered through: {through}\n{summary}"
            ));
        }
        let zero_tail = if max_entries == 0
            && matches!(
                self.store.history_encoding(),
                Some(crate::history_page::HistoryEncoding::Tlog)
            )
            && head_path
                .file_stem()
                .and_then(|s| s.to_str())
                .is_some_and(|s| uuid::Uuid::parse_str(s).is_ok())
        {
            Some(crate::compaction_locator::recovered_tail(head_path)?)
        } else {
            None
        };
        let archived_path = self.archive_head(
            head_path,
            &archive_path,
            &compressed_path,
            crate::compact_text::encode_entry(&compacted[0]).len(),
        )?;
        if let Some(tail) = zero_tail {
            crate::compaction_locator::record(head_path, &archived_path, &compacted[0], &tail)?;
            #[cfg(test)]
            crate::durable::fork_crash_probe("zero_registered");
        }

        if receipt_bound {
            crate::compact_text::rewrite_archived_records(
                head_path,
                &entries,
                entries.len() - max_entries,
                &compacted[0],
                &archived_path,
            )?;
        } else {
            self.store.replace_with_retained_suffix(
                head_path,
                &entries,
                entries.len() - max_entries,
                &compacted[..1],
            )?;
        }

        #[cfg(test)]
        if max_entries == 0 {
            crate::durable::fork_crash_probe("zero_published");
        }

        let _record_count = entries.len();
        let archived_bytes = std::fs::metadata(&archived_path)
            .map(|m| m.len())
            .unwrap_or(0);

        self.update_chain(
            session_dir,
            &segment_id,
            _record_count,
            original_bytes,
            archived_bytes,
        )?;

        Ok(CompactionResult::Compacted {
            segment_id,
            original_count: _record_count,
            compacted_count: compacted.len(),
            original_bytes,
            archived_bytes,
        })
    }

    fn archive_head(
        &self,
        head_path: &Path,
        archive_path: &Path,
        compressed_path: &Path,
        marker_bytes: usize,
    ) -> Result<PathBuf, SessionError> {
        let raw = std::fs::read(head_path)?;
        let compressed = self
            .compressor
            .compress(&raw)
            .map_err(|e| SessionError::ParseError(e.to_string()))?;
        let (destination, bytes) = if self.compressor.format_tag() == "none" {
            (archive_path, &raw)
        } else {
            (compressed_path, &compressed)
        };
        if raw.starts_with(crate::fork_receipt::ENVELOPE) {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            let id = head_path
                .file_stem()
                .and_then(|s| s.to_str())
                .and_then(|s| uuid::Uuid::parse_str(s).ok())
                .ok_or_else(|| SessionError::ParseError("fork head identity missing".into()))?;
            let session = crate::Session::new(id, String::new(), String::new(), head_path.into());
            let evidence = session.inspect_archive_recovery(deadline, || false)?;
            // Reserve the full added marker without crediting removed records:
            // a short removed record can be smaller than the new marker.
            let stored_bytes = bytes.len().checked_add(marker_bytes).ok_or_else(|| {
                SessionError::ParseError("archive publication size overflow".into())
            })?;
            let decoded_bytes = if self.compressor.format_tag() == "none" {
                0
            } else {
                raw.len()
            };
            #[cfg(test)]
            if let Some(limit) = self.publication_byte_limit {
                evidence.check_archive_addition_with_limit(stored_bytes, decoded_bytes, limit)?;
            } else {
                evidence.check_archive_addition(stored_bytes, decoded_bytes)?;
            }
            #[cfg(not(test))]
            evidence.check_archive_addition(stored_bytes, decoded_bytes)?;
            evidence.check_all_sources(deadline)?;
        }
        // Only complete, synced bytes receive a discoverable archive name.
        // Hard-link publication is create-new: never replace an existing archive
        // (or a symlink at its path). Unsupported filesystems fail before head
        // replacement, leaving the original transcript intact.
        if destination
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(crate::archive_publication::owned)
        {
            crate::archive_publication::record(head_path, destination, bytes)?;
            #[cfg(test)]
            crate::durable::fork_crash_probe("archive_intent");
        }
        let temporary = crate::store::temporary_sibling(destination);
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        std::fs::hard_link(&temporary, destination)?;
        #[cfg(test)]
        crate::durable::fork_crash_probe("archive_published");
        #[cfg(unix)]
        std::fs::File::open(
            destination
                .parent()
                .ok_or_else(|| SessionError::ParseError("archive directory missing".into()))?,
        )?
        .sync_all()?;
        std::fs::remove_file(&temporary)?;
        Ok(destination.to_path_buf())
    }

    fn apply_rules(&self, entries: &[SessionEntry], keep_recent: usize) -> Vec<SessionEntry> {
        if entries.len() <= keep_recent {
            return entries.to_vec();
        }

        let split = entries.len() - keep_recent;
        let mut result = Vec::new();

        result.push(SessionEntry {
            id: uuid::Uuid::new_v4().to_string(),
            parent_id: None,
            timestamp: chrono::Utc::now(),
            role: "system".into(),
            content: format!(
                "[Compaction summary: {} earlier entries summarized, {} recent entries preserved]",
                split, keep_recent
            ),
            metadata: crate::SessionMetadata::default(),
        });

        for entry in &entries[split..] {
            result.push(entry.clone());
        }

        result
    }

    fn update_chain(
        &self,
        session_dir: &Path,
        segment_id: &str,
        _record_count: usize,
        _orig_bytes: u64,
        archived_bytes: u64,
    ) -> Result<(), SessionError> {
        let chain_file = chain_path(session_dir);
        let mut chain = ChainMetadata::read(&chain_file)
            .map_err(|e| SessionError::ParseError(e.to_string()))?;

        let previous_archive = chain
            .head_segment()
            .and_then(|head| head.prev_segment_id.clone());
        let now = chrono::Utc::now().timestamp_millis();
        chain
            .segments
            .retain(|segment| segment.status != SegmentStatus::Active);
        chain.segments.push(SegmentMeta {
            segment_id: segment_id.to_string(),
            status: SegmentStatus::Compressed,
            prev_segment_id: previous_archive,
            record_count: _record_count,
            orig_bytes: _orig_bytes,
            archived_bytes: Some(archived_bytes),
            created_ts: now,
            archived_ts: Some(now),
            archive_format: Some(self.compressor.format_tag().to_string()),
            ref_count: 0,
        });
        chain.segments.push(SegmentMeta {
            segment_id: "head".to_string(),
            status: SegmentStatus::Active,
            prev_segment_id: Some(segment_id.to_string()),
            record_count: 0,
            orig_bytes: 0,
            archived_bytes: None,
            created_ts: now,
            archived_ts: None,
            archive_format: None,
            ref_count: 0,
        });

        chain
            .write(&chain_file)
            .map_err(|e| SessionError::ParseError(e.to_string()))?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum CompactionResult {
    Skipped,
    Compacted {
        segment_id: String,
        original_count: usize,
        compacted_count: usize,
        original_bytes: u64,
        archived_bytes: u64,
    },
}

#[cfg(test)]
mod tests {
    #[test]
    fn retirement_during_compression_cannot_split_archive_publication() {
        struct RetiringCompressor(std::path::PathBuf);
        impl crate::compression::SegmentCompressor for RetiringCompressor {
            fn compress(
                &self,
                bytes: &[u8],
            ) -> Result<Vec<u8>, crate::compression::CompressionError> {
                assert!(crate::artifacts::remove_session_transcript(&self.0).is_err());
                Ok(bytes.to_vec())
            }
            fn decompress(
                &self,
                bytes: &[u8],
            ) -> Result<Vec<u8>, crate::compression::CompressionError> {
                Ok(bytes.to_vec())
            }
            fn format_tag(&self) -> &'static str {
                "none"
            }
        }
        let root = tempfile::tempdir().expect("root");
        let id = uuid::Uuid::new_v4();
        let head = root.path().join(format!("{id}.tlog"));
        let database = root.path().join("history-bindings.sqlite");
        let incarnation = crate::reserve_session_identity(&database, id).expect("reserve");
        let session = crate::Session::new(id, "test".into(), String::new(), head.clone());
        for text in ["first", "second", "third"] {
            session
                .append(&talos_core::message::Message::User {
                    content: text.into(),
                })
                .expect("append");
        }
        let engine = CompactionEngine::new(Arc::new(CompactTextSessionStore))
            .with_compressor(Box::new(RetiringCompressor(head.clone())));
        assert!(matches!(
            engine
                .compact_segment(&head, root.path(), 1)
                .expect("publication"),
            CompactionResult::Compacted { .. }
        ));
        crate::require_active_session_identity(&database, id, incarnation).expect("still active");
        assert_eq!(session.read_entries().expect("head readable").len(), 2);
        crate::artifacts::remove_session_transcript(&head).expect("retire after publication");
        assert!(crate::require_active_session_identity(&database, id, incarnation).is_err());
    }
    use super::*;
    use crate::{CompactTextSessionStore, SessionStore};

    #[test]
    #[cfg(feature = "archive-compress-zstd")]
    fn compressed_archive_publication_preserves_existing_destination() {
        let root = tempfile::tempdir().expect("root");
        let head = root.path().join("head.tlog");
        let archive = root.path().join("s-test.tlog");
        let compressed = root.path().join("s-test.tlog.zst");
        std::fs::write(&head, b"original compressed source").expect("head");
        std::fs::write(&compressed, b"existing archive").expect("destination");
        let engine = CompactionEngine::new(Arc::new(CompactTextSessionStore));
        assert!(
            engine
                .archive_head(&head, &archive, &compressed, 0)
                .is_err()
        );
        assert_eq!(
            std::fs::read(&compressed).expect("preserved"),
            b"existing archive"
        );
        let fresh = root.path().join("s-fresh.tlog.zst");
        engine
            .archive_head(&head, &archive, &fresh, 0)
            .expect("publish");
        let bytes = std::fs::read(fresh).expect("compressed bytes");
        assert_eq!(
            engine.compressor.decompress(&bytes).expect("decode"),
            b"original compressed source"
        );
        assert_eq!(
            std::fs::read(head).expect("source"),
            b"original compressed source"
        );
    }

    #[test]
    fn archive_publication_never_overwrites_existing_destination() {
        let root = tempfile::tempdir().expect("root");
        let head = root.path().join("head.tlog");
        let archive = root.path().join("s-existing.tlog");
        let compressed = root.path().join("s-existing.tlog.zst");
        std::fs::write(&head, b"original source").expect("head");
        std::fs::write(&archive, b"existing archive").expect("archive");
        let engine = CompactionEngine::new(Arc::new(CompactTextSessionStore))
            .with_compressor(Box::new(crate::compression::NoCompressor));
        assert!(
            engine
                .archive_head(&head, &archive, &compressed, 0)
                .is_err()
        );
        assert_eq!(
            std::fs::read(&archive).expect("preserved"),
            b"existing archive"
        );
        assert_eq!(std::fs::read(&head).expect("source"), b"original source");
        let fresh = root.path().join("s-fresh.tlog");
        engine
            .archive_head(&head, &fresh, &compressed, 0)
            .expect("publish");
        assert_eq!(std::fs::read(fresh).expect("published"), b"original source");
    }

    #[derive(Debug)]
    struct CustomTlogExtensionStore(crate::JsonlSessionStore);

    impl SessionStore for CustomTlogExtensionStore {
        fn read_entries(&self, p: &Path) -> Result<Vec<SessionEntry>, SessionError> {
            self.0.read_entries(p)
        }
        fn append_entry(&self, p: &Path, e: &SessionEntry) -> Result<(), SessionError> {
            self.0.append_entry(p, e)
        }
        fn replace_entries_atomically(
            &self,
            p: &Path,
            e: &[SessionEntry],
        ) -> Result<(), SessionError> {
            self.0.replace_entries_atomically(p, e)
        }
        fn read_last_entry_id(&self, p: &Path) -> Option<String> {
            self.0.read_last_entry_id(p)
        }
        fn scan_file(&self, p: &Path) -> Result<crate::SessionInfo, SessionError> {
            self.0.scan_file(p)
        }
        fn read_bytes(&self, p: &Path) -> Result<Vec<u8>, SessionError> {
            self.0.read_bytes(p)
        }
        fn file_extension(&self) -> &'static str {
            "tlog"
        }
    }

    #[test]
    fn compaction_dispatches_custom_store_and_handles_zero_retention() {
        let root = tempfile::tempdir().expect("root");
        let store = Arc::new(CustomTlogExtensionStore(crate::JsonlSessionStore));
        let path = root.path().join("head.tlog");
        store
            .append_entry(
                &path,
                &SessionEntry {
                    id: uuid::Uuid::new_v4().to_string(),
                    parent_id: None,
                    timestamp: chrono::Utc::now(),
                    role: "user".into(),
                    content: "original".into(),
                    metadata: Default::default(),
                },
            )
            .expect("custom write");
        let engine = CompactionEngine::new(store.clone());
        engine
            .compact_segment(&path, root.path(), 0)
            .expect("custom compaction");
        let entries = store.read_entries(&path).expect("custom read");
        assert_eq!(entries.len(), 1);
        assert!(crate::archive_recovery::placeholder(&entries[0]));
        assert!(
            std::fs::read(&path)
                .expect("custom encoding")
                .starts_with(b"{")
        );
    }

    #[test]
    fn should_compact_returns_false_under_threshold() {
        let store = Arc::new(CompactTextSessionStore);
        let dir = std::env::temp_dir().join("compact_test_threshold");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("operation should succeed");
        let path = dir.join("head.tlog");

        let entry = SessionEntry {
            id: uuid::Uuid::new_v4().to_string(),
            parent_id: None,
            timestamp: chrono::Utc::now(),
            role: "user".into(),
            content: "hello".into(),
            metadata: crate::SessionMetadata::default(),
        };
        store
            .append_entry(&path, &entry)
            .expect("operation should succeed");

        let engine = CompactionEngine::new(store);
        assert!(!engine.should_compact(&path, 10));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn should_compact_returns_true_over_threshold() {
        let store = Arc::new(CompactTextSessionStore);
        let dir = std::env::temp_dir().join("compact_test_over");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("operation should succeed");
        let path = dir.join("head.tlog");

        for _ in 0..5 {
            let entry = SessionEntry {
                id: uuid::Uuid::new_v4().to_string(),
                parent_id: None,
                timestamp: chrono::Utc::now(),
                role: "user".into(),
                content: "test entry".into(),
                metadata: crate::SessionMetadata::default(),
            };
            store
                .append_entry(&path, &entry)
                .expect("operation should succeed");
        }

        let engine = CompactionEngine::new(store);
        assert!(engine.should_compact(&path, 3));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn undecodable_child_cannot_publish_archive_even_after_unchecked_append() {
        // A header-version bump is not a downgrade gate: legacy readers skip
        // TALOS headers without checking the version. Invalid UTF-8 instead
        // fails before record parsing. Exercise the actual writer boundary,
        // including a direct append that does not inspect the existing header.
        let root = tempfile::tempdir().expect("root");
        let path = root.path().join(format!("{}.tlog", uuid::Uuid::new_v4()));
        let store = Arc::new(CompactTextSessionStore);
        let entry = SessionEntry {
            id: uuid::Uuid::new_v4().to_string(),
            parent_id: None,
            timestamp: chrono::Utc::now(),
            role: "user".into(),
            content: "inherited original".into(),
            metadata: crate::SessionMetadata::default(),
        };
        store.append_entry(&path, &entry).expect("seed");
        let original = std::fs::read(&path).expect("original");
        let mut unsupported = vec![0xff];
        unsupported.extend_from_slice(&original);
        std::fs::write(&path, &unsupported).expect("unsupported format");
        let mut suffix = entry.clone();
        suffix.id = uuid::Uuid::new_v4().to_string();
        suffix.parent_id = Some(entry.id.clone());
        suffix.content = "child-only suffix".into();
        store
            .append_entry(&path, &suffix)
            .expect("unchecked append");
        let before = std::fs::read(&path).expect("before refusal");
        assert!(before.starts_with(&unsupported));
        let inventory = || {
            std::fs::read_dir(root.path())
                .expect("inventory")
                .map(|item| item.expect("entry").file_name())
                .collect::<std::collections::BTreeSet<_>>()
        };
        let initial_inventory = inventory();
        let engine = CompactionEngine::new(store.clone());
        for keep in [0, 1] {
            assert!(engine.compact_segment(&path, root.path(), keep).is_err());
            assert_eq!(std::fs::read(&path).expect("unchanged"), before);
            assert_eq!(inventory(), initial_inventory);
        }
        assert!(store.replace_entries_atomically(&path, &[entry]).is_err());
        assert_eq!(std::fs::read(&path).expect("still unchanged"), before);
        assert_eq!(inventory(), initial_inventory);
    }

    #[test]
    fn compact_freezes_and_archives() {
        let store = Arc::new(CompactTextSessionStore);
        let dir = std::env::temp_dir().join("compact_test_freeze");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("operation should succeed");
        let path = dir.join("head.tlog");

        for i in 0..10 {
            let entry = SessionEntry {
                id: uuid::Uuid::new_v4().to_string(),
                parent_id: None,
                timestamp: chrono::Utc::now(),
                role: "user".into(),
                content: format!("entry {i}"),
                metadata: crate::SessionMetadata::default(),
            };
            store
                .append_entry(&path, &entry)
                .expect("operation should succeed");
        }

        let engine = CompactionEngine::new(store);
        let result = engine
            .compact_segment(&path, &dir, 3)
            .expect("operation should succeed");

        match result {
            CompactionResult::Compacted {
                original_count,
                compacted_count,
                ..
            } => {
                assert_eq!(original_count, 10);
                assert!(compacted_count < original_count);
            }
            CompactionResult::Skipped => panic!("should have compacted"),
        }

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn compact_skips_when_under_threshold() {
        let store = Arc::new(CompactTextSessionStore);
        let dir = std::env::temp_dir().join("compact_test_skip");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("operation should succeed");
        let path = dir.join("head.tlog");

        let entry = SessionEntry {
            id: uuid::Uuid::new_v4().to_string(),
            parent_id: None,
            timestamp: chrono::Utc::now(),
            role: "user".into(),
            content: "single entry".into(),
            metadata: crate::SessionMetadata::default(),
        };
        store
            .append_entry(&path, &entry)
            .expect("operation should succeed");

        let engine = CompactionEngine::new(store);
        let result = engine
            .compact_segment(&path, &dir, 10)
            .expect("operation should succeed");
        assert!(matches!(result, CompactionResult::Skipped));

        std::fs::remove_dir_all(&dir).ok();
    }
}
