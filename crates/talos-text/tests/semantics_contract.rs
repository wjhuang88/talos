//! Headless consumers need no renderer or parser to use shared text semantics.
use talos_text::stream::{BlockDecision, FallbackReason, StreamBlockClassifier};
use talos_text::{HighlightResult, HighlightSpan, LanguageId};

fn source_lines(decisions: Vec<BlockDecision>) -> Vec<String> {
    decisions
        .into_iter()
        .flat_map(|decision| match decision {
            BlockDecision::ImmediateLine(line) => vec![line],
            BlockDecision::FinishHold { lines, .. }
            | BlockDecision::FallbackImmediate { lines, .. } => lines,
            _ => vec![],
        })
        .collect()
}

#[test]
fn all_block_kinds_have_bounded_first_line_fallback() {
    for prefix in ["```", "| a | b |", "- ", "> "] {
        let line = format!("{prefix}{}", "界".repeat(6000));
        let mut classifier = StreamBlockClassifier::default();
        let decisions = classifier.push_line(line.clone());
        assert!(matches!(
            decisions.as_slice(),
            [BlockDecision::FallbackImmediate {
                reason: FallbackReason::HeldBlockTooLarge,
                ..
            }]
        ));
        assert_eq!(source_lines(decisions), vec![line]);
        assert!(classifier.finish().is_empty());
    }
}

#[test]
fn list_and_quote_cannot_hold_unbounded_lines() {
    for prefix in ["- item", "> quote"] {
        let mut classifier = StreamBlockClassifier::default();
        for _ in 0..200 {
            assert!(source_lines(classifier.push_line(prefix.into())).is_empty());
        }
        let decisions = classifier.push_line(prefix.into());
        assert!(matches!(
            decisions.as_slice(),
            [BlockDecision::FallbackImmediate {
                reason: FallbackReason::HeldBlockTooLarge,
                ..
            }]
        ));
        assert_eq!(source_lines(decisions), vec![prefix.to_string(); 201]);
        assert!(classifier.finish().is_empty());
    }
}

#[test]
fn emitted_source_is_lossless_across_block_boundaries_and_flush() {
    for input in [
        "hello\n```rust\nlet 界 = 1;\n```\ntail",
        "| a | b |\n| --- | --- |\n| 一 | 二 |\nend",
        "- one\n- two\n> quote\n> more\nend",
        "| maybe | table |\nnot a separator\n~~~rust\nunclosed",
    ] {
        let mut classifier = StreamBlockClassifier::default();
        let mut emitted = vec![];
        for line in input.split('\n') {
            emitted.extend(source_lines(classifier.push_line(line.into())));
        }
        emitted.extend(source_lines(classifier.finish()));
        assert_eq!(emitted.join("\n"), input);
        assert!(classifier.finish().is_empty());
    }
}

#[test]
fn invalid_span_anywhere_rejects_the_whole_result() {
    let source = "a界z";
    for (start, end) in [(2, 4), (1, 3), (0, 99), (4, 1), (6, 6)] {
        let result = HighlightResult::Spans(vec![HighlightSpan {
            start,
            end,
            capture: "keyword".into(),
        }]);
        assert!(result.validated_spans(source).is_none(), "{start}..{end}");
    }
    for spans in [vec![(1, 4), (0, 1)], vec![(0, 4), (1, 4)]] {
        let result = HighlightResult::Spans(
            spans
                .into_iter()
                .map(|(start, end)| HighlightSpan {
                    start,
                    end,
                    capture: "keyword".into(),
                })
                .collect(),
        );
        assert!(result.validated_spans(source).is_none());
    }
    assert!(HighlightResult::PlainText.validated_spans(source).is_none());
    let valid = HighlightResult::Spans(vec![HighlightSpan {
        start: 1,
        end: 4,
        capture: "string".into(),
    }]);
    assert_eq!(valid.validated_spans(source).expect("valid").len(), 1);
}

#[test]
fn normalization_is_idempotent_without_requiring_a_provider() {
    for alias in [".RS", " TSX ", "PY", "c++", "yml", "unknown-language", "界"] {
        let id = LanguageId::parse(alias).expect("nonempty");
        assert_eq!(LanguageId::parse(id.as_str()), Some(id));
    }
}
