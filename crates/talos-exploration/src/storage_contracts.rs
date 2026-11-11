//! Domain-owned research provenance and lexical search contract (STORE-002 C).
//!
//! Does not modify the existing source/chunk/claim transactional guarantees,
//! citation checks, SQLite FTS5 ranking or the source of truth. No vector,
//! remote, network or generic object provider is introduced.

use crate::{
    Claim, ClaimEdge, ExplorationError, ExplorationStore, ResearchRun, SearchResult, Source,
    SourceChunk, Synthesis,
};

/// Existing research-run, citation integrity and FTS5 retrieval operations.
///
/// Methods use the concrete domain error and result types already exposed by
/// ExplorationStore. A storage descriptor alone cannot implement this trait.
pub trait ExplorationPersistence {
    /// Create an authoritative research run.
    fn create_run(
        &self,
        query: &str,
        plan: Option<&str>,
    ) -> Result<ResearchRun, ExplorationError>;

    /// Mark a run complete without altering its citation ownership.
    fn complete_run(&self, run_id: &str) -> Result<(), ExplorationError>;

    /// Atomically insert a source and its chunks under existing domain rules.
    fn insert_source_with_chunks(
        &mut self,
        source: &Source,
        chunks: &[SourceChunk],
    ) -> Result<(), ExplorationError>;

    /// Read authoritative source provenance.
    fn get_source(&self, id: &str) -> Result<Option<Source>, ExplorationError>;

    /// Search source chunks with the existing FTS rank and result shape.
    fn search_chunks(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, ExplorationError>;

    /// Insert a claim with citation validation.
    fn insert_claim(&self, claim: &Claim) -> Result<(), ExplorationError>;

    /// Insert one validated relation between claims.
    fn insert_claim_edge(&self, edge: &ClaimEdge) -> Result<(), ExplorationError>;

    /// Persist a synthesis while validating its cited sources.
    fn insert_synthesis(&self, synthesis: &Synthesis) -> Result<(), ExplorationError>;

    /// Count authoritative sources in the existing local store.
    fn count_sources(&self) -> Result<usize, ExplorationError>;
}

impl ExplorationPersistence for ExplorationStore {
    fn create_run(
        &self,
        query: &str,
        plan: Option<&str>,
    ) -> Result<ResearchRun, ExplorationError> {
        ExplorationStore::create_run(self, query, plan)
    }

    fn complete_run(&self, run_id: &str) -> Result<(), ExplorationError> {
        ExplorationStore::complete_run(self, run_id)
    }

    fn insert_source_with_chunks(
        &mut self,
        source: &Source,
        chunks: &[SourceChunk],
    ) -> Result<(), ExplorationError> {
        ExplorationStore::insert_source_with_chunks(self, source, chunks)
    }

    fn get_source(&self, id: &str) -> Result<Option<Source>, ExplorationError> {
        ExplorationStore::get_source(self, id)
    }

    fn search_chunks(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, ExplorationError> {
        ExplorationStore::search_chunks(self, query, limit)
    }

    fn insert_claim(&self, claim: &Claim) -> Result<(), ExplorationError> {
        ExplorationStore::insert_claim(self, claim)
    }

    fn insert_claim_edge(&self, edge: &ClaimEdge) -> Result<(), ExplorationError> {
        ExplorationStore::insert_claim_edge(self, edge)
    }

    fn insert_synthesis(&self, synthesis: &Synthesis) -> Result<(), ExplorationError> {
        ExplorationStore::insert_synthesis(self, synthesis)
    }

    fn count_sources(&self) -> Result<usize, ExplorationError> {
        ExplorationStore::count_sources(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn existing_exploration_store_has_typed_contract() {
        let store = ExplorationStore::open_memory().expect("exploration store");
        let contract: &dyn ExplorationPersistence = &store;
        assert_eq!(contract.count_sources().expect("count"), 0);
    }
}
