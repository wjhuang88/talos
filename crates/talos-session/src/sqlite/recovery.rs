//! Host-maintained recovery projection cache inside the existing Session index.

use super::{IndexError, IndexStoreError, SessionIndex};
use crate::{SessionEntry, history_projection::recovery_index_text};
use rusqlite::params;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

/// Cooperative cancellation and deadline for one recovery database operation.
#[derive(Clone)]
pub struct RecoveryWorkBudget {
    cancelled: Arc<AtomicBool>,
    deadline: Instant,
}

impl RecoveryWorkBudget {
    /// Create a work budget capped at ten seconds, even if the caller asks longer.
    pub fn new(timeout: Duration) -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
            deadline: Instant::now() + timeout.min(Duration::from_secs(10)),
        }
    }
    /// Stop all clones of this operation's budget.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }
    fn stopped(&self) -> bool {
        self.cancelled.load(Ordering::Acquire) || Instant::now() >= self.deadline
    }
    fn check(&self) -> Result<(), IndexError> {
        if self.stopped() {
            Err(IndexError::Store(IndexStoreError::Database(
                "recovery work stopped".into(),
            )))
        } else {
            Ok(())
        }
    }
}

struct ProgressGuard<'a>(&'a rusqlite::Connection, Duration);
impl Drop for ProgressGuard<'_> {
    fn drop(&mut self) {
        let _ = self.0.progress_handler(0, None::<fn() -> bool>);
        let _ = self.0.busy_timeout(self.1);
    }
}

fn install_budget<'a>(
    conn: &'a rusqlite::Connection,
    budget: &RecoveryWorkBudget,
) -> Result<ProgressGuard<'a>, IndexError> {
    budget.check()?;
    // Do not wait behind database locks while holding a cancellation-insensitive
    // SQLite busy handler. The host can retry a database-busy error.
    let prior_busy_ms: u32 = conn.query_row("PRAGMA busy_timeout", [], |r| r.get(0))?;
    conn.busy_timeout(Duration::ZERO)?;
    let guard = ProgressGuard(conn, Duration::from_millis(u64::from(prior_busy_ms)));
    let observed = budget.clone();
    conn.progress_handler(1000, Some(move || observed.stopped()))?;
    Ok(guard)
}

fn native_boundary<T>(operation: impl FnOnce() -> Result<T, IndexError>) -> Result<T, IndexError> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation)).unwrap_or_else(|_| {
        Err(IndexError::Store(IndexStoreError::Database(
            "recovery database operation panicked".into(),
        )))
    })
}

/// One source verified by the host against the authorized branch snapshot.
/// The index is a cache, not a source of branch authority.
pub struct RecoveryIndexSource<'a> {
    /// Original persisted entry, never an index-generated transcript copy.
    pub entry: &'a SessionEntry,
    /// Matching committed terminal marker for tool provenance, when needed.
    pub terminal: Option<&'a SessionEntry>,
    /// Stable source order in this snapshot.
    pub ordinal: u64,
}

/// Untrusted cache candidate requiring authoritative source and scope revalidation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryCandidate {
    /// Original entry identity; never an authorization capability by itself.
    pub entry_id: String,
    /// Snapshot order used for bounded keyset pagination.
    pub ordinal: u64,
}

