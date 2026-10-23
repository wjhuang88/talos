use super::*;
use std::sync::Mutex;
use talos_core::tool::{ToolNature, ToolPermissionFacet, ToolResourceKind};
use talos_permission::{
    GrantScope, InteractionCapability, PermissionContext, PermissionEngine, PermissionMode,
    PermissionRequest, PermissionSessionState,
};

#[derive(Clone, Copy)]
enum AfterFailure {
    Continue,
    Revoke,
    Expire,
    DenyAddress,
}

struct Resolver {
    hosts: Mutex<Vec<String>>,
    failures: Vec<usize>,
    after: AfterFailure,
    state: Arc<PermissionSessionState>,
}

#[async_trait]
impl SearchDns for Resolver {
    async fn resolve(&self, host: &str, port: u16) -> Result<Vec<SocketAddr>, TransportError> {
        let index = {
            let mut hosts = self.hosts.lock().expect("observations");
            let index = hosts.len();
            hosts.push(host.to_owned());
            index
        };
        if self.failures.contains(&index) {
            match self.after {
                AfterFailure::Revoke => self.state.clear().expect("revoke admitted grant"),
                AfterFailure::Expire => tokio::time::advance(Duration::from_secs(5)).await,
                AfterFailure::Continue | AfterFailure::DenyAddress => {}
            }
            return Err(TransportError::Dns);
        }
        let ip = if matches!(self.after, AfterFailure::DenyAddress) {
            [10, 0, 0, 1]
        } else {
            [127, 0, 0, 1]
        };
        Ok(vec![SocketAddr::from((ip, port))])
    }
}

fn setup(
    server: &Server,
    failures: Vec<usize>,
    requests: usize,
    after: AfterFailure,
) -> (
    PreparedCompatibilityChain,
    SearchTransport,
    Arc<Resolver>,
    Arc<TransportBudget>,
) {
    setup_with_ports(server, failures, requests, after, None)
}

fn setup_with_ports(
    server: &Server,
    failures: Vec<usize>,
    requests: usize,
    after: AfterFailure,
    ports: Option<[u16; 4]>,
) -> (
    PreparedCompatibilityChain,
    SearchTransport,
    Arc<Resolver>,
    Arc<TransportBudget>,
) {
    let (chain, manifest) = relocate_with_ports(prepare(), server, true, ports);
    let profile: Vec<_> = manifest
        .scope()
        .endpoints()
        .iter()
        .map(|endpoint| {
            ToolPermissionFacet::with_resource(
                ToolNature::Network,
                endpoint.host(),
                ToolResourceKind::Domain,
            )
        })
        .collect();
    let input = serde_json::json!({"query":" 中 +&~ ", "max_results":1});
    let state = Arc::new(PermissionSessionState::new(PermissionEngine::new()));
    let context = PermissionContext::new(
        PermissionMode::Interactive,
        InteractionCapability::Available,
    );
    let request =
        PermissionRequest::native("web_search", &profile, &input).with_network_manifest(&manifest);
    let proposal = state
        .propose(&request, &context, GrantScope::Once)
        .expect("whole-plan proposal");
    let pending = state
        .approve_once(proposal, &request, &context)
        .expect("once");
    let authority = state
        .admit_network(pending, &request, &context)
        .expect("live authority");
    let resolver = Arc::new(Resolver {
        hosts: Mutex::new(vec![]),
        failures,
        after,
        state,
    });
    let budget = Arc::new(
        TransportBudget::new(Duration::from_secs(5), requests, 1_048_576).expect("budget"),
    );
    let transport = SearchTransport::new(manifest, authority, budget.clone(), resolver.clone())
        .expect("transport");
    (chain, transport, resolver, budget)
}

mod connection_tests;

fn hosts(resolver: &Resolver) -> Vec<String> {
    resolver.hosts.lock().expect("observations").clone()
}

#[tokio::test]
async fn first_dns_failure_reaches_only_next_admitted_source_without_replay() {
    let server = Server::new(vec![valid(ORDER[1])]);
    let (chain, transport, resolver, budget) = setup(&server, vec![0], 4, AfterFailure::Continue);
    let expected_ua = chain.sources[1]
        .request
        .headers
        .expect("snapshot")
        .user_agent();
    let results = chain
        .execute(&transport)
        .await
        .expect("later admitted winner")
        .into_results();
    assert_eq!(results.len(), 1);
    assert!(results[0].url.starts_with("https://duckduckgo.com/"));
    assert_eq!(hosts(&resolver), ["source-0.invalid", "source-1.invalid"]);
    assert_eq!(budget.attempts.load(Ordering::Relaxed), 2);
    assert_eq!(server.count(), 1);
    let wire = server.request(0);
    assert!(wire.starts_with("GET /?q="));
    assert!(wire.contains("%E4%B8%AD+%2B%26%7E+"));
    assert!(
        wire.to_ascii_lowercase()
            .contains("host: source-1.invalid:")
    );
    assert!(
        wire.lines()
            .any(|line| line.eq_ignore_ascii_case(&format!("user-agent: {expected_ua}")))
    );
}

