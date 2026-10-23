use super::*;
use crate::search_backend::transport::tests::Server;
use serde_json::json;
use std::time::Duration;
use talos_permission::{
    GrantScope, InteractionCapability, PermissionContext, PermissionEngine, PermissionMode,
    PermissionRequest, PermissionSessionState,
};

fn response(value: serde_json::Value) -> TransportResponse {
    TransportResponse {
        status: 200,
        body: serde_json::to_vec(&value).expect("json"),
    }
}
fn row() -> serde_json::Value {
    json!({"url":"https://example.org/a", "content":"content", "snippet":"fallback"})
}
fn budget() -> Arc<TransportBudget> {
    Arc::new(TransportBudget::new(Duration::from_secs(5), 1, 1_048_576).expect("budget"))
}
fn authority(
    state: &Arc<PermissionSessionState>,
    plan: &PreparedSearXng,
) -> Arc<dyn ToolNetworkAuthority> {
    let input = json!({});
    let request = PermissionRequest::native("web_search", plan.permission_profile(), &input)
        .with_network_manifest(plan.manifest());
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
fn local(server: &Server) -> SearXngConfiguration {
    let mut url = server.url("/a%2Fb/%E4%B8%AD/");
    url.set_host(Some("127.0.0.1")).expect("fixture host");
    let mut config = SearXngConfiguration::new(url.as_str()).expect("config");
    config.scope = ToolNetworkScope::new_configuration(
        config.scope.endpoints().to_vec(),
        None,
        vec!["127.0.0.1".parse().expect("ip")],
        0,
    )
    .expect("test-only loopback scope");
    config
}
#[test]
fn explicit_prefix_and_exact_query_are_owned_and_redacted() {
    let config = SearXngConfiguration::new("https://search.example/prefix/").expect("config");
    let first = config.prepare("a b+~中&", 2).expect("first");
    let second = config.prepare("other", 2).expect("second");
    assert_eq!(
        first.request.url.as_str(),
        "https://search.example/prefix/search?format=json&q=a%20b%2B~%E4%B8%AD%26"
    );
    assert_eq!(first.manifest.scope(), second.manifest.scope());
    assert_ne!(first.manifest, second.manifest);
    assert!(!format!("{:?}", first.manifest).contains("a%20"));
    assert_eq!(format!("{first:?}"), "PreparedSearXng(<redacted>)");
    assert_eq!(format!("{config:?}"), "SearXngConfiguration(<redacted>)");
    assert_eq!(first.permission_profile().len(), 1);
    let decoded: Vec<_> = first.request.url.query_pairs().collect();
    assert_eq!(decoded[1].1, "a b+~中&");
}
#[test]
fn configuration_and_request_validation_have_no_effects() {
    for base in [
        "ftp://example.org",
        "https://user@example.org",
        "https://example.org/?x",
        "https://example.org/#x",
        "https://exam\nple.org",
        " https://example.org",
        "",
        "https://example.org:bad",
    ] {
        assert!(SearXngConfiguration::new(base).is_err(), "{base:?}");
    }
    let config = SearXngConfiguration::new("https://example.org").expect("config");
    for query in ["", " \n", &"x".repeat(4096), &"中".repeat(2000)] {
        assert!(config.prepare(query, 1).is_err());
    }
    for count in [0, 21, u32::MAX] {
        assert!(config.prepare("q", count).is_err());
    }
    assert!(config.scope.private_addresses().is_empty());
}
#[test]
fn parser_preserves_source_defaults_and_snippet_precedence() {
    let result = finish(
        response(json!({"results":[row(), {"url":"https://example.org/b", "snippet":"fallback"}]})),
        2,
        || Ok(()),
    )
    .expect("admitted")
    .into_results();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].backend, SearchBackendId::SearXng);
    assert_eq!(result[0].title, "Untitled");
    assert_eq!(result[0].snippet, "content");
    assert_eq!(result[1].snippet, "fallback");
}
#[test]
fn response_limits_and_invalid_results_are_closed_errors() {
    for value in [
        json!({}),
        json!({"results":null}),
        json!({"results":[]}),
        json!({"results":[{"url":"file:///tmp/x"}]}),
        json!({"results":[{"url":"https://u:p@example.org"}]}),
    ] {
        assert!(finish(response(value), 1, || Ok(())).is_err());
    }
    assert_eq!(
        parse(response(json!({"results":vec![row();21]}))).expect_err("limit"),
        SearXngError::Limit
    );
    let huge = json!({"url":"https://example.org", "title":"x".repeat(16385)});
    assert!(finish(response(json!({"results":[row(), huge]})), 1, || Ok(())).is_err());
    assert_eq!(
        parse(TransportResponse {
            status: 503,
            body: vec![]
        })
        .expect_err("status"),
        SearXngError::Status
    );
    assert_eq!(
        parse(TransportResponse {
            status: 200,
            body: b"{".to_vec()
        })
        .expect_err("parse"),
        SearXngError::Parse
    );
    assert_eq!(
        parse(TransportResponse {
            status: 200,
            body: vec![b' '; 1_048_577]
        })
        .expect_err("limit"),
        SearXngError::Limit
    );
}
#[test]
fn expiry_overrides_parser_and_admission_outcomes() {
    for value in [
        json!({}),
        json!({"results":[row()]}),
        json!({"results":[{"url":"https://example.org", "title":"x".repeat(16385)}]}),
    ] {
        for expire_at in 1..=12 {
            let mut calls = 0;
            let outcome = finish(response(value.clone()), 1, || {
                calls += 1;
                if calls >= expire_at {
                    Err(TransportError::Deadline)
                } else {
                    Ok(())
                }
            });
            if calls >= expire_at {
                assert!(matches!(
                    outcome,
                    Err(SearXngError::Transport(TransportError::Deadline))
                        | Err(SearXngError::Admission(CompatibilityError::Transport(
                            TransportError::Deadline
                        )))
                ));
            }
        }
    }
}
#[tokio::test]
async fn actual_once_authority_executes_exact_owned_request() {
    let body = json!({"results":[row()]}).to_string();
    let server = Server::new(vec![format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )]);
    let plan = local(&server).prepare("private query", 1).expect("plan");
    let state = Arc::new(PermissionSessionState::new(PermissionEngine::new()));
    let authority = authority(&state, &plan);
    let results = plan
        .execute(authority, budget())
        .await
        .expect("execute")
        .into_results();
    assert_eq!(results[0].backend, SearchBackendId::SearXng);
    assert_eq!(server.count(), 1);
    assert!(
        server
            .request(0)
            .contains("GET /a%2Fb/%E4%B8%AD/search?format=json&q=private%20query HTTP/1.1")
    );
}
#[tokio::test]
async fn mismatched_authority_cannot_send_and_private_address_is_not_implicitly_allowed() {
    let server = Server::new(vec![]);
    let config = local(&server);
    let first = config.prepare("q", 1).expect("first");
    let second = config.prepare("q", 1).expect("second");
    let state = Arc::new(PermissionSessionState::new(PermissionEngine::new()));
    let seal = authority(&state, &first);
    assert!(matches!(
        second.execute(seal, budget()).await,
        Err(SearXngError::Transport(TransportError::Unauthorized))
    ));
    let mut url = server.url("/");
    url.set_host(Some("127.0.0.1")).expect("fixture host");
    let config = SearXngConfiguration::new(url.as_str()).expect("config");
    let plan = config.prepare("q", 1).expect("plan");
    let seal = authority(&state, &plan);
    assert!(matches!(
        plan.execute(seal, budget()).await,
        Err(SearXngError::Transport(TransportError::Address))
    ));
    assert_eq!(server.count(), 0);
}

