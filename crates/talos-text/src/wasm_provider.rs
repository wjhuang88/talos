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

impl Default for WasmProviderLimits {
    fn default() -> Self {
        Self { fuel: 1_000_000, timeout: Duration::from_millis(500), max_source_bytes: 1_000_000 }
    }
}
