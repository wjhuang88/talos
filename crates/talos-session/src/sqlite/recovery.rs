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
    /// Exact original-record span, when supplied by the bounded source reader.
    /// Missing locations cannot support bounded source disclosure.
    pub record_range: Option<std::ops::Range<u64>>,
}

/// Untrusted cache candidate requiring authoritative source and scope revalidation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryCandidate {
    /// Original entry identity; never an authorization capability by itself.
    pub entry_id: String,
    /// Snapshot order used for bounded keyset pagination.
    pub ordinal: u64,
    /// Exact original-record span, when supplied by the bounded source reader.
    /// Missing locations cannot support bounded source disclosure.
    pub record_range: Option<std::ops::Range<u64>>,
}

/// Persisted scan position for one immutable, host-authorized source snapshot.
/// This is cache progress, not proof of source or branch authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryBuildCursor {
    source_end: u64,
    next_offset: u64,
    next_ordinal: u64,
}

impl RecoveryBuildCursor {
    /// Captured byte cutoff; later appends are outside this build.
    pub fn source_end(&self) -> u64 {
        self.source_end
    }
    /// Byte position from which host maintenance resumes.
    pub fn next_offset(&self) -> u64 {
        self.next_offset
    }
    /// Ordinal of the next scanned entry, including ineligible entries.
    pub fn next_ordinal(&self) -> u64 {
        self.next_ordinal
    }
    /// Whether all bytes through the captured cutoff have been scanned.
    pub fn complete(&self) -> bool {
        self.next_offset == self.source_end
    }
}

/// One bounded host-authorized scan page with its next source position.
pub struct RecoveryBuildPage<'a> {
    /// Byte offset after the complete records parsed in this page.
    pub next_offset: u64,
    /// Number of scanned records, including records ineligible for recovery.
    pub scanned_entries: u64,
    /// Eligible sources and their original scan ordinals.
    pub sources: &'a [RecoveryIndexSource<'a>],
}

struct RecoveryPageAdvance<'a> {
    prior: &'a RecoveryBuildCursor,
    next_offset: u64,
    scanned_entries: u64,
}

impl SessionIndex {
    /// Begin host maintenance for a fresh opaque snapshot scope. Existing state
    /// is never replaced: resume it or issue a new scope after revalidation.
    pub fn begin_recovery_build(
        &self,
        scope: &str,
        source_end: u64,
        budget: &RecoveryWorkBudget,
    ) -> Result<RecoveryBuildCursor, IndexError> {
        native_boundary(|| {
            if scope.is_empty() || scope.len() > 128 || source_end > i64::MAX as u64 {
                return Err(invalid_build());
            }
            let _guard = install_budget(&self.conn, budget)?;
            let tx = self.conn.unchecked_transaction()?;
            let existing: i64 = tx.query_row(
                "SELECT (SELECT count(*) FROM recovery_sources WHERE scope=?1) + (SELECT count(*) FROM recovery_terminals WHERE scope=?1)",
                [scope],
                |r| r.get(0),
            )?;
            if existing != 0 {
                return Err(invalid_build());
            }
            tx.execute("INSERT INTO recovery_builds(scope,source_end,next_offset,next_ordinal) VALUES(?1,?2,0,0)", params![scope, source_end as i64])?;
            budget.check()?;
            tx.commit()?;
            Ok(RecoveryBuildCursor {
                source_end,
                next_offset: 0,
                next_ordinal: 0,
            })
        })
    }

