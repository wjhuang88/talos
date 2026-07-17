//! Episodic-to-semantic memory consolidation pipeline (ADR-046).
//!
//! Reads session episodes, extracts semantic memory candidates via a pluggable
//! [`EpisodeExtractor`], and writes them to [`MemoryStore`] with evidence links.
//!
//! The I137 benchmark selected No-Go, so [`RuleBasedExtractor`] retains the
//! production baseline. Previously published admission types remain as a
//! compatibility/benchmark surface only. Sensitive content is rejected before
//! any memory write. No LLM or network calls are made.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{EvidenceLink, MemoryItem, MemoryKind, MemoryStore, MemoryStoreError};

/// Reason codes for memory admission decisions (ADR-046).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AdmissionReason {
    /// Candidate admitted: novelty × utility exceeded threshold.
    Admitted,
    /// Rejected: content matches sensitive patterns (credentials, tokens, etc.).
    SensitiveContent,
    /// Rejected: novelty × utility below admission threshold.
    BelowThreshold,
    /// Rejected: role is system/tool (not original user/assistant content).
    ExcludedRole,
    /// Rejected: content too short.
    TooShort,
}

/// The result of evaluating a memory candidate for admission (ADR-046).
///
/// This is separate from evidence confidence. A candidate may have low
/// admission score but still be admitted if it meets the threshold.
#[derive(Debug, Clone)]
pub struct AdmissionDecision {
    /// Whether the candidate should be written to the memory store.
    pub admit: bool,
    /// The admission score (novelty × committed_utility), [0, 1].
    pub score: f64,
    /// Stable reason code explaining the decision.
    pub reason: AdmissionReason,
}
/// Admission threshold: candidates with score below this are rejected.
/// Set to 0.15 to reject routine chatter.
const ADMISSION_THRESHOLD: f64 = 0.15;

/// Bare tokens matched as word boundaries to catch prose-form credentials.
const SENSITIVE_WORD_TOKENS: &[&str] = &[
    "password",
    "secret",
    "token",
    "credential",
    "api_key",
    "apikey",
];

/// Structured patterns always sensitive (unambiguous multi-char sequences).
const SENSITIVE_SUBSTRINGS: &[&str] = &[
    "access_token",
    "refresh_token",
    "authorization:",
    "bearer ",
    "cookie:",
    "set-cookie",
    "private_key",
    "-----begin",
    "sk-ant-",
    "sk-proj-",
];

/// Leading chatter prefixes. Only matched at START of message to avoid
/// vetoing substantive content that contains a noise word mid-sentence.
const NOISE_PREFIXES: &[&str] = &[
    "can you help",
    "how are you",
    "thanks",
    "thank you",
    "that's helpful",
    "hello",
    "hi,",
    "hi!",
    "hey ",
    "ok,",
    "got it",
    "i see",
    "makes sense",
    "understood",
    "sounds good",
    "let me think",
    "i was wondering",
    "i've been reading",
    "i'm trying to figure out",
    "i'm not sure",
    "seems like",
];

/// Check if content contains sensitive patterns that must not be persisted.
///
/// Uses word-boundary matching for bare tokens (password, secret, token)
/// to catch prose-form credentials like "my password is hunter2".
/// Uses substring matching for structured patterns (sk-ant-, bearer, etc.).
/// Does NOT use bare `sk-` substring (matches task-, risk-, disk-).
#[must_use]
pub fn is_sensitive_content(content: &str) -> bool {
    let lower = content.to_lowercase();

    // Structured patterns: safe substring match.
    if SENSITIVE_SUBSTRINGS.iter().any(|p| lower.contains(p)) {
        return true;
    }

    // Word-boundary tokens: match as whole words.
    for token in SENSITIVE_WORD_TOKENS {
        let mut search_from = 0;
        while let Some(pos) = lower[search_from..].find(token) {
            let abs_pos = search_from + pos;
            let after = abs_pos + token.len();
            let before_ok = abs_pos == 0
                || lower
                    .as_bytes()
                    .get(abs_pos - 1)
                    .is_some_and(|&c| !c.is_ascii_alphanumeric());
            let after_ok = after >= lower.len()
                || lower
                    .as_bytes()
                    .get(after)
                    .is_some_and(|&c| !c.is_ascii_alphanumeric());
            if before_ok && after_ok {
                return true;
            }
            search_from = abs_pos + 1;
        }
    }

    false
}

