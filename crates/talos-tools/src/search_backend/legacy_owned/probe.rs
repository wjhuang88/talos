//! Query-free legacy reachability and minimum interval.
//! Only used by the standalone (unprepared) historical WebSearchTool path.
//! Protected SearchRouter retains its separate authority-checked probe.

use std::{
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

use super::{LegacyConfig, SearchBackendError};
use reqwest::Client;

const TTL: Duration = Duration::from_secs(30);

#[derive(Clone, Copy)]
struct Entry {
    reachable: bool,
    checked: Instant,
}

#[derive(Default)]
struct Cache {
    ddg: Option<Entry>,
    bing: Option<Entry>,
}

static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();

fn cache() -> &'static Mutex<Cache> {
    CACHE.get_or_init(|| Mutex::new(Cache::default()))
}

fn valid(entry: Option<Entry>, now: Instant) -> Option<bool> {
    entry
        .filter(|e| now.saturating_duration_since(e.checked) < TTL)
        .map(|e| e.reachable)
}

pub(super) async fn reachability(
    _config: &LegacyConfig,
    client_for: impl Fn(Duration, usize) -> Result<Client, SearchBackendError>,
) -> (bool, bool) {
    let now = Instant::now();
    let (previous_ddg, previous_bing) = {
        let state = cache().lock().unwrap_or_else(|e| e.into_inner());
        (valid(state.ddg, now), valid(state.bing, now))
    };
    if previous_ddg.is_some() && previous_bing.is_some() {
        return (
            previous_ddg.unwrap_or(false),
            previous_bing.unwrap_or(false),
        );
    }

    let http = match client_for(Duration::from_secs(3), 2) {
        Ok(client) => client,
        Err(_) => {
            return (
                previous_ddg.unwrap_or(false),
                previous_bing.unwrap_or(false),
            );
        }
    };
    // Each async move owns its client handle; do not borrow the same handle
    // in the first future and then move it into the second (E0505).
    let ddg_client = http.clone();
    let ddg = async move {
        match previous_ddg {
            Some(cached) => cached,
            None => probe_one(ddg_client, "https://duckduckgo.com/").await,
        }
    };
    let bing = async move {
        match previous_bing {
            Some(cached) => cached,
            None => probe_one(http, "https://cn.bing.com/").await,
        }
    };
    let (ddg, bing) = tokio::join!(ddg, bing);
    let mut state = cache().lock().unwrap_or_else(|e| e.into_inner());
    if previous_ddg.is_none() {
        state.ddg = Some(Entry {
            reachable: ddg,
            checked: now,
        });
    }
    if previous_bing.is_none() {
        state.bing = Some(Entry {
            reachable: bing,
            checked: now,
        });
    }
    (ddg, bing)
}

async fn probe_one(client: Client, url: &str) -> bool {
    // Upstream falls back to GET only on a non-successful HEAD response,
    // not on a HEAD connection error; no query text enters either probe.
    match client.head(url).send().await {
        Ok(response) if response.status().is_success() || response.status().is_redirection() => {
            true
        }
        Ok(_) => client
            .get(url)
            .send()
            .await
            .map(|r| r.status().is_success() || r.status().is_redirection())
            .unwrap_or(false),
        Err(_) => false,
    }
}

// Tokio's fair async mutex serializes the wait itself. A std::sync::Mutex
// released before sleep allowed concurrent callers to read the same previous
// timestamp, sleep together, and issue their legacy queries in a burst.
static LAST_REQUEST: OnceLock<tokio::sync::Mutex<Option<Instant>>> = OnceLock::new();

async fn rate_limit_with(lock: &tokio::sync::Mutex<Option<Instant>>) -> Instant {
    // Preserve the pinned crate's 1s process-global minimum interval with
    // bounded +/-30% jitter, without carrying its rand dependency.
    const INTERVAL: Duration = Duration::from_secs(1);
    let mut last = lock.lock().await;
    if let Some(previous) = *last {
        let elapsed = Instant::now().saturating_duration_since(previous);
        if elapsed < INTERVAL {
            let jitter = i64::from(uuid::Uuid::new_v4().as_bytes()[0]) * 600 / 255 - 300;
            let remaining = (INTERVAL - elapsed).as_millis() as i64;
            tokio::time::sleep(Duration::from_millis((remaining + jitter).max(0) as u64)).await;
        }
    }
    // Update only when this caller is actually admitted; the mutex remains
    // held across the wait, including cancellation of a pending waiter.
    let admitted = Instant::now();
    *last = Some(admitted);
    admitted
}

pub(super) async fn rate_limit() {
    let lock = LAST_REQUEST.get_or_init(|| tokio::sync::Mutex::new(None));
    let _ = rate_limit_with(lock).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn concurrent_waiters_preserve_global_spacing() {
        // Private limiter avoids interference from unrelated workspace tests.
        // All four waiters become runnable at once, exercising the exact race
        // that a mutex dropped before sleep would permit.
        const WAITERS: usize = 4;
        let limiter = Arc::new(tokio::sync::Mutex::new(None));
        let gate = Arc::new(tokio::sync::Barrier::new(WAITERS + 1));
        let mut joins = tokio::task::JoinSet::new();
        for _ in 0..WAITERS {
            let limiter = Arc::clone(&limiter);
            let gate = Arc::clone(&gate);
            joins.spawn(async move {
                gate.wait().await;
                rate_limit_with(&limiter).await
            });
        }
        gate.wait().await;
        let mut admitted = Vec::new();
        while let Some(next) = joins.join_next().await {
            admitted.push(next.expect("rate limiter task"));
        }
        assert_eq!(admitted.len(), WAITERS);
        admitted.sort_unstable();
        for pair in admitted.windows(2) {
            assert!(
                pair[1].duration_since(pair[0]) >= Duration::from_millis(650),
                "concurrent legacy searches must not share the same rate-limit interval"
            );
        }
    }
}

#[cfg(test)]
pub(super) fn clear_cache() {
    *cache().lock().unwrap_or_else(|e| e.into_inner()) = Cache::default();
}