#[tokio::test]
async fn all_dns_failures_exhaust_exactly_four_prepared_sources_without_http() {
    let server = Server::new(vec![]);
    let (chain, transport, resolver, budget) =
        setup(&server, vec![0, 1, 2, 3], 4, AfterFailure::Continue);
    let error = chain
        .execute(&transport)
        .await
        .expect_err("finite exhaustion");
    assert_eq!(
        error,
        CompatibilityChainError::Exhausted([CompatibilityError::Transport(TransportError::Dns); 4])
    );
    assert_eq!(error.to_string(), "search_compatibility_chain_exhausted");
    assert_eq!(
        hosts(&resolver),
        [
            "source-0.invalid",
            "source-1.invalid",
            "source-2.invalid",
            "source-3.invalid"
        ]
    );
    assert_eq!(budget.attempts.load(Ordering::Relaxed), 4);
    assert_eq!(budget.bytes.load(Ordering::Relaxed), 0);
    assert_eq!(server.count(), 0);
}

#[tokio::test]
async fn dns_failure_does_not_bypass_revocation_spent_budget_or_denied_addresses() {
    for (after, requests, expected, lookups, attempts) in [
        (AfterFailure::Revoke, 4, TransportError::Unauthorized, 1, 1),
        (
            AfterFailure::Continue,
            1,
            TransportError::RequestLimit,
            1,
            1,
        ),
        (AfterFailure::DenyAddress, 4, TransportError::Address, 2, 2),
    ] {
        let server = Server::new(vec![]);
        let (chain, transport, resolver, budget) = setup(&server, vec![0], requests, after);
        assert_eq!(
            chain
                .execute(&transport)
                .await
                .expect_err("terminal boundary"),
            CompatibilityChainError::Terminal {
                protocol: ORDER[1],
                error: CompatibilityError::Transport(expected)
            }
        );
        assert_eq!(hosts(&resolver).len(), lookups);
        assert_eq!(budget.attempts.load(Ordering::Relaxed), attempts);
        assert_eq!(budget.bytes.load(Ordering::Relaxed), 0);
        assert_eq!(server.count(), 0);
    }
}

#[tokio::test]
async fn expired_dns_failure_cannot_launch_another_source() {
    let server = Server::new(vec![]);
    let (chain, transport, resolver, budget) = setup(&server, vec![0], 4, AfterFailure::Expire);
    tokio::time::pause();
    let result = chain.execute(&transport).await;
    tokio::time::resume();
    assert_eq!(
        result.expect_err("enclosing deadline wins"),
        CompatibilityChainError::Terminal {
            protocol: ORDER[0],
            error: CompatibilityError::Transport(TransportError::Deadline)
        }
    );
    assert_eq!(hosts(&resolver), ["source-0.invalid"]);
    assert_eq!(budget.attempts.load(Ordering::Relaxed), 1);
    assert_eq!(server.count(), 0);
}

#[tokio::test]
async fn semantic_and_dns_failures_share_existing_bytes_and_attempts() {
    let first = empty(ORDER[0]);
    let winner = valid(ORDER[2]);
    let server = Server::new(vec![first, winner]);
    let (chain, transport, resolver, budget) = setup(&server, vec![1], 4, AfterFailure::Continue);
    let rows = chain
        .execute(&transport)
        .await
        .expect("third source wins")
        .into_results();
    assert_eq!(rows[0].url, "https://first.invalid/");
    assert_eq!(
        hosts(&resolver),
        ["source-0.invalid", "source-1.invalid", "source-2.invalid"]
    );
    assert_eq!(budget.attempts.load(Ordering::Relaxed), 3);
    assert_eq!(
        budget.bytes.load(Ordering::Relaxed),
        "<html></html>".len() + super::super::super::execution_tests::body(ORDER[2]).len()
    );
    assert_eq!(server.count(), 2);
    assert!(server.request(0).starts_with("POST /lite/"));
    assert!(server.request(1).starts_with("GET /html/?q="));
}

#[tokio::test]
async fn actual_http_protocol_failure_remains_terminal_without_later_source() {
    let server = Server::new(vec!["not an HTTP response\r\n\r\n".into(), valid(ORDER[1])]);
    let (chain, transport, resolver, budget) = setup(&server, vec![], 4, AfterFailure::Continue);
    assert_eq!(
        chain
            .execute(&transport)
            .await
            .expect_err("coarse Request stays terminal"),
        CompatibilityChainError::Terminal {
            protocol: ORDER[0],
            error: CompatibilityError::Transport(TransportError::Request)
        }
    );
    assert_eq!(hosts(&resolver), ["source-0.invalid"]);
    assert_eq!(budget.attempts.load(Ordering::Relaxed), 1);
    assert_eq!(server.count(), 1);
}
