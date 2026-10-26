use super::*;
use crate::search_backend::transport::{TransportResponse, tests::Server};
use serde_json::json;
use std::{sync::atomic::Ordering, time::Duration};
use talos_core::tool::ToolNetworkScope;
use talos_permission::{
    GrantScope, InteractionCapability, PermissionContext, PermissionEngine, PermissionMode,
    PermissionRequest, PermissionSessionState,
};

fn limits(bytes: usize) -> TransportLimits {
    TransportLimits::new(Duration::from_secs(5), 1, bytes).expect("limits")
}
fn configuration(server: &Server) -> SearXngConfiguration {
    let mut url = server.url("/a%2Fb/");
    url.set_host(Some("127.0.0.1")).expect("fixture host");
    let mut config = SearXngConfiguration::new(url.as_str()).expect("config");
    config.scope = ToolNetworkScope::new_configuration(
        config.scope.endpoints().to_vec(),
        None,
        vec!["127.0.0.1".parse().expect("ip")],
        0,
    )
    .expect("test-only loopback");
    config
}
fn prepare(
    config: &SearXngConfiguration,
    snippets: bool,
    bytes: usize,
) -> PreparedSearXngInvocation {
    PreparedSearXngInvocation::prepare(config, "captured +中", 1, snippets, limits(bytes))
        .expect("prepare")
}
fn body() -> String {
    json!({"results":[{"title":"Title","url":"https://example.org/","content":"one"}]}).to_string()
}
fn response() -> String {
    let body = body();
    format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}
