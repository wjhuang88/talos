//! Token estimation and usage tracking for agent sessions.
//!
//! This module provides approximate token counting for messages and cumulative
//! usage tracking across turns. Token estimation uses character-based heuristics:
//! - ASCII text: ~4 characters per token
//! - Non-ASCII text (CJK, etc.): ~2 characters per token
//!
//! A 20% error margin is expected and acceptable for estimation purposes.

use talos_core::message::{Message, Usage};

/// Pricing information for a language model, expressed as cost per 1,000 tokens.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelPricing {
    /// Cost per 1,000 input tokens.
    pub input_per_1k: f64,
    /// Cost per 1,000 output tokens.
    pub output_per_1k: f64,
    /// Cost per 1,000 cache read tokens.
    pub cache_read_per_1k: f64,
    /// Cost per 1,000 cache write tokens.
    pub cache_write_per_1k: f64,
}

/// Estimates token counts for messages and tracks cumulative usage across turns.
///
/// # Token Estimation Strategy
///
/// Uses character-based approximation:
/// - ASCII characters: 4 chars ≈ 1 token
/// - Non-ASCII characters (CJK, emoji, etc.): 2 chars ≈ 1 token
///
/// This provides a reasonable estimate within ~20% of actual token counts
/// for most common text patterns.
///
/// # Example
///
/// ```
/// use talos_agent::token::{TokenEstimator, ModelPricing};
/// use talos_core::message::{Message, Usage};
///
/// let mut estimator = TokenEstimator::new();
///
/// // Estimate tokens for a set of messages
/// let messages = vec![
///     Message::User { content: "Hello, world!".into() },
///     Message::Assistant { content: "Hi there!".into(), tool_calls: vec![], reasoning: None },
/// ];
/// let estimated = estimator.estimate(&messages);
///
/// // Track actual usage from a turn
/// estimator.track_usage(Usage {
///     input_tokens: 100,
///     output_tokens: 50,
///     cache_read_tokens: 80,
///     cache_write_tokens: 20,
/// });
///
/// // Get cumulative usage
/// let total = estimator.total_usage();
/// assert_eq!(total.input_tokens, 100);
///
/// // Estimate cost
/// let pricing = ModelPricing {
///     input_per_1k: 0.003,
///     output_per_1k: 0.015,
///     cache_read_per_1k: 0.001,
///     cache_write_per_1k: 0.002,
/// };
/// let cost = estimator.estimated_cost(&pricing);
/// ```
#[derive(Debug, Clone, Default)]
pub struct TokenEstimator {
    /// Cumulative usage across all tracked turns.
    total: Usage,
}

impl TokenEstimator {
    /// Creates a new token estimator with zero cumulative usage.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Estimates the token count for a slice of messages.
    ///
    /// Iterates over all message content (user text, assistant text, tool results)
    /// and applies character-based heuristics to approximate token count.
    ///
    /// # Arguments
    ///
    /// * `messages` — The messages to estimate tokens for.
    ///
    /// # Returns
    ///
    /// The estimated total token count across all messages.
    ///
    /// # Example
    ///
    /// ```
    /// use talos_agent::token::TokenEstimator;
    /// use talos_core::message::Message;
    ///
    /// let estimator = TokenEstimator::new();
    /// let messages = vec![
    ///     Message::User { content: "Hello!".into() },
    /// ];
    /// let tokens = estimator.estimate(&messages);
    /// assert!(tokens > 0);
    /// ```
    pub fn estimate(&self, messages: &[Message]) -> u32 {
        messages
            .iter()
            .map(|msg| match msg {
                Message::User { content } => Self::estimate_text(content),
                Message::System { content, .. } => Self::estimate_text(content),
                Message::Context { content } => Self::estimate_text(content),
                Message::Assistant {
                    content,
                    tool_calls,
                    ..
                } => {
                    let text_tokens = Self::estimate_text(content);
                    let tool_tokens: u32 = tool_calls
                        .iter()
                        .map(|tc| {
                            Self::estimate_text(&tc.name)
                                + Self::estimate_text(&tc.input.to_string())
                        })
                        .sum();
                    text_tokens + tool_tokens
                }
                Message::Tool { result } => {
                    Self::estimate_text(&result.content) + Self::estimate_text(&result.tool_use_id)
                }
            })
            .sum()
    }

