use super::*;
use crate::search_backend::transport::{SearchDns, tests::Server};
use async_trait::async_trait;
use std::{
    net::SocketAddr,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
use talos_core::tool::{ToolNetworkAuthorizationError, ToolNetworkEndpoint};

fn prepare(query: &str) -> PreparedCompatibilityPlan {
    DirectCompatibilityConfiguration::new()
        .expect("direct configuration")
        .prepare(query, 1)
        .expect("plan")
}

fn response(body: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

fn valid() -> String {
    response(
        &String::from_utf8(super::super::execution_tests::body(
            super::super::CompatibilityProtocol::Lite,
        ))
        .expect("body"),
    )
}

pub(in crate::search_backend::transport::compatibility) struct Effects {
    manifest: ToolNetworkManifest,
    checks: AtomicUsize,
    pub(in crate::search_backend::transport::compatibility) dns: AtomicUsize,
    allowed: usize,
}

impl ToolNetworkAuthority for Effects {
    fn check(
        &self,
        invocation: uuid::Uuid,
        endpoint: &ToolNetworkEndpoint,
    ) -> Result<(), ToolNetworkAuthorizationError> {
        if self.checks.fetch_add(1, Ordering::Relaxed) >= self.allowed {
            return Err(ToolNetworkAuthorizationError::Invalidated);
        }
        if invocation != self.manifest.invocation_id() || !self.manifest.scope().contains(endpoint)
        {
            return Err(ToolNetworkAuthorizationError::Mismatch);
        }
        Ok(())
    }
}

#[async_trait]
impl SearchDns for Effects {
    async fn resolve(&self, _: &str, port: u16) -> Result<Vec<SocketAddr>, TransportError> {
        self.dns.fetch_add(1, Ordering::Relaxed);
        Ok(vec![SocketAddr::from(([127, 0, 0, 1], port))])
    }
}

// Only this cfg(test) module can relocate the fixed query chain or inject DNS. Existing plan
// fixtures predate reachability parity, so use a plan-local reachability override and keep these
// tests focused on query-chain authority/budget/drop behavior. Probe execution has its own fixtures.
pub(in crate::search_backend::transport::compatibility) fn controlled(
    mut plan: PreparedCompatibilityPlan,
    server: &Server,
    allowed: usize,
    mismatch: bool,
) -> (PreparedCompatibilityPlan, Arc<Effects>) {
    plan.reachability = Some(Reachability {
        duckduckgo: true,
        bing: true,
    });
    let (chain, manifest) = super::super::chain::tests::relocate(plan.chain, server);
    plan.chain = chain;
    plan.manifest = manifest;
    plan.facets = facets(&plan.manifest);
    let effects = Arc::new(Effects {
        manifest: if mismatch {
            ToolNetworkManifest::new(plan.manifest.scope().clone())
        } else {
            plan.manifest.clone()
        },
        checks: AtomicUsize::new(0),
        dns: AtomicUsize::new(0),
        allowed,
    });
    plan.dns = effects.clone();
    (plan, effects)
}

fn budget(requests: usize, bytes: usize) -> Arc<TransportBudget> {
    Arc::new(TransportBudget::new(Duration::from_secs(5), requests, bytes).expect("budget"))
}

fn terminal(
    protocol: super::super::CompatibilityProtocol,
    error: TransportError,
) -> CompatibilityPlanError {
    CompatibilityPlanError::Chain(CompatibilityChainError::Terminal {
        protocol,
        error: CompatibilityError::Transport(error),
    })
}

#[test]
fn configuration_scope_is_reused_but_each_plan_has_fresh_invocation_identity() {
    let config = DirectCompatibilityConfiguration::new().expect("config");
    let first = config.prepare("query-one", 1).expect("first");
    let second = config.prepare("query-two", 2).expect("second");
    assert_eq!(first.manifest().scope(), second.manifest().scope());
    assert_ne!(
        first.manifest().invocation_id(),
        second.manifest().invocation_id()
    );
    assert_eq!(first.permission_profile(), second.permission_profile());
    assert_ne!(
        first.manifest().scope(),
        prepare("query-one").manifest().scope()
    );
}

#[test]
fn manifest_and_domain_facets_are_complete_fixed_query_free_and_redacted() {
    let config = DirectCompatibilityConfiguration::new().expect("config");
    let plan = config.prepare("toy-private-query", 1).expect("plan");
    let scope = plan.manifest().scope();
    assert_eq!(scope.endpoints().len(), 9);
    for protocol in ORDER {
        assert!(
            scope
                .endpoints()
                .contains(&protocol.destination().expect("endpoint"))
        );
    }
    for endpoint in super::super::probe::destinations().expect("probe destinations") {
        assert!(scope.endpoints().contains(&endpoint));
    }
    assert!(
        scope
            .endpoints()
            .contains(&super::super::wikipedia::destination().expect("wikipedia endpoint"))
    );
    assert!(scope.proxy().is_none());
    assert!(scope.private_addresses().is_empty());
    assert_eq!(scope.redirect_limit(), 0);
    assert_eq!(scope.configuration_revision(), 0);
    let profile = plan.permission_profile();
    assert_eq!(profile.len(), 6);
    for (facet, host) in profile.iter().zip([
        "api.duckduckgo.com",
        "cn.bing.com",
        "duckduckgo.com",
        "en.wikipedia.org",
        "html.duckduckgo.com",
        "lite.duckduckgo.com",
    ]) {
        assert_eq!(facet.nature, ToolNature::Network);
        assert_eq!(facet.resource.as_deref(), Some(host));
        assert_eq!(facet.resource_kind, Some(ToolResourceKind::Domain));
        assert!(facet.description.is_none());
    }
    assert_eq!(
        format!("{config:?}"),
        "DirectCompatibilityConfiguration(<redacted>)"
    );
    assert_eq!(format!("{plan:?}"), "PreparedCompatibilityPlan(<redacted>)");
    assert!(!format!("{profile:?}{:?}", plan.manifest()).contains("toy-private-query"));
}

#[test]
fn invalid_preparation_cannot_revise_configuration_or_create_an_executable_plan() {
    let config = DirectCompatibilityConfiguration::new().expect("config");
    let scope = config.scope.clone();
    for (query, max) in [
        (" ".to_owned(), 1),
        ("q".to_owned(), 0),
        ("q".to_owned(), 21),
        ("x".repeat(4096), 1),
    ] {
        assert!(config.prepare(&query, max).is_err());
        assert_eq!(config.scope, scope);
    }
    assert_eq!(
        config.prepare("valid", 1).expect("plan").manifest().scope(),
        &scope
    );
}

#[tokio::test]
async fn owned_plan_executes_captured_query_after_configuration_and_input_are_dropped() {
    let config = DirectCompatibilityConfiguration::new().expect("config");
    let mut query = "captured +&".to_owned();
    let plan = config.prepare(&query, 1).expect("plan");
    query.clear();
    drop(config);
    let server = Server::new(vec![valid()]);
    let (plan, effects) = controlled(plan, &server, usize::MAX, false);
    let approved = plan.manifest().clone();
    let budget = budget(4, 1_048_576);
    let rows = plan
        .execute(effects.clone(), budget.clone())
        .await
        .expect("admitted")
        .into_results();
    assert_eq!(rows.len(), 1);
    assert_eq!(effects.manifest, approved);
    assert_eq!(server.count(), 1);
    assert!(server.request(0).ends_with("q=captured+%2B%26&kl=wt-wt"));
    assert_eq!(effects.dns.load(Ordering::Relaxed), 1);
    assert_eq!(budget.attempts.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn denied_or_different_invocation_authority_cannot_reach_dns_or_send() {
    for (allowed, mismatch) in [(0, false), (usize::MAX, true)] {
        let server = Server::new(vec![]);
        let (plan, effects) = controlled(prepare("q"), &server, allowed, mismatch);
        let budget = budget(4, 1_048_576);
        let error = plan
            .execute(effects.clone(), budget.clone())
            .await
            .expect_err("denied");
        assert_eq!(error, terminal(ORDER[0], TransportError::Unauthorized));
        assert_eq!(error.to_string(), "search_compatibility_plan_chain");
        assert_eq!(server.count(), 0);
        assert_eq!(effects.dns.load(Ordering::Relaxed), 0);
        assert_eq!(budget.attempts.load(Ordering::Relaxed), 0);
    }
}

#[tokio::test]
async fn later_revocation_stops_owned_plan_before_the_next_source_resolves() {
    let server = Server::new(vec![response("<html></html>"), valid()]);
    let (plan, effects) = controlled(prepare("q"), &server, 2, false);
    assert_eq!(
        plan.execute(effects.clone(), budget(4, 1_048_576))
            .await
            .expect_err("revoked"),
        terminal(ORDER[1], TransportError::Unauthorized)
    );
    assert_eq!(server.count(), 1);
    assert_eq!(effects.dns.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn preconsumed_budget_is_reused_without_reset_at_plan_dispatch() {
    let server = Server::new(vec![valid()]);
    let (plan, effects) = controlled(prepare("q"), &server, usize::MAX, false);
    let budget = budget(1, 1_048_576);
    TransportBudget::reserve(&budget.attempts, 1, 1, TransportError::RequestLimit)
        .expect("previous work");
    assert_eq!(
        plan.execute(effects.clone(), budget.clone())
            .await
            .expect_err("shared budget spent"),
        terminal(ORDER[0], TransportError::RequestLimit)
    );
    assert_eq!(budget.attempts.load(Ordering::Relaxed), 1);
    assert_eq!(effects.dns.load(Ordering::Relaxed), 0);
    assert_eq!(server.count(), 0);
}

#[tokio::test]
async fn already_expired_budget_cannot_be_restarted_by_plan_dispatch() {
    let server = Server::new(vec![]);
    let (plan, effects) = controlled(prepare("q"), &server, usize::MAX, false);
    let budget = budget(4, 1_048_576);
    tokio::time::pause();
    tokio::time::advance(Duration::from_secs(5)).await;
    let outcome = plan.execute(effects.clone(), budget.clone()).await;
    tokio::time::resume();
    assert_eq!(
        outcome.expect_err("expired"),
        terminal(ORDER[0], TransportError::Deadline)
    );
    assert_eq!(effects.checks.load(Ordering::Relaxed), 0);
    assert_eq!(effects.dns.load(Ordering::Relaxed), 0);
    assert_eq!(budget.attempts.load(Ordering::Relaxed), 0);
    assert_eq!(server.count(), 0);
}

#[tokio::test]
async fn dropping_plan_execution_closes_actual_peer_without_starting_later_sources() {
    let server = Server::new(vec![]);
    let (plan, effects) = controlled(prepare("q"), &server, usize::MAX, false);
    let observed = effects.clone();
    let task = tokio::spawn(async move { plan.execute(effects, budget(4, 1_048_576)).await });
    tokio::time::timeout(Duration::from_secs(3), async {
        while server.accepted.load(Ordering::Relaxed) == 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("actual connection");
    task.abort();
    assert!(task.await.expect_err("abort").is_cancelled());
    tokio::time::timeout(Duration::from_secs(3), async {
        while server.closed.load(Ordering::Relaxed) == 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("actual EOF");
    assert_eq!(observed.dns.load(Ordering::Relaxed), 1);
    assert_eq!(server.count(), 1);
}
