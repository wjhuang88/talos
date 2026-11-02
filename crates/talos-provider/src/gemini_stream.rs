//! Semantic generateContent stream decoding; SSE byte framing is a separate boundary.
//! Source: https://ai.google.dev/api/generate-content (checked 2026-11-01).

use serde_json::Value;
use std::collections::HashSet;
use talos_core::message::{AgentEvent, StopReason, Usage};
use talos_core::provider::{ProviderError, ProviderResult};

pub(crate) enum GeminiOutput {
    Event(AgentEvent),
    FunctionCall {
        id: String,
        name: String,
        arguments: Value,
    },
}

/// No Debug: native model parts may contain confidential thought signatures.
pub(crate) struct GeminiStream {
    model_parts: Vec<Value>,
    part_bytes: usize,
    call_ids: HashSet<String>,
    response_key: String,
    stop: Option<StopReason>,
    usage: Usage,
    ended: bool,
}

fn invalid(message: &'static str) -> ProviderError {
    ProviderError::InvalidResponse(message.into())
}

impl GeminiStream {
    pub(crate) fn new() -> Self {
        Self {
            model_parts: Vec::new(),
            part_bytes: 0,
            call_ids: HashSet::new(),
            response_key: uuid::Uuid::new_v4().to_string(),
            stop: None,
            usage: Usage::default(),
            ended: false,
        }
    }

    /// Native ordered parts for later identity-scoped replay; never presentation content.
    pub(crate) fn model_parts(&self) -> &[Value] {
        &self.model_parts
    }

    pub(crate) fn apply(&mut self, chunk: &Value) -> ProviderResult<Vec<GeminiOutput>> {
        if self.ended {
            return Err(invalid("Gemini stream already ended"));
        }
        let result = self.apply_chunk(chunk);
        if result.is_err() {
            self.ended = true;
        }
        result
    }

