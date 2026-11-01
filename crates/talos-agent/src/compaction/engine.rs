use talos_core::message::{Message, MessageToolResult};
use talos_core::provider::LanguageModel;

use crate::token::TokenEstimator;

use super::constants::{MAX_TOOL_RESULT_CHARS, TRIM_TURN_THRESHOLD, TRUNCATION_SUFFIX};
use super::{CompactionError, CompactionResult, CompactionStatus};

/// Legacy deterministic message transformations; not a session compaction authority.
///
/// These utilities do not perform whole-request admission, authorize disclosure,
/// or preserve durable history on behalf of the caller. Never replace authoritative
/// session history with their output. Model-assisted entry points are deprecated
/// and refuse execution; use RuntimeHandle::compact or SessionOp::Compact instead.
///
/// # Example
///
/// ```
/// use talos_agent::{compaction::Compactor, token::TokenEstimator};
/// use talos_core::message::Message;
/// let compactor = Compactor::new(TokenEstimator::new(), 128_000);
/// let projection = compactor.apply_budget(vec![Message::User { content: "Hello!".into() }]);
/// assert_eq!(projection.len(), 1);
/// ```
pub struct Compactor {
    /// Estimates token counts for messages.
    token_estimator: TokenEstimator,
    /// Maximum token limit of the target model.
    model_limit: u32,
    /// Trigger threshold as a fraction of model_limit (default: 0.8).
    trigger_threshold: f32,
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

    /// Refuses legacy model compaction without complete request authority.
    ///
    /// # Errors
    /// Always returns `CompactionFailed`; no provider call or preprocessing occurs.
    #[deprecated(
        note = "use RuntimeHandle::compact or SessionOp::Compact; bare history lacks request authority"
    )]
    pub async fn compact(
        &mut self,
        _messages: Vec<Message>,
        _provider: &dyn LanguageModel,
    ) -> CompactionResult<Vec<Message>> {
        Err(legacy_authority_error())
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
                    if result.content.chars().count() > MAX_TOOL_RESULT_CHARS {
                        let mut truncated: String =
                            result.content.chars().take(MAX_TOOL_RESULT_CHARS).collect();
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

    /// Refuses legacy summarization; migrate to the typed session compact operation.
    ///
    /// # Errors
    /// Always returns `CompactionFailed`, including for empty/below-threshold input.
    #[deprecated(
        note = "use RuntimeHandle::compact or SessionOp::Compact; bare history lacks request authority"
    )]
    pub async fn apply_collapse(
        &self,
        _messages: Vec<Message>,
        _provider: &dyn LanguageModel,
    ) -> CompactionResult<Vec<Message>> {
        Err(legacy_authority_error())
    }

    /// Refuses legacy summarization; migrate to the typed session compact operation.
    ///
    /// # Errors
    /// Always returns `CompactionFailed`, including for empty/below-threshold input.
    #[deprecated(
        note = "use RuntimeHandle::compact or SessionOp::Compact; bare history lacks request authority"
    )]
    pub async fn apply_autocompact(
        &self,
        _messages: Vec<Message>,
        _provider: &dyn LanguageModel,
    ) -> CompactionResult<Vec<Message>> {
        Err(legacy_authority_error())
    }

    /// Checks if the current messages fit within the model limit.
    fn fits(&self, messages: &[Message]) -> bool {
        let estimated = self.token_estimator.estimate(messages);
        estimated <= self.model_limit
    }

    /// Apply deterministic layers 1-3 (budget, trim, microcompact) and return status.
    ///
    /// This explicit local transformation does not authorize session dispatch,
    /// disclosure or persistent history replacement. `Skipped` means these
    /// utilities cannot fit the input; use typed session compaction for live work.
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
                    reason: "deterministic layers insufficient; use typed session compaction",
                    tokens_current: tokens_after,
                },
            )
        }
    }

    /// Refuses legacy manual compaction and returns the exact original messages.
    ///
    /// Always reports `Failed`; no threshold shortcut, preprocessing or provider
    /// call occurs. Use RuntimeHandle::compact or SessionOp::Compact and await
    /// the operation's transient terminal receipt instead.
    #[deprecated(
        note = "use RuntimeHandle::compact or SessionOp::Compact; bare history lacks request authority"
    )]
    pub async fn manual_compact(
        &mut self,
        messages: Vec<Message>,
        _provider: &dyn LanguageModel,
    ) -> (Vec<Message>, CompactionStatus) {
        (
            messages,
            CompactionStatus::Failed {
                error: legacy_authority_error().to_string(),
            },
        )
    }
}

fn legacy_authority_error() -> CompactionError {
    CompactionError::CompactionFailed(
        "legacy compaction lacks complete request authority; use RuntimeHandle::compact or SessionOp::Compact".into(),
    )
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
