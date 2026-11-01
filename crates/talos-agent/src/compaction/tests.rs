use super::constants::{
    CIRCUIT_BREAKER_THRESHOLD, COLLAPSE_TURN_THRESHOLD, MAX_TOOL_RESULT_CHARS, PRESERVED_TURNS,
    TRIM_TURN_THRESHOLD, TRUNCATION_SUFFIX,
};
use super::*;
use crate::token::TokenEstimator;
use async_trait::async_trait;
use talos_core::message::{AgentEvent, Message, MessageToolResult, ToolCall};
use talos_core::provider::{LanguageModel, ProviderResult};
use tokio::sync::mpsc;

type Receiver<T> = mpsc::Receiver<T>;

/// Helper: create a tool result message.
fn tool_msg(id: &str, content: &str) -> Message {
    Message::Tool {
        result: MessageToolResult {
            tool_use_id: id.into(),
            content: content.into(),
            is_error: false,
        },
    }
}

/// Helper: create a user message.
fn user_msg(content: &str) -> Message {
    Message::User {
        content: content.into(),
    }
}

/// Helper: create an assistant message.
fn assistant_msg(content: &str) -> Message {
    Message::Assistant {
        content: content.into(),
        tool_calls: vec![],
        reasoning: None,
    }
}

/// Helper: create an assistant message with tool calls.
fn assistant_with_tools(content: &str, tools: Vec<ToolCall>) -> Message {
    Message::Assistant {
        content: content.into(),
        tool_calls: tools,
        reasoning: None,
    }
}

/// Helper: create a full turn (user + assistant + tool results).
fn make_turn(
    user_content: &str,
    assistant_content: &str,
    tool_results: Vec<(&str, &str)>,
) -> Vec<Message> {
    let mut msgs = vec![user_msg(user_content)];
    let tool_calls: Vec<ToolCall> = tool_results
        .iter()
        .map(|(id, _)| ToolCall {
            id: id.to_string(),
            name: "test_tool".into(),
            input: serde_json::json!({}),
        })
        .collect();
    msgs.push(assistant_with_tools(assistant_content, tool_calls));
    for (id, content) in tool_results {
        msgs.push(tool_msg(id, content));
    }
    msgs
}

// ========== Layer 1: Budget ==========

#[test]
fn test_layer1_budget_truncates_long_tool_result() {
    let compactor = Compactor::new(TokenEstimator::new(), 100_000);
    let long_content = "x".repeat(5000);
    let messages = vec![tool_msg("call_1", &long_content)];

    let result = compactor.apply_budget(messages);

    assert_eq!(result.len(), 1);
    if let Message::Tool { result: tr } = &result[0] {
        assert!(tr.content.len() <= MAX_TOOL_RESULT_CHARS + TRUNCATION_SUFFIX.len());
        assert!(tr.content.ends_with(TRUNCATION_SUFFIX));
    } else {
        panic!("expected Tool message");
    }
}

#[test]
fn test_layer1_budget_preserves_short_tool_result() {
    let compactor = Compactor::new(TokenEstimator::new(), 100_000);
    let short_content = "short result";
    let messages = vec![tool_msg("call_1", short_content)];

    let result = compactor.apply_budget(messages);

    assert_eq!(result.len(), 1);
    if let Message::Tool { result: tr } = &result[0] {
        assert_eq!(tr.content, short_content);
    } else {
        panic!("expected Tool message");
    }
}

#[test]
fn test_layer1_budget_truncates_multibyte_content_on_char_boundary() {
    // Regression: byte-slicing at index 4000 panicked when the boundary
    // fell inside a 3-byte char like '责'. Truncation must count chars.
    let compactor = Compactor::new(TokenEstimator::new(), 100_000);
    let long_content = "责".repeat(5000);
    let messages = vec![tool_msg("call_1", &long_content)];

    let result = compactor.apply_budget(messages);

    assert_eq!(result.len(), 1);
    if let Message::Tool { result: tr } = &result[0] {
        let body = tr
            .content
            .strip_suffix(TRUNCATION_SUFFIX)
            .expect("operation should succeed");
        assert_eq!(body.chars().count(), MAX_TOOL_RESULT_CHARS);
        assert!(tr.content.ends_with(TRUNCATION_SUFFIX));
    } else {
        panic!("expected Tool message");
    }
}

#[test]
fn test_layer1_budget_preserves_short_multibyte_result() {
    // 3000 CJK chars = 9000 bytes: over the byte count but under the char
    // threshold, so it must survive untouched (threshold is char-based).
    let compactor = Compactor::new(TokenEstimator::new(), 100_000);
    let short_content = "责".repeat(3000);
    let messages = vec![tool_msg("call_1", &short_content)];

    let result = compactor.apply_budget(messages);

    assert_eq!(result.len(), 1);
    if let Message::Tool { result: tr } = &result[0] {
        assert_eq!(tr.content, short_content);
    } else {
        panic!("expected Tool message");
    }
}

