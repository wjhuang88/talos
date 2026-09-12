//! Bounded WASM language-provider boundary.

/// Versioned guest ABI identifier. Providers must expose this contract before loading.
pub const WASM_LANGUAGE_ABI_VERSION: u32 = 1;
/// Required guest export for provider probing (must not be called during discovery).
pub const WASM_LANGUAGE_ABI_EXPORT: &str = "talos_language_abi_version";

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

/// Validate the declared ABI version without executing a provider.
pub fn validate_abi_version(version: u32) -> Result<(), &'static str> {
    (version == WASM_LANGUAGE_ABI_VERSION).then_some(()).ok_or("unsupported provider ABI version")
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

/// Convert a bounded provider response into the existing renderer-neutral result.
pub fn fallback_response(request: &ProviderRequest, limits: WasmProviderLimits) -> ProviderResponse {
    match validate_request(request, limits) {
        Ok(()) => ProviderResponse::PlainText,
        Err(reason) => ProviderResponse::Unavailable(reason),
    }
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

    #[test]
    fn invalid_request_degrades_without_execution() {
        let request = ProviderRequest { language: String::new(), source: "x".into() };
        assert_eq!(fallback_response(&request, WasmProviderLimits::default()), ProviderResponse::Unavailable("language is empty"));
    }
}

impl Default for WasmProviderLimits {
    fn default() -> Self {
        Self { fuel: 1_000_000, timeout: Duration::from_millis(500), max_source_bytes: 1_000_000 }
    }
}
