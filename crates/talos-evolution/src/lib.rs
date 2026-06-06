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

// ─── MenteDB-aligned types (I021-S1) ────────────────────────────────────────

/// The kind of learning signal captured during agent execution.
/// Four base variants per the MenteDB cognitive-feedback blueprint
/// (`docs/reference/REFERENCE-PROJECTS.md` §17).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SignalKind {
    /// User corrected the agent's behavior ("don't do that", "use X instead")
    Correction,
    /// Agent encountered an error (tool failure, provider error, etc.)
    Error,
    /// User expressed satisfaction or approval
    Satisfaction,
    /// Agent identified inefficiency in its own behavior
    Inefficiency,
}

impl std::fmt::Display for SignalKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignalKind::Correction => write!(f, "Correction"),
            SignalKind::Error => write!(f, "Error"),
            SignalKind::Satisfaction => write!(f, "Satisfaction"),
            SignalKind::Inefficiency => write!(f, "Inefficiency"),
        }
    }
}

/// A single learning signal captured during agent execution.
///
/// Per the MenteDB blueprint, `context` is a **small window** (typically
/// < 500 bytes) centered on the marker phrase that triggered the signal,
/// NOT the full user message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Signal {
    /// What kind of signal this is
    pub kind: SignalKind,
    /// Signal intensity (0.0 – 1.0)
    pub intensity: f32,
    /// Small context window around the marker phrase (typically < 500 bytes)
    pub context: String,
    /// Which tool was involved, if applicable
    pub tool_name: Option<String>,
}

/// Outcome of a single agent turn.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TurnOutcome {
    /// Turn completed successfully
    Success,
    /// Turn completed but with partial results
    PartialSuccess,
    /// Turn failed (error, provider failure, etc.)
    Failure,
    /// User corrected the agent's output during this turn
    UserCorrected,
}

/// Record of a tool invocation during a turn.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolUsage {
    /// Name of the tool that was called
    pub tool_name: String,
    /// Hash of the tool arguments (for dedup / comparison)
    pub arguments_hash: u64,
    /// Brief summary of the tool result
    pub result_summary: String,
}

/// Per-turn observation that aggregates multiple signals with turn-level metadata.
///
/// This is the MenteDB-aligned replacement for the legacy [`Observation`] type.
/// A `TurnObservation` is the parent (per-turn), containing child [`Signal`]s
/// (per-event) plus tool usage, outcome, and timing data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnObservation {
    /// Signals captured during this turn
    pub signals: Vec<Signal>,
    /// Tools that were used during this turn
    pub tools_used: Vec<ToolUsage>,
    /// How the turn ended
    pub outcome: TurnOutcome,
    /// Turn duration in milliseconds
    pub duration_ms: u64,
    /// Session this turn belongs to
    pub session_id: uuid::Uuid,
    /// Turn number within the session
    pub turn_number: u32,
}

impl TurnObservation {
    /// Create a new `TurnObservation` with the given turn-level metadata.
    pub fn new(
        session_id: uuid::Uuid,
        turn_number: u32,
        outcome: TurnOutcome,
        duration_ms: u64,
    ) -> Self {
        Self {
            signals: Vec::new(),
            tools_used: Vec::new(),
            outcome,
            duration_ms,
            session_id,
            turn_number,
        }
    }

    /// Add a signal to this turn observation.
    pub fn add_signal(&mut self, signal: Signal) {
        self.signals.push(signal);
    }

    /// Record a tool usage for this turn.
    pub fn add_tool_usage(&mut self, usage: ToolUsage) {
        self.tools_used.push(usage);
    }
}

// ─── Legacy types (backward-compatible, kept for migration) ─────────────────

