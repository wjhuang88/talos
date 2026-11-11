//! Domain-owned semantic/procedural memory persistence contract (STORE-002 C).
//!
//! This trait describes the current MemoryStore operations. FTS5, evidence,
//! graph edges, add-only conflict preservation and ranking remain owned by
//! MemoryStore, not by a generic SQL or vector-search provider.

use crate::{
    AssociationResult, EvidenceLink, GraphEdge, GraphNode, MemoryItem, MemoryStore,
    MemoryStoreError, RetentionCandidate, RetentionPolicy, RetrievalResult,
};

/// Domain operations for persisted memories and their provenance.
///
/// A metadata-only storage capability descriptor is not an operational
/// implementation of this trait. The existing SQLite-backed store is the
/// only binding here; no default or caller is changed by this declaration.
pub trait MemoryPersistence {
    /// Preserve add-only semantics: false means an exact duplicate was ignored.
    fn insert(&mut self, item: MemoryItem) -> Result<bool, MemoryStoreError>;

    /// Attach source provenance to an existing memory item.
    fn insert_evidence(&self, link: EvidenceLink) -> Result<(), MemoryStoreError>;

    /// Return existing bounded, scored lexical retrieval results.
    fn retrieve(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<RetrievalResult>, MemoryStoreError>;

    /// Look up one memory by its stable domain identifier.
    fn get(&self, id: &str) -> Result<Option<MemoryItem>, MemoryStoreError>;

    /// Count authoritative memory records.
    fn count(&self) -> Result<usize, MemoryStoreError>;

    /// Upsert a domain graph node without treating it as a vector index.
    fn graph_upsert_node(&self, node: &GraphNode) -> Result<(), MemoryStoreError>;

    /// Upsert a domain graph edge preserving its evidence semantics.
    fn graph_upsert_edge(&self, edge: &GraphEdge) -> Result<(), MemoryStoreError>;

    /// Recall associated records within existing graph traversal bounds.
    fn graph_recall(
        &self,
        seed_ids: &[&str],
        max_hops: usize,
        min_edge_weight: f64,
        fanout: usize,
    ) -> Result<Vec<AssociationResult>, MemoryStoreError>;

    /// Select records eligible under the existing retention policy.
    fn retention_candidates(
        &self,
        policy: &RetentionPolicy,
    ) -> Result<Vec<RetentionCandidate>, MemoryStoreError>;
}

impl MemoryPersistence for MemoryStore {
    fn insert(&mut self, item: MemoryItem) -> Result<bool, MemoryStoreError> {
        MemoryStore::insert(self, item)
    }

    fn insert_evidence(&self, link: EvidenceLink) -> Result<(), MemoryStoreError> {
        MemoryStore::insert_evidence(self, link)
    }

    fn retrieve(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<RetrievalResult>, MemoryStoreError> {
        MemoryStore::retrieve(self, query, limit)
    }

    fn get(&self, id: &str) -> Result<Option<MemoryItem>, MemoryStoreError> {
        MemoryStore::get(self, id)
    }

    fn count(&self) -> Result<usize, MemoryStoreError> {
        MemoryStore::count(self)
    }

    fn graph_upsert_node(&self, node: &GraphNode) -> Result<(), MemoryStoreError> {
        MemoryStore::graph_upsert_node(self, node)
    }

    fn graph_upsert_edge(&self, edge: &GraphEdge) -> Result<(), MemoryStoreError> {
        MemoryStore::graph_upsert_edge(self, edge)
    }

    fn graph_recall(
        &self,
        seed_ids: &[&str],
        max_hops: usize,
        min_edge_weight: f64,
        fanout: usize,
    ) -> Result<Vec<AssociationResult>, MemoryStoreError> {
        MemoryStore::graph_recall(self, seed_ids, max_hops, min_edge_weight, fanout)
    }

    fn retention_candidates(
        &self,
        policy: &RetentionPolicy,
    ) -> Result<Vec<RetentionCandidate>, MemoryStoreError> {
        MemoryStore::retention_candidates(self, policy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn existing_in_memory_store_implements_persistence_contract() {
        let store = MemoryStore::open_memory().expect("memory store");
        let contract: &dyn MemoryPersistence = &store;
        assert_eq!(contract.count().expect("count"), 0);
        assert!(contract.get("absent").expect("lookup").is_none());
    }
}
