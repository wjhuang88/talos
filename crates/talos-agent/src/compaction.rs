//! 5-layer context compaction for agent sessions.
//!
//! When the conversation context approaches the model's token limit, this module
//! applies progressive compaction layers to reduce context size while preserving
//! recent conversation fidelity.
//!
//! # Compaction Layers
//!
//! Layers are applied in order, stopping as soon as the context fits:
//!
//! 1. **Budget** — Cap individual tool results to 4000 characters, truncating
//!    with `"... [truncated]"`.
//! 2. **Trim** — Remove tool results from turns older than 20.
//! 3. **Microcompact** — For each tool call ID, keep only the last result.
//! 4. **Collapse** — Summarize turns older than 10 into a single summary message
//!    using the LLM.
//! 5. **Autocompact** — Use the LLM to summarize the entire conversation history.
//!
//! # Preservation Guarantee
//!
//! The last 10 turns are **never** compacted by layers 4 or 5. They are always
//! preserved verbatim to maintain conversation continuity.
//!
//! # Circuit Breaker
//!
//! If compaction fails 3 consecutive times, the circuit breaker trips and
//! subsequent compaction attempts return [`CompactionError::CircuitBreakerTripped`]
//! immediately.

use std::sync::atomic::{AtomicUsize, Ordering};

use talos_core::message::{AgentEvent, Message, MessageToolResult};
use talos_core::provider::LanguageModel;
use thiserror::Error;

use crate::token::TokenEstimator;

/// Maximum characters allowed per tool result after budget compaction.
const MAX_TOOL_RESULT_CHARS: usize = 4000;

/// Truncation suffix appended when a tool result is truncated.
const TRUNCATION_SUFFIX: &str = "... [truncated]";

/// Number of recent turns to preserve verbatim (never compacted by layers 4/5).
const PRESERVED_TURNS: usize = 10;

/// Turn threshold for trim layer — tool results older than this are removed.
const TRIM_TURN_THRESHOLD: usize = 20;

/// Turn threshold for collapse layer — turns older than this are summarized.
const COLLAPSE_TURN_THRESHOLD: usize = 10;

/// Maximum consecutive compaction failures before the circuit breaker trips.
const CIRCUIT_BREAKER_THRESHOLD: usize = 3;

/// Errors that can occur during context compaction.
#[derive(Debug, Error)]
pub enum CompactionError {
    /// Token estimation failed during compaction.
    #[error("token estimation failed")]
    TokenEstimationFailed,

    /// Compaction could not reduce context sufficiently.
    #[error("compaction failed: {0}")]
    CompactionFailed(String),

    /// The circuit breaker has tripped due to repeated failures.
    #[error("circuit breaker tripped after repeated compaction failures")]
    CircuitBreakerTripped,

    /// The LLM provider returned an error during summarization.
    #[error("provider error: {0}")]
    ProviderError(String),
}

/// Result alias for compaction operations.
pub type CompactionResult<T> = Result<T, CompactionError>;

/// Applies 5-layer context compaction when context nears the model token limit.
///
/// The compactor is stateless except for the circuit breaker counter, which
/// tracks consecutive failures across invocations.
///
/// # Example
///
/// ```no_run
/// use talos_agent::compaction::Compactor;
/// use talos_agent::token::TokenEstimator;
/// use talos_core::message::Message;
/// # use talos_core::provider::{LanguageModel, ProviderResult, Receiver};
/// # use talos_core::message::AgentEvent;
/// # struct MyModel;
/// # #[async_trait::async_trait]
/// # impl LanguageModel for MyModel {
/// #     async fn stream(&self, _: &[Message]) -> ProviderResult<Receiver<AgentEvent>> { unimplemented!() }
/// # }
/// # async fn example() {
/// let estimator = TokenEstimator::new();
/// let mut compactor = Compactor::new(estimator, 128_000);
///
/// let messages = vec![Message::User { content: "Hello!".into() }];
/// if compactor.should_compact(&messages) {
///     let provider: &dyn LanguageModel = &MyModel;
///     let compacted = compactor.compact(messages, provider).await.unwrap();
/// }
/// # }
/// ```
pub struct Compactor {
    /// Estimates token counts for messages.
    token_estimator: TokenEstimator,
    /// Maximum token limit of the target model.
    model_limit: u32,
    /// Trigger threshold as a fraction of model_limit (default: 0.8).
    trigger_threshold: f32,
    /// Consecutive compaction failure counter for circuit breaker.
    consecutive_failures: AtomicUsize,
}

