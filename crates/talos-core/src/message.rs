//! Core message types and event protocol.

use serde::{Deserialize, Serialize};

use crate::tool::ToolProvenance;

/// Provider-side caching behavior for a system prompt range.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SystemCacheType {
    /// Cache this prompt range ephemerally when the provider supports it.
    Ephemeral,
}

/// A byte range in the system prompt that is stable enough for provider caching.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemCacheMarker {
    /// Starting byte offset in the system prompt content.
    pub offset: usize,
    /// Length of the cacheable range in bytes.
    pub length: usize,
    /// Cache behavior requested for this range.
    pub cache_type: SystemCacheType,
}

/// A tool call requested by the assistant.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCall {
    /// Unique identifier for this tool call.
    pub id: String,
    /// Name of the tool to invoke.
    pub name: String,
    /// JSON-encoded arguments for the tool.
    pub input: serde_json::Value,
}

/// Result of a tool execution (message-layer).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageToolResult {
    /// ID of the tool call this result corresponds to.
    pub tool_use_id: String,
    /// Text output from the tool.
    pub content: String,
    /// Whether the tool execution failed.
    pub is_error: bool,
}

/// One provider-native reasoning block attached to an assistant message.
///
/// See ADR-034 for the full boundary design.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReasoningBlock {
    /// Signed thinking (Anthropic `thinking` block). `text` may be empty when
    /// the provider omits display text; `signature` is opaque and must be
    /// replayed byte-for-byte, never inspected or trimmed.
    Thinking {
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// Encrypted redacted thinking (Anthropic `redacted_thinking`). Replayed
    /// byte-for-byte; never rendered anywhere.
    Redacted { data: String },
    /// Plain reasoning text (OpenAI-compatible `reasoning_content`).
    Plain { text: String },
}

/// Reasoning payload for one assistant message, stamped with the identity
/// that produced it. Request-history metadata only — never display content.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssistantReasoning {
    /// Config provider key that produced the blocks (e.g. `anthropic`, `my-gateway`).
    pub provider: String,
    /// Model id that produced the blocks (e.g. `claude-sonnet-4-5`).
    pub model: String,
    /// Provider-native blocks in stream order.
    pub blocks: Vec<ReasoningBlock>,
}

/// Cryptographic digest of an image attachment's bytes at grant time.
///
/// Stored on `ContentPart::Image` so the provider adapter can detect
/// same-path replacement attacks: when the adapter re-reads the file
/// at request time, it recomputes the digest and compares. A mismatch
/// means the file was replaced between grant and read, and the part
/// MUST be omitted (Owner P1-B security rework, 2026-07-21).
///
/// Backed by SHA-256 (`[u8; 32]`). The hash itself is computed in
/// `talos_cli::image_validation`; talos-core only carries the typed
/// digest so it has no `sha2` dependency. Serde uses a lowercase hex
/// string so TLOG dumps remain human-readable.
///
/// The default is the all-zero "unverified" sentinel: a freshly
/// constructed ContentPart that has not yet been through
/// `validate_image_path` carries this. Provider adapters treat the
/// default as "verification intentionally skipped" — only test
/// fixtures and unverified in-memory parts use this path.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContentDigest([u8; 32]);

impl ContentDigest {
    /// Wrap an existing raw SHA-256 digest.
    pub const fn from_raw(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the raw 32-byte digest.
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Returns the digest formatted as a lowercase hex string.
    pub fn to_hex(&self) -> String {
        let mut out = String::with_capacity(64);
        for byte in self.0 {
            out.push_str(&format!("{byte:02x}"));
        }
        out
    }

    /// Parses a 64-char lowercase hex string into a digest.
    pub fn from_hex(s: &str) -> Result<Self, String> {
        if s.len() != 64 {
            return Err(format!(
                "content_digest must be 64 hex chars, got {}",
                s.len()
            ));
        }
        let mut bytes = [0u8; 32];
        for (i, chunk) in s.as_bytes().chunks(2).enumerate() {
            let hex = std::str::from_utf8(chunk).map_err(|e| e.to_string())?;
            bytes[i] = u8::from_str_radix(hex, 16).map_err(|e| e.to_string())?;
        }
        Ok(Self(bytes))
    }
}

impl std::fmt::Display for ContentDigest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_hex())
    }
}

impl serde::Serialize for ContentDigest {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_hex())
    }
}

impl<'de> serde::Deserialize<'de> for ContentDigest {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

/// One part of an ordered multimodal message content (ADR-050).
///
/// Provider wire format (data URL, base64 source) is constructed inside
/// `talos-provider` adapters at request time. The core type carries the
/// canonical path, MIME type, byte count, and a content digest — no
/// image bytes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    Text {
        text: String,
    },
    Image {
        path: std::path::PathBuf,
        mime: String,
        byte_count: u64,
        /// SHA-256 digest of the file bytes observed at grant time.
        /// The provider adapter recomputes the digest at read time and
        /// omits the part on mismatch. Defaults to all-zero for
        /// newly-constructed parts that have not yet been validated.
        #[serde(default)]
        content_digest: ContentDigest,
    },
}

