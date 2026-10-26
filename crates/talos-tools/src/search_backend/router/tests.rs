use std::future::{Future, poll_fn};
use std::sync::{Arc, Mutex};
use std::task::Poll;
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::oneshot;
use tokio::time::{Instant, sleep};

use super::*;

#[derive(Default, Debug)]
struct Counts {
    starts: Vec<SearchBackendId>,
    drops: Vec<SearchBackendId>,
    active: usize,
    peak: usize,
}

type SharedCounts = Arc<Mutex<Counts>>;

struct ActiveGuard {
    id: SearchBackendId,
    counts: SharedCounts,
}

impl Drop for ActiveGuard {
    fn drop(&mut self) {
        let mut counts = self.counts.lock().expect("fixture counts mutex");
        counts.active -= 1;
        counts.drops.push(self.id);
    }
}

#[derive(Clone)]
struct MockBackend {
    id: SearchBackendId,
    delay: Duration,
    outcome: Result<Vec<SearchBackendResult>, SearchBackendError>,
    counts: SharedCounts,
}

#[async_trait]
impl SearchBackend for MockBackend {
    fn id(&self) -> SearchBackendId {
        self.id
    }

    async fn search(
        &self,
        _request: SearchBackendRequest,
    ) -> Result<Vec<SearchBackendResult>, SearchBackendError> {
        let guard = {
            let mut counts = self.counts.lock().expect("fixture counts mutex");
            counts.starts.push(self.id);
            counts.active += 1;
            counts.peak = counts.peak.max(counts.active);
            ActiveGuard {
                id: self.id,
                counts: Arc::clone(&self.counts),
            }
        };
        let _guard = guard;
        sleep(self.delay).await;
        self.outcome.clone()
    }
}

fn result(backend: SearchBackendId, url: &str) -> SearchBackendResult {
    SearchBackendResult {
        backend,
        title: format!("{backend:?} title"),
        url: url.into(),
        snippet: "fixture snippet".into(),
    }
}

fn backend(
    id: SearchBackendId,
    delay_ms: u64,
    outcome: Result<Vec<SearchBackendResult>, SearchBackendError>,
    counts: &SharedCounts,
) -> Arc<dyn SearchBackend> {
    Arc::new(MockBackend {
        id,
        delay: Duration::from_millis(delay_ms),
        outcome,
        counts: Arc::clone(counts),
    })
}

fn request(max_results: u32) -> SearchBackendRequest {
    SearchBackendRequest {
        query: "private router fixture".into(),
        max_results,
    }
}

fn policy(hedge_ms: u64, max_in_flight: usize) -> SearchRouterPolicy {
    SearchRouterPolicy::new(Duration::from_millis(hedge_ms), max_in_flight, 20, 512)
        .expect("fixture policy")
}

fn assert_released(counts: &SharedCounts, starts: &[SearchBackendId], peak: usize) {
    let counts = counts.lock().expect("fixture counts mutex");
    assert_eq!(counts.starts, starts);
    assert_eq!(counts.active, 0);
    assert_eq!(counts.peak, peak);
    let mut actual = counts.drops.clone();
    let mut expected = starts.to_vec();
    actual.sort_by_key(|id| *id as u8);
    expected.sort_by_key(|id| *id as u8);
    assert_eq!(actual, expected);
}

#[tokio::test(start_paused = true)]
async fn fast_failure_launches_replacement_and_waits_for_first_valid_success() {
    let counts = SharedCounts::default();
    let router = SearchRouter::new(
        vec![
            backend(
                SearchBackendId::DuckDuckGo,
                5,
                Err(SearchBackendError::RequestFailed("fixture".into())),
                &counts,
            ),
            backend(
                SearchBackendId::SearXng,
                15,
                Ok(vec![result(
                    SearchBackendId::SearXng,
                    "https://example.invalid/result",
                )]),
                &counts,
            ),
        ],
        policy(50, 2),
    )
    .expect("router");
    let (_cancel, mut cancellation) = oneshot::channel();
    let start = Instant::now();

    let success = router
        .search(
            request(1),
            start + Duration::from_millis(100),
            &mut cancellation,
        )
        .await
        .expect("later valid route wins");

    assert_eq!(success.backend, SearchBackendId::SearXng);
    assert_eq!(success.results.len(), 1);
    assert_eq!(start.elapsed(), Duration::from_millis(20));
    assert_released(
        &counts,
        &[SearchBackendId::DuckDuckGo, SearchBackendId::SearXng],
        1,
    );
}

