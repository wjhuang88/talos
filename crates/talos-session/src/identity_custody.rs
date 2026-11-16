//! Host-private, permanent identity reservations for receipt-bound sessions.
//!
//! This ledger is not a transcript receipt. Callers must hold their publication
//! or cleanup lease and enforce the returned identity at every file boundary.

use std::path::Path;

use rusqlite::params;
use uuid::Uuid;

use crate::SessionError;

/// Operation-local ownership, distinct from the host's execution lease.
/// Permanent files prevent deletion/recreation from splitting lock identity.
#[derive(Clone)]
pub(crate) struct IdentityLease {
    _file: Option<std::sync::Arc<IdentityLock>>,
    key: Option<std::path::PathBuf>,
}

// Arc owns the logical lease lifetime. Closing the last local File alone can
// leave flock held by a transient descriptor inherited during process creation.
// Unlock only when the final logical lease ends, not when any clone is dropped.
struct IdentityLock(std::fs::File);

impl Drop for IdentityLock {
    fn drop(&mut self) {
        // Release failure must not panic during unwinding. File still closes as
        // the fallback; a failed unlock can delay availability, never grant access.
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.0.unlock()));
    }
}

impl IdentityLease {
    fn acquire(directory: &Path, id: Uuid, exclusive: bool) -> Result<Self, SessionError> {
        boundary(|| {
            std::fs::create_dir_all(directory)?;
            let directory = directory.canonicalize()?.join(".identity-locks");
            match std::fs::create_dir(&directory) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => return Err(e.into()),
            }
            if !std::fs::symlink_metadata(&directory)?.file_type().is_dir() {
                return Err(invalid());
            }
            let key = directory.join(id.to_string());
            let mut options = std::fs::OpenOptions::new();
            options.read(true).write(true).create(true).truncate(false);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
            }
            #[cfg(windows)]
            {
                use std::os::windows::fs::OpenOptionsExt;
                options.custom_flags(
                    windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT,
                );
            }
            let file = options.open(&key)?;
            let metadata = file.metadata()?;
            if !metadata.is_file() {
                return Err(invalid());
            }
            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                if metadata.file_attributes()
                    & windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT
                    != 0
                {
                    return Err(invalid());
                }
            }
            #[cfg(test)]
            custody_stage(if exclusive {
                "exclusive-lock"
            } else {
                "shared-lock"
            });
            if exclusive {
                file.try_lock()
            } else {
                file.try_lock_shared()
            }
            .map_err(|_| invalid())?;
            #[cfg(test)]
            custody_stage("lock-acquired");
            Ok(Self {
                _file: Some(std::sync::Arc::new(IdentityLock(file))),
                key: Some(key),
            })
        })
    }

    fn transcript(path: &Path, exclusive: bool) -> Result<Self, SessionError> {
        match path
            .file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| Uuid::parse_str(s).ok())
        {
            Some(id) => Self::acquire(path.parent().ok_or_else(invalid)?, id, exclusive),
            None => Ok(Self {
                _file: None,
                key: None,
            }),
        }
    }

    pub(crate) fn check_path(&self, path: &Path) -> Result<(), SessionError> {
        let expected = match path
            .file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| Uuid::parse_str(s).ok())
        {
            Some(id) => Some(
                path.parent()
                    .ok_or_else(invalid)?
                    .canonicalize()?
                    .join(".identity-locks")
                    .join(id.to_string()),
            ),
            None => None,
        };
        if self.key != expected {
            return Err(invalid());
        }
        Ok(())
    }
}

#[cfg(test)]
type CustodyTrace = (std::time::Instant, Vec<(&'static str, u128)>);

#[cfg(test)]
thread_local! {
    static CUSTODY_TRACE: std::cell::RefCell<Option<CustodyTrace>> = const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
pub(crate) fn start_custody_trace() {
    CUSTODY_TRACE.with(|trace| {
        *trace.borrow_mut() = Some((std::time::Instant::now(), Vec::new()));
    });
}

#[cfg(test)]
pub(crate) fn custody_stage(stage: &'static str) {
    CUSTODY_TRACE.with(|trace| {
        if let Some((start, events)) = trace.borrow_mut().as_mut()
            && events.len() < 64
        {
            events.push((stage, start.elapsed().as_micros()));
        }
    });
}

#[cfg(test)]
pub(crate) fn finish_custody_trace() -> Vec<(&'static str, u128)> {
    CUSTODY_TRACE.with(|trace| {
        trace
            .borrow_mut()
            .take()
            .map(|(_, events)| events)
            .unwrap_or_default()
    })
}

fn invalid() -> SessionError {
    SessionError::ParseError("session identity custody unavailable".into())
}

fn boundary<T>(operation: impl FnOnce() -> Result<T, SessionError>) -> Result<T, SessionError> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation))
        .unwrap_or_else(|_| Err(invalid()))
}

fn open(path: &Path) -> Result<rusqlite::Connection, SessionError> {
    let connection = crate::durable::open_bindings(path)?;
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS session_identity_custody (
                session_id TEXT PRIMARY KEY,
                incarnation TEXT NOT NULL UNIQUE,
                retired INTEGER NOT NULL CHECK(retired IN (0,1))
             );",
        )
        .map_err(|_| invalid())?;
    Ok(connection)
}