/// A message in the conversation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "role", rename_all = "snake_case")]
pub enum Message {
    /// System-level instruction (identity, rules, tool guide).
    System {
        /// System prompt content.
        content: String,
        /// Stable prompt ranges suitable for provider-side caching.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        cache_markers: Vec<SystemCacheMarker>,
    },
    /// Workspace context (AGENTS.md, history summary, retrieved files).
    Context {
        /// Context content.
        content: String,
    },
    /// Message from the user.
    User {
        /// The user's message text.
        content: String,
    },
    /// Multimodal user message with ordered text and image parts (ADR-050).
    ///
    /// The existing `User { content: String }` variant is preserved for
    /// text-only backward compatibility. This variant is additive and requires
    /// a pre-1.0 minor release for exhaustive match migration.
    Multimodal {
        /// Ordered content parts — text and image interleaved in the order
        /// the user composed them.
        parts: Vec<ContentPart>,
    },
    /// Response from the assistant.
    Assistant {
        /// The assistant's response text.
        content: String,
        /// Tool calls requested by the assistant.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        tool_calls: Vec<ToolCall>,
        /// Provider-native reasoning blocks attached to this message.
        ///
        /// Request-history metadata only — never display content. See ADR-034.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reasoning: Option<AssistantReasoning>,
    },
    /// Result of a tool execution.
    Tool {
        /// The tool result.
        result: MessageToolResult,
    },
}

/// Reason the assistant stopped generating.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// Assistant finished its response.
    EndTurn,
    /// Assistant wants to call a tool.
    ToolUse,
    /// Reached the maximum token limit.
    MaxTokens,
}

/// Token usage statistics for a turn.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Usage {
    /// Tokens in the input prompt.
    pub input_tokens: u32,
    /// Tokens generated by the model.
    pub output_tokens: u32,
    /// Tokens read from cache.
    #[serde(default)]
    pub cache_read_tokens: u32,
    /// Tokens written to cache.
    #[serde(default)]
    pub cache_write_tokens: u32,
    /// Reasoning/thinking tokens — informational subset of `output_tokens`.
    #[serde(default)]
    pub reasoning_tokens: u32,
}

/// Events emitted during a turn for streaming.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum AgentEvent {
    /// Turn has started.
    TurnStart,
    /// A text delta was received from the provider.
    TextDelta {
        /// The text chunk.
        delta: String,
    },
    /// A transient thinking/reasoning delta was received from the provider.
    ///
    /// Thinking deltas are live UI preview data. They must not be persisted as normal
    /// conversation history or included in the final assistant text.
    ThinkingDelta {
        /// The thinking text chunk.
        delta: String,
    },
    /// Emitted once per provider response, before `TurnEnd`, when the response
    /// carried reasoning blocks. Durable replay payload; never display content.
    ReasoningComplete {
        /// Provider-native reasoning blocks in stream order.
        blocks: Vec<ReasoningBlock>,
    },
    /// Tool call detected: parameters still streaming.
    ToolCallStarted {
        /// Name of the tool being called.
        name: String,
    },
    /// A tool call was requested.
    ToolCall {
        /// The tool call details.
        call: ToolCall,
        /// The provenance of the tool being called.
        provenance: ToolProvenance,
        /// Fields to display in the TUI summary (from tool summary_fields()).
        summary_fields: Vec<String>,
    },
    /// A tool call completed.
    ToolResult {
        /// The tool result.
        result: MessageToolResult,
    },
    /// Turn has ended.
    TurnEnd {
        /// Why the turn ended.
        stop_reason: StopReason,
        /// Token usage for this turn.
        usage: Usage,
    },
    /// An error occurred.
    Error {
        /// Error message.
        message: String,
    },
}

#[cfg(test)]
#[allow(warnings)]
#[allow(warnings)]
#[allow(warnings)]
#[allow(warnings)]
mod tests {
    use super::*;

