//! Bounded source reads for host-owned history recovery indexing.
//!
//! Pages are raw storage input, not model disclosure authority. Callers must bind
//! offsets to a validated source snapshot and apply branch/projection receipts.

use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::time::Instant;

use crate::SessionEntry;

/// Maximum bytes inspected by one recovery source read.
pub const HISTORY_PAGE_BYTES: usize = 2 * 1024 * 1024;
/// Maximum entries returned by one recovery source read.
pub const HISTORY_PAGE_ENTRIES: usize = 8192;

/// Authoritative transcript encoding selected by the host.
#[derive(Debug, Clone, Copy)]
pub enum HistoryEncoding {
    /// Length-prefixed compact text transcript.
    Tlog,
    /// Native SessionEntry JSON lines (legacy message envelopes are unavailable).
    Jsonl,
}

/// A bounded page of storage entries; never directly return it to a model.
#[derive(Debug)]
pub struct HistoryPage {
    /// Complete records decoded within this page.
    pub entries: Vec<SessionEntry>,
    /// Byte offset of the next record, suitable only for the same source snapshot.
    pub next_offset: u64,
    /// Whether the captured source end was reached.
    pub complete: bool,
    /// Bytes actually read, including a bounded incomplete trailing record.
    pub bytes_read: usize,
}

/// Failures of a bounded source scan, without transcript content in diagnostics.
#[derive(Debug, thiserror::Error)]
pub enum HistoryPageError {
    /// Reading the transcript failed.
    #[error("history source I/O failed")]
    Io(#[from] std::io::Error),
    /// Cancellation or the caller's deadline stopped the scan.
    #[error("history source scan stopped")]
    Stopped,
    /// The source no longer has the captured length or the offset is invalid.
    #[error("history source snapshot is stale")]
    Stale,
    /// The source is not a supported regular file.
    #[error("history source must be a regular file")]
    UnsupportedSource,
    /// A malformed or unsupported record cannot establish recovery provenance.
    #[error("history source record is invalid or unsupported")]
    InvalidRecord,
    /// A record cannot fit within the per-operation source bound.
    #[error("history source record exceeds the scan bound")]
    RecordTooLarge,
}

/// Read a bounded page without materializing the entire transcript.
///
/// `source_end` is the host-captured immutable cutoff. Appends after it are not
/// read. This function checks length, not source identity: the host must validate
/// the source digest/lineage before indexing and again before model delivery.
/// `stopped` is checked before each 8 KiB read and each decoded entry. Regular-file
/// kernel reads are synchronous; cancellation is cooperative between reads.
pub fn read_history_page(
    path: &Path,
    encoding: HistoryEncoding,
    offset: u64,
    source_end: u64,
    deadline: Instant,
    stopped: impl Fn() -> bool,
) -> Result<HistoryPage, HistoryPageError> {
    let check = || {
        if stopped() || Instant::now() >= deadline {
            Err(HistoryPageError::Stopped)
        } else {
            Ok(())
        }
    };
    check()?;
    let mut file = open_source(path)?;
    if offset > source_end || file.metadata()?.len() < source_end {
        return Err(HistoryPageError::Stale);
    }
    file.seek(SeekFrom::Start(offset))?;
    let amount = (source_end - offset).min(HISTORY_PAGE_BYTES as u64) as usize;
    let mut bytes = Vec::with_capacity(amount);
    let mut chunk = [0; 8192];
    while bytes.len() < amount {
        check()?;
        let wanted = chunk.len().min(amount - bytes.len());
        let n = file.read(&mut chunk[..wanted])?;
        if n == 0 {
            return Err(HistoryPageError::Stale);
        }
        bytes.extend_from_slice(&chunk[..n]);
    }
    let mut pos = 0;
    if offset == 0 && matches!(encoding, HistoryEncoding::Tlog) && !bytes.is_empty() {
        let end = bytes
            .iter()
            .position(|b| *b == b'\n')
            .ok_or(HistoryPageError::InvalidRecord)?;
        let header =
            std::str::from_utf8(&bytes[..end]).map_err(|_| HistoryPageError::InvalidRecord)?;
        let fields: Vec<_> = header.split('\t').collect();
        if fields.len() != 3
            || fields[0] != "TALOS"
            || fields[1] != "1"
            || fields[2].parse::<i64>().is_err()
        {
            return Err(HistoryPageError::InvalidRecord);
        }
        pos = end + 1;
    }
    let mut entries = Vec::new();
    while pos < bytes.len() && entries.len() < HISTORY_PAGE_ENTRIES {
        check()?;
        if matches!(encoding, HistoryEncoding::Jsonl) && matches!(bytes[pos], b'\n' | b'\r') {
            pos += 1;
            continue;
        }
        let decoded = match encoding {
            HistoryEncoding::Jsonl => {
                json_record(&bytes[pos..], offset + bytes.len() as u64 == source_end)?
            }
            HistoryEncoding::Tlog => tlog_record(&bytes[pos..])?,
        };
        let Some((entry, consumed)) = decoded else {
            if offset + bytes.len() as u64 == source_end {
                return Err(HistoryPageError::InvalidRecord);
            }
            if entries.is_empty() && pos == 0 {
                return Err(HistoryPageError::RecordTooLarge);
            }
            break;
        };
        entries.push(entry);
        pos += consumed;
    }
    check()?;
    let next_offset = offset + pos as u64;
    Ok(HistoryPage {
        entries,
        next_offset,
        complete: next_offset == source_end,
        bytes_read: bytes.len(),
    })
}

fn open_source(path: &Path) -> Result<std::fs::File, HistoryPageError> {
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NONBLOCK | libc::O_NOFOLLOW);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        use std::path::{Component, Prefix};
        use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT;
        // Recovery is local storage. Refuse device, pipe and network namespaces
        // before opening; require an absolute local drive path from the host.
        if !matches!(path.components().next(), Some(Component::Prefix(p))
            if matches!(p.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_)))
            || !path.is_absolute()
        {
            return Err(HistoryPageError::UnsupportedSource);
        }
        options.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
    }
    #[cfg(not(any(unix, windows)))]
    return Err(HistoryPageError::UnsupportedSource);
    let file = options.open(path)?;
    let metadata = file.metadata()?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(HistoryPageError::UnsupportedSource);
        }
    }
    if !metadata.is_file() {
        return Err(HistoryPageError::UnsupportedSource);
    }
    Ok(file)
}

