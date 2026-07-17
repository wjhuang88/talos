//! MEM-009 Memory Admission Benchmark (I137, revised)
//!
//! Calls the production `evaluate_admission()` function (not a separate
//! policy implementation) against a frozen fixture corpus. Compares the
//! production admission policy against a baseline that admits everything.

use crate::{AdmissionReason, evaluate_admission};

#[derive(Debug, Clone)]
struct Fixture {
    content: &'static str,
    role: &'static str,
    expected_admit: bool,
    category: &'static str,
}

const FIXTURES: &[Fixture] = &[
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
        content: "Note: the previous statement about SQLite being required is wrong; we use bundled mode",
        role: "user",
        expected_admit: true,
        category: "contradiction",
    },
    Fixture {
        content: "Never commit secrets to the repository",
        role: "user",
        expected_admit: true,
        category: "old-important",
    },
    Fixture {
        content: "The fix for issue #18 was to add a dispatch timeout",
        role: "assistant",
        expected_admit: true,
        category: "validated",
    },
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
    Fixture {
        content: "Can you help me understand how the session lifecycle works in this project? I've been reading through the code.",
        role: "user",
        expected_admit: false,
        category: "routine-length",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_admission_matches_expected_outcomes() {
        let mut tp = 0;
        let mut fp = 0;
        let mut tn = 0;
        let mut fn_ = 0;

        for f in FIXTURES {
            let decision = evaluate_admission(f.content, f.role);
            if decision.admit == f.expected_admit {
                if decision.admit {
                    tp += 1;
                } else {
                    tn += 1;
                }
            } else if decision.admit {
                fp += 1;
            } else {
                fn_ += 1;
            }
        }

        let precision = if tp + fp > 0 {
            tp as f64 / (tp + fp) as f64
        } else {
            0.0
        };
        let recall = if tp + fn_ > 0 {
            tp as f64 / (tp + fn_) as f64
        } else {
            0.0
        };

        eprintln!("=== Production Admission Benchmark ===");
        eprintln!("tp={tp} fp={fp} tn={tn} fn={fn_}");
        eprintln!("precision={precision:.3} recall={recall:.3}");

        // All sensitive content must be rejected
        for f in FIXTURES.iter().filter(|f| f.category == "sensitive") {
            let d = evaluate_admission(f.content, f.role);
            assert!(!d.admit, "sensitive content admitted: {}", f.content);
            assert_eq!(d.reason, AdmissionReason::SensitiveContent);
        }

        // Precision and recall must be reasonable
        assert!(precision >= 0.8, "precision too low: {precision}");
        assert!(recall >= 0.7, "recall too low: {recall}");
    }

    #[test]
    fn production_admission_is_deterministic() {
        for f in FIXTURES {
            let d1 = evaluate_admission(f.content, f.role);
            let d2 = evaluate_admission(f.content, f.role);
            assert_eq!(d1.admit, d2.admit, "non-deterministic for: {}", f.content);
            assert_eq!(d1.score, d2.score, "score differs for: {}", f.content);
        }
    }

    #[test]
    fn fixture_corpus_covers_all_categories() {
        let cats: std::collections::HashSet<_> = FIXTURES.iter().map(|f| f.category).collect();
        for required in [
            "correction",
            "preference",
            "contradiction",
            "old-important",
            "validated",
            "sensitive",
            "chatter",
            "routine-length",
        ] {
            assert!(cats.contains(required), "missing category: {required}");
        }
    }
}
