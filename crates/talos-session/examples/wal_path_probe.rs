//! Isolated diagnostic for native SQLite path spelling and concurrent WAL opens.
//! Run with `cargo run --locked -p talos-session --example wal_path_probe`.
//! A failed case is reported and exits nonzero; this is not recovery acceptance.
use rusqlite::{Connection, OpenFlags};
use std::{
    path::Path,
    sync::Barrier,
    time::{Duration, Instant},
};

fn shape(path: &str) -> &'static str {
    if path.starts_with(r"\\?\UNC\") {
        "extended-unc"
    } else if path.starts_with(r"\\?\") {
        "extended"
    } else if path.starts_with(r"\\") {
        "unc"
    } else {
        "ordinary"
    }
}

fn ordinary_fixture_path(path: &Path) -> std::path::PathBuf {
    // Only this generated short local-drive fixture is converted. Never apply this to
    // user paths: stripping the prefix can change Windows filename semantics.
    #[cfg(windows)]
    {
        let text = path.to_str().expect("UTF-8 temporary fixture path");
        if let Some(drive) = text.strip_prefix(r"\\?\") {
            assert!(
                drive.as_bytes().get(1) == Some(&b':'),
                "local drive required"
            );
            assert!(drive.len() < 240, "short fixture required");
            return drive.into();
        }
    }
    path.into()
}

fn worker(path: &Path, schema: &str) -> (bool, &'static str, &'static str, u128, u32, Option<i32>) {
    let start = Instant::now();
    let mut stage = "open";
    let mut filename_shape = "unopened";
    let mut attempts = 0;
    let result = (|| -> rusqlite::Result<()> {
        let mut connection = Connection::open_with_flags(
            path,
            OpenFlags::default() | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )?;
        // sqlite3_db_filename spelling does not establish the internal VFS branch.
        filename_shape = shape(connection.path().unwrap_or(""));
        connection.busy_timeout(Duration::ZERO)?;
        let deadline = Instant::now() + Duration::from_millis(500);
        loop {
            attempts += 1;
            let attempt = (|| -> rusqlite::Result<()> {
                stage = "journal-mode";
                connection.execute_batch("PRAGMA journal_mode=WAL;")?;
                stage = "synchronous";
                connection.execute_batch("PRAGMA synchronous=FULL;")?;
                stage = "begin";
                let transaction = connection.transaction()?;
                stage = "schema";
                transaction.execute_batch(schema)?;
                stage = "commit";
                transaction.commit()
            })();
            match attempt {
                Ok(()) => {
                    stage = "done";
                    return Ok(());
                }
                Err(error)
                    if connection.is_autocommit()
                        && attempts <= 20
                        && Instant::now() < deadline
                        && matches!(
                            error.sqlite_error_code(),
                            Some(
                                rusqlite::ErrorCode::DatabaseBusy
                                    | rusqlite::ErrorCode::DatabaseLocked
                            )
                        ) =>
                {
                    std::thread::sleep(
                        Duration::from_millis(25)
                            .min(deadline.saturating_duration_since(Instant::now())),
                    );
                    if Instant::now() >= deadline {
                        return Err(error);
                    }
                }
                Err(error) => return Err(error),
            }
        }
    })();
    let code = result
        .as_ref()
        .err()
        .and_then(|e| e.sqlite_error())
        .map(|e| e.extended_code);
    (
        result.is_ok(),
        filename_shape,
        stage,
        start.elapsed().as_micros(),
        attempts,
        code,
    )
}

fn main() {
    let schema = include_str!("wal_path_probe_schema.sql");
    let mut failed = false;
    println!("os={} sqlite={}", std::env::consts::OS, rusqlite::version());
    for round in 0..4 {
        for canonical in if round % 2 == 0 {
            [false, true]
        } else {
            [true, false]
        } {
            let directory = tempfile::tempdir().expect("isolated fixture");
            let canonical_path = directory
                .path()
                .canonicalize()
                .expect("canonical directory")
                .join("probe.sqlite");
            let path = if canonical {
                canonical_path
            } else {
                ordinary_fixture_path(&canonical_path)
            };
            println!(
                "round={round} canonical={canonical} input_shape={} fixture={:?}",
                shape(path.to_str().expect("UTF-8 fixture path")),
                directory.path()
            );
            let barrier = Barrier::new(6);
            let results = std::thread::scope(|scope| {
                let handles: Vec<_> = (0..6)
                    .map(|_| {
                        scope.spawn(|| {
                            barrier.wait();
                            worker(&path, schema)
                        })
                    })
                    .collect();
                handles
                    .into_iter()
                    .map(|h| h.join().expect("worker panic"))
                    .collect::<Vec<_>>()
            });
            for (worker, result) in results.iter().enumerate() {
                println!("round={round} canonical={canonical} worker={worker} result={result:?}");
                failed |= !result.0;
            }
            // Retain the directory/database. SQLite may remove WAL/SHM when the
            // last worker connection closes; this is not a sidecar snapshot.
            let _retained = directory.keep();
        }
    }
    if failed {
        std::process::exit(1);
    }
}
