//! Shared models.dev wire-format evidence for the runtime and catalog generator.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WireProtocolHint {
    Chat,
    Responses,
    Anthropic,
}

pub(crate) fn infer_wire_protocol(
    npm: Option<&str>,
    shape: Option<&str>,
) -> Option<WireProtocolHint> {
    if let Some(shape) = shape {
        return match shape.trim() {
            "responses" => Some(WireProtocolHint::Responses),
            "completions" => Some(WireProtocolHint::Chat),
            _ => None,
        };
    }
    match npm.map(str::trim) {
        Some("@ai-sdk/openai") => Some(WireProtocolHint::Responses),
        Some("@ai-sdk/openai-compatible") => Some(WireProtocolHint::Chat),
        Some("@ai-sdk/anthropic") => Some(WireProtocolHint::Anthropic),
        _ => None,
    }
}
