use crate::entities::extract_entities;
use crate::{
    EntityKind, EvidenceLink, MemoryItem, MemoryKind, MemoryStatus, MemoryStoreError,
    RetentionCandidate, RetentionPolicy, RetrievalResult,
};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::Path;

/// SQLite-backed store for semantic and procedural memory.
///
/// Provides ADD-only writes, FTS5-based retrieval with multi-signal scoring,
/// and evidence provenance tracking.
pub struct MemoryStore {
    pub(crate) conn: Connection,
}

impl MemoryStore {
    /// Open or create a memory store at the given file path.
    ///
    /// Creates parent directories if they do not exist. Runs schema migration
    /// automatically.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, MemoryStoreError> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        configure_connection(&conn)?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    /// Open an in-memory store for testing.
    pub fn open_memory() -> Result<Self, MemoryStoreError> {
        let conn = Connection::open_in_memory()?;
        configure_connection(&conn)?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    /// Create the database schema if it does not exist.
    fn migrate(&self) -> Result<(), MemoryStoreError> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS schema_version (version INTEGER PRIMARY KEY);

            CREATE TABLE IF NOT EXISTS memory_items (
                id TEXT PRIMARY KEY,
                kind TEXT NOT NULL,
                key TEXT NOT NULL,
                content TEXT NOT NULL,
                content_hash TEXT NOT NULL,
                confidence REAL NOT NULL,
                created_at TEXT NOT NULL,
                last_reinforced TEXT NOT NULL,
                last_accessed TEXT,
                contradiction_ref TEXT
            );

            CREATE UNIQUE INDEX IF NOT EXISTS idx_memory_items_content_hash
                ON memory_items(content_hash);

            CREATE TABLE IF NOT EXISTS evidence_links (
                id TEXT PRIMARY KEY,
                memory_id TEXT NOT NULL,
                source_type TEXT NOT NULL,
                source_ref TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (memory_id) REFERENCES memory_items(id)
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS memory_fts USING fts5(
                memory_id UNINDEXED, content, tokenize='unicode61'
            );

            CREATE TABLE IF NOT EXISTS entities (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                kind TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS memory_entities (
                memory_id TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                PRIMARY KEY (memory_id, entity_id),
                FOREIGN KEY (memory_id) REFERENCES memory_items(id),
                FOREIGN KEY (entity_id) REFERENCES entities(id)
            );
            "#,
        )?;

        // Initialize schema_version if empty (new database).
        let version_count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM schema_version", [], |row| row.get(0))?;
        if version_count == 0 {
            let _ = self
                .conn
                .execute("INSERT INTO schema_version (version) VALUES (2)", []);
        } else {
            // Migrate from version 1 to 2: entity tables are created above with
            // IF NOT EXISTS, so they are idempotent. Just bump the version.
            let current_version: i64 =
                self.conn
                    .query_row("SELECT version FROM schema_version", [], |row| row.get(0))?;
            if current_version < 2 {
                let _ = self
                    .conn
                    .execute("UPDATE schema_version SET version = 2", []);
            }
        }

        Ok(())
    }

    /// Insert a memory item.
    ///
    /// ADD-only: exact duplicates (same `key` + `content` hash) are silently ignored.
    /// Conflicting entries with the same `key` but different content are preserved
    /// as separate rows (ADD-only semantics per ADR-016).
    ///
    /// Returns `true` if the item was inserted, `false` if it was an exact duplicate.
    pub fn insert(&mut self, item: MemoryItem) -> Result<bool, MemoryStoreError> {
        let content_hash = compute_content_hash(&item.key, &item.content);
        let kind_str = match item.kind {
            MemoryKind::Semantic => "semantic",
            MemoryKind::Procedural => "procedural",
        };

        let tx = self.conn.transaction()?;

        let changes = tx.execute(
            "INSERT OR IGNORE INTO memory_items \
             (id, kind, key, content, content_hash, confidence, created_at, last_reinforced, last_accessed, contradiction_ref) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                item.id,
                kind_str,
                item.key,
                item.content,
                content_hash,
                item.confidence,
                item.created_at.to_rfc3339(),
                item.last_reinforced.to_rfc3339(),
                item.last_accessed.as_ref().map(|dt| dt.to_rfc3339()),
                item.contradiction_ref,
            ],
        )?;

        if changes > 0 {
            tx.execute(
                "INSERT INTO memory_fts (memory_id, content) VALUES (?1, ?2)",
                params![item.id, item.content],
            )?;

            let entities = extract_entities(&item.content);
            for (name, kind) in entities {
                let kind_str = match kind {
                    EntityKind::File => "file",
                    EntityKind::Url => "url",
                    EntityKind::Code => "code",
                    EntityKind::Concept => "concept",
                };
                let entity_id = format!("{kind_str}:{name}");
                let _ = tx.execute(
                    "INSERT OR IGNORE INTO entities (id, name, kind, created_at) VALUES (?1, ?2, ?3, ?4)",
                    params![entity_id, name, kind_str, Utc::now().to_rfc3339()],
                );
                let _ = tx.execute(
                    "INSERT OR IGNORE INTO memory_entities (memory_id, entity_id) VALUES (?1, ?2)",
                    params![item.id, entity_id],
                );
            }
        }

        tx.commit()?;

        Ok(changes > 0)
    }

