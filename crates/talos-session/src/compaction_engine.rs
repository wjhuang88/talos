#![allow(dead_code)]
//! Session compaction engine — freezes segments, applies rules, archives (ADR-037 Mechanism B).

use crate::compression::{NoCompressor, SegmentCompressor};
use crate::segment_chain::{ChainMetadata, SegmentMeta, SegmentStatus, chain_path};
use crate::store::SessionStore;
use crate::{SessionEntry, SessionError};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct CompactionEngine {
    store: Arc<dyn SessionStore>,
    compressor: Box<dyn SegmentCompressor>,
    rules: CompactionRules,
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
            compressor: Box::new(NoCompressor),
            rules: CompactionRules::default(),
        }
    }

    pub fn with_compressor(mut self, compressor: Box<dyn SegmentCompressor>) -> Self {
        self.compressor = compressor;
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
        let entries = self.store.read_entries(head_path)?;
        if entries.len() <= max_entries {
            return Ok(CompactionResult::Skipped);
        }

        let segment_id = format!("s{:03}", chrono::Utc::now().timestamp() % 1000);
        let archive_path = session_dir.join(format!("{segment_id}.tlog"));
        let compressed_path = session_dir.join(format!("{segment_id}.tlog.zst"));

        let original_bytes = std::fs::metadata(head_path)
            .map(|m| m.len())
            .unwrap_or(0);

        let archived_path = self.archive_head(head_path, &archive_path, &compressed_path)?;

        let compacted = self.apply_rules(&entries, max_entries);

        std::fs::write(head_path, b"")?;
        for entry in &compacted {
            self.store.append_entry(head_path, entry)?;
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
    ) -> Result<PathBuf, SessionError> {
        let raw = std::fs::read(head_path)?;
        let compressed = self
            .compressor
            .compress(&raw)
            .map_err(|e| SessionError::ParseError(e.to_string()))?;
        std::fs::write(compressed_path, &compressed)?;
        std::fs::write(archive_path, &raw)?;
        Ok(compressed_path.to_path_buf())
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
                split,
                keep_recent
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

        if let Some(head) = chain.segments.iter_mut().find(|s| s.status == SegmentStatus::Active) {
            head.status = SegmentStatus::Compressed;
            head.archived_bytes = Some(archived_bytes);
            head.archived_ts = Some(chrono::Utc::now().timestamp_millis());
            head.archive_format = Some(self.compressor.format_tag().to_string());
        }

        let now = chrono::Utc::now().timestamp_millis();
        chain.segments.push(SegmentMeta {
            segment_id: segment_id.to_string(),
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

        chain.write(&chain_file)
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
    use super::*;
    use crate::{CompactTextSessionStore, SessionStore};

    #[test]
    fn should_compact_returns_false_under_threshold() {
        let store = Arc::new(CompactTextSessionStore);
        let dir = std::env::temp_dir().join("compact_test_threshold");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("head.tlog");

        let entry = SessionEntry {
            id: uuid::Uuid::new_v4().to_string(),
            parent_id: None,
            timestamp: chrono::Utc::now(),
            role: "user".into(),
            content: "hello".into(),
            metadata: crate::SessionMetadata::default(),
        };
        store.append_entry(&path, &entry).unwrap();

        let engine = CompactionEngine::new(store);
        assert!(!engine.should_compact(&path, 10));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn should_compact_returns_true_over_threshold() {
        let store = Arc::new(CompactTextSessionStore);
        let dir = std::env::temp_dir().join("compact_test_over");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
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
            store.append_entry(&path, &entry).unwrap();
        }

        let engine = CompactionEngine::new(store);
        assert!(engine.should_compact(&path, 3));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn compact_freezes_and_archives() {
        let store = Arc::new(CompactTextSessionStore);
        let dir = std::env::temp_dir().join("compact_test_freeze");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
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
            store.append_entry(&path, &entry).unwrap();
        }

        let engine = CompactionEngine::new(store);
        let result = engine.compact_segment(&path, &dir, 3).unwrap();

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
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("head.tlog");

        let entry = SessionEntry {
            id: uuid::Uuid::new_v4().to_string(),
            parent_id: None,
            timestamp: chrono::Utc::now(),
            role: "user".into(),
            content: "single entry".into(),
            metadata: crate::SessionMetadata::default(),
        };
        store.append_entry(&path, &entry).unwrap();

        let engine = CompactionEngine::new(store);
        let result = engine.compact_segment(&path, &dir, 10).unwrap();
        assert!(matches!(result, CompactionResult::Skipped));

        std::fs::remove_dir_all(&dir).ok();
    }
}
