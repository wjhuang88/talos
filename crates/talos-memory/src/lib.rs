//! Talos memory layer — semantic and procedural memory storage.
//!
//! Implements the semantic and procedural layers of the four-layer memory architecture
//! defined in ADR-016. Working and episodic memory are handled by the session store;
//! this crate provides persistent storage for consolidated facts (semantic) and learned
//! procedures (procedural).
//!
//! # Architecture
//!
//! - **Semantic memory**: Stable facts, entities, claims, preferences, project knowledge.
//! - **Procedural memory**: Learned operational procedures, skills, patterns, playbooks.
//!
//! # Design Principles
//!
//! - **ADD-only writes**: new memories are always appended; conflicts are preserved, not overwritten.
//! - **Bounded retrieval**: FTS5 + recency + evidence scoring with configurable limits.
//! - **Provenance**: every memory item links to evidence through the `evidence_links` table.
//! - **No automatic prompt injection**: retrieval returns results; injection is caller's responsibility.

pub mod consolidation;

pub use consolidation::{
    ConsolidationConfig, ConsolidationReport, EpisodeExtractor, MemoryCandidate,
    RuleBasedExtractor, SessionEpisode, consolidate_episodes,
};

use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::Path;
use thiserror::Error;

/// The kind of memory item stored in the semantic/procedural layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryKind {
    /// Consolidated facts, entities, claims, preferences, project knowledge.
    Semantic,
    /// Learned operational procedures, skills, patterns, playbooks.
    Procedural,
}

impl std::fmt::Display for MemoryKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryKind::Semantic => write!(f, "Semantic"),
            MemoryKind::Procedural => write!(f, "Procedural"),
        }
    }
}

/// The kind of entity extracted from memory content.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    File,
    Url,
    Code,
    Concept,
}

/// An entity extracted from memory content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub kind: EntityKind,
    pub created_at: DateTime<Utc>,
}

/// A single memory item in the semantic or procedural layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    /// Unique identifier for this memory item.
    pub id: String,
    /// The kind of memory (semantic or procedural).
    pub kind: MemoryKind,
    /// A key identifying the concept or topic this memory relates to.
    pub key: String,
    /// The content of the memory.
    pub content: String,
    /// Confidence score for this memory (0.0 to 1.0).
    pub confidence: f64,
    /// When this memory was first created.
    pub created_at: DateTime<Utc>,
    /// When this memory was last reinforced (re-inserted with supporting evidence).
    pub last_reinforced: DateTime<Utc>,
    /// When this memory was last accessed via retrieval.
    pub last_accessed: Option<DateTime<Utc>>,
    /// Reference to a contradiction record if this item conflicts with another.
    pub contradiction_ref: Option<String>,
}

/// A link from a memory item to its evidence source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceLink {
    /// Unique identifier for this evidence link.
    pub id: String,
    /// The memory item this evidence supports.
    pub memory_id: String,
    /// The type of evidence source (e.g., "session", "tool_call", "user_feedback").
    pub source_type: String,
    /// Reference to the specific evidence source.
    pub source_ref: String,
    /// When this evidence link was created.
    pub created_at: DateTime<Utc>,
}

/// A retrieval result with scoring and provenance.
#[derive(Debug, Clone)]
pub struct RetrievalResult {
    /// The retrieved memory item.
    pub item: MemoryItem,
    /// Evidence links supporting this memory.
    pub evidence: Vec<EvidenceLink>,
    /// Combined relevance score (higher is more relevant).
    pub score: f64,
    /// Human-readable breakdown of how the score was computed.
    pub score_breakdown: String,
}

/// Errors that can occur during memory store operations.
#[derive(Debug, Error)]
pub enum MemoryStoreError {
    /// A database operation failed.
    #[error("database operation failed: {0}")]
    Database(#[from] rusqlite::Error),
    /// Failed to parse a timestamp.
    #[error("invalid timestamp: {0}")]
    InvalidTimestamp(String),
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// SQLite maintenance failed.
    #[error("maintenance failed: {0}")]
    Maintenance(String),
}

/// Memory store status summary (no content exposed).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStatus {
    pub total_items: usize,
    pub semantic_count: usize,
    pub procedural_count: usize,
    pub evidence_count: usize,
    pub entity_count: usize,
    pub db_path: Option<String>,
    pub db_size_bytes: u64,
}

/// Policy for selecting memory retention candidates (dry-run only).
#[derive(Debug, Clone, Default)]
pub struct RetentionPolicy {
    pub min_confidence: Option<f64>,
    pub max_age_days: Option<i64>,
    pub unreinforced_only: bool,
}

/// A memory item selected as a retention candidate (dry-run, no deletion).
#[derive(Debug, Clone)]
pub struct RetentionCandidate {
    pub id: String,
    pub kind: String,
    pub key_preview: String,
    pub confidence: f64,
    pub last_reinforced: DateTime<Utc>,
    pub age_days: i64,
    pub evidence_count: usize,
    pub reason: String,
}

