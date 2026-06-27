use std::sync::atomic::{AtomicUsize, Ordering};

use talos_core::message::{AgentEvent, Message, MessageToolResult};
use talos_core::provider::LanguageModel;

use crate::token::TokenEstimator;

use super::constants::{
    CIRCUIT_BREAKER_THRESHOLD, COLLAPSE_TURN_THRESHOLD, MAX_TOOL_RESULT_CHARS, PRESERVED_TURNS,
    TRIM_TURN_THRESHOLD, TRUNCATION_SUFFIX,
};
use super::{CompactionError, CompactionResult, CompactionStatus};

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

    /// Apply deterministic layers 1-3 (budget, trim, microcompact) and return status.
    ///
    /// Safe at any boundary (pre-turn, manual) because it does not invoke the
    /// LLM. If deterministic layers are insufficient, the status reports
    /// `Skipped` with the reason — the caller can then decide whether to
    /// escalate to [`compact`](Self::compact) (which uses LLM layers 4-5).
    #[must_use]
    pub fn compact_deterministic(
        &self,
        messages: Vec<Message>,
    ) -> (Vec<Message>, CompactionStatus) {
        let tokens_before = self.token_estimator.estimate(&messages);
        let mut current = messages;
        let mut layers = Vec::new();

        current = self.apply_budget(current);
        layers.push("budget");
        if self.fits(&current) {
            let tokens_after = self.token_estimator.estimate(&current);
            return (
                current,
                CompactionStatus::Applied {
                    layers_applied: layers,
                    tokens_before,
                    tokens_after,
                },
            );
        }

        current = self.apply_trim(current);
        layers.push("trim");
        if self.fits(&current) {
            let tokens_after = self.token_estimator.estimate(&current);
            return (
                current,
                CompactionStatus::Applied {
                    layers_applied: layers,
                    tokens_before,
                    tokens_after,
                },
            );
        }

        current = self.apply_microcompact(current);
        layers.push("microcompact");
        let tokens_after = self.token_estimator.estimate(&current);

        if self.fits(&current) {
            (
                current,
                CompactionStatus::Applied {
                    layers_applied: layers,
                    tokens_before,
                    tokens_after,
                },
            )
        } else {
            (
                current,
                CompactionStatus::Skipped {
                    reason: "deterministic layers insufficient; LLM layers required",
                    tokens_current: tokens_after,
                },
            )
        }
    }

    /// Manual compaction trigger that returns status without exposing hidden output.
    ///
    /// Checks the trigger threshold first. If the context fits, returns
    /// `Skipped`. If the circuit breaker is tripped, returns `Failed`.
    /// Otherwise delegates to [`compact`](Self::compact) and wraps the result.
    pub async fn manual_compact(
        &mut self,
        messages: Vec<Message>,
        provider: &dyn LanguageModel,
    ) -> (Vec<Message>, CompactionStatus) {
        if !self.should_compact(&messages) {
            let tokens = self.token_estimator.estimate(&messages);
            return (
                messages,
                CompactionStatus::Skipped {
                    reason: "below trigger threshold",
                    tokens_current: tokens,
                },
            );
        }

        let tokens_before = self.token_estimator.estimate(&messages);

        match self.compact(messages.clone(), provider).await {
            Ok(compacted) => {
                let tokens_after = self.token_estimator.estimate(&compacted);
                (
                    compacted,
                    CompactionStatus::Applied {
                        layers_applied: vec!["manual"],
                        tokens_before,
                        tokens_after,
                    },
                )
            }
            Err(e) => (
                messages,
                CompactionStatus::Failed {
                    error: e.to_string(),
                },
            ),
        }
    }

    /// Records a compaction failure for the circuit breaker.
    fn record_failure(&self) {
        self.consecutive_failures.fetch_add(1, Ordering::SeqCst);
    }

    /// Returns the current consecutive failure count (for testing).
    #[cfg(test)]
    pub(super) fn failure_count(&self) -> usize {
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
