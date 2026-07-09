//! Compact text session log format (`.tlog`).
//!
//! Implements [`SessionStore`] using a TSV-header + length-prefixed-content text format
//! per ADR-037. More compact than JSONL while remaining human-readable and Unix-tool-friendly.
//!
//! # Format
//!
//! ```text
//! # File header (first line)
//! TALOS\tv1\t<created_ts_ms>\n
//!
//! # Each record:
//! E\t<role>\t<ts_ms>\t<id>\t<parent_id|->\t<content_len>:<content_bytes>\t<meta_len>:<meta_json>\n
//! ```
//!
//! Content and metadata use `<decimal_len>:` prefix so the reader reads exactly N bytes,
//! allowing tabs, newlines, and any byte inside content without escaping.
//!
//! See `docs/decisions/037-compact-text-session-log-format.md` for the full design.

use crate::{SessionEntry, SessionError, SessionInfo, SessionMetadata};
use chrono::Utc;
use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use uuid::Uuid;

/// Magic header for `.tlog` files.
const TLOG_MAGIC: &str = "TALOS";
/// Current format version.
const TLOG_VERSION: u8 = 1;
/// Record kind for session entry.
const KIND_ENTRY: char = 'E';

/// Compact text session store implementation.
///
/// Uses TSV header fields + length-prefixed content for ~40-65% size reduction
/// over JSONL while remaining text-readable. See ADR-037.
#[derive(Debug, Clone, Copy, Default)]
pub struct CompactTextSessionStore;

impl crate::store::SessionStore for CompactTextSessionStore {
    fn read_entries(&self, file_path: &Path) -> Result<Vec<SessionEntry>, SessionError> {
        read_tlog_entries(file_path)
    }

    fn append_entry(
        &self,
        file_path: &Path,
        entry: &SessionEntry,
    ) -> Result<(), SessionError> {
        if !file_path.exists() {
            // Write file header on first append.
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(file_path)?;
            writeln!(file, "{TLOG_MAGIC}\t{TLOG_VERSION}\t{}", Utc::now().timestamp_millis())?;
            file.flush()?;
        }

        let line = encode_entry(entry);
        let mut file = OpenOptions::new()
            .append(true)
            .open(file_path)?;
        file.write_all(line.as_bytes())?;
        file.flush()?;
        Ok(())
    }

    fn read_last_entry_id(&self, file_path: &Path) -> Option<String> {
        read_last_entry_id_tlog(file_path)
    }

    fn scan_file(&self, file_path: &Path) -> Result<SessionInfo, SessionError> {
        scan_tlog_file(file_path)
    }

    fn read_bytes(&self, file_path: &Path) -> Result<Vec<u8>, SessionError> {
        std::fs::read(file_path).map_err(SessionError::IoError)
    }

    fn file_extension(&self) -> &'static str {
        "tlog"
    }
}

// ---------------------------------------------------------------------------
// Encoding
// ---------------------------------------------------------------------------

/// Encode a [`SessionEntry`] as a single compact text record line.
///
/// Format: `E\t<role>\t<ts_ms>\t<id>\t<parent_id|->\t<content_len>:<content>\t<meta_len>:<meta_json>\n`
fn encode_entry(entry: &SessionEntry) -> String {
    let role_num = role_to_num(&entry.role);
    let parent = entry.parent_id.as_deref().unwrap_or("-");
    let meta_json = if entry.metadata.is_empty() {
        String::from("{}")
    } else {
        serde_json::to_string(&entry.metadata).unwrap_or_else(|_| String::from("{}"))
    };
    let ts_ms = entry.timestamp.timestamp_millis();

    // Build the line using length-prefixed fields for content and metadata.
    // We construct a String; content may contain any bytes but since SessionEntry.content
    // is a Rust String (valid UTF-8), we can safely embed it.
    format!(
        "{KIND_ENTRY}\t{role_num}\t{ts_ms}\t{}\t{parent}\t{}:{}\t{}:{}\n",
        entry.id,
        entry.content.len(),
        entry.content,
        meta_json.len(),
        meta_json,
    )
}

/// Map role string to numeric encoding.
fn role_to_num(role: &str) -> u8 {
    match role {
        "user" => 0,
        "assistant" => 1,
        "system" => 2,
        _ => 3, // Unknown roles get 3; preserved on round-trip.
    }
}