    /// Read persisted maintenance progress without initializing or repairing it.
    /// A complete cursor still requires current source/branch revalidation.
    pub fn recovery_build_cursor(
        &self,
        scope: &str,
        budget: &RecoveryWorkBudget,
    ) -> Result<Option<RecoveryBuildCursor>, IndexError> {
        native_boundary(|| {
            if scope.is_empty() || scope.len() > 128 {
                return Err(invalid_build());
            }
            let _guard = install_budget(&self.conn, budget)?;
            let row: Option<(i64, i64, i64)> = self.conn.query_row(
                "SELECT source_end,next_offset,next_ordinal FROM recovery_builds WHERE scope=?1",
                [scope], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?)),
            ).optional()?;
            budget.check()?;
            row.map(|(end, offset, ordinal)| {
                if end < 0 || offset < 0 || offset > end || ordinal < 0 {
                    return Err(invalid_build());
                }
                Ok(RecoveryBuildCursor {
                    source_end: end as u64,
                    next_offset: offset as u64,
                    next_ordinal: ordinal as u64,
                })
            })
            .transpose()
        })
    }

    /// Atomically cache an authorized page and advance its resumable scan cursor.
    /// `scanned_entries` includes entries omitted from eligible projections.
    /// Source parsing, immutable-prefix verification and branch membership are
    /// host responsibilities. A stale/concurrent cursor rolls back the whole page.
    pub fn advance_recovery_build(
        &mut self,
        scope: &str,
        prior: &RecoveryBuildCursor,
        page: RecoveryBuildPage<'_>,
        budget: &RecoveryWorkBudget,
    ) -> Result<(), IndexError> {
        native_boundary(|| {
            self.cache_page_inner(
                scope,
                page.sources,
                budget,
                Some(RecoveryPageAdvance {
                    prior,
                    next_offset: page.next_offset,
                    scanned_entries: page.scanned_entries,
                }),
            )
        })
    }

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
            let building: bool = self.conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM recovery_builds WHERE scope=?1 AND next_offset<>source_end)",
                [scope], |r| r.get(0),
            )?;
            if building {
                return Err(IndexError::Store(IndexStoreError::Database(
                    "recovery index is building".into(),
                )));
            }
            let sql = if query.is_empty() {
                "SELECT s.entry_id,s.ordinal,l.start,l.end FROM recovery_sources s LEFT JOIN recovery_locations l ON l.source_id=s.id WHERE s.scope=?1 AND s.ordinal>?2 ORDER BY s.ordinal LIMIT ?3"
            } else {
                "SELECT s.entry_id,s.ordinal,l.start,l.end FROM recovery_sources s JOIN recovery_projection_fts f ON f.rowid=s.id LEFT JOIN recovery_locations l ON l.source_id=s.id WHERE s.scope=?1 AND s.ordinal>?2 AND recovery_projection_fts MATCH ?4 ORDER BY s.ordinal LIMIT ?3"
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
                let start: Option<i64> = row.get(2)?;
                let end: Option<i64> = row.get(3)?;
                let record_range = match (start, end) {
                    (None, None) => None,
                    (Some(start), Some(end))
                        if start >= 0
                            && end > start
                            && end - start <= crate::history_page::HISTORY_PAGE_BYTES as i64 =>
                    {
                        Some(start as u64..end as u64)
                    }
                    _ => return Err(invalid_build()),
                };
                candidates.push(RecoveryCandidate {
                    entry_id: id,
                    ordinal: ordinal as u64,
                    record_range,
                });
            }
            local.check()?;
            Ok(candidates)
        })
    }

    /// Locate an original terminal marker during host maintenance, including one
    /// encountered on a later scan page. This returns an untrusted locator only;
    /// reread and verify the original marker and branch scope before disclosure.
    pub fn recovery_terminal_candidate(
        &self,
        scope: &str,
        turn_id: &str,
        budget: &RecoveryWorkBudget,
    ) -> Result<Option<RecoveryCandidate>, IndexError> {
        native_boundary(|| {
            if scope.is_empty() || scope.len() > 128 || turn_id.is_empty() || turn_id.len() > 1024 {
                return Err(invalid_build());
            }
            let _guard = install_budget(&self.conn, budget)?;
            let row: Option<(String, i64, i64, i64)> = self.conn.query_row(
                "SELECT entry_id,ordinal,start,end FROM recovery_terminals WHERE scope=?1 AND turn_id=?2",
                params![scope,turn_id], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?))).optional()?;
            budget.check()?;
            row.map(|(entry_id, ordinal, start, end)| {
                if entry_id.len() > 1024
                    || ordinal < 0
                    || start < 0
                    || end <= start
                    || end - start > crate::history_page::HISTORY_PAGE_BYTES as i64
                {
                    return Err(invalid_build());
                }
                Ok(RecoveryCandidate {
                    entry_id,
                    ordinal: ordinal as u64,
                    record_range: Some(start as u64..end as u64),
                })
            })
            .transpose()
        })
    }

    /// Initialize the additive recovery cache schema during authorized host maintenance.
    /// Model queries must never call this method implicitly.
    pub fn init_recovery_schema(&self) -> Result<(), IndexError> {
        self.init_recovery_schema_with_budget(&RecoveryWorkBudget::new(Duration::from_secs(10)))
    }

    /// Initialize recovery cache tables atomically under the host's work budget.
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
                 CREATE VIRTUAL TABLE IF NOT EXISTS recovery_projection_fts USING fts5(content);
                 CREATE TABLE IF NOT EXISTS recovery_builds (
                    scope TEXT PRIMARY KEY, source_end INTEGER NOT NULL,
                    next_offset INTEGER NOT NULL, next_ordinal INTEGER NOT NULL);
                 CREATE TABLE IF NOT EXISTS recovery_locations (
                    source_id INTEGER PRIMARY KEY, start INTEGER NOT NULL, end INTEGER NOT NULL);
                 CREATE TABLE IF NOT EXISTS recovery_terminals (
                    scope TEXT NOT NULL, turn_id TEXT NOT NULL, entry_id TEXT NOT NULL,
                    ordinal INTEGER NOT NULL, start INTEGER NOT NULL, end INTEGER NOT NULL,
                    PRIMARY KEY(scope, turn_id), UNIQUE(scope, entry_id));",
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
        native_boundary(|| self.cache_page_inner(scope, sources, budget, None))
    }

    fn cache_page_inner(
        &self,
        scope: &str,
        sources: &[RecoveryIndexSource<'_>],
        budget: &RecoveryWorkBudget,
        advance: Option<RecoveryPageAdvance<'_>>,
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
            if source.record_range.as_ref().is_some_and(|r| {
                r.start >= r.end
                    || r.end > i64::MAX as u64
                    || r.end - r.start > crate::history_page::HISTORY_PAGE_BYTES as u64
            }) {
                return Err(invalid());
            }
            projected.push((source, recovery_index_text(source.entry, source.terminal)));
        }
        let tx = self.conn.unchecked_transaction()?;
        let append_only = advance.is_some();
        if let Some(advance) = advance {
            let prior = advance.prior;
            let ordinal = prior
                .next_ordinal
                .checked_add(advance.scanned_entries)
                .filter(|n| *n <= i64::MAX as u64)
                .ok_or_else(invalid_build)?;
            if prior.complete()
                || advance.next_offset <= prior.next_offset
                || advance.next_offset > prior.source_end
                || advance.next_offset - prior.next_offset
                    > crate::history_page::HISTORY_PAGE_BYTES as u64
                || advance.scanned_entries > crate::history_page::HISTORY_PAGE_ENTRIES as u64
                || sources.len() as u64 > advance.scanned_entries
                || sources.iter().any(|s| {
                    s.ordinal < prior.next_ordinal
                        || s.ordinal >= ordinal
                        || s.record_range.as_ref().is_some_and(|r| {
                            r.start < prior.next_offset || r.end > advance.next_offset
                        })
                })
            {
                return Err(invalid_build());
            }
            let changed = tx.execute(
                "UPDATE recovery_builds SET next_offset=?1,next_ordinal=?2 WHERE scope=?3 AND source_end=?4 AND next_offset=?5 AND next_ordinal=?6",
                params![advance.next_offset as i64, ordinal as i64, scope, prior.source_end as i64, prior.next_offset as i64, prior.next_ordinal as i64],
            )?;
            if changed != 1 {
                return Err(invalid_build());
            }
        } else {
            let tracked: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM recovery_builds WHERE scope=?1)",
                [scope],
                |r| r.get(0),
            )?;
            if tracked {
                return Err(invalid_build());
            }
        }
        for (source, content) in projected {
            budget.check()?;
            if !append_only {
                tx.execute(
                    "DELETE FROM recovery_terminals WHERE scope=?1 AND entry_id=?2",
                    params![scope, source.entry.id],
                )?;
            }
            if source.entry.role == "system"
                && let Some(marker) =
                    crate::turn_outcome::decode_turn_transcript_outcome(&source.entry.content)
                && marker.version == 1
                && source.entry.metadata.turn_id.as_deref() == Some(marker.turn_id.as_str())
                && let Some(range) = &source.record_range
            {
                if marker.turn_id.is_empty() || marker.turn_id.len() > 1024 {
                    return Err(invalid_build());
                }
                // Duplicate terminal authority is ambiguous, even across pages.
                // UNIQUE conflicts roll back the page and its cursor together.
                tx.execute("INSERT INTO recovery_terminals(scope,turn_id,entry_id,ordinal,start,end) VALUES(?1,?2,?3,?4,?5,?6)",
                    params![scope,marker.turn_id,source.entry.id,source.ordinal as i64,range.start as i64,range.end as i64])?;
            }
            let old: Option<i64> = tx
                .query_row(
                    "SELECT id FROM recovery_sources WHERE scope=?1 AND entry_id=?2",
                    params![scope, source.entry.id],
                    |r| r.get(0),
                )
                .optional()?;
            if let Some(id) = old {
                if append_only {
                    return Err(invalid_build());
                }
                tx.execute("DELETE FROM recovery_locations WHERE source_id=?1", [id])?;
                tx.execute("DELETE FROM recovery_projection_fts WHERE rowid=?1", [id])?;
                tx.execute("DELETE FROM recovery_sources WHERE id=?1", [id])?;
            }
            if let Some(content) = content.filter(|s| !s.is_empty()) {
                tx.execute(
                    "INSERT INTO recovery_sources(scope,entry_id,ordinal) VALUES(?1,?2,?3)",
                    params![scope, source.entry.id, source.ordinal as i64],
                )?;
                let id = tx.last_insert_rowid();
                if let Some(range) = &source.record_range {
                    tx.execute(
                        "INSERT INTO recovery_locations(source_id,start,end) VALUES(?1,?2,?3)",
                        params![id, range.start as i64, range.end as i64],
                    )?;
                }
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

fn invalid_build() -> IndexError {
    IndexError::Store(IndexStoreError::Database(
        "invalid or stale recovery build cursor".into(),
    ))
}

use rusqlite::OptionalExtension;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_tlog_later_page_marker_recovers_committed_tool_fact() {
        use talos_core::message::{Message, MessageToolResult, ToolCall};
        let dir = tempfile::tempdir().expect("directory");
        let session =
            crate::DurableSession::open_or_create(dir.path(), "recovery").expect("session");
        let mut messages = vec![
            Message::Assistant {
                content: String::new(),
                reasoning: None,
                tool_calls: vec![ToolCall {
                    id: "call".into(),
                    name: "read".into(),
                    input: serde_json::json!({}),
                }],
            },
            Message::Tool {
                result: MessageToolResult {
                    tool_use_id: "call".into(),
                    content: "original tool fact".into(),
                    is_error: false,
                },
            },
        ];
        messages.extend((0..600).map(|_| Message::User {
            content: "padding ".repeat(600),
        }));
        let commit = session
            .finalize_turn_with_recovery(
                "turn",
                &messages,
                &crate::PersistencePolicy::default(),
                crate::TurnTranscriptOutcome::Success,
                &[crate::RecoveryProjection {
                    message_index: 1,
                    tool_call_id: "call".into(),
                    tool_name: "read".into(),
                    content: "original tool fact".into(),
                }],
            )
            .expect("atomic turn");
        let source = session.capture_recovery_source().expect("source");
        assert!(source.source_end() > crate::history_page::HISTORY_PAGE_BYTES as u64);
        let path = dir.path().join("index.db");
        let mut host = SessionIndex::new(&path).expect("index");
        host.init_recovery_schema().expect("schema");
        let budget = RecoveryWorkBudget::new(Duration::from_secs(10));
        let mut cursor = host
            .begin_recovery_build("scope", source.source_end(), &budget)
            .expect("begin");
        let mut tool = None;
        let mut pages = 0;
        while !cursor.complete() {
            let page = source
                .read_page(cursor.next_offset(), budget.deadline, || budget.stopped())
                .expect("bounded source page");
            for (i, entry) in page.entries.iter().enumerate() {
                if entry.id == commit.entry_ids[1] {
                    tool = Some(
                        source
                            .capture_record(page.record_ranges[i].clone(), budget.deadline, || {
                                budget.stopped()
                            })
                            .expect("tool source"),
                    );
                }
            }
            let sources: Vec<_> = page
                .entries
                .iter()
                .enumerate()
                .map(|(i, entry)| RecoveryIndexSource {
                    entry,
                    terminal: None,
                    ordinal: cursor.next_ordinal() + i as u64,
                    record_range: Some(page.record_ranges[i].clone()),
                })
                .collect();
            host.advance_recovery_build(
                "scope",
                &cursor,
                RecoveryBuildPage {
                    next_offset: page.next_offset,
                    scanned_entries: page.entries.len() as u64,
                    sources: &sources,
                },
                &budget,
            )
            .expect("page commit");
            cursor = host
                .recovery_build_cursor("scope", &budget)
                .expect("cursor")
                .expect("present");
            pages += 1;
            if pages == 1 {
                assert!(
                    host.recovery_terminal_candidate("scope", "turn", &budget)
                        .expect("lookup")
                        .is_none()
                );
            }
        }
        assert!(pages > 1);
        drop(host);
        let reader = SessionIndex::open_read_only(&path).expect("read-only reopen");
        let marker = reader
            .recovery_terminal_candidate("scope", "turn", &budget)
            .expect("lookup")
            .expect("later marker");
        let marker = source
            .capture_record(marker.record_range.expect("range"), budget.deadline, || {
                budget.stopped()
            })
            .expect("marker reread");
        let text = tool
            .expect("early tool")
            .recover_text(
                Some(&marker),
                crate::history_projection::RecoveryTextKind::Detail,
                budget.deadline,
                || budget.stopped(),
            )
            .expect("verified source and receipt")
            .expect("eligible");
        assert_eq!(text.text, "original tool fact");
    }

    #[test]
    fn later_page_terminal_locator_survives_reopen_and_duplicate_rolls_back_cursor() {
        let dir = tempfile::tempdir().expect("directory");
        let path = dir.path().join("index.db");
        let mut host = SessionIndex::new(&path).expect("host");
        host.init_recovery_schema().expect("schema");
        let budget = RecoveryWorkBudget::new(Duration::from_secs(2));
        let first = host
            .begin_recovery_build("scope", 300, &budget)
            .expect("begin");
        host.advance_recovery_build(
            "scope",
            &first,
            RecoveryBuildPage {
                next_offset: 100,
                scanned_entries: 1,
                sources: &[],
            },
            &budget,
        )
        .expect("first page");
        assert!(
            host.recovery_terminal_candidate("scope", "turn", &budget)
                .expect("lookup")
                .is_none()
        );
        let marker = SessionEntry {
            id: "terminal".into(),
            parent_id: None,
            timestamp: chrono::Utc::now(),
            role: "system".into(),
            content: crate::turn_outcome::encode_turn_transcript_outcome(
                &crate::TurnTranscriptOutcomeRecord::new(
                    "turn",
                    crate::TurnTranscriptOutcome::Success,
                ),
            )
            .expect("marker"),
            metadata: crate::SessionMetadata {
                turn_id: Some("turn".into()),
                ..Default::default()
            },
        };
        let prior = host
            .recovery_build_cursor("scope", &budget)
            .expect("cursor")
            .expect("present");
        host.advance_recovery_build(
            "scope",
            &prior,
            RecoveryBuildPage {
                next_offset: 200,
                scanned_entries: 1,
                sources: &[RecoveryIndexSource {
                    entry: &marker,
                    terminal: None,
                    ordinal: 1,
                    record_range: Some(100..200),
                }],
            },
            &budget,
        )
        .expect("terminal page");
        drop(host);
        let mut host = SessionIndex::new(&path).expect("reopen");
        let found = host
            .recovery_terminal_candidate("scope", "turn", &budget)
            .expect("lookup")
            .expect("locator");
        assert_eq!(found.record_range, Some(100..200));
        assert_eq!(found.entry_id, "terminal");
        assert!(
            host.recovery_terminal_candidate("sibling", "turn", &budget)
                .expect("scoped")
                .is_none()
        );
        let prior = host
            .recovery_build_cursor("scope", &budget)
            .expect("cursor")
            .expect("present");
        let mut duplicate = marker;
        duplicate.id = "duplicate".into();
        assert!(
            host.advance_recovery_build(
                "scope",
                &prior,
                RecoveryBuildPage {
                    next_offset: 300,
                    scanned_entries: 1,
                    sources: &[RecoveryIndexSource {
                        entry: &duplicate,
                        terminal: None,
                        ordinal: 2,
                        record_range: Some(200..300)
                    }]
                },
                &budget
            )
            .is_err()
        );
        assert_eq!(
            host.recovery_build_cursor("scope", &budget)
                .expect("cursor"),
            Some(prior)
        );
        assert_eq!(
            host.recovery_terminal_candidate("scope", "turn", &budget)
                .expect("locator"),
            Some(found)
        );
    }

    #[test]
    fn model_query_connection_never_creates_or_repairs_index() {
        let dir = tempfile::tempdir().expect("directory");
        let path = dir.path().join("index.db");
        assert!(SessionIndex::open_read_only(&path).is_err());
        assert!(!path.exists());
        let index = SessionIndex::new(&path).expect("host creates index");
        index.init_recovery_schema().expect("host schema");
        let budget = RecoveryWorkBudget::new(Duration::from_secs(2));
        let reader = SessionIndex::open_read_only(&path).expect("reader");
        assert!(
            reader
                .recovery_candidates("scope", "", None, 20, &budget)
                .expect("read query")
                .is_empty()
        );
        assert!(reader.begin_recovery_build("scope", 10, &budget).is_err());
        assert!(
            reader
                .conn
                .execute_batch("CREATE TABLE forbidden_write(value TEXT)")
                .is_err()
        );
        let no_schema = dir.path().join("empty.db");
        let _host = SessionIndex::new(&no_schema).expect("empty host database");
        let empty = SessionIndex::open_read_only(&no_schema).expect("read empty database");
        assert!(
            empty
                .recovery_candidates("scope", "", None, 20, &budget)
                .is_err()
        );
        let count: i64 = empty
            .conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE name='recovery_sources'",
                [],
                |row| row.get(0),
            )
            .expect("schema query");
        assert_eq!(count, 0, "query never repairs missing schema");
    }

    #[test]
    fn read_only_recovery_queries_observe_host_pages_and_refuse_incomplete_or_corrupt_cache() {
        let dir = tempfile::tempdir().expect("directory");
        let path = dir.path().join("index.db");
        let mut host = SessionIndex::new(&path).expect("host");
        host.init_recovery_schema().expect("schema");
        let budget = RecoveryWorkBudget::new(Duration::from_secs(2));
        let prior = host
            .begin_recovery_build("authorized", 100, &budget)
            .expect("begin");
        let reader = SessionIndex::open_read_only(&path).expect("reader");
        assert!(
            reader
                .recovery_candidates("authorized", "fact", None, 20, &budget)
                .is_err()
        );
        let entry = SessionEntry {
            id: "original".into(),
            parent_id: None,
            timestamp: chrono::Utc::now(),
            role: "user".into(),
            content: "retained fact".into(),
            metadata: Default::default(),
        };
        host.advance_recovery_build(
            "authorized",
            &prior,
            RecoveryBuildPage {
                next_offset: 100,
                scanned_entries: 1,
                sources: &[RecoveryIndexSource {
                    entry: &entry,
                    terminal: None,
                    ordinal: 0,
                    record_range: Some(0..100),
                }],
            },
            &budget,
        )
        .expect("host finishes build");
        let hits = reader
            .recovery_candidates("authorized", "fact", None, 20, &budget)
            .expect("read completed cache");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].entry_id, "original");
        assert!(
            reader
                .recovery_candidates("sibling", "fact", None, 20, &budget)
                .expect("other scope")
                .is_empty()
        );
        budget.cancel();
        assert!(
            reader
                .recovery_candidates("authorized", "fact", None, 20, &budget)
                .is_err()
        );
        let corrupt = dir.path().join("corrupt.db");
        let bytes = b"not a sqlite database";
        std::fs::write(&corrupt, bytes).expect("corrupt fixture");
        if let Ok(reader) = SessionIndex::open_read_only(&corrupt) {
            assert!(
                reader
                    .recovery_candidates(
                        "authorized",
                        "fact",
                        None,
                        20,
                        &RecoveryWorkBudget::new(Duration::from_secs(2))
                    )
                    .is_err()
            );
        }
        assert_eq!(
            std::fs::read(&corrupt).expect("unchanged corrupt file"),
            bytes
        );
    }

    #[test]
    fn build_cursor_resumes_across_reopen_and_commits_with_page_atomically() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("index.db");
        let mut index = SessionIndex::new(&path).expect("index");
        index.init_recovery_schema().expect("schema");
        let budget = RecoveryWorkBudget::new(Duration::from_secs(10));
        let page_bytes = crate::history_page::HISTORY_PAGE_BYTES as u64;
        let first = index
            .begin_recovery_build("scope", page_bytes * 2, &budget)
            .expect("start");
        let entry = SessionEntry {
            id: "first".into(),
            parent_id: None,
            timestamp: chrono::Utc::now(),
            role: "user".into(),
            content: "recoverable fact".into(),
            metadata: Default::default(),
        };
        let sources = [RecoveryIndexSource {
            record_range: None,
            entry: &entry,
            terminal: None,
            ordinal: 0,
        }];
        index
            .advance_recovery_build(
                "scope",
                &first,
                RecoveryBuildPage {
                    next_offset: page_bytes,
                    scanned_entries: 8192,
                    sources: &sources,
                },
                &budget,
            )
            .expect("first page");
        drop(index);
        let mut index = SessionIndex::new(&path).expect("reopen");
        let resumed = index
            .recovery_build_cursor("scope", &budget)
            .expect("cursor")
            .expect("present");
        assert_eq!(resumed.next_offset(), page_bytes);
        assert_eq!(resumed.next_ordinal(), 8192);
        assert!(!resumed.complete());
        assert!(
            index
                .recovery_candidates("scope", "recoverable", None, 20, &budget)
                .is_err()
        );
        assert!(
            index
                .begin_recovery_build("scope", page_bytes * 2, &budget)
                .is_err()
        );
        assert!(
            index
                .advance_recovery_build(
                    "scope",
                    &first,
                    RecoveryBuildPage {
                        next_offset: page_bytes,
                        scanned_entries: 8192,
                        sources: &sources,
                    },
                    &budget
                )
                .is_err()
        );
        // A repeated source ID must roll back even the cursor UPDATE executed
        // earlier in the transaction. It cannot erase the prior eligible fact.
        let duplicate = [RecoveryIndexSource {
            record_range: None,
            entry: &entry,
            terminal: None,
            ordinal: 8192,
        }];
        assert!(
            index
                .advance_recovery_build(
                    "scope",
                    &resumed,
                    RecoveryBuildPage {
                        next_offset: page_bytes * 2,
                        scanned_entries: 1,
                        sources: &duplicate,
                    },
                    &budget
                )
                .is_err()
        );
        assert_eq!(
            index
                .recovery_build_cursor("scope", &budget)
                .expect("cursor"),
            Some(resumed.clone())
        );
        assert!(index.cache_recovery_page("scope", &sources).is_err());
        index
            .advance_recovery_build(
                "scope",
                &resumed,
                RecoveryBuildPage {
                    next_offset: page_bytes * 2,
                    scanned_entries: 8192,
                    sources: &[],
                },
                &budget,
            )
            .expect("second page including ineligible entries");
        let done = index
            .recovery_build_cursor("scope", &budget)
            .expect("cursor")
            .expect("present");
        assert!(done.complete());
        assert_eq!(done.next_ordinal(), 16384);
        assert_eq!(
            index
                .recovery_candidates("scope", "recoverable", None, 20, &budget)
                .expect("candidates")[0]
                .entry_id,
            "first"
        );
    }

    #[test]
    fn actual_source_pages_rebuild_beyond_single_page_limits() {
        use std::io::Write;
        for content in ["fact".to_string(), "fact ".repeat(70)] {
            let dir = tempfile::tempdir().expect("tempdir");
            let path = dir.path().join("source.jsonl");
            let mut file = std::io::BufWriter::new(std::fs::File::create(&path).expect("source"));
            for n in 0..10_000 {
                let entry = SessionEntry {
                    id: n.to_string(),
                    parent_id: None,
                    timestamp: chrono::Utc::now(),
                    role: "user".into(),
                    content: content.clone(),
                    metadata: Default::default(),
                };
                serde_json::to_writer(&mut file, &entry).expect("record");
                writeln!(file).expect("newline");
            }
            file.flush().expect("flush");
            let end = std::fs::metadata(&path).expect("metadata").len();
            let mut index = SessionIndex::new(&dir.path().join("index.db")).expect("index");
            index.init_recovery_schema().expect("schema");
            let budget = RecoveryWorkBudget::new(Duration::from_secs(10));
            let mut cursor = index
                .begin_recovery_build("scope", end, &budget)
                .expect("start");
            let mut pages = 0;
            while !cursor.complete() {
                let page = crate::history_page::read_history_page(
                    &path,
                    crate::history_page::HistoryEncoding::Jsonl,
                    cursor.next_offset(),
                    end,
                    budget.deadline,
                    || budget.stopped(),
                )
                .expect("bounded source read");
                let sources: Vec<_> = page
                    .entries
                    .iter()
                    .enumerate()
                    .map(|(n, entry)| RecoveryIndexSource {
                        record_range: Some(page.record_ranges[n].clone()),
                        entry,
                        terminal: None,
                        ordinal: cursor.next_ordinal() + n as u64,
                    })
                    .collect();
                index
                    .advance_recovery_build(
                        "scope",
                        &cursor,
                        RecoveryBuildPage {
                            next_offset: page.next_offset,
                            scanned_entries: page.entries.len() as u64,
                            sources: &sources,
                        },
                        &budget,
                    )
                    .expect("atomic page");
                cursor = index
                    .recovery_build_cursor("scope", &budget)
                    .expect("progress")
                    .expect("cursor");
                pages += 1;
            }
            assert!(pages > 1);
            assert_eq!(cursor.next_ordinal(), 10_000);
            let tail = index
                .recovery_candidates("scope", "fact", Some(9998), 20, &budget)
                .expect("last fact");
            assert_eq!(tail.len(), 1);
            assert_eq!(tail[0].entry_id, "9999");
            let recovered = crate::history_page::read_history_record(
                &path,
                crate::history_page::HistoryEncoding::Jsonl,
                tail[0].record_range.clone().expect("original locator"),
                budget.deadline,
                || budget.stopped(),
            )
            .expect("bounded authoritative reread");
            assert_eq!(recovered.id, tail[0].entry_id);
            assert_eq!(recovered.content, content);
            let detail = crate::history_projection::recovery_response_text(
                &recovered,
                None,
                crate::history_projection::RecoveryTextKind::Detail,
            )
            .expect("eligible reread detail");
            assert_eq!(detail.text, content);
            assert!(!detail.truncated);
            let snippet = crate::history_projection::recovery_response_text(
                &recovered,
                None,
                crate::history_projection::RecoveryTextKind::Snippet,
            )
            .expect("bounded snippet");
            assert!(snippet.text.len() <= 512);
            assert_eq!(snippet.truncated, content.len() > 512);
        }
    }

    #[test]
    fn locator_replacement_clears_old_span_and_corrupt_cache_fails_closed() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut index = SessionIndex::new(&dir.path().join("index.db")).expect("index");
        index.init_recovery_schema().expect("schema");
        let entry = SessionEntry {
            id: "entry".into(),
            parent_id: None,
            timestamp: chrono::Utc::now(),
            role: "user".into(),
            content: "fact".into(),
            metadata: Default::default(),
        };
        let budget = RecoveryWorkBudget::new(Duration::from_secs(2));
        let mut source = RecoveryIndexSource {
            entry: &entry,
            terminal: None,
            ordinal: 0,
            record_range: Some(10..20),
        };
        index
            .cache_recovery_page("scope", std::slice::from_ref(&source))
            .expect("cache");
        assert_eq!(
            index
                .recovery_candidates("scope", "fact", None, 1, &budget)
                .expect("candidate")[0]
                .record_range,
            Some(10..20)
        );
        source.record_range = None;
        index
            .cache_recovery_page("scope", std::slice::from_ref(&source))
            .expect("replace");
        assert!(
            index
                .recovery_candidates("scope", "fact", None, 1, &budget)
                .expect("candidate")[0]
                .record_range
                .is_none()
        );
        source.record_range = Some(10..20);
        index
            .cache_recovery_page("scope", &[source])
            .expect("restore locator");
        index
            .conn
            .execute("UPDATE recovery_locations SET start=-1", [])
            .expect("corrupt cache");
        assert!(
            index
                .recovery_candidates("scope", "fact", None, 1, &budget)
                .is_err()
        );
    }

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
                        record_range: None,
                        entry: &first,
                        terminal: None,
                        ordinal: 1,
                    },
                    RecoveryIndexSource {
                        record_range: None,
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
                    record_range: None,
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
                        record_range: None,
                        entry: &public,
                        terminal: None,
                        ordinal: 0,
                    },
                    RecoveryIndexSource {
                        record_range: None,
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
                        record_range: None,
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
                    record_range: None,
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