#[test]
fn test_layer1_budget_does_not_affect_other_messages() {
    let compactor = Compactor::new(TokenEstimator::new(), 100_000);
    let messages = vec![
        user_msg("Hello"),
        assistant_msg("Hi there"),
        tool_msg("call_1", "short"),
    ];

    let result = compactor.apply_budget(messages);

    assert_eq!(result.len(), 3);
    assert!(matches!(&result[0], Message::User { .. }));
    assert!(matches!(&result[1], Message::Assistant { .. }));
    assert!(matches!(&result[2], Message::Tool { .. }));
}

// ========== Layer 2: Trim ==========

#[test]
fn test_layer2_trim_removes_old_tool_results() {
    let compactor = Compactor::new(TokenEstimator::new(), 100_000);
    let mut messages = Vec::new();

    // Create 25 turns
    for i in 0..25 {
        messages.extend(make_turn(
            &format!("query {i}"),
            &format!("response {i}"),
            vec![(&format!("call_{i}"), &format!("result {i}"))],
        ));
    }

    let result = compactor.apply_trim(messages);

    // Tool results from turns 1-5 should be empty (trimmed)
    // Turns 6-25 should be preserved
    let mut trimmed_count = 0;
    let mut preserved_count = 0;
    for msg in &result {
        if let Message::Tool { result: tr } = msg {
            if tr.content.is_empty() {
                trimmed_count += 1;
            } else {
                preserved_count += 1;
            }
        }
    }

    assert_eq!(trimmed_count, 5); // First 5 turns trimmed
    assert_eq!(preserved_count, 20); // Last 20 turns preserved
}

#[test]
fn test_layer2_trim_no_op_when_under_threshold() {
    let compactor = Compactor::new(TokenEstimator::new(), 100_000);
    let messages = make_turn("query", "response", vec![("call_1", "result")]);

    let result = compactor.apply_trim(messages.clone());

    assert_eq!(result, messages);
}

// ========== Layer 3: Microcompact ==========

#[test]
fn test_layer3_microcompact_keeps_last_result_per_id() {
    let compactor = Compactor::new(TokenEstimator::new(), 100_000);
    let messages = vec![
        tool_msg("call_1", "first result"),
        tool_msg("call_2", "other result"),
        tool_msg("call_1", "second result"),
        tool_msg("call_2", "final result"),
    ];

    let result = compactor.apply_microcompact(messages);

    let contents: Vec<_> = result
        .iter()
        .filter_map(|m| {
            if let Message::Tool { result: tr } = m {
                Some(tr.content.clone())
            } else {
                None
            }
        })
        .collect();

    // call_1: first should be empty, second preserved
    // call_2: first should be empty, final preserved
    assert_eq!(contents[0], ""); // call_1 first → emptied
    assert_eq!(contents[1], ""); // call_2 first → emptied
    assert_eq!(contents[2], "second result"); // call_1 last → kept
    assert_eq!(contents[3], "final result"); // call_2 last → kept
}

#[test]
fn test_layer3_microcompact_no_duplicates() {
    let compactor = Compactor::new(TokenEstimator::new(), 100_000);
    let messages = vec![
        tool_msg("call_1", "unique result"),
        tool_msg("call_2", "another result"),
    ];

    let result = compactor.apply_microcompact(messages.clone());

    assert_eq!(result, messages);
}

// ========== should_compact ==========

#[test]
fn test_should_compact_returns_true_at_80_percent() {
    // model_limit = 1000, threshold = 0.8 → trigger at > 800 tokens
    let compactor = Compactor::new(TokenEstimator::new(), 1000);

    // Create messages that estimate to ~801 tokens
    // ASCII: 4 chars per token → need ~3204 chars
    let long_content = "x".repeat(3204);
    let messages = vec![user_msg(&long_content)];

    assert!(compactor.should_compact(&messages));
}

#[test]
fn test_should_compact_returns_false_below_threshold() {
    let compactor = Compactor::new(TokenEstimator::new(), 1000);

    // Small message: well under 800 tokens
    let messages = vec![user_msg("Hello, world!")];

    assert!(!compactor.should_compact(&messages));
}

#[test]
fn test_should_compact_custom_threshold() {
    let compactor = Compactor::new(TokenEstimator::new(), 1000).with_threshold(0.5);

    // 500 token threshold → need > 500 tokens → ~2001 chars
    let content = "x".repeat(2001);
    let messages = vec![user_msg(&content)];

    assert!(compactor.should_compact(&messages));
}

// ========== Edge cases ==========

#[test]
fn test_empty_messages() {
    let compactor = Compactor::new(TokenEstimator::new(), 1000);
    let messages: Vec<Message> = vec![];

    assert!(!compactor.should_compact(&messages));
}

