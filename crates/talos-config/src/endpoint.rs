use crate::ProviderProtocol;

/// Provider endpoint normalized for Talos provider adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedProviderEndpoint {
    /// Protocol adapter implied by the endpoint.
    pub protocol: ProviderProtocol,
    /// Base URL in the shape expected by that protocol adapter.
    pub base_url: String,
}

/// Normalize catalog or user-entered provider endpoints.
///
/// OpenAI-compatible providers expect a gateway root because the OpenAI adapter
/// appends `/chat/completions`. Anthropic-compatible providers expect the full
/// `/messages` endpoint.
#[must_use]
pub fn normalize_provider_endpoint(url: &str) -> NormalizedProviderEndpoint {
    let trimmed = url.trim().trim_end_matches('/');
    let lower = trimmed.to_ascii_lowercase();

    if lower.ends_with("/anthropic/v1") || lower.ends_with("/v1") && lower.contains("/anthropic/") {
        return NormalizedProviderEndpoint {
            protocol: ProviderProtocol::AnthropicMessages,
            base_url: format!("{trimmed}/messages"),
        };
    }

    if lower.ends_with("/messages") && lower.contains("/anthropic/") {
        return NormalizedProviderEndpoint {
            protocol: ProviderProtocol::AnthropicMessages,
            base_url: trimmed.to_string(),
        };
    }

    let base_url = lower
        .strip_suffix("/chat/completions")
        .map(|_| &trimmed[..trimmed.len() - "/chat/completions".len()])
        .unwrap_or(trimmed)
        .to_string();

    NormalizedProviderEndpoint {
        protocol: ProviderProtocol::OpenAIChat,
        base_url,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_anthropic_root_to_messages_endpoint() {
        let endpoint = normalize_provider_endpoint("https://api.minimax.io/anthropic/v1");

        assert_eq!(endpoint.protocol, ProviderProtocol::AnthropicMessages);
        assert_eq!(
            endpoint.base_url,
            "https://api.minimax.io/anthropic/v1/messages"
        );
    }

    #[test]
    fn preserves_anthropic_messages_endpoint() {
        let endpoint = normalize_provider_endpoint("https://api.minimax.io/anthropic/v1/messages");

        assert_eq!(endpoint.protocol, ProviderProtocol::AnthropicMessages);
        assert_eq!(
            endpoint.base_url,
            "https://api.minimax.io/anthropic/v1/messages"
        );
    }

    #[test]
    fn strips_openai_chat_completions_endpoint_to_root() {
        let endpoint = normalize_provider_endpoint("https://example.test/v1/chat/completions");

        assert_eq!(endpoint.protocol, ProviderProtocol::OpenAIChat);
        assert_eq!(endpoint.base_url, "https://example.test/v1");
    }

    #[test]
    fn preserves_openai_gateway_root() {
        let endpoint = normalize_provider_endpoint("https://example.test/v1/");

        assert_eq!(endpoint.protocol, ProviderProtocol::OpenAIChat);
        assert_eq!(endpoint.base_url, "https://example.test/v1");
    }
}
