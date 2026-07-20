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

/// Validates a custom provider name (MODEL-008-A/I147).
///
/// A canonical slug: starts with a lowercase ASCII letter or digit, then
/// continues with lowercase letters, digits, or `-`. Length 1–64.
pub fn validate_provider_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Provider name cannot be empty.".to_string());
    }
    if name.len() > 64 {
        return Err("Provider name cannot exceed 64 characters.".to_string());
    }
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return Err("Provider name cannot be empty.".to_string());
    };
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return Err("Provider name must start with a lowercase letter or digit.".to_string());
    }
    for ch in chars {
        if !ch.is_ascii_lowercase() && !ch.is_ascii_digit() && ch != '-' {
            return Err(
                "Provider name may only contain lowercase letters, digits, and hyphens."
                    .to_string(),
            );
        }
    }
    Ok(())
}

/// Validates a custom provider protocol string against the closed set
/// (MODEL-008-A/I147). Returns the typed `ProviderProtocol` on success.
pub fn validate_provider_protocol(protocol: &str) -> Result<ProviderProtocol, String> {
    match protocol {
        "openai-chat" => Ok(ProviderProtocol::OpenAIChat),
        "anthropic-messages" => Ok(ProviderProtocol::AnthropicMessages),
        other => Err(format!(
            "Unknown protocol '{other}'. Allowed: openai-chat, anthropic-messages."
        )),
    }
}

/// Validates a custom provider base URL (MODEL-008-A/I147).
///
/// HTTPS is always allowed. HTTP is allowed only for loopback addresses
/// (`127.0.0.1`, `::1`, `localhost`). Returns the normalized endpoint.
pub fn validate_provider_base_url(url: &str) -> Result<NormalizedProviderEndpoint, String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err("Base URL cannot be empty.".to_string());
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("https://") {
        return Ok(normalize_provider_endpoint(trimmed));
    }
    if lower.starts_with("http://") {
        let after_scheme = lower.strip_prefix("http://").unwrap_or("");
        let host_part = after_scheme.split('/').next().unwrap_or(after_scheme);
        let host = if let Some(rest) = host_part.strip_prefix('[') {
            rest.split(']').next().unwrap_or("")
        } else {
            host_part.split(':').next().unwrap_or(host_part)
        };
        if host == "127.0.0.1" || host == "::1" || host == "localhost" {
            return Ok(normalize_provider_endpoint(trimmed));
        }
        return Err(
            "HTTP is only allowed for loopback addresses (127.0.0.1, ::1, localhost).".to_string(),
        );
    }
    Err("Base URL must start with https:// (or http:// for loopback).".to_string())
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

    #[test]
    fn validate_provider_name_accepts_valid_slugs() {
        assert!(validate_provider_name("a").is_ok());
        assert!(validate_provider_name("my-provider").is_ok());
        assert!(validate_provider_name("openai").is_ok());
        assert!(validate_provider_name("123abc").is_ok());
        assert!(validate_provider_name(&"a".repeat(64)).is_ok());
    }

    #[test]
    fn validate_provider_name_rejects_empty() {
        assert!(validate_provider_name("").is_err());
    }

    #[test]
    fn validate_provider_name_rejects_too_long() {
        assert!(validate_provider_name(&"a".repeat(65)).is_err());
    }

    #[test]
    fn validate_provider_name_rejects_uppercase() {
        assert!(validate_provider_name("MyProvider").is_err());
    }

    #[test]
    fn validate_provider_name_rejects_special_chars() {
        assert!(validate_provider_name("my_provider").is_err());
        assert!(validate_provider_name("my.provider").is_err());
        assert!(validate_provider_name("my provider").is_err());
        assert!(validate_provider_name("-myprovider").is_err());
    }

    #[test]
    fn validate_provider_protocol_accepts_closed_set() {
        assert_eq!(
            validate_provider_protocol("openai-chat").unwrap(),
            ProviderProtocol::OpenAIChat
        );
        assert_eq!(
            validate_provider_protocol("anthropic-messages").unwrap(),
            ProviderProtocol::AnthropicMessages
        );
    }

    #[test]
    fn validate_provider_protocol_rejects_freeform() {
        assert!(validate_provider_protocol("custom").is_err());
        assert!(validate_provider_protocol("").is_err());
        assert!(validate_provider_protocol("openai").is_err());
    }

    #[test]
    fn validate_provider_base_url_accepts_https() {
        let result = validate_provider_base_url("https://api.example.com/v1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().protocol, ProviderProtocol::OpenAIChat);
    }

    #[test]
    fn validate_provider_base_url_accepts_loopback_http() {
        assert!(validate_provider_base_url("http://127.0.0.1:8080/v1").is_ok());
        assert!(validate_provider_base_url("http://localhost:3000").is_ok());
        assert!(validate_provider_base_url("http://[::1]:8080").is_ok());
    }

    #[test]
    fn validate_provider_base_url_rejects_non_loopback_http() {
        assert!(validate_provider_base_url("http://api.example.com/v1").is_err());
        assert!(validate_provider_base_url("http://192.168.1.1/v1").is_err());
    }

    #[test]
    fn validate_provider_base_url_rejects_empty() {
        assert!(validate_provider_base_url("").is_err());
        assert!(validate_provider_base_url("   ").is_err());
    }

    #[test]
    fn validate_provider_base_url_rejects_non_url() {
        assert!(validate_provider_base_url("not-a-url").is_err());
        assert!(validate_provider_base_url("ftp://example.com").is_err());
    }
}