#[tokio::test(start_paused = true)]
async fn hedge_is_bounded_and_winner_drops_pending_loser_without_starting_third() {
    let counts = SharedCounts::default();
    let router = SearchRouter::new(
        vec![
            backend(
                SearchBackendId::DuckDuckGo,
                100,
                Ok(vec![result(
                    SearchBackendId::DuckDuckGo,
                    "https://example.invalid/slow",
                )]),
                &counts,
            ),
            backend(
                SearchBackendId::SearXng,
                5,
                Ok(vec![result(
                    SearchBackendId::SearXng,
                    "https://example.invalid/fast",
                )]),
                &counts,
            ),
            backend(
                SearchBackendId::Tavily,
                0,
                Ok(vec![result(
                    SearchBackendId::Tavily,
                    "https://example.invalid/not-started",
                )]),
                &counts,
            ),
        ],
        policy(10, 2),
    )
    .expect("router");
    let (_cancel, mut cancellation) = oneshot::channel();
    let start = Instant::now();

    let success = router
        .search(
            request(1),
            start + Duration::from_millis(200),
            &mut cancellation,
        )
        .await
        .expect("hedged route wins");

    assert_eq!(success.backend, SearchBackendId::SearXng);
    assert_eq!(start.elapsed(), Duration::from_millis(15));
    assert_released(
        &counts,
        &[SearchBackendId::DuckDuckGo, SearchBackendId::SearXng],
        2,
    );
}

#[tokio::test(start_paused = true)]
async fn mixed_results_filter_invalid_urls_before_truncation() {
    let counts = SharedCounts::default();
    let expected_first = result(SearchBackendId::DuckDuckGo, "https://example.invalid/first");
    let expected_second = result(SearchBackendId::DuckDuckGo, "http://example.invalid/second");
    let router = SearchRouter::new(
        vec![backend(
            SearchBackendId::DuckDuckGo,
            0,
            Ok(vec![
                result(SearchBackendId::DuckDuckGo, "file:///invalid"),
                expected_first.clone(),
                result(SearchBackendId::DuckDuckGo, "/relative"),
                expected_second.clone(),
                result(SearchBackendId::DuckDuckGo, "https://example.invalid/third"),
            ]),
            &counts,
        )],
        policy(10, 1),
    )
    .expect("router");
    let (_cancel, mut cancellation) = oneshot::channel();

    let success = router
        .search(
            request(2),
            Instant::now() + Duration::from_millis(100),
            &mut cancellation,
        )
        .await
        .expect("mixed response remains usable");

    assert_eq!(success.results, vec![expected_first, expected_second]);
    assert_released(&counts, &[SearchBackendId::DuckDuckGo], 1);
}

