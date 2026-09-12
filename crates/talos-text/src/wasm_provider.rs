//! Bounded WASM language-provider boundary.

use std::time::Duration;

/// Resource limits applied before a guest provider is admitted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WasmProviderLimits {
    /// Maximum guest fuel.
    pub fuel: u64,
    /// Maximum wall-clock execution time.
    pub timeout: Duration,
    /// Maximum source bytes presented to the guest.
    pub max_source_bytes: usize,
}

/// Validates a provider source request before it crosses the WASM boundary.
pub fn validate_source(source: &str, limits: WasmProviderLimits) -> Result<(), &'static str> {
    if source.len() > limits.max_source_bytes {
        return Err("source exceeds provider limit");
    }
    Ok(())
}

/// UI-neutral request sent to a language provider.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRequest {
    /// Canonical language identifier.
    pub language: String,
    /// Source text to inspect.
    pub source: String,
}

/// Safe result returned when a provider is unavailable or fails.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProviderResponse {
    /// Provider produced semantic spans.
    Spans(Vec<(usize, usize, String)>),
    /// Plain source fallback; consumers remain usable.
    PlainText,
    /// Provider was rejected before execution.
    Unavailable(&'static str),
}

/// Validate a request without executing guest code.
pub fn validate_request(request: &ProviderRequest, limits: WasmProviderLimits) -> Result<(), &'static str> {
    if request.language.trim().is_empty() { return Err("language is empty"); }
    validate_source(&request.source, limits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_oversized_source() {
        let limits = WasmProviderLimits { max_source_bytes: 2, ..Default::default() };
        assert!(validate_source("abc", limits).is_err());
        assert!(validate_source("ab", limits).is_ok());
    }
}

impl Default for WasmProviderLimits {
    fn default() -> Self {
        Self { fuel: 1_000_000, timeout: Duration::from_millis(500), max_source_bytes: 1_000_000 }
    }
}
