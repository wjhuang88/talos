//! Session-private signed history, separate from serializable conversation messages.

use super::{ProviderError, ProviderResult};
use crate::message::{AssistantReasoning, Message, ReasoningBlock};

const NAMESPACE: &str = "talos:private-replay:";
const PREFIX: &str = "talos:private-replay:v1:";
/// Maximum serialized payload reserved by one private history attachment.
pub const MAX_TRANSIENT_REPLAY_BYTES: usize = 8 * 1024 * 1024;

fn invalid() -> ProviderError {
    ProviderError::InvalidResponse("private replay context is invalid or unavailable".into())
}

/// An immutable signed message and its exact public projection.
///
/// No serialization or Clone implementation is provided. Session owners may retain an Arc
/// across live user turns; durable history contains only the generated content-free marker.
/// The trusted adapter is responsible for removing private fields from the public projection.
pub struct TransientReplay {
    public: Message,
    private: Message,
    bytes: u32,
    private_values: Vec<String>,
}

impl std::fmt::Debug for TransientReplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TransientReplay([redacted])")
    }
}

impl TransientReplay {
    /// Binds two assistant views; the public view must not already contain reasoning.
    /// The private view supplies the exact provider/model identity and signed reasoning.
    pub fn new(mut public: Message, private: Message) -> ProviderResult<Self> {
        let mut bytes = serde_json::to_vec(&private)
            .map_err(|_| invalid())?
            .len()
            .checked_add(serde_json::to_vec(&public).map_err(|_| invalid())?.len())
            .filter(|bytes| *bytes > 0 && *bytes <= MAX_TRANSIENT_REPLAY_BYTES)
            .ok_or_else(invalid)? as u32;
        let (
            Message::Assistant {
                tool_calls: public_calls,
                reasoning: public_reasoning,
                ..
            },
            Message::Assistant {
                tool_calls: private_calls,
                reasoning: Some(reasoning),
                ..
            },
        ) = (&mut public, &private)
        else {
            return Err(invalid());
        };
        if public_reasoning.is_some()
            || reasoning.provider.is_empty()
            || reasoning.model.is_empty()
            || reasoning.blocks.is_empty()
            || public_calls.len() != private_calls.len()
            || public_calls.iter().zip(private_calls).any(|(a, b)| {
                a.id.is_empty() || a.name.is_empty() || a.id != b.id || a.name != b.name
            })
        {
            return Err(invalid());
        }
        let mut private_values = Vec::new();
        for (safe, raw) in public_calls.iter().zip(private_calls) {
            if safe.input == serde_json::json!({"protocolVersion":2}) && safe.input != raw.input {
                if let Some(fields) = raw.input.as_object() {
                    for (name, value) in fields {
                        if name == "operation"
                            && value
                                .as_str()
                                .and_then(crate::tool::PreparedInvocationError::browser_operation)
                                .is_some()
                        {
                            continue;
                        }
                        if let Some(value) = value.as_str().filter(|value| !value.is_empty()) {
                            if !private_values.iter().any(|stored| stored == value) {
                                private_values.push(value.to_owned());
                            }
                        }
                    }
                }
            }
        }
        let value_bytes = private_values.iter().map(String::len).sum::<usize>();
        if private_values.len() > 256 || value_bytes > 64 * 1024 {
            return Err(invalid());
        }
        bytes = bytes
            .checked_add(value_bytes as u32)
            .filter(|bytes| *bytes as usize <= MAX_TRANSIENT_REPLAY_BYTES)
            .ok_or_else(invalid)?;
        *public_reasoning = Some(AssistantReasoning {
            provider: reasoning.provider.clone(),
            model: reasoning.model.clone(),
            blocks: vec![ReasoningBlock::Redacted {
                data: format!("{PREFIX}{}:{bytes}", uuid::Uuid::new_v4()),
            }],
        });
        Ok(Self {
            public,
            private,
            bytes,
            private_values,
        })
    }

    /// Rebinds an Agent-owned privacy projection after the original full view matched.
    /// This is not an authorization path for hook edits. IDs/names and selected identity
    /// remain bound; the private signed message is never exposed to the projection code.
    pub fn with_public_projection(&self, mut public: Message) -> ProviderResult<Self> {
        let (
            Message::Assistant { reasoning, .. },
            Message::Assistant {
                reasoning: expected,
                ..
            },
        ) = (&mut public, &self.public)
        else {
            return Err(invalid());
        };
        if reasoning != expected {
            return Err(invalid());
        }
        *reasoning = None;
        let mut rebound = Self::new(public, self.private.clone())?;
        // Privacy projection of an ordinary call must not redefine browser-private metadata.
        rebound.private_values = self.private_values.clone();
        Ok(rebound)
    }

    /// Candidate argument projection with marker/reserve. Generated text still requires
    /// the trusted Agent privacy filter before this view can become durable history.
    pub fn projected_message(&self) -> &Message {
        &self.public
    }

