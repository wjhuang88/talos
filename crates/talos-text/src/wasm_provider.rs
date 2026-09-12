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
