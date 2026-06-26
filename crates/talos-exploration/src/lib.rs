//! Talos exploration layer — local research storage with citation integrity.
//!
//! Implements the exploration storage schema defined in ADR-017. Provides
//! SQLite/FTS5-backed storage for research runs, sources, source chunks,
//! claims, claim edges, and syntheses. Citation integrity is enforced:
//! claims must reference valid chunks, syntheses must reference valid sources.
//!
//! # Design Principles
//!
//! - **Citation integrity**: every claim and synthesis must reference existing entities.
//! - **FTS5 search**: source chunks are indexed for full-text search.
//! - **No network**: this crate is storage-only; no fetching or web access.
//! - **No vector/graph**: pure relational storage with FTS5.

pub mod ingestion;

use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

// ---------------------------------------------------------------------------
// Edge type enumeration
// ---------------------------------------------------------------------------

/// The type of relationship between two claims.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    Supports,
    Contradicts,
    Refines,
    DependsOn,
    DerivedFrom,
}

impl std::fmt::Display for EdgeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EdgeType::Supports => write!(f, "supports"),
            EdgeType::Contradicts => write!(f, "contradicts"),
            EdgeType::Refines => write!(f, "refines"),
            EdgeType::DependsOn => write!(f, "depends_on"),
            EdgeType::DerivedFrom => write!(f, "derived_from"),
        }
    }
}

// ---------------------------------------------------------------------------
// Entity types
// ---------------------------------------------------------------------------

/// A research run tracking a single exploration session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchRun {
    pub id: String,
    pub query: String,
    pub plan: Option<String>,
    pub tools_used: Option<String>,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// A source document ingested during a research run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub id: String,
    pub run_id: Option<String>,
    pub url: Option<String>,
    pub title: String,
    pub authors: Option<String>,
    pub publication_date: Option<String>,
    pub fetched_at: DateTime<Utc>,
    pub license_notes: Option<String>,
    pub content_hash: String,
}

/// A chunk of text extracted from a source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceChunk {
    pub id: String,
    pub source_id: String,
    pub chunk_ordinal: i64,
    pub text: String,
    pub token_estimate: Option<i64>,
}

/// A claim extracted from a source chunk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub id: String,
    pub run_id: Option<String>,
    pub source_chunk_id: Option<String>,
    pub normalized_text: String,
    pub confidence: f64,
    pub status: String,
    pub freshness: String,
    pub created_at: DateTime<Utc>,
}

/// An edge connecting two claims.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimEdge {
    pub id: String,
    pub source_claim_id: String,
    pub target_claim_id: String,
    pub edge_type: EdgeType,
    pub created_at: DateTime<Utc>,
}

/// A synthesis combining claims from a research run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Synthesis {
    pub id: String,
    pub run_id: Option<String>,
    pub conclusion: String,
    pub caveats: Option<String>,
    pub cited_source_ids: Vec<String>,
    pub unresolved_questions: Vec<String>,
    pub created_at: DateTime<Utc>,
}

