//! Bounded WASM language-provider boundary.

/// Versioned guest ABI identifier. Providers must expose this contract before loading.
pub const WASM_LANGUAGE_ABI_VERSION: u32 = 1;
/// Required guest export for provider probing (must not be called during discovery).
pub const WASM_LANGUAGE_ABI_EXPORT: &str = "talos_language_abi_version";
/// Required guest export for a provider invocation. It receives a pointer/length pair
/// in guest memory and returns a pointer/length pair for a JSON response.
pub const WASM_LANGUAGE_RUN_EXPORT: &str = "talos_language_run";
/// Guest run signature: request pointer/length, packed response pointer/length.
pub const WASM_LANGUAGE_RUN_SIGNATURE: &str = "(i32,i32)->i64";
/// ABI contract version for the memory transport described by [`WASM_LANGUAGE_RUN_EXPORT`].
pub const WASM_LANGUAGE_MEMORY_ABI_VERSION: u32 = 1;

/// Versioned symbol operation transport contract.
pub const WASM_LANGUAGE_SYMBOL_ABI_VERSION: u32 = 1;

/// Source-only symbol operation requested from a provider.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum SymbolOperation {
    /// Find definitions and references for a name.
    FindSymbol { name: String },
    /// List symbols in the supplied source.
    ListSymbols { kind: Option<String> },
    /// List imports in the supplied source.
    ListImports,
    /// Find references for a name.
    FindReferences { name: String },
}

/// Request envelope for a bounded source-only symbol query.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SymbolRequest {
    /// Contract version.
    pub abi_version: u32,
    /// Canonical language identifier.
    pub language: String,
    /// Source text; providers must not access the filesystem.
    pub source: String,
    /// Requested operation.
    pub operation: SymbolOperation,
}

/// Provider-neutral symbol response envelope.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SymbolResponse {
    /// JSON-compatible provider result.
    Result(serde_json::Value),
    /// Provider cannot serve this operation.
    Unavailable(String),
}

/// Validate a symbol request before crossing the provider boundary.
pub fn validate_symbol_request(
    request: &SymbolRequest,
    limits: WasmProviderLimits,
) -> Result<(), &'static str> {
    if request.abi_version != WASM_LANGUAGE_SYMBOL_ABI_VERSION {
        return Err("unsupported symbol provider ABI version");
    }
    if request.language.trim().is_empty() {
        return Err("language is empty");
    }
    if request.source.len() > limits.max_source_bytes {
        return Err("source exceeds provider limit");
    }
    match &request.operation {
        SymbolOperation::FindSymbol { name } | SymbolOperation::FindReferences { name }
            if name.trim().is_empty() =>
        {
            Err("symbol name is empty")
        }
        _ => Ok(()),
    }
}

/// Encode a validated symbol request for guest transport.
pub fn encode_symbol_request(
    request: &SymbolRequest,
    limits: WasmProviderLimits,
) -> Result<Vec<u8>, &'static str> {
    validate_symbol_request(request, limits)?;
    serde_json::to_vec(request).map_err(|_| "symbol request serialization failed")
}

/// Decode a bounded symbol response from guest transport.
pub fn decode_symbol_response(
    bytes: &[u8],
    max_bytes: usize,
) -> Result<SymbolResponse, &'static str> {
    if bytes.len() > max_bytes {
        return Err("symbol response exceeds limit");
    }
    serde_json::from_slice(bytes).map_err(|_| "symbol response decode failed")
}

/// Decode a successful provider value into a caller-selected typed result.
pub fn decode_symbol_value<T: serde::de::DeserializeOwned>(
    response: SymbolResponse,
) -> Result<T, &'static str> {
    match response {
        SymbolResponse::Result(value) => {
            serde_json::from_value(value).map_err(|_| "symbol provider result has an invalid shape")
        }
        SymbolResponse::Unavailable(_) => Err("symbol provider result unavailable"),
    }
}

/// Validate a guest memory range without allowing integer overflow.
pub fn validate_memory_range(
    offset: u32,
    length: u32,
    memory_len: usize,
) -> Result<std::ops::Range<usize>, &'static str> {
    let start = offset as usize;
    let end = start
        .checked_add(length as usize)
        .ok_or("provider memory range overflow")?;
    if end > memory_len {
        return Err("provider memory range out of bounds");
    }
    Ok(start..end)
}

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
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProviderRequest {
    /// Canonical language identifier.
    pub language: String,
    /// Source text to inspect.
    pub source: String,
}

/// Validate the declared ABI version without executing a provider.
pub fn validate_abi_version(version: u32) -> Result<(), &'static str> {
    (version == WASM_LANGUAGE_ABI_VERSION)
        .then_some(())
        .ok_or("unsupported provider ABI version")
}

/// Safe result returned when a provider is unavailable or fails.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ProviderResponse {
    /// Provider produced semantic spans.
    Spans(Vec<(usize, usize, String)>),
    /// Plain source fallback; consumers remain usable.
    PlainText,
    /// Provider was rejected before execution.
    Unavailable(String),
}

/// Reject malformed or overlapping guest spans before exposing them to consumers.
pub fn validate_spans(
    spans: &[(usize, usize, String)],
    source_len: usize,
) -> Result<(), &'static str> {
    let mut end = 0;
    for (start, stop, _) in spans {
        if *start > *stop || *stop > source_len || *start < end {
            return Err("invalid provider span");
        }
        end = *stop;
    }
    Ok(())
}

