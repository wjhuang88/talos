//! Evolution runtime pipeline — wires the I008 self-evolution engine into the
//! CLI execution path (story #I008, residual R1/R2).
//!
//! Implements the runtime half of ADR-001's learning loop:
//!
//! - **Observe**: capture objective [`AgentEvent::Error`] signals during a turn,
//!   plus user-correction signals detected heuristically from prompt text.
//! - **Accumulate**: persist observations and merge repeated evidence into an
//!   existing pattern (deduplicated by `category` + `description`) so confidence
//!   only crosses the injection threshold after sufficient corroboration.
//! - **Extract**: promote observations into patterns via [`PatternExtractor`].
//! - **Apply**: inject high-confidence patterns into the system prompt via
//!   [`BehaviorAdapter`].
//!
//! All storage stays on the CLI layer; the agent core never touches the
//! `!Sync` SQLite connection. The pipeline is best-effort — storage failures
//! degrade evolution silently rather than breaking agent execution.

use anyhow::{Context, Result};
use talos_core::message::AgentEvent;
use talos_evolution::adapter::BehaviorAdapter;
use talos_evolution::extractor::PatternExtractor;
use talos_evolution::observer::TurnObserver;
use talos_evolution::store::KnowledgeStore;
use talos_evolution::EvolutionConfig;

/// Intensity assigned to an objective agent error signal.
const ERROR_INTENSITY: f64 = 1.0;

/// Intensity assigned to a heuristically-detected user correction.
const CORRECTION_INTENSITY: f64 = 0.8;

/// Lowercased keyword fragments that suggest the user is correcting the agent.
const CORRECTION_MARKERS: &[&str] = &[
    "don't",
    "do not",
    "instead",
    "actually",
    "that's wrong",
    "thats wrong",
    "is wrong",
    "should be",
    "no, ",
    "不对",
    "不要",
    "别",
    "错了",
];

/// Runtime owner of the evolution learning loop for a single CLI invocation.
///
/// Holds the knowledge store, evolution configuration, and the per-turn
/// observer. Construct via [`EvolutionRuntime::open_default`] in production or
/// [`EvolutionRuntime::with_store`] (e.g. with an in-memory store) for tests.
pub struct EvolutionRuntime {
    store: KnowledgeStore,
    config: EvolutionConfig,
    observer: TurnObserver,
}

impl EvolutionRuntime {
    /// Open the default knowledge store at `~/.talos/index.db`, creating the
    /// `.talos` directory if it does not exist.
    ///
    /// Returns `Ok(None)` when the home directory cannot be resolved, allowing
    /// the caller to continue without evolution rather than aborting the run.
    pub fn open_default(session_id: Option<String>) -> Result<Option<Self>> {
        let Some(home) = dirs::home_dir() else {
            return Ok(None);
        };
        let dir = home.join(".talos");
        std::fs::create_dir_all(&dir).context("failed to create .talos directory")?;
        let db_path = dir.join("index.db");
        let store = KnowledgeStore::open(db_path.to_str().unwrap_or_default())
            .context("failed to open knowledge store")?;
        Ok(Some(Self::with_store(
            store,
            EvolutionConfig::default(),
            session_id,
        )))
    }

    /// Construct a runtime around an explicit store and configuration.
    ///
    /// Primarily used by tests with an in-memory [`KnowledgeStore`].
    pub fn with_store(
        store: KnowledgeStore,
        config: EvolutionConfig,
        session_id: Option<String>,
    ) -> Self {
        Self {
            store,
            config,
            observer: TurnObserver::new(session_id),
        }
    }

    /// Build the evolution context block to inject into the system prompt.
    ///
    /// Returns an empty string when no patterns clear the configured confidence
    /// and evidence thresholds.
    pub fn evolution_context(&self) -> String {
        BehaviorAdapter::new(&self.store, self.config.clone()).get_evolution_context()
    }

    /// Begin a new observation turn, clearing any pending observations.
    pub fn start_turn(&mut self) {
        self.observer.start_turn();
    }

    /// Inspect a streamed agent event and record an objective error signal when
    /// the turn fails. Non-error events are ignored.
    pub fn observe_event(&mut self, event: &AgentEvent) {
        if let AgentEvent::Error { message } = event {
            self.observer.record_error(message.clone(), ERROR_INTENSITY);
        }
    }

    /// Heuristically record a user-correction signal from raw prompt text when
    /// it contains a known correction marker.
    pub fn observe_user_input(&mut self, input: &str) {
        if let Some(intensity) = detect_correction(input) {
            self.observer
                .record_correction(input.trim().to_string(), intensity);
        }
    }