/// A single FTS search result.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub chunk_id: String,
    pub source_id: String,
    pub source_title: String,
    pub snippet: String,
    pub rank: f64,
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur during exploration store operations.
#[derive(Debug, Error)]
pub enum ExplorationError {
    /// A database operation failed.
    #[error("database operation failed: {0}")]
    Database(#[from] rusqlite::Error),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The requested entity was not found.
    #[error("entity not found: {0}")]
    NotFound(String),

    /// Citation validation failed: a referenced entity does not exist.
    #[error("citation validation failed: {0}")]
    CitationValidation(String),

    /// Invalid input provided.
    #[error("invalid input: {0}")]
    InvalidInput(String),
}

// ---------------------------------------------------------------------------
// ExplorationStore
// ---------------------------------------------------------------------------

/// SQLite-backed store for exploration data.
///
/// Provides CRUD operations for research runs, sources, source chunks,
/// claims, claim edges, and syntheses with citation integrity enforcement.
pub struct ExplorationStore {
    conn: Connection,
}

impl ExplorationStore {
    /// Open or create an exploration store at the given file path.
    ///
    /// Creates parent directories if they do not exist. Runs schema migration
    /// automatically.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, ExplorationError> {
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
    pub fn open_memory() -> Result<Self, ExplorationError> {
        let conn = Connection::open_in_memory()?;
        configure_connection(&conn)?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    /// Create the database schema if it does not exist.
    fn migrate(&self) -> Result<(), ExplorationError> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS schema_version (version INTEGER PRIMARY KEY);

            CREATE TABLE IF NOT EXISTS research_runs (
                id TEXT PRIMARY KEY,
                query TEXT NOT NULL,
                plan TEXT,
                tools_used TEXT,
                model TEXT,
                provider TEXT,
                status TEXT NOT NULL DEFAULT 'active',
                started_at TEXT NOT NULL,
                completed_at TEXT
            );

            CREATE TABLE IF NOT EXISTS sources (
                id TEXT PRIMARY KEY,
                run_id TEXT REFERENCES research_runs(id),
                url TEXT,
                title TEXT NOT NULL,
                authors TEXT,
                publication_date TEXT,
                fetched_at TEXT NOT NULL,
                license_notes TEXT,
                content_hash TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS source_chunks (
                id TEXT PRIMARY KEY,
                source_id TEXT NOT NULL REFERENCES sources(id),
                chunk_ordinal INTEGER NOT NULL,
                text TEXT NOT NULL,
                token_estimate INTEGER
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS source_chunks_fts USING fts5(
                chunk_id UNINDEXED, text, tokenize='unicode61'
            );

            CREATE TABLE IF NOT EXISTS claims (
                id TEXT PRIMARY KEY,
                run_id TEXT REFERENCES research_runs(id),
                source_chunk_id TEXT REFERENCES source_chunks(id),
                normalized_text TEXT NOT NULL,
                confidence REAL NOT NULL DEFAULT 0.5,
                status TEXT NOT NULL DEFAULT 'active',
                freshness TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS claim_edges (
                id TEXT PRIMARY KEY,
                source_claim_id TEXT NOT NULL REFERENCES claims(id),
                target_claim_id TEXT NOT NULL REFERENCES claims(id),
                edge_type TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS syntheses (
                id TEXT PRIMARY KEY,
                run_id TEXT REFERENCES research_runs(id),
                conclusion TEXT NOT NULL,
                caveats TEXT,
                cited_source_ids TEXT NOT NULL,
                unresolved_questions TEXT,
                created_at TEXT NOT NULL
            );
            "#,
        )?;

        // Initialize schema_version if empty.
        let version_count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM schema_version", [], |row| row.get(0))?;
        if version_count == 0 {
            let _ = self
                .conn
                .execute("INSERT INTO schema_version (version) VALUES (1)", []);
        }

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Research runs
    // -----------------------------------------------------------------------

    /// Create a new research run.
    pub fn create_run(
        &self,
        query: &str,
        plan: Option<&str>,
    ) -> Result<ResearchRun, ExplorationError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        self.conn.execute(
            "INSERT INTO research_runs (id, query, plan, status, started_at) VALUES (?1, ?2, ?3, 'active', ?4)",
            params![id, query, plan, now.to_rfc3339()],
        )?;

        Ok(ResearchRun {
            id,
            query: query.to_string(),
            plan: plan.map(String::from),
            tools_used: None,
            model: None,
            provider: None,
            status: "active".to_string(),
            started_at: now,
            completed_at: None,
        })
    }

    /// Mark a research run as completed.
    pub fn complete_run(&self, run_id: &str) -> Result<(), ExplorationError> {
        let now = Utc::now();
        let changes = self.conn.execute(
            "UPDATE research_runs SET status = 'completed', completed_at = ?1 WHERE id = ?2",
            params![now.to_rfc3339(), run_id],
        )?;
        if changes == 0 {
            return Err(ExplorationError::NotFound(format!("research run {run_id}")));
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Sources
    // -----------------------------------------------------------------------

    /// Insert a source record.
    pub fn insert_source(&self, source: &Source) -> Result<(), ExplorationError> {
        self.conn.execute(
            "INSERT INTO sources (id, run_id, url, title, authors, publication_date, fetched_at, license_notes, content_hash) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                source.id,
                source.run_id,
                source.url,
                source.title,
                source.authors,
                source.publication_date,
                source.fetched_at.to_rfc3339(),
                source.license_notes,
                source.content_hash,
            ],
        )?;
        Ok(())
    }

    /// Get a source by ID.
    pub fn get_source(&self, id: &str) -> Result<Option<Source>, ExplorationError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, run_id, url, title, authors, publication_date, fetched_at, license_notes, content_hash \
             FROM sources WHERE id = ?1",
        )?;

        let row_data = stmt
            .query_row(params![id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, String>(8)?,
                ))
            })
            .optional()?;

        match row_data {
            Some((
                id,
                run_id,
                url,
                title,
                authors,
                publication_date,
                fetched_at_str,
                license_notes,
                content_hash,
            )) => Ok(Some(Source {
                id,
                run_id,
                url,
                title,
                authors,
                publication_date,
                fetched_at: parse_datetime(&fetched_at_str)?,
                license_notes,
                content_hash,
            })),
            None => Ok(None),
        }
    }