// Shared operation lease, acquired before checking retirement. The caller must
// retain it through completion; this does not validate an envelope/fork receipt.
pub(crate) fn require_not_retired(transcript: &Path) -> Result<IdentityLease, SessionError> {
    require_not_retired_until(
        transcript,
        std::time::Instant::now() + std::time::Duration::from_secs(1),
    )
}

pub(crate) fn require_not_retired_until(
    transcript: &Path,
    deadline: std::time::Instant,
) -> Result<IdentityLease, SessionError> {
    boundary(|| {
        let budget = crate::sqlite::RecoveryWorkBudget::new(
            deadline
                .saturating_duration_since(std::time::Instant::now())
                .min(std::time::Duration::from_secs(1)),
        );
        budget.check().map_err(|_| invalid())?;
        let lease = IdentityLease::transcript(transcript, false)?;
        check_not_retired(transcript, &budget, &lease)?;
        Ok(lease)
    })
}

// Reconciliation owns the exclusive identity lease across retirement and
// quarantine. A committed receipt is never eligible, even after caller timeout.
pub(crate) fn retire_pending_fork(
    transcript: &Path,
    deadline: std::time::Instant,
) -> Result<IdentityLease, SessionError> {
    boundary(|| {
        let budget = crate::sqlite::RecoveryWorkBudget::new(
            deadline.saturating_duration_since(std::time::Instant::now()),
        );
        budget.check().map_err(|_| invalid())?;
        let id = transcript
            .file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(invalid)?;
        if transcript.file_name().and_then(|s| s.to_str()) != Some(&format!("{id}.tlog")) {
            return Err(invalid());
        }
        let lease = IdentityLease::transcript(transcript, true)?;
        let database = transcript
            .parent()
            .ok_or_else(invalid)?
            .join("history-bindings.sqlite");
        let mut connection = crate::durable::open_binding_connection_with_flags(
            database,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .map_err(|_| invalid())?;
        connection
            .busy_timeout(std::time::Duration::ZERO)
            .map_err(|_| invalid())?;
        let connection =
            crate::sqlite::install_budget(&mut connection, &budget).map_err(|_| invalid())?;
        let mut query = connection
            .prepare("SELECT published FROM archived_fork_receipts WHERE child=?1 LIMIT 2")
            .map_err(|_| invalid())?;
        let mut rows = query.query([id.to_string()]).map_err(|_| invalid())?;
        let row = rows.next().map_err(|_| invalid())?.ok_or_else(invalid)?;
        let published: i64 = row.get(0).map_err(|_| invalid())?;
        if published != 0 || rows.next().map_err(|_| invalid())?.is_some() {
            return Err(invalid());
        }
        drop(rows);
        drop(query);
        connection
            .execute_batch("PRAGMA synchronous=FULL")
            .map_err(|_| invalid())?;
        if connection
            .execute(
                "UPDATE session_identity_custody SET retired=1 WHERE session_id=?1",
                [id.to_string()],
            )
            .map_err(|_| invalid())?
            != 1
        {
            return Err(invalid());
        }
        budget.check().map_err(|_| invalid())?;
        Ok(lease)
    })
}

// Publisher-only acquisition: a pending intent deliberately blocks ordinary
// readers/writers. This bypasses only publication state, not active incarnation.
pub(crate) fn lock_reserved_identity(
    transcript: &Path,
    incarnation: Uuid,
    deadline: std::time::Instant,
) -> Result<IdentityLease, SessionError> {
    boundary(|| {
        if std::time::Instant::now() >= deadline {
            return Err(invalid());
        }
        let id = transcript
            .file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(invalid)?;
        let lease = IdentityLease::transcript(transcript, true)?;
        let database = transcript
            .parent()
            .ok_or_else(invalid)?
            .join("history-bindings.sqlite");
        require_active_session_identity_until(&database, id, incarnation, deadline)?;
        if std::time::Instant::now() >= deadline {
            return Err(invalid());
        }
        Ok(lease)
    })
}

fn check_not_retired(
    transcript: &Path,
    budget: &crate::sqlite::RecoveryWorkBudget,
    lease: &IdentityLease,
) -> Result<(), SessionError> {
    boundary(|| {
        budget.check().map_err(|_| invalid())?;
        let Some(id) = transcript
            .file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| Uuid::parse_str(s).ok())
        else {
            return Ok(());
        };
        let database = transcript
            .parent()
            .ok_or_else(invalid)?
            .join("history-bindings.sqlite");
        match std::fs::symlink_metadata(&database) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(e.into()),
            Ok(meta) if !meta.file_type().is_file() => return Err(invalid()),
            Ok(_) => {}
        }
        let mut connection = crate::durable::open_binding_connection_with_flags(
            &database,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .map_err(|_| invalid())?;
        connection
            .busy_timeout(std::time::Duration::ZERO)
            .map_err(|_| invalid())?;
        let connection =
            crate::sqlite::install_budget(&mut connection, budget).map_err(|_| invalid())?;
        if crate::fork_receipt::check_publication(&connection, id)? {
            // A published receipt must never fall back to a stripped legacy
            // header, including through ordinary append/atomic writer paths.
            if transcript.extension().and_then(|s| s.to_str()) != Some("tlog") {
                return Err(invalid());
            }
            let mut file = crate::history_page::open_source_under_identity(transcript, lease)
                .map_err(|_| invalid())?;
            let mut marker = [0u8; crate::fork_receipt::ENVELOPE.len()];
            std::io::Read::read_exact(&mut file, &mut marker)?;
            if marker != crate::fork_receipt::ENVELOPE {
                return Err(invalid());
            }
        }
        let exists: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type='table' AND name='session_identity_custody')",
            [], |row| row.get(0)).map_err(|_| invalid())?;
        if !exists {
            return budget.check().map_err(|_| invalid());
        }
        let mut query = connection
            .prepare("SELECT retired FROM session_identity_custody WHERE session_id=?1 LIMIT 2")
            .map_err(|_| invalid())?;
        let mut rows = query.query([id.to_string()]).map_err(|_| invalid())?;
        if let Some(row) = rows.next().map_err(|_| invalid())? {
            let state: i64 = row.get(0).map_err(|_| invalid())?;
            if state != 0 || rows.next().map_err(|_| invalid())?.is_some() {
                return Err(invalid());
            }
        }
        budget.check().map_err(|_| invalid())
    })
}

