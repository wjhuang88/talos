use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;

use talos_core::tool::{ToolNetworkAuthority, ToolNetworkAuthorizationError, ToolNetworkEndpoint};
use tokio::sync::oneshot;
use tokio::time::Instant;
use uuid::Uuid;

use super::*;
use crate::search_backend::composition::SearchCompositionConfiguration;
use crate::search_backend::transport::{
    TransportLimits, compatibility::searxng::SearXngConfiguration,
};

#[derive(Default)]
struct CountingRejectAuthority {
    checks: AtomicUsize,
}

impl ToolNetworkAuthority for CountingRejectAuthority {
    fn check(
        &self,
        _invocation_id: Uuid,
        _destination: &ToolNetworkEndpoint,
    ) -> Result<(), ToolNetworkAuthorizationError> {
        self.checks.fetch_add(1, Ordering::SeqCst);
        Err(ToolNetworkAuthorizationError::Mismatch)
    }
}

fn policy() -> SearchRouterPolicy {
    SearchRouterPolicy::new(Duration::from_millis(50), 1, 20, 8192).expect("policy")
}

fn limits() -> TransportLimits {
    TransportLimits::new(Duration::from_secs(5), 5, 1_048_576).expect("limits")
}

#[tokio::test(start_paused = true)]
async fn prepared_candidates_share_router_scheduler_and_budget_deadline() {
    let searxng = SearXngConfiguration::new("https://search.example/").expect("searxng");
    let configuration = SearchCompositionConfiguration::new(Some(searxng)).expect("config");
    let prepared = configuration
        .prepare("private prepared query", 2, limits())
        .expect("prepared");
    let authority = Arc::new(CountingRejectAuthority::default());
    let started = Instant::now();
    let running = prepared.start(authority.clone());

    assert_eq!(running.deadline(), started + Duration::from_secs(5));
    assert_eq!(running.requested_results(), 2);

    let (_cancel, mut cancellation) = oneshot::channel();
    let error = search_prepared(running, policy(), &mut cancellation)
        .await
        .expect_err("rejected authority exhausts prepared candidates");

    assert_eq!(
        error,
        SearchRouterError::Exhausted(vec![
            SearchRouterFailure {
                backend: SearchBackendId::DuckDuckGo,
                kind: SearchRouterFailureKind::Backend(SearchBackendError::RequestFailed(
                    "controlled search transport failed".into(),
                )),
            },
            SearchRouterFailure {
                backend: SearchBackendId::SearXng,
                kind: SearchRouterFailureKind::Backend(SearchBackendError::RequestFailed(
                    "controlled search transport failed".into(),
                )),
            },
        ])
    );
    assert_eq!(authority.checks.load(Ordering::SeqCst), 2);
}

#[tokio::test(start_paused = true)]
async fn prepared_router_honors_pre_cancellation_before_any_authority_check() {
    let configuration = SearchCompositionConfiguration::new(None).expect("config");
    let prepared = configuration
        .prepare("private prepared query", 1, limits())
        .expect("prepared");
    let authority = Arc::new(CountingRejectAuthority::default());
    let running = prepared.start(authority.clone());
    let (cancel, mut cancellation) = oneshot::channel();
    cancel.send(()).expect("receiver remains owned");

    assert_eq!(
        search_prepared(running, policy(), &mut cancellation).await,
        Err(SearchRouterError::Cancelled)
    );
    assert_eq!(authority.checks.load(Ordering::SeqCst), 0);
}
