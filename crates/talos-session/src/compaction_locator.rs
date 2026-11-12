//! Host-custodied bridges for zero-retention compaction. Not fork authority.
use crate::{Session, SessionEntry, SessionError};
use rusqlite::params;
use sha2::{Digest, Sha256};
use std::{
    io::Read,
    path::Path,
    time::{Duration, Instant},
};

pub(crate) struct Locator {
    pub marker: String,
    pub marker_digest: String,
    pub archive: String,
    pub archive_digest: String,
    pub tail: String,
}

fn invalid() -> SessionError {
    SessionError::ParseError("zero-retention compaction provenance unavailable".into())
}
fn identity(path: &Path) -> Result<String, SessionError> {
    let id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(invalid)?;
    uuid::Uuid::parse_str(id).map_err(|_| invalid())?;
    if path.extension().and_then(|s| s.to_str()) != Some("tlog") {
        return Err(invalid());
    }
    Ok(id.into())
}
fn boundary<T>(f: impl FnOnce() -> Result<T, SessionError>) -> Result<T, SessionError> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or_else(|_| Err(invalid()))
}
pub(crate) fn digest(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

pub(crate) fn load(
    head: &Path,
    deadline: Instant,
    stopped: &impl Fn() -> bool,
) -> Result<Vec<Locator>, SessionError> {
    boundary(|| {
        let check = || {
            if stopped() || Instant::now() >= deadline {
                Err(invalid())
            } else {
                Ok(())
            }
        };
        check()?;
        let id = identity(head)?;
        let db = head
            .parent()
            .ok_or_else(invalid)?
            .join("history-bindings.sqlite");
        match std::fs::symlink_metadata(&db) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                check()?;
                return Ok(Vec::new());
            }
            Err(e) => return Err(e.into()),
            Ok(m) if !m.file_type().is_file() => return Err(invalid()),
            Ok(_) => {}
        }
        let canonical = db
            .parent()
            .ok_or_else(invalid)?
            .canonicalize()?
            .join("history-bindings.sqlite");
        let connection = rusqlite::Connection::open_with_flags(
            canonical,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .map_err(|_| invalid())?;
        connection
            .busy_timeout(Duration::ZERO)
            .map_err(|_| invalid())?;
        let budget = crate::sqlite::RecoveryWorkBudget::new(
            deadline.saturating_duration_since(Instant::now()),
        );
        let _guard = crate::sqlite::install_budget(&connection, &budget).map_err(|_| invalid())?;
        check()?;
        let exists: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='compaction_zero_locators')", [], |r| r.get(0)).map_err(|_| invalid())?;
        if !exists {
            check()?;
            return Ok(Vec::new());
        }
        // Limit values before SQLite returns them to Rust. A corrupt oversized
        // column becomes NULL and is rejected rather than allocated as a String.
        let mut query = connection.prepare("SELECT CASE WHEN length(CAST(marker AS BLOB))=36 THEN marker END, CASE WHEN length(CAST(marker_digest AS BLOB))=64 THEN marker_digest END, CASE WHEN length(CAST(archive AS BLOB))<=47 THEN archive END, CASE WHEN length(CAST(archive_digest AS BLOB))=64 THEN archive_digest END, CASE WHEN length(CAST(tail AS BLOB)) BETWEEN 1 AND 512 THEN tail END FROM compaction_zero_locators WHERE owner=?1 LIMIT 257").map_err(|_| invalid())?;
        let mut rows = query.query([id]).map_err(|_| invalid())?;
        let mut result = Vec::new();
        loop {
            check()?;
            let Some(row) = rows.next().map_err(|_| invalid())? else {
                break;
            };
            let locator = Locator {
                marker: row.get(0).map_err(|_| invalid())?,
                marker_digest: row.get(1).map_err(|_| invalid())?,
                archive: row.get(2).map_err(|_| invalid())?,
                archive_digest: row.get(3).map_err(|_| invalid())?,
                tail: row.get(4).map_err(|_| invalid())?,
            };
            let marker = uuid::Uuid::parse_str(&locator.marker).map_err(|_| invalid())?;
            let archive_id = locator
                .archive
                .strip_prefix("s-")
                .and_then(|s| {
                    s.strip_suffix(".tlog.zst")
                        .or_else(|| s.strip_suffix(".tlog"))
                })
                .ok_or_else(invalid)?;
            let archive_id = uuid::Uuid::parse_str(archive_id).map_err(|_| invalid())?;
            if marker.to_string() != locator.marker
                || ![
                    format!("s-{archive_id}.tlog"),
                    format!("s-{archive_id}.tlog.zst"),
                ]
                .contains(&locator.archive)
                || !locator
                    .marker_digest
                    .bytes()
                    .chain(locator.archive_digest.bytes())
                    .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
                || result.len() == 256
            {
                return Err(invalid());
            }
            result.push(locator);
        }
        check()?;
        Ok(result)
    })
}

