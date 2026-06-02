//! Evolution `HookHandler` — wires the I008 self-evolution engine (ADR-001)
//! into any agent run path via the I009 hook system (see
//! [`talos_plugin::handler::HookHandler`]).
//!
//! One registered handler covers all three CLI run paths (print / interactive /
//! tui) uniformly: the agent's [`run_inner`][1] fires the subscribed events
//! once per turn, the same way for every path, so per-Agent registration
//! guarantees no double-firing. The pre-#I008 concern that "evolution must
//! attach once at a future `AppServerSession` seam" is satisfied at the hook
//! layer instead; see [ADR-005 → "Hook-Driven
//! Evolution"](../docs/decisions/005-tui-event-architecture.md#hook-driven-evolution-2026-06-01-pre-i008-re-scope).
//!
//! Capability mapping to the four-phase learning loop (ADR-001):
//!
//! | Phase | Hook event(s) |
//! |-------|---------------|
//! | Observe | `OnProviderError` (objective error), `BeforeProviderCall` (user-correction heuristic) |
//! | Accumulate | Handler-internal `Mutex<TurnObserver>` (reset on `TurnStart`) |
//! | Extract | `PatternExtractor::extract_from_observation` at flush time |
//! | Apply | `OnSystemPromptBuilt` + `HookResult::Modify` returns augmented prompt |
//! | Ingest | Flush in `TurnComplete` (overridden timeout = 5s for SQLite write) |
//!
//! [1]: talos_agent::Agent::run_inner

use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use talos_core::message::Message;
use talos_plugin::event::{HookEvent, HookEventKind};
use talos_plugin::handler::{HookContext, HookHandler, HookResult};

use crate::EvolutionConfig;
use crate::adapter::BehaviorAdapter;
use crate::extractor::PatternExtractor;
use crate::observer::TurnObserver;
use crate::store::KnowledgeStore;

const ERROR_INTENSITY: f64 = 1.0;
const CORRECTION_INTENSITY: f64 = 0.8;

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

/// Hook handler implementing the I008 self-evolution loop. Registered per-Agent
/// in the [`HookRegistry`][1] alongside `LoggingHandler`.
///
/// The same `Arc<EvolutionHookHandler>` instance is reused across all turns
/// for the Agent's lifetime; accumulation is stateful via interior
/// mutability (`Mutex<TurnObserver>` + `Mutex<KnowledgeStore>`).
///
/// [1]: talos_plugin::registry::HookRegistry
pub struct EvolutionHookHandler {
    store: Mutex<KnowledgeStore>,
    config: EvolutionConfig,
    observer: Mutex<TurnObserver>,
}

impl EvolutionHookHandler {
    /// Build a handler backed by the given store, config, and session ID.
    #[must_use]
    pub fn new(store: KnowledgeStore, config: EvolutionConfig, session_id: Option<String>) -> Self {
        Self {
            store: Mutex::new(store),
            config,
            observer: Mutex::new(TurnObserver::new(session_id)),
        }
    }

    /// Open the default knowledge store at `~/.talos/index.db`, creating the
    /// `.talos` directory if it does not exist.
    ///
    /// Returns `Ok(None)` when the home directory cannot be resolved, allowing
    /// the caller to continue without evolution rather than aborting the run.
    pub fn open_default(
        config: EvolutionConfig,
        session_id: Option<String>,
    ) -> anyhow::Result<Option<Self>> {
        use anyhow::Context;
        let Some(home) = dirs::home_dir() else {
            return Ok(None);
        };
        let dir = home.join(".talos");
        std::fs::create_dir_all(&dir).context("failed to create .talos directory")?;
        let db_path = dir.join("index.db");
        let store = KnowledgeStore::open(db_path.to_str().unwrap_or_default())
            .context("failed to open knowledge store")?;
        Ok(Some(Self::new(store, config, session_id)))
    }

    fn evolution_context(&self) -> String {
        let store = self.store.lock().expect("evolution store poisoned");
        BehaviorAdapter::new(&store, self.config.clone()).get_evolution_context()
    }