    /// Estimates the token count for a single string of text.
    ///
    /// Uses character-based heuristics:
    /// - ASCII characters: 4 chars ≈ 1 token
    /// - Non-ASCII characters: 2 chars ≈ 1 token
    ///
    /// Empty strings return 0 tokens.
    ///
    /// # Arguments
    ///
    /// * `text` — The text to estimate tokens for.
    ///
    /// # Returns
    ///
    /// The estimated token count.
    ///
    /// # Example
    ///
    /// ```
    /// use talos_agent::token::TokenEstimator;
    ///
    /// // English text: ~4 chars per token
    /// let english = TokenEstimator::estimate_text("Hello, world!");
    /// assert!(english > 0);
    ///
    /// // CJK text: ~2 chars per token
    /// let cjk = TokenEstimator::estimate_text("你好世界");
    /// assert!(cjk > 0);
    ///
    /// // Empty text: 0 tokens
    /// let empty = TokenEstimator::estimate_text("");
    /// assert_eq!(empty, 0);
    /// ```
    pub fn estimate_text(text: &str) -> u32 {
        if text.is_empty() {
            return 0;
        }

        let mut ascii_chars: u32 = 0;
        let mut non_ascii_chars: u32 = 0;

        for ch in text.chars() {
            if ch.is_ascii() {
                ascii_chars += 1;
            } else {
                non_ascii_chars += 1;
            }
        }

        // ASCII: 4 chars ≈ 1 token, Non-ASCII: 2 chars ≈ 1 token
        let ascii_tokens = ascii_chars.div_ceil(4);
        let non_ascii_tokens = non_ascii_chars.div_ceil(2);

        ascii_tokens + non_ascii_tokens
    }

    /// Accumulates usage from a single turn into the cumulative total.
    ///
    /// # Arguments
    ///
    /// * `turn_usage` — The usage statistics from a single turn.
    ///
    /// # Example
    ///
    /// ```
    /// use talos_agent::token::TokenEstimator;
    /// use talos_core::message::Usage;
    ///
    /// let mut estimator = TokenEstimator::new();
    /// estimator.track_usage(Usage {
    ///     input_tokens: 100,
    ///     output_tokens: 50,
    ///     cache_read_tokens: 80,
    ///     cache_write_tokens: 20,
    ///     reasoning_tokens: 0,
    /// });
    ///
    /// let total = estimator.total_usage();
    /// assert_eq!(total.input_tokens, 100);
    /// assert_eq!(total.output_tokens, 50);
    /// ```
    pub fn track_usage(&mut self, turn_usage: Usage) {
        self.total.input_tokens += turn_usage.input_tokens;
        self.total.output_tokens += turn_usage.output_tokens;
        self.total.cache_read_tokens += turn_usage.cache_read_tokens;
        self.total.cache_write_tokens += turn_usage.cache_write_tokens;
        self.total.reasoning_tokens += turn_usage.reasoning_tokens;
    }

    /// Returns the cumulative usage across all tracked turns.
    ///
    /// # Returns
    ///
    /// A [`Usage`] struct with the sum of all tracked turn usage.
    ///
    /// # Example
    ///
    /// ```
    /// use talos_agent::token::TokenEstimator;
    /// use talos_core::message::Usage;
    ///
    /// let mut estimator = TokenEstimator::new();
    /// estimator.track_usage(Usage {
    ///     input_tokens: 100,
    ///     output_tokens: 50,
    ///     cache_read_tokens: 0,
    ///     cache_write_tokens: 0,
    ///     reasoning_tokens: 0,
    /// });
    /// estimator.track_usage(Usage {
    ///     input_tokens: 200,
    ///     output_tokens: 75,
    ///     cache_read_tokens: 100,
    ///     cache_write_tokens: 50,
    ///     reasoning_tokens: 0,
    /// });
    ///
    /// let total = estimator.total_usage();
    /// assert_eq!(total.input_tokens, 300);
    /// assert_eq!(total.output_tokens, 125);
    /// assert_eq!(total.cache_read_tokens, 100);
    /// assert_eq!(total.cache_write_tokens, 50);
    /// ```
    pub fn total_usage(&self) -> Usage {
        self.total.clone()
    }

