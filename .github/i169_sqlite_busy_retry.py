from pathlib import Path

path = Path("crates/talos-session/src/pending_submission.rs")
text = path.read_text()

replacements = [
    (
        "use std::time::Duration;",
        "use std::time::{Duration, Instant};",
    ),
    (
        "const MAX_TOMBSTONES: usize = MAX_PENDING_SUBMISSIONS * 2;",
        """const MAX_TOMBSTONES: usize = MAX_PENDING_SUBMISSIONS * 2;
const SQLITE_BUSY_TIMEOUT: Duration = Duration::from_secs(5);
const SQLITE_BUSY_RETRY_DELAY: Duration = Duration::from_millis(10);""",
    ),
    (
        """        let connection = Connection::open(self.path.as_ref())?;
        connection.busy_timeout(Duration::from_secs(5))?;
        connection.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL;")?;
        Ok(connection)
    }
}

type RecordTuple""",
        """        let connection = Connection::open(self.path.as_ref())?;
        connection.busy_timeout(SQLITE_BUSY_TIMEOUT)?;
        retry_sqlite_busy(SQLITE_BUSY_TIMEOUT, || {
            connection.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL;")
        })?;
        Ok(connection)
    }
}

fn retry_sqlite_busy<T>(
    timeout: Duration,
    mut operation: impl FnMut() -> Result<T, rusqlite::Error>,
) -> Result<T, rusqlite::Error> {
    let deadline = Instant::now() + timeout;
    loop {
        match operation() {
            Err(error) if sqlite_is_busy_or_locked(&error) && Instant::now() < deadline => {
                let remaining = deadline.saturating_duration_since(Instant::now());
                std::thread::sleep(SQLITE_BUSY_RETRY_DELAY.min(remaining));
            }
            result => return result,
        }
    }
}

fn sqlite_is_busy_or_locked(error: &rusqlite::Error) -> bool {
    matches!(
        error,
        rusqlite::Error::SqliteFailure(code, _)
            if matches!(
                code.code,
                rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked
            )
    )
}

type RecordTuple""",
    ),
]

for old, new in replacements:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"expected one anchor, found {count}: {old[:120]!r}")
    text = text.replace(old, new, 1)

path.write_text(text)
