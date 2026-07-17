//! MEM-009 deterministic admission benchmark (I137 corrective review).
//!
//! The benchmark compares five policies on one frozen corpus and emits a
//! byte-stable JSON report. It does not mutate runtime admission behavior.

use std::collections::{BTreeMap, HashSet};

use crate::consolidation::{AdmissionReason, evaluate_admission, is_sensitive_content};

#[derive(Debug, Clone, Copy)]
struct Fixture {
    id: &'static str,
    content: &'static str,
    role: &'static str,
    expected_admit: bool,
    category: &'static str,
}

const FIXTURES: &[Fixture] = &[
    Fixture {
        id: "correction-path",
        content: "Actually, the config file is at ~/.talos/config.toml not /etc/talos.conf",
        role: "user",
        expected_admit: true,
        category: "correction",
    },
    Fixture {
        id: "correction-command",
        content: "No, use cargo fmt not rustfmt directly",
        role: "user",
        expected_admit: true,
        category: "correction",
    },
    Fixture {
        id: "preference-theme",
        content: "I prefer Nord theme",
        role: "user",
        expected_admit: true,
        category: "preference",
    },
    Fixture {
        id: "preference-tests",
        content: "Always run tests before committing",
        role: "user",
        expected_admit: true,
        category: "preference",
    },
    Fixture {
        id: "old-important-secrets",
        content: "Never commit secrets to the repository",
        role: "user",
        expected_admit: true,
        category: "old-important",
    },
    Fixture {
        id: "contradiction-sqlite",
        content: "Note: the previous statement about SQLite being required is wrong; we use bundled mode",
        role: "user",
        expected_admit: true,
        category: "contradiction",
    },
    Fixture {
        id: "validated-timeout",
        content: "Important: the fix for issue #18 was caused by missing dispatch timeout",
        role: "assistant",
        expected_admit: true,
        category: "validated",
    },
    Fixture {
        id: "recovery-retry",
        content: "Important: recovery succeeded after the bounded retry released the SQLite lock",
        role: "assistant",
        expected_admit: true,
        category: "recovery",
    },
    Fixture {
        id: "recency-conflict-endpoint",
        content: "Correction: use the v2 endpoint; the older v1 endpoint is obsolete",
        role: "user",
        expected_admit: true,
        category: "recency-conflict",
    },
    Fixture {
        id: "sensitive-api-key",
        content: "api_key = sk-ant-fixture-redacted",
        role: "user",
        expected_admit: false,
        category: "sensitive",
    },
    Fixture {
        id: "sensitive-authorization",
        content: "Authorization: Bearer fixture-redacted",
        role: "user",
        expected_admit: false,
        category: "sensitive",
    },
    Fixture {
        id: "sensitive-password",
        content: "password = fixture-redacted",
        role: "user",
        expected_admit: false,
        category: "sensitive",
    },
    Fixture {
        id: "chatter-hello",
        content: "Hello, how are you today?",
        role: "user",
        expected_admit: false,
        category: "chatter",
    },
    Fixture {
        id: "chatter-thanks",
        content: "Thanks, that's helpful",
        role: "user",
        expected_admit: false,
        category: "chatter",
    },
    Fixture {
        id: "routine-question",
        content: "Can you help me understand how the session lifecycle works? I've been reading through the code.",
        role: "user",
        expected_admit: false,
        category: "routine-length",
    },
    Fixture {
        id: "duplicate-tests",
        content: "Always run tests before committing",
        role: "user",
        expected_admit: false,
        category: "duplicate",
    },
];

#[derive(Debug, Clone, Copy)]
enum Policy {
    CurrentHeuristic,
    Recency,
    NoveltyOnly,
    UtilityOnly,
    NoveltyTimesUtility,
}

