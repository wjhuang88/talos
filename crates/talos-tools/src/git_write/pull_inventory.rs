//! Shared bounded capture allowance for Git configuration and receiver evidence.

use std::path::Path;
use std::sync::{Arc, Mutex};

use super::GitToolError;

/// Cumulative file capture allowance shared across discovery continuations.
/// Full transient-resource and identity-handle accounting remain separate.
#[derive(Clone, Debug)]
pub(super) struct ReferenceBudget(Arc<Mutex<(usize, usize)>>);

impl Default for ReferenceBudget {
    fn default() -> Self {
        Self(Arc::new(Mutex::new((4096, 4 * 1024 * 1024))))
    }
}

impl ReferenceBudget {
    pub(super) fn capture(
        &self,
        path: &Path,
    ) -> Result<super::snapshot::RegularEvidence, GitToolError> {
        self.capture_limited(path, usize::MAX)
    }

    fn capture_limited(
        &self,
        path: &Path,
        max_bytes: usize,
    ) -> Result<super::snapshot::RegularEvidence, GitToolError> {
        self.capture_with_budget(
            path,
            &mut super::snapshot::Budget {
                entries: usize::MAX,
                bytes: max_bytes,
            },
        )
    }

    pub(super) fn capture_with_budget(
        &self,
        path: &Path,
        local: &mut super::snapshot::Budget,
    ) -> Result<super::snapshot::RegularEvidence, GitToolError> {
        let mut remaining = self.0.lock().map_err(|_| invalid())?;
        let available_entries = remaining.0.min(local.entries);
        let available_bytes = remaining.1.min(local.bytes);
        let mut allowance = super::snapshot::Budget {
            entries: available_entries,
            bytes: available_bytes,
        };
        let result = super::snapshot::RegularEvidence::capture(path, &mut allowance);
        // Failed captures also consume any work already charged. A continuation
        // or replacement inventory must never restore the invocation allowance.
        let entries = available_entries
            .checked_sub(allowance.entries)
            .ok_or_else(invalid)?;
        let bytes = available_bytes
            .checked_sub(allowance.bytes)
            .ok_or_else(invalid)?;
        remaining.0 -= entries;
        remaining.1 -= bytes;
        local.entries -= entries;
        local.bytes -= bytes;
        result.map_err(|_| invalid())
    }
}

#[cfg_attr(test, track_caller)]
fn invalid() -> GitToolError {
    // Keep production errors opaque, but identify the rejecting invariant in
    // test failures without logging repository paths or reference contents.
    #[cfg(test)]
    eprintln!(
        "Pull inventory rejection at {}",
        std::panic::Location::caller()
    );
    GitToolError::InvalidInput(
        "Pull graph inventory changed, redirected, or exceeded its limit".into(),
    )
}
