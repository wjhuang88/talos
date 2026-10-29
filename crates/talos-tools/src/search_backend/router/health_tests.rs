use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::oneshot;
use tokio::time::{Instant, sleep};

use super::super::health::{SearchHealthSignal, SearchHealthState};
use super::*;

type SharedStarts = Arc<Mutex<Vec<SearchBackendId>>>;

#[derive(Clone)]
struct HealthMockBackend {
    id: SearchBackendId,
    delay: Duration,
    outcome: Result<Vec<SearchBackendResult>, SearchBackendError>,
    starts: SharedStarts,
}

#[async_trait]
impl SearchBackend for HealthMockBackend {
    fn id(&self) -> SearchBackendId {
        self.id
    }

    async fn search(
        &self,
        _request: SearchBackendRequest,
    ) -> Result<Vec<SearchBackendResult>, SearchBackendError> {
        self.starts
            .lock()
            .expect("health fixture starts mutex")
            .push(self.id);
        sleep(self.delay).await;
        self.outcome.clone()
    }
}

fn result(backend: SearchBackendId, suffix: &str) -> SearchBackendResult {
    SearchBackendResult {
        backend,
        title: format!("{backend:?} health fixture"),
        url: format!("https://example.invalid/{suffix}"),
        snippet: "health fixture snippet".into(),
    }
}

fn backend(
    id: SearchBackendId,
    delay_ms: u64,
    outcome: Result<Vec<SearchBackendResult>, SearchBackendError>,
    starts: &SharedStarts,
) -> Arc<dyn SearchBackend> {
    Arc::new(HealthMockBackend {
        id,
        delay: Duration::from_millis(delay_ms),
        outcome,
        starts: Arc::clone(starts),
    })
}

fn request() -> SearchBackendRequest {
    SearchBackendRequest {
        query: "bounded health fixture".into(),
        max_results: 1,
    }
}

fn policy(hedge_ms: u64, max_in_flight: usize) -> SearchRouterPolicy {
    SearchRouterPolicy::new(Duration::from_millis(hedge_ms), max_in_flight, 8, 512)
        .expect("health fixture policy")
}

fn health(ttl: Duration) -> SearchHealthState {
    SearchHealthState::new(4, ttl).expect("health fixture state")
}

#[tokio::test(start_paused = true)]
async fn observed_healthy_candidate_moves_ahead_of_degraded_static_primary() {
    let starts = SharedStarts::default();
    let now = Instant::now();
    let mut health = health(Duration::from_secs(60));
    health.record(
        SearchBackendId::DuckDuckGo,
        SearchHealthSignal::Timeout,
        Duration::from_millis(20),
        now,
    );
    health.record(
        SearchBackendId::SearXng,
        SearchHealthSignal::Success,
        Duration::from_millis(5),
        now,
    );
    let router = SearchRouter::new(
        vec![
            backend(
                SearchBackendId::DuckDuckGo,
                0,
                Ok(vec![result(SearchBackendId::DuckDuckGo, "static-first")]),
                &starts,
            ),
            backend(
                SearchBackendId::SearXng,
                0,
                Ok(vec![result(SearchBackendId::SearXng, "healthy-first")]),
                &starts,
            ),
        ],
        policy(50, 1),
    )
    .expect("router");
    let (_cancel, mut cancellation) = oneshot::channel();

    let success = router
        .search_with_health(
            request(),
            now + Duration::from_secs(1),
            &mut cancellation,
            &mut health,
        )
        .await
        .expect("healthy route wins");

    assert_eq!(success.backend, SearchBackendId::SearXng);
    assert_eq!(
        *starts.lock().expect("health fixture starts mutex"),
        vec![SearchBackendId::SearXng]
    );
}

#[tokio::test(start_paused = true)]
async fn no_results_health_is_neutral_and_preserves_static_order_against_unknown() {
    let starts = SharedStarts::default();
    let now = Instant::now();
    let mut health = health(Duration::from_secs(60));
    health.record(
        SearchBackendId::DuckDuckGo,
        SearchHealthSignal::NoResults,
        Duration::from_millis(2),
        now,
    );
    let router = SearchRouter::new(
        vec![
            backend(
                SearchBackendId::DuckDuckGo,
                0,
                Ok(vec![result(SearchBackendId::DuckDuckGo, "neutral-static")]),
                &starts,
            ),
            backend(
                SearchBackendId::SearXng,
                0,
                Ok(vec![result(SearchBackendId::SearXng, "unknown-second")]),
                &starts,
            ),
        ],
        policy(50, 1),
    )
    .expect("router");
    let (_cancel, mut cancellation) = oneshot::channel();

    let success = router
        .search_with_health(
            request(),
            now + Duration::from_secs(1),
            &mut cancellation,
            &mut health,
        )
        .await
        .expect("static route remains first");

    assert_eq!(success.backend, SearchBackendId::DuckDuckGo);
    assert_eq!(
        *starts.lock().expect("health fixture starts mutex"),
        vec![SearchBackendId::DuckDuckGo]
    );
}

