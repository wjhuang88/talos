use super::*;
use crate::search_backend::transport::{
    compatibility::plan::tests::{Effects, controlled},
    tests::Server,
};
use serde_json::{Value, json};
use std::{sync::atomic::Ordering, time::Duration};
use talos_core::tool::ToolNetworkAuthorizationError;
use talos_permission::{
    GrantScope, GrantSource, InteractionCapability, PermissionContext, PermissionDecision,
    PermissionEngine, PermissionMode, PermissionRequest, PermissionRule, PermissionSessionState,
    ResourceKind,
};

fn context() -> PermissionContext {
    PermissionContext::new(
        PermissionMode::Interactive,
        InteractionCapability::Available,
    )
}

fn limits(requests: usize) -> TransportLimits {
    TransportLimits::new(Duration::from_secs(5), requests, 1_048_576).expect("limits")
}

fn prepare(query: &str, snippets: bool, requests: usize) -> PreparedCompatibilityInvocation {
    PreparedCompatibilityInvocation::prepare(
        &DirectCompatibilityConfiguration::new().expect("config"),
        query,
        1,
        snippets,
        limits(requests),
    )
    .expect("invocation")
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

fn local(
    mut invocation: PreparedCompatibilityInvocation,
    server: &Server,
) -> (Box<dyn PreparedNetworkToolInvocation>, Arc<Effects>) {
    let (plan, effects) = controlled(invocation.plan, server, usize::MAX, false);
    invocation.plan = plan;
    (Box::new(invocation), effects)
}

fn admit(
    state: &Arc<PermissionSessionState>,
    invocation: &dyn PreparedNetworkToolInvocation,
    input: &Value,
    scope: GrantScope,
) -> Arc<dyn ToolNetworkAuthority> {
    let request = PermissionRequest::native("web_search", invocation.permission_profile(), input)
        .with_network_manifest(invocation.manifest());
    let proposal = state
        .propose(&request, &context(), scope)
        .expect("proposal");
    assert_eq!(
        proposal.preview().network_scope(),
        Some(invocation.manifest().scope())
    );
    let pending = match scope {
        GrantScope::Once => state
            .approve_once(proposal, &request, &context())
            .expect("once"),
        GrantScope::Session => state
            .approve_session(proposal, &request, &context(), GrantSource::SdkHostApproval)
            .expect("session"),
    };
    state
        .admit_network(pending, &request, &context())
        .expect("live authority")
}

#[test]
fn validated_limits_preserve_existing_bounds_without_a_running_budget() {
    for (duration, requests, bytes) in [
        (Duration::ZERO, 1, 1),
        (Duration::from_secs(121), 1, 1),
        (Duration::from_secs(1), 0, 1),
        (Duration::from_secs(1), 17, 1),
        (Duration::from_secs(1), 1, 0),
        (Duration::from_secs(1), 1, 1_048_577),
    ] {
        assert!(TransportLimits::new(duration, requests, bytes).is_err());
    }
    assert!(TransportLimits::new(Duration::from_secs(120), 16, 1_048_576).is_ok());
}

#[test]
fn trait_object_retains_exact_manifest_profile_and_redacts_captured_input() {
    let mut query = "private-query".to_owned();
    let invocation = prepare(&query, false, 4);
    query.clear();
    assert_eq!(
        format!("{invocation:?}"),
        "PreparedCompatibilityInvocation(<redacted>)"
    );
    let manifest = invocation.plan.manifest().clone();
    let profile = invocation.plan.permission_profile().to_vec();
    let object: Box<dyn PreparedNetworkToolInvocation> = Box::new(invocation);
    assert_eq!(object.manifest(), &manifest);
    assert_eq!(object.permission_profile(), profile);
    assert_eq!(profile.len(), 4);
}

#[tokio::test]
async fn real_once_approval_executes_owned_trait_object_after_long_preparation_wait() {
    for snippets in [false, true] {
        let server = Server::new(vec![valid()]);
        let (object, effects) = local(prepare("captured +&", snippets, 4), &server);
        let input = json!({"query":"captured +&", "max_results":1,"include_snippets":snippets});
        let state = Arc::new(PermissionSessionState::new(PermissionEngine::new()));
        // Simulate a wait longer than the execution budget, then resume before real socket I/O.
        tokio::time::pause();
        tokio::time::advance(Duration::from_secs(65)).await;
        tokio::time::resume();
        let authority = admit(&state, object.as_ref(), &input, GrantScope::Once);
        let output = object.execute(authority).await;
        assert!(!output.result.is_error);
        assert!(output.next_provider_parts.is_empty());
        assert_eq!(
            output.result.content,
            format!(
                "Searched: \"captured +&\"\nSource: DuckDuckGo\nResults: 1\n\n1. First\n   https://first.invalid/\n{}\n",
                if snippets { "   one\n" } else { "" },
            )
        );
        assert_eq!(effects.dns.load(Ordering::Relaxed), 1);
        assert_eq!(server.count(), 1);
        assert!(server.request(0).ends_with("q=captured+%2B%26&kl=wt-wt"));
    }
}

#[tokio::test]
async fn cleared_real_authority_returns_fixed_error_before_dns_or_send() {
    let server = Server::new(vec![]);
    let (object, effects) = local(prepare("private-query", true, 4), &server);
    let state = Arc::new(PermissionSessionState::new(PermissionEngine::new()));
    let authority = admit(
        &state,
        object.as_ref(),
        &json!({"query":"private-query"}),
        GrantScope::Once,
    );
    state.clear().expect("revoke");
    let output = object.execute(authority).await;
    assert!(output.result.is_error);
    assert_eq!(output.result.content, "search_compatibility_plan_chain");
    assert!(output.next_provider_parts.is_empty());
    assert_eq!(effects.dns.load(Ordering::Relaxed), 0);
    assert_eq!(server.count(), 0);
}

#[tokio::test]
async fn another_invocations_real_authority_cannot_execute_the_trait_object() {
    let server = Server::new(vec![]);
    let (first, _) = local(prepare("first", true, 4), &server);
    let (second, effects) = local(prepare("second-private", true, 4), &server);
    let state = Arc::new(PermissionSessionState::new(PermissionEngine::new()));
    let authority = admit(
        &state,
        first.as_ref(),
        &json!({"query":"first"}),
        GrantScope::Once,
    );
    let output = second.execute(authority).await;
    assert!(output.result.is_error);
    assert_eq!(output.result.content, "search_compatibility_plan_chain");
    assert_eq!(effects.dns.load(Ordering::Relaxed), 0);
    assert_eq!(server.count(), 0);
}

#[test]
fn real_session_grant_reuses_configuration_but_mints_fresh_invocation_authority() {
    let config = DirectCompatibilityConfiguration::new().expect("config");
    let first = PreparedCompatibilityInvocation::prepare(&config, "one", 1, true, limits(4))
        .expect("first");
    let second = PreparedCompatibilityInvocation::prepare(&config, "two", 2, false, limits(4))
        .expect("second");
    let state = Arc::new(PermissionSessionState::new(PermissionEngine::new()));
    let first_authority = admit(&state, &first, &json!({"query":"one"}), GrantScope::Session);
    let input = json!({"query":"two"});
    let request = PermissionRequest::native("web_search", second.permission_profile(), &input)
        .with_network_manifest(second.manifest());
    let pending = state
        .prepare_authorized(&request, &context())
        .expect("lookup")
        .expect("matching scope");
    let second_authority = state
        .admit_network(pending, &request, &context())
        .expect("fresh admission");
    for endpoint in second.manifest().scope().endpoints() {
        second_authority
            .check(second.manifest().invocation_id(), endpoint)
            .expect("own invocation");
        assert_eq!(
            first_authority.check(second.manifest().invocation_id(), endpoint),
            Err(ToolNetworkAuthorizationError::Mismatch)
        );
    }
    assert_eq!(state.grant_count().expect("count"), 1);
}

#[test]
fn denied_fallback_in_trait_profile_prevents_whole_plan_approval_and_facet_stripping() {
    let object = prepare("private", true, 4);
    let allow = PermissionRule::new_nature(
        talos_core::tool::ToolNature::Network,
        None,
        None,
        PermissionDecision::Allow,
    );
    let deny = PermissionRule::new_nature(
        talos_core::tool::ToolNature::Network,
        Some("cn.bing.com".into()),
        Some(ResourceKind::Domain),
        PermissionDecision::Deny("fixture".into()),
    );
    let state = PermissionSessionState::new(PermissionEngine::from_rules(vec![allow, deny]));
    let input = json!({"query":"private"});
    let request = PermissionRequest::native("web_search", object.permission_profile(), &input)
        .with_network_manifest(object.manifest());
    assert!(matches!(
        state
            .evaluate(&request, &context())
            .expect("evaluate")
            .decision(),
        PermissionDecision::Deny(_)
    ));
    assert!(
        state
            .propose(&request, &context(), GrantScope::Once)
            .is_err()
    );
    let incomplete: Vec<_> = object
        .permission_profile()
        .iter()
        .filter(|f| f.resource.as_deref() != Some("cn.bing.com"))
        .cloned()
        .collect();
    let stripped = PermissionRequest::native("web_search", &incomplete, &input)
        .with_network_manifest(object.manifest());
    assert!(state.begin_invocation(&stripped, &context()).is_err());
}

#[tokio::test]
async fn one_dispatch_budget_is_shared_by_all_sources_through_real_authority() {
    let server = Server::new(vec![response("<html></html>"), valid()]);
    let (object, effects) = local(prepare("private", true, 1), &server);
    let state = Arc::new(PermissionSessionState::new(PermissionEngine::new()));
    let authority = admit(
        &state,
        object.as_ref(),
        &json!({"query":"private"}),
        GrantScope::Once,
    );
    let output = object.execute(authority).await;
    assert!(output.result.is_error);
    assert_eq!(output.result.content, "search_compatibility_plan_chain");
    assert_eq!(effects.dns.load(Ordering::Relaxed), 1);
    assert_eq!(server.count(), 1);
}

#[tokio::test]
async fn caller_abort_of_admitted_trait_object_closes_actual_peer() {
    let server = Server::new(vec![]);
    let (object, effects) = local(prepare("private", true, 4), &server);
    let state = Arc::new(PermissionSessionState::new(PermissionEngine::new()));
    let authority = admit(
        &state,
        object.as_ref(),
        &json!({"query":"private"}),
        GrantScope::Once,
    );
    let task = tokio::spawn(async move { object.execute(authority).await });
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
    .expect("actual peer EOF");
    assert_eq!(effects.dns.load(Ordering::Relaxed), 1);
    assert_eq!(server.count(), 1);
}
