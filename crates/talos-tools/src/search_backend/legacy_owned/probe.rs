//! Query-free legacy reachability and minimum interval.
//! Only used by the standalone (unprepared) historical WebSearchTool path.
//! Protected SearchRouter retains its separate authority-checked probe.

use std::{
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

use reqwest::Client;
use super::{LegacyConfig, SearchBackendError};

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
    entry.filter(|e| now.saturating_duration_since(e.checked) < TTL)
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
        return (previous_ddg.unwrap_or(false), previous_bing.unwrap_or(false));
    }

    let http = match client_for(Duration::from_secs(3), 2) {
        Ok(client) => client,
        Err(_) => return (previous_ddg.unwrap_or(false), previous_bing.unwrap_or(false)),
    };
    let ddg = async {
        match previous_ddg {
            Some(cached) => cached,
            None => probe_one(http.clone(), "https://duckduckgo.com/").await,
        }
    };
    let bing = async {
        match previous_bing {
            Some(cached) => cached,
            None => probe_one(http, "https://cn.bing.com/").await,
        }
    };
    let (ddg, bing) = tokio::join!(ddg, bing);
    let mut state = cache().lock().unwrap_or_else(|e| e.into_inner());
    if previous_ddg.is_none() {
        state.ddg = Some(Entry { reachable: ddg, checked: now });
    }
    if previous_bing.is_none() {
        state.bing = Some(Entry { reachable: bing, checked: now });
    }
    (ddg, bing)
}

async fn probe_one(client: Client, url: &str) -> bool {
    // Upstream falls back to GET only on a non-successful HEAD response,
    // not on a HEAD connection error; no query text enters either probe.
    match client.head(url).send().await {
        Ok(response) if response.status().is_success() || response.status().is_redirection() => true,
        Ok(_) => client.get(url).send().await
            .map(|r| r.status().is_success() || r.status().is_redirection())
            .unwrap_or(false),
        Err(_) => false,
    }
}

static LAST_REQUEST: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();

pub(super) async fn rate_limit() {
    // Preserve the pinned crate's 1s process-global minimum interval with
    // bounded +/-30% jitter, without carrying its rand dependency.
    const INTERVAL: Duration = Duration::from_secs(1);
    let lock = LAST_REQUEST.get_or_init(|| Mutex::new(None));
    let delay = {
        let mut last = lock.lock().unwrap_or_else(|e| e.into_inner());
        let now = Instant::now();
        if let Some(previous) = *last {
            let elapsed = now.saturating_duration_since(previous);
            if elapsed < INTERVAL {
                let jitter = i64::from(uuid::Uuid::new_v4().as_bytes()[0]) * 600 / 255 - 300;
                let remaining = (INTERVAL - elapsed).as_millis() as i64;
                Some(Duration::from_millis((remaining + jitter).max(0) as u64))
            } else {
                *last = Some(now);
                None
            }
        } else {
            *last = Some(now);
            None
        }
    };
    if let Some(wait) = delay {
        tokio::time::sleep(wait).await;
        *lock.lock().unwrap_or_else(|e| e.into_inner()) = Some(Instant::now());
    }
}

#[cfg(test)]
pub(super) fn clear_cache() {
    *cache().lock().unwrap_or_else(|e| e.into_inner()) = Cache::default();
}
