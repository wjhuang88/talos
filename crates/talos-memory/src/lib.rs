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

mod entities;
mod graph;
mod prompt;
mod store;
#[cfg(test)]
mod tests;
mod types;

pub use consolidation::{
    ConsolidationConfig, ConsolidationReport, EpisodeExtractor, MemoryCandidate,
    RuleBasedExtractor, SessionEpisode, consolidate_episodes,
};
pub use entities::extract_entities;
pub use graph::{AssociationResult, GraphEdge, GraphNode};
pub use prompt::{MemoryPromptConfig, format_memory_prompt};
pub use store::MemoryStore;
pub use types::{
    Entity, EntityKind, EvidenceLink, MemoryItem, MemoryKind, MemoryStatus, MemoryStoreError,
    RetentionCandidate, RetentionPolicy, RetrievalResult,
};