/// Check if content starts with conversational noise.
/// Only matches at the START to avoid vetoing substantive content.
#[must_use]
fn is_noise_content(content: &str) -> bool {
    let lower = content.trim_start().to_lowercase();
    NOISE_PREFIXES.iter().any(|p| lower.starts_with(p))
}

/// Evaluate a memory candidate for admission using ADR-046 policy.
///
/// Returns an [`AdmissionDecision`] with the admission score and reason.
/// The caller must respect `admit == false` and not write the candidate.
#[must_use]
pub fn evaluate_admission(content: &str, role: &str) -> AdmissionDecision {
    // System and tool messages are never admitted.
    if role == "system" || role == "tool" {
        return AdmissionDecision {
            admit: false,
            score: 0.0,
            reason: AdmissionReason::ExcludedRole,
        };
    }

    // Content too short to be meaningful.
    if content.len() < 10 {
        return AdmissionDecision {
            admit: false,
            score: 0.0,
            reason: AdmissionReason::TooShort,
        };
    }

    // Sensitive content is always rejected before any write.
    if is_sensitive_content(content) {
        return AdmissionDecision {
            admit: false,
            score: 0.0,
            reason: AdmissionReason::SensitiveContent,
        };
    }

    // Conversational noise is always rejected.
    if is_noise_content(content) {
        return AdmissionDecision {
            admit: false,
            score: 0.0,
            reason: AdmissionReason::BelowThreshold,
        };
    }

    // Compute admission score: novelty × committed_utility.
    let score = compute_admission_score(content, role);

    AdmissionDecision {
        admit: score >= ADMISSION_THRESHOLD,
        score,
        reason: if score >= ADMISSION_THRESHOLD {
            AdmissionReason::Admitted
        } else {
            AdmissionReason::BelowThreshold
        },
    }
}

/// Compute the admission score (novelty × committed_utility) for content.
///
/// Novelty is based on content markers, NOT message length. ADR-046 envisions
/// novelty computed against existing memory coverage; the current implementation
/// is a documented keyword-based downgrade.
fn compute_admission_score(content: &str, _role: &str) -> f64 {
    let lower = content.trim_start().to_lowercase();

    // Novelty: corrections and technical content are inherently novel.
    // Length alone is NOT a novelty signal.
    let novelty: f64 = if lower.starts_with("actually")
        || lower.starts_with("no,")
        || lower.starts_with("correction")
    {
        0.9 // Corrections are inherently novel
    } else if lower.starts_with("note")
        || lower.starts_with("important")
        || lower.contains("fix for")
        || lower.contains("deadlock")
        || lower.contains("caused by")
    {
        0.8 // Technical notes and fixes
    } else if lower.starts_with("remember")
        || lower.starts_with("always")
        || lower.starts_with("never")
    {
        0.7 // Directives and explicit memory requests
    } else if lower.starts_with("prefer") || lower.contains("i prefer") {
        0.6 // Preferences
    } else {
        0.1 // Default: likely not novel
    };

    // Committed utility: did the information change/guide behavior?
    let utility: f64 = if lower.starts_with("always")
        || lower.starts_with("never")
        || lower.starts_with("remember")
        || lower.starts_with("note")
    {
        0.9
    } else if lower.starts_with("actually")
        || lower.starts_with("no,")
        || lower.contains("was wrong")
        || lower.contains("is wrong")
    {
        0.85
    } else if lower.starts_with("important")
        || lower.contains("fix for")
        || lower.contains("caused by")
        || lower.contains("fixed it")
    {
        0.8
    } else if lower.starts_with("prefer") || lower.contains("i prefer") {
        0.7
    } else {
        0.05 // Default: low utility (was 0.1)
    };
    novelty * utility
}

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
    /// Compatibility-only admission score published during the I138
    /// experiment. The production extractor does not use this field for
    /// admission or ordering after the I137 No-Go correction.
    pub admission_score: f64,
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
            if episode.content.len() < 10 {
                continue;
            }
            if episode.role == "system" || episode.role == "tool" {
                continue;
            }
            if episode.role != "user" && episode.role != "assistant" {
                continue;
            }
            // Credential-shaped content is a hard safety rejection independent
            // of the I137 policy experiment.
            if is_sensitive_content(&episode.content) {
                continue;
            }

            let key = derive_key(&episode.content);
            let confidence = compute_confidence(&episode.content, &episode.role);

            candidates.push(MemoryCandidate {
                kind: MemoryKind::Semantic,
                key,
                content: episode.content.clone(),
                confidence,
                admission_score: confidence,
                source_session_id: episode.session_id.clone(),
                source_entry_id: episode.entry_id.clone(),
                source_turn_index: episode.turn_index,
            });
        }

        // I137 selected No-Go, so the production extractor retains its
        // pre-experiment confidence ordering.
        candidates.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.source_turn_index.cmp(&b.source_turn_index))
        });

        candidates.truncate(self.max_candidates);
        candidates
    }
}