impl Compactor {
    /// Creates a new compactor with the given token estimator and model limit.
    ///
    /// The trigger threshold defaults to 0.8 (80% of model_limit).
    ///
    /// # Arguments
    ///
    /// * `token_estimator` — The token estimator for measuring context size.
    /// * `model_limit` — Maximum token limit of the target language model.
    #[must_use]
    pub fn new(token_estimator: TokenEstimator, model_limit: u32) -> Self {
        Self {
            token_estimator,
            model_limit,
            trigger_threshold: 0.8,
            consecutive_failures: AtomicUsize::new(0),
        }
    }

    /// Sets the trigger threshold (fraction of model_limit that triggers compaction).
    ///
    /// # Arguments
    ///
    /// * `threshold` — A value between 0.0 and 1.0. Default is 0.8.
    #[must_use]
    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.trigger_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Checks whether compaction should be triggered for the given messages.
    ///
    /// Returns `true` if the estimated token usage exceeds `model_limit * trigger_threshold`.
    ///
    /// # Arguments
    ///
    /// * `messages` — The current conversation messages.
    pub fn should_compact(&self, messages: &[Message]) -> bool {
        let estimated = self.token_estimator.estimate(messages);
        let threshold_tokens = (self.model_limit as f32 * self.trigger_threshold) as u32;
        estimated > threshold_tokens
    }

    /// Applies compaction layers to reduce context size.
    ///
    /// Layers are applied in order (budget → trim → microcompact → collapse → autocompact),
    /// stopping as soon as the context fits within the model limit.
    ///
    /// The last 10 turns are always preserved verbatim.
    ///
    /// # Arguments
    ///
    /// * `messages` — The current conversation messages (consumed).
    /// * `provider` — The language model provider for LLM-based summarization.
    ///
    /// # Errors
    ///
    /// Returns [`CompactionError::CircuitBreakerTripped`] if the circuit breaker
    /// has tripped due to repeated failures.
    ///
    /// Returns [`CompactionError::CompactionFailed`] if all layers were applied
    /// but the context still exceeds the limit.
    ///
    /// Returns [`CompactionError::ProviderError`] if the LLM provider fails
    /// during summarization.
    pub async fn compact(
        &mut self,
        messages: Vec<Message>,
        provider: &dyn LanguageModel,
    ) -> CompactionResult<Vec<Message>> {
        if self.consecutive_failures.load(Ordering::SeqCst) >= CIRCUIT_BREAKER_THRESHOLD {
            return Err(CompactionError::CircuitBreakerTripped);
        }

        let mut current = messages;

        current = self.apply_budget(current);
        if self.fits(&current) {
            self.consecutive_failures.store(0, Ordering::SeqCst);
            return Ok(current);
        }

        current = self.apply_trim(current);
        if self.fits(&current) {
            self.consecutive_failures.store(0, Ordering::SeqCst);
            return Ok(current);
        }

        current = self.apply_microcompact(current);
        if self.fits(&current) {
            self.consecutive_failures.store(0, Ordering::SeqCst);
            return Ok(current);
        }

        current = match self.apply_collapse(current, provider).await {
            Ok(msgs) => msgs,
            Err(e) => {
                self.record_failure();
                return Err(e);
            }
        };
        if self.fits(&current) {
            self.consecutive_failures.store(0, Ordering::SeqCst);
            return Ok(current);
        }

        current = match self.apply_autocompact(current, provider).await {
            Ok(msgs) => msgs,
            Err(e) => {
                self.record_failure();
                return Err(e);
            }
        };

        if self.fits(&current) {
            self.consecutive_failures.store(0, Ordering::SeqCst);
            Ok(current)
        } else {
            self.record_failure();
            Err(CompactionError::CompactionFailed(
                "all compaction layers applied but context still exceeds limit".into(),
            ))
        }
    }

    /// Layer 1: Cap tool result sizes to max 4000 chars each.
    ///
    /// Truncates tool results exceeding [`MAX_TOOL_RESULT_CHARS`] characters,
    /// appending [`TRUNCATION_SUFFIX`].
    #[must_use]
    pub fn apply_budget(&self, messages: Vec<Message>) -> Vec<Message> {
        messages
            .into_iter()
            .map(|msg| match msg {
                Message::Tool { result } => {
                    if result.content.len() > MAX_TOOL_RESULT_CHARS {
                        let mut truncated = result.content[..MAX_TOOL_RESULT_CHARS].to_string();
                        truncated.push_str(TRUNCATION_SUFFIX);
                        Message::Tool {
                            result: MessageToolResult {
                                content: truncated,
                                ..result
                            },
                        }
                    } else {
                        Message::Tool { result }
                    }
                }
                other => other,
            })
            .collect()
    }

