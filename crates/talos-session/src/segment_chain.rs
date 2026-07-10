#![allow(dead_code)]
//! Segment chain metadata for session archival (ADR-037).
//!
//! When session compaction triggers (Slice D), the current segment is frozen and
//! archived. A new active segment is created. The chain of segments is tracked in
//! `chain.tlog`. This module provides the data structures and reader/writer for
//! that metadata file.
//!
//! In the current state (pre-Slice-D), there is always exactly one segment (the
//! head). The chain.tlog exists only as infrastructure for future archival.


use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Unique identifier for a segment within a session.
pub type SegmentId = String;

/// Status of a segment in the chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentStatus {
    Active,
    Archived,
    Compressed,
}

impl SegmentStatus {
    fn as_str(&self) -> &'static str {
        match self {
            SegmentStatus::Active => "active",
            SegmentStatus::Archived => "archived",
            SegmentStatus::Compressed => "compressed",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s {
            "active" => Some(SegmentStatus::Active),
            "archived" => Some(SegmentStatus::Archived),
            "compressed" => Some(SegmentStatus::Compressed),
            _ => None,
        }
    }
}

/// Metadata for a single segment in the chain.
#[derive(Debug, Clone)]
pub struct SegmentMeta {
    pub segment_id: SegmentId,
    pub status: SegmentStatus,
    pub prev_segment_id: Option<SegmentId>,
    pub record_count: usize,
    pub orig_bytes: u64,
    pub archived_bytes: Option<u64>,
    pub created_ts: i64,
    pub archived_ts: Option<i64>,
    pub archive_format: Option<String>,
    pub ref_count: u32,
}

/// The segment chain for a session directory.
#[derive(Debug, Clone, Default)]
pub struct ChainMetadata {
    pub segments: Vec<SegmentMeta>,
}

impl ChainMetadata {
    pub fn head_segment(&self) -> Option<&SegmentMeta> {
        self.segments.iter().find(|s| s.status == SegmentStatus::Active)
    }

    pub fn archived_segments(&self) -> impl Iterator<Item = &SegmentMeta> {
        self.segments.iter().filter(|s| s.status != SegmentStatus::Active)
    }

    pub fn total_ref_count(&self, segment_id: &str) -> u32 {
        self.segments
            .iter()
            .find(|s| s.segment_id == segment_id)
            .map(|s| s.ref_count)
            .unwrap_or(0)
    }

    /// Read chain metadata from a `chain.tlog` file.
    /// Returns empty metadata if the file does not exist (pre-archival state).
    pub fn read(path: &Path) -> Result<Self, std::io::Error> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let data = fs::read_to_string(path)?;
        let mut segments = Vec::new();

        for line in data.lines() {
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 11 || parts[0] != "S" {
                continue;
            }
            let segment_id = parts[1].to_string();
            let status = SegmentStatus::from_str(parts[2]).unwrap_or(SegmentStatus::Active);
            let prev_segment_id = if parts[3] == "-" {
                None
            } else {
                Some(parts[3].to_string())
            };
            let record_count: usize = parts[4].parse().unwrap_or(0);
            let orig_bytes: u64 = parts[5].parse().unwrap_or(0);
            let archived_bytes: Option<u64> = if parts[6] == "-" {
                None
            } else {
                parts[6].parse().ok()
            };
            let created_ts: i64 = parts[7].parse().unwrap_or(0);
            let archived_ts: Option<i64> = if parts[8] == "-" {
                None
            } else {
                parts[8].parse().ok()
            };
            let archive_format: Option<String> = if parts[9] == "-" {
                None
            } else {
                Some(parts[9].to_string())
            };
            let ref_count: u32 = parts[10].parse().unwrap_or(0);

            segments.push(SegmentMeta {
                segment_id,
                status,
                prev_segment_id,
                record_count,
                orig_bytes,
                archived_bytes,
                created_ts,
                archived_ts,
                archive_format,
                ref_count,
            });
        }

