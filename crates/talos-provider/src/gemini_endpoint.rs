//! Official Gemini destinations. Validate resource components before credential discovery.
//!
//! References checked 2026-11-01: https://ai.google.dev/api/generate-content and
//! https://docs.cloud.google.com/vertex-ai/generative-ai/docs/model-reference/inference.
//! Custom gateways require a separate explicit credential profile; ADC never follows a URL.

fn component(value: &str, model: bool) -> Result<(), &'static str> {
    if value.is_empty()
        || value.len() > 256
        || value == "."
        || value == ".."
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || (model && matches!(b, b'.' | b'_')))
    {
        return Err("invalid Gemini resource component");
    }
    Ok(())
}

fn operation(stream: bool) -> &'static str {
    if stream {
        "streamGenerateContent?alt=sse"
    } else {
        "generateContent"
    }
}

/// Construct the AI Studio URL without allowing a model name to replace its origin or path.
pub(crate) fn ai_studio(model: &str, stream: bool) -> Result<String, &'static str> {
    let model = model.strip_prefix("models/").unwrap_or(model);
    component(model, true)?;
    Ok(format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{model}:{}",
        operation(stream)
    ))
}

/// Construct standard Vertex Google-publisher URLs before admitting ADC credentials.
pub(crate) fn vertex(
    project: &str,
    location: &str,
    model: &str,
    stream: bool,
) -> Result<String, &'static str> {
    component(project, false)?;
    component(location, false)?;
    component(model, true)?;
    // Location is also a DNS label. Reject labels that cannot name a regional Google host.
    if location.len() > 52
        || location.starts_with('-')
        || location.ends_with('-')
        || location.bytes().any(|b| b.is_ascii_uppercase())
    {
        return Err("invalid Vertex location");
    }
    let host = if location == "global" {
        "aiplatform.googleapis.com".to_owned()
    } else {
        format!("{location}-aiplatform.googleapis.com")
    };
    Ok(format!(
        "https://{host}/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:{}",
        operation(stream)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn official_destinations_and_stream_query_are_exact() {
        assert_eq!(
            ai_studio("models/gemini-2.5-flash", false).expect("AI Studio"),
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent"
        );
        assert_eq!(
            ai_studio("gemini-2.5-flash", true).expect("SSE"),
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:streamGenerateContent?alt=sse"
        );
        assert_eq!(
            vertex("project-123", "us-central1", "gemini-2.5-flash", true).expect("regional"),
            "https://us-central1-aiplatform.googleapis.com/v1/projects/project-123/locations/us-central1/publishers/google/models/gemini-2.5-flash:streamGenerateContent?alt=sse"
        );
        assert_eq!(
            vertex("123456", "global", "gemini-2.5-flash", false).expect("global"),
            "https://aiplatform.googleapis.com/v1/projects/123456/locations/global/publishers/google/models/gemini-2.5-flash:generateContent"
        );
    }

    #[test]
    fn components_cannot_inject_paths_queries_hosts_or_credentials() {
        for bad in [
            "",
            ".",
            "..",
            "../other",
            "a/b",
            "a%2fb",
            "a?key=secret",
            "a#frag",
            "https://evil.invalid",
            "user@host",
            "a\\b",
            "a\r\nheader",
            " a",
            "非ascii",
        ] {
            assert!(ai_studio(bad, true).is_err(), "model: {bad:?}");
            assert!(
                vertex(bad, "global", "model", true).is_err(),
                "project: {bad:?}"
            );
            assert!(
                vertex("project", bad, "model", true).is_err(),
                "location: {bad:?}"
            );
            assert!(
                vertex("project", "global", bad, true).is_err(),
                "Vertex model: {bad:?}"
            );
        }
        for bad in ["-us", "us-", "US", &"a".repeat(64)] {
            assert!(vertex("project", bad, "model", true).is_err());
        }
        assert!(ai_studio(&"a".repeat(257), true).is_err());
    }
}