    fn apply_chunk(&mut self, chunk: &Value) -> ProviderResult<Vec<GeminiOutput>> {
        if !chunk.is_object() || chunk.get("error").is_some() {
            return Err(invalid("invalid Gemini response"));
        }
        if chunk
            .get("promptFeedback")
            .and_then(|v| v.get("blockReason"))
            .and_then(Value::as_str)
            .is_some_and(|reason| reason != "BLOCK_REASON_UNSPECIFIED")
        {
            return Err(invalid("Gemini prompt blocked"));
        }
        if let Some(usage) = chunk.get("usageMetadata") {
            if !usage.is_object() {
                return Err(invalid("invalid Gemini usage"));
            }
            let count = |key: &str| -> ProviderResult<u32> {
                match usage.get(key) {
                    None => Ok(0),
                    Some(value) => value
                        .as_u64()
                        .and_then(|n| u32::try_from(n).ok())
                        .ok_or_else(|| invalid("invalid Gemini usage")),
                }
            };
            self.usage.input_tokens = count("promptTokenCount")?;
            self.usage.reasoning_tokens = count("thoughtsTokenCount")?;
            self.usage.output_tokens = count("candidatesTokenCount")?
                .checked_add(self.usage.reasoning_tokens)
                .ok_or_else(|| invalid("Gemini usage overflow"))?;
            self.usage.cache_read_tokens = count("cachedContentTokenCount")?;
        }
        let Some(candidates) = chunk.get("candidates") else {
            return Ok(Vec::new());
        };
        let candidates = candidates
            .as_array()
            .ok_or_else(|| invalid("invalid Gemini candidates"))?;
        if candidates.is_empty() {
            return Ok(Vec::new());
        }
        if candidates.len() != 1
            || candidates[0]
                .get("index")
                .is_some_and(|n| n.as_u64() != Some(0))
        {
            return Err(invalid("multiple Gemini candidates are unsupported"));
        }
        let candidate = &candidates[0];
        let mut output = Vec::new();
        if let Some(content) = candidate.get("content") {
            if self.stop.is_some() {
                return Err(invalid("Gemini content after terminal outcome"));
            }
            if content
                .get("role")
                .is_some_and(|role| role.as_str() != Some("model"))
            {
                return Err(invalid("invalid Gemini candidate role"));
            }
            let parts = content
                .get("parts")
                .and_then(Value::as_array)
                .ok_or_else(|| invalid("invalid Gemini candidate parts"))?;
            for part in parts {
                if !part.is_object() {
                    return Err(invalid("invalid Gemini content part"));
                }
                if part.get("thoughtSignature").is_some_and(|v| !v.is_string()) {
                    return Err(invalid("invalid Gemini thought signature"));
                }
                if part.get("text").is_some() && part.get("functionCall").is_some() {
                    return Err(invalid("ambiguous Gemini content part"));
                }
                let size = part.to_string().len();
                self.part_bytes = self
                    .part_bytes
                    .checked_add(size)
                    .filter(|n| *n <= 8 * 1024 * 1024)
                    .ok_or_else(|| invalid("Gemini response exceeds limit"))?;
                if let Some(text) = part.get("text") {
                    let delta = text
                        .as_str()
                        .ok_or_else(|| invalid("invalid Gemini text"))?
                        .to_owned();
                    let thought = match part.get("thought") {
                        None => false,
                        Some(value) => value
                            .as_bool()
                            .ok_or_else(|| invalid("invalid Gemini thought marker"))?,
                    };
                    output.push(GeminiOutput::Event(if thought {
                        AgentEvent::ThinkingDelta { delta }
                    } else {
                        AgentEvent::TextDelta { delta }
                    }));
                } else if let Some(call) = part.get("functionCall") {
                    let name = call
                        .get("name")
                        .and_then(Value::as_str)
                        .filter(|s| !s.is_empty())
                        .ok_or_else(|| invalid("invalid Gemini function name"))?
                        .to_owned();
                    let arguments = call
                        .get("args")
                        .filter(|v| v.is_object())
                        .cloned()
                        .ok_or_else(|| invalid("invalid Gemini function arguments"))?;
                    let id = match call.get("id") {
                        Some(id) => id
                            .as_str()
                            .filter(|s| !s.is_empty())
                            .map(str::to_owned)
                            .ok_or_else(|| invalid("invalid Gemini function id"))?,
                        None => format!("gemini_{}_{}", self.response_key, self.model_parts.len()),
                    };
                    if !self.call_ids.insert(id.clone()) {
                        return Err(invalid("duplicate Gemini function id"));
                    }
                    output.push(GeminiOutput::FunctionCall {
                        id,
                        name,
                        arguments,
                    });
                } else if part.get("thoughtSignature").is_none() {
                    return Err(invalid("unsupported Gemini content part"));
                }
                // Preserve complete original parts, including signatures, without flattening them.
                self.model_parts.push(part.clone());
            }
        }
        if let Some(reason) = candidate.get("finishReason") {
            if self.stop.is_some() {
                return Err(invalid("duplicate Gemini terminal outcome"));
            }
            self.stop = Some(match reason.as_str() {
                Some("STOP") if !self.call_ids.is_empty() => StopReason::ToolUse,
                Some("STOP") => StopReason::EndTurn,
                Some("MAX_TOKENS") if self.call_ids.is_empty() => StopReason::MaxTokens,
                Some("MAX_TOKENS") => return Err(invalid("Gemini tool response truncated")),
                _ => {
                    return Err(invalid(
                        "Gemini generation blocked or unsupported finish reason",
                    ));
                }
            });
        }
        Ok(output)
    }

