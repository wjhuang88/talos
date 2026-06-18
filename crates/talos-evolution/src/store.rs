//! KnowledgeStore — SQLite persistence for observations and patterns.

use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};

use crate::EvolutionResult as Result;
use crate::{Observation, Pattern, SignalType};

/// SQLite-backed store for evolution data.
pub struct KnowledgeStore {
    conn: Connection,
}

impl KnowledgeStore {
    /// Open or create a knowledge store at the given path.
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    /// Open an in-memory knowledge store for testing.
    pub fn open_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS observations (
                id TEXT PRIMARY KEY,
                signal_type TEXT NOT NULL,
                intensity REAL NOT NULL,
                context TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                session_id TEXT,
                turn_number INTEGER
            );

            CREATE TABLE IF NOT EXISTS patterns (
                id TEXT PRIMARY KEY,
                description TEXT NOT NULL,
                instruction TEXT NOT NULL,
                confidence REAL NOT NULL,
                evidence_count INTEGER NOT NULL,
                first_observed TEXT NOT NULL,
                last_updated TEXT NOT NULL,
                category TEXT NOT NULL,
                active INTEGER NOT NULL DEFAULT 1,
                content_hash TEXT NOT NULL DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS conflicts (
                id TEXT PRIMARY KEY,
                pattern_a_id TEXT NOT NULL,
                pattern_b_id TEXT NOT NULL,
                description TEXT NOT NULL,
                detected_at TEXT NOT NULL,
                resolved INTEGER NOT NULL DEFAULT 0,
                winner_id TEXT
            );

            -- I021-S1: MenteDB-aligned tables
            CREATE TABLE IF NOT EXISTS signals (
                id TEXT PRIMARY KEY,
                kind TEXT NOT NULL,
                intensity REAL NOT NULL,
                context TEXT NOT NULL,
                tool_name TEXT,
                turn_observation_id TEXT REFERENCES turn_observations(id),
                timestamp TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS turn_observations (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                turn_number INTEGER NOT NULL,
                outcome TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                timestamp TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER NOT NULL
            );",
        )?;

        // I021-S4: Detect v1 schema (missing pattern.key) and hard-reset if needed.
        let has_key_column: i64 = self
            .conn
            .prepare("SELECT COUNT(*) FROM pragma_table_info('patterns') WHERE name = 'key'")?
            .query_row([], |row| row.get(0))?;

        if has_key_column == 0 {
            let obs_count: i64 = self
                .conn
                .query_row("SELECT COUNT(*) FROM observations", [], |row| row.get(0))
                .unwrap_or(0);
            let pat_count: i64 = self
                .conn
                .query_row("SELECT COUNT(*) FROM patterns", [], |row| row.get(0))
                .unwrap_or(0);

            if obs_count > 0 || pat_count > 0 {
                let _ = self.conn.execute("DELETE FROM observations", []);
                let _ = self.conn.execute("DELETE FROM patterns", []);
                let _ = self.conn.execute("DELETE FROM conflicts", []);
                let _ = self.conn.execute("DELETE FROM signals", []);
                let _ = self.conn.execute("DELETE FROM turn_observations", []);
                tracing::warn!(
                    observations = obs_count,
                    patterns = pat_count,
                    "knowledge.db schema migration: hard-reset, removed observations and patterns"
                );
            }
        }

        // Add content_hash column to existing databases (SQLite ALTER doesn't support IF NOT EXISTS).
        // We catch the "duplicate column" error to make this idempotent.
        let has_column = self
            .conn
            .prepare(
                "SELECT COUNT(*) FROM pragma_table_info('patterns') WHERE name = 'content_hash'",
            )?
            .query_row([], |row| row.get::<_, i64>(0))?;
        if has_column == 0 {
            let _ = self.conn.execute(
                "ALTER TABLE patterns ADD COLUMN content_hash TEXT NOT NULL DEFAULT ''",
                [],
            );
        }

        // I021-S3: Add MenteDB-aligned columns to patterns table.
        for (col, default) in [
            ("key", "''"),
            ("value", "'null'"),
            ("contradicting_count", "0"),
            ("last_reinforced", "''"),
            ("source_sessions", "'[]'"),
        ] {
            let has_col: i64 = self
                .conn
                .prepare(&format!(
                    "SELECT COUNT(*) FROM pragma_table_info('patterns') WHERE name = '{col}'"
                ))?
                .query_row([], |row| row.get(0))?;
            if has_col == 0 {
                let _ = self.conn.execute(
                    &format!(
                        "ALTER TABLE patterns ADD COLUMN {col} TEXT NOT NULL DEFAULT {default}"
                    ),
                    [],
                );
            }
        }

        // Initialize schema_version if empty (new database).
        let version_count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM schema_version", [], |row| row.get(0))?;
        if version_count == 0 {
            let _ = self
                .conn
                .execute("INSERT INTO schema_version (version) VALUES (2)", []);
        }

        Ok(())
    }