#[tokio::test(start_paused = true)]
async fn expired_degraded_evidence_restores_static_order_on_later_search() {
    let starts = SharedStarts::default();
    let observed_at = Instant::now();
    let mut health = health(Duration::from_secs(5));
    health.record(
        SearchBackendId::DuckDuckGo,
        SearchHealthSignal::Unreachable,
        Duration::from_millis(30),
        observed_at,
    );
    sleep(Duration::from_secs(5)).await;
    let search_at = Instant::now();
    let router = SearchRouter::new(
        vec![
            backend(
                SearchBackendId::DuckDuckGo,
                0,
                Ok(vec![result(SearchBackendId::DuckDuckGo, "reevaluated")]),
                &starts,
            ),
            backend(
                SearchBackendId::SearXng,
                0,
                Ok(vec![result(SearchBackendId::SearXng, "second")]),
                &starts,
            ),
        ],
        policy(50, 1),
    )
    .expect("router");
    let (_cancel, mut cancellation) = oneshot::channel();

    let success = router
        .search_with_health(
            request(),
            search_at + Duration::from_secs(1),
            &mut cancellation,
            &mut health,
        )
        .await
        .expect("expired route is reevaluated");

    assert_eq!(success.backend, SearchBackendId::DuckDuckGo);
    assert_eq!(
        *starts.lock().expect("health fixture starts mutex"),
        vec![SearchBackendId::DuckDuckGo]
    );
}

#[tokio::test(start_paused = true)]
async fn completed_failure_records_coarse_signal_and_latency_before_later_success() {
    let starts = SharedStarts::default();
    let start = Instant::now();
    let mut health = health(Duration::from_secs(60));
    let router = SearchRouter::new(
        vec![
            backend(
                SearchBackendId::DuckDuckGo,
                5,
                Err(SearchBackendError::Timeout),
                &starts,
            ),
            backend(
                SearchBackendId::SearXng,
                5,
                Ok(vec![result(SearchBackendId::SearXng, "success")]),
                &starts,
            ),
        ],
        policy(50, 1),
    )
    .expect("router");
    let (_cancel, mut cancellation) = oneshot::channel();

    let success = router
        .search_with_health(
            request(),
            start + Duration::from_secs(1),
            &mut cancellation,
            &mut health,
        )
        .await
        .expect("second route succeeds");

    assert_eq!(success.backend, SearchBackendId::SearXng);
    let now = Instant::now();
    let failed = health
        .observation(SearchBackendId::DuckDuckGo, now)
        .expect("failure observation");
    assert_eq!(failed.signal, SearchHealthSignal::Timeout);
    assert_eq!(failed.latency, Duration::from_millis(5));
    let succeeded = health
        .observation(SearchBackendId::SearXng, now)
        .expect("success observation");
    assert_eq!(succeeded.signal, SearchHealthSignal::Success);
    assert_eq!(succeeded.latency, Duration::from_millis(5));
}

#[tokio::test(start_paused = true)]
async fn pending_loser_drop_is_not_recorded_as_health_failure() {
    let starts = SharedStarts::default();
    let start = Instant::now();
    let mut health = health(Duration::from_secs(60));
    let router = SearchRouter::new(
        vec![
            backend(
                SearchBackendId::DuckDuckGo,
                100,
                Ok(vec![result(SearchBackendId::DuckDuckGo, "slow-loser")]),
                &starts,
            ),
            backend(
                SearchBackendId::SearXng,
                5,
                Ok(vec![result(SearchBackendId::SearXng, "fast-winner")]),
                &starts,
            ),
        ],
        policy(10, 2),
    )
    .expect("router");
    let (_cancel, mut cancellation) = oneshot::channel();

    let success = router
        .search_with_health(
            request(),
            start + Duration::from_secs(1),
            &mut cancellation,
            &mut health,
        )
        .await
        .expect("hedged route wins");

    assert_eq!(success.backend, SearchBackendId::SearXng);
    assert_eq!(start.elapsed(), Duration::from_millis(15));
    assert_eq!(
        *starts.lock().expect("health fixture starts mutex"),
        vec![SearchBackendId::DuckDuckGo, SearchBackendId::SearXng]
    );
    let now = Instant::now();
    assert!(
        health
            .observation(SearchBackendId::DuckDuckGo, now)
            .is_none()
    );
    assert_eq!(
        health
            .observation(SearchBackendId::SearXng, now)
            .expect("winner observation")
            .signal,
        SearchHealthSignal::Success
    );
}
