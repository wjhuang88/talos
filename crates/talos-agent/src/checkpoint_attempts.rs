//! Bounded suppression of failed checkpoint attempts for an Agent instance.
use std::collections::VecDeque;
use std::hash::Hasher;
use std::io::Write;
use std::time::{Duration, Instant};

#[derive(Default)]
pub(crate) struct CheckpointAttempts {
    failed: VecDeque<u64>,
    active: Option<u64>,
    consecutive: usize,
    last_attempt: Option<Instant>,
}

impl CheckpointAttempts {
    pub(crate) fn begin(&mut self, key: u64) -> bool {
        if self.active.is_some() || self.failed.contains(&key) {
            return false;
        }
        if self.consecutive >= 3
            && self
                .last_attempt
                .is_some_and(|time| time.elapsed() < Duration::from_secs(60))
        {
            return false;
        }
        self.active = Some(key);
        true
    }

    fn finish(&mut self, key: u64, success: bool) {
        if self.active != Some(key) {
            return;
        }
        self.active = None;
        if success {
            self.consecutive = 0;
            return;
        }
        if self.failed.len() == 64 {
            self.failed.pop_front();
        }
        self.failed.push_back(key);
        self.consecutive = self.consecutive.saturating_add(1);
        self.last_attempt = Some(Instant::now());
    }
}

/// Own one attempt across await; dropping it records cancellation as failure.
pub(crate) struct AttemptGuard<'a> {
    state: &'a std::sync::Mutex<CheckpointAttempts>,
    key: u64,
    success: bool,
}

impl<'a> AttemptGuard<'a> {
    pub(crate) fn acquire(
        state: &'a std::sync::Mutex<CheckpointAttempts>,
        key: u64,
    ) -> Option<Self> {
        if !state.lock().ok()?.begin(key) {
            return None;
        }
        Some(Self {
            state,
            key,
            success: false,
        })
    }
    pub(crate) fn succeed(&mut self) {
        self.success = true;
    }
}

impl Drop for AttemptGuard<'_> {
    fn drop(&mut self) {
        if let Ok(mut state) = self.state.lock() {
            state.finish(self.key, self.success);
        }
    }
}

/// Stream serialization into a hash, avoiding a second serialized transcript.
/// Hash collisions can only suppress an optimization, never authorize execution.
pub(crate) fn fingerprint<T: serde::Serialize + ?Sized>(
    value: &T,
    deadline: tokio::time::Instant,
) -> Option<u64> {
    struct Sink(
        std::collections::hash_map::DefaultHasher,
        usize,
        tokio::time::Instant,
    );
    impl Write for Sink {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if tokio::time::Instant::now() >= self.2 || bytes.len() > self.1 {
                return Err(std::io::Error::other(
                    "checkpoint fingerprint byte limit exceeded",
                ));
            }
            self.1 -= bytes.len();
            self.0.write(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut sink = Sink(
        std::collections::hash_map::DefaultHasher::new(),
        4 * 1024 * 1024,
        deadline,
    );
    serde_json::to_writer(&mut sink, value).ok()?;
    Some(sink.0.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[test]
    fn active_attempt_is_exclusive_and_drop_suppresses_cancelled_candidate() {
        let state = Mutex::new(CheckpointAttempts::default());
        let guard = AttemptGuard::acquire(&state, 1).expect("first attempt");
        assert!(AttemptGuard::acquire(&state, 1).is_none());
        assert!(AttemptGuard::acquire(&state, 2).is_none());
        drop(guard);
        assert!(AttemptGuard::acquire(&state, 1).is_none());
        let mut next = AttemptGuard::acquire(&state, 2).expect("new candidate");
        next.succeed();
        drop(next);
        assert!(AttemptGuard::acquire(&state, 2).is_some());
    }

    #[test]
    fn three_failures_open_circuit_and_success_after_cooldown_resets_it() {
        let state = Mutex::new(CheckpointAttempts::default());
        for key in 0..3 {
            drop(AttemptGuard::acquire(&state, key).expect("attempt"));
        }
        assert!(AttemptGuard::acquire(&state, 4).is_none());
        state.lock().expect("state").last_attempt = Some(Instant::now() - Duration::from_secs(61));
        let mut guard = AttemptGuard::acquire(&state, 4).expect("cooldown elapsed");
        guard.succeed();
        drop(guard);
        assert!(AttemptGuard::acquire(&state, 5).is_some());
    }
    #[test]
    fn fingerprint_rejects_aggregate_serialization_over_budget() {
        let entries = vec!["x".repeat(1_000_000); 5];
        assert!(
            fingerprint(
                &entries,
                tokio::time::Instant::now() + Duration::from_secs(10)
            )
            .is_none()
        );
        assert!(
            fingerprint(
                &entries[..4],
                tokio::time::Instant::now() + Duration::from_secs(10)
            )
            .is_some()
        );
    }
    #[test]
    fn expired_fingerprint_budget_refuses_before_hashing() {
        assert!(fingerprint(&"source", tokio::time::Instant::now()).is_none());
    }
}