#[tokio::test]
async fn revoked_authority_and_expired_or_exhausted_shared_budget_send_nothing() {
    let server = Server::new(vec![]);
    let config = local(&server);
    for mode in 0..3 {
        let plan = config.prepare("q", 1).expect("plan");
        let state = Arc::new(PermissionSessionState::new(PermissionEngine::new()));
        let seal = authority(&state, &plan);
        let mut shared =
            TransportBudget::new(Duration::from_secs(5), 1, 1_048_576).expect("budget");
        let expected = match mode {
            0 => {
                state.clear().expect("revoke");
                TransportError::Unauthorized
            }
            1 => {
                shared.deadline = tokio::time::Instant::now();
                TransportError::Deadline
            }
            _ => {
                shared
                    .attempts
                    .store(1, std::sync::atomic::Ordering::Relaxed);
                TransportError::RequestLimit
            }
        };
        assert!(
            matches!(plan.execute(seal, Arc::new(shared)).await, Err(SearXngError::Transport(error)) if error == expected)
        );
    }
    assert_eq!(server.count(), 0);
}

#[test]
fn encoded_prefix_is_retained_without_double_encoding() {
    let config =
        SearXngConfiguration::new("https://search.example/a%2Fb/%E4%B8%AD/").expect("config");
    let plan = config.prepare("q", 1).expect("plan");
    assert_eq!(plan.request.url.path(), "/a%2Fb/%E4%B8%AD/search");
    assert_eq!(
        plan.manifest.scope().endpoints()[0].path(),
        "/a%2Fb/%E4%B8%AD/search"
    );
}