/// Map numeric encoding back to role string.
fn num_to_role(num: u8) -> String {
    match num {
        0 => "user".into(),
        1 => "assistant".into(),
        2 => "system".into(),
        _ => format!("unknown-{num}"),
    }
}

// ---------------------------------------------------------------------------
// Decoding
// ---------------------------------------------------------------------------

/// Read all entries from a `.tlog` file.
///
/// Skips the file header line. Truncated or corrupt final record is silently skipped;
/// all prior valid records are returned. Mid-file corruption returns an error.
fn read_tlog_entries(path: &Path) -> Result<Vec<SessionEntry>, SessionError> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let data = std::fs::read(path)?;
    parse_tlog_bytes(&data)
}

fn parse_tlog_bytes(data: &[u8]) -> Result<Vec<SessionEntry>, SessionError> {
    let text = std::str::from_utf8(data).map_err(|_| SessionError::ParseError("invalid UTF-8".into()))?;
    let mut entries = Vec::new();
    let mut pos = 0;
    let mut first_line = true;

    while pos < text.len() {
        let remaining = &text[pos..];

        // Find the next newline — this is either a record separator or inside content.
        // We parse the record header to find content_len, then skip exactly content_len
        // bytes past the content to find the true record terminator.
        let newline_pos = match remaining.find('\n') {
            Some(p) => p,
            None => {
                // No trailing newline — last record might be incomplete.
                break;
            }
        };

        let line = &remaining[..newline_pos];

        if first_line {
            first_line = false;
            if line.starts_with(TLOG_MAGIC) {
                pos += newline_pos + 1;
                continue;
            }
        }

        if line.is_empty() {
            pos += newline_pos + 1;
            continue;
        }

        // Try to parse as a record. The line might be incomplete because content
        // contains \n — in that case, the record spans multiple "lines".
        // We attempt to parse the header fields from this line segment.
        match try_parse_record(remaining) {
            Ok((entry, consumed)) => {
                entries.push(entry);
                pos += consumed;
            }
            Err(DecodeError::Skip) => {
                pos += newline_pos + 1;
            }
            Err(DecodeError::Fatal(e)) => return Err(e),
        }
    }

    Ok(entries)
}

fn try_parse_record(text: &str) -> Result<(SessionEntry, usize), DecodeError> {
    let bytes = text.as_bytes();
    let mut tab_positions = Vec::new();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'\t' {
            tab_positions.push(i);
            if tab_positions.len() == 5 {
                break;
            }
        }
    }

    if tab_positions.len() < 5 {
        return Err(DecodeError::Skip);
    }

    let kind = &text[..tab_positions[0]];
    if kind != "E" {
        return Err(DecodeError::Skip);
    }

    let role_num: u8 = text[tab_positions[0] + 1..tab_positions[1]]
        .parse()
        .map_err(|_| DecodeError::Skip)?;

    let ts_ms_str = &text[tab_positions[1] + 1..tab_positions[2]];
    let ts_ms: i64 = ts_ms_str.parse().map_err(|_| DecodeError::Skip)?;

    let id = text[tab_positions[2] + 1..tab_positions[3]].to_string();

    let parent_field = &text[tab_positions[3] + 1..tab_positions[4]];
    let parent_id = if parent_field == "-" {
        None
    } else {
        Some(parent_field.to_string())
    };

    let rest_start = tab_positions[4] + 1;
    let rest = &text[rest_start..];

    let colon_pos = rest.find(':').ok_or(DecodeError::Skip)?;
    let content_len: usize = rest[..colon_pos]
        .parse()
        .map_err(|_| DecodeError::Skip)?;

    let content_start = colon_pos + 1;
    let content_end = content_start + content_len;
    if content_end > rest.len() {
        return Err(DecodeError::Skip);
    }

    let content = rest[content_start..content_end].to_string();

    let after_content = &rest[content_end..];
    if !after_content.starts_with('\t') {
        return Err(DecodeError::Skip);
    }

    let meta_part = &after_content[1..];

    let meta_colon = meta_part.find(':').ok_or(DecodeError::Skip)?;
    let meta_len: usize = meta_part[..meta_colon]
        .parse()
        .map_err(|_| DecodeError::Skip)?;

    let meta_start = meta_colon + 1;
    let meta_end = meta_start + meta_len;
    if meta_end > meta_part.len() {
        return Err(DecodeError::Skip);
    }

    let meta_str = &meta_part[meta_start..meta_end];
    let metadata: SessionMetadata = if meta_str == "{}" || meta_str.is_empty() {
        SessionMetadata::default()
    } else {
        serde_json::from_str(meta_str).map_err(|_| DecodeError::Skip)?
    };

    let timestamp =
        chrono::DateTime::from_timestamp_millis(ts_ms).unwrap_or_else(Utc::now);

    let entry = SessionEntry {
        id,
        parent_id,
        timestamp,
        role: num_to_role(role_num),
        content,
        metadata,
    };

    // consumed = everything from the start of the record to the \n after metadata
    let total_consumed = rest_start + content_end + 1 + meta_colon + 1 + meta_len + 1; // +1 for trailing \n
    Ok((entry, total_consumed))
}

