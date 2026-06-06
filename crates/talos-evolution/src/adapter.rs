//! BehaviorAdapter — injects high-confidence patterns into system prompt.

use crate::store::KnowledgeStore;
use crate::{EvolutionConfig, Pattern};

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

        let mut filtered: Vec<&Pattern> = patterns
            .iter()
            .filter(|p| p.evidence_count >= self.config.min_evidence)
            .collect();
        filtered.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));

        if filtered.is_empty() {
            return String::new();
        }

        let max_output = self.config.max_output_bytes;
        let header = "## Learned Patterns\n\nBased on past interactions, the following patterns have been learned:\n\n";
        let mut context = String::from(header);
        let mut dropped = 0;

        for (i, pattern) in filtered.iter().enumerate() {
            if pattern.instruction.len() > max_output {
                dropped += 1;
                tracing::warn!(
                    pattern_id = %pattern.id,
                    bytes = pattern.instruction.len(),
                    "dropped oversized pattern"
                );
                continue;
            }

            let entry = format!(
                "{}. [{}] {} (confidence: {:.0}%, evidence: {})\n",
                i + 1 - dropped,
                pattern.category,
                pattern.instruction,
                pattern.confidence * 100.0,
                pattern.evidence_count
            );

            if context.len() + entry.len() > max_output {
                break;
            }
            context.push_str(&entry);
        }

        if dropped > 0 {
            tracing::warn!(count = dropped, "dropped oversized patterns");
        }

        if context.len() > max_output {
            let byte_len = context.len();
            context.truncate(max_output);
            context.push_str(&format!("... [truncated, original was {byte_len} bytes]"));
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

    #[test]
    fn test_get_evolution_context_caps_output_bytes() {
        let store = KnowledgeStore::open_memory().unwrap();
        let mut config = EvolutionConfig::default();
        config.max_output_bytes = 200;

        let mut pattern = Pattern::new(
            "Test".to_string(),
            "x".repeat(150),
            "test".to_string(),
        );
        pattern.confidence = 0.9;
        pattern.evidence_count = 5;
        store.insert_pattern(&pattern).unwrap();

        let adapter = BehaviorAdapter::new(&store, config);
        let context = adapter.get_evolution_context();

        assert!(
            context.len() <= 200,
            "output {} exceeds max_output_bytes 200",
            context.len()
        );
    }

    #[test]
    fn test_get_evolution_context_drops_oversized_single_pattern() {
        let store = KnowledgeStore::open_memory().unwrap();
        let mut config = EvolutionConfig::default();
        config.max_output_bytes = 100;

        let mut pattern = Pattern::new(
            "Big".to_string(),
            "x".repeat(5000),
            "test".to_string(),
        );
        pattern.confidence = 0.9;
        pattern.evidence_count = 5;
        store.insert_pattern(&pattern).unwrap();

        let adapter = BehaviorAdapter::new(&store, config);
        let context = adapter.get_evolution_context();

        assert!(
            context.len() <= 100,
            "output {} exceeds max_output_bytes 100",
            context.len()
        );
        assert!(!context.contains("xxxxx"), "oversized pattern should be dropped");
    }

    #[test]
    fn test_get_evolution_context_orders_by_confidence_first() {
        let store = KnowledgeStore::open_memory().unwrap();
        let mut config = EvolutionConfig::default();
        config.max_output_bytes = 500;
        config.min_confidence = 0.0;

        for (conf, label) in [(0.5, "low"), (0.9, "high"), (0.7, "mid")] {
            let mut pattern = Pattern::new(
                format!("{label} confidence"),
                format!("[{label}] instruction"),
                "test".to_string(),
            );
            pattern.confidence = conf;
            pattern.evidence_count = 5;
            store.insert_pattern(&pattern).unwrap();
        }

        let adapter = BehaviorAdapter::new(&store, config);
        let context = adapter.get_evolution_context();

        let high_pos = context.find("[high]").unwrap();
        let mid_pos = context.find("[mid]").unwrap();
        let low_pos = context.find("[low]").unwrap();
        assert!(high_pos < mid_pos, "high confidence should appear before mid");
        assert!(mid_pos < low_pos, "mid confidence should appear before low");
    }
}