    // -----------------------------------------------------------------------
    // Source chunks
    // -----------------------------------------------------------------------

    /// Insert a source chunk and index it in FTS5.
    pub fn insert_chunk(&mut self, chunk: &SourceChunk) -> Result<(), ExplorationError> {
        let tx = self.conn.transaction()?;

        tx.execute(
            "INSERT INTO source_chunks (id, source_id, chunk_ordinal, text, token_estimate) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                chunk.id,
                chunk.source_id,
                chunk.chunk_ordinal,
                chunk.text,
                chunk.token_estimate,
            ],
        )?;

        tx.execute(
            "INSERT INTO source_chunks_fts (chunk_id, text) VALUES (?1, ?2)",
            params![chunk.id, chunk.text],
        )?;

        tx.commit()?;
        Ok(())
    }

    /// Search source chunks using FTS5.
    pub fn search_chunks(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, ExplorationError> {
        if query.trim().is_empty() || limit == 0 {
            return Ok(Vec::new());
        }

        let fts_query = escape_fts_query(query);

        let mut stmt = self.conn.prepare(
            "SELECT fts.chunk_id, fts.rank, sc.source_id, s.title, sc.text \
             FROM source_chunks_fts fts \
             JOIN source_chunks sc ON fts.chunk_id = sc.id \
             JOIN sources s ON sc.source_id = s.id \
             WHERE source_chunks_fts MATCH ?1 \
             ORDER BY rank ASC \
             LIMIT ?2",
        )?;

        let rows = stmt
            .query_map(params![fts_query, limit as i64], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, f64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        let results = rows
            .into_iter()
            .map(|(chunk_id, rank, source_id, source_title, text)| {
                let snippet = if text.len() > 200 {
                    format!("{}...", &text[..197])
                } else {
                    text
                };
                SearchResult {
                    chunk_id,
                    source_id,
                    source_title,
                    snippet,
                    rank: rank.abs(),
                }
            })
            .collect();

        Ok(results)
    }

    // -----------------------------------------------------------------------
    // Claims
    // -----------------------------------------------------------------------

    /// Insert a claim with citation validation.
    ///
    /// If `source_chunk_id` is provided, it must reference an existing chunk.
    pub fn insert_claim(&self, claim: &Claim) -> Result<(), ExplorationError> {
        // Validate source_chunk_id if provided.
        if let Some(ref chunk_id) = claim.source_chunk_id {
            let exists: i64 = self.conn.query_row(
                "SELECT COUNT(*) FROM source_chunks WHERE id = ?1",
                params![chunk_id],
                |row| row.get(0),
            )?;
            if exists == 0 {
                return Err(ExplorationError::CitationValidation(format!(
                    "source chunk {chunk_id} does not exist"
                )));
            }
        }

        // Validate run_id if provided.
        if let Some(ref run_id) = claim.run_id {
            let exists: i64 = self.conn.query_row(
                "SELECT COUNT(*) FROM research_runs WHERE id = ?1",
                params![run_id],
                |row| row.get(0),
            )?;
            if exists == 0 {
                return Err(ExplorationError::CitationValidation(format!(
                    "research run {run_id} does not exist"
                )));
            }
        }

        self.conn.execute(
            "INSERT INTO claims (id, run_id, source_chunk_id, normalized_text, confidence, status, freshness, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                claim.id,
                claim.run_id,
                claim.source_chunk_id,
                claim.normalized_text,
                claim.confidence,
                claim.status,
                claim.freshness,
                claim.created_at.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Claim edges
    // -----------------------------------------------------------------------

    /// Insert a claim edge with FK validation.
    ///
    /// Both source and target claim IDs must exist.
    pub fn insert_claim_edge(&self, edge: &ClaimEdge) -> Result<(), ExplorationError> {
        // Validate source claim exists.
        let src_exists: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM claims WHERE id = ?1",
            params![edge.source_claim_id],
            |row| row.get(0),
        )?;
        if src_exists == 0 {
            return Err(ExplorationError::CitationValidation(format!(
                "source claim {} does not exist",
                edge.source_claim_id
            )));
        }

        // Validate target claim exists.
        let tgt_exists: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM claims WHERE id = ?1",
            params![edge.target_claim_id],
            |row| row.get(0),
        )?;
        if tgt_exists == 0 {
            return Err(ExplorationError::CitationValidation(format!(
                "target claim {} does not exist",
                edge.target_claim_id
            )));
        }

        self.conn.execute(
            "INSERT INTO claim_edges (id, source_claim_id, target_claim_id, edge_type, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                edge.id,
                edge.source_claim_id,
                edge.target_claim_id,
                edge.edge_type.to_string(),
                edge.created_at.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Syntheses
    // -----------------------------------------------------------------------

    /// Insert a synthesis with citation validation.
    ///
    /// Every ID in `cited_source_ids` must exist in the sources table.
    pub fn insert_synthesis(&self, synthesis: &Synthesis) -> Result<(), ExplorationError> {
        // Validate all cited source IDs exist.
        for source_id in &synthesis.cited_source_ids {
            let exists: i64 = self.conn.query_row(
                "SELECT COUNT(*) FROM sources WHERE id = ?1",
                params![source_id],
                |row| row.get(0),
            )?;
            if exists == 0 {
                return Err(ExplorationError::CitationValidation(format!(
                    "cited source {source_id} does not exist"
                )));
            }
        }

        // Validate run_id if provided.
        if let Some(ref run_id) = synthesis.run_id {
            let exists: i64 = self.conn.query_row(
                "SELECT COUNT(*) FROM research_runs WHERE id = ?1",
                params![run_id],
                |row| row.get(0),
            )?;
            if exists == 0 {
                return Err(ExplorationError::CitationValidation(format!(
                    "research run {run_id} does not exist"
                )));
            }
        }

        let cited_json = serde_json::to_string(&synthesis.cited_source_ids)
            .map_err(|e| ExplorationError::InvalidInput(e.to_string()))?;
        let unresolved_json = serde_json::to_string(&synthesis.unresolved_questions)
            .map_err(|e| ExplorationError::InvalidInput(e.to_string()))?;

        self.conn.execute(
            "INSERT INTO syntheses (id, run_id, conclusion, caveats, cited_source_ids, unresolved_questions, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                synthesis.id,
                synthesis.run_id,
                synthesis.conclusion,
                synthesis.caveats,
                cited_json,
                unresolved_json,
                synthesis.created_at.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Counts
    // -----------------------------------------------------------------------

    /// Count total sources.
    pub fn count_sources(&self) -> Result<usize, ExplorationError> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM sources", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    /// Count total claims.
    pub fn count_claims(&self) -> Result<usize, ExplorationError> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM claims", [], |row| row.get(0))?;
        Ok(count as usize)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn configure_connection(conn: &Connection) -> Result<(), ExplorationError> {
    conn.execute_batch(
        "PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;",
    )?;
    Ok(())
}

/// Parse an RFC 3339 datetime string into `DateTime<Utc>`.
fn parse_datetime(s: &str) -> Result<DateTime<Utc>, ExplorationError> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| ExplorationError::InvalidInput(format!("invalid timestamp: {e}")))
}

/// Escape a user query for safe use in FTS5 MATCH.
fn escape_fts_query(query: &str) -> String {
    query.replace(['/', '.', '"', '-'], " ")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_source(run_id: Option<&str>, title: &str) -> Source {
        Source {
            id: uuid::Uuid::new_v4().to_string(),
            run_id: run_id.map(String::from),
            url: Some("https://example.com/doc".to_string()),
            title: title.to_string(),
            authors: Some("Test Author".to_string()),
            publication_date: Some("2025-01-01".to_string()),
            fetched_at: Utc::now(),
            license_notes: None,
            content_hash: "abc123".to_string(),
        }
    }

    fn make_chunk(source_id: &str, ordinal: i64, text: &str) -> SourceChunk {
        SourceChunk {
            id: uuid::Uuid::new_v4().to_string(),
            source_id: source_id.to_string(),
            chunk_ordinal: ordinal,
            text: text.to_string(),
            token_estimate: Some(text.len() as i64 / 4),
        }
    }

    fn make_claim(run_id: Option<&str>, chunk_id: Option<&str>, text: &str) -> Claim {
        Claim {
            id: uuid::Uuid::new_v4().to_string(),
            run_id: run_id.map(String::from),
            source_chunk_id: chunk_id.map(String::from),
            normalized_text: text.to_string(),
            confidence: 0.8,
            status: "active".to_string(),
            freshness: "current".to_string(),
            created_at: Utc::now(),
        }
    }

    fn make_synthesis(run_id: Option<&str>, cited: Vec<String>, conclusion: &str) -> Synthesis {
        Synthesis {
            id: uuid::Uuid::new_v4().to_string(),
            run_id: run_id.map(String::from),
            conclusion: conclusion.to_string(),
            caveats: None,
            cited_source_ids: cited,
            unresolved_questions: vec![],
            created_at: Utc::now(),
        }
    }

    #[test]
    fn schema_migration_creates_tables() {
        let store = ExplorationStore::open_memory().unwrap();

        let tables = [
            "schema_version",
            "research_runs",
            "sources",
            "source_chunks",
            "source_chunks_fts",
            "claims",
            "claim_edges",
            "syntheses",
        ];

        for table in &tables {
            let count: i64 = store
                .conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type IN ('table', 'view') AND name = ?1",
                    params![table],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "Table {table} should exist");
        }
    }

    #[test]
    fn source_chunk_round_trip() {
        let mut store = ExplorationStore::open_memory().unwrap();

        let run = store.create_run("test query", Some("test plan")).unwrap();
        let source = make_source(Some(&run.id), "Test Document");
        store.insert_source(&source).unwrap();

        let chunk = make_chunk(&source.id, 0, "Rust is a safe systems programming language");
        store.insert_chunk(&chunk).unwrap();

        let results = store.search_chunks("Rust safe programming", 10).unwrap();
        assert!(!results.is_empty(), "FTS should find the chunk");
        assert_eq!(results[0].chunk_id, chunk.id);
    }

    #[test]
    fn fts_search_returns_bounded_results() {
        let mut store = ExplorationStore::open_memory().unwrap();

        let source = make_source(None, "Multi-chunk source");
        store.insert_source(&source).unwrap();

        for i in 0..5 {
            let chunk = make_chunk(
                &source.id,
                i,
                &format!("This is chunk number {i} about testing FTS search limits"),
            );
            store.insert_chunk(&chunk).unwrap();
        }

        let results = store.search_chunks("testing FTS search", 3).unwrap();
        assert!(
            results.len() <= 3,
            "Should respect limit of 3, got {}",
            results.len()
        );
    }

    #[test]
    fn claim_citation_validation_fails_on_missing_chunk() {
        let store = ExplorationStore::open_memory().unwrap();

        let claim = make_claim(None, Some("nonexistent-chunk-id"), "some claim");
        let err = store.insert_claim(&claim).unwrap_err();
        assert!(
            matches!(err, ExplorationError::CitationValidation(_)),
            "Expected CitationValidation error, got: {err}"
        );
    }

    #[test]
    fn synthesis_citation_validation_fails_on_missing_source() {
        let store = ExplorationStore::open_memory().unwrap();

        let synthesis = make_synthesis(
            None,
            vec!["nonexistent-source-id".to_string()],
            "some conclusion",
        );
        let err = store.insert_synthesis(&synthesis).unwrap_err();
        assert!(
            matches!(err, ExplorationError::CitationValidation(_)),
            "Expected CitationValidation error, got: {err}"
        );
    }

    #[test]
    fn claim_edge_insert_validates_both_claims() {
        let mut store = ExplorationStore::open_memory().unwrap();

        // Create one valid claim.
        let source = make_source(None, "Edge test source");
        store.insert_source(&source).unwrap();
        let chunk = make_chunk(&source.id, 0, "test text");
        store.insert_chunk(&chunk).unwrap();
        let valid_claim = make_claim(None, Some(&chunk.id), "valid claim");
        store.insert_claim(&valid_claim).unwrap();

        // Edge with nonexistent source claim.
        let edge_bad_src = ClaimEdge {
            id: uuid::Uuid::new_v4().to_string(),
            source_claim_id: "nonexistent-src".to_string(),
            target_claim_id: valid_claim.id.clone(),
            edge_type: EdgeType::Supports,
            created_at: Utc::now(),
        };
        let err = store.insert_claim_edge(&edge_bad_src).unwrap_err();
        assert!(
            matches!(err, ExplorationError::CitationValidation(_)),
            "Expected CitationValidation for bad source claim, got: {err}"
        );

        // Edge with nonexistent target claim.
        let edge_bad_tgt = ClaimEdge {
            id: uuid::Uuid::new_v4().to_string(),
            source_claim_id: valid_claim.id.clone(),
            target_claim_id: "nonexistent-tgt".to_string(),
            edge_type: EdgeType::Contradicts,
            created_at: Utc::now(),
        };
        let err = store.insert_claim_edge(&edge_bad_tgt).unwrap_err();
        assert!(
            matches!(err, ExplorationError::CitationValidation(_)),
            "Expected CitationValidation for bad target claim, got: {err}"
        );
    }

    #[test]
    fn full_research_run_round_trip() {
        let mut store = ExplorationStore::open_memory().unwrap();

        // Create run.
        let run = store
            .create_run(
                "What is Rust's ownership model?",
                Some("Research Rust ownership"),
            )
            .unwrap();

        // Insert source.
        let source = make_source(Some(&run.id), "The Rust Book");
        store.insert_source(&source).unwrap();

        // Insert chunks.
        let chunk1 = make_chunk(
            &source.id,
            0,
            "Rust uses ownership to manage memory without a garbage collector",
        );
        let chunk2 = make_chunk(
            &source.id,
            1,
            "Borrowing allows references without transferring ownership",
        );
        store.insert_chunk(&chunk1).unwrap();
        store.insert_chunk(&chunk2).unwrap();

        // Insert claims.
        let claim1 = make_claim(
            Some(&run.id),
            Some(&chunk1.id),
            "Rust manages memory via ownership",
        );
        let claim2 = make_claim(
            Some(&run.id),
            Some(&chunk2.id),
            "Borrowing enables safe references",
        );
        store.insert_claim(&claim1).unwrap();
        store.insert_claim(&claim2).unwrap();

        // Insert claim edge.
        let edge = ClaimEdge {
            id: uuid::Uuid::new_v4().to_string(),
            source_claim_id: claim1.id.clone(),
            target_claim_id: claim2.id.clone(),
            edge_type: EdgeType::Refines,
            created_at: Utc::now(),
        };
        store.insert_claim_edge(&edge).unwrap();

        // Insert synthesis.
        let synthesis = make_synthesis(
            Some(&run.id),
            vec![source.id.clone()],
            "Rust's ownership model eliminates garbage collection",
        );
        store.insert_synthesis(&synthesis).unwrap();

        // Verify counts.
        assert_eq!(store.count_sources().unwrap(), 1);
        assert_eq!(store.count_claims().unwrap(), 2);

        // Verify source retrievable.
        let retrieved_source = store.get_source(&source.id).unwrap();
        assert!(retrieved_source.is_some());
        assert_eq!(retrieved_source.unwrap().title, "The Rust Book");

        // Verify FTS search works.
        let search_results = store.search_chunks("ownership memory", 10).unwrap();
        assert!(!search_results.is_empty());
    }

    #[test]
    fn open_existing_db_idempotent() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("exploration.db");

        // First open: insert data.
        {
            let store = ExplorationStore::open(&db_path).unwrap();
            let run = store.create_run("persistent query", None).unwrap();
            let source = make_source(Some(&run.id), "Persistent Source");
            store.insert_source(&source).unwrap();
            assert_eq!(store.count_sources().unwrap(), 1);
        }

        // Reopen: verify data persists.
        {
            let store = ExplorationStore::open(&db_path).unwrap();
            assert_eq!(store.count_sources().unwrap(), 1);
            let _source = store.get_source("persistent-source").unwrap();
            let sources_count = store.count_sources().unwrap();
            assert_eq!(sources_count, 1, "Data should persist across reopen");
        }
    }
}
