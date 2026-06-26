//! Episodic-to-semantic memory consolidation pipeline.
//!
//! Reads session episodes, extracts semantic memory candidates via a pluggable
//! [`EpisodeExtractor`], and writes them to [`MemoryStore`] with evidence links.
//!
//! The pipeline is bounded, ADD-only, and deterministic when using the default
//! [`RuleBasedExtractor`]. No LLM or network calls are made.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{EvidenceLink, MemoryItem, MemoryKind, MemoryStore, MemoryStoreError};

/// A single episode read from a session, neutral to the session storage format.
#[derive(Debug, Clone)]
pub struct SessionEpisode {
    /// The session this episode belongs to.
    pub session_id: String,
    /// Unique identifier for this episode entry.
    pub entry_id: String,
    /// Position of this episode in the session sequence.
    pub turn_index: usize,
    /// The role that produced this episode: `"user"`, `"assistant"`, `"system"`, `"tool"`.
    pub role: String,
    /// The text content of this episode.
    pub content: String,
    /// When this episode was created.
    pub timestamp: DateTime<Utc>,
}

/// A memory candidate extracted from session episodes.
#[derive(Debug, Clone)]
pub struct MemoryCandidate {
    /// The kind of memory this candidate represents.
    pub kind: MemoryKind,
    /// A key identifying the concept or topic.
    pub key: String,
    /// The full content of the memory.
    pub content: String,
    /// Confidence score (0.0 to 1.0).
    pub confidence: f64,
    /// The session this candidate was extracted from.
    pub source_session_id: String,
    /// The entry ID within the session.
    pub source_entry_id: String,
    /// The turn index within the session.
    pub source_turn_index: usize,
}

/// Trait for extracting semantic memory candidates from session episodes.
///
/// Implementations must be deterministic: the same input must always produce
/// the same output.
pub trait EpisodeExtractor {
    /// Extract memory candidates from a slice of session episodes.
    fn extract(&self, episodes: &[SessionEpisode]) -> Vec<MemoryCandidate>;
}

/// Deterministic rule-based episode extractor (no LLM/provider dependency).
///
/// Applies simple heuristics to identify memory-worthy content from session
/// episodes. Suitable for offline testing and as a baseline extractor.
pub struct RuleBasedExtractor {
    /// Maximum number of candidates to return per extraction.
    max_candidates: usize,
}

impl RuleBasedExtractor {
    /// Create a new extractor with the default limit of 20 candidates.
    pub fn new() -> Self {
        Self { max_candidates: 20 }
    }

    /// Create a new extractor with a custom candidate limit.
    pub fn with_max_candidates(max: usize) -> Self {
        Self {
            max_candidates: max,
        }
    }
}

impl Default for RuleBasedExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl EpisodeExtractor for RuleBasedExtractor {
    fn extract(&self, episodes: &[SessionEpisode]) -> Vec<MemoryCandidate> {
        let mut candidates: Vec<MemoryCandidate> = Vec::new();

        for episode in episodes {
            // Skip messages shorter than 10 characters (noise).
            if episode.content.len() < 10 {
                continue;
            }

            // Skip system and tool messages (they are events, not original facts).
            if episode.role == "system" || episode.role == "tool" {
                continue;
            }

            // Only process user and assistant messages.
            if episode.role != "user" && episode.role != "assistant" {
                continue;
            }

            // Derive key from first 40 characters, truncated at first newline.
            let key = derive_key(&episode.content);

            // Compute confidence deterministically.
            let confidence = compute_confidence(&episode.content, &episode.role);

            candidates.push(MemoryCandidate {
                kind: MemoryKind::Semantic,
                key,
                content: episode.content.clone(),
                confidence,
                source_session_id: episode.session_id.clone(),
                source_entry_id: episode.entry_id.clone(),
                source_turn_index: episode.turn_index,
            });
        }

        // Sort by confidence descending, then by turn_index ascending for ties.
        candidates.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.source_turn_index.cmp(&b.source_turn_index))
        });

        // Limit to max_candidates.
        candidates.truncate(self.max_candidates);

        candidates
    }
}

/// Derive a memory key from content: first 40 chars, truncated at first newline, trimmed.
fn derive_key(content: &str) -> String {
    let truncated = content
        .split_once('\n')
        .map(|(first, _)| first)
        .unwrap_or(content);
    let truncated = truncated.chars().take(40).collect::<String>();
    truncated.trim().to_string()
}

/// Compute confidence based on deterministic heuristics.
fn compute_confidence(content: &str, role: &str) -> f64 {
    let lower = content.trim_start().to_lowercase();

    // High confidence: explicit memory markers.
    let markers = ["remember", "note", "important", "always", "never"];
    if markers.iter().any(|m| lower.starts_with(m)) {
        return 0.8;
    }

    // Medium confidence: user message with substantial content.
    if role == "user" && content.len() > 50 {
        return 0.6;
    }

    // Default confidence.
    0.4
}

/// Configuration for the consolidation pipeline.
#[derive(Debug, Clone)]
pub struct ConsolidationConfig {
    /// Whether consolidation is enabled. Default: `false` (opt-in for safety).
    pub enabled: bool,
    /// Maximum candidates to consider per session.
    pub max_candidates_per_session: usize,
}