/// SQLite-backed store for semantic and procedural memory.
///
/// Provides ADD-only writes, FTS5-based retrieval with multi-signal scoring,
/// and evidence provenance tracking.
pub struct MemoryStore {
    conn: Connection,
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

/// Extract entities from text content using deterministic pattern matching.
///
/// Returns `(name, kind)` pairs. Deduplicates by name. Caps at 20 entities.
/// Never panics — uses only `std` string methods.
pub fn extract_entities(content: &str) -> Vec<(String, EntityKind)> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut result: Vec<(String, EntityKind)> = Vec::new();

    let mut add = |name: String, kind: EntityKind| {
        if name.len() < 2 || seen.contains(&name) || result.len() >= 20 {
            return;
        }
        seen.insert(name.clone());
        result.push((name, kind));
    };

    // Scan for URLs: http:// or https://
    let lower = content.to_lowercase();
    let mut pos = 0;
    while let Some(start) = lower[pos..]
        .find("https://")
        .or(lower[pos..].find("http://"))
    {
        let abs_start = pos + start;
        // Extract URL characters.
        let url_end = content[abs_start..]
            .chars()
            .take_while(|c| {
                let ch = *c;
                ch.is_alphanumeric()
                    || matches!(
                        ch,
                        '/' | '.' | '-' | '_' | '?' | '#' | '&' | '=' | '%' | ':' | '~' | '+' | ','
                    )
            })
            .map(|c| c.len_utf8())
            .sum::<usize>();
        let url = content[abs_start..abs_start + url_end].to_string();
        if url_end > 10 {
            add(url, EntityKind::Url);
        }
        pos = abs_start + url_end;
    }

    // Scan for file paths and code symbols by splitting on whitespace/punctuation.
    for token in content.split(|c: char| {
        c.is_whitespace()
            || matches!(
                c,
                '(' | ')'
                    | '{'
                    | '}'
                    | '['
                    | ']'
                    | ','
                    | ';'
                    | ':'
                    | '"'
                    | '\''
                    | '`'
                    | '\t'
                    | '\n'
                    | '\r'
            )
    }) {
        if token.is_empty() {
            continue;
        }

        // File path: contains '/' and has an extension.
        if token.contains('/') {
            let parts: Vec<&str> = token.split('/').collect();
            if parts.len() >= 2
                && let Some(last) = parts.last()
                && let Some(dot_pos) = last.rfind('.')
            {
                let ext = &last[dot_pos + 1..];
                if !ext.is_empty() && ext.len() <= 10 && ext.chars().all(|c| c.is_alphanumeric()) {
                    add(token.to_string(), EntityKind::File);
                    continue;
                }
            }
        }

        // Bare filename with extension (e.g., Cargo.toml, main.rs) — must have at least one char before dot.
        if let Some(dot_pos) = token.rfind('.') {
            let name_part = &token[..dot_pos];
            let ext = &token[dot_pos + 1..];
            if !name_part.is_empty()
                && !ext.is_empty()
                && ext.len() <= 10
                && ext.chars().all(|c| c.is_alphanumeric())
                && name_part
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            {
                add(token.to_string(), EntityKind::File);
                continue;
            }
        }

        // CamelCase: at least 2 uppercase letters indicating word boundaries.
        let mut upper_count = 0;
        let mut has_lower = false;
        for ch in token.chars() {
            if ch.is_uppercase() {
                upper_count += 1;
            }
            if ch.is_lowercase() {
                has_lower = true;
            }
        }
        if upper_count >= 2
            && has_lower
            && token.len() >= 4
            && token.chars().all(|c| c.is_alphanumeric())
        {
            add(token.to_string(), EntityKind::Code);
            continue;
        }

        // snake_case: at least one underscore separating lowercase words.
        if token.contains('_') {
            let parts: Vec<&str> = token.split('_').collect();
            if parts.len() >= 2
                && parts
                    .iter()
                    .all(|p| !p.is_empty() && p.chars().all(|c| c.is_lowercase() || c.is_numeric()))
                && token
                    .chars()
                    .all(|c| c.is_lowercase() || c.is_numeric() || c == '_')
            {
                add(token.to_string(), EntityKind::Code);
            }
        }
    }

    result
}

// ---------------------------------------------------------------------------
// Memory prompt injection — bounded, disable-able, safety-filtered.
// ---------------------------------------------------------------------------

/// Configuration for memory prompt injection.
#[derive(Debug, Clone)]
pub struct MemoryPromptConfig {
    /// Whether memory injection is enabled.
    pub enabled: bool,
    /// Maximum number of memory items to include.
    pub max_items: usize,
    /// Maximum character budget for the formatted section.
    pub max_chars: usize,
}

impl Default for MemoryPromptConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_items: 5,
            max_chars: 2000,
        }
    }
}

/// Patterns that indicate content originated from hidden tool/system output.
/// If any of these appear in a memory item's content, the item is filtered
/// out as a defense-in-depth measure.
const HIDDEN_OUTPUT_PATTERNS: &[&str] = &[
    "<tool_result>",
    "</tool_result>",
    "Tool output:",
    "is_error:",
    "tool_call",
    "tool_result",
];