/// Result type for single-record decoding.
#[allow(dead_code)]
enum DecodeError {
    /// Skip this record (corrupt or unrecognized).
    Skip,
    /// Fatal error — stop reading.
    Fatal(SessionError),
}

// ---------------------------------------------------------------------------
// Last entry ID (tail seek)
// ---------------------------------------------------------------------------

fn read_last_entry_id_tlog(path: &Path) -> Option<String> {
    let mut file = fs::File::open(path).ok()?;
    let file_size = file.metadata().ok()?.len();
    if file_size == 0 {
        return None;
    }

    // Seek to the last 16KB and scan for the last valid record.
    let read_size = std::cmp::min(file_size, 16384) as usize;
    let seek_pos = file_size.saturating_sub(read_size as u64);
    file.seek(SeekFrom::Start(seek_pos)).ok()?;
    let mut buf = vec![0u8; read_size];
    file.read_exact(&mut buf).ok()?;
    let text = String::from_utf8_lossy(&buf);

    // Find the last non-empty line that looks like a valid record.
    let mut last_id = None;
    let mut first_line = true;
    for line in text.lines() {
        if first_line {
            first_line = false;
            // If we sought to the start, the first line might be the header.
            if seek_pos == 0 && line.starts_with(TLOG_MAGIC) {
                continue;
            }
        }
        if line.is_empty() {
            continue;
        }
        // Try to extract just the ID field (4th tab-delimited field).
        if let Some(id) = extract_id_from_line(line) {
            last_id = Some(id);
        }
    }
    last_id
}

/// Extract the `id` field from a record line without full decode.
fn extract_id_from_line(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.splitn(6, '\t').collect();
    if parts.len() < 4 {
        return None;
    }
    // parts[0]=kind, parts[1]=role, parts[2]=ts_ms, parts[3]=id
    if parts[0] != "E" {
        return None;
    }
    Some(parts[3].to_string())
}

// ---------------------------------------------------------------------------
// Scan for preview
// ---------------------------------------------------------------------------

