//! KnowledgeStore — SQLite persistence for observations and patterns.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};

use crate::{Observation, Pattern, SignalType};

/// SQLite-backed store for evolution data.
pub struct KnowledgeStore {
    conn: Connection,
}

impl KnowledgeStore {
    /// Open or create a knowledge store at the given path.
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path).context("failed to open knowledge store")?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    /// Open an in-memory knowledge store for testing.
    pub fn open_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().context("failed to open in-memory store")?;
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
                active INTEGER NOT NULL DEFAULT 1
            );

            CREATE TABLE IF NOT EXISTS conflicts (
                id TEXT PRIMARY KEY,
                pattern_a_id TEXT NOT NULL,
                pattern_b_id TEXT NOT NULL,
                description TEXT NOT NULL,
                detected_at TEXT NOT NULL,
                resolved INTEGER NOT NULL DEFAULT 0,
                winner_id TEXT
            );",
        )?;
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
            .collect::<Result<Vec<_>, _>>()?;

        Ok(observations)
    }

    /// Insert a pattern.
    pub fn insert_pattern(&self, pattern: &Pattern) -> Result<()> {
        self.conn.execute(
            "INSERT INTO patterns (id, description, instruction, confidence, evidence_count, first_observed, last_updated, category, active)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
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
            ],
        )?;
        Ok(())
    }

    /// Get active patterns with confidence above threshold.
    pub fn get_active_patterns(&self, min_confidence: f64) -> Result<Vec<Pattern>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, description, instruction, confidence, evidence_count, first_observed, last_updated, category, active
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
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

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

    /// Get all patterns (including inactive).
    pub fn get_all_patterns(&self) -> Result<Vec<Pattern>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, description, instruction, confidence, evidence_count, first_observed, last_updated, category, active
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
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(patterns)
    }
}

#[cfg(test)]
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
}
