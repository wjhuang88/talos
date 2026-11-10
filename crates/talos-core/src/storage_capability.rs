//! Storage and retrieval capability classification (STORE-002 / I321 B).
//!
//! This module is **descriptive and additive**. It introduces no storage engine,
//! adapter, provider registry, runtime binding or persistence operation. A
//! descriptor advertised through the existing capability registry is metadata,
//! not evidence that an operational storage implementation is bound. This
//! module intentionally contains **no** `Bound`/`is_bound` abstraction: the
//! operational binding API is deferred to domain-owned STORE-002 C/D work.
//!
//! Real Session/Memory/Evolution/Exploration contracts and existing adapters
//! remain domain-owned and are introduced in separately governed STORE-002
//! slices. No current consumer changes behavior by importing this module.

use crate::capability::CapabilityRequest;

/// One infrastructure capability family, distinct from any domain Store API.
///
/// Semantic retrieval is a composition of these capabilities, not an eighth
/// standalone storage engine. The set deliberately covers currently unbound
/// capabilities so their unavailability can be queried explicitly using the
/// existing capability registry.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StorageCapabilityKind {
    /// Ordered durable record append and replay (existing TLOG/JSONL Session stores).
    Log,
    /// Structured authoritative records and transactions (currently bundled SQLite).
    Relational,
    /// Object/blob persistence (no shared object provider currently bound).
    Object,
    /// Ranked lexical search (currently domain-owned SQLite FTS5).
    LexicalSearch,
    /// Nearest-neighbor vector indexing and retrieval (currently unbound).
    VectorSearch,
    /// Content to embedding conversion (currently unbound in this layer).
    Embedding,
    /// Independent candidate reranking (no standalone provider currently bound).
    Rerank,
}

impl StorageCapabilityKind {
    /// The complete seven-family initial storage contract taxonomy.
    pub const ALL: [Self; 7] = [
        Self::Log,
        Self::Relational,
        Self::Object,
        Self::LexicalSearch,
        Self::VectorSearch,
        Self::Embedding,
        Self::Rerank,
    ];

    /// Stable, storage-specific capability ID for the *existing* descriptor registry.
    ///
    /// No registry or provider identity is created by this operation.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Log => "storage.log",
            Self::Relational => "storage.relational",
            Self::Object => "storage.object",
            Self::LexicalSearch => "storage.lexical-search",
            Self::VectorSearch => "storage.vector-search",
            Self::Embedding => "storage.embedding",
            Self::Rerank => "storage.rerank",
        }
    }

    /// Parse a known ID, returning `None` for unknown future or unqualified IDs.
    ///
    /// Unknown IDs never fall back to relational or lexical capabilities.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "storage.log" => Some(Self::Log),
            "storage.relational" => Some(Self::Relational),
            "storage.object" => Some(Self::Object),
            "storage.lexical-search" => Some(Self::LexicalSearch),
            "storage.vector-search" => Some(Self::VectorSearch),
            "storage.embedding" => Some(Self::Embedding),
            "storage.rerank" => Some(Self::Rerank),
            _ => None,
        }
    }

    /// Create a typed request for the existing descriptor registry.
    ///
    /// Resolution checks only advertised descriptor compatibility. It does
    /// **not** produce a bound, operational storage provider or perform I/O.
    #[must_use]
    pub fn request(self, required_version: impl Into<String>) -> CapabilityRequest {
        CapabilityRequest {
            capability_id: self.id().into(),
            version: required_version.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::{CapabilityRegistry, ResolutionResult};
    use std::collections::BTreeSet;

    #[test]
    fn all_seven_capability_ids_are_distinct_and_round_trip() {
        let mut ids = BTreeSet::new();
        for kind in StorageCapabilityKind::ALL {
            assert!(ids.insert(kind.id()), "duplicate storage capability ID");
            assert_eq!(StorageCapabilityKind::from_id(kind.id()), Some(kind));
        }
        assert_eq!(ids.len(), 7);
    }

    #[test]
    fn unknown_and_unqualified_ids_fail_closed() {
        for unknown in [
            "",
            "log",
            "storage",
            "storage.vector",
            "storage.semantic-retrieval",
            "storage.VectorSearch",
            "storage.sql",
            "text.search",
        ] {
            assert_eq!(StorageCapabilityKind::from_id(unknown), None);
        }
    }

    #[test]
    fn empty_existing_registry_reports_all_categories_unavailable() {
        let registry = CapabilityRegistry::default();
        for kind in StorageCapabilityKind::ALL {
            let request = kind.request("1.0.0");
            assert_eq!(request.capability_id, kind.id());
            assert_eq!(request.version, "1.0.0");
            assert_eq!(registry.resolve(&request), ResolutionResult::Unavailable);
        }
    }

}