#[tokio::test(start_paused = true)]
async fn empty_invalid_and_backend_failures_are_typed_in_exhaustion_order() {
    let counts = SharedCounts::default();
    let router = SearchRouter::new(
        vec![
            backend(SearchBackendId::DuckDuckGo, 1, Ok(Vec::new()), &counts),
            backend(
                SearchBackendId::SearXng,
                1,
                Ok(vec![result(SearchBackendId::SearXng, "file:///invalid")]),
                &counts,
            ),
            backend(
                SearchBackendId::Tavily,
                1,
                Err(SearchBackendError::NotConfigured),
                &counts,
            ),
        ],
        policy(50, 1),
    )
    .expect("router");
    let (_cancel, mut cancellation) = oneshot::channel();

    let error = router
        .search(
            request(1),
            Instant::now() + Duration::from_millis(100),
            &mut cancellation,
        )
        .await
        .expect_err("all routes unusable");

    assert_eq!(
        error,
        SearchRouterError::Exhausted(vec![
            SearchRouterFailure {
                backend: SearchBackendId::DuckDuckGo,
                kind: SearchRouterFailureKind::NoResults,
            },
            SearchRouterFailure {
                backend: SearchBackendId::SearXng,
                kind: SearchRouterFailureKind::InvalidResults,
            },
            SearchRouterFailure {
                backend: SearchBackendId::Tavily,
                kind: SearchRouterFailureKind::Backend(SearchBackendError::NotConfigured),
            },
        ])
    );
    assert_released(
        &counts,
        &[
            SearchBackendId::DuckDuckGo,
            SearchBackendId::SearXng,
            SearchBackendId::Tavily,
        ],
        1,
    );
}

#[tokio::test(start_paused = true)]
async fn no_eligible_route_is_distinct_from_exhaustion() {
    let router = SearchRouter::new(Vec::new(), policy(10, 1)).expect("empty router is valid");
    let (_cancel, mut cancellation) = oneshot::channel();

    assert_eq!(
        router
            .search(
                request(1),
                Instant::now() + Duration::from_millis(100),
                &mut cancellation,
            )
            .await,
        Err(SearchRouterError::NoEligible)
    );
}

#[tokio::test(start_paused = true)]
async fn one_enclosing_deadline_stops_late_candidates_and_releases_owned_work() {
    let counts = SharedCounts::default();
    let router = SearchRouter::new(
        vec![
            backend(
                SearchBackendId::DuckDuckGo,
                100,
                Ok(vec![result(
                    SearchBackendId::DuckDuckGo,
                    "https://example.invalid/slow",
                )]),
                &counts,
            ),
            backend(
                SearchBackendId::SearXng,
                100,
                Ok(vec![result(
                    SearchBackendId::SearXng,
                    "https://example.invalid/slow2",
                )]),
                &counts,
            ),
            backend(
                SearchBackendId::Tavily,
                0,
                Ok(vec![result(
                    SearchBackendId::Tavily,
                    "https://example.invalid/late",
                )]),
                &counts,
            ),
        ],
        policy(10, 2),
    )
    .expect("router");
    let (_cancel, mut cancellation) = oneshot::channel();
    let start = Instant::now();

    assert_eq!(
        router
            .search(
                request(1),
                start + Duration::from_millis(25),
                &mut cancellation,
            )
            .await,
        Err(SearchRouterError::Deadline)
    );
    assert_eq!(start.elapsed(), Duration::from_millis(25));
    assert_released(
        &counts,
        &[SearchBackendId::DuckDuckGo, SearchBackendId::SearXng],
        2,
    );
}

#[tokio::test(start_paused = true)]
async fn pre_and_inflight_cancellation_prevent_success_and_release_work() {
    let counts = SharedCounts::default();
    let router = SearchRouter::new(
        vec![backend(
            SearchBackendId::DuckDuckGo,
            100,
            Ok(vec![result(
                SearchBackendId::DuckDuckGo,
                "https://example.invalid/result",
            )]),
            &counts,
        )],
        policy(10, 1),
    )
    .expect("router");

    let (cancel, mut cancellation) = oneshot::channel();
    cancel.send(()).expect("receiver remains owned");
    assert_eq!(
        router
            .search(
                request(1),
                Instant::now() + Duration::from_millis(100),
                &mut cancellation,
            )
            .await,
        Err(SearchRouterError::Cancelled)
    );
    assert_released(&counts, &[], 0);

    let counts = SharedCounts::default();
    let router = SearchRouter::new(
        vec![backend(
            SearchBackendId::DuckDuckGo,
            100,
            Ok(vec![result(
                SearchBackendId::DuckDuckGo,
                "https://example.invalid/result",
            )]),
            &counts,
        )],
        policy(10, 1),
    )
    .expect("router");
    let (cancel, mut cancellation) = oneshot::channel();
    let search = router.search(
        request(1),
        Instant::now() + Duration::from_millis(200),
        &mut cancellation,
    );
    let cancel_later = async {
        sleep(Duration::from_millis(20)).await;
        cancel.send(()).expect("receiver remains owned");
    };
    let (outcome, ()) = tokio::join!(search, cancel_later);
    assert_eq!(outcome, Err(SearchRouterError::Cancelled));
    assert_released(&counts, &[SearchBackendId::DuckDuckGo], 1);
}