    /// Wait until transport EOF so trailing usage metadata is included in TurnEnd.
    pub(crate) fn finish_eof(&mut self) -> ProviderResult<AgentEvent> {
        if self.ended {
            return Err(invalid("Gemini stream already ended"));
        }
        self.ended = true;
        let stop_reason = self
            .stop
            .clone()
            .ok_or_else(|| invalid("Gemini stream ended without terminal outcome"))?;
        Ok(AgentEvent::TurnEnd {
            stop_reason,
            usage: self.usage.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    proptest::proptest! {
        #[test]
        fn text_chunks_preserve_unicode_and_never_infer_terminal(
            chunks in proptest::collection::vec(".{0,128}", 0..16)
        ) {
            let mut stream = GeminiStream::new();
            let mut text = String::new();
            for chunk in &chunks {
                let events = stream.apply(&json!({"candidates":[{"content":{"parts":[{"text":chunk}]}}]})).expect("text");
                for event in events {
                    if let GeminiOutput::Event(AgentEvent::TextDelta { delta }) = event { text.push_str(&delta); }
                }
            }
            proptest::prop_assert_eq!(text, chunks.concat());
            proptest::prop_assert!(stream.finish_eof().is_err());
        }
    }

    #[test]
    fn text_thinking_signatures_and_trailing_usage_remain_separate() {
        let mut stream = GeminiStream::new();
        let parts = json!([{"text":"summary","thought":true,"thoughtSignature":"opaque-fixture"},{"text":"answer"}]);
        let output = stream.apply(&json!({"candidates":[{"content":{"role":"model","parts":parts},"finishReason":"STOP"}]})).expect("chunk");
        assert!(
            matches!(&output[0], GeminiOutput::Event(AgentEvent::ThinkingDelta{delta}) if delta == "summary")
        );
        assert!(
            matches!(&output[1], GeminiOutput::Event(AgentEvent::TextDelta{delta}) if delta == "answer")
        );
        assert_eq!(stream.model_parts(), parts.as_array().expect("parts"));
        stream.apply(&json!({"usageMetadata":{"promptTokenCount":10,"candidatesTokenCount":3,"thoughtsTokenCount":2}})).expect("usage");
        assert!(
            matches!(stream.finish_eof().expect("end"), AgentEvent::TurnEnd{stop_reason:StopReason::EndTurn,usage} if usage.output_tokens == 5 && usage.reasoning_tokens == 2)
        );
        assert!(stream.finish_eof().is_err());
    }

    #[test]
    fn actual_and_missing_call_ids_preserve_parallel_association() {
        let mut stream = GeminiStream::new();
        let calls = stream
            .apply(&json!({"candidates":[{"content":{"parts":[
            {"functionCall":{"id":"actual","name":"lookup","args":{"q":1}}},
            {"functionCall":{"name":"lookup","args":{"q":2}}}]},"finishReason":"STOP"}]}))
            .expect("calls");
        assert!(
            matches!(&calls[0], GeminiOutput::FunctionCall{id,name,arguments} if id == "actual" && name == "lookup" && arguments["q"] == 1)
        );
        assert!(
            matches!(&calls[1], GeminiOutput::FunctionCall{id,..} if id != "actual" && id.starts_with("gemini_"))
        );
        assert!(matches!(
            stream.finish_eof().expect("end"),
            AgentEvent::TurnEnd {
                stop_reason: StopReason::ToolUse,
                ..
            }
        ));
    }

    #[test]
    fn blocked_malformed_duplicate_and_unterminated_streams_fail_without_payloads() {
        for chunk in [
            json!({"promptFeedback":{"blockReason":"SAFETY"}}),
            json!({"error":{"message":"fixture-private"}}),
            json!({"candidates":[{"finishReason":"SAFETY"}]}),
            json!({"candidates":[{"content":{"parts":[{"functionCall":{"name":"tool","args":"bad"}}]}}]}),
            json!({"candidates":[{"content":{"parts":[{"functionCall":{"id":"same","name":"tool","args":{}}},{"functionCall":{"id":"same","name":"tool","args":{}}}]}}]}),
        ] {
            let error = GeminiStream::new().apply(&chunk).err().expect("invalid");
            assert!(!error.to_string().contains("fixture-private"));
        }
        assert!(GeminiStream::new().finish_eof().is_err());
    }
}