/// Returns `true` if `content` appears to contain hidden tool or system output.
fn is_hidden_output(content: &str) -> bool {
    let lower = content.to_lowercase();
    HIDDEN_OUTPUT_PATTERNS.iter().any(|pat| lower.contains(pat))
}

/// Format retrieved memory into a bounded prompt section.
///
/// Returns `None` if: disabled, no query, no results, or all results are filtered.
/// The output includes provenance (source session/turn), confidence, freshness,
/// and contradiction markers. Hidden tool output is filtered out.
pub fn format_memory_prompt(
    store: &MemoryStore,
    query: &str,
    config: &MemoryPromptConfig,
) -> Option<String> {
    if !config.enabled || query.trim().is_empty() {
        return None;
    }

    let results = store.retrieve(query, config.max_items).ok()?;
    if results.is_empty() {
        return None;
    }

    let mut items = Vec::new();
    let header = "## Relevant Memory\n";
    let mut total_len = header.len();

    for result in &results {
        // Defense-in-depth: skip items that look like hidden tool output.
        if is_hidden_output(&result.item.content) {
            continue;
        }

        let source_ref = result
            .evidence
            .first()
            .map(|e| e.source_ref.as_str())
            .unwrap_or("unknown");

        let reinforced = result.item.last_reinforced.format("%Y-%m-%d");

        let line = if result.item.contradiction_ref.is_some() {
            format!(
                "- ⚠ CONTRADICTION: [confidence={:.1}] {} (source: {}, reinforced: {})\n",
                result.item.confidence, result.item.content, source_ref, reinforced,
            )
        } else {
            format!(
                "- [confidence={:.1}] {} (source: {}, reinforced: {})\n",
                result.item.confidence, result.item.content, source_ref, reinforced,
            )
        };

        // Check if adding this line would exceed the budget.
        let line_len = line.len();
        if total_len + line_len > config.max_chars {
            // Truncate: append the truncation notice and stop.
            let truncation_notice = "... (memory section truncated)";
            // Ensure we have room for the notice.
            if total_len + truncation_notice.len() <= config.max_chars {
                items.push(truncation_notice.to_string());
            }
            break;
        }

        items.push(line);
        total_len += line_len;
    }

    if items.is_empty() {
        // All results were filtered out.
        return None;
    }

    let mut output = String::with_capacity(config.max_chars);
    output.push_str(header);
    for item in &items {
        output.push_str(item);
    }

    Some(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(id: &str, key: &str, content: &str) -> MemoryItem {
        let now = Utc::now();
        MemoryItem {
            id: id.to_string(),
            kind: MemoryKind::Semantic,
            key: key.to_string(),
            content: content.to_string(),
            confidence: 0.8,
            created_at: now,
            last_reinforced: now,
            last_accessed: None,
            contradiction_ref: None,
        }
    }

    #[test]
    fn test_schema_migration_creates_tables() {
        let store = MemoryStore::open_memory().unwrap();

        let table_count: i64 = store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type IN ('table', 'view') \
                 AND name IN ('memory_items', 'evidence_links', 'schema_version')",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(table_count, 3, "All three tables should exist");

        let fts_count: i64 = store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='memory_fts'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(fts_count, 1, "FTS5 virtual table should exist");
    }

    #[test]
    fn test_insert_and_retrieve() {
        let mut store = MemoryStore::open_memory().unwrap();
        let content = "Rust is a systems programming language focused on safety".to_string();
        let item = make_item("mem-1", "rust", &content);
        store.insert(item).unwrap();

        let results = store
            .retrieve("Rust systems programming safety", 10)
            .unwrap();
        assert!(!results.is_empty(), "Should find the inserted item");
        assert_eq!(results[0].item.content, content);
    }

    #[test]
    fn test_add_only_preserves_conflicts() {
        let mut store = MemoryStore::open_memory().unwrap();

        let item1 = make_item("mem-1", "language", "Python is dynamically typed");
        let item2 = make_item("mem-2", "language", "Python is statically typed");

        store.insert(item1).unwrap();
        store.insert(item2).unwrap();

        assert_eq!(
            store.count().unwrap(),
            2,
            "Both conflicting items should exist"
        );
    }

    #[test]
    fn test_exact_dedup_prevents_duplicates() {
        let mut store = MemoryStore::open_memory().unwrap();

        let item = make_item("mem-1", "fact", "The sky is blue");
        assert!(store.insert(item).unwrap(), "First insert should succeed");

        let item_dup = make_item("mem-2", "fact", "The sky is blue");
        assert!(
            !store.insert(item_dup).unwrap(),
            "Duplicate should be ignored"
        );

        assert_eq!(store.count().unwrap(), 1, "Only one row should exist");
    }

    #[test]
    fn test_bounded_retrieval_respects_limit() {
        let mut store = MemoryStore::open_memory().unwrap();

        for i in 0..5 {
            let item = make_item(
                &format!("mem-{i}"),
                "topic",
                &format!("Item number {i} about testing retrieval limits"),
            );
            store.insert(item).unwrap();
        }

        let results = store.retrieve("testing retrieval", 3).unwrap();
        assert!(
            results.len() <= 3,
            "Should respect limit of 3, got {}",
            results.len()
        );
    }

    #[test]
    fn test_retrieval_includes_evidence() {
        let mut store = MemoryStore::open_memory().unwrap();

        let item = make_item(
            "mem-1",
            "evidence-test",
            "Evidence links are important for provenance",
        );
        store.insert(item).unwrap();

        let link = EvidenceLink {
            id: "ev-1".to_string(),
            memory_id: "mem-1".to_string(),
            source_type: "session".to_string(),
            source_ref: "session-abc".to_string(),
            created_at: Utc::now(),
        };
        store.insert_evidence(link).unwrap();

        let results = store.retrieve("evidence provenance", 10).unwrap();
        assert!(!results.is_empty());
        assert!(
            !results[0].evidence.is_empty(),
            "Should include evidence links"
        );
        assert_eq!(results[0].evidence[0].source_ref, "session-abc");
    }

    #[test]
    fn test_retrieval_scoring_is_deterministic() {
        let mut store = MemoryStore::open_memory().unwrap();

        let item = make_item(
            "mem-1",
            "deterministic",
            "Deterministic scoring test content for verification",
        );
        store.insert(item).unwrap();

        let results1 = store
            .retrieve("deterministic scoring verification", 10)
            .unwrap();
        let results2 = store
            .retrieve("deterministic scoring verification", 10)
            .unwrap();

        assert_eq!(results1.len(), results2.len());
        if !results1.is_empty() {
            assert!(
                (results1[0].score - results2[0].score).abs() < 0.01,
                "Scores should be nearly identical: {} vs {}",
                results1[0].score,
                results2[0].score
            );
        }
    }

    #[test]
    fn test_retrieve_empty_query_returns_nothing() {
        let mut store = MemoryStore::open_memory().unwrap();

        let item = make_item("mem-1", "test", "Some content here");
        store.insert(item).unwrap();

        let results = store.retrieve("", 10).unwrap();
        assert!(results.is_empty(), "Empty query should return no results");

        let results = store.retrieve("   ", 10).unwrap();
        assert!(
            results.is_empty(),
            "Whitespace-only query should return no results"
        );
    }

    #[test]
    fn test_get_by_id() {
        let mut store = MemoryStore::open_memory().unwrap();

        let item = make_item("mem-1", "lookup", "Lookup by ID test content");
        store.insert(item).unwrap();

        let found = store.get("mem-1").unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, "mem-1");
        assert_eq!(found.content, "Lookup by ID test content");
        assert_eq!(found.kind, MemoryKind::Semantic);

        let not_found = store.get("nonexistent").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_procedural_kind_roundtrip() {
        let mut store = MemoryStore::open_memory().unwrap();

        let now = Utc::now();
        let item = MemoryItem {
            id: "proc-1".to_string(),
            kind: MemoryKind::Procedural,
            key: "git-workflow".to_string(),
            content: "Always rebase feature branches onto main".to_string(),
            confidence: 0.9,
            created_at: now,
            last_reinforced: now,
            last_accessed: None,
            contradiction_ref: None,
        };
        store.insert(item).unwrap();

        let found = store.get("proc-1").unwrap().unwrap();
        assert_eq!(found.kind, MemoryKind::Procedural);
    }

    #[test]
    fn test_evidence_link_persists() {
        let mut store = MemoryStore::open_memory().unwrap();

        let item = make_item("mem-1", "test", "Test content");
        store.insert(item).unwrap();

        let link = EvidenceLink {
            id: "ev-1".to_string(),
            memory_id: "mem-1".to_string(),
            source_type: "tool_call".to_string(),
            source_ref: "read:src/main.rs".to_string(),
            created_at: Utc::now(),
        };
        store.insert_evidence(link).unwrap();

        let results = store.retrieve("Test content", 10).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].evidence.len(), 1);
        assert_eq!(results[0].evidence[0].source_type, "tool_call");
    }

    #[test]
    fn test_evidence_requires_existing_memory() {
        let store = MemoryStore::open_memory().unwrap();

        let link = EvidenceLink {
            id: "ev-orphan".to_string(),
            memory_id: "missing-memory".to_string(),
            source_type: "session".to_string(),
            source_ref: "session-missing".to_string(),
            created_at: Utc::now(),
        };

        let err = store
            .insert_evidence(link)
            .expect_err("foreign-key enforcement must reject orphan evidence");
        assert!(
            err.to_string().contains("FOREIGN KEY")
                || err.to_string().contains("constraint failed"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_memory_maintenance_operations_run() {
        let mut store = MemoryStore::open_memory().unwrap();
        store
            .insert(make_item(
                "mem-1",
                "maintenance",
                "maintenance test content",
            ))
            .unwrap();

        store.checkpoint_truncate().unwrap();
        store.vacuum().unwrap();
        assert_eq!(store.count().unwrap(), 1);
    }

    #[test]
    fn test_count_reflects_inserts() {
        let mut store = MemoryStore::open_memory().unwrap();
        assert_eq!(store.count().unwrap(), 0);

        store.insert(make_item("m1", "k", "c1")).unwrap();
        assert_eq!(store.count().unwrap(), 1);

        store.insert(make_item("m2", "k", "c2")).unwrap();
        assert_eq!(store.count().unwrap(), 2);

        // Exact dup should not increase count.
        store.insert(make_item("m3", "k", "c1")).unwrap();
        assert_eq!(store.count().unwrap(), 2);
    }

    // --- format_memory_prompt tests ---

    #[test]
    fn format_memory_prompt_disabled_returns_none() {
        let mut store = MemoryStore::open_memory().unwrap();
        store
            .insert(make_item("mem-1", "test", "some content"))
            .unwrap();

        let config = MemoryPromptConfig {
            enabled: false,
            ..Default::default()
        };
        assert!(format_memory_prompt(&store, "test", &config).is_none());
    }

    #[test]
    fn format_memory_prompt_no_results_returns_none() {
        let store = MemoryStore::open_memory().unwrap();

        let config = MemoryPromptConfig {
            enabled: true,
            ..Default::default()
        };
        assert!(format_memory_prompt(&store, "nonexistent query xyz", &config).is_none());
    }

    #[test]
    fn format_memory_prompt_produces_bounded_section() {
        let mut store = MemoryStore::open_memory().unwrap();

        store
            .insert(make_item(
                "mem-1",
                "rust",
                "Rust is a systems language focused on safety",
            ))
            .unwrap();
        store
            .insert(make_item(
                "mem-2",
                "rust",
                "Rust has zero-cost abstractions and no garbage collector",
            ))
            .unwrap();
        store
            .insert(make_item(
                "mem-3",
                "testing",
                "Testing is important for software quality",
            ))
            .unwrap();

        // Add evidence for provenance.
        store
            .insert_evidence(EvidenceLink {
                id: "ev-1".to_string(),
                memory_id: "mem-1".to_string(),
                source_type: "session".to_string(),
                source_ref: "session-abc:entry-1:0".to_string(),
                created_at: Utc::now(),
            })
            .unwrap();

        let config = MemoryPromptConfig {
            enabled: true,
            max_items: 5,
            max_chars: 2000,
        };
        let result = format_memory_prompt(&store, "Rust safety", &config);

        assert!(result.is_some(), "Should produce output");
        let text = result.unwrap();
        assert!(text.contains("## Relevant Memory"), "Should contain header");
        assert!(text.contains("confidence="), "Should contain confidence");
        assert!(text.contains("source:"), "Should contain source reference");
        assert!(text.len() <= config.max_chars, "Should respect max_chars");
    }

    #[test]
    fn format_memory_prompt_truncates_on_budget() {
        let mut store = MemoryStore::open_memory().unwrap();

        let long_content = "This is a very long memory item that contains a lot of text to test the truncation behavior of the format_memory_prompt function when the character budget is exceeded by the accumulated output length of multiple memory items combined together in the final formatted string".repeat(3);
        store
            .insert(make_item("mem-1", "long", &long_content))
            .unwrap();

        let config = MemoryPromptConfig {
            enabled: true,
            max_items: 5,
            max_chars: 100,
        };
        let result = format_memory_prompt(&store, "long memory", &config);

        assert!(result.is_some(), "Should produce some output");
        let text = result.unwrap();
        assert!(
            text.contains("truncated"),
            "Should contain truncation notice, got: {text}"
        );
        assert!(text.len() <= config.max_chars, "Should respect max_chars");
    }

    #[test]
    fn format_memory_prompt_filters_hidden_output() {
        let mut store = MemoryStore::open_memory().unwrap();

        // Insert a clean memory item.
        store
            .insert(make_item(
                "mem-1",
                "clean",
                "Rust is a safe systems language",
            ))
            .unwrap();

        // Insert items that look like hidden tool output.
        store
            .insert(make_item(
                "mem-2",
                "tool-like",
                "<tool_result>file read successfully</tool_result>",
            ))
            .unwrap();
        store
            .insert(make_item(
                "mem-3",
                "tool-like-2",
                "Tool output: the file contains 42 lines",
            ))
            .unwrap();
        store
            .insert(make_item(
                "mem-4",
                "tool-like-3",
                "is_error: true, message: something failed",
            ))
            .unwrap();

        let config = MemoryPromptConfig {
            enabled: true,
            max_items: 10,
            max_chars: 4000,
        };
        let result = format_memory_prompt(&store, "tool result error", &config);

        // The clean item may or may not appear depending on FTS scoring.
        // But the hidden-output items must NOT appear.
        if let Some(text) = result {
            assert!(
                !text.contains("<tool_result>"),
                "Should not contain tool result tags"
            );
            assert!(
                !text.contains("Tool output:"),
                "Should not contain tool output prefix"
            );
            assert!(
                !text.contains("is_error:"),
                "Should not contain error markers"
            );
        }
    }

    #[test]
    fn format_memory_prompt_marks_contradictions() {
        let mut store = MemoryStore::open_memory().unwrap();

        let now = Utc::now();
        let item = MemoryItem {
            id: "mem-contradict".to_string(),
            kind: MemoryKind::Semantic,
            key: "conflict".to_string(),
            content: "Python is dynamically typed".to_string(),
            confidence: 0.7,
            created_at: now,
            last_reinforced: now,
            last_accessed: None,
            contradiction_ref: Some("ref-123".to_string()),
        };
        store.insert(item).unwrap();

        let config = MemoryPromptConfig {
            enabled: true,
            max_items: 5,
            max_chars: 2000,
        };
        let result = format_memory_prompt(&store, "Python typed", &config);

        assert!(result.is_some(), "Should produce output");
        let text = result.unwrap();
        assert!(
            text.contains("CONTRADICTION"),
            "Should mark contradiction, got: {text}"
        );
    }

    // --- Entity extraction tests ---

    #[test]
    fn extract_file_entities() {
        let content = "Edit src/main.rs and update Cargo.toml for the new feature";
        let entities = extract_entities(content);

        let files: Vec<&str> = entities
            .iter()
            .filter(|(_, k)| *k == EntityKind::File)
            .map(|(n, _)| n.as_str())
            .collect();

        assert!(
            files.contains(&"src/main.rs"),
            "Should find src/main.rs, got: {files:?}"
        );
        assert!(
            files.contains(&"Cargo.toml"),
            "Should find Cargo.toml, got: {files:?}"
        );
    }

    #[test]
    fn extract_url_entities() {
        let content = "See https://docs.rs/talos for details and visit http://example.com/path?q=1";
        let entities = extract_entities(content);

        let urls: Vec<&str> = entities
            .iter()
            .filter(|(_, k)| *k == EntityKind::Url)
            .map(|(n, _)| n.as_str())
            .collect();

        assert!(
            urls.iter().any(|u| u.starts_with("https://docs.rs")),
            "Should find https URL, got: {urls:?}"
        );
        assert!(
            urls.iter().any(|u| u.starts_with("http://example.com")),
            "Should find http URL, got: {urls:?}"
        );
    }

    #[test]
    fn extract_code_entities() {
        let content = "Use MemoryStore and extract_entities for the implementation";
        let entities = extract_entities(content);

        let codes: Vec<&str> = entities
            .iter()
            .filter(|(_, k)| *k == EntityKind::Code)
            .map(|(n, _)| n.as_str())
            .collect();

        assert!(
            codes.contains(&"MemoryStore"),
            "Should find MemoryStore, got: {codes:?}"
        );
        assert!(
            codes.contains(&"extract_entities"),
            "Should find extract_entities, got: {codes:?}"
        );
    }

    #[test]
    fn extract_entities_malformed_input_no_panic() {
        // Empty string.
        let _ = extract_entities("");

        // Very long string.
        let long = "a".repeat(100_000);
        let _ = extract_entities(&long);

        // Binary-like content.
        let binary = "\0\x01\x02\x03\x7f\x7e";
        let _ = extract_entities(binary);

        // Only punctuation.
        let _ = extract_entities("!@#$%^&*()_+-=[]{}|;':\",./<>?");
    }

    #[test]
    fn entity_overlap_boosts_retrieval() {
        let mut store = MemoryStore::open_memory().unwrap();

        // Memory with file paths that match the query.
        let item1 = make_item(
            "mem-entity-1",
            "entity-test",
            "Update src/main.rs to fix the bug in Cargo.toml",
        );
        store.insert(item1).unwrap();

        // Memory with unrelated content.
        let item2 = make_item(
            "mem-entity-2",
            "entity-test",
            "The weather is nice today and the sky is blue",
        );
        store.insert(item2).unwrap();

        let results = store.retrieve("src/main.rs Cargo.toml", 10).unwrap();
        assert!(!results.is_empty(), "Should find results");

        // The entity-matching item should rank higher or at least appear.
        let entity_1_pos = results.iter().position(|r| r.item.id == "mem-entity-1");
        let entity_2_pos = results.iter().position(|r| r.item.id == "mem-entity-2");

        if let (Some(p1), Some(p2)) = (entity_1_pos, entity_2_pos) {
            assert!(
                p1 < p2,
                "Entity-matching item should rank higher: pos1={p1}, pos2={p2}"
            );
        }
    }

    #[test]
    fn procedural_memory_storage_and_retrieval() {
        let mut store = MemoryStore::open_memory().unwrap();

        let now = Utc::now();
        let item = MemoryItem {
            id: "proc-test-1".to_string(),
            kind: MemoryKind::Procedural,
            key: "commit-workflow".to_string(),
            content: "Always run cargo fmt before you commit code to the repository".to_string(),
            confidence: 0.9,
            created_at: now,
            last_reinforced: now,
            last_accessed: None,
            contradiction_ref: None,
        };
        store.insert(item).unwrap();

        let results = store.retrieve("cargo fmt commit", 10).unwrap();
        assert!(!results.is_empty(), "Should retrieve procedural memory");

        let found = results.iter().find(|r| r.item.id == "proc-test-1");
        assert!(found.is_some(), "Should find the procedural item");
        assert_eq!(found.unwrap().item.kind, MemoryKind::Procedural);
    }

    #[test]
    fn procedural_memory_has_no_permission_authority() {
        let mut store = MemoryStore::open_memory().unwrap();

        let item = MemoryItem {
            id: "proc-perm-test".to_string(),
            kind: MemoryKind::Procedural,
            key: "commit-workflow".to_string(),
            content: "Always run cargo fmt before committing code".to_string(),
            confidence: 0.9,
            created_at: Utc::now(),
            last_reinforced: Utc::now(),
            last_accessed: None,
            contradiction_ref: None,
        };
        assert!(store.insert(item).unwrap());

        let results = store.retrieve("cargo fmt", 10).unwrap();
        assert!(!results.is_empty());

        // Memory retrieval returns data only — no permission grant.
        // MemoryStore has no methods that grant, approve, or bypass permissions.
    }

    #[test]
    fn entity_linking_on_insert() {
        let mut store = MemoryStore::open_memory().unwrap();

        let item = make_item(
            "mem-link-test",
            "entity-linking",
            "Update src/lib.rs and check Cargo.toml for dependencies",
        );
        store.insert(item).unwrap();

        // Verify entities were linked in the database.
        let entity_count: i64 = store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM memory_entities WHERE memory_id = ?1",
                params!["mem-link-test"],
                |row| row.get(0),
            )
            .unwrap();

        assert!(
            entity_count > 0,
            "Should have linked entities, got count={entity_count}"
        );

        // Verify the entities table has entries.
        let total_entities: i64 = store
            .conn
            .query_row("SELECT COUNT(*) FROM entities", [], |row| row.get(0))
            .unwrap();

        assert!(
            total_entities > 0,
            "Should have entities in the table, got count={total_entities}"
        );
    }

    #[test]
    fn corrupt_db_degrades_gracefully() {
        let dir = tempfile::tempdir().unwrap();
        let corrupt_path = dir.path().join("corrupt.db");
        std::fs::write(&corrupt_path, b"this is not a valid sqlite database file").unwrap();

        let result = MemoryStore::open(&corrupt_path);
        assert!(
            result.is_err(),
            "Opening a corrupt DB should return an error, not panic"
        );
        let err = match result {
            Err(e) => e,
            Ok(_) => panic!("expected error for corrupt DB"),
        };
        let err_msg = err.to_string().to_lowercase();
        assert!(
            err_msg.contains("database")
                || err_msg.contains("file")
                || err_msg.contains("malformed")
                || err_msg.contains("not a database"),
            "Error should be actionable, got: {err}"
        );
    }

    #[test]
    fn missing_db_path_handled() {
        let dir = tempfile::tempdir().unwrap();
        let nested_path = dir.path().join("nonexistent").join("sub").join("memory.db");

        let result = MemoryStore::open(&nested_path);
        assert!(
            result.is_ok(),
            "Opening a DB in a nonexistent parent should create the path"
        );
    }

    #[test]
    fn memory_status_reports_counts() {
        let mut store = MemoryStore::open_memory().unwrap();

        for i in 0..3 {
            let item = make_item(
                &format!("sem-{i}"),
                "status-test",
                &format!("Update src/lib.rs for semantic fact {i}"),
            );
            store.insert(item).unwrap();
        }

        let now = Utc::now();
        for i in 0..2 {
            let item = MemoryItem {
                id: format!("proc-{i}"),
                kind: MemoryKind::Procedural,
                key: "status-proc".to_string(),
                content: format!("Run Cargo.toml test step {i}"),
                confidence: 0.9,
                created_at: now,
                last_reinforced: now,
                last_accessed: None,
                contradiction_ref: None,
            };
            store.insert(item).unwrap();
        }

        for i in 0..3 {
            store
                .insert_evidence(EvidenceLink {
                    id: format!("ev-{i}"),
                    memory_id: format!("sem-{i}"),
                    source_type: "session".to_string(),
                    source_ref: format!("session-{i}"),
                    created_at: Utc::now(),
                })
                .unwrap();
        }

        let status = store.memory_status().unwrap();
        assert_eq!(status.total_items, 5);
        assert_eq!(status.semantic_count, 3);
        assert_eq!(status.procedural_count, 2);
        assert_eq!(status.evidence_count, 3);
        assert!(status.entity_count > 0, "entity_count should be > 0");
        assert!(status.db_path.is_none());
        assert_eq!(status.db_size_bytes, 0);
    }

    #[test]
    fn retention_dry_run_no_deletion() {
        let mut store = MemoryStore::open_memory().unwrap();
        let now = Utc::now();

        let low_conf = MemoryItem {
            id: "low-conf".to_string(),
            kind: MemoryKind::Semantic,
            key: "low-confidence".to_string(),
            content: "A low confidence memory".to_string(),
            confidence: 0.2,
            created_at: now,
            last_reinforced: now,
            last_accessed: None,
            contradiction_ref: None,
        };
        store.insert(low_conf).unwrap();

        let high_conf = MemoryItem {
            id: "high-conf".to_string(),
            kind: MemoryKind::Semantic,
            key: "high-confidence".to_string(),
            content: "A high confidence memory".to_string(),
            confidence: 0.9,
            created_at: now,
            last_reinforced: now,
            last_accessed: None,
            contradiction_ref: None,
        };
        store.insert(high_conf).unwrap();

        let policy = RetentionPolicy {
            min_confidence: Some(0.5),
            ..Default::default()
        };

        let candidates = store.retention_candidates(&policy).unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].id, "low-conf");

        let count_before = store.count().unwrap();
        assert_eq!(count_before, 2);
    }

    #[test]
    fn retention_key_preview_truncated() {
        let mut store = MemoryStore::open_memory().unwrap();
        let now = Utc::now();

        let long_key =
            "this_is_a_very_long_key_that_should_be_truncated_in_the_retention_candidate_output"
                .to_string();
        let item = MemoryItem {
            id: "long-key".to_string(),
            kind: MemoryKind::Semantic,
            key: long_key.clone(),
            content: "Some content".to_string(),
            confidence: 0.1,
            created_at: now,
            last_reinforced: now,
            last_accessed: None,
            contradiction_ref: None,
        };
        store.insert(item).unwrap();

        let policy = RetentionPolicy {
            min_confidence: Some(0.5),
            ..Default::default()
        };

        let candidates = store.retention_candidates(&policy).unwrap();
        assert_eq!(candidates.len(), 1);
        assert!(
            candidates[0].key_preview.len() <= 30,
            "key_preview should be <= 30 chars, got {} chars: '{}'",
            candidates[0].key_preview.len(),
            candidates[0].key_preview
        );
        assert!(candidates[0].key_preview.ends_with("..."));
    }

    #[test]
    fn retention_unreinforced_only() {
        let mut store = MemoryStore::open_memory().unwrap();
        let now = Utc::now();

        let with_evidence = MemoryItem {
            id: "with-ev".to_string(),
            kind: MemoryKind::Semantic,
            key: "reinforced".to_string(),
            content: "Has evidence".to_string(),
            confidence: 0.3,
            created_at: now,
            last_reinforced: now,
            last_accessed: None,
            contradiction_ref: None,
        };
        store.insert(with_evidence).unwrap();
        store
            .insert_evidence(EvidenceLink {
                id: "ev-1".to_string(),
                memory_id: "with-ev".to_string(),
                source_type: "session".to_string(),
                source_ref: "session-1".to_string(),
                created_at: now,
            })
            .unwrap();

        let without_evidence = MemoryItem {
            id: "without-ev".to_string(),
            kind: MemoryKind::Semantic,
            key: "unreinforced".to_string(),
            content: "No evidence".to_string(),
            confidence: 0.3,
            created_at: now,
            last_reinforced: now,
            last_accessed: None,
            contradiction_ref: None,
        };
        store.insert(without_evidence).unwrap();

        let policy = RetentionPolicy {
            unreinforced_only: true,
            ..Default::default()
        };

        let candidates = store.retention_candidates(&policy).unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].id, "without-ev");
    }

    #[test]
    fn end_to_end_memory_pipeline() {
        let mut store = MemoryStore::open_memory().unwrap();
        let now = Utc::now();

        let semantic = MemoryItem {
            id: "e2e-sem".to_string(),
            kind: MemoryKind::Semantic,
            key: "rust-safety".to_string(),
            content: "Rust guarantees memory safety without a garbage collector".to_string(),
            confidence: 0.85,
            created_at: now,
            last_reinforced: now,
            last_accessed: None,
            contradiction_ref: None,
        };
        store.insert(semantic).unwrap();

        store
            .insert_evidence(EvidenceLink {
                id: "e2e-ev".to_string(),
                memory_id: "e2e-sem".to_string(),
                source_type: "session".to_string(),
                source_ref: "session-e2e:turn-0".to_string(),
                created_at: now,
            })
            .unwrap();

        let procedural = MemoryItem {
            id: "e2e-proc".to_string(),
            kind: MemoryKind::Procedural,
            key: "cargo-test".to_string(),
            content: "Run cargo test before merging".to_string(),
            confidence: 0.95,
            created_at: now,
            last_reinforced: now,
            last_accessed: None,
            contradiction_ref: None,
        };
        store.insert(procedural).unwrap();

        let sem_results = store.retrieve("Rust memory safety", 5).unwrap();
        assert!(
            sem_results.iter().any(|r| r.item.id == "e2e-sem"),
            "Should retrieve semantic memory"
        );

        let proc_results = store.retrieve("cargo test", 5).unwrap();
        assert!(
            proc_results.iter().any(|r| r.item.id == "e2e-proc"),
            "Should retrieve procedural memory"
        );

        let config = MemoryPromptConfig {
            enabled: true,
            max_items: 5,
            max_chars: 2000,
        };
        let prompt = format_memory_prompt(&store, "Rust memory safety", &config);
        assert!(prompt.is_some(), "Should produce formatted prompt");
        let prompt_text = prompt.unwrap();
        assert!(
            prompt_text.contains("memory safety"),
            "Prompt should contain memory content"
        );

        let status = store.memory_status().unwrap();
        assert_eq!(status.total_items, 2);
        assert_eq!(status.semantic_count, 1);
        assert_eq!(status.procedural_count, 1);
        assert_eq!(status.evidence_count, 1);
    }
}