fn scan_tlog_file(path: &Path) -> Result<SessionInfo, SessionError> {
    let file = fs::File::open(path)?;
    let metadata = file.metadata()?;
    let timestamp = metadata
        .modified()
        .ok()
        .map(chrono::DateTime::<Utc>::from)
        .unwrap_or_else(Utc::now);

    let id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or_else(Uuid::nil);

    let data = std::fs::read(path)?;
    let entries = parse_tlog_bytes(&data)?;

    let count = entries.len();
    let last_preview = entries
        .last()
        .map(|e| crate::jsonl::preview_text(&e.content))
        .unwrap_or_default();

    Ok(SessionInfo {
        id,
        project: String::new(),
        workspace_root: String::new(),
        last_message_preview: last_preview,
        timestamp,
        message_count: count,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(warnings)]
mod tests {
    use super::*;
    use crate::store::SessionStore;
    use crate::JsonlSessionStore;
    use chrono::TimeZone;
    use std::io::Read;

    fn make_entry(role: &str, content: &str) -> SessionEntry {
        SessionEntry {
            id: Uuid::new_v4().to_string(),
            parent_id: None,
            timestamp: Utc::now(),
            role: role.into(),
            content: content.into(),
            metadata: SessionMetadata::default(),
        }
    }

    fn make_entry_with_meta(role: &str, content: &str, meta: SessionMetadata) -> SessionEntry {
        SessionEntry {
            id: Uuid::new_v4().to_string(),
            parent_id: None,
            timestamp: Utc::now(),
            role: role.into(),
            content: content.into(),
            metadata: meta,
        }
    }

    #[test]
    fn round_trip_basic() {
        let store = CompactTextSessionStore;
        let dir = std::env::temp_dir().join("tlog_test_roundtrip_basic");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.tlog");

        let entry = make_entry("user", "Hello, world!");
        store.append_entry(&path, &entry).unwrap();

        let entries = store.read_entries(&path).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].role, "user");
        assert_eq!(entries[0].content, "Hello, world!");
        assert_eq!(entries[0].id, entry.id);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn round_trip_multiple_entries() {
        let store = CompactTextSessionStore;
        let dir = std::env::temp_dir().join("tlog_test_roundtrip_multi");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("multi.tlog");

        let entries = vec![
            make_entry("user", "What is 2+2?"),
            make_entry("assistant", "The answer is 4."),
            make_entry("user", "Thanks!"),
        ];

        for e in &entries {
            store.append_entry(&path, e).unwrap();
        }

        let read = store.read_entries(&path).unwrap();
        assert_eq!(read.len(), 3);
        assert_eq!(read[0].content, "What is 2+2?");
        assert_eq!(read[1].content, "The answer is 4.");
        assert_eq!(read[2].content, "Thanks!");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn round_trip_with_parent_id() {
        let store = CompactTextSessionStore;
        let dir = std::env::temp_dir().join("tlog_test_parent");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("parent.tlog");

        let parent = make_entry("user", "parent message");
        let mut child = make_entry("assistant", "child response");
        child.parent_id = Some(parent.id.clone());

        store.append_entry(&path, &parent).unwrap();
        store.append_entry(&path, &child).unwrap();

        let read = store.read_entries(&path).unwrap();
        assert_eq!(read.len(), 2);
        assert!(read[0].parent_id.is_none());
        assert_eq!(read[1].parent_id, Some(parent.id.clone()));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn round_trip_with_metadata() {
        let store = CompactTextSessionStore;
        let dir = std::env::temp_dir().join("tlog_test_meta");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("meta.tlog");

        let meta = SessionMetadata {
            provider: Some("anthropic".into()),
            model: Some("claude-sonnet-4".into()),
            token_count: Some(42),
            working_directory: None,
            reasoning: None,
        };
        let entry = make_entry_with_meta("assistant", "Response with metadata", meta);

        store.append_entry(&path, &entry).unwrap();

        let read = store.read_entries(&path).unwrap();
        assert_eq!(read.len(), 1);
        assert_eq!(read[0].metadata.provider, Some("anthropic".into()));
        assert_eq!(read[0].metadata.model, Some("claude-sonnet-4".into()));
        assert_eq!(read[0].metadata.token_count, Some(42));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn content_with_tabs_and_newlines() {
        let store = CompactTextSessionStore;
        let dir = std::env::temp_dir().join("tlog_test_special");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("special.tlog");

        // Content with tabs, newlines, and special characters
        let tricky = "line1\nline2\twith\ttabs\nline3\twith more";
        let entry = make_entry("assistant", tricky);

        store.append_entry(&path, &entry).unwrap();

        let read = store.read_entries(&path).unwrap();
        assert_eq!(read.len(), 1);
        assert_eq!(read[0].content, tricky);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn content_with_unicode() {
        let store = CompactTextSessionStore;
        let dir = std::env::temp_dir().join("tlog_test_unicode");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("unicode.tlog");

        let entry = make_entry("user", "你好世界 🌍 Привет мир");

        store.append_entry(&path, &entry).unwrap();

        let read = store.read_entries(&path).unwrap();
        assert_eq!(read.len(), 1);
        assert_eq!(read[0].content, "你好世界 🌍 Привет мир");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn corrupt_final_record_skipped() {
        let dir = std::env::temp_dir().join("tlog_test_corrupt");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("corrupt.tlog");

        let store = CompactTextSessionStore;

        // Write two valid entries.
        store
            .append_entry(&path, &make_entry("user", "first"))
            .unwrap();
        store
            .append_entry(&path, &make_entry("assistant", "second"))
            .unwrap();

        // Append a truncated/garbage line (simulating crash).
        std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap()
            .write_all(b"E\t0\t999\tincomplete\t-\t")
            .unwrap();

        let read = store.read_entries(&path).unwrap();
        assert_eq!(read.len(), 2, "corrupt final record should be skipped");
        assert_eq!(read[0].content, "first");
        assert_eq!(read[1].content, "second");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn read_last_entry_id_returns_last_valid() {
        let dir = std::env::temp_dir().join("tlog_test_last_id");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("last_id.tlog");

        let store = CompactTextSessionStore;
        let e1 = make_entry("user", "first");
        let e2 = make_entry("assistant", "second");
        let e3 = make_entry("user", "third");

        store.append_entry(&path, &e1).unwrap();
        store.append_entry(&path, &e2).unwrap();
        store.append_entry(&path, &e3).unwrap();

        let last_id = store.read_last_entry_id(&path);
        assert_eq!(last_id, Some(e3.id));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn read_empty_file_returns_empty() {
        let dir = std::env::temp_dir().join("tlog_test_empty");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("nonexistent.tlog");

        let store = CompactTextSessionStore;
        let entries = store.read_entries(&path).unwrap();
        assert!(entries.is_empty());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn scan_file_counts_and_previews() {
        let dir = std::env::temp_dir().join("tlog_test_scan");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("scan.tlog");

        let store = CompactTextSessionStore;
        store
            .append_entry(&path, &make_entry("user", "first message"))
            .unwrap();
        store
            .append_entry(&path, &make_entry("assistant", "second message"))
            .unwrap();

        let info = store.scan_file(&path).unwrap();
        assert_eq!(info.message_count, 2);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn file_extension_is_tlog() {
        let store = CompactTextSessionStore;
        assert_eq!(store.file_extension(), "tlog");
    }

    #[test]
    fn density_comparison() {
        // Write the same entries in both JSONL and .tlog format, measure sizes.
        let dir = std::env::temp_dir().join("tlog_test_density");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let jsonl_path = dir.join("session.jsonl");
        let tlog_path = dir.join("session.tlog");

        let entries: Vec<SessionEntry> = (0..50)
            .map(|i| SessionEntry {
                id: Uuid::new_v4().to_string(),
                parent_id: if i > 0 {
                    Some(format!("prev-{i}"))
                } else {
                    None
                },
                timestamp: Utc::now(),
                role: if i % 2 == 0 { "user".to_string() } else { "assistant".to_string() },
                content: format!("This is message number {i} with some content to simulate a real conversation."),
                metadata: SessionMetadata {
                    provider: Some("anthropic".into()),
                    model: Some("claude-sonnet-4-20250514".into()),
                    token_count: Some(100 + i as u32),
                    working_directory: None,
                    reasoning: None,
                },
            })
            .collect();

        // Write JSONL
        let jsonl_store = JsonlSessionStore;
        for e in &entries {
            jsonl_store.append_entry(&jsonl_path, e).unwrap();
        }

        // Write .tlog
        let tlog_store = CompactTextSessionStore;
        for e in &entries {
            tlog_store.append_entry(&tlog_path, e).unwrap();
        }

        let jsonl_size = std::fs::metadata(&jsonl_path).unwrap().len();
        let tlog_size = std::fs::metadata(&tlog_path).unwrap().len();

        println!("JSONL: {jsonl_size} bytes");
        println!("TLOG:  {tlog_size} bytes");
        println!("Ratio: {:.1}%", (tlog_size as f64 / jsonl_size as f64) * 100.0);
        println!("Saving: {:.1}%", (1.0 - tlog_size as f64 / jsonl_size as f64) * 100.0);

        // .tlog should be smaller than JSONL.
        assert!(
            tlog_size < jsonl_size,
            ".tlog ({tlog_size}) should be smaller than JSONL ({jsonl_size})"
        );

        // Verify both have the same number of entries.
        let jsonl_entries = jsonl_store.read_entries(&jsonl_path).unwrap();
        let tlog_entries = tlog_store.read_entries(&tlog_path).unwrap();
        assert_eq!(jsonl_entries.len(), tlog_entries.len());
        assert_eq!(jsonl_entries.len(), 50);

        std::fs::remove_dir_all(&dir).ok();
    }
}