impl Policy {
    const ALL: [Self; 5] = [
        Self::CurrentHeuristic,
        Self::Recency,
        Self::NoveltyOnly,
        Self::UtilityOnly,
        Self::NoveltyTimesUtility,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::CurrentHeuristic => "current_heuristic",
            Self::Recency => "recency",
            Self::NoveltyOnly => "novelty_only",
            Self::UtilityOnly => "committed_utility_only",
            Self::NoveltyTimesUtility => "novelty_times_committed_utility",
        }
    }

    fn evaluate(self, fixture: &Fixture, index: usize) -> (bool, &'static str) {
        match self {
            Self::CurrentHeuristic => {
                let lower = fixture.content.trim_start().to_lowercase();
                let marked = ["remember", "note", "important", "always", "never"]
                    .iter()
                    .any(|marker| lower.starts_with(marker));
                (
                    marked || fixture.role == "user" && fixture.content.len() > 50,
                    if marked {
                        "marker"
                    } else if fixture.role == "user" && fixture.content.len() > 50 {
                        "length"
                    } else {
                        "below_baseline"
                    },
                )
            }
            Self::Recency => {
                let recent = index >= FIXTURES.len().saturating_sub(8);
                (
                    recent && !is_sensitive_content(fixture.content),
                    if recent { "recent" } else { "stale" },
                )
            }
            Self::NoveltyOnly => {
                let lower = fixture.content.trim_start().to_lowercase();
                let novel = [
                    "actually",
                    "no,",
                    "correction",
                    "note",
                    "important",
                    "always",
                    "never",
                    "remember",
                    "prefer",
                ]
                .iter()
                .any(|marker| lower.starts_with(marker))
                    || lower.contains("i prefer");
                (
                    novel && !is_sensitive_content(fixture.content),
                    if novel { "novel_marker" } else { "low_novelty" },
                )
            }
            Self::UtilityOnly => {
                let lower = fixture.content.trim_start().to_lowercase();
                let useful = [
                    "actually",
                    "no,",
                    "correction",
                    "note",
                    "important",
                    "always",
                    "never",
                    "remember",
                    "prefer",
                    "fix for",
                    "caused by",
                    "succeeded after",
                ]
                .iter()
                .any(|marker| lower.starts_with(marker) || lower.contains(marker));
                (
                    useful && !is_sensitive_content(fixture.content),
                    if useful {
                        "utility_signal"
                    } else {
                        "low_utility"
                    },
                )
            }
            Self::NoveltyTimesUtility => {
                let decision = evaluate_admission(fixture.content, fixture.role);
                let reason = match decision.reason {
                    AdmissionReason::Admitted => "admitted",
                    AdmissionReason::SensitiveContent => "sensitive",
                    AdmissionReason::BelowThreshold => "below_threshold",
                    AdmissionReason::ExcludedRole => "excluded_role",
                    AdmissionReason::TooShort => "too_short",
                };
                (decision.admit, reason)
            }
        }
    }
}

#[derive(Debug)]
struct Metrics {
    policy: &'static str,
    tp: usize,
    fp: usize,
    tn: usize,
    fn_: usize,
    admitted_chars: usize,
    duplicate_admitted: bool,
    old_important_retained: bool,
    contradiction_correct: bool,
    failures: Vec<serde_json::Value>,
    reason_counts: BTreeMap<&'static str, usize>,
}

impl Metrics {
    fn precision(&self) -> f64 {
        (self.tp as f64) / ((self.tp + self.fp).max(1) as f64)
    }

    fn important_recall(&self) -> f64 {
        (self.tp as f64) / ((self.tp + self.fn_).max(1) as f64)
    }
}

fn evaluate_policy(policy: Policy) -> Metrics {
    let mut metrics = Metrics {
        policy: policy.name(),
        tp: 0,
        fp: 0,
        tn: 0,
        fn_: 0,
        admitted_chars: 0,
        duplicate_admitted: false,
        old_important_retained: false,
        contradiction_correct: false,
        failures: Vec::new(),
        reason_counts: BTreeMap::new(),
    };
    for (index, fixture) in FIXTURES.iter().enumerate() {
        let (actual, reason) = policy.evaluate(fixture, index);
        if actual {
            metrics.admitted_chars += fixture.content.len();
        }
        match (fixture.expected_admit, actual) {
            (true, true) => metrics.tp += 1,
            (false, true) => metrics.fp += 1,
            (false, false) => metrics.tn += 1,
            (true, false) => metrics.fn_ += 1,
        }
        if actual != fixture.expected_admit {
            metrics.failures.push(serde_json::json!({
                "fixture_id": fixture.id,
                "category": fixture.category,
                "expected": fixture.expected_admit,
                "actual": actual,
            }));
        }
        *metrics.reason_counts.entry(reason).or_default() += 1;
        if fixture.category == "duplicate" {
            metrics.duplicate_admitted = actual;
        }
        if fixture.category == "old-important" {
            metrics.old_important_retained = actual;
        }
        if fixture.category == "contradiction" {
            metrics.contradiction_correct = actual == fixture.expected_admit;
        }
    }
    metrics
}