impl SessionIndex {
    /// Find at most twenty candidates in a host-selected snapshot scope.
    /// An empty query discovers entries in source order. Nonempty queries use FTS
    /// only over eligible projection text, without global rank/statistics.
    /// This never initializes/repairs storage or returns raw text. The caller
    /// must verify index completeness and reread authorized sources before use.
    pub fn recovery_candidates(
        &self,
        scope: &str,
        query: &str,
        after: Option<u64>,
        limit: usize,
        budget: &RecoveryWorkBudget,
    ) -> Result<Vec<RecoveryCandidate>, IndexError> {
        native_boundary(|| {
            if scope.is_empty()
                || scope.len() > 128
                || query.len() > 1024
                || limit == 0
                || limit > 20
                || after.is_some_and(|n| n > i64::MAX as u64)
            {
                return Err(IndexError::Store(IndexStoreError::Database(
                    "invalid recovery query bounds".into(),
                )));
            }
            let local = RecoveryWorkBudget {
                cancelled: budget.cancelled.clone(),
                deadline: budget.deadline.min(Instant::now() + Duration::from_secs(2)),
            };
            let _guard = install_budget(&self.conn, &local)?;
            let sql = if query.is_empty() {
                "SELECT entry_id,ordinal FROM recovery_sources WHERE scope=?1 AND ordinal>?2 ORDER BY ordinal LIMIT ?3"
            } else {
                "SELECT s.entry_id,s.ordinal FROM recovery_sources s JOIN recovery_projection_fts f ON f.rowid=s.id WHERE s.scope=?1 AND s.ordinal>?2 AND recovery_projection_fts MATCH ?4 ORDER BY s.ordinal LIMIT ?3"
            };
            let mut statement = self.conn.prepare(sql)?;
            let cursor = after.map_or(-1, |n| n as i64);
            let mut rows = if query.is_empty() {
                statement.query(params![scope, cursor, limit as i64])?
            } else {
                statement.query(params![scope, cursor, limit as i64, query])?
            };
            let mut candidates = Vec::with_capacity(limit);
            while let Some(row) = rows.next()? {
                local.check()?;
                let id: String = row.get(0)?;
                let ordinal: i64 = row.get(1)?;
                if id.len() > 1024 || ordinal < 0 {
                    return Err(IndexError::Store(IndexStoreError::Database(
                        "invalid recovery cache row".into(),
                    )));
                }
                candidates.push(RecoveryCandidate {
                    entry_id: id,
                    ordinal: ordinal as u64,
                });
            }
            local.check()?;
            Ok(candidates)
        })
    }

    /// Initialize the additive recovery cache schema during authorized host maintenance.
    /// Model queries must never call this method implicitly.
    pub fn init_recovery_schema(&self) -> Result<(), IndexError> {
        self.init_recovery_schema_with_budget(&RecoveryWorkBudget::new(Duration::from_secs(10)))
    }

