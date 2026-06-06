//! Talos Evolution — self-evolution engine for agent behavior adaptation.
//!
//! Implements a 4-phase learning loop per ADR-001:
//! 1. Observe: Capture signals during agent execution
//! 2. Extract: Identify patterns from observations
//! 3. Store: Persist patterns with confidence scores
//! 4. Apply: Inject high-confidence patterns into system prompt

pub mod adapter;
pub mod extractor;
pub mod hook;
pub mod observer;
pub mod store;

pub use hook::EvolutionHookHandler;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A signal captured during agent execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SignalType {
    /// User corrected the agent's behavior
    Correction,
    /// Agent encountered an error
    Error,
    /// User expressed satisfaction
    Satisfaction,
    /// Agent identified inefficiency in its own behavior
    Inefficiency,
}

/// An observation captured from a single turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    /// Unique identifier
    pub id: String,
    /// Type of signal
    pub signal_type: SignalType,
    /// Intensity of the signal (0.0 - 1.0)
    pub intensity: f64,
    /// Context description
    pub context: String,
    /// When the observation was made
    pub timestamp: DateTime<Utc>,
    /// Session ID where this was observed
    pub session_id: Option<String>,
    /// Turn number within the session
    pub turn_number: Option<u32>,
}

/// A pattern extracted from multiple observations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    /// Unique identifier
    pub id: String,
    /// Human-readable description of the pattern
    pub description: String,
    /// Natural language instruction to inject into system prompt
    pub instruction: String,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Number of observations supporting this pattern
    pub evidence_count: u32,
    /// When the pattern was first observed
    pub first_observed: DateTime<Utc>,
    /// When the pattern was last updated
    pub last_updated: DateTime<Utc>,
    /// Category of the pattern (e.g., "preference", "error_avoidance", "efficiency")
    pub category: String,
    /// Whether this pattern is active (can be injected into prompts)
    pub active: bool,
    /// Normalized fingerprint for content-based dedup: "{category}|{first 1KB of instruction}"
    pub content_hash: String,
}

/// A conflict between two patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    /// Unique identifier
    pub id: String,
    /// ID of the first pattern
    pub pattern_a_id: String,
    /// ID of the second pattern
    pub pattern_b_id: String,
    /// Description of the conflict
    pub description: String,
    /// When the conflict was detected
    pub detected_at: DateTime<Utc>,
    /// Whether the conflict has been resolved
    pub resolved: bool,
    /// ID of the winning pattern (if resolved)
    pub winner_id: Option<String>,
}

/// Configuration for the evolution engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionConfig {
    /// Minimum confidence to inject pattern into system prompt
    pub min_confidence: f64,
    /// Minimum evidence count to consider a pattern stable
    pub min_evidence: u32,
    /// Half-life for time decay in days
    pub half_life_days: f64,
    /// Maximum number of patterns to inject into system prompt
    pub max_patterns: usize,
    /// Whether to enable automatic pattern extraction
    pub auto_extract: bool,
    /// Maximum bytes stored per observation.context (defense layer 1).
    /// Observations longer than this are truncated with a marker.
    pub max_context_bytes: usize,
    /// Maximum bytes injected into system prompt by BehaviorAdapter (defense layer 2).
    /// Final output is truncated to fit; oversized patterns are dropped first.
    pub max_output_bytes: usize,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.7,
            min_evidence: 3,
            half_life_days: 70.0,
            max_patterns: 5,
            auto_extract: true,
            max_context_bytes: 4096,
            max_output_bytes: 8192,
        }
    }
}

impl Observation {
    /// Create a new observation with the current timestamp.
    pub fn new(
        signal_type: SignalType,
        intensity: f64,
        context: String,
        session_id: Option<String>,
        turn_number: Option<u32>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            signal_type,
            intensity,
            context,
            timestamp: Utc::now(),
            session_id,
            turn_number,
        }
    }

    /// Calculate the age of this observation in days.
    pub fn age_days(&self) -> f64 {
        let now = Utc::now();
        let duration = now.signed_duration_since(self.timestamp);
        duration.num_days() as f64
    }
}

/// Compute a normalized fingerprint for content-based dedup.
/// Format: "{category}|{first 1KB of instruction}" hashed via DefaultHasher.
pub fn compute_content_hash(category: &str, instruction: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let prefix = if instruction.len() > 1024 {
        &instruction[..1024]
    } else {
        instruction
    };
    let fingerprint = format!("{category}|{prefix}");

    let mut hasher = DefaultHasher::new();
    fingerprint.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

impl Pattern {
    /// Create a new pattern with the current timestamp.
    pub fn new(description: String, instruction: String, category: String) -> Self {
        let now = Utc::now();
        let content_hash = compute_content_hash(&category, &instruction);
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            description,
            instruction,
            confidence: 0.0,
            evidence_count: 0,
            first_observed: now,
            last_updated: now,
            category,
            active: true,
            content_hash,
        }
    }

    /// Calculate the time-decayed confidence based on evidence and age.
    pub fn decayed_confidence(&self, half_life_days: f64) -> f64 {
        let age = self.last_updated.signed_duration_since(self.first_observed);
        let days = age.num_days() as f64;
        let decay = (-0.693 * days / half_life_days).exp();
        self.confidence * decay
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observation_new() {
        let obs = Observation::new(
            SignalType::Correction,
            0.8,
            "User said to use functional style".to_string(),
            Some("session-1".to_string()),
            Some(5),
        );

        assert_eq!(obs.signal_type, SignalType::Correction);
        assert_eq!(obs.intensity, 0.8);
        assert!(obs.id.len() > 0);
    }

    #[test]
    fn test_pattern_new() {
        let pattern = Pattern::new(
            "Prefer functional style".to_string(),
            "Use functional programming patterns when writing Rust code".to_string(),
            "preference".to_string(),
        );

        assert_eq!(pattern.confidence, 0.0);
        assert_eq!(pattern.evidence_count, 0);
        assert!(pattern.active);
    }

    #[test]
    fn test_pattern_decay() {
        let mut pattern = Pattern::new(
            "Test pattern".to_string(),
            "Test instruction".to_string(),
            "test".to_string(),
        );
        pattern.confidence = 0.8;

        // Fresh pattern should have full confidence
        let decayed = pattern.decayed_confidence(70.0);
        assert!((decayed - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_evolution_config_default() {
        let config = EvolutionConfig::default();
        assert_eq!(config.min_confidence, 0.7);
        assert_eq!(config.min_evidence, 3);
        assert_eq!(config.half_life_days, 70.0);
    }

    #[test]
    fn test_evolution_config_default_has_byte_caps() {
        let config = EvolutionConfig::default();
        assert_eq!(config.max_context_bytes, 4096);
        assert_eq!(config.max_output_bytes, 8192);
    }

    #[test]
    fn test_evolution_config_max_context_bytes_default_4kb() {
        let config = EvolutionConfig::default();
        assert_eq!(config.max_context_bytes, 4 * 1024);
    }

    #[test]
    fn test_evolution_config_max_output_bytes_default_8kb() {
        let config = EvolutionConfig::default();
        assert_eq!(config.max_output_bytes, 8 * 1024);
    }
}
