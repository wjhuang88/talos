//! Immutable physical ownership recorded before owned archives become discoverable.
use crate::{SessionError, compaction_locator::digest};
use rusqlite::{Connection, OpenFlags, params};
use std::{
    path::Path,
    time::{Duration, Instant},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Ownership {
    pub name: String,
    pub owner: String,
    pub digest: String,
}

fn invalid() -> SessionError {
    SessionError::ParseError("archive publication ownership unavailable".into())
}

pub(crate) fn archive_id(name: &str) -> Option<uuid::Uuid> {
    let stem = name
        .strip_prefix("s-owned-")
        .or_else(|| name.strip_prefix("s-"))?;
    let id = stem
        .strip_suffix(".tlog.zst")
        .or_else(|| stem.strip_suffix(".tlog"))?;
    let parsed = uuid::Uuid::parse_str(id).ok()?;
    (parsed.to_string() == id).then_some(parsed)
}

pub(crate) fn owned(name: &str) -> bool {
    name.starts_with("s-owned-") && (name.ends_with(".tlog") || name.ends_with(".tlog.zst"))
}

pub(crate) fn record(head: &Path, archive: &Path, bytes: &[u8]) -> Result<(), SessionError> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let name = archive.file_name().and_then(|s| s.to_str()).ok_or_else(invalid)?;
        if !owned(name) || archive_id(name).is_none() || head.parent() != archive.parent() {
            return Err(invalid());
        }
        let owner = head.file_stem().and_then(|s| s.to_str()).ok_or_else(invalid)?;
        let id = uuid::Uuid::parse_str(owner).map_err(|_| invalid())?;
        if id.to_string() != owner { return Err(invalid()); }
        let db = head.parent().ok_or_else(invalid)?.join("history-bindings.sqlite");
        let connection = crate::durable::open_bindings(&db)?;
        connection.execute_batch("CREATE TABLE IF NOT EXISTS archive_publication_intents (archive TEXT PRIMARY KEY, owner TEXT NOT NULL, digest TEXT NOT NULL)").map_err(|_| invalid())?;
        // Never overwrite an earlier owner, including pending/failed publications.
        connection.execute("INSERT INTO archive_publication_intents(archive,owner,digest) VALUES(?1,?2,?3)", params![name,owner,digest(bytes)]).map_err(|_| invalid())?;
        Ok(())
    })).unwrap_or_else(|_| Err(invalid()))
}

/// Read only records for the bounded physical inventory, including foreign owners.
pub(crate) fn load(
    head: &Path,
    names: &[String],
    deadline: Instant,
    stopped: &impl Fn() -> bool,
) -> Result<Vec<Ownership>, SessionError> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let check = || if stopped() || Instant::now() >= deadline { Err(invalid()) } else { Ok(()) };
        check()?;
        if names.len() > 256 { return Err(invalid()); }
        if names.is_empty() { return Ok(Vec::new()); }
        let db = head.parent().ok_or_else(invalid)?.join("history-bindings.sqlite");
        if !std::fs::symlink_metadata(&db)?.file_type().is_file() { return Err(invalid()); }
        let db = db.parent().ok_or_else(invalid)?.canonicalize()?.join("history-bindings.sqlite");
        let connection = Connection::open_with_flags(db, OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW).map_err(|_| invalid())?;
        connection.busy_timeout(Duration::ZERO).map_err(|_| invalid())?;
        let budget = crate::sqlite::RecoveryWorkBudget::new(deadline.saturating_duration_since(Instant::now()));
        let _guard = crate::sqlite::install_budget(&connection, &budget).map_err(|_| invalid())?;
        let mut query = connection.prepare("SELECT CASE WHEN length(CAST(owner AS BLOB))=36 THEN owner END, CASE WHEN length(CAST(digest AS BLOB))=64 THEN digest END FROM archive_publication_intents WHERE archive=?1 LIMIT 2").map_err(|_| invalid())?;
        let mut result = Vec::new();
        for name in names {
            check()?;
            if !owned(name) || archive_id(name).is_none() { return Err(invalid()); }
            let mut rows = query.query([name]).map_err(|_| invalid())?;
            let row = rows.next().map_err(|_| invalid())?.ok_or_else(invalid)?;
            let owner: String = row.get(0).map_err(|_| invalid())?;
            let hash: String = row.get(1).map_err(|_| invalid())?;
            let id = uuid::Uuid::parse_str(&owner).map_err(|_| invalid())?;
            if id.to_string() != owner || !hash.bytes().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()) || rows.next().map_err(|_| invalid())?.is_some() {
                return Err(invalid());
            }
            result.push(Ownership { name: name.clone(), owner, digest: hash });
        }
        check()?;
        Ok(result)
    })).unwrap_or_else(|_| Err(invalid()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_name_never_accepts_missing_custody_or_overwrites_owner() {
        let root = tempfile::tempdir().expect("root");
        let head = root.path().join(format!("{}.tlog", uuid::Uuid::new_v4()));
        let name = format!("s-owned-{}.tlog", uuid::Uuid::new_v4());
        let archive = root.path().join(&name);
        let names = vec![name];
        let deadline = || Instant::now() + Duration::from_secs(5);
        assert!(load(&head, &names, deadline(), &|| false).is_err());
        assert!(!root.path().join("history-bindings.sqlite").exists());
        record(&head, &archive, b"original").expect("intent");
        let initial = load(&head, &names, deadline(), &|| false).expect("read");
        let other = root.path().join(format!("{}.tlog", uuid::Uuid::new_v4()));
        assert!(record(&other, &archive, b"replacement").is_err());
        assert_eq!(
            load(&head, &names, deadline(), &|| false).expect("same"),
            initial
        );
        assert!(load(&head, &names, Instant::now(), &|| false).is_err());
        assert!(load(&head, &names, deadline(), &|| true).is_err());
    }
}