// Called under the artifact cleanup lease, before deleting sidecars or the head.
// Legacy sessions without this schema remain untouched; never create custody as
// a side effect of deleting an ordinary transcript.
pub(crate) fn retire_registered_transcript(
    transcript: &Path,
) -> Result<IdentityLease, SessionError> {
    let lease = IdentityLease::transcript(transcript, true)?;
    retire_registered_under_lease(transcript)?;
    Ok(lease)
}

fn retire_registered_under_lease(transcript: &Path) -> Result<(), SessionError> {
    boundary(|| {
        let database = transcript
            .parent()
            .ok_or_else(invalid)?
            .join("history-bindings.sqlite");
        match std::fs::symlink_metadata(&database) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(e.into()),
            Ok(meta) if !meta.file_type().is_file() => return Err(invalid()),
            Ok(_) => {}
        }
        let id = transcript
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(invalid)?;
        let id = Uuid::parse_str(id).map_err(|_| invalid())?;
        let mut connection = crate::durable::open_binding_connection_with_flags(
            &database,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .map_err(|_| invalid())?;
        connection
            .busy_timeout(std::time::Duration::ZERO)
            .map_err(|_| invalid())?;
        let budget = crate::sqlite::RecoveryWorkBudget::new(std::time::Duration::from_secs(1));
        let connection =
            crate::sqlite::install_budget(&mut connection, &budget).map_err(|_| invalid())?;
        let exists: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type='table' AND name='session_identity_custody')",
            [], |row| row.get(0),
        ).map_err(|_| invalid())?;
        if !exists {
            return Ok(());
        }
        connection
            .execute_batch("PRAGMA synchronous=FULL")
            .map_err(|_| invalid())?;
        connection
            .execute(
                "UPDATE session_identity_custody SET retired=1 WHERE session_id=?1",
                [id.to_string()],
            )
            .map_err(|_| invalid())?;
        Ok(())
    })
}

/// Permanently reserve a fresh session UUID in a host-selected private database.
///
/// Transcript hosts must use the sibling `history-bindings.sqlite`, matching the
/// artifact cleanup boundary. Other database locations are not retired by cleanup.
/// Duplicate reservations fail, even for an active identity. Commit this before
/// publishing any associated file. A failed or cancelled publication must retire
/// the reservation, never delete it. This does not validate source history or mint
/// a fork receipt. The caller must retain the returned incarnation in its receipt.
/// If a crash loses the returned incarnation, retire that UUID and reserve a new
/// one; repeating reservation must never recover an existing active identity.
pub fn reserve_session_identity(database: &Path, session_id: Uuid) -> Result<Uuid, SessionError> {
    boundary(|| {
        let _identity =
            IdentityLease::acquire(database.parent().ok_or_else(invalid)?, session_id, true)?;
        let connection = open(database)?;
        let incarnation = Uuid::new_v4();
        connection
            .execute(
                "INSERT INTO session_identity_custody(session_id,incarnation,retired)
                 VALUES(?1,?2,0)",
                params![session_id.to_string(), incarnation.to_string()],
            )
            .map_err(|_| invalid())?;
        Ok(incarnation)
    })
}

