//! MEM-009 Memory Admission Benchmark (I137, revised v2)
//!
//! Compares four admission policies against a frozen fixture corpus.
//! Calls the production `evaluate_admission()` for the combined policy.
//! Emits machine-readable JSON results.
//!
use crate::evaluate_admission;
use std::collections::HashSet;

// ── Frozen fixture corpus ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct Fixture {
    content: &'static str,
    role: &'static str,
    expected_admit: bool,
    category: &'static str,
}

/// Fixtures are frozen BEFORE any policy runs. Categories cover all MEM-009
/// acceptance scenarios.
const FIXTURES: &[Fixture] = &[
    // Corrections — should be admitted
    Fixture {
        content: "Actually, the config file is at ~/.talos/config.toml not /etc/talos.conf",
        role: "user",
        expected_admit: true,
        category: "correction",
    },
    Fixture {
        content: "No, use cargo fmt not rustfmt directly",
        role: "user",
        expected_admit: true,
        category: "correction",
    },
    // Preferences — should be admitted
    Fixture {
        content: "I prefer Nord theme",
        role: "user",
        expected_admit: true,
        category: "preference",
    },
    Fixture {
        content: "Always run tests before committing",
        role: "user",
        expected_admit: true,
        category: "preference",
    },
    Fixture {
        content: "Never commit secrets to the repository",
        role: "user",
        expected_admit: true,
        category: "old-important",
    },
    // Contradictions — should be admitted
    Fixture {
        content: "Note: the previous statement about SQLite being required is wrong; we use bundled mode",
        role: "user",
        expected_admit: true,
        category: "contradiction",
    },
    // Validated results — should be admitted
    Fixture {
        content: "Important: the fix for issue #18 was caused by missing dispatch timeout",
        role: "assistant",
        expected_admit: true,
        category: "validated",
    },
    // Sensitive content — must NEVER be admitted
    Fixture {
        content: "api_key = sk-ant-1234567890abcdef",
        role: "user",
        expected_admit: false,
        category: "sensitive",
    },
    Fixture {
        content: "Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9",
        role: "user",
        expected_admit: false,
        category: "sensitive",
    },
    Fixture {
        content: "password = secret123",
        role: "user",
        expected_admit: false,
        category: "sensitive",
    },
    // Routine chatter — must be rejected
    Fixture {
        content: "Hello, how are you today?",
        role: "user",
        expected_admit: false,
        category: "chatter",
    },
    Fixture {
        content: "Thanks, that's helpful",
        role: "user",
        expected_admit: false,
        category: "chatter",
    },
    // Routine length — must be rejected (this was previously wrongly admitted)
    Fixture {
        content: "Can you help me understand how the session lifecycle works? I've been reading through the code.",
        role: "user",
        expected_admit: false,
        category: "routine-length",
    },
    // Duplicate — must be rejected
    Fixture {
        content: "Always run tests before committing",
        role: "user",
        expected_admit: false,
        category: "duplicate",
    },
];

// ── Decision rule (frozen before any results are read) ────────────────────
//
// Go conditions (ALL must hold):
// 1. combined precision > 0.80
// 2. combined recall >= 0.70
// 3. All sensitive items rejected
// 4. All routine-length/chatter items rejected

// ── Policy implementations ────────────────────────────────────────────────

/// Baseline: the OLD keyword/length heuristic (admits anything > 50 chars user msg
/// or with keyword markers, threshold > 0.5).
fn baseline_policy(content: &str, role: &str) -> bool {
    let lower = content.trim_start().to_lowercase();
    let markers = ["remember", "note", "important", "always", "never"];
    let score = if markers.iter().any(|m| lower.starts_with(m)) {
        0.8
    } else if role == "user" && content.len() > 50 {
        0.6
    } else {
        0.4
    };
    score > 0.5
}

/// Novelty-only: admit if content contains novelty markers (corrections, notes, directives).
fn novelty_only_policy(content: &str, _role: &str) -> bool {
    let lower = content.trim_start().to_lowercase();
    lower.starts_with("actually")
        || lower.starts_with("no,")
        || lower.starts_with("note")
        || lower.starts_with("important")
        || lower.starts_with("always")
        || lower.starts_with("never")
        || lower.starts_with("remember")
        || lower.starts_with("prefer")
        || lower.contains("fix for")
        || lower.contains("caused by")
}

/// Utility-only: admit if content contains utility markers (directives, corrections, fixes).
fn utility_only_policy(content: &str, _role: &str) -> bool {
    let lower = content.trim_start().to_lowercase();
    lower.starts_with("always")
        || lower.starts_with("never")
        || lower.starts_with("remember")
        || lower.starts_with("note")
        || lower.starts_with("important")
        || lower.starts_with("actually")
        || lower.starts_with("no,")
        || lower.contains("fix for")
        || lower.contains("caused by")
        || lower.contains("fixed it")
        || lower.contains("i prefer")
}