    /// Persist all observations from the current turn, then extract and
    /// accumulate patterns.
    ///
    /// For each observation that yields a pattern candidate, an existing
    /// pattern with the same `category` and `description` has its evidence
    /// merged (raising confidence and evidence count); otherwise the candidate
    /// is inserted fresh. This accumulation is what allows repeated, identical
    /// signals to eventually cross the injection threshold.
    pub fn ingest(&mut self) -> Result<()> {
        let observations = self.observer.drain_observations();

        for obs in &observations {
            self.store
                .insert_observation(obs)
                .context("failed to persist observation")?;

            let Some(candidate) = PatternExtractor::extract_from_observation(obs) else {
                continue;
            };

            let existing = self
                .store
                .get_all_patterns()
                .context("failed to load existing patterns")?;

            let matched = existing.into_iter().find(|p| {
                p.category == candidate.category && p.description == candidate.description
            });

            match matched {
                Some(mut pattern) => {
                    PatternExtractor::merge_evidence(&mut pattern, std::slice::from_ref(obs));
                    self.store
                        .update_pattern(&pattern)
                        .context("failed to update accumulated pattern")?;
                }
                None => {
                    self.store
                        .insert_pattern(&candidate)
                        .context("failed to insert new pattern")?;
                }
            }
        }

        Ok(())
    }
}

/// Detect whether `input` looks like a user correction, returning the signal
/// intensity when a known marker is present.
pub fn detect_correction(input: &str) -> Option<f64> {
    let lower = input.to_lowercase();
    if CORRECTION_MARKERS.iter().any(|marker| lower.contains(marker)) {
        Some(CORRECTION_INTENSITY)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn error_event(message: &str) -> AgentEvent {
        AgentEvent::Error {
            message: message.to_string(),
        }
    }

    fn runtime() -> EvolutionRuntime {
        let store = KnowledgeStore::open_memory().expect("in-memory store");
        EvolutionRuntime::with_store(store, EvolutionConfig::default(), Some("test".to_string()))
    }

    #[test]
    fn detect_correction_matches_markers() {
        assert_eq!(detect_correction("No, don't do that"), Some(0.8));
        assert_eq!(detect_correction("Actually use a HashMap instead"), Some(0.8));
        assert_eq!(detect_correction("这样不对"), Some(0.8));
        assert!(detect_correction("please continue with the plan").is_none());
        assert!(detect_correction("write a function").is_none());
    }

    #[test]
    fn single_error_does_not_inject() {
        let mut rt = runtime();
        rt.start_turn();
        rt.observe_event(&error_event("compilation failed: missing semicolon"));
        rt.ingest().unwrap();
        assert!(
            rt.evolution_context().is_empty(),
            "a single error is below the confidence/evidence threshold"
        );
    }

    #[test]
    fn repeated_identical_errors_accumulate_into_injectable_pattern() {
        let mut rt = runtime();
        let message = "compilation failed: missing semicolon";

        // Turn 1: confidence 0.4, evidence 1 — below threshold.
        rt.start_turn();
        rt.observe_event(&error_event(message));
        rt.ingest().unwrap();
        assert!(rt.evolution_context().is_empty());

        // Turn 2: confidence 0.7, evidence 2 — evidence still below min (3).
        rt.start_turn();
        rt.observe_event(&error_event(message));
        rt.ingest().unwrap();
        assert!(rt.evolution_context().is_empty());

        // Turn 3: confidence 0.85, evidence 3 — crosses both thresholds.
        rt.start_turn();
        rt.observe_event(&error_event(message));
        rt.ingest().unwrap();

        let context = rt.evolution_context();
        assert!(
            context.contains("Learned Patterns"),
            "third identical error should produce an injectable pattern, got: {context:?}"
        );
        assert!(context.contains(message));
    }

    #[test]
    fn distinct_errors_do_not_merge() {
        let mut rt = runtime();
        for message in ["error alpha", "error beta", "error gamma"] {
            rt.start_turn();
            rt.observe_event(&error_event(message));
            rt.ingest().unwrap();
        }
        // Three distinct patterns, each with evidence 1 — none injectable.
        assert!(rt.evolution_context().is_empty());
    }

    #[test]
    fn non_error_events_are_ignored() {
        let mut rt = runtime();
        rt.start_turn();
        rt.observe_event(&AgentEvent::TurnStart);
        rt.observe_event(&AgentEvent::TextDelta {
            delta: "hello".to_string(),
        });
        rt.ingest().unwrap();
        assert!(rt.evolution_context().is_empty());
    }
}