#[tokio::test(start_paused = true)]
async fn mismatched_backend_identity_cannot_win() {
    let counts = SharedCounts::default();
    let router = SearchRouter::new(
        vec![backend(
            SearchBackendId::DuckDuckGo,
            0,
            Ok(vec![result(
                SearchBackendId::SearXng,
                "https://example.invalid/wrong-source",
            )]),
            &counts,
        )],
        policy(10, 1),
    )
    .expect("router");
    let (_cancel, mut cancellation) = oneshot::channel();

    assert_eq!(
        router
            .search(
                request(1),
                Instant::now() + Duration::from_millis(100),
                &mut cancellation,
            )
            .await,
        Err(SearchRouterError::Exhausted(vec![SearchRouterFailure {
            backend: SearchBackendId::DuckDuckGo,
            kind: SearchRouterFailureKind::InvalidResults,
        }]))
    );
    assert_released(&counts, &[SearchBackendId::DuckDuckGo], 1);
}

#[tokio::test(start_paused = true)]
async fn dropping_router_future_drops_owned_candidate_without_synthetic_terminal_error() {
    let counts = SharedCounts::default();
    let router = SearchRouter::new(
        vec![backend(
            SearchBackendId::DuckDuckGo,
            100,
            Ok(vec![result(
                SearchBackendId::DuckDuckGo,
                "https://example.invalid/result",
            )]),
            &counts,
        )],
        policy(10, 1),
    )
    .expect("router");
    let (_cancel, mut cancellation) = oneshot::channel();
    let mut search = Box::pin(router.search(
        request(1),
        Instant::now() + Duration::from_millis(200),
        &mut cancellation,
    ));

    poll_fn(|cx| {
        assert!(search.as_mut().poll(cx).is_pending());
        Poll::Ready(())
    })
    .await;
    drop(search);

    assert_released(&counts, &[SearchBackendId::DuckDuckGo], 1);
}

#[test]
fn router_rejects_zero_bounds_and_duplicate_backend_ids() {
    assert_eq!(
        SearchRouterPolicy::new(Duration::from_millis(1), 0, 20, 512),
        Err(SearchRouterPolicyError::ZeroBound)
    );

    let counts = SharedCounts::default();
    let candidates = vec![
        backend(SearchBackendId::DuckDuckGo, 0, Ok(Vec::new()), &counts),
        backend(SearchBackendId::DuckDuckGo, 0, Ok(Vec::new()), &counts),
    ];
    assert!(matches!(
        SearchRouter::new(candidates, policy(10, 1)),
        Err(SearchRouterPolicyError::DuplicateBackend(
            SearchBackendId::DuckDuckGo
        ))
    ));
}

#[tokio::test(start_paused = true)]
async fn request_limit_must_fit_explicit_router_admission_bound() {
    let router = SearchRouter::new(Vec::new(), policy(10, 1)).expect("router");
    let (_cancel, mut cancellation) = oneshot::channel();

    assert_eq!(
        router
            .search(
                request(21),
                Instant::now() + Duration::from_millis(100),
                &mut cancellation,
            )
            .await,
        Err(SearchRouterError::InvalidRequest)
    );
}