/// Convert validated guest spans to the shared highlighting result.
pub fn spans_to_highlight(
    spans: Vec<(usize, usize, String)>,
    source_len: usize,
) -> super::HighlightResult {
    if validate_spans(&spans, source_len).is_err() {
        return super::HighlightResult::PlainText;
    }
    super::HighlightResult::Spans(
        spans
            .into_iter()
            .map(|(start, end, capture)| super::HighlightSpan {
                start,
                end,
                capture,
            })
            .collect(),
    )
}

/// Encode a request for the versioned guest boundary.
pub fn encode_request(
    request: &ProviderRequest,
    limits: WasmProviderLimits,
) -> Result<Vec<u8>, &'static str> {
    validate_request(request, limits)?;
    serde_json::to_vec(request).map_err(|_| "provider request serialization failed")
}

/// Decode a bounded provider response payload.
pub fn decode_response(bytes: &[u8], max_bytes: usize) -> Result<ProviderResponse, &'static str> {
    if bytes.len() > max_bytes {
        return Err("provider response exceeds limit");
    }
    serde_json::from_slice(bytes).map_err(|_| "provider response decode failed")
}

/// Decode and validate a guest response for a source buffer.
pub fn decode_highlight(
    bytes: &[u8],
    max_bytes: usize,
    source_len: usize,
) -> super::HighlightResult {
    match decode_response(bytes, max_bytes) {
        Ok(ProviderResponse::Spans(spans)) => spans_to_highlight(spans, source_len),
        _ => super::HighlightResult::PlainText,
    }
}

/// Convert a bounded provider response into the existing renderer-neutral result.
pub fn fallback_response(
    request: &ProviderRequest,
    limits: WasmProviderLimits,
) -> ProviderResponse {
    match validate_request(request, limits) {
        Ok(()) => ProviderResponse::PlainText,
        Err(reason) => ProviderResponse::Unavailable(reason.to_owned()),
    }
}

/// Validate a request without executing guest code.
pub fn validate_request(
    request: &ProviderRequest,
    limits: WasmProviderLimits,
) -> Result<(), &'static str> {
    if request.language.trim().is_empty() {
        return Err("language is empty");
    }
    validate_source(&request.source, limits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_oversized_source() {
        let limits = WasmProviderLimits {
            max_source_bytes: 2,
            ..Default::default()
        };
        assert!(validate_source("abc", limits).is_err());
        assert!(validate_source("ab", limits).is_ok());
    }

    #[test]
    fn invalid_request_degrades_without_execution() {
        let request = ProviderRequest {
            language: String::new(),
            source: "x".into(),
        };
        assert_eq!(
            fallback_response(&request, WasmProviderLimits::default()),
            ProviderResponse::Unavailable("language is empty".into())
        );
    }

    #[test]
    fn malformed_guest_spans_are_rejected() {
        assert!(validate_spans(&[(0, 2, "x".into()), (1, 3, "y".into())], 3).is_err());
        assert!(validate_spans(&[(0, 2, "x".into()), (2, 3, "y".into())], 3).is_ok());
    }

    #[test]
    fn request_response_wire_format_is_stable() {
        let request = ProviderRequest {
            language: "rust".into(),
            source: "fn main() {}".into(),
        };
        let value = serde_json::to_value(&request).expect("serialize");
        assert_eq!(value["language"], "rust");
        let decoded: ProviderRequest = serde_json::from_value(value).expect("decode");
        assert_eq!(decoded, request);
    }

    #[test]
    fn wire_helpers_enforce_bounds() {
        let request = ProviderRequest {
            language: "rust".into(),
            source: "fn main() {}".into(),
        };
        let bytes = encode_request(&request, WasmProviderLimits::default()).expect("encode");
        assert_eq!(
            decode_response(&bytes, 1),
            Err("provider response exceeds limit")
        );
    }

    #[test]
    fn guest_abi_exports_are_versioned_and_stable() {
        assert_eq!(WASM_LANGUAGE_ABI_VERSION, 1);
        assert_eq!(WASM_LANGUAGE_MEMORY_ABI_VERSION, 1);
        assert_eq!(WASM_LANGUAGE_RUN_EXPORT, "talos_language_run");
        assert_eq!(WASM_LANGUAGE_RUN_SIGNATURE, "(i32,i32)->i64");
    }

    #[test]
    fn memory_ranges_are_checked_before_guest_decode() {
        assert_eq!(validate_memory_range(2, 3, 8).unwrap(), 2..5);
        assert!(validate_memory_range(7, 2, 8).is_err());
    }

    #[test]
    fn symbol_request_wire_contract_is_versioned_and_bounded() {
        let request = SymbolRequest {
            abi_version: WASM_LANGUAGE_SYMBOL_ABI_VERSION,
            language: "rust".into(),
            source: "fn main() {}".into(),
            operation: SymbolOperation::FindSymbol {
                name: "main".into(),
            },
        };
        validate_symbol_request(&request, WasmProviderLimits::default()).expect("valid request");
        let encoded = serde_json::to_vec(&request).expect("encode");
        let decoded: SymbolRequest = serde_json::from_slice(&encoded).expect("decode");
        assert_eq!(decoded, request);
    }
}

impl Default for WasmProviderLimits {
    fn default() -> Self {
        Self {
            fuel: 1_000_000,
            timeout: Duration::from_millis(500),
            max_source_bytes: 1_000_000,
        }
    }
}