fn report_value() -> serde_json::Value {
    let metrics = Policy::ALL.map(evaluate_policy);
    let combined = &metrics[4];
    let sensitive_rejected = FIXTURES
        .iter()
        .enumerate()
        .filter(|(_, fixture)| fixture.category == "sensitive")
        .all(|(index, fixture)| !Policy::NoveltyTimesUtility.evaluate(fixture, index).0);
    let chatter_rejected = FIXTURES
        .iter()
        .enumerate()
        .filter(|(_, fixture)| {
            fixture.category == "chatter" || fixture.category == "routine-length"
        })
        .all(|(index, fixture)| !Policy::NoveltyTimesUtility.evaluate(fixture, index).0);
    let go = combined.precision() > 0.80
        && combined.important_recall() >= 0.70
        && sensitive_rejected
        && chatter_rejected
        && !combined.duplicate_admitted
        && combined.old_important_retained
        && combined.contradiction_correct;

    serde_json::json!({
        "schema_version": 1,
        "fixtures_count": FIXTURES.len(),
        "categories": FIXTURES.iter().map(|fixture| fixture.category).collect::<HashSet<_>>().len(),
        "decision": if go { "Go" } else { "No-Go" },
        "decision_rule": "precision > 0.80; important_recall >= 0.70; zero sensitive/chatter/duplicate admissions; retain old-important; handle contradiction",
        "production_action": if go { "eligible_for_separate_implementation_review" } else { "retain_current_heuristic" },
        "sparse_reference_index": {
            "decision": "No-Go",
            "implemented": false,
            "reason": "no frozen exact-recall query corpus or material-benefit evidence; direct TLOG transcript remains canonical",
        },
        "policies": metrics.iter().map(|metric| serde_json::json!({
            "name": metric.policy,
            "precision": metric.precision(),
            "important_recall": metric.important_recall(),
            "tp": metric.tp,
            "fp": metric.fp,
            "tn": metric.tn,
            "fn": metric.fn_,
            "duplicate_admitted": metric.duplicate_admitted,
            "old_important_retained": metric.old_important_retained,
            "contradiction_correct": metric.contradiction_correct,
            "admitted_chars": metric.admitted_chars,
            "failures": metric.failures,
            "reason_counts": metric.reason_counts,
        })).collect::<Vec<_>>(),
    })
}

fn report_json() -> String {
    serde_json::to_string_pretty(&report_value()).expect("benchmark report serializes")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_corpus_covers_required_categories() {
        let categories = FIXTURES
            .iter()
            .map(|fixture| fixture.category)
            .collect::<HashSet<_>>();
        for required in [
            "correction",
            "preference",
            "routine-length",
            "duplicate",
            "validated",
            "recovery",
            "recency-conflict",
            "contradiction",
            "sensitive",
            "old-important",
        ] {
            assert!(categories.contains(required), "missing {required}");
        }
    }

    #[test]
    fn benchmark_compares_baseline_and_all_required_ablations_and_selects_no_go() {
        let report = report_value();
        assert_eq!(report["policies"].as_array().map(Vec::len), Some(5));
        assert_eq!(report["decision"], "No-Go");
        assert_eq!(report["production_action"], "retain_current_heuristic");
        assert_eq!(report["sparse_reference_index"]["decision"], "No-Go");
        let combined = &report["policies"][4];
        assert_eq!(combined["duplicate_admitted"], true);
    }

    #[test]
    fn benchmark_is_byte_stable_and_matches_checked_in_artifact() {
        let first = report_json();
        let second = report_json();
        assert_eq!(first, second);
        let artifact =
            include_str!("../../../docs/reference/MEM-009-BENCHMARK-RESULT-2026-07-17.json");
        assert_eq!(first.trim(), artifact.trim());
        serde_json::from_str::<serde_json::Value>(artifact).expect("artifact is valid JSON");
    }
}