fn json_record(
    bytes: &[u8],
    at_eof: bool,
) -> Result<Option<(SessionEntry, usize)>, HistoryPageError> {
    let newline = bytes.iter().position(|b| *b == b'\n');
    let end = match newline {
        Some(end) => end,
        None if at_eof => bytes.len(),
        None => return Ok(None),
    };
    let entry =
        serde_json::from_slice(&bytes[..end]).map_err(|_| HistoryPageError::InvalidRecord)?;
    Ok(Some((entry, end + usize::from(newline.is_some()))))
}

fn tlog_record(bytes: &[u8]) -> Result<Option<(SessionEntry, usize)>, HistoryPageError> {
    let mut fields = Vec::with_capacity(5);
    let mut pos = 0;
    for _ in 0..5 {
        let Some(end) = bytes[pos..].iter().position(|b| *b == b'\t') else {
            return Ok(None);
        };
        fields.push(
            std::str::from_utf8(&bytes[pos..pos + end])
                .map_err(|_| HistoryPageError::InvalidRecord)?,
        );
        pos += end + 1;
    }
    if fields[0] != "E" {
        return Err(HistoryPageError::InvalidRecord);
    }
    let role = match fields[1] {
        "0" => "user",
        "1" => "assistant",
        "2" => "system",
        "3" => "unknown-3",
        _ => return Err(HistoryPageError::InvalidRecord),
    };
    let timestamp = fields[2]
        .parse::<i64>()
        .ok()
        .and_then(chrono::DateTime::from_timestamp_millis)
        .ok_or(HistoryPageError::InvalidRecord)?;
    let Some(content) = prefixed(bytes, &mut pos)? else {
        return Ok(None);
    };
    if bytes.get(pos) != Some(&b'\t') {
        return if pos == bytes.len() {
            Ok(None)
        } else {
            Err(HistoryPageError::InvalidRecord)
        };
    }
    pos += 1;
    let Some(metadata) = prefixed(bytes, &mut pos)? else {
        return Ok(None);
    };
    if bytes.get(pos) != Some(&b'\n') {
        return if pos == bytes.len() {
            Ok(None)
        } else {
            Err(HistoryPageError::InvalidRecord)
        };
    }
    Ok(Some((
        SessionEntry {
            id: fields[3].to_owned(),
            parent_id: (fields[4] != "-").then(|| fields[4].to_owned()),
            timestamp,
            role: role.into(),
            content: std::str::from_utf8(content)
                .map_err(|_| HistoryPageError::InvalidRecord)?
                .into(),
            metadata: if metadata.is_empty() {
                crate::SessionMetadata::default()
            } else {
                serde_json::from_slice(metadata).map_err(|_| HistoryPageError::InvalidRecord)?
            },
        },
        pos + 1,
    )))
}