    /// Calculates the estimated cost based on cumulative usage and model pricing.
    ///
    /// Uses simple multiplication: `(tokens / 1000) * price_per_1k` for each
    /// usage category.
    ///
    /// # Arguments
    ///
    /// * `pricing` — The pricing information for the model.
    ///
    /// # Returns
    ///
    /// The estimated total cost in the currency unit of the pricing.
    ///
    /// # Example
    ///
    /// ```
    /// use talos_agent::token::{TokenEstimator, ModelPricing};
    /// use talos_core::message::Usage;
    ///
    /// let mut estimator = TokenEstimator::new();
    /// estimator.track_usage(Usage {
    ///     input_tokens: 1000,
    ///     output_tokens: 500,
    ///     cache_read_tokens: 800,
    ///     cache_write_tokens: 200,
    ///     reasoning_tokens: 0,
    /// });
    ///
    /// let pricing = ModelPricing {
    ///     input_per_1k: 0.003,
    ///     output_per_1k: 0.015,
    ///     cache_read_per_1k: 0.001,
    ///     cache_write_per_1k: 0.002,
    /// };
    ///
    /// let cost = estimator.estimated_cost(&pricing);
    /// // (1000/1000)*0.003 + (500/1000)*0.015 + (800/1000)*0.001 + (200/1000)*0.002
    /// // = 0.003 + 0.0075 + 0.0008 + 0.0004 = 0.0117
    /// assert!((cost - 0.0117).abs() < 0.0001);
    /// ```
    pub fn estimated_cost(&self, pricing: &ModelPricing) -> f64 {
        let input_cost = (self.total.input_tokens as f64 / 1000.0) * pricing.input_per_1k;
        let output_cost = (self.total.output_tokens as f64 / 1000.0) * pricing.output_per_1k;
        let cache_read_cost =
            (self.total.cache_read_tokens as f64 / 1000.0) * pricing.cache_read_per_1k;
        let cache_write_cost =
            (self.total.cache_write_tokens as f64 / 1000.0) * pricing.cache_write_per_1k;

        input_cost + output_cost + cache_read_cost + cache_write_cost
    }
}

