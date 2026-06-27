use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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