// Adopt an existing source, never a destination reservation. Return the stable
// incarnation together with a shared lease that the fork publisher must retain
// through capture and publication. No transcript/provenance authority is minted.
pub(crate) fn adopt_fork_source(
    transcript: &Path,
    deadline: std::time::Instant,
) -> Result<(Uuid, IdentityLease), SessionError> {
    boundary(|| {
        use rusqlite::OptionalExtension;
        let budget = crate::sqlite::RecoveryWorkBudget::new(
            deadline.saturating_duration_since(std::time::Instant::now()),
        );
        budget.check().map_err(|_| invalid())?;
        let id = transcript
            .file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(invalid)?;
        if transcript.file_name().and_then(|s| s.to_str()) != Some(&format!("{id}.tlog")) {
            return Err(invalid());
        }
        let directory = transcript.parent().ok_or_else(invalid)?;
        let database = directory.join("history-bindings.sqlite");
        let incarnation = {
            let lease = IdentityLease::acquire(directory, id, true)?;
            check_not_retired(transcript, &budget, &lease)?;
            // A missing source must not silently become a reserved new identity.
            let source = crate::history_page::open_source_under_identity(transcript, &lease)
                .map_err(|_| invalid())?;
            let mut connection = open(&database)?;
            let connection =
                crate::sqlite::install_budget(&mut connection, &budget).map_err(|_| invalid())?;
            let existing: Option<(String, i64)> = connection
                .query_row(
                    "SELECT CASE WHEN length(CAST(incarnation AS BLOB))=36 THEN incarnation END,retired FROM session_identity_custody WHERE session_id=?1",
                    [id.to_string()],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()
                .map_err(|_| invalid())?;
            let incarnation = match existing {
                Some((value, 0)) => {
                    let parsed = Uuid::parse_str(&value).map_err(|_| invalid())?;
                    if parsed.is_nil() || parsed.to_string() != value {
                        return Err(invalid());
                    }
                    parsed
                }
                Some(_) => return Err(invalid()),
                None => {
                    let value = Uuid::new_v4();
                    connection.execute(
                        "INSERT INTO session_identity_custody(session_id,incarnation,retired) VALUES(?1,?2,0)",
                        params![id.to_string(), value.to_string()],
                    ).map_err(|_| invalid())?;
                    value
                }
            };
            drop(source);
            incarnation
        };
        // Reacquisition is not assumed atomic: retirement in this gap must fail
        // the recheck. Only the returned lease authorizes the capture lifetime.
        let lease = require_not_retired_until(transcript, deadline)?;
        require_active_session_identity_until(&database, id, incarnation, deadline)?;
        let _source = crate::history_page::open_source_under_identity(transcript, &lease)
            .map_err(|_| invalid())?;
        budget.check().map_err(|_| invalid())?;
        Ok((incarnation, lease))
    })
}

/// Irreversibly retire an identity before removing its published artifacts.
///
/// Repeated retirement succeeds. Retiring an unregistered UUID reserves a tombstone
/// so it cannot subsequently be issued. Existing incarnation values are preserved:
/// retirement must not revoke independently validated descendant receipts.
/// The host must hold its cleanup lease; this operation does not delete any files.
pub fn retire_session_identity(database: &Path, session_id: Uuid) -> Result<(), SessionError> {
    boundary(|| {
        let _identity =
            IdentityLease::acquire(database.parent().ok_or_else(invalid)?, session_id, true)?;
        let connection = open(database)?;
        connection
            .execute(
                "INSERT INTO session_identity_custody(session_id,incarnation,retired)
                 VALUES(?1,?2,1)
                 ON CONFLICT(session_id) DO UPDATE SET retired=1",
                params![session_id.to_string(), Uuid::new_v4().to_string()],
            )
            .map_err(|_| invalid())?;
        Ok(())
    })
}

/// Verify that a receipt's identity is still active in existing private custody.
///
/// Missing databases, missing schema/rows, malformed values, retired identities,
/// or incarnation mismatches fail closed. This read creates no database or schema.
/// It is an identity check only, not proof that transcript bytes match a receipt.
/// It proves state only at query time; the caller's lifecycle lease must exclude
/// retirement until publication or mutation completes. SQL work is bounded to one
/// second, with no lock waiting; synchronous filesystem I/O is not interruptible.
pub fn require_active_session_identity(
    database: &Path,
    session_id: Uuid,
    incarnation: Uuid,
) -> Result<(), SessionError> {
    require_active_session_identity_until(
        database,
        session_id,
        incarnation,
        std::time::Instant::now() + std::time::Duration::from_secs(1),
    )
}

pub(crate) fn require_active_session_identity_until(
    database: &Path,
    session_id: Uuid,
    incarnation: Uuid,
    deadline: std::time::Instant,
) -> Result<(), SessionError> {
    boundary(|| {
        #[cfg(test)]
        custody_stage("active-deadline");
        if std::time::Instant::now() >= deadline {
            return Err(invalid());
        }
        #[cfg(test)]
        custody_stage("active-metadata");
        if !std::fs::symlink_metadata(database)?.file_type().is_file() {
            return Err(invalid());
        }
        #[cfg(test)]
        custody_stage("active-open");
        let mut connection = crate::durable::open_binding_connection_with_flags(
            database,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .map_err(|_| invalid())?;
        #[cfg(test)]
        custody_stage("active-busy-config");
        connection
            .busy_timeout(std::time::Duration::ZERO)
            .map_err(|_| invalid())?;
        let budget = crate::sqlite::RecoveryWorkBudget::new(
            deadline
                .saturating_duration_since(std::time::Instant::now())
                .min(std::time::Duration::from_secs(1)),
        );
        #[cfg(test)]
        custody_stage("active-budget-install");
        let connection =
            crate::sqlite::install_budget(&mut connection, &budget).map_err(|_| invalid())?;
        #[cfg(test)]
        custody_stage("active-prepare");
        let mut query = connection
            .prepare("SELECT CASE WHEN length(CAST(incarnation AS BLOB))=36 THEN incarnation END,retired FROM session_identity_custody WHERE session_id=?1 LIMIT 2")
            .map_err(|_| invalid())?;
        #[cfg(test)]
        custody_stage("active-query");
        let mut rows = query
            .query([session_id.to_string()])
            .map_err(|_| invalid())?;
        #[cfg(test)]
        custody_stage("active-row");
        let row = rows.next().map_err(|_| invalid())?.ok_or_else(invalid)?;
        #[cfg(test)]
        custody_stage("active-decode");
        let stored: String = row.get(0).map_err(|_| invalid())?;
        let retired: i64 = row.get(1).map_err(|_| invalid())?;
        #[cfg(test)]
        custody_stage("active-validate");
        if stored != incarnation.to_string()
            || retired != 0
            || rows.next().map_err(|_| invalid())?.is_some()
        {
            return Err(invalid());
        }
        #[cfg(test)]
        custody_stage("active-budget-final");
        budget.check().map_err(|_| invalid())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn final_logical_lease_releases_lock_with_extra_file_reference() {
        let root = tempfile::tempdir().expect("root");
        let id = Uuid::new_v4();
        for exclusive in [false, true] {
            let lease = IdentityLease::acquire(root.path(), id, exclusive).expect("lease");
            // A duplicate models the extra open-file reference retained during spawn.
            // It is not another logical custody lease and must not extend custody.
            let descriptor = lease
                ._file
                .as_ref()
                .expect("file")
                .0
                .try_clone()
                .expect("duplicate");
            let clone = lease.clone();
            drop(lease);
            assert!(IdentityLease::acquire(root.path(), id, true).is_err());
            drop(clone);
            let next = IdentityLease::acquire(root.path(), id, true)
                .expect("last logical lease releases despite extra descriptor");
            drop(descriptor);
            assert!(IdentityLease::acquire(root.path(), id, true).is_err());
            drop(next);
            IdentityLease::acquire(root.path(), id, true).expect("next lease released");
        }
    }

    #[test]
    fn releasing_one_independent_shared_lease_keeps_other_custody() {
        let root = tempfile::tempdir().expect("root");
        let id = Uuid::new_v4();
        let first = IdentityLease::acquire(root.path(), id, false).expect("first");
        let second = IdentityLease::acquire(root.path(), id, false).expect("second");
        drop(first);
        assert!(IdentityLease::acquire(root.path(), id, true).is_err());
        drop(second);
        IdentityLease::acquire(root.path(), id, true).expect("all readers released");
    }

    #[test]
    fn fork_source_adoption_is_stable_and_lease_excludes_retirement() {
        let root = tempfile::tempdir().expect("root");
        let id = Uuid::new_v4();
        let head = root.path().join(format!("{id}.tlog"));
        let database = root.path().join("history-bindings.sqlite");
        let deadline = || std::time::Instant::now() + std::time::Duration::from_secs(5);
        assert!(adopt_fork_source(&head, deadline()).is_err());
        assert!(!database.exists());
        std::fs::write(&head, b"TALOS\t1\t123\n").expect("source");
        let (first, lease) = adopt_fork_source(&head, deadline()).expect("adopt");
        assert!(retire_session_identity(&database, id).is_err());
        assert!(reserve_session_identity(&database, id).is_err());
        drop(lease);
        let (second, lease) = adopt_fork_source(&head, deadline()).expect("reuse identity");
        assert_eq!(first, second);
        drop(lease);
        retire_session_identity(&database, id).expect("retire");
        assert!(adopt_fork_source(&head, deadline()).is_err());
        assert_eq!(
            std::fs::read(&head).expect("source intact"),
            b"TALOS\t1\t123\n"
        );
    }

    #[test]
    fn missing_transcript_read_creates_only_permanent_lock_anchor() {
        use crate::SessionStore;
        let root = tempfile::tempdir().expect("root");
        let directory = root.path().join("missing");
        let id = Uuid::new_v4();
        let head = directory.join(format!("{id}.tlog"));
        assert!(
            crate::CompactTextSessionStore
                .read_entries(&head)
                .expect("empty legacy head")
                .is_empty()
        );
        assert!(!head.exists());
        assert!(!directory.join("history-bindings.sqlite").exists());
        assert!(
            directory
                .join(".identity-locks")
                .join(id.to_string())
                .is_file()
        );
        assert_eq!(std::fs::read_dir(&directory).expect("directory").count(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn readonly_parent_refuses_identity_read_without_modifying_transcript() {
        use crate::SessionStore;
        use std::os::unix::fs::PermissionsExt;
        let root = tempfile::tempdir().expect("root");
        let directory = root.path().join("readonly");
        std::fs::create_dir(&directory).expect("directory");
        let head = directory.join(format!("{}.tlog", Uuid::new_v4()));
        let bytes = b"TALOS\t1\t123\n";
        std::fs::write(&head, bytes).expect("head");
        std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o500))
            .expect("readonly");
        // Probe actual host enforcement: privileged runners can bypass mode bits.
        let probe = directory.join("permission-probe");
        let denied = std::fs::write(&probe, b"probe").is_err();
        let result = crate::CompactTextSessionStore.read_entries(&head);
        let anchor_created = directory.join(".identity-locks").exists();
        std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700))
            .expect("restore permissions");
        if denied {
            assert!(result.is_err());
            assert!(!anchor_created);
        }
        assert_eq!(std::fs::read(&head).expect("unchanged head"), bytes);
        assert!(!directory.join("history-bindings.sqlite").exists());
    }

    #[test]
    fn identity_lease_process_child() {
        let Some(head) = std::env::var_os("TALOS_TEST_IDENTITY_LOCK_HEAD") else {
            return;
        };
        let head = std::path::PathBuf::from(head);
        let _lease = require_not_retired(&head).expect("child operation lease");
        let ready = head.with_extension("ready");
        std::fs::write(&ready, b"locked").expect("ready");
        let release = head.with_extension("release");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        while !release.exists() {
            assert!(std::time::Instant::now() < deadline, "release timeout");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        // Exit without destructors: the OS must release the file lock.
        std::process::exit(73);
    }

    #[test]
    fn process_exit_releases_identity_lease_without_removing_anchor() {
        let root = tempfile::tempdir().expect("root");
        let id = Uuid::new_v4();
        let head = root.path().join(format!("{id}.tlog"));
        let database = root.path().join("history-bindings.sqlite");
        let incarnation = reserve_session_identity(&database, id).expect("reserve");
        std::fs::write(&head, b"preserved").expect("head");
        let mut child = std::process::Command::new(std::env::current_exe().expect("test binary"))
            .args([
                "--exact",
                "identity_custody::tests::identity_lease_process_child",
            ])
            .env("TALOS_TEST_IDENTITY_LOCK_HEAD", &head)
            .stdout(std::process::Stdio::null())
            .spawn()
            .expect("child");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        while !head.with_extension("ready").exists() {
            if std::time::Instant::now() >= deadline
                || child.try_wait().expect("child status").is_some()
            {
                let _ = child.kill();
                let _ = child.wait();
                panic!("child did not acquire lock");
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let retirement = retire_session_identity(&database, id);
        let deletion = crate::artifacts::remove_session_transcript(&head);
        let still_active = require_active_session_identity(&database, id, incarnation);
        std::fs::write(head.with_extension("release"), b"exit").expect("release child");
        let status = loop {
            if let Some(status) = child.try_wait().expect("child status") {
                break status;
            }
            if std::time::Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("child exit timeout");
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        };
        assert_eq!(status.code(), Some(73));
        assert!(retirement.is_err());
        assert!(deletion.is_err());
        still_active.expect("failed cleanup must not retire");
        assert_eq!(std::fs::read(&head).expect("untouched"), b"preserved");
        crate::artifacts::remove_session_transcript(&head).expect("retry after process exit");
        assert!(!head.exists());
        assert!(require_active_session_identity(&database, id, incarnation).is_err());
        assert!(
            root.path()
                .join(".identity-locks")
                .join(id.to_string())
                .is_file()
        );
    }

    #[test]
    fn operation_lease_excludes_adoption_and_retirement_across_formats() {
        let root = tempfile::tempdir().expect("root");
        let id = Uuid::new_v4();
        let head = root.path().join(format!("{id}.tlog"));
        let alias = root.path().join(format!("{id}.jsonl"));
        let database = root.path().join("history-bindings.sqlite");
        let legacy = require_not_retired(&head).expect("legacy operation");
        assert!(reserve_session_identity(&database, id).is_err());
        assert!(IdentityLease::transcript(&alias, true).is_err());
        let nested = legacy.clone();
        drop(legacy);
        assert!(retire_session_identity(&database, id).is_err());
        drop(nested);
        let incarnation = reserve_session_identity(&database, id).expect("adopt after operation");
        let active = require_not_retired(&head).expect("active operation");
        assert!(retire_session_identity(&database, id).is_err());
        require_active_session_identity(&database, id, incarnation)
            .expect("failed retire changed nothing");
        drop(active);
        retire_session_identity(&database, id).expect("retire after operation");
        assert!(require_not_retired(&alias).is_err());
        assert!(
            root.path()
                .join(".identity-locks")
                .join(id.to_string())
                .is_file()
        );
    }

    #[test]
    fn expired_identity_budget_does_not_allow_legacy_or_registered_sources() {
        let root = tempfile::tempdir().expect("root");
        let head = root.path().join(format!("{}.tlog", Uuid::new_v4()));
        let expired = std::time::Instant::now();
        assert!(require_not_retired_until(&head, expired).is_err());
        assert!(!root.path().join("history-bindings.sqlite").exists());
        assert!(matches!(
            crate::history_page::open_source_until(&head, expired),
            Err(crate::history_page::HistoryPageError::Stopped)
        ));
        let id = Uuid::new_v4();
        let head = root.path().join(format!("{id}.tlog"));
        reserve_session_identity(&root.path().join("history-bindings.sqlite"), id)
            .expect("reserve");
        assert!(require_not_retired_until(&head, expired).is_err());
        require_not_retired(&head).expect("active identity with fresh budget");
    }

    #[test]
    fn stale_stores_cannot_read_or_append_after_failed_cleanup() {
        use crate::store::{CompactTextSessionStore, JsonlSessionStore, SessionStore};
        for store in [
            &JsonlSessionStore as &dyn SessionStore,
            &CompactTextSessionStore,
        ] {
            let root = tempfile::tempdir().expect("root");
            let id = Uuid::new_v4();
            let head = root.path().join(format!("{id}.{}", store.file_extension()));
            let database = root.path().join("history-bindings.sqlite");
            reserve_session_identity(&database, id).expect("reserve");
            let entry = crate::SessionEntry {
                id: Uuid::new_v4().to_string(),
                parent_id: None,
                timestamp: chrono::Utc::now(),
                role: "user".into(),
                content: "original history".into(),
                metadata: crate::SessionMetadata::default(),
            };
            store.append_entry(&head, &entry).expect("active append");
            assert_eq!(store.read_entries(&head).expect("active read").len(), 1);
            let before = std::fs::read(&head).expect("original bytes");
            assert!(
                crate::artifacts::cleanup_session_with_index(&head, || Err(invalid())).is_err()
            );
            assert!(store.read_entries(&head).is_err());
            assert!(store.append_entry(&head, &entry).is_err());
            assert!(store.replace_entries_atomically(&head, &[]).is_err());
            assert!(
                store
                    .replace_with_retained_suffix(&head, std::slice::from_ref(&entry), 0, &[])
                    .is_err()
            );
            assert!(store.scan_file(&head).is_err());
            assert!(store.read_bytes(&head).is_err());
            assert!(store.read_last_entry_id(&head).is_none());
            assert!(
                crate::history_page::read_history_page(
                    &head,
                    store.history_encoding().expect("built-in encoding"),
                    0,
                    before.len() as u64,
                    std::time::Instant::now() + std::time::Duration::from_secs(5),
                    || false,
                )
                .is_err()
            );
            assert_eq!(std::fs::read(&head).expect("retained bytes"), before);
            crate::artifacts::cleanup_session_with_index(&head, || Ok(())).expect("retry");
            assert!(store.append_entry(&head, &entry).is_err());
            assert!(store.replace_entries_atomically(&head, &[]).is_err());
            assert!(
                !head.exists(),
                "stale writer must not recreate retired transcript"
            );
        }
    }

    #[test]
    fn legacy_database_without_identity_schema_is_not_migrated_by_deletion() {
        let root = tempfile::tempdir().expect("root");
        let database = root.path().join("history-bindings.sqlite");
        let head = root.path().join(format!("{}.tlog", Uuid::new_v4()));
        let connection = rusqlite::Connection::open(&database).expect("legacy database");
        connection
            .execute_batch(
                "CREATE TABLE sentinel(value TEXT); INSERT INTO sentinel VALUES('keep');",
            )
            .expect("legacy schema");
        std::fs::write(&head, b"legacy head").expect("head");
        crate::artifacts::remove_session_artifacts_for_transcript(&head).expect("delete legacy");
        assert!(!head.exists());
        let value: String = connection
            .query_row("SELECT value FROM sentinel", [], |row| row.get(0))
            .expect("sentinel");
        assert_eq!(value, "keep");
        let count: i64 = connection
            .query_row(
                "SELECT count(*) FROM sqlite_schema WHERE name='session_identity_custody'",
                [],
                |row| row.get(0),
            )
            .expect("schema");
        assert_eq!(count, 0);
    }

    #[test]
    fn artifact_cleanup_retires_before_index_failure_and_allows_retry() {
        let root = tempfile::tempdir().expect("root");
        let id = Uuid::new_v4();
        let head = root.path().join(format!("{id}.tlog"));
        let database = root.path().join("history-bindings.sqlite");
        let incarnation = reserve_session_identity(&database, id).expect("reserve");
        std::fs::write(&head, b"preserved head").expect("head");
        assert!(crate::artifacts::cleanup_session_with_index(&head, || Err(invalid())).is_err());
        assert_eq!(
            std::fs::read(&head).expect("head remains"),
            b"preserved head"
        );
        assert!(require_active_session_identity(&database, id, incarnation).is_err());
        crate::artifacts::cleanup_session_with_index(&head, || Ok(())).expect("retry");
        assert!(!head.exists());
        assert!(reserve_session_identity(&database, id).is_err());
    }

    #[test]
    fn direct_deletion_retires_but_sidecar_only_cleanup_does_not() {
        for all in [false, true] {
            let root = tempfile::tempdir().expect("root");
            let id = Uuid::new_v4();
            let head = root.path().join(format!("{id}.tlog"));
            let database = root.path().join("history-bindings.sqlite");
            let incarnation = reserve_session_identity(&database, id).expect("reserve");
            std::fs::write(&head, b"head").expect("head");
            crate::artifacts::remove_session_sidecars_for_transcript(&head).expect("sidecars");
            require_active_session_identity(&database, id, incarnation).expect("still active");
            if all {
                crate::artifacts::remove_session_artifacts_for_transcript(&head).expect("all");
            } else {
                crate::artifacts::remove_session_transcript(&head).expect("transcript");
            }
            assert!(!head.exists());
            assert!(require_active_session_identity(&database, id, incarnation).is_err());
        }
    }

    #[test]
    fn duplicate_or_malformed_custody_rows_never_authorize() {
        let root = tempfile::tempdir().expect("root");
        let database = root.path().join("history-bindings.sqlite");
        let id = Uuid::new_v4();
        let incarnation = reserve_session_identity(&database, id).expect("reserve");
        let connection = open(&database).expect("open");
        connection
            .execute_batch(
                "ALTER TABLE session_identity_custody RENAME TO old_identity;
            CREATE TABLE session_identity_custody AS SELECT * FROM old_identity;
            INSERT INTO session_identity_custody SELECT * FROM old_identity;",
            )
            .expect("damaged schema fixture");
        assert!(require_active_session_identity(&database, id, incarnation).is_err());
        connection
            .execute_batch("DELETE FROM session_identity_custody;")
            .expect("clear fixture");
        for state in ["2", "invalid"] {
            connection
                .execute(
                    "INSERT INTO session_identity_custody VALUES(?1,?2,?3)",
                    params![id.to_string(), incarnation.to_string(), state],
                )
                .expect("malformed row");
            assert!(require_active_session_identity(&database, id, incarnation).is_err());
            connection
                .execute_batch("DELETE FROM session_identity_custody;")
                .expect("next fixture");
        }
    }

    #[test]
    fn competing_reservations_issue_exactly_one_identity() {
        let root = tempfile::tempdir().expect("root");
        let database = root.path().join("history-bindings.sqlite");
        drop(open(&database).expect("initialize"));
        let id = Uuid::new_v4();
        let barrier = std::sync::Barrier::new(2);
        let results = std::thread::scope(|scope| {
            let one = scope.spawn(|| {
                barrier.wait();
                reserve_session_identity(&database, id)
            });
            let two = scope.spawn(|| {
                barrier.wait();
                reserve_session_identity(&database, id)
            });
            [
                one.join().expect("first worker"),
                two.join().expect("second worker"),
            ]
        });
        let issued: Vec<_> = results.into_iter().filter_map(Result::ok).collect();
        assert_eq!(issued.len(), 1);
        require_active_session_identity(&database, id, issued[0]).expect("winner active");
    }

    #[test]
    fn retirement_racing_reservation_never_leaves_reusable_identity() {
        let root = tempfile::tempdir().expect("root");
        let database = root.path().join("history-bindings.sqlite");
        drop(open(&database).expect("initialize"));
        for _ in 0..16 {
            let id = Uuid::new_v4();
            let barrier = std::sync::Barrier::new(2);
            let reservation = std::thread::scope(|scope| {
                let reserve = scope.spawn(|| {
                    barrier.wait();
                    reserve_session_identity(&database, id)
                });
                let retire = scope.spawn(|| {
                    barrier.wait();
                    retire_session_identity(&database, id)
                });
                retire.join().expect("retirement worker").ok();
                reserve.join().expect("reservation worker")
            });
            // Lock contention is nonblocking; retry after both workers finish.
            retire_session_identity(&database, id).expect("retirement retry");
            assert!(reserve_session_identity(&database, id).is_err());
            if let Ok(incarnation) = reservation {
                assert!(require_active_session_identity(&database, id, incarnation).is_err());
            }
            let connection = open(&database).expect("reopen");
            let retired: i64 = connection
                .query_row(
                    "SELECT retired FROM session_identity_custody WHERE session_id=?1",
                    [id.to_string()],
                    |row| row.get(0),
                )
                .expect("permanent row");
            assert_eq!(retired, 1);
        }
    }

    #[test]
    fn retirement_survives_reopen_without_reassigning_incarnation() {
        let root = tempfile::tempdir().expect("root");
        let database = root.path().join("history-bindings.sqlite");
        let parent = Uuid::new_v4();
        let child = Uuid::new_v4();
        let incarnation = reserve_session_identity(&database, parent).expect("reserve parent");
        let child_incarnation = reserve_session_identity(&database, child).expect("reserve child");
        require_active_session_identity(&database, parent, incarnation).expect("active");
        assert!(reserve_session_identity(&database, parent).is_err());
        assert!(require_active_session_identity(&database, parent, child_incarnation).is_err());
        retire_session_identity(&database, parent).expect("retire before delete");
        retire_session_identity(&database, parent).expect("retry cleanup");
        assert!(require_active_session_identity(&database, parent, incarnation).is_err());
        assert!(reserve_session_identity(&database, parent).is_err());
        require_active_session_identity(&database, child, child_incarnation)
            .expect("child independent");
        let connection = open(&database).expect("cold reopen");
        let stored: String = connection
            .query_row(
                "SELECT incarnation FROM session_identity_custody WHERE session_id=?1",
                [parent.to_string()],
                |row| row.get(0),
            )
            .expect("retained origin");
        assert_eq!(stored, incarnation.to_string());
    }

    #[test]
    fn absent_custody_never_mints_identity_and_retirement_reserves_unknown_uuid() {
        let root = tempfile::tempdir().expect("root");
        let database = root.path().join("history-bindings.sqlite");
        let id = Uuid::new_v4();
        assert!(require_active_session_identity(&database, id, Uuid::new_v4()).is_err());
        assert!(!database.exists());
        retire_session_identity(&database, id).expect("reserve tombstone");
        assert!(reserve_session_identity(&database, id).is_err());
    }
}