    #[test]
    fn message_roundtrip_user() {
        let msg = Message::User {
            content: "Hello, world!".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn message_roundtrip_assistant() {
        let msg = Message::Assistant {
            content: "I can help with that.".into(),
            tool_calls: vec![ToolCall {
                id: "call_1".into(),
                name: "read_file".into(),
                input: serde_json::json!({"path": "src/main.rs"}),
            }],
            reasoning: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn message_roundtrip_tool() {
        let msg = Message::Tool {
            result: MessageToolResult {
                tool_use_id: "call_1".into(),
                content: "fn main() {}".into(),
                is_error: false,
            },
        };
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn event_roundtrip() {
        let events = vec![
            AgentEvent::TurnStart,
            AgentEvent::TextDelta {
                delta: "Hello".into(),
            },
            AgentEvent::ToolCall {
                call: ToolCall {
                    id: "c1".into(),
                    name: "bash".into(),
                    input: serde_json::json!({"command": "ls"}),
                },
                provenance: ToolProvenance::Native,
                summary_fields: vec![],
            },
            AgentEvent::ToolResult {
                result: MessageToolResult {
                    tool_use_id: "c1".into(),
                    content: "file.rs".into(),
                    is_error: false,
                },
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: Usage {
                    input_tokens: 100,
                    output_tokens: 50,
                    cache_read_tokens: 80,
                    cache_write_tokens: 20,
                    reasoning_tokens: 0,
                },
            },
            AgentEvent::Error {
                message: "something failed".into(),
            },
        ];
        for event in events {
            let json = serde_json::to_string(&event).unwrap();
            let decoded: AgentEvent = serde_json::from_str(&json).unwrap();
            assert_eq!(event, decoded);
        }
    }

    #[test]
    fn extract_tool_calls_preserves_id_from_json_tool_block() {
        let text = r#"I'll run that for you.
```json-tool
{"id":"call_abc123","args":{"command":"ls"},"name":"bash"}
```
Done."#;
        let calls = extract_tool_calls_from_text(text);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].id, "call_abc123");
        assert_eq!(calls[0].name, "bash");
        assert_eq!(calls[0].input, serde_json::json!({"command": "ls"}));
    }

    #[test]
    fn extract_tool_calls_falls_back_to_synthetic_id_when_missing() {
        let text = r#"```json-tool
{"args":{"command":"ls"},"name":"bash"}
```"#;
        let calls = extract_tool_calls_from_text(text);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].id, "tc_0");
        assert_eq!(calls[0].name, "bash");
    }

    #[test]
    fn extract_tool_calls_falls_back_when_id_is_empty() {
        let text = r#"```json-tool
{"id":"","args":{"command":"ls"},"name":"bash"}
```"#;
        let calls = extract_tool_calls_from_text(text);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].id, "tc_0");
    }

    #[test]
    fn content_part_text_roundtrip() {
        let part = ContentPart::Text {
            text: "Hello, image!".into(),
        };
        let json = serde_json::to_string(&part).unwrap();
        let decoded: ContentPart = serde_json::from_str(&json).unwrap();
        assert_eq!(part, decoded);
    }

    #[test]
    fn content_part_image_roundtrip() {
        let part = ContentPart::Image {
            path: "/tmp/test.png".into(),
            mime: "image/png".into(),
            byte_count: 12345,
            content_digest: ContentDigest::from_raw([7u8; 32]),
        };
        let json = serde_json::to_string(&part).unwrap();
        let decoded: ContentPart = serde_json::from_str(&json).unwrap();
        assert_eq!(part, decoded);
    }

    #[test]
    fn message_multimodal_roundtrip() {
        let msg = Message::Multimodal {
            parts: vec![
                ContentPart::Text {
                    text: "What is in this image?".into(),
                },
                ContentPart::Image {
                    path: "/tmp/screenshot.png".into(),
                    mime: "image/png".into(),
                    byte_count: 67890,
                    content_digest: ContentDigest::from_raw([9u8; 32]),
                },
            ],
        };
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn message_user_still_works_after_multimodal_addition() {
        let msg = Message::User {
            content: "text only".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, decoded);
    }
}

pub fn extract_tool_calls_from_text(text: &str) -> Vec<ToolCall> {
    let mut calls = Vec::new();
    let mut remaining = text;

    while let Some(start) = remaining.find("```json-tool") {
        let inner_start = start + "```json-tool".len();
        let inner = remaining[inner_start..].trim_start();
        let end = inner.find("```").unwrap_or(inner.len());
        let content = inner[..end].trim();

        if let Ok(obj) = serde_json::from_str::<serde_json::Value>(content)
            && let (Some(name), Some(args)) = (obj["name"].as_str(), Some(obj["args"].clone()))
        {
            let id = obj["id"]
                .as_str()
                .filter(|s| !s.is_empty())
                .map(String::from)
                .unwrap_or_else(|| format!("tc_{}", calls.len()));
            calls.push(ToolCall {
                id,
                name: name.to_string(),
                input: args,
            });
        }

        remaining = &inner[end..];
        if end + 3 < remaining.len() {
            remaining = &remaining[3..];
        } else {
            break;
        }
    }

    calls
}

pub fn strip_tool_syntax(text: &str) -> String {
    let mut result = text.to_string();
    while let Some(start) = result.find("```json-tool") {
        let inner_start = start + "```json-tool".len();
        let inner = &result[inner_start..];
        let end = inner_start + inner.find("```").unwrap_or(inner.len()) + 3;
        result.replace_range(start..end, "");
    }
    result.trim().to_string()
}

pub fn project_displayable_reasoning(ar: &AssistantReasoning) -> Option<String> {
    let mut parts = Vec::new();
    for block in &ar.blocks {
        match block {
            ReasoningBlock::Thinking { text, .. } if !text.is_empty() => parts.push(text.clone()),
            ReasoningBlock::Plain { text } if !text.is_empty() => parts.push(text.clone()),
            _ => {}
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n"))
    }
}
