//! Bounded display-result admission; grants no fetch or rendering authority.

use crate::search_backend::{SearchBackendId, SearchBackendResult};

use super::{CompatibilityError, TransportError, Url};
use thiserror::Error;

pub(super) const MAX_CANDIDATES: u32 = 20;
const MAX_FIELD_BYTES: usize = 16_384;
const MAX_URL_BYTES: usize = 8192;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub(in crate::search_backend) enum AdmissionError {
    #[error("search_results_limit")]
    Limit,
    #[error("search_results_identity")]
    Identity,
    #[error("search_results_empty")]
    Empty,
    #[error("search_results_invalid")]
    Invalid,
    #[error("search_results_parse")]
    Parse,
}

/// Only bounded URL-admissible display data, not a router winner or network capability.
pub(in crate::search_backend) struct AdmittedCompatibilityResults(Vec<SearchBackendResult>);

impl std::fmt::Debug for AdmittedCompatibilityResults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AdmittedCompatibilityResults(<redacted>)")
    }
}

impl AdmittedCompatibilityResults {
    pub(in crate::search_backend) fn into_results(self) -> Vec<SearchBackendResult> {
        self.0
    }
}

pub(super) fn admit(
    expected_backend: SearchBackendId,
    candidates: Vec<SearchBackendResult>,
    max_results: u32,
    mut check_live: impl FnMut() -> Result<(), TransportError>,
) -> Result<AdmittedCompatibilityResults, CompatibilityError> {
    check_live()?;
    if !(1..=MAX_CANDIDATES).contains(&max_results) {
        return Err(TransportError::InvalidRequest.into());
    }
    if candidates.len() > MAX_CANDIDATES as usize {
        return Err(AdmissionError::Limit.into());
    }
    if candidates.is_empty() {
        return Err(AdmissionError::Empty.into());
    }
    // Validate the entire already-materialized response before truncating any output.
    for candidate in &candidates {
        check_live()?;
        if candidate.backend != expected_backend {
            return Err(AdmissionError::Identity.into());
        }
        if candidate.title.len() > MAX_FIELD_BYTES
            || candidate.snippet.len() > MAX_FIELD_BYTES
            || candidate.url.len() > MAX_URL_BYTES
        {
            return Err(AdmissionError::Limit.into());
        }
        check_live()?;
    }
    let mut accepted = Vec::new();
    for candidate in candidates {
        check_live()?;
        // WHATWG parsing may repair raw text; original display fields are preserved.
        // This is unwind recovery, not panic-hook suppression or abort/OOM recovery.
        let valid = std::panic::catch_unwind(|| {
            Url::parse(&candidate.url).is_ok_and(|url| {
                matches!(url.scheme(), "http" | "https")
                    && url.host_str().is_some()
                    && url.username().is_empty()
                    && url.password().is_none()
            })
        });
        check_live()?;
        if valid.map_err(|_| AdmissionError::Parse)? {
            accepted.push(candidate);
            if accepted.len() == max_results as usize {
                break;
            }
        }
    }
    if accepted.is_empty() {
        return Err(AdmissionError::Invalid.into());
    }
    check_live()?;
    Ok(AdmittedCompatibilityResults(accepted))
}

#[cfg(test)]
mod tests;