    /// Initialize both recovery tables atomically under the host's work budget.
    pub fn init_recovery_schema_with_budget(
        &self,
        budget: &RecoveryWorkBudget,
    ) -> Result<(), IndexError> {
        native_boundary(|| {
            let _guard = install_budget(&self.conn, budget)?;
            let tx = self.conn.unchecked_transaction()?;
            tx.execute_batch(
                "CREATE TABLE IF NOT EXISTS recovery_sources (
                    id INTEGER PRIMARY KEY, scope TEXT NOT NULL, entry_id TEXT NOT NULL,
                    ordinal INTEGER NOT NULL, UNIQUE(scope, entry_id), UNIQUE(scope, ordinal));
                 CREATE VIRTUAL TABLE IF NOT EXISTS recovery_projection_fts USING fts5(content);",
            )?;
            budget.check()?;
            tx.commit()?;
            Ok(())
        })
    }

    /// Atomically cache one bounded page of host-authorized projection sources.
    /// `scope` must be a fresh opaque host-issued snapshot identity, never a model
    /// supplied session ID. This does not mark indexing complete or authorize query.
    /// Retries replace only the same scope/entry mapping. Raw-only text never enters
    /// the recovery FTS table, including for ineligible tool results.
    pub fn cache_recovery_page(
        &mut self,
        scope: &str,
        sources: &[RecoveryIndexSource<'_>],
    ) -> Result<(), IndexError> {
        self.cache_recovery_page_with_budget(
            scope,
            sources,
            &RecoveryWorkBudget::new(Duration::from_secs(10)),
        )
    }

    /// Cache a page with cancellation checked during projection and SQLite work.
    /// SQLite's VM progress callback interrupts work; no detached query keeps
    /// running after the caller stops waiting. Kernel I/O remains synchronous.
    pub fn cache_recovery_page_with_budget(
        &mut self,
        scope: &str,
        sources: &[RecoveryIndexSource<'_>],
        budget: &RecoveryWorkBudget,
    ) -> Result<(), IndexError> {
        native_boundary(|| self.cache_page_inner(scope, sources, budget))
    }

    fn cache_page_inner(
        &self,
        scope: &str,
        sources: &[RecoveryIndexSource<'_>],
        budget: &RecoveryWorkBudget,
    ) -> Result<(), IndexError> {
        let _guard = install_budget(&self.conn, budget)?;
        let invalid = || {
            IndexError::Store(IndexStoreError::Database(
                "invalid recovery cache page".into(),
            ))
        };
        if scope.is_empty() || scope.len() > 128 || sources.len() > 8192 {
            return Err(invalid());
        }
        let mut bytes = 0usize;
        let mut projected = Vec::with_capacity(sources.len());
        for source in sources {
            budget.check()?;
            bytes = bytes
                .checked_add(source.entry.content.len())
                .and_then(|n| n.checked_add(source.terminal.map_or(0, |e| e.content.len())))
                .ok_or_else(invalid)?;
            if bytes > 2 * 1024 * 1024
                || source.entry.id.len() > 1024
                || source.ordinal > i64::MAX as u64
            {
                return Err(invalid());
            }
            projected.push((source, recovery_index_text(source.entry, source.terminal)));
        }
        let tx = self.conn.unchecked_transaction()?;
        for (source, content) in projected {
            budget.check()?;
            let old: Option<i64> = tx
                .query_row(
                    "SELECT id FROM recovery_sources WHERE scope=?1 AND entry_id=?2",
                    params![scope, source.entry.id],
                    |r| r.get(0),
                )
                .optional()?;
            if let Some(id) = old {
                tx.execute("DELETE FROM recovery_projection_fts WHERE rowid=?1", [id])?;
                tx.execute("DELETE FROM recovery_sources WHERE id=?1", [id])?;
            }
            if let Some(content) = content.filter(|s| !s.is_empty()) {
                tx.execute(
                    "INSERT INTO recovery_sources(scope,entry_id,ordinal) VALUES(?1,?2,?3)",
                    params![scope, source.entry.id, source.ordinal as i64],
                )?;
                let id = tx.last_insert_rowid();
                tx.execute(
                    "INSERT INTO recovery_projection_fts(rowid,content) VALUES(?1,?2)",
                    params![id, content],
                )?;
            }
        }
        budget.check()?;
        tx.commit()?;
        Ok(())
    }
}

use rusqlite::OptionalExtension;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn progress_budget_interrupts_actual_sqlite_work_and_clears_handler() {
        let dir = tempfile::tempdir().expect("tempdir");
        let index = SessionIndex::new(&dir.path().join("index.db")).expect("index");
        let budget = RecoveryWorkBudget::new(Duration::from_secs(2));
        let guard = install_budget(&index.conn, &budget).expect("install");
        let cancel = budget.clone();
        let task = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(10));
            cancel.cancel();
        });
        let result = index.conn.query_row("WITH RECURSIVE n(x) AS (VALUES(1) UNION ALL SELECT x+1 FROM n WHERE x<1000000000) SELECT sum(x) FROM n", [], |r| r.get::<_, i64>(0));
        task.join().expect("cancel thread");
        assert!(
            matches!(result, Err(rusqlite::Error::SqliteFailure(ref e, _)) if e.code == rusqlite::ErrorCode::OperationInterrupted)
        );
        drop(guard);
        assert_eq!(
            index
                .conn
                .query_row("SELECT 1", [], |r| r.get::<_, i64>(0))
                .expect("next query"),
            1
        );
    }

    #[test]
    fn panic_rolls_back_and_restores_connection_settings() {
        let dir = tempfile::tempdir().expect("tempdir");
        let index = SessionIndex::new(&dir.path().join("index.db")).expect("index");
        index.init_recovery_schema().expect("schema");
        index
            .conn
            .busy_timeout(Duration::from_millis(123))
            .expect("timeout");
        let result: Result<(), IndexError> = native_boundary(|| {
            let budget = RecoveryWorkBudget::new(Duration::from_secs(1));
            let _guard = install_budget(&index.conn, &budget)?;
            let tx = index.conn.unchecked_transaction()?;
            tx.execute(
                "INSERT INTO recovery_sources(scope,entry_id,ordinal) VALUES('scope','partial',0)",
                [],
            )?;
            panic!("injected fixture panic");
        });
        assert!(
            result
                .expect_err("caught panic")
                .to_string()
                .contains("operation panicked")
        );
        assert_eq!(
            index
                .conn
                .query_row("SELECT count(*) FROM recovery_sources", [], |r| r
                    .get::<_, i64>(0))
                .expect("count"),
            0
        );
        assert_eq!(
            index
                .conn
                .query_row("PRAGMA busy_timeout", [], |r| r.get::<_, u32>(0))
                .expect("restored"),
            123
        );
    }

    #[test]
    fn expired_schema_budget_creates_no_tables() {
        let dir = tempfile::tempdir().expect("tempdir");
        let index = SessionIndex::new(&dir.path().join("index.db")).expect("index");
        assert!(
            index
                .init_recovery_schema_with_budget(&RecoveryWorkBudget::new(Duration::ZERO))
                .is_err()
        );
        assert_eq!(
            index
                .conn
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE name LIKE 'recovery_%'",
                    [],
                    |r| r.get::<_, i64>(0)
                )
                .expect("tables"),
            0
        );
    }

    #[test]
    fn scoped_query_filters_before_limit_and_ignores_sibling_rank() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut index = SessionIndex::new(&dir.path().join("index.db")).expect("index");
        index.init_recovery_schema().expect("schema");
        let source = |id: &str, content: &str| SessionEntry {
            id: id.into(),
            parent_id: None,
            timestamp: chrono::Utc::now(),
            role: "user".into(),
            content: content.into(),
            metadata: Default::default(),
        };
        let first = source("first", "needle");
        let second = source("second", "needle needle");
        index
            .cache_recovery_page(
                "current",
                &[
                    RecoveryIndexSource {
                        entry: &first,
                        terminal: None,
                        ordinal: 1,
                    },
                    RecoveryIndexSource {
                        entry: &second,
                        terminal: None,
                        ordinal: 2,
                    },
                ],
            )
            .expect("current");
        let budget = RecoveryWorkBudget::new(Duration::from_secs(2));
        let before = index
            .recovery_candidates("current", "needle", None, 1, &budget)
            .expect("query");
        let sibling = source("foreign", "needle ".repeat(100).as_str());
        index
            .cache_recovery_page(
                "sibling",
                &[RecoveryIndexSource {
                    entry: &sibling,
                    terminal: None,
                    ordinal: 0,
                }],
            )
            .expect("sibling");
        assert_eq!(
            index
                .recovery_candidates("current", "needle", None, 1, &budget)
                .expect("query"),
            before
        );
        assert_eq!(before[0].entry_id, "first");
        let page = index
            .recovery_candidates("current", "needle", Some(1), 1, &budget)
            .expect("next");
        assert_eq!(page[0].entry_id, "second");
        assert_eq!(
            index
                .recovery_candidates("current", "", None, 20, &budget)
                .expect("discover")
                .len(),
            2
        );
        let changes = index.conn.total_changes();
        assert!(
            index
                .recovery_candidates("current", "needle", None, 21, &budget)
                .is_err()
        );
        assert_eq!(index.conn.total_changes(), changes);
    }

    #[test]
    fn queries_do_not_repair_missing_schema_or_mutate_on_failure() {
        let dir = tempfile::tempdir().expect("tempdir");
        let index = SessionIndex::new(&dir.path().join("index.db")).expect("index");
        let budget = RecoveryWorkBudget::new(Duration::from_secs(2));
        assert!(
            index
                .recovery_candidates("scope", "", None, 1, &budget)
                .is_err()
        );
        let tables: i64 = index
            .conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE name LIKE 'recovery_%'",
                [],
                |r| r.get(0),
            )
            .expect("tables");
        assert_eq!(tables, 0);
        index.init_recovery_schema().expect("host maintenance");
        // Populate the selected scope so SQLite evaluates MATCH rather than
        // short-circuiting the join on an empty source table.
        index.conn.execute("INSERT INTO recovery_sources(id,scope,entry_id,ordinal) VALUES(1,'scope','entry',0)", []).expect("source");
        index
            .conn
            .execute(
                "INSERT INTO recovery_projection_fts(rowid,content) VALUES(1,'public')",
                [],
            )
            .expect("projection");
        let changes = index.conn.total_changes();
        let cancelled = RecoveryWorkBudget::new(Duration::from_secs(2));
        cancelled.cancel();
        for stopped in [cancelled, RecoveryWorkBudget::new(Duration::ZERO)] {
            assert!(
                index
                    .recovery_candidates("scope", "", None, 1, &stopped)
                    .is_err()
            );
        }
        assert!(
            index
                .recovery_candidates("scope", "\"", None, 1, &budget)
                .is_err()
        );
        assert_eq!(index.conn.total_changes(), changes);
        assert!(
            index
                .recovery_candidates("scope", "", None, 1, &budget)
                .expect("connection usable after failure")
                .len()
                == 1
        );
    }

    #[test]
    fn cancelled_page_does_not_mutate_cache() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut index = SessionIndex::new(&dir.path().join("index.db")).expect("index");
        index.init_recovery_schema().expect("schema");
        let budget = RecoveryWorkBudget::new(Duration::from_secs(1));
        budget.cancel();
        assert!(
            index
                .cache_recovery_page_with_budget("scope", &[], &budget)
                .is_err()
        );
        assert_eq!(
            index
                .conn
                .query_row("SELECT count(*) FROM recovery_sources", [], |r| r
                    .get::<_, i64>(0))
                .expect("count"),
            0
        );
    }

    #[test]
    fn cache_excludes_raw_only_text_and_rolls_back_conflicting_pages() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut index = SessionIndex::new(&dir.path().join("index.db")).expect("index");
        index.init_recovery_schema().expect("schema");
        let entry = |id: &str, role: &str, content: &str| SessionEntry {
            id: id.into(),
            parent_id: None,
            timestamp: chrono::Utc::now(),
            role: role.into(),
            content: content.into(),
            metadata: Default::default(),
        };
        let public = entry("one", "user", "public fact");
        let hidden = entry("two", "system", "__OK__:call__\nHIDDEN_ONLY");
        index
            .cache_recovery_page(
                "scope",
                &[
                    RecoveryIndexSource {
                        entry: &public,
                        terminal: None,
                        ordinal: 0,
                    },
                    RecoveryIndexSource {
                        entry: &hidden,
                        terminal: None,
                        ordinal: 1,
                    },
                ],
            )
            .expect("cache");
        let count: i64 = index.conn.query_row("SELECT count(*) FROM recovery_projection_fts WHERE recovery_projection_fts MATCH 'HIDDEN_ONLY'", [], |r| r.get(0)).expect("count");
        assert_eq!(count, 0);
        let other = entry("other", "user", "rollback");
        assert!(
            index
                .cache_recovery_page(
                    "scope",
                    &[RecoveryIndexSource {
                        entry: &other,
                        terminal: None,
                        ordinal: 0
                    }]
                )
                .is_err()
        );
        let text: String = index
            .conn
            .query_row("SELECT content FROM recovery_projection_fts", [], |r| {
                r.get(0)
            })
            .expect("retained");
        assert_eq!(text, "public fact");
        index
            .cache_recovery_page(
                "scope",
                &[RecoveryIndexSource {
                    entry: &public,
                    terminal: None,
                    ordinal: 0,
                }],
            )
            .expect("retry");
        let count: i64 = index
            .conn
            .query_row("SELECT count(*) FROM recovery_sources", [], |r| r.get(0))
            .expect("count");
        assert_eq!(count, 1);
    }
}
