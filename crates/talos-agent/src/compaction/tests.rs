use super::constants::{
    CIRCUIT_BREAKER_THRESHOLD, COLLAPSE_TURN_THRESHOLD, MAX_TOOL_RESULT_CHARS, PRESERVED_TURNS,
    TRIM_TURN_THRESHOLD, TRUNCATION_SUFFIX,
};
use super::*;
use crate::token::TokenEstimator;
use async_trait::async_trait;
use talos_core::message::{AgentEvent, Message, MessageToolResult, ToolCall, Usage};
use talos_core::provider::{LanguageModel, ProviderError, ProviderResult};
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

// --- Mock provider that returns a fixed summary ---

struct SummaryMockProvider {
    summary: String,
}

impl SummaryMockProvider {
    fn new(summary: &str) -> Self {
        Self {
            summary: summary.into(),
        }
    }
}

#[async_trait]
impl LanguageModel for SummaryMockProvider {
    async fn stream(&self, _messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
        let (tx, rx) = mpsc::channel(32);
        let summary = self.summary.clone();
        tokio::spawn(async move {
            let _ = tx.send(AgentEvent::TurnStart).await;
            let _ = tx.send(AgentEvent::TextDelta { delta: summary }).await;
            let _ = tx
                .send(AgentEvent::TurnEnd {
                    stop_reason: talos_core::message::StopReason::EndTurn,
                    usage: Usage::default(),
                })
                .await;
        });
        Ok(rx)
    }
}

// --- Mock provider that always fails ---

struct FailingProvider;

#[async_trait]
impl LanguageModel for FailingProvider {
    async fn stream(&self, _messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
        Err(ProviderError::ServerError("mock failure".into()))
    }
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

// ========== Layer 4: Collapse ==========

#[tokio::test]
async fn test_layer4_collapse_summarizes_old_turns() {
    let compactor = Compactor::new(TokenEstimator::new(), 100_000);
    let mut messages = Vec::new();

    // Create 15 turns
    for i in 0..15 {
        messages.extend(make_turn(
            &format!("query {i}"),
            &format!("response {i}"),
            vec![(&format!("call_{i}"), &format!("result {i}"))],
        ));
    }

    let provider = SummaryMockProvider::new("Summary of old turns");
    let result = compactor.apply_collapse(messages, &provider).await.unwrap();

    // First message should be the summary
    assert!(
        matches!(&result[0], Message::User { content } if content.contains("Conversation summary"))
    );
    assert!(
        matches!(&result[0], Message::User { content } if content.contains("Summary of old turns"))
    );

    // Remaining messages should be the last 10 turns (30 messages: 10 * 3)
    let recent_count = result.len() - 1;
    assert_eq!(recent_count, 30); // 10 turns * 3 messages each
}

#[tokio::test]
async fn test_layer4_collapse_no_op_when_under_threshold() {
    let compactor = Compactor::new(TokenEstimator::new(), 100_000);
    let messages = make_turn("query", "response", vec![("call_1", "result")]);

    let provider = SummaryMockProvider::new("summary");
    let result = compactor
        .apply_collapse(messages.clone(), &provider)
        .await
        .unwrap();

    assert_eq!(result, messages);
}

// ========== Layer 5: Autocompact ==========

#[tokio::test]
async fn test_layer5_autocompact_summarizes_all_old_turns() {
    let compactor = Compactor::new(TokenEstimator::new(), 100_000);
    let mut messages = Vec::new();

    // Create 15 turns
    for i in 0..15 {
        messages.extend(make_turn(
            &format!("query {i}"),
            &format!("response {i}"),
            vec![(&format!("call_{i}"), &format!("result {i}"))],
        ));
    }

    let provider = SummaryMockProvider::new("Full conversation summary");
    let result = compactor
        .apply_autocompact(messages, &provider)
        .await
        .unwrap();

    // First message should be the summary
    assert!(
        matches!(&result[0], Message::User { content } if content.contains("Full conversation summary"))
    );

    // Remaining messages should be the last 10 turns
    let recent_count = result.len() - 1;
    assert_eq!(recent_count, 30); // 10 turns * 3 messages each
}

#[tokio::test]
async fn test_layer5_autocompact_no_op_when_under_threshold() {
    let compactor = Compactor::new(TokenEstimator::new(), 100_000);
    let messages = make_turn("query", "response", vec![("call_1", "result")]);

    let provider = SummaryMockProvider::new("summary");
    let result = compactor
        .apply_autocompact(messages.clone(), &provider)
        .await
        .unwrap();

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

// ========== Circuit Breaker ==========

#[tokio::test]
async fn test_circuit_breaker_trips_after_3_failures() {
    let mut compactor = Compactor::new(TokenEstimator::new(), 1);
    let mut messages = Vec::new();
    for i in 0..15 {
        messages.extend(make_turn(
            &format!("query {i}"),
            &format!("response {i}"),
            vec![(&format!("call_{i}"), &format!("result {i}"))],
        ));
    }

    let provider = FailingProvider;

    for _ in 0..3 {
        let result = compactor.compact(messages.clone(), &provider).await;
        assert!(result.is_err(), "Expected failure");
    }

    assert_eq!(compactor.failure_count(), 3);

    let result = compactor.compact(messages.clone(), &provider).await;
    assert!(matches!(
        result.unwrap_err(),
        CompactionError::CircuitBreakerTripped
    ));
}

// ========== Recent turns preservation ==========

#[tokio::test]
async fn test_recent_turns_preserved_verbatim() {
    let mut compactor = Compactor::new(TokenEstimator::new(), 100_000);
    let mut messages = Vec::new();

    // Create 15 turns with unique content
    for i in 0..15 {
        messages.extend(make_turn(
            &format!("unique_query_{i}"),
            &format!("unique_response_{i}"),
            vec![(&format!("call_{i}"), &format!("unique_result_{i}"))],
        ));
    }

    let provider = SummaryMockProvider::new("summary");
    let result = compactor
        .compact(messages.clone(), &provider)
        .await
        .unwrap();

    // The last 30 messages (10 turns * 3) should match the original last 30
    let original_recent = &messages[messages.len() - 30..];
    let result_recent = &result[result.len() - 30..];

    assert_eq!(original_recent, result_recent);
}

// ========== Seamless continuation ==========

#[tokio::test]
async fn test_compaction_continues_conversation_seamlessly() {
    let mut compactor = Compactor::new(TokenEstimator::new(), 160);
    let mut messages = Vec::new();

    for i in 0..15 {
        messages.extend(make_turn(
            &format!("query {i}"),
            &format!("response {i}"),
            vec![(&format!("call_{i}"), &format!("result {i}"))],
        ));
    }

    let provider = SummaryMockProvider::new("Summary: user asked about files, assistant read them");
    let compacted = compactor.compact(messages, &provider).await.unwrap();

    assert!(compacted.len() > 1);
    if let Message::User { content } = &compacted[0] {
        assert!(
            content.contains("Summary"),
            "first message content: {content}"
        );
    } else {
        panic!("first message is not User: {:?}", compacted[0]);
    }
    assert!(matches!(compacted.last(), Some(Message::Tool { .. })));

    let mut continued = compacted;
    continued.push(user_msg("Follow-up question"));
    assert!(matches!(continued.last(), Some(Message::User { .. })));
}

// ========== Full compact flow ==========

#[tokio::test]
async fn test_compact_stops_early_when_budget_layer_suffices() {
    let mut compactor = Compactor::new(TokenEstimator::new(), 100_000);
    let long_content = "x".repeat(5000);
    let messages = vec![tool_msg("call_1", &long_content)];

    // Budget layer should truncate and make it fit
    let result = compactor.compact(messages, &FailingProvider).await.unwrap();

    assert_eq!(result.len(), 1);
    if let Message::Tool { result: tr } = &result[0] {
        assert!(tr.content.len() < 5000);
        assert!(tr.content.ends_with(TRUNCATION_SUFFIX));
    }
}

#[tokio::test]
async fn test_compact_provider_error_propagates() {
    let mut compactor = Compactor::new(TokenEstimator::new(), 1);
    let mut messages = Vec::new();

    // Create enough turns to trigger collapse layer
    for i in 0..15 {
        messages.extend(make_turn(
            &format!("query {i}"),
            &format!("response {i}"),
            vec![(&format!("call_{i}"), &format!("result {i}"))],
        ));
    }

    let provider = FailingProvider;
    let result = compactor.compact(messages, &provider).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CompactionError::ProviderError(_)
    ));
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

#[tokio::test]
async fn test_compact_resets_failure_count_on_success() {
    let mut compactor = Compactor::new(TokenEstimator::new(), 100_000);
    let long_content = "x".repeat(5000);
    let messages = vec![tool_msg("call_1", &long_content)];

    // This should succeed (budget layer is enough)
    let result = compactor.compact(messages, &FailingProvider).await;
    assert!(result.is_ok());

    // Failure count should be reset to 0
    assert_eq!(compactor.failure_count(), 0);
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
            reason: "deterministic layers insufficient; LLM layers required",
            ..
        }
    ));
}