    /// Insert an evidence link for a memory item.
    pub fn insert_evidence(&self, link: EvidenceLink) -> Result<(), MemoryStoreError> {
        self.conn.execute(
            "INSERT INTO evidence_links (id, memory_id, source_type, source_ref, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                link.id,
                link.memory_id,
                link.source_type,
                link.source_ref,
                link.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Retrieve memory items matching the query, scored by FTS5 relevance, recency, and evidence.
    ///
    /// # Scoring Formula
    ///
    /// - `fts_score = |bm25| / (1 + |bm25|)` — normalized FTS5 relevance (higher = more relevant)
    /// - `recency = exp(-days_since_last_reinforced / 30.0)` — exponential decay over 30 days
    /// - `evidence_score = confidence × ln(1 + evidence_count)` — logarithmic evidence strength
    /// - `final_score = fts_score × 1.0 + recency × 0.3 + evidence_score × 0.5`
    ///
    /// Results are sorted by `final_score` descending and truncated to `limit`.
    pub fn retrieve(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<RetrievalResult>, MemoryStoreError> {
        if query.trim().is_empty() || limit == 0 {
            return Ok(Vec::new());
        }

        // Fetch more candidates from FTS5 to allow scoring to reorder.
        let candidate_limit = (limit.max(5) * 3) as i64;

        let fts_query = escape_fts_query(query);

        let mut stmt = self.conn.prepare(
            "SELECT memory_id, rank FROM memory_fts \
             WHERE memory_fts MATCH ?1 \
             ORDER BY rank ASC \
             LIMIT ?2",
        )?;

        let candidates = stmt
            .query_map(params![fts_query, candidate_limit], |row| {
                let memory_id: String = row.get(0)?;
                let rank: f64 = row.get(1)?;
                Ok((memory_id, rank))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        let query_entities: HashSet<String> = extract_entities(query)
            .into_iter()
            .map(|(name, kind)| {
                let kind_str = match kind {
                    EntityKind::File => "file",
                    EntityKind::Url => "url",
                    EntityKind::Code => "code",
                    EntityKind::Concept => "concept",
                };
                format!("{kind_str}:{name}")
            })
            .collect();

        let mut results = Vec::new();
        let now = Utc::now();

        for (memory_id, bm25_rank) in candidates {
            let item = match self.get(&memory_id)? {
                Some(item) => item,
                None => continue,
            };

            // FTS5 bm25: more negative = more relevant. Normalize to [0, 1).
            let fts_score = bm25_rank.abs() / (1.0 + bm25_rank.abs());

            let seconds_since_reinforced = (now - item.last_reinforced).num_seconds().max(0) as f64;
            let days_since_reinforced = seconds_since_reinforced / 86400.0;
            let recency = (-days_since_reinforced / 30.0).exp();

            let evidence_count: i64 = self.conn.query_row(
                "SELECT COUNT(*) FROM evidence_links WHERE memory_id = ?1",
                params![memory_id],
                |row| row.get(0),
            )?;
            let evidence_score = item.confidence * (1.0 + evidence_count as f64).ln();

            let memory_entity_ids: HashSet<String> = self
                .conn
                .prepare("SELECT entity_id FROM memory_entities WHERE memory_id = ?1")
                .ok()
                .and_then(|mut stmt| {
                    stmt.query_map(params![memory_id], |row| row.get::<_, String>(0))
                        .ok()
                        .map(|rows| rows.filter_map(|r| r.ok()).collect::<HashSet<String>>())
                })
                .unwrap_or_default();

            let overlap = query_entities.intersection(&memory_entity_ids).count();
            let entity_score = overlap as f64 * 0.5;

            let final_score = fts_score * 1.0 + recency * 0.3 + evidence_score * 0.5 + entity_score;

            let evidence = self.load_evidence(&memory_id)?;

            // Best-effort update of last_accessed (ignore errors — ranking signal only).
            let _ = self.conn.execute(
                "UPDATE memory_items SET last_accessed = ?1 WHERE id = ?2",
                params![now.to_rfc3339(), memory_id],
            );

            let score_breakdown = format!(
                "fts={:.3}, recency={:.3}, evidence={:.3}, entity={:.3}",
                fts_score, recency, evidence_score, entity_score
            );

            results.push(RetrievalResult {
                item,
                evidence,
                score: final_score,
                score_breakdown,
            });
        }

        // Sort by final_score descending.
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results.truncate(limit);

        Ok(results)
    }

    /// Get a memory item by ID.
    pub fn get(&self, id: &str) -> Result<Option<MemoryItem>, MemoryStoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, kind, key, content, confidence, created_at, last_reinforced, \
             last_accessed, contradiction_ref \
             FROM memory_items WHERE id = ?1",
        )?;

        let row_data = stmt
            .query_row(params![id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, f64>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                ))
            })
            .optional()?;

        match row_data {
            Some((
                id,
                kind_str,
                key,
                content,
                confidence,
                created_at_str,
                last_reinforced_str,
                last_accessed_str,
                contradiction_ref,
            )) => {
                let kind = match kind_str.as_str() {
                    "procedural" => MemoryKind::Procedural,
                    _ => MemoryKind::Semantic,
                };
                Ok(Some(MemoryItem {
                    id,
                    kind,
                    key,
                    content,
                    confidence,
                    created_at: parse_datetime(&created_at_str)?,
                    last_reinforced: parse_datetime(&last_reinforced_str)?,
                    last_accessed: last_accessed_str
                        .as_deref()
                        .map(parse_datetime)
                        .transpose()?,
                    contradiction_ref,
                }))
            }
            None => Ok(None),
        }
    }

    /// Count total memory items.
    pub fn count(&self) -> Result<usize, MemoryStoreError> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM memory_items", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    /// Checkpoint the SQLite write-ahead log, truncating it when possible.
    ///
    /// This is an explicit maintenance operation for storage lifecycle workflows;
    /// normal memory reads and writes do not invoke it.
    pub fn checkpoint_truncate(&self) -> Result<(), MemoryStoreError> {
        self.conn
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(|e| MemoryStoreError::Maintenance(e.to_string()))
    }

    /// Rebuild the SQLite database file to reclaim free pages.
    ///
    /// This should only be called by explicit maintenance commands, never inside
    /// the agent turn loop.
    pub fn vacuum(&self) -> Result<(), MemoryStoreError> {
        self.conn
            .execute_batch("VACUUM;")
            .map_err(|e| MemoryStoreError::Maintenance(e.to_string()))
    }

    /// Report memory store status without exposing any content.
    pub fn memory_status(&self) -> Result<MemoryStatus, MemoryStoreError> {
        let total_items: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM memory_items", [], |row| row.get(0))?;

        let semantic_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM memory_items WHERE kind = 'semantic'",
            [],
            |row| row.get(0),
        )?;

        let procedural_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM memory_items WHERE kind = 'procedural'",
            [],
            |row| row.get(0),
        )?;

