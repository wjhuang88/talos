//! SQLite-based full-text search index for session messages.
//!
//! This module provides a [`SessionIndex`] that wraps a SQLite database with FTS5
//! support, enabling efficient full-text search across session messages. The index
//! is supplementary to the primary JSONL storage — it is created lazily on first use
//! and updated whenever sessions are saved.
//!
//! # Schema
//!
//! - `sessions` table: metadata about indexed sessions
//! - `messages_fts` FTS5 virtual table: full-text searchable message content

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension, Result as RusqliteResult};
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

use crate::{Session, SessionInfo};

/// Errors that can occur during SQLite index operations.
#[derive(Debug, Error)]
pub enum IndexError {
    /// A rusqlite error occurred.
    #[error("SQLite error: {0}")]
    SqliteError(#[from] rusqlite::Error),

    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Failed to parse a UUID.
    #[error("invalid UUID: {0}")]
    InvalidUuid(String),
}

/// A search result from the FTS5 index.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The session ID that contains the matching message.
    pub session_id: String,

    /// The project name associated with the session.
    pub project: String,

    /// A snippet of the matching content with highlighted matches.
    pub snippet: String,

    /// The timestamp of the matching message.
    pub timestamp: DateTime<Utc>,

    /// The BM25 rank of the match (lower is more relevant).
    pub rank: f64,
}

/// SQLite-based full-text search index for session messages.
///
/// Wraps a [`Connection`] to a SQLite database with FTS5 support. The index is
/// created lazily on first use and must be explicitly updated when sessions change.
#[derive(Debug)]
pub struct SessionIndex {
    conn: Connection,
    db_path: PathBuf,
}

impl SessionIndex {
    /// Open or create a SQLite database at the given path.
    ///
    /// The database file is created if it does not exist. The schema is NOT
    /// automatically initialized — call [`SessionIndex::init_schema`] after creation.
    ///
    /// # Arguments
    ///
    /// * `path` — Path to the SQLite database file.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be opened.
    pub fn new(path: &Path) -> Result<Self, IndexError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;