fn state() -> Arc<PermissionSessionState> {
    Arc::new(PermissionSessionState::new(PermissionEngine::new()))
}
fn authority(
    state: &Arc<PermissionSessionState>,
    object: &dyn PreparedNetworkToolInvocation,
) -> Arc<dyn ToolNetworkAuthority> {
    let input = json!({"query":"captured +中"});
    let request = PermissionRequest::native("web_search", object.permission_profile(), &input)
        .with_network_manifest(object.manifest());
    let context = PermissionContext::new(
        PermissionMode::Interactive,
        InteractionCapability::Available,
    );
    let proposal = state
        .propose(&request, &context, GrantScope::Once)
        .expect("proposal");
    let pending = state
        .approve_once(proposal, &request, &context)
        .expect("once");
    state
        .admit_network(pending, &request, &context)
        .expect("authority")
}
#[test]
fn owning_trait_object_retains_plan_identity_profile_query_and_redaction() {
    let config = SearXngConfiguration::new("https://example.org/prefix/").expect("config");
    let mut query = String::from("private query");
    let invocation = PreparedSearXngInvocation::prepare(&config, &query, 1, true, limits(1000))
        .expect("prepare");
    query.clear();
    assert_eq!(invocation.query, "private query");
    let expected = invocation.plan.manifest().clone();
    let profile = invocation.plan.permission_profile().to_vec();
    assert_eq!(
        format!("{invocation:?}"),
        "PreparedSearXngInvocation(<redacted>)"
    );
    let object: Box<dyn PreparedNetworkToolInvocation> = Box::new(invocation);
    assert_eq!(object.manifest(), &expected);
    assert_eq!(object.permission_profile(), profile);
    assert!(!format!("{:?}", object.manifest()).contains("private query"));
    let next =
        PreparedSearXngInvocation::prepare(&config, "next", 1, true, limits(1000)).expect("next");
    assert_eq!(object.manifest().scope(), next.manifest().scope());
    assert_ne!(
        object.manifest().invocation_id(),
        next.manifest().invocation_id()
    );
    for count in [0, 21] {
        assert!(
            PreparedSearXngInvocation::prepare(&config, "q", count, true, limits(1000)).is_err()
        );
    }
}
#[tokio::test]
async fn once_trait_dispatch_after_approval_wait_preserves_request_and_source() {
    for snippets in [false, true] {
        let server = Server::new(vec![response()]);
        let object: Box<dyn PreparedNetworkToolInvocation> =
            Box::new(prepare(&configuration(&server), snippets, 1_048_576));
        // Longer than execution's five seconds; real I/O begins after resuming the clock.
        tokio::time::pause();
        tokio::time::advance(Duration::from_secs(65)).await;
        tokio::time::resume();
        let state = state();
        let seal = authority(&state, object.as_ref());
        let output = object.execute(seal).await;
        assert!(!output.result.is_error, "{}", output.result.content);
        assert!(output.next_provider_parts.is_empty());
        assert_eq!(
            output.result.content,
            format!(
                "Searched: \"captured +中\"\nSource: SearXNG\nResults: 1\n\n1. Title\n   https://example.org/\n{}\n",
                if snippets { "   one\n" } else { "" }
            )
        );
        assert_eq!(server.count(), 1);
        assert!(
            server
                .request(0)
                .contains("GET /a%2Fb/search?format=json&q=captured%20%2B%E4%B8%AD HTTP/1.1")
        );
    }
}
#[tokio::test]
async fn revoked_and_mismatched_once_authorities_fail_without_connection() {
    for revoked in [false, true] {
        let server = Server::new(vec![]);
        let config = configuration(&server);
        let first: Box<dyn PreparedNetworkToolInvocation> = Box::new(prepare(&config, true, 1000));
        let second: Box<dyn PreparedNetworkToolInvocation> = Box::new(prepare(&config, true, 1000));
        let state = state();
        let seal = authority(&state, first.as_ref());
        let object = if revoked {
            state.clear().expect("revoke");
            first
        } else {
            second
        };
        let output = object.execute(seal).await;
        assert!(output.result.is_error);
        assert_eq!(output.result.content, "search_transport_unauthorized");
        assert!(output.next_provider_parts.is_empty());
        assert_eq!(server.count(), 0);
    }
}
#[tokio::test]
async fn configured_response_byte_budget_reaches_real_transport() {
    let server = Server::new(vec![response()]);
    let object: Box<dyn PreparedNetworkToolInvocation> =
        Box::new(prepare(&configuration(&server), true, 1));
    let state = state();
    let seal = authority(&state, object.as_ref());
    let output = object.execute(seal).await;
    assert!(output.result.is_error);
    assert_eq!(output.result.content, "search_transport_response_limit");
    assert!(output.next_provider_parts.is_empty());
    assert_eq!(server.count(), 1);
}
#[test]
fn final_expiry_overrides_formatted_success_and_parser_error() {
    for success in [false, true] {
        let outcome = if success {
            super::super::finish(
                TransportResponse {
                    status: 200,
                    body: body().into_bytes(),
                },
                1,
                || Ok(()),
            )
        } else {
            Err(SearXngError::Parse)
        };
        let output = finish_output(outcome, "private", true, || Err(TransportError::Deadline));
        assert!(output.result.is_error);
        assert_eq!(output.result.content, "search_transport_deadline");
        assert!(output.next_provider_parts.is_empty());
    }
    let output = finish_output(Err(SearXngError::Parse), "private", true, || Ok(()));
    assert_eq!(output.result.content, "search_searxng_parse");
}
#[tokio::test]
async fn one_dispatch_budget_expires_pending_actual_io() {
    let server = Server::new(vec![]);
    let object: Box<dyn PreparedNetworkToolInvocation> =
        Box::new(prepare(&configuration(&server), true, 1000));
    let state = state();
    let seal = authority(&state, object.as_ref());
    let task = tokio::spawn(async move { object.execute(seal).await });
    tokio::time::timeout(Duration::from_secs(3), async {
        while server.accepted.load(Ordering::Relaxed) == 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("actual connection");
    tokio::time::pause();
    tokio::time::advance(Duration::from_secs(6)).await;
    let output = task.await.expect("task");
    tokio::time::resume();
    assert!(output.result.is_error);
    assert_eq!(output.result.content, "search_transport_deadline");
    assert_eq!(server.count(), 1);
}
#[tokio::test]
async fn caller_abort_closes_the_real_pending_connection() {
    let server = Server::new(vec![]);
    let object: Box<dyn PreparedNetworkToolInvocation> =
        Box::new(prepare(&configuration(&server), true, 1000));
    let state = state();
    let seal = authority(&state, object.as_ref());
    let task = tokio::spawn(async move { object.execute(seal).await });
    tokio::time::timeout(Duration::from_secs(3), async {
        while server.accepted.load(Ordering::Relaxed) == 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("actual connection");
    task.abort();
    assert!(task.await.expect_err("aborted").is_cancelled());
    tokio::time::timeout(Duration::from_secs(3), async {
        while server.closed.load(Ordering::Relaxed) == 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("peer EOF");
    assert_eq!(server.count(), 1);
}