    /// Insert an observation.
    pub fn insert_observation(&self, obs: &Observation) -> Result<()> {
        self.conn.execute(
            "INSERT INTO observations (id, signal_type, intensity, context, timestamp, session_id, turn_number)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                obs.id,
                format!("{:?}", obs.signal_type),
                obs.intensity,
                obs.context,
                obs.timestamp.to_rfc3339(),
                obs.session_id,
                obs.turn_number,
            ],
        )?;
        Ok(())
    }

    /// Get all observations.
    pub fn get_observations(&self) -> Result<Vec<Observation>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, signal_type, intensity, context, timestamp, session_id, turn_number
             FROM observations ORDER BY timestamp DESC",
        )?;

        let observations = stmt
            .query_map([], |row| {
                let signal_type_str: String = row.get(1)?;
                let signal_type = match signal_type_str.as_str() {
                    "Correction" => SignalType::Correction,
                    "Error" => SignalType::Error,
                    "Satisfaction" => SignalType::Satisfaction,
                    "Inefficiency" => SignalType::Inefficiency,
                    _ => SignalType::Correction,
                };

                let timestamp_str: String = row.get(4)?;
                let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(Observation {
                    id: row.get(0)?,
                    signal_type,
                    intensity: row.get(2)?,
                    context: row.get(3)?,
                    timestamp,
                    session_id: row.get(5)?,
                    turn_number: row.get(6)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(observations)
    }

    /// Insert a pattern.
    pub fn insert_pattern(&self, pattern: &Pattern) -> Result<()> {
        let value_json = serde_json::to_string(&pattern.value).unwrap_or_else(|_| "null".into());
        let sessions_json =
            serde_json::to_string(&pattern.source_sessions).unwrap_or_else(|_| "[]".into());
        self.conn.execute(
            "INSERT INTO patterns (id, description, instruction, confidence, evidence_count, first_observed, last_updated, category, active, content_hash, key, value, contradicting_count, last_reinforced, source_sessions)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                pattern.id,
                pattern.description,
                pattern.instruction,
                pattern.confidence,
                pattern.evidence_count,
                pattern.first_observed.to_rfc3339(),
                pattern.last_updated.to_rfc3339(),
                pattern.category,
                pattern.active as i32,
                pattern.content_hash,
                pattern.key,
                value_json,
                pattern.contradicting_count as i32,
                pattern.last_reinforced.to_rfc3339(),
                sessions_json,
            ],
        )?;
        Ok(())
    }

    /// Get active patterns with confidence above threshold.
    pub fn get_active_patterns(&self, min_confidence: f64) -> Result<Vec<Pattern>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, description, instruction, confidence, evidence_count, first_observed, last_updated, category, active, content_hash, key, value, contradicting_count, last_reinforced, source_sessions
             FROM patterns WHERE active = 1 AND confidence >= ?1 ORDER BY confidence DESC",
        )?;

        let patterns = stmt
            .query_map(params![min_confidence], |row| {
                let first_observed_str: String = row.get(5)?;
                let first_observed = DateTime::parse_from_rfc3339(&first_observed_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                let last_updated_str: String = row.get(6)?;
                let last_updated = DateTime::parse_from_rfc3339(&last_updated_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                let key: String = row.get(10)?;
                let value_str: String = row.get(11)?;
                let value: serde_json::Value =
                    serde_json::from_str(&value_str).unwrap_or(serde_json::Value::Null);
                let contradicting_raw: String = row.get(12)?;
                let contradicting_count: u32 = contradicting_raw.parse().unwrap_or(0);

                let last_reinforced_str: String = row.get(13)?;
                let last_reinforced = if last_reinforced_str.is_empty() {
                    first_observed
                } else {
                    DateTime::parse_from_rfc3339(&last_reinforced_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or(first_observed)
                };

                let sessions_str: String = row.get(14)?;
                let source_sessions: Vec<uuid::Uuid> =
                    serde_json::from_str(&sessions_str).unwrap_or_default();

                Ok(Pattern {
                    id: row.get(0)?,
                    description: row.get(1)?,
                    instruction: row.get(2)?,
                    confidence: row.get(3)?,
                    evidence_count: row.get(4)?,
                    first_observed,
                    last_updated,
                    category: row.get(7)?,
                    active: row.get::<_, i32>(8)? == 1,
                    content_hash: row.get(9)?,
                    key,
                    value,
                    contradicting_count,
                    last_reinforced,
                    source_sessions,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(patterns)
    }

    /// Update pattern confidence and evidence count.
    pub fn update_pattern(&self, pattern: &Pattern) -> Result<()> {
        self.conn.execute(
            "UPDATE patterns SET confidence = ?1, evidence_count = ?2, last_updated = ?3
             WHERE id = ?4",
            params![
                pattern.confidence,
                pattern.evidence_count,
                pattern.last_updated.to_rfc3339(),
                pattern.id,
            ],
        )?;
        Ok(())
    }

    /// Deactivate a pattern.
    pub fn deactivate_pattern(&self, pattern_id: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE patterns SET active = 0 WHERE id = ?1",
            params![pattern_id],
        )?;
        Ok(())
    }

    /// Deactivate patterns whose instruction exceeds `max_bytes`. Returns count.
    pub fn delete_oversized_patterns(&self, max_bytes: usize) -> Result<usize> {
        let changes = self.conn.execute(
            "UPDATE patterns SET active = 0 WHERE length(instruction) > ?1",
            params![max_bytes as i64],
        )?;
        Ok(changes)
    }

    /// Get all patterns (including inactive).
    pub fn get_all_patterns(&self) -> Result<Vec<Pattern>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, description, instruction, confidence, evidence_count, first_observed, last_updated, category, active, content_hash, key, value, contradicting_count, last_reinforced, source_sessions
             FROM patterns ORDER BY confidence DESC",
        )?;

        let patterns = stmt
            .query_map([], |row| {
                let first_observed_str: String = row.get(5)?;
                let first_observed = DateTime::parse_from_rfc3339(&first_observed_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                let last_updated_str: String = row.get(6)?;
                let last_updated = DateTime::parse_from_rfc3339(&last_updated_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                let key: String = row.get(10)?;
                let value_str: String = row.get(11)?;
                let value: serde_json::Value =
                    serde_json::from_str(&value_str).unwrap_or(serde_json::Value::Null);
                let contradicting_raw: String = row.get(12)?;
                let contradicting_count: u32 = contradicting_raw.parse().unwrap_or(0);

                let last_reinforced_str: String = row.get(13)?;
                let last_reinforced = if last_reinforced_str.is_empty() {
                    first_observed
                } else {
                    DateTime::parse_from_rfc3339(&last_reinforced_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or(first_observed)
                };

                let sessions_str: String = row.get(14)?;
                let source_sessions: Vec<uuid::Uuid> =
                    serde_json::from_str(&sessions_str).unwrap_or_default();

                Ok(Pattern {
                    id: row.get(0)?,
                    description: row.get(1)?,
                    instruction: row.get(2)?,
                    confidence: row.get(3)?,
                    evidence_count: row.get(4)?,
                    first_observed,
                    last_updated,
                    category: row.get(7)?,
                    active: row.get::<_, i32>(8)? == 1,
                    content_hash: row.get(9)?,
                    key,
                    value,
                    contradicting_count,
                    last_reinforced,
                    source_sessions,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(patterns)
    }
}

#[cfg(test)]
#[allow(warnings)]
mod tests {
    use super::*;

    #[test]
    fn test_store_operations() {
        let store = KnowledgeStore::open_memory().unwrap();

        let obs = Observation::new(
            SignalType::Correction,
            0.8,
            "Use functional style".to_string(),
            Some("session-1".to_string()),
            Some(5),
        );
        store.insert_observation(&obs).unwrap();

        let observations = store.get_observations().unwrap();
        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0].signal_type, SignalType::Correction);

        let mut pattern = Pattern::new(
            "Prefer functional style".to_string(),
            "Use functional programming patterns".to_string(),
            "preference".to_string(),
        );
        pattern.confidence = 0.8;
        pattern.evidence_count = 3;
        store.insert_pattern(&pattern).unwrap();

        let patterns = store.get_active_patterns(0.7).unwrap();
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].confidence, 0.8);
    }

    #[test]
    fn test_update_pattern() {
        let store = KnowledgeStore::open_memory().unwrap();

        let mut pattern = Pattern::new(
            "Test pattern".to_string(),
            "Test instruction".to_string(),
            "test".to_string(),
        );
        pattern.confidence = 0.5;
        store.insert_pattern(&pattern).unwrap();

        pattern.confidence = 0.9;
        pattern.evidence_count = 10;
        store.update_pattern(&pattern).unwrap();

        let patterns = store.get_all_patterns().unwrap();
        assert_eq!(patterns[0].confidence, 0.9);
        assert_eq!(patterns[0].evidence_count, 10);
    }

    #[test]
    fn test_delete_oversized_patterns_deactivates_but_keeps_row() {
        let store = KnowledgeStore::open_memory().unwrap();

        let mut pattern = Pattern::new(
            "Big pattern".to_string(),
            "x".repeat(10_000),
            "test".to_string(),
        );
        pattern.confidence = 0.9;
        store.insert_pattern(&pattern).unwrap();

        let count = store.delete_oversized_patterns(4096).unwrap();
        assert_eq!(count, 1);

        let all = store.get_all_patterns().unwrap();
        assert_eq!(all.len(), 1);
        assert!(!all[0].active, "pattern should be deactivated, not deleted");
    }

    #[test]
    fn test_delete_oversized_patterns_returns_count() {
        let store = KnowledgeStore::open_memory().unwrap();

        for i in 0..3 {
            let mut pattern = Pattern::new(
                format!("pattern {i}"),
                "x".repeat(5000 + i * 1000),
                "test".to_string(),
            );
            pattern.confidence = 0.9;
            store.insert_pattern(&pattern).unwrap();
        }

        let count = store.delete_oversized_patterns(5500).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_pattern_roundtrip_preserves_content_hash() {
        let store = KnowledgeStore::open_memory().unwrap();

        let mut pattern = Pattern::new(
            "Test".to_string(),
            "Test instruction content".to_string(),
            "test".to_string(),
        );
        pattern.confidence = 0.8;
        let original_hash = pattern.content_hash.clone();
        store.insert_pattern(&pattern).unwrap();

        let patterns = store.get_active_patterns(0.0).unwrap();
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].content_hash, original_hash);
    }

    #[test]
    fn test_hard_reset_on_v1_schema_db() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("knowledge.db");

        // Create a v1-schema database (without the new MenteDB columns).
        {
            let conn = Connection::open(&db_path).unwrap();
            conn.execute_batch(
                "CREATE TABLE observations (
                    id TEXT PRIMARY KEY, signal_type TEXT NOT NULL, intensity REAL NOT NULL,
                    context TEXT NOT NULL, timestamp TEXT NOT NULL, session_id TEXT, turn_number INTEGER
                );
                CREATE TABLE patterns (
                    id TEXT PRIMARY KEY, description TEXT NOT NULL, instruction TEXT NOT NULL,
                    confidence REAL NOT NULL, evidence_count INTEGER NOT NULL,
                    first_observed TEXT NOT NULL, last_updated TEXT NOT NULL,
                    category TEXT NOT NULL, active INTEGER NOT NULL DEFAULT 1,
                    content_hash TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE conflicts (
                    id TEXT PRIMARY KEY, pattern_a_id TEXT NOT NULL, pattern_b_id TEXT NOT NULL,
                    description TEXT NOT NULL, detected_at TEXT NOT NULL,
                    resolved INTEGER NOT NULL DEFAULT 0, winner_id TEXT
                );",
            )
            .unwrap();

            // Insert v1 data.
            conn.execute(
                "INSERT INTO observations (id, signal_type, intensity, context, timestamp, session_id, turn_number)
                 VALUES ('v1-obs', 'Correction', 0.8, 'v1 context', '2026-01-01T00:00:00Z', 'sess-1', 1)",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO patterns (id, description, instruction, confidence, evidence_count, first_observed, last_updated, category, active, content_hash)
                 VALUES ('v1-pat', 'v1 pattern', 'v1 instruction', 0.9, 5, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', 'preference', 1, 'hash123')",
                [],
            )
            .unwrap();
        }

        // Open with new code — should detect v1 schema and hard-reset.
        let store = KnowledgeStore::open(db_path.to_str().unwrap()).expect("open v1 db");

        // Data should be wiped.
        let observations = store.get_observations().unwrap();
        assert!(
            observations.is_empty(),
            "v1 observations should be wiped, got {:?}",
            observations
        );
        let patterns = store.get_all_patterns().unwrap();
        assert!(
            patterns.is_empty(),
            "v1 patterns should be wiped, got {:?}",
            patterns
        );

        // Second open should be idempotent (no re-reset, schema is now v2).
        drop(store);
        let store2 = KnowledgeStore::open(db_path.to_str().unwrap()).expect("reopen v2 db");
        let observations2 = store2.get_observations().unwrap();
        assert!(observations2.is_empty(), "second open should not re-reset");
    }
}