    /// Retains the trusted Agent's bounded privacy tokens for later live user turns.
    /// This includes tool-owned private references, not only original browser arguments.
    pub fn with_private_values(
        mut self,
        values: impl IntoIterator<Item = String>,
    ) -> ProviderResult<Self> {
        for value in values {
            if !value.is_empty() && !self.private_values.contains(&value) {
                self.private_values.push(value);
            }
            if self.private_values.len() > 256
                || self.private_values.iter().map(String::len).sum::<usize>() > 64 * 1024
            {
                return Err(invalid());
            }
        }
        Ok(self)
    }

    /// Declared browser-private scalar strings for the trusted Agent observer filter.
    /// Values are sensitive: never serialize, display or log this iterator.
    pub fn private_values(&self) -> impl Iterator<Item = &str> {
        self.private_values.iter().map(String::as_str)
    }

    /// Full serialized private/public byte reserve, without revealing its payload.
    pub fn byte_count(&self) -> u32 {
        self.bytes
    }

    /// Verifies original browser invocation evidence against the same private signed call.
    /// No payload is returned to the caller or observer path.
    pub fn matches_original_call(
        &self,
        id: &str,
        name: &str,
        arguments: &crate::tool::BrowserRawArguments,
    ) -> bool {
        let Message::Assistant { tool_calls, .. } = &self.private else {
            return false;
        };
        tool_calls.iter().any(|call| {
            call.id == id
                && call.name == name
                && call.input.as_object().is_some_and(|fields| {
                    fields.len() == arguments.fields().len()
                        && arguments
                            .fields()
                            .iter()
                            .all(|(key, value)| fields.get(key) == Some(value))
                })
        })
    }

    /// Exact whole-message binding, including marker, text, IDs, names and inputs.
    pub fn matches(&self, message: &Message) -> bool {
        &self.public == message
    }

    /// Restores only after exact public-view and selected identity validation.
    /// Call inside the provider request encoder, never a hook, preview or persistence path.
    pub fn restore(
        &self,
        message: &Message,
        provider: &str,
        model: &str,
    ) -> ProviderResult<Message> {
        if !self.matches(message) {
            return Err(invalid());
        }
        let Message::Assistant {
            reasoning: Some(reasoning),
            ..
        } = message
        else {
            return Err(invalid());
        };
        if reasoning.provider != provider || reasoning.model != model {
            return Err(invalid());
        }
        Ok(self.private.clone())
    }

    /// Parses only the reserved reasoning-block namespace, not ordinary message text.
    /// Malformed or mixed reserved markers are errors, never ordinary replay payloads.
    pub fn marker(message: &Message) -> ProviderResult<Option<(&str, u32)>> {
        let Message::Assistant {
            reasoning: Some(reasoning),
            ..
        } = message
        else {
            return Ok(None);
        };
        let markers: Vec<_> = reasoning
            .blocks
            .iter()
            .filter_map(|block| match block {
                ReasoningBlock::Redacted { data } if data.starts_with(NAMESPACE) => {
                    Some(data.as_str())
                }
                _ => None,
            })
            .collect();
        if markers.is_empty() {
            return Ok(None);
        }
        if markers.len() != 1 || reasoning.blocks.len() != 1 {
            return Err(invalid());
        }
        let data = markers[0];
        let (id, bytes) = data
            .strip_prefix(PREFIX)
            .and_then(|suffix| suffix.split_once(':'))
            .ok_or_else(invalid)?;
        uuid::Uuid::parse_str(id).map_err(|_| invalid())?;
        let bytes: u32 = bytes.parse().map_err(|_| invalid())?;
        if bytes == 0 || bytes as usize > MAX_TRANSIENT_REPLAY_BYTES {
            return Err(invalid());
        }
        Ok(Some((data, bytes)))
    }

    /// Conservatively reserves one token per private byte; malformed markers saturate.
    /// Existing request-level safety margins and output reserves still apply.
    pub fn reserved_tokens(message: &Message) -> Option<u32> {
        match Self::marker(message) {
            Ok(marker) => marker.map(|(_, bytes)| bytes),
            Err(_) => Some(u32::MAX),
        }
    }

    /// Resolves exact session-private attachments inside an admitted provider encoder.
    /// Extra, missing or repeated attachments/markers are refused before HTTP.
    pub fn restore_history(
        messages: &[Message],
        provider: &str,
        model: &str,
        replay: &[std::sync::Arc<Self>],
    ) -> ProviderResult<Vec<Message>> {
        let mut used = std::collections::HashSet::new();
        let mut restored = Vec::with_capacity(messages.len());
        for message in messages {
            if let Some((marker, _)) = Self::marker(message)? {
                if !used.insert(marker) {
                    return Err(invalid());
                }
                let matches = replay
                    .iter()
                    .filter(|item| item.matches(message))
                    .collect::<Vec<_>>();
                if matches.len() != 1 {
                    return Err(invalid());
                }
                restored.push(matches[0].restore(message, provider, model)?);
            } else {
                restored.push(message.clone());
            }
        }
        if used.len() != replay.len() {
            return Err(invalid());
        }
        Ok(restored)
    }

