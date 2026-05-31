//! BehaviorAdapter — injects high-confidence patterns into system prompt.

use crate::{EvolutionConfig, Pattern};
use crate::store::KnowledgeStore;

/// Injects learned patterns into the system prompt.
pub struct BehaviorAdapter<'a> {
    store: &'a KnowledgeStore,
    config: EvolutionConfig,
}

impl<'a> BehaviorAdapter<'a> {
    /// Create a new BehaviorAdapter.
    pub fn new(store: &'a KnowledgeStore, config: EvolutionConfig) -> Self {
        Self { store, config }
    }

    /// Get the evolution context to inject into the system prompt.
    pub fn get_evolution_context(&self) -> String {
        let patterns = match self.store.get_active_patterns(self.config.min_confidence) {
            Ok(p) => p,
            Err(_) => return String::new(),
        };

        let filtered: Vec<&Pattern> = patterns
            .iter()
            .filter(|p| p.evidence_count >= self.config.min_evidence)
            .take(self.config.max_patterns)
            .collect();

        if filtered.is_empty() {
            return String::new();
        }

        let mut context = String::from("## Learned Patterns\n\n");
        context.push_str("Based on past interactions, the following patterns have been learned:\n\n");

        for (i, pattern) in filtered.iter().enumerate() {
            context.push_str(&format!(
                "{}. [{}] {} (confidence: {:.0}%, evidence: {})\n",
                i + 1,
                pattern.category,
                pattern.instruction,
                pattern.confidence * 100.0,
                pattern.evidence_count
            ));
        }

        context
    }

    /// Get patterns for a specific category.
    pub fn get_patterns_by_category(&self, category: &str) -> Vec<Pattern> {
        match self.store.get_active_patterns(0.0) {
            Ok(patterns) => patterns
                .into_iter()
                .filter(|p| p.category == category && p.evidence_count >= self.config.min_evidence)
                .collect(),
            Err(_) => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_evolution_context_empty() {
        let store = KnowledgeStore::open_memory().unwrap();
        let config = EvolutionConfig::default();
        let adapter = BehaviorAdapter::new(&store, config);

        let context = adapter.get_evolution_context();
        assert!(context.is_empty());
    }

    #[test]
    fn test_get_evolution_context_with_patterns() {
        let store = KnowledgeStore::open_memory().unwrap();
        let config = EvolutionConfig::default();

        let mut pattern = Pattern::new(
            "Prefer functional style".to_string(),
            "Use functional programming patterns".to_string(),
            "preference".to_string(),
        );
        pattern.confidence = 0.9;
        pattern.evidence_count = 5;
        store.insert_pattern(&pattern).unwrap();

        let adapter = BehaviorAdapter::new(&store, config);
        let context = adapter.get_evolution_context();

        assert!(context.contains("Learned Patterns"));
        assert!(context.contains("Use functional programming patterns"));
    }
}
