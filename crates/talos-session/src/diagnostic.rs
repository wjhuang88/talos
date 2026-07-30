use serde::{Deserialize, Serialize};
use talos_core::message::{AgentEvent, StopReason};

pub(crate) const TERMINAL_DIAGNOSTIC_PREFIX: &str = "__TALOS_PROVIDER_TERMINAL_DIAGNOSTIC__:";
const MAX_IDENTITY_CHARS: usize = 128;
const MAX_REASON_CHARS: usize = 160;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderTerminalOutcome {
    Completed,
    ToolUse,
    Truncated,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderTerminalSource {
    Explicit,
    MissingTerminal,
    UnsupportedReason,
    DecodeError,
    TransportError,
    Timeout,
    ProviderError,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderTerminalDiagnostic {
    pub version: u8,
    pub turn_id: String,
    pub response_ordinal: u32,
    pub outcome: ProviderTerminalOutcome,
    pub source: ProviderTerminalSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

impl ProviderTerminalDiagnostic {
    pub fn from_agent_event(
        turn_id: &str,
        response_ordinal: u32,
        event: &AgentEvent,
        provider: Option<&str>,
        model: Option<&str>,
    ) -> Option<Self> {
        let (outcome, source, reason) = match event {
            AgentEvent::TurnEnd { stop_reason, .. } => match stop_reason {
                StopReason::EndTurn => (
                    ProviderTerminalOutcome::Completed,
                    ProviderTerminalSource::Explicit,
                    None,
                ),
                StopReason::ToolUse => (
                    ProviderTerminalOutcome::ToolUse,
                    ProviderTerminalSource::Explicit,
                    None,
                ),
                StopReason::MaxTokens => (
                    ProviderTerminalOutcome::Truncated,
                    ProviderTerminalSource::Explicit,
                    Some("max_tokens".to_string()),
                ),
            },
            AgentEvent::Error { message } => {
                let (source, reason) = classify_error(message);
                (ProviderTerminalOutcome::Error, source, Some(reason))
            }
            _ => return None,
        };
        Some(Self {
            version: 1,
            turn_id: bound(turn_id, MAX_IDENTITY_CHARS),
            response_ordinal: response_ordinal.max(1),
            outcome,
            source,
            reason,
            provider: provider.map(|value| bound(value, MAX_IDENTITY_CHARS)),
            model: model.map(|value| bound(value, MAX_IDENTITY_CHARS)),
        })
    }
}

fn classify_error(message: &str) -> (ProviderTerminalSource, String) {
    if message.contains("closed without explicit terminal signal") {
        return (
            ProviderTerminalSource::MissingTerminal,
            "missing_explicit_terminal".into(),
        );
    }
    for prefix in [
        "unsupported provider finish_reason:",
        "unsupported provider stop_reason:",
    ] {
        if let Some(reason) = message.strip_prefix(prefix) {
            return (
                ProviderTerminalSource::UnsupportedReason,
                bound(reason.trim(), MAX_REASON_CHARS),
            );
        }
    }
    if message.contains("decode error") || message.contains("invalid UTF-8") {
        return (ProviderTerminalSource::DecodeError, "invalid_utf8".into());
    }
    if message.contains("transport read error") {
        return (ProviderTerminalSource::TransportError, "read_error".into());
    }
    if message.contains("first-packet timeout") {
        return (
            ProviderTerminalSource::Timeout,
            "first_packet_timeout".into(),
        );
    }
    if message.contains("stream-idle timeout") {
        return (
            ProviderTerminalSource::Timeout,
            "stream_idle_timeout".into(),
        );
    }
    (
        ProviderTerminalSource::ProviderError,
        "provider_error".into(),
    )
}

fn bound(value: &str, max_chars: usize) -> String {
    let bounded = value
        .chars()
        .filter(|character| !character.is_control())
        .take(max_chars)
        .collect::<String>();
    if bounded.is_empty() {
        "unknown".into()
    } else {
        bounded
    }
}

pub(crate) fn encode_terminal_diagnostic(
    diagnostic: &ProviderTerminalDiagnostic,
) -> Result<String, serde_json::Error> {
    serde_json::to_string(diagnostic)
        .map(|encoded| format!("{TERMINAL_DIAGNOSTIC_PREFIX}{encoded}"))
}

pub(crate) fn decode_terminal_diagnostic(content: &str) -> Option<ProviderTerminalDiagnostic> {
    content
        .strip_prefix(TERMINAL_DIAGNOSTIC_PREFIX)
        .and_then(|encoded| serde_json::from_str(encoded).ok())
}

pub(crate) fn is_terminal_diagnostic_content(content: &str) -> bool {
    content.starts_with(TERMINAL_DIAGNOSTIC_PREFIX)
}