    /// Protected messages must survive hook projection exactly once, before any pruning.
    pub fn validate_hook_projection(before: &[Message], after: &[Message]) -> ProviderResult<()> {
        for message in before {
            if Self::marker(message)?.is_some()
                && after
                    .iter()
                    .filter(|candidate| *candidate == message)
                    .count()
                    != 1
            {
                return Err(invalid());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::ToolCall;
    fn replay() -> TransientReplay {
        let call = ToolCall {
            id: "b".into(),
            name: "browser".into(),
            input: serde_json::json!({"protocolVersion":2}),
        };
        let mut private_call = call.clone();
        private_call.input = serde_json::json!({"text":"private-sentinel"});
        TransientReplay::new(
            Message::Assistant {
                content: "safe".into(),
                tool_calls: vec![call],
                reasoning: None,
            },
            Message::Assistant {
                content: "safe".into(),
                tool_calls: vec![private_call],
                reasoning: Some(AssistantReasoning {
                    provider: "p".into(),
                    model: "m".into(),
                    blocks: vec![ReasoningBlock::Redacted {
                        data: "private-signature".into(),
                    }],
                }),
            },
        )
        .expect("bounded replay")
    }
    #[test]
    fn safe_marker_roundtrip_never_contains_private_payload() {
        let replay = replay();
        let public = replay.projected_message();
        let json = serde_json::to_string(public).expect("public history");
        assert!(!json.contains("private-sentinel"));
        assert!(!json.contains("private-signature"));
        assert!(!format!("{replay:?}").contains("sentinel"));
        let restored: Message = serde_json::from_str(&json).expect("roundtrip");
        assert!(replay.matches(&restored));
        assert_eq!(
            TransientReplay::reserved_tokens(&restored),
            Some(replay.byte_count())
        );
        let raw = replay
            .restore(&restored, "p", "m")
            .expect("request-only restore");
        assert!(
            serde_json::to_string(&raw)
                .expect("raw")
                .contains("private-sentinel")
        );
        assert!(replay.restore(&restored, "other", "m").is_err());
        let mut edited = restored;
        if let Message::Assistant { tool_calls, .. } = &mut edited {
            tool_calls[0].id = "other".into();
        }
        assert!(replay.restore(&edited, "p", "m").is_err());
    }
    #[test]
    fn hook_stripping_editing_and_duplication_are_rejected() {
        let replay = replay();
        let public = replay.projected_message().clone();
        assert!(
            TransientReplay::validate_hook_projection(
                std::slice::from_ref(&public),
                std::slice::from_ref(&public)
            )
            .is_ok()
        );
        assert!(
            TransientReplay::validate_hook_projection(std::slice::from_ref(&public), &[]).is_err()
        );
        assert!(
            TransientReplay::validate_hook_projection(
                std::slice::from_ref(&public),
                &[public.clone(), public.clone()]
            )
            .is_err()
        );
        let mut stripped = public.clone();
        if let Message::Assistant { reasoning, .. } = &mut stripped {
            *reasoning = None;
        }
        assert!(TransientReplay::validate_hook_projection(&[public], &[stripped]).is_err());
    }
    #[test]
    fn malformed_reserved_markers_fail_and_user_text_is_not_authority() {
        for data in [
            "talos:private-replay:v9:bad",
            "talos:private-replay:v1:no-uuid:1",
            "talos:private-replay:v1:00000000-0000-0000-0000-000000000000:0",
        ] {
            let message = Message::Assistant {
                content: String::new(),
                tool_calls: vec![],
                reasoning: Some(AssistantReasoning {
                    provider: "p".into(),
                    model: "m".into(),
                    blocks: vec![ReasoningBlock::Redacted { data: data.into() }],
                }),
            };
            assert!(TransientReplay::marker(&message).is_err());
            assert_eq!(TransientReplay::reserved_tokens(&message), Some(u32::MAX));
            assert!(
                TransientReplay::marker(&Message::User {
                    content: data.into()
                })
                .expect("user text")
                .is_none()
            );
        }
    }
    #[test]
    fn unknown_operation_values_remain_private_until_semantic_admission() {
        for operation in ["fill", "private-sentinel"] {
            let public_call = ToolCall {
                id: "b".into(),
                name: "browser".into(),
                input: serde_json::json!({"protocolVersion":2}),
            };
            let raw_call = ToolCall {
                input: serde_json::json!({"operation":operation}),
                ..public_call.clone()
            };
            let replay = TransientReplay::new(
                Message::Assistant {
                    content: "safe".into(),
                    tool_calls: vec![public_call],
                    reasoning: None,
                },
                Message::Assistant {
                    content: "safe".into(),
                    tool_calls: vec![raw_call],
                    reasoning: Some(AssistantReasoning {
                        provider: "p".into(),
                        model: "m".into(),
                        blocks: vec![ReasoningBlock::Redacted {
                            data: "signed".into(),
                        }],
                    }),
                },
            )
            .expect("replay");
            assert_eq!(
                replay.private_values().any(|value| value == operation),
                operation != "fill"
            );
        }
    }
}
