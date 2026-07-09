//! Session storage abstraction.
//!
//! The [`SessionStore`] trait separates session entry persistence from the [`Session`] type,
//! enabling future compact text format support alongside JSONL.

use crate::{SessionEntry, SessionError, SessionInfo, SessionMetadata};
use chrono::Utc;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::Path;
use talos_core::message::Message;
use uuid::Uuid;

pub use crate::compact_text::CompactTextSessionStore;

/// Trait abstracting session entry persistence.
///
/// Enables different storage formats (JSONL, compact text, segment chains)
/// while keeping the `Session` and `SessionManager` types format-agnostic.
pub trait SessionStore: Send + Sync + std::fmt::Debug {
    /// Read all entries from a session file.
    fn read_entries(&self, file_path: &Path) -> Result<Vec<SessionEntry>, SessionError>;

    /// Append a single entry to a session file.
    fn append_entry(&self, file_path: &Path, entry: &SessionEntry) -> Result<(), SessionError>;

    /// Read the ID of the last entry in the file.
    fn read_last_entry_id(&self, file_path: &Path) -> Option<String>;

    /// Scan a session file for message count and last preview text.
    fn scan_file(&self, file_path: &Path) -> Result<SessionInfo, SessionError>;

    /// Read the session file as raw bytes.
    fn read_bytes(&self, file_path: &Path) -> Result<Vec<u8>, SessionError>;

    /// The file extension for this store's format (e.g., `"jsonl"`).
    fn file_extension(&self) -> &'static str;
}

/// JSONL-based session store implementation.
///
/// This is the default store, preserving backward compatibility with
/// existing `.jsonl` session files.
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonlSessionStore;

impl SessionStore for JsonlSessionStore {
    fn read_entries(&self, file_path: &Path) -> Result<Vec<SessionEntry>, SessionError> {
        read_entries_from_path(file_path)
    }

    fn append_entry(&self, file_path: &Path, entry: &SessionEntry) -> Result<(), SessionError> {
        let line =
            serde_json::to_string(entry).map_err(|e| SessionError::InvalidJson(e.to_string()))?;

        if !file_path.exists()
            && let Some(parent) = file_path.parent()
        {
            fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)?;
        writeln!(file, "{line}")?;

        Ok(())
    }

    fn read_last_entry_id(&self, file_path: &Path) -> Option<String> {
        read_last_entry_id(file_path)
    }

    fn scan_file(&self, file_path: &Path) -> Result<SessionInfo, SessionError> {
        let file = fs::File::open(file_path)?;
        let metadata = file.metadata()?;
        let timestamp = metadata
            .modified()
            .ok()
            .map(chrono::DateTime::<Utc>::from)
            .unwrap_or_else(Utc::now);

        let id = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_else(Uuid::nil);

        let reader = BufReader::new(file);
        let mut count = 0;
        let mut last_preview = String::new();

        for line in reader.lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }

            if let Ok(entry) = serde_json::from_str::<SessionEntry>(&line) {
                count += 1;
                last_preview = crate::jsonl::preview_text(&entry.content);
                continue;
            }

            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line)
                && value.get("type").and_then(|t| t.as_str()) == Some("message")
                && let Some(data) = value.get("data")
                && let Ok(msg) = serde_json::from_value::<Message>(data.clone())
            {
                count += 1;
                let (_, content) = crate::jsonl::message_parts(&msg);
                last_preview = crate::jsonl::preview_text(&content);
            }
        }

        Ok(SessionInfo {
            id,
            project: String::new(),
            workspace_root: String::new(),
            last_message_preview: last_preview,
            timestamp,
            message_count: count,
        })
    }

    fn read_bytes(&self, file_path: &Path) -> Result<Vec<u8>, SessionError> {
        std::fs::read(file_path).map_err(SessionError::IoError)
    }

    fn file_extension(&self) -> &'static str {
        "jsonl"
    }
}

fn read_last_entry_id(path: &Path) -> Option<String> {
    let mut file = fs::File::open(path).ok()?;
    let file_size = file.metadata().ok()?.len();
    if file_size == 0 {
        return None;
    }
    let read_size = std::cmp::min(file_size, 8192) as usize;
    let seek_pos = file_size.saturating_sub(read_size as u64);
    file.seek(SeekFrom::Start(seek_pos)).ok()?;
    let mut buf = vec![0u8; read_size];
    file.read_exact(&mut buf).ok()?;
    let text = String::from_utf8_lossy(&buf);
    let last_line = text.lines().rev().find(|l| !l.is_empty())?;
    let entry: SessionEntry = serde_json::from_str(last_line).ok()?;
    Some(entry.id)
}

fn read_entries_from_path(path: &Path) -> Result<Vec<SessionEntry>, SessionError> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    let mut synthetic_counter: u64 = 0;

    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }

        if let Ok(entry) = serde_json::from_str::<SessionEntry>(&line) {
            entries.push(entry);
            continue;
        }

        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line)
            && value.get("type").and_then(|t| t.as_str()) == Some("message")
            && let Some(data) = value.get("data")
            && let Ok(msg) = serde_json::from_value::<Message>(data.clone())
        {
            let (role, content) = crate::jsonl::message_parts(&msg);
            let id = format!("synthetic-{synthetic_counter}");
            let parent_id = if synthetic_counter > 0 {
                Some(format!("synthetic-{}", synthetic_counter - 1))
            } else {
                None
            };

            entries.push(SessionEntry {
                id,
                parent_id,
                timestamp: Utc::now(),
                role,
                content,
                metadata: SessionMetadata::default(),
            });
            synthetic_counter += 1;
        }
        // Invalid lines are silently skipped (crash-safety guarantee)
    }

    Ok(entries)
}