fn prefixed<'a>(bytes: &'a [u8], pos: &mut usize) -> Result<Option<&'a [u8]>, HistoryPageError> {
    let Some(colon) = bytes[*pos..].iter().position(|b| *b == b':') else {
        return Ok(None);
    };
    let len = std::str::from_utf8(&bytes[*pos..*pos + colon])
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .ok_or(HistoryPageError::InvalidRecord)?;
    if len > HISTORY_PAGE_BYTES {
        return Err(HistoryPageError::RecordTooLarge);
    }
    let start = *pos + colon + 1;
    let end = start
        .checked_add(len)
        .ok_or(HistoryPageError::InvalidRecord)?;
    let Some(content) = bytes.get(start..end) else {
        return Ok(None);
    };
    *pos = end;
    Ok(Some(content))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CompactTextSessionStore, JsonlSessionStore, SessionMetadata, SessionStore};
    use std::cell::Cell;
    use std::time::Duration;

    fn entry(i: usize, content: String) -> SessionEntry {
        SessionEntry {
            id: i.to_string(),
            parent_id: i.checked_sub(1).map(|n| n.to_string()),
            timestamp: chrono::DateTime::from_timestamp_millis(1000).expect("timestamp"),
            role: "user".into(),
            content,
            metadata: SessionMetadata::default(),
        }
    }

    #[test]
    fn both_formats_resume_across_byte_and_entry_bounds() {
        let dir = tempfile::tempdir().expect("tempdir");
        for (encoding, store) in [
            (
                HistoryEncoding::Jsonl,
                &JsonlSessionStore as &dyn SessionStore,
            ),
            (
                HistoryEncoding::Tlog,
                &CompactTextSessionStore as &dyn SessionStore,
            ),
        ] {
            let path = dir.path().join(store.file_extension());
            let originals = (0..8300)
                .map(|i| entry(i, format!("tab\tnewline\n你好{}", "x".repeat(300))))
                .collect::<Vec<_>>();
            store
                .replace_entries_atomically(&path, &originals)
                .expect("write");
            let end = std::fs::metadata(&path).expect("metadata").len();
            // An append after the captured cutoff must not enter this snapshot.
            store
                .append_entry(&path, &entry(8300, "later".into()))
                .expect("append");
            let mut offset = 0;
            let mut actual = Vec::new();
            let mut pages = 0;
            loop {
                let page = read_history_page(
                    &path,
                    encoding,
                    offset,
                    end,
                    Instant::now() + Duration::from_secs(5),
                    || false,
                )
                .expect("page");
                assert!(page.bytes_read <= HISTORY_PAGE_BYTES);
                assert!(page.entries.len() <= HISTORY_PAGE_ENTRIES);
                assert!(page.next_offset > offset);
                offset = page.next_offset;
                actual.extend(page.entries);
                pages += 1;
                if page.complete {
                    break;
                }
            }
            assert!(pages > 1);
            assert_eq!(actual.len(), originals.len());
            for (actual, expected) in actual.iter().zip(&originals) {
                assert_eq!(
                    serde_json::to_value(actual).expect("serialize"),
                    serde_json::to_value(expected).expect("serialize")
                );
            }
        }
    }

    #[test]
    fn header_does_not_reduce_the_first_records_byte_allowance() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("history.tlog");
        // Record fits exactly in one operation; header requires its own page.
        let overhead = "E\t0\t1000\tid\t-\t2097152:\t2:{}\n".len();
        let content = "x".repeat(HISTORY_PAGE_BYTES - overhead);
        let record = format!("E\t0\t1000\tid\t-\t{}:{}\t2:{{}}\n", content.len(), content);
        assert_eq!(record.len(), HISTORY_PAGE_BYTES);
        std::fs::write(&path, format!("TALOS\t1\t1000\n{record}")).expect("write");
        let end = std::fs::metadata(&path).expect("metadata").len();
        let first = read_history_page(
            &path,
            HistoryEncoding::Tlog,
            0,
            end,
            Instant::now() + Duration::from_secs(5),
            || false,
        )
        .expect("header page");
        assert!(first.entries.is_empty());
        assert!(first.next_offset > 0);
        assert!(!first.complete);
        let second = read_history_page(
            &path,
            HistoryEncoding::Tlog,
            first.next_offset,
            end,
            Instant::now() + Duration::from_secs(5),
            || false,
        )
        .expect("record page");
        assert_eq!(second.entries[0].content, content);
        assert!(second.complete);
    }

    #[test]
    fn entry_count_bound_advances_without_dropping_records() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("history.tlog");
        let entries = (0..HISTORY_PAGE_ENTRIES + 1)
            .map(|i| entry(i, String::new()))
            .collect::<Vec<_>>();
        CompactTextSessionStore
            .replace_entries_atomically(&path, &entries)
            .expect("write");
        let end = std::fs::metadata(&path).expect("metadata").len();
        assert!(end < HISTORY_PAGE_BYTES as u64);
        let first = read_history_page(
            &path,
            HistoryEncoding::Tlog,
            0,
            end,
            Instant::now() + Duration::from_secs(5),
            || false,
        )
        .expect("first");
        assert_eq!(first.entries.len(), HISTORY_PAGE_ENTRIES);
        assert!(!first.complete);
        let second = read_history_page(
            &path,
            HistoryEncoding::Tlog,
            first.next_offset,
            end,
            Instant::now() + Duration::from_secs(5),
            || false,
        )
        .expect("second");
        assert!(second.complete);
        assert_eq!(second.entries.len(), 1);
        assert_eq!(second.entries[0].id, HISTORY_PAGE_ENTRIES.to_string());
    }

    #[test]
    fn jsonl_blank_crlf_and_complete_eof_record_are_compatible() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("history.jsonl");
        let record = serde_json::to_string(&entry(0, "kept".into())).expect("encode");
        std::fs::write(&path, format!("\r\n\n{record}\r\n\n{record}")).expect("write");
        let end = std::fs::metadata(&path).expect("metadata").len();
        let page = read_history_page(
            &path,
            HistoryEncoding::Jsonl,
            0,
            end,
            Instant::now() + Duration::from_secs(5),
            || false,
        )
        .expect("page");
        assert!(page.complete);
        assert_eq!(page.entries.len(), 2);
        std::fs::write(&path, "TALOS\t1\t1000\nE\t0\t1000\tid\t-\t2:ok\t0:\n").expect("write");
        let end = std::fs::metadata(&path).expect("metadata").len();
        let page = read_history_page(
            &path,
            HistoryEncoding::Tlog,
            0,
            end,
            Instant::now() + Duration::from_secs(5),
            || false,
        )
        .expect("empty metadata");
        assert_eq!(page.entries[0].content, "ok");
    }

    #[cfg(unix)]
    #[test]
    fn special_files_and_leaf_symlinks_are_rejected_without_waiting() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("file");
        std::fs::write(&file, "data").expect("write");
        let link = dir.path().join("link");
        std::os::unix::fs::symlink(&file, &link).expect("link");
        assert!(open_source(&link).is_err());
        assert!(open_source(dir.path()).is_err());
        let fifo = dir.path().join("fifo");
        assert!(
            std::process::Command::new("mkfifo")
                .arg(&fifo)
                .status()
                .expect("mkfifo fixture")
                .success()
        );
        assert!(matches!(
            open_source(&fifo),
            Err(HistoryPageError::UnsupportedSource)
        ));
    }

    #[test]
    fn cancellation_stops_reads_before_decoding() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("history.jsonl");
        std::fs::write(&path, vec![b'x'; HISTORY_PAGE_BYTES * 2]).expect("write");
        let checks = Cell::new(0);
        let result = read_history_page(
            &path,
            HistoryEncoding::Jsonl,
            0,
            (HISTORY_PAGE_BYTES * 2) as u64,
            Instant::now() + Duration::from_secs(5),
            || {
                checks.set(checks.get() + 1);
                checks.get() >= 4
            },
        );
        assert!(matches!(result, Err(HistoryPageError::Stopped)));
        assert_eq!(checks.get(), 4);
    }

    #[test]
    fn corruption_never_returns_partial_authority_or_panics() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("history.tlog");
        for record in [
            "E\t0\t1000\tid\t-\t1:你\t2:{}\n",
            "E\t0\t1000\tid\t-\t18446744073709551615:x\t2:{}\n",
            "E\t0\t1000\tid\t-\t2:ok\t2:{}X",
            "E\t0\t1000\tid\t-\t2:ok\t99:{}\n",
        ] {
            std::fs::write(&path, format!("TALOS\t1\t1000\n{record}")).expect("write");
            let end = std::fs::metadata(&path).expect("metadata").len();
            assert!(
                read_history_page(
                    &path,
                    HistoryEncoding::Tlog,
                    0,
                    end,
                    Instant::now() + Duration::from_secs(5),
                    || false
                )
                .is_err()
            );
        }
    }

    #[test]
    fn truncated_source_and_expired_deadline_fail_closed() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("history.jsonl");
        std::fs::write(&path, b"{}").expect("write");
        assert!(matches!(
            read_history_page(
                &path,
                HistoryEncoding::Jsonl,
                0,
                3,
                Instant::now() + Duration::from_secs(5),
                || false
            ),
            Err(HistoryPageError::Stale)
        ));
        assert!(matches!(
            read_history_page(&path, HistoryEncoding::Jsonl, 0, 2, Instant::now(), || {
                false
            }),
            Err(HistoryPageError::Stopped)
        ));
    }
}
