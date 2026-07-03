use std::time::{SystemTime, UNIX_EPOCH};

use talos_core::provider::ProviderError;

/// Classification of a provider failure for retry decisions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetryDecision {
    Retry { attempt: u32, delay_ms: u64 },
    DoNotRetry,
}

const DEFAULT_BACKOFF_BASE_MS: u64 = 500;
const DEFAULT_BACKOFF_MAX_MS: u64 = 8_000;
const JITTER_PERCENT: u64 = 20;

/// Classify a provider error and decide whether to retry.
///
/// Retryable: rate limit, server errors, and network/transport failures.
#[must_use]
pub fn classify_retry(error: &ProviderError, attempt: u32, max_attempts: u32) -> RetryDecision {
    classify_retry_with_backoff(
        error,
        attempt,
        max_attempts,
        DEFAULT_BACKOFF_BASE_MS,
        DEFAULT_BACKOFF_MAX_MS,
    )
}

/// Classify a provider error and decide whether to retry with configurable backoff bounds.
#[must_use]
pub fn classify_retry_with_backoff(
    error: &ProviderError,
    attempt: u32,
    max_attempts: u32,
    backoff_base_ms: u64,
    backoff_max_ms: u64,
) -> RetryDecision {
    let retryable = matches!(
        error,
        ProviderError::RateLimited(_)
            | ProviderError::ServerError(_)
            | ProviderError::NetworkError(_)
    );

    if !retryable || attempt >= max_attempts {
        return RetryDecision::DoNotRetry;
    }

    let delay_ms = backoff_delay_ms_with_bounds(attempt, backoff_base_ms, backoff_max_ms);
    RetryDecision::Retry {
        attempt: attempt + 1,
        delay_ms,
    }
}

/// Exponential backoff with ±20% jitter.
#[must_use]
pub fn backoff_delay_ms(attempt: u32) -> u64 {
    backoff_delay_ms_with_bounds(attempt, DEFAULT_BACKOFF_BASE_MS, DEFAULT_BACKOFF_MAX_MS)
}

/// Calculate backoff delay with a deterministic seed (for tests).
#[must_use]
pub fn backoff_delay_ms_seeded(attempt: u32, seed: u64) -> u64 {
    backoff_delay_ms_seeded_with_bounds(
        attempt,
        seed,
        DEFAULT_BACKOFF_BASE_MS,
        DEFAULT_BACKOFF_MAX_MS,
    )
}

#[must_use]
pub fn backoff_delay_ms_with_bounds(attempt: u32, base_ms: u64, max_ms: u64) -> u64 {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::from(d.subsec_nanos()));
    backoff_delay_ms_seeded_with_bounds(attempt, seed, base_ms, max_ms)
}

fn backoff_delay_ms_seeded_with_bounds(attempt: u32, seed: u64, base_ms: u64, max_ms: u64) -> u64 {
    let exp = base_ms.saturating_mul(2u64.saturating_pow(attempt));
    let capped = exp.min(max_ms);
    let jitter_range = capped.saturating_mul(JITTER_PERCENT) / 100;
    if jitter_range == 0 {
        return capped;
    }

    let spread = jitter_range.saturating_mul(2).saturating_add(1);
    let jitter = (seed % spread) as i64 - jitter_range as i64;

    if jitter >= 0 {
        capped.saturating_add(jitter as u64)
    } else {
        capped.saturating_sub((-jitter) as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_429_is_retryable() {
        let decision = classify_retry(&ProviderError::RateLimited("429".into()), 0, 3);
        assert!(matches!(decision, RetryDecision::Retry { .. }));
    }

    #[test]
    fn test_retry_500_is_retryable() {
        let decision = classify_retry(&ProviderError::ServerError("500".into()), 0, 3);
        assert!(matches!(decision, RetryDecision::Retry { .. }));
    }

    #[test]
    fn test_retry_401_not_retryable() {
        let decision = classify_retry(&ProviderError::AuthenticationFailed("401".into()), 0, 3);
        assert_eq!(decision, RetryDecision::DoNotRetry);
    }

    #[test]
    fn test_retry_400_not_retryable() {
        let decision = classify_retry(&ProviderError::InvalidResponse("400".into()), 0, 3);
        assert_eq!(decision, RetryDecision::DoNotRetry);
    }

    #[test]
    fn test_retry_exhausted() {
        let decision = classify_retry(&ProviderError::RateLimited("429".into()), 3, 3);
        assert_eq!(decision, RetryDecision::DoNotRetry);
    }

    #[test]
    fn test_backoff_delay_growth() {
        let d0 = backoff_delay_ms_seeded(0, 0);
        let d1 = backoff_delay_ms_seeded(1, 0);
        let d2 = backoff_delay_ms_seeded(2, 0);
        assert!(d0 < d1);
        assert!(d1 < d2);
    }

    #[test]
    fn test_backoff_delay_capped() {
        for attempt in 0..16 {
            let delay = backoff_delay_ms_seeded(attempt, 0);
            // Max capped delay plus +20% jitter.
            assert!(delay <= 9_600);
        }
    }

    #[test]
    fn test_backoff_jitter_range() {
        let attempt = 3;
        let base = 500_u64.saturating_mul(2u64.saturating_pow(attempt));
        let capped = base.min(8_000);
        let jitter_range = capped * 20 / 100;
        for seed in [0, 1, 2, 13, 97, 10_001, u64::MAX] {
            let delay = backoff_delay_ms_seeded(attempt, seed);
            let min = capped.saturating_sub(jitter_range);
            let max = capped.saturating_add(jitter_range);
            assert!(delay >= min);
            assert!(delay <= max);
        }
    }
}