/// A signal captured during agent execution.
///
/// **Deprecated**: Use [`Signal`] + [`TurnObservation`] instead.
/// This type is retained for backward compatibility with the pre-I021 schema.
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
///
/// **Deprecated**: Use [`TurnObservation`] + [`Signal`] instead.
/// This type is retained for backward compatibility with the pre-I021 schema.
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
    // ─── I021-S3: MenteDB-aligned fields ────────────────────────────────────
    /// Structured key identifying this pattern (e.g., "prefer_functional_style")
    pub key: String,
    /// Structured value as JSON (replaces free-text instruction at the data level)
    pub value: serde_json::Value,
    /// Number of contradicting observations
    pub contradicting_count: u32,
    /// When this pattern was last reinforced by a matching signal
    pub last_reinforced: DateTime<Utc>,
    /// Session IDs where this pattern was observed (traceability)
    pub source_sessions: Vec<uuid::Uuid>,
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
    ///
    /// Backward-compatible constructor. Derives `key` and `value` from
    /// `description` and `instruction` for migration purposes.
    pub fn new(description: String, instruction: String, category: String) -> Self {
        let now = Utc::now();
        let content_hash = compute_content_hash(&category, &instruction);
        let key = category.clone();
        let value = serde_json::json!({ "instruction": instruction });
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
            key,
            value,
            contradicting_count: 0,
            last_reinforced: now,
            source_sessions: Vec::new(),
        }
    }

    /// Create a pattern with MenteDB-aligned fields.
    ///
    /// `description` and `instruction` are derived from `key` + `value`
    /// rendering, keeping the BehaviorAdapter output format unchanged.
    pub fn new_with_key(
        key: String,
        value: serde_json::Value,
        category: String,
        source_session: Option<uuid::Uuid>,
    ) -> Self {
        let now = Utc::now();
        let description = format!("Pattern: {key}");
        let instruction = value.to_string();
        let content_hash = compute_content_hash(&category, &instruction);
        let mut source_sessions = Vec::new();
        if let Some(sid) = source_session {
            source_sessions.push(sid);
        }
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
            key,
            value,
            contradicting_count: 0,
            last_reinforced: now,
            source_sessions,
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

    // ─── I021-S1: New MenteDB-aligned type tests ────────────────────────────

    #[test]
    fn test_signal_roundtrip_preserves_all_fields() {
        let signal = Signal {
            kind: SignalKind::Correction,
            intensity: 0.85,
            context: "不要用 sed".to_string(),
            tool_name: Some("bash".to_string()),
        };

        // Serialize and deserialize roundtrip
        let json = serde_json::to_string(&signal).expect("serialize Signal");
        let restored: Signal = serde_json::from_str(&json).expect("deserialize Signal");

        assert_eq!(restored.kind, SignalKind::Correction);
        assert!((restored.intensity - 0.85).abs() < f32::EPSILON);
        assert_eq!(restored.context, "不要用 sed");
        assert_eq!(restored.tool_name, Some("bash".to_string()));
    }

    #[test]
    fn test_turn_observation_multi_signal_flush() {
        let session_id = uuid::Uuid::new_v4();
        let mut turn = TurnObservation::new(session_id, 3, TurnOutcome::Success, 1500);

        turn.add_signal(Signal {
            kind: SignalKind::Correction,
            intensity: 0.9,
            context: "use HashMap".to_string(),
            tool_name: None,
        });
        turn.add_signal(Signal {
            kind: SignalKind::Inefficiency,
            intensity: 0.4,
            context: "took 10 steps".to_string(),
            tool_name: Some("bash".to_string()),
        });
        turn.add_tool_usage(ToolUsage {
            tool_name: "read".to_string(),
            arguments_hash: 42,
            result_summary: "file contents".to_string(),
        });

        assert_eq!(turn.signals.len(), 2);
        assert_eq!(turn.tools_used.len(), 1);
        assert_eq!(turn.outcome, TurnOutcome::Success);
        assert_eq!(turn.duration_ms, 1500);
        assert_eq!(turn.session_id, session_id);
        assert_eq!(turn.turn_number, 3);

        // Roundtrip
        let json = serde_json::to_string(&turn).expect("serialize TurnObservation");
        let restored: TurnObservation =
            serde_json::from_str(&json).expect("deserialize TurnObservation");
        assert_eq!(restored.signals.len(), 2);
        assert_eq!(restored.signals[0].kind, SignalKind::Correction);
        assert_eq!(restored.signals[1].kind, SignalKind::Inefficiency);
        assert_eq!(restored.tools_used[0].tool_name, "read");
    }

    #[test]
    fn test_signal_kind_display() {
        assert_eq!(format!("{}", SignalKind::Correction), "Correction");
        assert_eq!(format!("{}", SignalKind::Error), "Error");
        assert_eq!(format!("{}", SignalKind::Satisfaction), "Satisfaction");
        assert_eq!(format!("{}", SignalKind::Inefficiency), "Inefficiency");
    }

    #[test]
    fn test_tool_usage_roundtrip() {
        let usage = ToolUsage {
            tool_name: "write".to_string(),
            arguments_hash: 12345,
            result_summary: "wrote 50 bytes".to_string(),
        };

        let json = serde_json::to_string(&usage).expect("serialize ToolUsage");
        let restored: ToolUsage = serde_json::from_str(&json).expect("deserialize ToolUsage");

        assert_eq!(restored.tool_name, "write");
        assert_eq!(restored.arguments_hash, 12345);
        assert_eq!(restored.result_summary, "wrote 50 bytes");
    }

    #[test]
    fn test_pattern_roundtrip_with_mentedb_fields() {
        let session_id = uuid::Uuid::new_v4();
        let mut pattern = Pattern::new(
            "Prefer functional style".to_string(),
            "Use functional programming patterns".to_string(),
            "preference".to_string(),
        );
        pattern.key = "prefer_functional_style".to_string();
        pattern.value = serde_json::json!({ "style": "functional", "language": "rust" });
        pattern.contradicting_count = 2;
        pattern.last_reinforced = Utc::now();
        pattern.source_sessions = vec![session_id];

        let json = serde_json::to_string(&pattern).expect("serialize Pattern");
        let restored: Pattern = serde_json::from_str(&json).expect("deserialize Pattern");

        assert_eq!(restored.key, "prefer_functional_style");
        assert_eq!(restored.value["style"], "functional");
        assert_eq!(restored.contradicting_count, 2);
        assert_eq!(restored.source_sessions.len(), 1);
        assert_eq!(restored.source_sessions[0], session_id);
    }
}