        Ok(Self { segments })
    }

    /// Write chain metadata to a `chain.tlog` file.
    pub fn write(&self, path: &Path) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::File::create(path)?;
        for seg in &self.segments {
            let prev = seg.prev_segment_id.as_deref().unwrap_or("-");
            let archived_bytes = seg
                .archived_bytes
                .map(|b| b.to_string())
                .unwrap_or_else(|| "-".into());
            let archived_ts = seg
                .archived_ts
                .map(|t| t.to_string())
                .unwrap_or_else(|| "-".into());
            let archive_format = seg
                .archive_format
                .as_deref()
                .unwrap_or("-");
            writeln!(
                file,
                "S\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                seg.segment_id,
                seg.status.as_str(),
                prev,
                seg.record_count,
                seg.orig_bytes,
                archived_bytes,
                seg.created_ts,
                archived_ts,
                archive_format,
                seg.ref_count
            )?;
        }
        Ok(())
    }

    /// Create a single-segment chain for a new session (no archival yet).
    pub fn single_active(segment_id: &str, created_ts: i64) -> Self {
        Self {
            segments: vec![SegmentMeta {
                segment_id: segment_id.to_string(),
                status: SegmentStatus::Active,
                prev_segment_id: None,
                record_count: 0,
                orig_bytes: 0,
                archived_bytes: None,
                created_ts,
                archived_ts: None,
                archive_format: None,
                ref_count: 0,
            }],
        }
    }
}

/// Return the path to `chain.tlog` for a given session directory.
pub fn chain_path(session_dir: &Path) -> PathBuf {
    session_dir.join("chain.tlog")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_round_trip() {
        let dir = std::env::temp_dir().join("chain_test_rt");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let chain = ChainMetadata {
            segments: vec![
                SegmentMeta {
                    segment_id: "head".into(),
                    status: SegmentStatus::Active,
                    prev_segment_id: Some("s001".into()),
                    record_count: 42,
                    orig_bytes: 48000,
                    archived_bytes: None,
                    created_ts: 1720520000,
                    archived_ts: None,
                    archive_format: None,
                    ref_count: 0,
                },
                SegmentMeta {
                    segment_id: "s001".into(),
                    status: SegmentStatus::Compressed,
                    prev_segment_id: None,
                    record_count: 380,
                    orig_bytes: 180000,
                    archived_bytes: Some(42000),
                    created_ts: 1720510000,
                    archived_ts: Some(1720515000),
                    archive_format: Some("zstd".into()),
                    ref_count: 2,
                },
            ],
        };

        let path = chain_path(&dir);
        chain.write(&path).unwrap();

        let read_back = ChainMetadata::read(&path).unwrap();
        assert_eq!(read_back.segments.len(), 2);
        assert_eq!(read_back.segments[0].segment_id, "head");
        assert_eq!(read_back.segments[0].status, SegmentStatus::Active);
        assert_eq!(read_back.segments[0].prev_segment_id, Some("s001".into()));
        assert_eq!(read_back.segments[1].status, SegmentStatus::Compressed);
        assert_eq!(read_back.segments[1].archived_bytes, Some(42000));
        assert_eq!(read_back.segments[1].archive_format, Some("zstd".into()));
        assert_eq!(read_back.segments[1].ref_count, 2);

        let head = read_back.head_segment().unwrap();
        assert_eq!(head.segment_id, "head");

        let archived: Vec<_> = read_back.archived_segments().collect();
        assert_eq!(archived.len(), 1);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn chain_read_missing_file_returns_empty() {
        let dir = std::env::temp_dir().join("chain_test_missing");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let chain = ChainMetadata::read(&chain_path(&dir)).unwrap();
        assert!(chain.segments.is_empty());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn chain_single_active() {
        let chain = ChainMetadata::single_active("seg-001", 1720520000);
        assert_eq!(chain.segments.len(), 1);
        assert_eq!(chain.segments[0].status, SegmentStatus::Active);
        assert!(chain.segments[0].prev_segment_id.is_none());
    }
}