    fn flush(&self) {
        let observations = {
            let mut observer = self.observer.lock().expect("evolution observer poisoned");
            observer.drain_observations()
        };
        if observations.is_empty() {
            return;
        }

        let store = self.store.lock().expect("evolution store poisoned");
        for obs in &observations {
            if let Err(e) = store.insert_observation(obs) {
                tracing::warn!(error = %e, "evolution: failed to persist observation");
                continue;
            }
            let Some(candidate) = PatternExtractor::extract_from_observation(obs) else {
                continue;
            };
            let existing = match store.get_all_patterns() {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!(error = %e, "evolution: failed to load existing patterns");
                    continue;
                }
            };
            let matched = existing.into_iter().find(|p| {
                p.category == candidate.category && p.description == candidate.description
            });
            let result = match matched {
                Some(mut pattern) => {
                    PatternExtractor::merge_evidence(&mut pattern, std::slice::from_ref(obs));
                    store.update_pattern(&pattern)
                }
                None => store.insert_pattern(&candidate),
            };
            if let Err(e) = result {
                tracing::warn!(error = %e, "evolution: failed to update pattern");
            }
        }
    }
}

#[async_trait]
impl HookHandler for EvolutionHookHandler {
    fn name(&self) -> &str {
        "evolution"
    }

    fn subscribed(&self) -> &'static [HookEventKind] {
        &[
            HookEventKind::TurnStart,
            HookEventKind::OnSystemPromptBuilt,
            HookEventKind::BeforeProviderCall,
            HookEventKind::OnProviderError,
            HookEventKind::OnTextDelta,
            HookEventKind::OnToolResultObserved,
            HookEventKind::AfterToolCall,
            HookEventKind::OnTurnEnd,
            HookEventKind::TurnComplete,
        ]
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(5)
    }

    async fn on_event(&self, _ctx: &HookContext, event: &mut HookEvent<'_>) -> HookResult {
        match event {
            HookEvent::TurnStart { .. } => {
                let mut observer = self.observer.lock().expect("evolution observer poisoned");
                observer.start_turn();
                HookResult::Continue
            }
            HookEvent::OnSystemPromptBuilt { prompt } => {
                let context = self.evolution_context();
                if context.is_empty() {
                    return HookResult::Continue;
                }
                // The 'static bound on HookResult::Modify(HookEvent<'static>) forces
                // us to leak the augmented prompt. One small permanent allocation
                // per turn (typically a few KB). The long-term fix is an additive
                // HookResult::ModifyOwned variant (tracked separately, out of
                // I008 scope; see ADR-005 → "Hook-Driven Evolution").
                let augmented = format!("{prompt}\n\n{context}");
                HookResult::Modify(HookEvent::OnSystemPromptBuilt {
                    prompt: Box::leak(augmented.into_boxed_str()),
                })
            }
            HookEvent::BeforeProviderCall { messages } => {
                if let Some(text) = messages.iter().find_map(|m| match m {
                    Message::User { content } => Some(content.as_str()),
                    _ => None,
                }) {
                    if let Some(intensity) = detect_correction(text) {
                        let mut observer =
                            self.observer.lock().expect("evolution observer poisoned");
                        observer.record_correction(text.to_string(), intensity);
                    }
                }
                HookResult::Continue
            }
            HookEvent::OnProviderError { error } => {
                let message = format!("{error:?}");
                let mut observer = self.observer.lock().expect("evolution observer poisoned");
                observer.record_error(message, ERROR_INTENSITY);
                HookResult::Continue
            }
            HookEvent::TurnComplete { .. } => {
                self.flush();
                HookResult::Continue
            }
            HookEvent::OnTurnEnd { .. }
            | HookEvent::OnTextDelta { .. }
            | HookEvent::OnToolResultObserved { .. }
            | HookEvent::AfterToolCall { .. } => HookResult::Continue,
            _ => HookResult::Continue,
        }
    }
}

