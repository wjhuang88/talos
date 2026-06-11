//! In-flight turn cancellation registry.

use std::collections::HashMap;
use std::sync::Mutex;

use tokio_util::sync::CancellationToken;

/// Registry mapping turn IDs to cancellation tokens.
#[derive(Default)]
pub struct CancelRegistry {
    inner: Mutex<HashMap<String, CancellationToken>>,
}

impl CancelRegistry {
    /// Creates an empty cancellation registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts or replaces a token for `turn_id`.
    pub fn insert(&self, turn_id: String, token: CancellationToken) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.insert(turn_id, token);
        }
    }

    /// Cancels a token by `turn_id` and removes it.
    #[must_use]
    pub fn cancel(&self, turn_id: &str) -> bool {
        if let Ok(mut guard) = self.inner.lock()
            && let Some(token) = guard.remove(turn_id)
        {
            token.cancel();
            return true;
        }
        false
    }

    /// Removes a token by `turn_id` without cancelling it.
    pub fn remove(&self, turn_id: &str) {
        if let Ok(mut guard) = self.inner.lock() {
            let _ = guard.remove(turn_id);
        }
    }
}
