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

// Catalog routing is HTTP-only. A whole-root environment placeholder is retained at build time
// (e.g. Neon); it must expand to a valid HTTP endpoint before any request is constructed.
pub(crate) fn is_http_model_endpoint(endpoint: &str) -> bool {
    let endpoint = endpoint.trim();
    // Validate template syntax without consulting the build machine's environment. Substitute a
    // fixed HTTPS origin only for parsing; runtime validates the actual expanded destination.
    let expanded;
    let candidate = if let Some(template) = endpoint.strip_prefix("${") {
        let Some((name, suffix)) = template.split_once('}') else {
            return false;
        };
        if name.is_empty()
            || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
            || !(suffix.is_empty() || suffix.starts_with('/'))
            || suffix.contains("${")
        {
            return false;
        }
        expanded = format!("https://catalog-template.invalid{suffix}");
        expanded.as_str()
    } else {
        endpoint
    };
    is_http_endpoint(candidate)
}

pub(crate) fn is_http_endpoint(endpoint: &str) -> bool {
    if endpoint
        .bytes()
        .any(|byte| byte.is_ascii_whitespace() || byte.is_ascii_control())
    {
        return false;
    }
    let Ok(url) = url::Url::parse(endpoint) else {
        return false;
    };
    if !url.username().is_empty()
        || url.password().is_some()
        || url.host().is_none()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return false;
    }
    match url.scheme() {
        "https" => true,
        "http" => match url.host() {
            Some(url::Host::Domain("localhost")) => true,
            Some(url::Host::Ipv4(address)) => address.is_loopback(),
            Some(url::Host::Ipv6(address)) => address.is_loopback(),
            _ => false,
        },
        _ => false,
    }
}