/// Combined: calls the PRODUCTION evaluate_admission() function.
fn combined_policy(content: &str, role: &str) -> bool {
    evaluate_admission(content, role).admit
}

// ── Metrics ───────────────────────────────────────────────────────────────

#[derive(Debug)]
struct Metrics {
    name: &'static str,
    tp: usize,
    fp: usize,
    tn: usize,
    fn_: usize,
}

impl Metrics {
    fn precision(&self) -> f64 {
        if self.tp + self.fp == 0 {
            0.0
        } else {
            self.tp as f64 / (self.tp + self.fp) as f64
        }
    }
    fn recall(&self) -> f64 {
        if self.tp + self.fn_ == 0 {
            0.0
        } else {
            self.tp as f64 / (self.tp + self.fn_) as f64
        }
    }
}

fn evaluate_policy(name: &'static str, policy: fn(&str, &str) -> bool) -> Metrics {
    let mut tp = 0;
    let mut fp = 0;
    let mut tn = 0;
    let mut fn_ = 0;
    for f in FIXTURES {
        let admitted = policy(f.content, f.role);
        if admitted == f.expected_admit {
            if admitted {
                tp += 1;
            } else {
                tn += 1;
            }
        } else if admitted {
            fp += 1;
        } else {
            fn_ += 1;
        }
    }
    Metrics {
        name,
        tp,
        fp,
        tn,
        fn_,
    }
}

fn decide(combined: &Metrics) -> bool {
    combined.precision() > 0.80
        && combined.recall() >= 0.70
        && FIXTURES
            .iter()
            .filter(|f| f.category == "sensitive")
            .all(|f| !combined_policy(f.content, f.role))
        && FIXTURES
            .iter()
            .filter(|f| f.category == "routine-length" || f.category == "chatter")
            .all(|f| !combined_policy(f.content, f.role))
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn benchmark_compares_all_four_policies() {
        let baseline = evaluate_policy("baseline", baseline_policy);
        let novelty = evaluate_policy("novelty_only", novelty_only_policy);
        let utility = evaluate_policy("utility_only", utility_only_policy);
        let combined = evaluate_policy("combined", combined_policy);
        let go = decide(&combined);

        // Machine-readable JSON output
        let json = serde_json::json!({
            "fixtures_count": FIXTURES.len(),
            "categories": FIXTURES.iter().map(|f| f.category).collect::<HashSet<_>>().len(),
            "policies": [
                { "name": baseline.name, "precision": baseline.precision(), "recall": baseline.recall(), "tp": baseline.tp, "fp": baseline.fp, "tn": baseline.tn, "fn": baseline.fn_ },
                { "name": novelty.name, "precision": novelty.precision(), "recall": novelty.recall(), "tp": novelty.tp, "fp": novelty.fp, "tn": novelty.tn, "fn": novelty.fn_ },
                { "name": utility.name, "precision": utility.precision(), "recall": utility.recall(), "tp": utility.tp, "fp": utility.fp, "tn": utility.tn, "fn": utility.fn_ },
                { "name": combined.name, "precision": combined.precision(), "recall": combined.recall(), "tp": combined.tp, "fp": combined.fp, "tn": combined.tn, "fn": combined.fn_ },
            ],
            "decision": if go { "Go" } else { "No-Go" },
            "decision_rule": "precision > 0.80 AND recall >= 0.70 AND all sensitive rejected AND all chatter rejected"
        });

        eprintln!("=== MEM-009 BENCHMARK (machine-readable JSON) ===");
        eprintln!("{}", serde_json::to_string_pretty(&json).unwrap());

        // All sensitive content must be rejected by combined
        for f in FIXTURES.iter().filter(|f| f.category == "sensitive") {
            assert!(
                !combined_policy(f.content, f.role),
                "sensitive leaked: {}",
                f.content
            );
        }

        // Decision must be deterministic
        assert_eq!(go, decide(&combined), "non-deterministic decision");
    }

    #[test]
    fn benchmark_is_deterministic() {
        let r1 = evaluate_policy("combined", combined_policy);
        let r2 = evaluate_policy("combined", combined_policy);
        assert_eq!(r1.tp, r2.tp);
        assert_eq!(r1.fp, r2.fp);
    }

    #[test]
    fn fixture_corpus_covers_all_categories() {
        let cats: HashSet<_> = FIXTURES.iter().map(|f| f.category).collect();
        for required in [
            "correction",
            "preference",
            "contradiction",
            "old-important",
            "validated",
            "sensitive",
            "chatter",
            "routine-length",
            "duplicate",
        ] {
            assert!(cats.contains(required), "missing: {required}");
        }
    }

    #[test]
    fn routine_length_fixture_is_rejected() {
        let f = FIXTURES
            .iter()
            .find(|f| f.category == "routine-length")
            .unwrap();
        let d = evaluate_admission(f.content, f.role);
        assert!(
            !d.admit,
            "routine-length must be rejected (score={})",
            d.score
        );
    }
}