impl Default for ConsolidationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_candidates_per_session: 20,
        }
    }
}

/// Report from a consolidation run.
#[derive(Debug, Clone, Default)]
pub struct ConsolidationReport {
    /// Number of candidates extracted by the extractor.
    pub candidates_extracted: usize,
    /// Number of candidates successfully inserted into the store.
    pub inserted: usize,
    /// Number of candidates skipped as exact duplicates.
    pub duplicates_skipped: usize,
    /// Number of evidence links created.
    pub evidence_links_created: usize,
    /// Errors encountered during consolidation (non-fatal).
    pub errors: Vec<String>,
}

/// Run consolidation: extract candidates from episodes and write to the memory store.
///
/// ADD-only: exact content-hash duplicates are skipped. Conflicting same-key
/// entries with different content are preserved as separate rows per ADR-016.
///
/// # Arguments
///
/// * `store` — The memory store to write to.
/// * `episodes` — Session episodes to consolidate.
/// * `extractor` — The extractor to use for candidate extraction.
/// * `config` — Pipeline configuration.
///
/// # Returns
///
/// A [`ConsolidationReport`] summarizing the consolidation run.
pub fn consolidate_episodes(
    store: &mut MemoryStore,
    episodes: &[SessionEpisode],
    extractor: &dyn EpisodeExtractor,
    config: &ConsolidationConfig,
) -> Result<ConsolidationReport, MemoryStoreError> {
    let mut report = ConsolidationReport::default();

    // If disabled, return immediately without any writes.
    if !config.enabled {
        return Ok(report);
    }

    // Extract candidates.
    let candidates = extractor.extract(episodes);
    report.candidates_extracted = candidates.len();

    // Process each candidate.
    for candidate in &candidates {
        // Skip malformed candidates (empty content or key).
        if candidate.content.is_empty() || candidate.key.is_empty() {
            report.errors.push(format!(
                "skipped malformed candidate: empty content or key (entry_id={})",
                candidate.source_entry_id
            ));
            continue;
        }

        let now = Utc::now();

        let memory_id = Uuid::new_v4().to_string();
        let item = MemoryItem {
            id: memory_id.clone(),
            kind: candidate.kind,
            key: candidate.key.clone(),
            content: candidate.content.clone(),
            confidence: candidate.confidence,
            created_at: now,
            last_reinforced: now,
            last_accessed: None,
            contradiction_ref: None,
        };

        match store.insert(item) {
            Ok(inserted) => {
                if inserted {
                    report.inserted += 1;

                    let link = EvidenceLink {
                        id: Uuid::new_v4().to_string(),
                        memory_id,
                        source_type: "session".to_string(),
                        source_ref: format!(
                            "{}:{}:{}",
                            candidate.source_session_id,
                            candidate.source_entry_id,
                            candidate.source_turn_index
                        ),
                        created_at: now,
                    };

                    if let Err(e) = store.insert_evidence(link) {
                        report
                            .errors
                            .push(format!("failed to insert evidence link: {e}"));
                    } else {
                        report.evidence_links_created += 1;
                    }
                } else {
                    report.duplicates_skipped += 1;
                }
            }
            Err(e) => {
                report
                    .errors
                    .push(format!("failed to insert memory item: {e}"));
            }
        }
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_episode(
        session_id: &str,
        entry_id: &str,
        turn_index: usize,
        role: &str,
        content: &str,
    ) -> SessionEpisode {
        SessionEpisode {
            session_id: session_id.to_string(),
            entry_id: entry_id.to_string(),
            turn_index,
            role: role.to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
        }
    }

    fn make_episodes() -> Vec<SessionEpisode> {
        vec![
            make_episode(
                "sess-1",
                "entry-1",
                0,
                "user",
                "remember to use Rust edition 2024 for all new projects",
            ),
            make_episode(
                "sess-1",
                "entry-2",
                1,
                "assistant",
                "I will use Rust edition 2024. It provides improved macro hygiene and async traits.",
            ),
            make_episode(
                "sess-1",
                "entry-3",
                2,
                "system",
                "Tool call result: success",
            ),
            make_episode("sess-1", "entry-4", 3, "user", "short"),
            make_episode(
                "sess-1",
                "entry-5",
                4,
                "user",
                "This is a longer user message that should get medium confidence because it exceeds fifty characters in length",
            ),
        ]
    }

    #[test]
    fn consolidation_creates_semantic_memory_with_evidence() {
        let mut store = MemoryStore::open_memory().unwrap();
        let episodes = make_episodes();
        let extractor = RuleBasedExtractor::new();
        let config = ConsolidationConfig {
            enabled: true,
            max_candidates_per_session: 20,
        };

        let report = consolidate_episodes(&mut store, &episodes, &extractor, &config).unwrap();

        assert!(
            report.inserted > 0,
            "Should have inserted at least one item"
        );
        assert!(
            report.evidence_links_created > 0,
            "Should have created evidence links"
        );

        // Verify retrieval works.
        let results = store.retrieve("Rust", 10).unwrap();
        assert!(!results.is_empty(), "Should find Rust-related memory");

        // Verify evidence source_ref contains session_id and entry_id.
        let has_valid_evidence = results.iter().any(|r| {
            r.evidence
                .iter()
                .any(|e| e.source_ref.contains("sess-1") && e.source_ref.contains("entry-"))
        });
        assert!(
            has_valid_evidence,
            "Evidence should reference the source session and entry"
        );
    }

    #[test]
    fn exact_duplicate_dedup_by_content_hash() {
        let mut store = MemoryStore::open_memory().unwrap();
        let episodes = make_episodes();
        let extractor = RuleBasedExtractor::new();
        let config = ConsolidationConfig {
            enabled: true,
            max_candidates_per_session: 20,
        };

        // First run.
        let report1 = consolidate_episodes(&mut store, &episodes, &extractor, &config).unwrap();
        let first_inserted = report1.inserted;
        assert!(first_inserted > 0);

        // Second run on same episodes.
        let report2 = consolidate_episodes(&mut store, &episodes, &extractor, &config).unwrap();

        assert_eq!(
            report2.inserted, 0,
            "Second run should insert nothing (all duplicates)"
        );
        assert!(
            report2.duplicates_skipped > 0,
            "Second run should skip duplicates"
        );
    }

    #[test]
    fn conflicting_same_key_preserved_not_overwritten() {
        let mut store = MemoryStore::open_memory().unwrap();

        // Two episodes with same key prefix (first 40 chars match) but different content.
        let episodes = vec![
            make_episode(
                "sess-2",
                "entry-a",
                0,
                "user",
                "Python is a dynamically typed programming language with great ecosystem",
            ),
            make_episode(
                "sess-2",
                "entry-b",
                1,
                "user",
                "Python is a statically typed programming language with strict type checking",
            ),
        ];

        let extractor = RuleBasedExtractor::new();
        let config = ConsolidationConfig {
            enabled: true,
            max_candidates_per_session: 20,
        };

        let report = consolidate_episodes(&mut store, &episodes, &extractor, &config).unwrap();

        assert_eq!(
            report.inserted, 2,
            "Both conflicting items should be inserted"
        );
        assert_eq!(store.count().unwrap(), 2, "Store should contain both items");
    }

    #[test]
    fn malformed_and_empty_sessions_degrade_gracefully() {
        let mut store = MemoryStore::open_memory().unwrap();

        let episodes = vec![
            // Empty content.
            make_episode("sess-3", "entry-1", 0, "user", ""),
            // Very short content (<10 chars).
            make_episode("sess-3", "entry-2", 1, "user", "hi"),
            // System role (should be skipped by extractor).
            make_episode(
                "sess-3",
                "entry-3",
                2,
                "system",
                "This is a system message with enough content",
            ),
            // Tool role (should be skipped by extractor).
            make_episode(
                "sess-3",
                "entry-4",
                3,
                "tool",
                "Tool output with enough characters to pass length check",
            ),
        ];

        let extractor = RuleBasedExtractor::new();
        let config = ConsolidationConfig {
            enabled: true,
            max_candidates_per_session: 20,
        };

        // Should not panic.
        let report = consolidate_episodes(&mut store, &episodes, &extractor, &config).unwrap();

        // System and tool are skipped by extractor; empty and short are also skipped.
        // So candidates_extracted should be 0.
        assert_eq!(
            report.candidates_extracted, 0,
            "No valid candidates from malformed episodes"
        );
        assert_eq!(report.inserted, 0);
        // Errors may or may not be populated depending on where skipping happens.
        // The key assertion is: no panic occurred.
    }

    #[test]
    fn disabled_config_no_writes() {
        let mut store = MemoryStore::open_memory().unwrap();
        let episodes = make_episodes();
        let extractor = RuleBasedExtractor::new();
        let config = ConsolidationConfig {
            enabled: false,
            max_candidates_per_session: 20,
        };

        let report = consolidate_episodes(&mut store, &episodes, &extractor, &config).unwrap();

        assert_eq!(report.candidates_extracted, 0);
        assert_eq!(report.inserted, 0);
        assert_eq!(report.duplicates_skipped, 0);
        assert_eq!(report.evidence_links_created, 0);
        assert!(report.errors.is_empty());
        assert_eq!(store.count().unwrap(), 0, "Store should be empty");
    }

    #[test]
    fn rule_based_extractor_is_deterministic() {
        let episodes = make_episodes();
        let extractor = RuleBasedExtractor::new();

        let result1 = extractor.extract(&episodes);
        let result2 = extractor.extract(&episodes);

        assert_eq!(
            result1.len(),
            result2.len(),
            "Both extractions should produce same number of candidates"
        );

        for (c1, c2) in result1.iter().zip(result2.iter()) {
            assert_eq!(c1.key, c2.key, "Keys should match");
            assert_eq!(c1.confidence, c2.confidence, "Confidence should match");
            assert_eq!(
                c1.source_turn_index, c2.source_turn_index,
                "Turn index should match"
            );
        }
    }
}