#[test]
fn test_apply_budget_exact_boundary() {
    let compactor = Compactor::new(TokenEstimator::new(), 100_000);
    let exact_content = "x".repeat(MAX_TOOL_RESULT_CHARS);
    let messages = vec![tool_msg("call_1", &exact_content)];

    let result = compactor.apply_budget(messages);

    // Exactly at boundary should NOT be truncated
    if let Message::Tool { result: tr } = &result[0] {
        assert_eq!(tr.content.len(), MAX_TOOL_RESULT_CHARS);
        assert!(!tr.content.ends_with(TRUNCATION_SUFFIX));
    }
}

#[test]
fn test_apply_budget_one_over_boundary() {
    let compactor = Compactor::new(TokenEstimator::new(), 100_000);
    let over_content = "x".repeat(MAX_TOOL_RESULT_CHARS + 1);
    let messages = vec![tool_msg("call_1", &over_content)];

    let result = compactor.apply_budget(messages);

    if let Message::Tool { result: tr } = &result[0] {
        assert!(tr.content.ends_with(TRUNCATION_SUFFIX));
    }
}

// ========== MEM-005-A: CompactionPolicy ==========

#[test]
fn test_policy_defaults_match_constants() {
    let policy = CompactionPolicy::default();
    assert_eq!(policy.trigger_threshold, 0.8);
    assert_eq!(policy.max_tool_result_chars, MAX_TOOL_RESULT_CHARS);
    assert_eq!(policy.preserved_turns, PRESERVED_TURNS);
    assert_eq!(policy.trim_turn_threshold, TRIM_TURN_THRESHOLD);
    assert_eq!(policy.collapse_turn_threshold, COLLAPSE_TURN_THRESHOLD);
    assert_eq!(policy.circuit_breaker_threshold, CIRCUIT_BREAKER_THRESHOLD);
    assert_eq!(policy.output_reserve, 0);
}

#[test]
fn test_policy_trigger_tokens_calculation() {
    let policy = CompactionPolicy::default();
    assert_eq!(policy.trigger_tokens(128_000), 102_400);
    assert_eq!(policy.trigger_tokens(100_000), 80_000);

    let policy_with_reserve = CompactionPolicy {
        output_reserve: 4096,
        ..Default::default()
    };
    assert_eq!(policy_with_reserve.trigger_tokens(100_000), 80_000 - 4096);
}

#[test]
fn test_policy_trigger_tokens_saturates_on_small_limit() {
    let policy = CompactionPolicy {
        output_reserve: 100_000,
        ..Default::default()
    };
    assert_eq!(policy.trigger_tokens(1000), 0);
}

// ========== MEM-005-A: compact_deterministic ==========

#[test]
fn test_compact_deterministic_applies_budget_when_sufficient() {
    let compactor = Compactor::new(TokenEstimator::new(), 100_000);
    let long_content = "x".repeat(5000);
    let messages = vec![tool_msg("call_1", &long_content)];

    let (result, status) = compactor.compact_deterministic(messages);

    assert!(matches!(status, CompactionStatus::Applied { .. }));
    if let CompactionStatus::Applied { layers_applied, .. } = &status {
        assert_eq!(layers_applied, &vec!["budget"]);
    }
    assert_eq!(result.len(), 1);
}

#[test]
fn test_compact_deterministic_returns_skipped_when_insufficient() {
    let compactor = Compactor::new(TokenEstimator::new(), 1);
    let mut messages = Vec::new();
    for i in 0..5 {
        messages.extend(make_turn(
            &format!("q{i}"),
            &format!("r{i}"),
            vec![(&format!("c{i}"), &format!("d{i}"))],
        ));
    }

    let (_result, status) = compactor.compact_deterministic(messages);

    assert!(matches!(
        status,
        CompactionStatus::Skipped {
            reason: "deterministic layers insufficient; use typed session compaction",
            ..
        }
    ));
}

// ========== MEM-005-A: hidden-output guard ==========

#[test]
fn test_compaction_status_never_contains_tool_content() {
    let compactor = Compactor::new(TokenEstimator::new(), 100_000);
    let messages = vec![tool_msg("call_1", "SECRET_TOOL_OUTPUT_12345")];

    let (_result, status) = compactor.compact_deterministic(messages);

    let status_str = format!("{status:?}");
    assert!(
        !status_str.contains("SECRET_TOOL_OUTPUT_12345"),
        "CompactionStatus must never expose tool result content"
    );
}

// ── TOOL-021 error propagation fixtures: compaction preserves is_error ──

fn error_tool_msg(id: &str, content: &str) -> Message {
    Message::Tool {
        result: MessageToolResult {
            tool_use_id: id.into(),
            content: content.into(),
            is_error: true,
        },
    }
}