/// Compute confidence using the production baseline retained by I137 No-Go.
fn compute_confidence(content: &str, role: &str) -> f64 {
    let lower = content.trim_start().to_lowercase();
    let markers = ["remember", "note", "important", "always", "never"];
    if markers.iter().any(|marker| lower.starts_with(marker)) {
        return 0.8;
    }
    if role == "user" && content.len() > 50 {
        return 0.6;
    }
    0.4
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
                "Note: Python is a dynamically typed programming language with great ecosystem",
            ),
            make_episode(
                "sess-2",
                "entry-b",
                1,
                "user",
                "Note: Python is a statically typed programming language with strict type checking",
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

    // ── ADR-046 admission tests ──────────────────────────────────────────

    #[test]
    fn sensitive_content_is_rejected_before_write() {
        let decision = evaluate_admission("api_key = sk-ant-1234567890abcdef", "user");
        assert!(!decision.admit, "api_key content must not be admitted");
        assert_eq!(decision.reason, AdmissionReason::SensitiveContent);

        let decision2 = evaluate_admission("Authorization: Bearer token123", "user");
        assert!(
            !decision2.admit,
            "Authorization header must not be admitted"
        );
        assert_eq!(decision2.reason, AdmissionReason::SensitiveContent);

        let decision3 = evaluate_admission("password = secret123", "user");
        assert!(!decision3.admit, "password content must not be admitted");
        assert_eq!(decision3.reason, AdmissionReason::SensitiveContent);

        // Prose-form credentials (review fix)
        let prose1 = evaluate_admission("my password is hunter2xyz", "user");
        assert!(!prose1.admit, "prose-form password must not be admitted");
        assert_eq!(prose1.reason, AdmissionReason::SensitiveContent);

        let prose2 = evaluate_admission("the deploy secret is prod-live-9f8a", "user");
        assert!(!prose2.admit, "prose-form secret must not be admitted");
        assert_eq!(prose2.reason, AdmissionReason::SensitiveContent);

        let prose3 = evaluate_admission("session token is abc123", "user");
        assert!(!prose3.admit, "prose-form token must not be admitted");

        // sk- false positive fix: ordinary words must NOT be rejected
        let normal1 = evaluate_admission("Always break the task-list into subtasks", "user");
        assert!(
            normal1.reason != AdmissionReason::SensitiveContent,
            "'task-list' must not be flagged as sk- credential"
        );

        let normal2 = evaluate_admission("Important: the risk-averse policy applies here", "user");
        assert!(
            normal2.reason != AdmissionReason::SensitiveContent,
            "'risk-averse' must not be flagged as sk- credential"
        );

        // Noise contains fix: mid-sentence noise words must NOT reject
        let substantive1 = evaluate_admission(
            "Note: make sure the deadlock fix lands before release",
            "user",
        );
        assert!(
            substantive1.admit,
            "'make sure' mid-sentence must not be treated as noise: {:?}",
            substantive1.reason
        );

        let substantive2 =
            evaluate_admission("Note: the API returns ok status codes on success", "user");
        assert!(
            substantive2.admit,
            "'ok' mid-sentence must not be treated as noise: {:?}",
            substantive2.reason
        );
    }

    #[test]
    fn short_correction_is_admitted() {
        let decision = evaluate_admission("No, use cargo fmt not rustfmt", "user");
        assert!(decision.admit, "short correction should be admitted");
        assert_eq!(decision.reason, AdmissionReason::Admitted);
        assert!(decision.score > 0.5, "correction should have high score");
    }

    #[test]
    fn long_noise_is_rejected() {
        let long_noise = "Can you help me understand how the session lifecycle works? \
            I've been reading through the code and trying to figure out the relationship. \
            It seems like there's a turn loop that processes events and then calls tools.";
        let decision = evaluate_admission(long_noise, "user");
        assert!(
            !decision.admit,
            "routine chatter must be rejected (admit={}), score={}",
            decision.admit, decision.score
        );
    }

    #[test]
    fn no_go_keeps_memory_candidate_shape_and_baseline_confidence() {
        let episodes = vec![SessionEpisode {
            session_id: "s1".into(),
            entry_id: "e1".into(),
            turn_index: 0,
            role: "user".into(),
            content: "Always run tests before committing".into(),
            timestamp: Utc::now(),
        }];
        let extractor = RuleBasedExtractor::new();
        let candidates = extractor.extract(&episodes);
        assert!(!candidates.is_empty());
        let c = &candidates[0];
        assert_eq!(c.confidence, 0.8);
        assert_eq!(c.admission_score, c.confidence);
        let cloned = c.clone();
        assert_eq!(cloned.key, c.key);
        assert!(format!("{cloned:?}").contains("MemoryCandidate"));
    }

    #[test]
    fn disabled_mode_admits_nothing() {
        let mut store = MemoryStore::open_memory().unwrap();
        let episodes = vec![SessionEpisode {
            session_id: "s1".into(),
            entry_id: "e1".into(),
            turn_index: 0,
            role: "user".into(),
            content: "Always run tests before committing".into(),
            timestamp: Utc::now(),
        }];
        let extractor = RuleBasedExtractor::new();
        let config = ConsolidationConfig::default(); // enabled = false
        let report = consolidate_episodes(&mut store, &episodes, &extractor, &config).unwrap();
        assert_eq!(
            report.inserted, 0,
            "disabled consolidation should write nothing"
        );
    }

    #[test]
    fn credential_shaped_content_not_in_store() {
        let mut store = MemoryStore::open_memory().unwrap();
        let episodes = vec![SessionEpisode {
            session_id: "s1".into(),
            entry_id: "e1".into(),
            turn_index: 0,
            role: "user".into(),
            content: "api_key = sk-ant-test-secret-key-value".into(),
            timestamp: Utc::now(),
        }];
        let extractor = RuleBasedExtractor::new();
        let config = ConsolidationConfig {
            enabled: true,
            max_candidates_per_session: 20,
        };
        let report = consolidate_episodes(&mut store, &episodes, &extractor, &config).unwrap();
        assert_eq!(
            report.candidates_extracted, 0,
            "sensitive content should not be extracted"
        );
        assert_eq!(
            report.inserted, 0,
            "sensitive content must not be written to store"
        );
        let results = store.retrieve("api_key", 10).unwrap();
        assert!(results.is_empty(), "no sensitive content in store");
    }
}