pub(crate) fn record(
    head: &Path,
    archive: &Path,
    marker: &SessionEntry,
    tail: &str,
) -> Result<(), SessionError> {
    boundary(|| {
        let id = identity(head)?;
        let name = archive
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(invalid)?;
        if archive.parent() != head.parent() || tail.is_empty() || tail.len() > 512 {
            return Err(invalid());
        }
        if load(head, Instant::now() + Duration::from_secs(5), &|| false)?.len() >= 256 {
            return Err(invalid());
        }
        let file = crate::history_page::open_source(archive).map_err(|_| invalid())?;
        let before = crate::recovery_stamp::from_file(&file).map_err(|_| invalid())?;
        let mut bytes = Vec::new();
        file.take(64 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
        if bytes.len() > 64 * 1024 * 1024 {
            return Err(invalid());
        }
        let rebound = crate::history_page::open_source(archive).map_err(|_| invalid())?;
        let after = crate::recovery_stamp::from_file(&rebound).map_err(|_| invalid())?;
        if !crate::recovery_seal::same_file_version(&before, &after).map_err(|_| invalid())? {
            return Err(invalid());
        }
        let db = head
            .parent()
            .ok_or_else(invalid)?
            .join("history-bindings.sqlite");
        let connection = crate::durable::open_bindings(&db)?;
        connection.execute_batch("CREATE TABLE IF NOT EXISTS compaction_zero_locators(owner TEXT NOT NULL, marker TEXT NOT NULL, marker_digest TEXT NOT NULL, archive TEXT NOT NULL, archive_digest TEXT NOT NULL, tail TEXT NOT NULL, PRIMARY KEY(owner,marker))").map_err(|_| invalid())?;
        connection.execute("INSERT INTO compaction_zero_locators(owner,marker,marker_digest,archive,archive_digest,tail) VALUES(?1,?2,?3,?4,?5,?6)",
            params![id, marker.id, digest(crate::compact_text::encode_entry(marker).as_bytes()), name, digest(&bytes), tail]).map_err(|_| invalid())?;
        Ok(())
    })
}

// Full source validation is required before the private tail can influence append.
// A separate Session handle avoids reacquiring the caller's held clone write lock.
// The caller must already hold that lock and the host execution lease.
pub(crate) fn recovered_tail(head: &Path) -> Result<String, SessionError> {
    let id = uuid::Uuid::parse_str(&identity(head)?).map_err(|_| invalid())?;
    let session = Session::new(id, String::new(), String::new(), head.into());
    let evidence =
        session.inspect_archive_recovery(Instant::now() + Duration::from_secs(5), || false)?;
    evidence
        .entries()
        .last()
        .map(|entry| entry.id.clone())
        .ok_or_else(invalid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locator_reads_do_not_initialize_schema_and_observe_stop() {
        let root = tempfile::tempdir().expect("root");
        let head = root.path().join(format!("{}.tlog", uuid::Uuid::new_v4()));
        let db = root.path().join("history-bindings.sqlite");
        let connection = rusqlite::Connection::open(&db).expect("db");
        connection
            .execute_batch("CREATE TABLE sentinel(value TEXT);")
            .expect("schema");
        assert!(
            load(&head, Instant::now() + Duration::from_secs(1), &|| false)
                .expect("read")
                .is_empty()
        );
        let names: Vec<String> = connection
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .expect("query")
            .query_map([], |r| r.get(0))
            .expect("rows")
            .collect::<Result<_, _>>()
            .expect("names");
        assert_eq!(names, ["sentinel"]);
        assert!(load(&head, Instant::now(), &|| false).is_err());
        assert!(load(&head, Instant::now() + Duration::from_secs(1), &|| true).is_err());
    }

    #[test]
    fn oversized_locator_fields_are_rejected() {
        let root = tempfile::tempdir().expect("root");
        let id = uuid::Uuid::new_v4();
        let head = root.path().join(format!("{id}.tlog"));
        let connection =
            rusqlite::Connection::open(root.path().join("history-bindings.sqlite")).expect("db");
        connection.execute_batch("CREATE TABLE compaction_zero_locators(owner TEXT,marker TEXT,marker_digest TEXT,archive TEXT,archive_digest TEXT,tail TEXT);").expect("schema");
        connection
            .execute(
                "INSERT INTO compaction_zero_locators VALUES(?1,?2,?3,?4,?3,?5)",
                params![
                    id.to_string(),
                    uuid::Uuid::new_v4().to_string(),
                    "0".repeat(64),
                    format!("s-{}.tlog", uuid::Uuid::new_v4()),
                    "x".repeat(1024 * 1024)
                ],
            )
            .expect("oversized row");
        assert!(load(&head, Instant::now() + Duration::from_secs(1), &|| false).is_err());
    }
}