    /// Layer 2: Remove tool results from turns older than 20.
    ///
    /// Counts turns from the start of the conversation. Tool results belonging
    /// to turns beyond [`TRIM_TURN_THRESHOLD`] are removed (replaced with empty content).
    #[must_use]
    pub fn apply_trim(&self, messages: Vec<Message>) -> Vec<Message> {
        let total_turns = count_turns(&messages);
        if total_turns <= TRIM_TURN_THRESHOLD {
            return messages;
        }

        let turns_to_trim = total_turns - TRIM_TURN_THRESHOLD;
        let mut current_turn: usize = 0;
        let mut in_trimmed_turn = false;

        messages
            .into_iter()
            .map(|msg| {
                if matches!(&msg, Message::User { .. }) {
                    if current_turn > 0 {
                        in_trimmed_turn = false;
                    }
                    current_turn += 1;
                    if current_turn <= turns_to_trim {
                        in_trimmed_turn = true;
                    }
                }

                if in_trimmed_turn && matches!(&msg, Message::Tool { .. }) {
                    if let Message::Tool { result } = msg {
                        Message::Tool {
                            result: MessageToolResult {
                                content: String::new(),
                                ..result
                            },
                        }
                    } else {
                        msg
                    }
                } else {
                    msg
                }
            })
            .collect()
    }

    /// Layer 3: Keep only the last tool result for each tool call ID.
    ///
    /// Iterates through messages and for each `tool_use_id`, only preserves
    /// the most recent (last occurring) tool result. Earlier duplicates are
    /// replaced with empty content.
    #[must_use]
    pub fn apply_microcompact(&self, messages: Vec<Message>) -> Vec<Message> {
        let mut last_occurrence: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for (i, msg) in messages.iter().enumerate() {
            if let Message::Tool { result } = msg {
                last_occurrence.insert(result.tool_use_id.clone(), i);
            }
        }

        messages
            .into_iter()
            .enumerate()
            .map(|(i, msg)| {
                if let Message::Tool { result } = msg {
                    if last_occurrence.get(&result.tool_use_id) == Some(&i) {
                        Message::Tool { result }
                    } else {
                        Message::Tool {
                            result: MessageToolResult {
                                content: String::new(),
                                ..result
                            },
                        }
                    }
                } else {
                    msg
                }
            })
            .collect()
    }

    /// Layer 4: Summarize turns older than 10 into a single summary message.
    ///
    /// Uses the LLM to generate a concise summary of old turns. The last
    /// [`PRESERVED_TURNS`] turns are preserved verbatim.
    ///
    /// # Errors
    ///
    /// Returns [`CompactionError::ProviderError`] if the LLM provider fails.
    pub async fn apply_collapse(
        &self,
        messages: Vec<Message>,
        provider: &dyn LanguageModel,
    ) -> CompactionResult<Vec<Message>> {
        let total_turns = count_turns(&messages);
        if total_turns <= COLLAPSE_TURN_THRESHOLD {
            return Ok(messages);
        }

        let (old_messages, recent_messages) = split_at_turn(&messages, COLLAPSE_TURN_THRESHOLD);
        if old_messages.is_empty() {
            return Ok(messages);
        }

        let summary = self.summarize_with_llm(&old_messages, provider).await?;

        let mut result = Vec::with_capacity(1 + recent_messages.len());
        result.push(Message::User {
            content: format!(
                "[Conversation summary of {} earlier turns]\n{}",
                total_turns - COLLAPSE_TURN_THRESHOLD,
                summary
            ),
        });
        result.extend(recent_messages);

        Ok(result)
    }

    /// Layer 5: Use LLM to summarize the entire conversation history.
    ///
    /// Preserves the last [`PRESERVED_TURNS`] turns verbatim and summarizes
    /// everything before them.
    ///
    /// # Errors
    ///
    /// Returns [`CompactionError::ProviderError`] if the LLM provider fails.
    pub async fn apply_autocompact(
        &self,
        messages: Vec<Message>,
        provider: &dyn LanguageModel,
    ) -> CompactionResult<Vec<Message>> {
        let total_turns = count_turns(&messages);
        if total_turns <= PRESERVED_TURNS {
            return Ok(messages);
        }

        let (old_messages, recent_messages) = split_at_turn(&messages, PRESERVED_TURNS);
        if old_messages.is_empty() {
            return Ok(messages);
        }

        let summary = self.summarize_with_llm(&old_messages, provider).await?;

        let mut result = Vec::with_capacity(1 + recent_messages.len());
        result.push(Message::User {
            content: format!("[Full conversation summary]\n{}", summary),
        });
        result.extend(recent_messages);

        Ok(result)
    }