#[test]
fn fixture_budget_truncation_preserves_is_error() {
    // F6: budget truncation preserves is_error flag even when content is truncated
    let large_content = "x".repeat(MAX_TOOL_RESULT_CHARS + 1000);
    let messages = vec![error_tool_msg("call_1", &large_content)];
    let compactor = Compactor::new(TokenEstimator::new(), 100_000);
    let result = compactor.apply_budget(messages);

    assert_eq!(result.len(), 1);
    if let Message::Tool { result } = &result[0] {
        assert!(result.is_error, "is_error must survive budget truncation");
        assert!(
            result.content.ends_with(TRUNCATION_SUFFIX),
            "content must be truncated with suffix"
        );
        assert!(
            result.content.len() <= MAX_TOOL_RESULT_CHARS + TRUNCATION_SUFFIX.len(),
            "truncated content must be within budget"
        );
    } else {
        panic!("expected Tool message");
    }
}

#[test]
fn fixture_trim_preserves_is_error() {
    // F7: trim compaction empties content but preserves is_error
    let mut messages = Vec::new();
    for turn in 0..(TRIM_TURN_THRESHOLD + 5) {
        messages.push(Message::User {
            content: format!("turn {turn}"),
        });
        messages.push(Message::Assistant {
            content: format!("reply {turn}"),
            tool_calls: vec![],
            reasoning: None,
        });
        messages.push(error_tool_msg(&format!("call_{turn}"), "error happened"));
    }
    let compactor = Compactor::new(TokenEstimator::new(), 100_000);
    let result = compactor.apply_trim(messages);

    // First few turns should have emptied tool content but preserved is_error
    let first_tool = result
        .iter()
        .find_map(|m| {
            if let Message::Tool { result } = m {
                Some(result)
            } else {
                None
            }
        })
        .expect("at least one tool message");
    assert!(
        first_tool.is_error,
        "is_error must survive trim even when content is emptied"
    );
    assert!(
        first_tool.content.is_empty(),
        "trimmed old tool content must be empty"
    );
}

#[test]
fn fixture_microcompact_preserves_is_error() {
    // F8: microcompact deduplicates by tool_use_id, preserves is_error on kept entry
    let messages = vec![
        error_tool_msg("dup_1", "first error"),
        error_tool_msg("dup_1", "second error"),
    ];
    let compactor = Compactor::new(TokenEstimator::new(), 100_000);
    let result = compactor.apply_microcompact(messages);

    assert_eq!(result.len(), 2);
    // First (older) should have content emptied
    if let Message::Tool { result } = &result[0] {
        assert!(result.is_error, "is_error preserved on deduped entry");
        assert!(result.content.is_empty(), "older duplicate content emptied");
    } else {
        panic!("expected Tool message");
    }
    // Second (newer) should retain content
    if let Message::Tool { result } = &result[1] {
        assert!(result.is_error, "is_error preserved on kept entry");
        assert_eq!(result.content, "second error");
    } else {
        panic!("expected Tool message");
    }
}

struct ForbiddenLegacyProvider;

#[async_trait]
impl LanguageModel for ForbiddenLegacyProvider {
    async fn stream(&self, _: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
        panic!("legacy compaction must never disclose history to a provider");
    }
}

#[tokio::test]
#[allow(deprecated)]
async fn legacy_model_entry_points_refuse_without_disclosure_or_partial_transformation() {
    let provider = ForbiddenLegacyProvider;
    // Empty and fitting input must not disguise an unsupported API as success.
    for turns in [0, 1, 50] {
        let mut raw = Vec::new();
        for turn in 0..turns {
            raw.extend(make_turn(
                &format!("original request {turn}"),
                "original response",
                vec![("private_tool", &"private-output".repeat(1_000))],
            ));
        }
        let original = serde_json::to_value(&raw).expect("raw snapshot");
        for limit in [1, u32::MAX] {
            let mut legacy = Compactor::new(TokenEstimator::new(), limit);
            for result in [
                legacy.compact(raw.clone(), &provider).await,
                legacy.apply_collapse(raw.clone(), &provider).await,
                legacy.apply_autocompact(raw.clone(), &provider).await,
            ] {
                let error = result.expect_err("bare history cannot grant request authority");
                assert!(matches!(error, CompactionError::CompactionFailed(_)));
                assert!(error.to_string().contains("RuntimeHandle::compact"));
                assert!(!error.to_string().contains("private-output"));
            }
            let (returned, status) = legacy.manual_compact(raw.clone(), &provider).await;
            assert_eq!(
                serde_json::to_value(returned).expect("returned snapshot"),
                original
            );
            assert!(matches!(status, CompactionStatus::Failed { .. }));
            assert!(!format!("{status:?}").contains("private-output"));
            assert_eq!(
                serde_json::to_value(&raw).expect("input snapshot"),
                original
            );
        }
    }
}