        let evidence_count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM evidence_links", [], |row| row.get(0))?;

        let entity_count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM entities", [], |row| row.get(0))?;

        let (db_path, db_size_bytes) = self.db_file_info()?;

        Ok(MemoryStatus {
            total_items: total_items as usize,
            semantic_count: semantic_count as usize,
            procedural_count: procedural_count as usize,
            evidence_count: evidence_count as usize,
            entity_count: entity_count as usize,
            db_path,
            db_size_bytes,
        })
    }

    /// Report memory items that match retention criteria.
    /// This is DRY-RUN ONLY — no items are deleted. ADD-only semantics are preserved.
    pub fn retention_candidates(
        &self,
        policy: &RetentionPolicy,
    ) -> Result<Vec<RetentionCandidate>, MemoryStoreError> {
        let mut query = String::from(
            "SELECT id, kind, key, confidence, last_reinforced \
             FROM memory_items WHERE 1=1",
        );
        let mut params_vec: Vec<rusqlite::types::Value> = Vec::new();

        if let Some(min_conf) = policy.min_confidence {
            query.push_str(" AND confidence < ?");
            params_vec.push(rusqlite::types::Value::Real(min_conf));
        }

        if let Some(max_age_days) = policy.max_age_days {
            query.push_str(" AND last_reinforced < datetime('now', ?)");
            params_vec.push(rusqlite::types::Value::Text(format!(
                "-{} days",
                max_age_days
            )));
        }

        if policy.unreinforced_only {
            query.push_str(" AND id NOT IN (SELECT DISTINCT memory_id FROM evidence_links)");
        }

        let mut stmt = self.conn.prepare(&query)?;

        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            params_vec.iter().map(|v| v as _).collect();

        let rows = stmt.query_map(rusqlite::params_from_iter(param_refs.iter()), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, String>(4)?,
            ))
        })?;

        let now = Utc::now();
        let mut candidates = Vec::new();

        for row in rows {
            let (id, kind, key, confidence, last_reinforced_str) = row?;
            let last_reinforced = parse_datetime(&last_reinforced_str)?;

            let evidence_count: i64 = self.conn.query_row(
                "SELECT COUNT(*) FROM evidence_links WHERE memory_id = ?1",
                params![&id],
                |row| row.get(0),
            )?;

            let key_preview = if key.len() > 30 {
                format!("{}...", &key[..27])
            } else {
                key.clone()
            };

            let mut reasons = Vec::new();
            if policy.min_confidence.is_some_and(|m| confidence < m) {
                reasons.push(format!("confidence {:.2} below threshold", confidence));
            }
            if policy.max_age_days.is_some() {
                let age_days = (now - last_reinforced).num_days();
                reasons.push(format!("age {} days", age_days));
            }
            if policy.unreinforced_only && evidence_count == 0 {
                reasons.push("no evidence links".to_string());
            }

            let age_days = (now - last_reinforced).num_days();

            candidates.push(RetentionCandidate {
                id,
                kind,
                key_preview,
                confidence,
                last_reinforced,
                age_days,
                evidence_count: evidence_count as usize,
                reason: reasons.join("; "),
            });
        }

        Ok(candidates)
    }

    fn db_file_info(&self) -> Result<(Option<String>, u64), MemoryStoreError> {
        let mut stmt = self.conn.prepare("PRAGMA database_list")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;

        for row in rows {
            let (_seq, _name, path) = row?;
            if path.is_empty() || path == ":memory:" {
                continue;
            }
            if let Ok(meta) = std::fs::metadata(&path) {
                return Ok((Some(path), meta.len()));
            }
        }

        Ok((None, 0))
    }

    /// Load evidence links for a memory item.
    fn load_evidence(&self, memory_id: &str) -> Result<Vec<EvidenceLink>, MemoryStoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, memory_id, source_type, source_ref, created_at \
             FROM evidence_links WHERE memory_id = ?1 ORDER BY created_at DESC",
        )?;

        let raw_links = stmt
            .query_map(params![memory_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        raw_links
            .into_iter()
            .map(|(id, memory_id, source_type, source_ref, created_at_str)| {
                Ok(EvidenceLink {
                    id,
                    memory_id,
                    source_type,
                    source_ref,
                    created_at: parse_datetime(&created_at_str)?,
                })
            })
            .collect()
    }
}

fn configure_connection(conn: &Connection) -> Result<(), MemoryStoreError> {
    conn.execute_batch(
        "PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;",
    )?;
    Ok(())
}

/// Compute SHA-256 hash of `key|content` for exact-duplicate detection.
fn compute_content_hash(key: &str, content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hasher.update(b"|");
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

/// Parse an RFC 3339 datetime string into `DateTime<Utc>`.
fn parse_datetime(s: &str) -> Result<DateTime<Utc>, MemoryStoreError> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| MemoryStoreError::InvalidTimestamp(e.to_string()))
}

/// Escape a user query for safe use in FTS5 MATCH.
///
/// Replaces FTS5-special characters with spaces so tokenization works
/// without syntax errors. Entity extraction still runs on the original query.
fn escape_fts_query(query: &str) -> String {
    query.replace(['/', '.', '"', '-'], " ")
}