    /// Uses the LLM to summarize a set of messages.
    async fn summarize_with_llm(
        &self,
        messages: &[Message],
        provider: &dyn LanguageModel,
    ) -> CompactionResult<String> {
        let conversation_text = messages_to_text(messages);
        let prompt_messages = vec![Message::User {
            content: format!(
                "Summarize the following conversation concisely. \
                     Preserve key decisions, tool call outcomes, and important context. \
                     Keep the summary under 500 words.\n\n\
                     Conversation:\n{conversation_text}"
            ),
        }];

        let mut rx = provider
            .stream(&prompt_messages)
            .await
            .map_err(|e| CompactionError::ProviderError(e.to_string()))?;

        let mut summary = String::new();
        while let Some(event) = rx.recv().await {
            if let AgentEvent::TextDelta { delta } = event {
                summary.push_str(&delta);
            }
        }

        if summary.is_empty() {
            summary = "[No summary generated]".into();
        }

        Ok(summary)
    }

    /// Checks if the current messages fit within the model limit.
    fn fits(&self, messages: &[Message]) -> bool {
        let estimated = self.token_estimator.estimate(messages);
        estimated <= self.model_limit
    }

    /// Records a compaction failure for the circuit breaker.
    fn record_failure(&self) {
        self.consecutive_failures.fetch_add(1, Ordering::SeqCst);
    }

    /// Returns the current consecutive failure count (for testing).
    #[cfg(test)]
    fn failure_count(&self) -> usize {
        self.consecutive_failures.load(Ordering::SeqCst)
    }
}

/// Counts the number of turns in a message list.
///
/// A turn is counted each time a `User` message appears.
fn count_turns(messages: &[Message]) -> usize {
    messages
        .iter()
        .filter(|m| matches!(m, Message::User { .. }))
        .count()
}

/// Splits messages at the given turn boundary (counting from the end).
///
/// Returns `(old_messages, recent_messages)` where `recent_messages` contains
/// the last `turns_from_end` turns.
fn split_at_turn(messages: &[Message], turns_from_end: usize) -> (Vec<Message>, Vec<Message>) {
    let total_turns = count_turns(messages);
    if total_turns <= turns_from_end {
        return (Vec::new(), messages.to_vec());
    }

    let turns_to_keep = turns_from_end;
    let turns_to_skip = total_turns - turns_to_keep;

    let mut current_turn: usize = 0;
    let mut split_idx = 0;

    for (i, msg) in messages.iter().enumerate() {
        if matches!(msg, Message::User { .. }) {
            current_turn += 1;
            if current_turn > turns_to_skip {
                split_idx = i;
                break;
            }
        }
    }

    let old = messages[..split_idx].to_vec();
    let recent = messages[split_idx..].to_vec();
    (old, recent)
}

/// Converts messages to a plain text representation for LLM summarization.
fn messages_to_text(messages: &[Message]) -> String {
    messages
        .iter()
        .map(|msg| match msg {
            Message::User { content } => format!("User: {content}"),
            Message::System { content, .. } => format!("System: {content}"),
            Message::Context { content } => format!("Context: {content}"),
            Message::Assistant {
                content,
                tool_calls,
            } => {
                let mut text = format!("Assistant: {content}");
                for tc in tool_calls {
                    text.push_str(&format!("\n  [Tool call: {}({})]", tc.name, tc.input));
                }
                text
            }
            Message::Tool { result } => {
                format!("Tool result ({}): {}", result.tool_use_id, result.content)
            }
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use talos_core::message::{ToolCall, Usage};
    use talos_core::provider::{ProviderError, ProviderResult};
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
        }
    }

    /// Helper: create an assistant message with tool calls.
    fn assistant_with_tools(content: &str, tools: Vec<ToolCall>) -> Message {
        Message::Assistant {
            content: content.into(),
            tool_calls: tools,
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

        let provider =
            SummaryMockProvider::new("Summary: user asked about files, assistant read them");
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
}