        Ok(Self {
            conn,
            db_path: path.to_path_buf(),
        })
    }

    /// Initialize the database schema.
    ///
    /// Creates the `sessions` table and the `messages_fts` FTS5 virtual table
    /// if they do not already exist. Safe to call multiple times.
    ///
    /// # Errors
    ///
    /// Returns an error if the schema cannot be created.
    pub fn init_schema(&self) -> Result<(), IndexError> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                project TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                message_count INTEGER NOT NULL DEFAULT 0
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
                session_id,
                role,
                content,
                timestamp
            );
            "#,
        )?;

        Ok(())
    }

    /// Index a session and all its messages.
    ///
    /// Upserts the session metadata into the `sessions` table and inserts all
    /// entries into the `messages_fts` table. Existing entries for the same session
    /// are deleted before re-indexing to avoid duplicates.
    ///
    /// # Arguments
    ///
    /// * `session` — The session to index.
    ///
    /// # Errors
    ///
    /// Returns an error if the index cannot be updated.
    pub fn index_session(&mut self, session: &Session) -> Result<(), IndexError> {
        let entries = session.read_entries().map_err(|e| {
            IndexError::IoError(std::io::Error::other(e.to_string()))
        })?;

        let tx = self.conn.transaction()?;

        tx.execute(
            "DELETE FROM messages_fts WHERE session_id = ?1",
            params![session.id.to_string()],
        )?;

        let created_at = session.created_at.to_rfc3339();
        let updated_at = entries
            .last()
            .map(|e| e.timestamp.to_rfc3339())
            .unwrap_or_else(|| Utc::now().to_rfc3339());
        let message_count = entries.len() as i64;

        tx.execute(
            r#"
            INSERT INTO sessions (id, project, created_at, updated_at, message_count)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(id) DO UPDATE SET
                project = excluded.project,
                updated_at = excluded.updated_at,
                message_count = excluded.message_count
            "#,
            params![
                session.id.to_string(),
                session.project,
                created_at,
                updated_at,
                message_count,
            ],
        )?;

        {
            let mut stmt = tx.prepare(
                "INSERT INTO messages_fts (session_id, role, content, timestamp) VALUES (?1, ?2, ?3, ?4)",
            )?;

            for entry in &entries {
                stmt.execute(params![
                    session.id.to_string(),
                    entry.role,
                    entry.content,
                    entry.timestamp.to_rfc3339(),
                ])?;
            }
        }

        tx.commit()?;

        Ok(())
    }

    /// Perform a full-text search across all indexed messages.
    ///
    /// Returns results ranked by relevance (BM25), with the most relevant first.
    /// Each result includes a snippet with matching terms highlighted.
    ///
    /// # Arguments
    ///
    /// * `query` — The FTS5 search query. Supports FTS5 syntax (e.g., `"exact phrase"`, `term1 OR term2`).
    /// * `limit` — Maximum number of results to return.
    ///
    /// # Errors
    ///
    /// Returns an error if the search cannot be executed.
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, IndexError> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                f.session_id,
                s.project,
                snippet(messages_fts, 2, '<b>', '</b>', '...', 32) AS snippet,
                f.timestamp,
                bm25(messages_fts) AS rank
            FROM messages_fts f
            LEFT JOIN sessions s ON f.session_id = s.id
            WHERE messages_fts MATCH ?1
            ORDER BY rank ASC
            LIMIT ?2
            "#,
        )?;

        let results = stmt
            .query_map(params![query, limit as i64], |row| {
                let session_id: String = row.get(0)?;
                let project: String = row.get(1).unwrap_or_else(|_| "unknown".to_string());
                let snippet: String = row.get(2)?;
                let timestamp_str: String = row.get(3)?;
                let rank: f64 = row.get(4)?;

                let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(SearchResult {
                    session_id,
                    project,
                    snippet,
                    timestamp,
                    rank,
                })
            })?
            .collect::<RusqliteResult<Vec<_>>>()?;

        Ok(results)
    }

    /// List the most recently updated sessions.
    ///
    /// Returns sessions ordered by `updated_at` descending.
    ///
    /// # Arguments
    ///
    /// * `limit` — Maximum number of sessions to return.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub fn list_recent(&self, limit: usize) -> Result<Vec<SessionInfo>, IndexError> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, project, message_count, updated_at
            FROM sessions
            ORDER BY updated_at DESC
            LIMIT ?1
            "#,
        )?;

        let results = stmt
            .query_map(params![limit as i64], |row| {
                let id_str: String = row.get(0)?;
                let project: String = row.get(1)?;
                let message_count: i64 = row.get(2)?;
                let updated_at_str: String = row.get(3)?;

                let id = Uuid::parse_str(&id_str)
                    .map_err(|_e| rusqlite::Error::InvalidColumnType(0, "uuid".to_string(), rusqlite::types::Type::Text))?;

                let timestamp = DateTime::parse_from_rfc3339(&updated_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(SessionInfo {
                    id,
                    project,
                    last_message_preview: String::new(),
                    timestamp,
                    message_count: message_count as usize,
                })
            })?
            .collect::<RusqliteResult<Vec<_>>>()?;

        Ok(results)
    }

    /// Get metadata for a specific session.
    ///
    /// Returns `None` if the session is not in the index.
    ///
    /// # Arguments
    ///
    /// * `session_id` — The session ID to look up.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub fn get_session_info(&self, session_id: &str) -> Result<Option<SessionInfo>, IndexError> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, project, message_count, updated_at
            FROM sessions
            WHERE id = ?1
            "#,
        )?;

        let result = stmt
            .query_row(params![session_id], |row| {
                let id_str: String = row.get(0)?;
                let project: String = row.get(1)?;
                let message_count: i64 = row.get(2)?;
                let updated_at_str: String = row.get(3)?;

                let id = Uuid::parse_str(&id_str)
                    .map_err(|_e| rusqlite::Error::InvalidColumnType(0, "uuid".to_string(), rusqlite::types::Type::Text))?;

                let timestamp = DateTime::parse_from_rfc3339(&updated_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(SessionInfo {
                    id,
                    project,
                    last_message_preview: String::new(),
                    timestamp,
                    message_count: message_count as usize,
                })
            })
            .optional()?;

        Ok(result)
    }

    /// Return the path to the SQLite database file.
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Session, SessionManager};
    use chrono::Datelike;
    use talos_core::message::Message;

    fn temp_index() -> (SessionIndex, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test_index.db");
        let index = SessionIndex::new(&db_path).unwrap();
        index.init_schema().unwrap();
        (index, dir)
    }

    fn test_session(manager: &SessionManager) -> Session {
        let session = manager.create_session("test-project").unwrap();
        session
            .append(&Message::User {
                content: "Hello, how do I implement full-text search in Rust?".into(),
            })
            .unwrap();
        session
            .append(&Message::Assistant {
                content: "You can use SQLite with FTS5 extension. It provides efficient full-text indexing.".into(),
                tool_calls: vec![],
            })
            .unwrap();
        session
            .append(&Message::User {
                content: "What about ranking and relevance?".into(),
            })
            .unwrap();
        session
            .append(&Message::Assistant {
                content: "FTS5 uses BM25 ranking by default. Lower scores indicate more relevant matches.".into(),
                tool_calls: vec![],
            })
            .unwrap();
        session
    }

    #[test]
    fn test_schema_creation() {
        let (index, _dir) = temp_index();

        let table_count: i64 = index
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type IN ('table', 'view') AND name IN ('sessions', 'messages_fts')",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(table_count, 2, "Both sessions and messages_fts tables should exist");
    }

    #[test]
    fn test_session_indexing() {
        let manager = SessionManager::with_dir(tempfile::tempdir().unwrap().path().to_path_buf());
        let session = test_session(&manager);

        let (mut index, _dir) = temp_index();
        index.index_session(&session).unwrap();

        let info = index.get_session_info(&session.id.to_string()).unwrap();
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.project, "test-project");
        assert_eq!(info.message_count, 4);
    }

    #[test]
    fn test_full_text_search_basic() {
        let manager = SessionManager::with_dir(tempfile::tempdir().unwrap().path().to_path_buf());
        let session = test_session(&manager);

        let (mut index, _dir) = temp_index();
        index.index_session(&session).unwrap();

        let results = index.search("SQLite", 10).unwrap();
        assert!(!results.is_empty(), "Should find results for 'SQLite'");

        for result in &results {
            assert_eq!(result.session_id, session.id.to_string());
        }
    }

    #[test]
    fn test_full_text_search_bm25() {
        let manager = SessionManager::with_dir(tempfile::tempdir().unwrap().path().to_path_buf());
        let session = test_session(&manager);

        let (mut index, _dir) = temp_index();
        index.index_session(&session).unwrap();

        let results = index.search("ranking BM25", 10).unwrap();
        assert!(!results.is_empty(), "Should find results for 'ranking BM25'");

        for i in 1..results.len() {
            assert!(
                results[i].rank >= results[i - 1].rank,
                "Results should be ordered by rank ascending"
            );
        }
    }

    #[test]
    fn test_full_text_search_no_results() {
        let manager = SessionManager::with_dir(tempfile::tempdir().unwrap().path().to_path_buf());
        let session = test_session(&manager);

        let (mut index, _dir) = temp_index();
        index.index_session(&session).unwrap();

        let results = index.search("nonexistent_term_xyz_12345", 10).unwrap();
        assert!(results.is_empty(), "Should return empty results for non-matching query");
    }

    #[test]
    fn test_full_text_search_with_limit() {
        let manager = SessionManager::with_dir(tempfile::tempdir().unwrap().path().to_path_buf());
        let session = test_session(&manager);

        let (mut index, _dir) = temp_index();
        index.index_session(&session).unwrap();

        let results_all = index.search("session", 100).unwrap();
        let results_limited = index.search("session", 2).unwrap();

        assert!(results_limited.len() <= 2, "Should respect limit");
        assert!(results_limited.len() <= results_all.len(), "Limited results should not exceed all results");
    }

    #[test]
    fn test_search_result_snippet() {
        let manager = SessionManager::with_dir(tempfile::tempdir().unwrap().path().to_path_buf());
        let session = test_session(&manager);

        let (mut index, _dir) = temp_index();
        index.index_session(&session).unwrap();

        let results = index.search("Rust", 10).unwrap();
        assert!(!results.is_empty(), "Should find results for 'Rust'");

        let found_highlight = results.iter().any(|r| r.snippet.contains("<b>"));
        assert!(found_highlight, "Snippet should contain highlighted matches");
    }

    #[test]
    fn test_list_recent_ordering() {
        let manager = SessionManager::with_dir(tempfile::tempdir().unwrap().path().to_path_buf());

        let session1 = manager.create_session("project-alpha").unwrap();
        session1
            .append(&Message::User {
                content: "First session message".into(),
            })
            .unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10));

        let session2 = manager.create_session("project-beta").unwrap();
        session2
            .append(&Message::User {
                content: "Second session message".into(),
            })
            .unwrap();

        let (mut index, _dir) = temp_index();
        index.index_session(&session1).unwrap();
        index.index_session(&session2).unwrap();

        let recent = index.list_recent(10).unwrap();
        assert_eq!(recent.len(), 2, "Should return both sessions");

        assert_eq!(recent[0].id, session2.id, "Most recent session should be first");
        assert_eq!(recent[1].id, session1.id, "Older session should be second");
    }

    #[test]
    fn test_list_recent_with_limit() {
        let manager = SessionManager::with_dir(tempfile::tempdir().unwrap().path().to_path_buf());
        let db_path = tempfile::tempdir().unwrap().path().join("test_limit.db");
        let mut index = SessionIndex::new(&db_path).unwrap();
        index.init_schema().unwrap();

        for i in 0..5 {
            let session = manager.create_session(&format!("project-limit-{i}")).unwrap();
            session
                .append(&Message::User {
                    content: format!("Message in project {i}"),
                })
                .unwrap();
            index.index_session(&session).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        let limited = index.list_recent(3).unwrap();
        assert_eq!(limited.len(), 3, "Should respect limit");
    }

    #[test]
    fn test_get_session_info_existing() {
        let manager = SessionManager::with_dir(tempfile::tempdir().unwrap().path().to_path_buf());
        let session = test_session(&manager);

        let (mut index, _dir) = temp_index();
        index.index_session(&session).unwrap();

        let info = index.get_session_info(&session.id.to_string()).unwrap();
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.id, session.id);
        assert_eq!(info.project, "test-project");
        assert_eq!(info.message_count, 4);
    }

    #[test]
    fn test_get_session_info_nonexistent() {
        let (index, _dir) = temp_index();

        let info = index.get_session_info("nonexistent-id").unwrap();
        assert!(info.is_none(), "Should return None for non-existent session");
    }

    #[test]
    fn test_index_session_upsert() {
        let manager = SessionManager::with_dir(tempfile::tempdir().unwrap().path().to_path_buf());
        let session = manager.create_session("test-project").unwrap();

        session
            .append(&Message::User {
                content: "Initial message".into(),
            })
            .unwrap();

        let (mut index, _dir) = temp_index();
        index.index_session(&session).unwrap();

        session
            .append(&Message::Assistant {
                content: "Reply to initial message".into(),
                tool_calls: vec![],
            })
            .unwrap();
        index.index_session(&session).unwrap();

        let info = index.get_session_info(&session.id.to_string()).unwrap();
        assert!(info.is_some());
        assert_eq!(info.unwrap().message_count, 2, "Message count should be updated");

        let results = index.search("message", 100).unwrap();
        let unique_count = results.iter().filter(|r| r.session_id == session.id.to_string()).count();
        assert_eq!(unique_count, 2, "Should not have duplicate entries");
    }

    #[test]
    fn test_search_with_phrase_query() {
        let manager = SessionManager::with_dir(tempfile::tempdir().unwrap().path().to_path_buf());
        let session = test_session(&manager);

        let (mut index, _dir) = temp_index();
        index.index_session(&session).unwrap();

        let results = index.search("\"BM25 ranking\"", 10).unwrap();
        assert!(!results.is_empty(), "Should find exact phrase match");
    }

    #[test]
    fn test_search_result_timestamp() {
        let manager = SessionManager::with_dir(tempfile::tempdir().unwrap().path().to_path_buf());
        let session = test_session(&manager);

        let (mut index, _dir) = temp_index();
        index.index_session(&session).unwrap();

        let results = index.search("Rust", 10).unwrap();
        assert!(!results.is_empty());

        for result in &results {
            assert!(result.timestamp.year() > 2020, "Timestamp should be reasonable");
        }
    }

    #[test]
    fn test_db_path() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test_path.db");
        let index = SessionIndex::new(&db_path).unwrap();

        assert_eq!(index.db_path(), db_path);
    }
}