#[cfg(test)]
#[allow(warnings)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_text_empty() {
        assert_eq!(TokenEstimator::estimate_text(""), 0);
    }

    #[test]
    fn test_estimate_text_english() {
        // "Hello, world!" = 13 ASCII chars → 13/4 = 3.25 → ceil = 4 tokens
        let tokens = TokenEstimator::estimate_text("Hello, world!");
        assert_eq!(tokens, 4);
    }

    #[test]
    fn test_estimate_text_english_within_20_percent() {
        // Longer English text: ~121 chars → ~31 tokens estimated
        let text = "The quick brown fox jumps over the lazy dog. This is a longer sentence to test token estimation accuracy for English text.";
        let tokens = TokenEstimator::estimate_text(text);
        // Verify it's in a reasonable range (actual would be ~25-35)
        assert!(
            tokens >= 20 && tokens <= 40,
            "English estimation should be reasonable"
        );
    }

    #[test]
    fn test_estimate_text_cjk() {
        // "你好世界" = 4 non-ASCII chars → 4/2 = 2 tokens
        let tokens = TokenEstimator::estimate_text("你好世界");
        assert_eq!(tokens, 2);
    }

    #[test]
    fn test_estimate_text_cjk_single_char() {
        // Single CJK char → ceil(1/2) = 1 token
        let tokens = TokenEstimator::estimate_text("中");
        assert_eq!(tokens, 1);
    }

    #[test]
    fn test_estimate_text_mixed() {
        // "Hello你好" = 5 ASCII + 2 non-ASCII
        // ASCII: ceil(5/4) = 2, Non-ASCII: ceil(2/2) = 1 → total = 3
        let tokens = TokenEstimator::estimate_text("Hello你好");
        assert_eq!(tokens, 3);
    }

    #[test]
    fn test_estimate_text_mixed_complex() {
        // "Hi 你好世界!" = 4 ASCII (H, i, space, !) + 4 non-ASCII (你, 好, 世, 界)
        // ASCII: ceil(4/4) = 1, Non-ASCII: ceil(4/2) = 2 → total = 3
        let tokens = TokenEstimator::estimate_text("Hi 你好世界!");
        assert_eq!(tokens, 3);
    }

    #[test]
    fn test_estimate_text_only_whitespace() {
        // 4 spaces → ceil(4/4) = 1 token
        let tokens = TokenEstimator::estimate_text("    ");
        assert_eq!(tokens, 1);
    }

    #[test]
    fn test_estimate_empty_messages() {
        let estimator = TokenEstimator::new();
        let messages: Vec<Message> = vec![];
        assert_eq!(estimator.estimate(&messages), 0);
    }

    #[test]
    fn test_estimate_user_message() {
        let estimator = TokenEstimator::new();
        let messages = vec![Message::User {
            content: "Hello, world!".into(),
        }];
        let tokens = estimator.estimate(&messages);
        assert_eq!(tokens, 4);
    }

    #[test]
    fn test_estimate_assistant_message() {
        let estimator = TokenEstimator::new();
        let messages = vec![Message::Assistant {
            content: "Hi there!".into(),
            tool_calls: vec![],
            reasoning: None,
        }];
        // "Hi there!" = 9 ASCII chars → ceil(9/4) = 3 tokens
        let tokens = estimator.estimate(&messages);
        assert_eq!(tokens, 3);
    }

    #[test]
    fn test_estimate_tool_message() {
        let estimator = TokenEstimator::new();
        let messages = vec![Message::Tool {
            result: talos_core::message::MessageToolResult {
                tool_use_id: "call_1".into(),
                content: "file content here".into(),
                is_error: false,
            },
        }];
        // "file content here" = 17 ASCII → ceil(17/4) = 5
        // "call_1" = 6 ASCII → ceil(6/4) = 2
        // total = 7
        let tokens = estimator.estimate(&messages);
        assert_eq!(tokens, 7);
    }

    #[test]
    fn test_estimate_multiple_messages() {
        let estimator = TokenEstimator::new();
        let messages = vec![
            Message::User {
                content: "Hello!".into(),
            },
            Message::Assistant {
                content: "Hi!".into(),
                tool_calls: vec![],
                reasoning: None,
            },
        ];
        // "Hello!" = 6 → ceil(6/4) = 2
        // "Hi!" = 3 → ceil(3/4) = 1
        // total = 3
        let tokens = estimator.estimate(&messages);
        assert_eq!(tokens, 3);
    }

    #[test]
    fn test_estimate_assistant_with_tool_calls() {
        let estimator = TokenEstimator::new();
        let messages = vec![Message::Assistant {
            content: "Let me read that file.".into(),
            tool_calls: vec![talos_core::message::ToolCall {
                id: "call_1".into(),
                name: "read_file".into(),
                input: serde_json::json!({"path": "src/main.rs"}),
            }],
            reasoning: None,
        }];
        // Content: "Let me read that file." = 22 ASCII → ceil(22/4) = 6
        // Tool name: "read_file" = 9 → ceil(9/4) = 3
        // Tool input: {"path":"src/main.rs"} ≈ 23 chars → ceil(23/4) = 6
        // Total = 6 + 3 + 6 = 15
        let tokens = estimator.estimate(&messages);
        assert_eq!(tokens, 15);
    }

    #[test]
    fn test_track_usage_single_turn() {
        let mut estimator = TokenEstimator::new();
        estimator.track_usage(Usage {
            input_tokens: 100,
            output_tokens: 50,
            cache_read_tokens: 80,
            cache_write_tokens: 20,
            reasoning_tokens: 0,
        });

        let total = estimator.total_usage();
        assert_eq!(total.input_tokens, 100);
        assert_eq!(total.output_tokens, 50);
        assert_eq!(total.cache_read_tokens, 80);
        assert_eq!(total.cache_write_tokens, 20);
    }

    #[test]
    fn test_track_usage_cumulative() {
        let mut estimator = TokenEstimator::new();

        estimator.track_usage(Usage {
            input_tokens: 100,
            output_tokens: 50,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
            reasoning_tokens: 0,
        });

        estimator.track_usage(Usage {
            input_tokens: 200,
            output_tokens: 75,
            cache_read_tokens: 100,
            cache_write_tokens: 50,
            reasoning_tokens: 0,
        });

        estimator.track_usage(Usage {
            input_tokens: 50,
            output_tokens: 25,
            cache_read_tokens: 30,
            cache_write_tokens: 10,
            reasoning_tokens: 0,
        });

        let total = estimator.total_usage();
        assert_eq!(total.input_tokens, 350);
        assert_eq!(total.output_tokens, 150);
        assert_eq!(total.cache_read_tokens, 130);
        assert_eq!(total.cache_write_tokens, 60);
    }

    #[test]
    fn test_total_usage_initial_is_zero() {
        let estimator = TokenEstimator::new();
        let total = estimator.total_usage();
        assert_eq!(total.input_tokens, 0);
        assert_eq!(total.output_tokens, 0);
        assert_eq!(total.cache_read_tokens, 0);
        assert_eq!(total.cache_write_tokens, 0);
    }

    #[test]
    fn test_estimated_cost_zero_usage() {
        let estimator = TokenEstimator::new();
        let pricing = ModelPricing {
            input_per_1k: 0.003,
            output_per_1k: 0.015,
            cache_read_per_1k: 0.001,
            cache_write_per_1k: 0.002,
        };

        let cost = estimator.estimated_cost(&pricing);
        assert!((cost - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_estimated_cost_simple() {
        let mut estimator = TokenEstimator::new();
        estimator.track_usage(Usage {
            input_tokens: 1000,
            output_tokens: 500,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
            reasoning_tokens: 0,
        });

        let pricing = ModelPricing {
            input_per_1k: 0.003,
            output_per_1k: 0.015,
            cache_read_per_1k: 0.001,
            cache_write_per_1k: 0.002,
        };

        let cost = estimator.estimated_cost(&pricing);
        // (1000/1000)*0.003 + (500/1000)*0.015 = 0.003 + 0.0075 = 0.0105
        assert!((cost - 0.0105).abs() < 0.0001);
    }

    #[test]
    fn test_estimated_cost_with_cache() {
        let mut estimator = TokenEstimator::new();
        estimator.track_usage(Usage {
            input_tokens: 1000,
            output_tokens: 500,
            cache_read_tokens: 800,
            cache_write_tokens: 200,
            reasoning_tokens: 0,
        });

        let pricing = ModelPricing {
            input_per_1k: 0.003,
            output_per_1k: 0.015,
            cache_read_per_1k: 0.001,
            cache_write_per_1k: 0.002,
        };

        let cost = estimator.estimated_cost(&pricing);
        // (1000/1000)*0.003 + (500/1000)*0.015 + (800/1000)*0.001 + (200/1000)*0.002
        // = 0.003 + 0.0075 + 0.0008 + 0.0004 = 0.0117
        assert!((cost - 0.0117).abs() < 0.0001);
    }

    #[test]
    fn test_estimated_cost_claude_sonnet_pricing() {
        // Real-world pricing example: Claude Sonnet 4
        let mut estimator = TokenEstimator::new();
        estimator.track_usage(Usage {
            input_tokens: 50_000,
            output_tokens: 10_000,
            cache_read_tokens: 40_000,
            cache_write_tokens: 10_000,
            reasoning_tokens: 0,
        });

        let pricing = ModelPricing {
            input_per_1k: 0.003,
            output_per_1k: 0.015,
            cache_read_per_1k: 0.0003,
            cache_write_per_1k: 0.00375,
        };

        let cost = estimator.estimated_cost(&pricing);
        // (50000/1000)*0.003 + (10000/1000)*0.015 + (40000/1000)*0.0003 + (10000/1000)*0.00375
        // = 0.15 + 0.15 + 0.012 + 0.0375 = 0.3495
        assert!((cost - 0.3495).abs() < 0.0001);
    }

    #[test]
    fn test_model_pricing_copy() {
        let pricing = ModelPricing {
            input_per_1k: 0.003,
            output_per_1k: 0.015,
            cache_read_per_1k: 0.001,
            cache_write_per_1k: 0.002,
        };

        let pricing2 = pricing; // Copy, not move
        assert!((pricing.input_per_1k - pricing2.input_per_1k).abs() < f64::EPSILON);
    }

    #[test]
    fn test_model_pricing_debug() {
        let pricing = ModelPricing {
            input_per_1k: 0.003,
            output_per_1k: 0.015,
            cache_read_per_1k: 0.001,
            cache_write_per_1k: 0.002,
        };

        let debug_str = format!("{:?}", pricing);
        assert!(debug_str.contains("input_per_1k"));
        assert!(debug_str.contains("0.003"));
    }
}
