//! Exploration domain entities.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