fn detect_correction(input: &str) -> Option<f64> {
    let lower = input.to_lowercase();
    if CORRECTION_MARKERS
        .iter()
        .any(|marker| lower.contains(marker))
    {
        Some(CORRECTION_INTENSITY)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Pattern, SignalType};
    use std::path::PathBuf;
    use talos_core::provider::ProviderError;

    use talos_plugin::event::{TurnId, TurnStatus};

    fn handler() -> EvolutionHookHandler {
        let store = KnowledgeStore::open_memory().expect("in-memory store");
        EvolutionHookHandler::new(store, EvolutionConfig::default(), Some("test".into()))
    }

    fn ctx() -> HookContext {
        HookContext::new(TurnId::new(), PathBuf::from("."))
    }

    #[tokio::test]
    async fn turn_start_then_turn_complete_with_no_observations_is_noop() {
        let h = handler();
        let c = ctx();
        h.on_event(&c, &mut HookEvent::TurnStart { turn_id: c.turn_id })
            .await;
        h.on_event(
            &c,
            &mut HookEvent::TurnComplete {
                turn_id: c.turn_id,
                status: TurnStatus::Success,
            },
        )
        .await;
        let store = h.store.lock().expect("store poisoned");
        assert!(store.get_observations().unwrap().is_empty());
    }

    #[tokio::test]
    async fn provider_error_records_observation_and_persists() {
        let h = handler();
        let c = ctx();
        let err = ProviderError::ServerError("test failure".into());
        h.on_event(&c, &mut HookEvent::TurnStart { turn_id: c.turn_id })
            .await;
        h.on_event(&c, &mut HookEvent::OnProviderError { error: &err })
            .await;
        h.on_event(
            &c,
            &mut HookEvent::TurnComplete {
                turn_id: c.turn_id,
                status: TurnStatus::ProviderError,
            },
        )
        .await;

        let store = h.store.lock().expect("store poisoned");
        let observations = store.get_observations().unwrap();
        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0].signal_type, SignalType::Error);
    }

    #[tokio::test]
    async fn repeated_errors_accumulate_into_injectable_pattern() {
        let h = handler();
        let c = ctx();
        for _ in 0..3 {
            let err = ProviderError::ServerError("compilation failed".into());
            h.on_event(&c, &mut HookEvent::TurnStart { turn_id: c.turn_id })
                .await;
            h.on_event(&c, &mut HookEvent::OnProviderError { error: &err })
                .await;
            h.on_event(
                &c,
                &mut HookEvent::TurnComplete {
                    turn_id: c.turn_id,
                    status: TurnStatus::ProviderError,
                },
            )
            .await;
        }
        let context = h.evolution_context();
        assert!(
            context.contains("Learned Patterns"),
            "third identical error should produce an injectable pattern, got: {context:?}"
        );
    }

    #[tokio::test]
    async fn distinct_errors_do_not_merge() {
        let h = handler();
        let c = ctx();
        for message in ["alpha", "beta", "gamma"] {
            let err = ProviderError::ServerError(message.into());
            h.on_event(&c, &mut HookEvent::TurnStart { turn_id: c.turn_id })
                .await;
            h.on_event(&c, &mut HookEvent::OnProviderError { error: &err })
                .await;
            h.on_event(
                &c,
                &mut HookEvent::TurnComplete {
                    turn_id: c.turn_id,
                    status: TurnStatus::ProviderError,
                },
            )
            .await;
        }
        assert!(h.evolution_context().is_empty());
    }

    #[tokio::test]
    async fn on_system_prompt_built_returns_continue_when_no_patterns() {
        let h = handler();
        let c = ctx();
        let mut event = HookEvent::OnSystemPromptBuilt {
            prompt: "base prompt",
        };
        let outcome = h.on_event(&c, &mut event).await;
        assert!(matches!(outcome, HookResult::Continue));
    }

    #[tokio::test]
    async fn on_system_prompt_built_returns_modify_with_context() {
        let h = handler();
        {
            let store = h.store.lock().expect("store poisoned");
            let mut pattern = Pattern::new(
                "Prefer local state".into(),
                "Avoid global mutable state in library code".into(),
                "preference".into(),
            );
            pattern.confidence = 0.9;
            pattern.evidence_count = 5;
            store.insert_pattern(&pattern).unwrap();
        }

        let c = ctx();
        let mut event = HookEvent::OnSystemPromptBuilt { prompt: "BASE" };
        let outcome = h.on_event(&c, &mut event).await;
        match outcome {
            HookResult::Modify(HookEvent::OnSystemPromptBuilt { prompt }) => {
                assert!(
                    prompt.contains("BASE"),
                    "augmented prompt must keep the original prefix"
                );
                assert!(prompt.contains("Learned Patterns"));
                assert!(prompt.contains("Avoid global mutable state"));
            }
            other => panic!("expected Modify, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn before_provider_call_detects_user_correction() {
        let h = handler();
        let c = ctx();
        let messages = vec![Message::User {
            content: "No, don't do that, use a HashMap instead".into(),
        }];
        h.on_event(
            &c,
            &mut HookEvent::BeforeProviderCall {
                messages: &messages,
            },
        )
        .await;
        h.on_event(
            &c,
            &mut HookEvent::TurnComplete {
                turn_id: c.turn_id,
                status: TurnStatus::Success,
            },
        )
        .await;
        let store = h.store.lock().expect("store poisoned");
        let observations = store.get_observations().unwrap();
        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0].signal_type, SignalType::Correction);
    }

    #[tokio::test]
    async fn before_provider_call_ignores_non_correction_input() {
        let h = handler();
        let c = ctx();
        let messages = vec![Message::User {
            content: "Please continue with the plan.".into(),
        }];
        h.on_event(
            &c,
            &mut HookEvent::BeforeProviderCall {
                messages: &messages,
            },
        )
        .await;
        h.on_event(
            &c,
            &mut HookEvent::TurnComplete {
                turn_id: c.turn_id,
                status: TurnStatus::Success,
            },
        )
        .await;
        let store = h.store.lock().expect("store poisoned");
        assert!(store.get_observations().unwrap().is_empty());
    }

    #[tokio::test]
    async fn flush_resets_observer_for_next_turn() {
        let h = handler();
        let c = ctx();
        let err = ProviderError::ServerError("first turn".into());
        h.on_event(&c, &mut HookEvent::TurnStart { turn_id: c.turn_id })
            .await;
        h.on_event(&c, &mut HookEvent::OnProviderError { error: &err })
            .await;
        h.on_event(
            &c,
            &mut HookEvent::TurnComplete {
                turn_id: c.turn_id,
                status: TurnStatus::ProviderError,
            },
        )
        .await;

        h.on_event(&c, &mut HookEvent::TurnStart { turn_id: c.turn_id })
            .await;
        h.on_event(
            &c,
            &mut HookEvent::TurnComplete {
                turn_id: c.turn_id,
                status: TurnStatus::Success,
            },
        )
        .await;
        let store = h.store.lock().expect("store poisoned");
        let observations = store.get_observations().unwrap();
        assert_eq!(observations.len(), 1, "second turn added nothing");
    }

    #[test]
    fn detect_correction_matches_known_markers() {
        assert!(detect_correction("No, don't do that").is_some());
        assert!(detect_correction("Actually use a HashMap instead").is_some());
        assert!(detect_correction("这样不对").is_some());
        assert!(detect_correction("please continue with the plan").is_none());
        assert!(detect_correction("write a function").is_none());
    }

    #[test]
    fn subscribed_kinds_match_audit() {
        let h = handler();
        let kinds = h.subscribed();
        assert!(kinds.contains(&HookEventKind::TurnStart));
        assert!(kinds.contains(&HookEventKind::OnSystemPromptBuilt));
        assert!(kinds.contains(&HookEventKind::BeforeProviderCall));
        assert!(kinds.contains(&HookEventKind::OnProviderError));
        assert!(kinds.contains(&HookEventKind::TurnComplete));
    }

    #[test]
    fn handler_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<EvolutionHookHandler>();
    }

    #[test]
    fn timeout_allows_sqlite_flush() {
        let h = handler();
        assert!(h.timeout() >= Duration::from_secs(1));
    }
}