// ========== MEM-005-A: manual_compact ==========

#[tokio::test]
async fn test_manual_compact_skipped_below_threshold() {
    let mut compactor = Compactor::new(TokenEstimator::new(), 100_000);
    let messages = vec![user_msg("Hello")];

    let (_result, status) = compactor.manual_compact(messages, &FailingProvider).await;

    assert!(matches!(
        status,
        CompactionStatus::Skipped {
            reason: "below trigger threshold",
            ..
        }
    ));
}

#[tokio::test]
async fn test_manual_compact_applied_when_over_threshold() {
    let mut compactor = Compactor::new(TokenEstimator::new(), 1100);
    let long_content = "x".repeat(5000);
    let messages = vec![tool_msg("call_1", &long_content)];

    let (_result, status) = compactor.manual_compact(messages, &FailingProvider).await;

    assert!(matches!(status, CompactionStatus::Applied { .. }));
}

#[tokio::test]
async fn test_manual_compact_failed_on_provider_error() {
    let mut compactor = Compactor::new(TokenEstimator::new(), 1);
    let mut messages = Vec::new();
    for i in 0..15 {
        messages.extend(make_turn(
            &format!("q{i}"),
            &format!("r{i}"),
            vec![(&format!("c{i}"), &format!("d{i}"))],
        ));
    }

    let original = messages.clone();
    let (result, status) = compactor.manual_compact(messages, &FailingProvider).await;

    assert!(matches!(status, CompactionStatus::Failed { .. }));
    assert_eq!(
        result, original,
        "manual compaction failure must preserve original context"
    );
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
