//! Domain-owned evolution knowledge contract (STORE-002 / I322 C).
//!
//! Only forwards existing typed observation/pattern operations. SQLite
//! migrations, schema/version handling, active filtering and learning logic
//! remain in KnowledgeStore; no provider or consumer is switched.

use crate::store::KnowledgeStore;
use crate::{EvolutionResult, Observation, Pattern};

/// Existing observation and learned-pattern persistence semantics.
///
/// Uses the same `EvolutionResult` errors and types as KnowledgeStore.
pub trait KnowledgePersistence {
    /// Persist an observation under the existing schema rules.
    fn insert_observation(&self, observation: &Observation) -> EvolutionResult<()>;

    /// Return observations with existing order and filtering.
    fn get_observations(&self) -> EvolutionResult<Vec<Observation>>;

    /// Persist a learned pattern, retaining existing conflict semantics.
    fn insert_pattern(&self, pattern: &Pattern) -> EvolutionResult<()>;

    /// Query active patterns under an existing confidence threshold.
    fn get_active_patterns(&self, min_confidence: f64) -> EvolutionResult<Vec<Pattern>>;

    /// Update one existing pattern with the domain's validation and error policy.
    fn update_pattern(&self, pattern: &Pattern) -> EvolutionResult<()>;

    /// Deactivate rather than changing the existing schema or migration behavior.
    fn deactivate_pattern(&self, pattern_id: &str) -> EvolutionResult<()>;

    /// Return patterns including inactive records under existing semantics.
    fn get_all_patterns(&self) -> EvolutionResult<Vec<Pattern>>;
}

impl KnowledgePersistence for KnowledgeStore {
    fn insert_observation(&self, observation: &Observation) -> EvolutionResult<()> {
        KnowledgeStore::insert_observation(self, observation)
    }

    fn get_observations(&self) -> EvolutionResult<Vec<Observation>> {
        KnowledgeStore::get_observations(self)
    }

    fn insert_pattern(&self, pattern: &Pattern) -> EvolutionResult<()> {
        KnowledgeStore::insert_pattern(self, pattern)
    }

    fn get_active_patterns(&self, min_confidence: f64) -> EvolutionResult<Vec<Pattern>> {
        KnowledgeStore::get_active_patterns(self, min_confidence)
    }

    fn update_pattern(&self, pattern: &Pattern) -> EvolutionResult<()> {
        KnowledgeStore::update_pattern(self, pattern)
    }

    fn deactivate_pattern(&self, pattern_id: &str) -> EvolutionResult<()> {
        KnowledgeStore::deactivate_pattern(self, pattern_id)
    }

    fn get_all_patterns(&self) -> EvolutionResult<Vec<Pattern>> {
        KnowledgeStore::get_all_patterns(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn existing_knowledge_store_satisfies_typed_contract() {
        let store = KnowledgeStore::open_memory().expect("knowledge store");
        let contract: &dyn KnowledgePersistence = &store;
        assert!(
            contract
                .get_observations()
                .expect("observations")
                .is_empty()
        );
    }

    #[test]
    fn local_knowledge_trait_delegates_observation_and_active_pattern_lifecycle() {
        let store = KnowledgeStore::open_memory().expect("knowledge store");
        let contract: &dyn KnowledgePersistence = &store;
        let observation = Observation::new(
            crate::SignalType::Correction,
            0.8,
            "Keep typed persistence boundaries".into(),
            None,
            None,
        );
        contract
            .insert_observation(&observation)
            .expect("observation");
        let observations = contract.get_observations().expect("observations");
        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0].id, observation.id);

        let mut pattern = Pattern::new(
            "Typed local delegates".into(),
            "Preserve existing storage behavior".into(),
            "test".into(),
        );
        pattern.confidence = 0.8;
        contract.insert_pattern(&pattern).expect("insert pattern");
        let active = contract.get_active_patterns(0.7).expect("active patterns");
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, pattern.id);
        contract
            .deactivate_pattern(&pattern.id)
            .expect("deactivate");
        assert!(
            contract
                .get_active_patterns(0.7)
                .expect("inactive")
                .is_empty()
        );
        assert_eq!(contract.get_all_patterns().expect("all patterns").len(), 1);
    }
}
