use super::*;
use crate::search_backend::transport::tests::Server;
use serde_json::json;
use std::{sync::Arc, time::Duration};
use talos_core::tool::{
    ToolAuthorizationScope, ToolExecutionAuthorization, ToolNature, ToolNetworkScope,
};
use talos_permission::{
    GrantScope, InteractionCapability, PermissionContext, PermissionEngine, PermissionMode,
    PermissionRequest, PermissionSessionState,
};

fn tool() -> GuardedSearXngSearchTool {
    configured(SearXngConfiguration::new("https://example.org/prefix/").expect("config"))
}
fn configured(configuration: SearXngConfiguration) -> GuardedSearXngSearchTool {
    GuardedSearXngSearchTool::new(
        configuration,
        TransportLimits::new(Duration::from_secs(5), 1, 1_048_576).expect("limits"),
    )
}
fn local_tool(server: &Server) -> GuardedSearXngSearchTool {
    let mut url = server.url("/a%2Fb/");
    url.set_host(Some("127.0.0.1")).expect("host");
    let mut config = SearXngConfiguration::new(url.as_str()).expect("config");
    config.scope = ToolNetworkScope::new_configuration(
        config.scope.endpoints().to_vec(),
        None,
        vec!["127.0.0.1".parse().expect("ip")],
        0,
    )
    .expect("test-only loopback");
    configured(config)
}
fn authority(
    state: &Arc<PermissionSessionState>,
    object: &dyn PreparedNetworkToolInvocation,
    input: &serde_json::Value,
) -> Arc<dyn talos_core::tool::ToolNetworkAuthority> {
    let context = PermissionContext::new(
        PermissionMode::Interactive,
        InteractionCapability::Available,
    );
    let request = PermissionRequest::native("web_search", object.permission_profile(), input)
        .with_network_manifest(object.manifest());
    let proposal = state
        .propose(&request, &context, GrantScope::Once)
        .expect("proposal");
    let pending = state
        .approve_once(proposal, &request, &context)
        .expect("once");
    state
        .admit_network(pending, &request, &context)
        .expect("admit")
}
#[test]
fn tool_contract_preserves_schema_and_requires_preparation() {
    let tool = tool();
    let object: &dyn AgentTool = &tool;
    assert_eq!(object.name(), "web_search");
    assert_eq!(
        object.parameters(),
        crate::web_search::WebSearchTool::new().parameters()
    );
    assert_eq!(object.nature(), ToolNature::Network);
    assert_eq!(object.family(), ToolFamily::Network);
    assert_eq!(object.summary_fields(), &["query"]);
    assert!(!object.is_read_only());
    assert!(object.requires_network_invocation());
    assert!(!object.requires_original_invocation());
    assert_eq!(format!("{tool:?}"), "GuardedSearXngSearchTool(<redacted>)");
    assert!(object.description().contains("SearXNG"));
    // A hidden opt-in wrapper retains this untyped profile; it cannot mint network authority.
    assert!(
        object.permission_profile(&json!({"query":"q"}))[0]
            .resource
            .is_none()
    );
}

#[test]
fn preparation_reuses_configuration_but_never_an_invocation() {
    let tool = tool();
    let input = json!({"query":"private-one"});
    let first = tool
        .prepare_network_invocation(&input)
        .expect("prepare")
        .expect("owned");
    let second = tool
        .prepare_network_invocation(&json!({"query":"private-two"}))
        .expect("prepare")
        .expect("owned");
    assert_eq!(first.manifest().scope(), second.manifest().scope());
    assert_ne!(
        first.manifest().invocation_id(),
        second.manifest().invocation_id()
    );
    assert_eq!(first.permission_profile(), second.permission_profile());
    assert_eq!(first.permission_profile().len(), 1);
    assert_eq!(first.manifest().scope().endpoints().len(), 1);
    let replacement = self::tool()
        .prepare_network_invocation(&input)
        .expect("prepare")
        .expect("owned");
    assert_ne!(first.manifest().scope(), replacement.manifest().scope());
    assert!(!format!("{:?}", first.manifest()).contains("private-one"));
}

#[test]
fn defaults_and_clamp_match_existing_input_semantics() {
    let tool = tool();
    let default = tool.prepare(&json!({"query":"q"})).expect("defaults");
    assert!(default.include_snippets);
    assert_eq!(default.plan.max_results, 10);
    for count in [0, 1, 20, 21, u32::MAX] {
        let invocation = tool
            .prepare(&json!({"query":"q", "max_results":count, "include_snippets":false}))
            .expect("existing clamp");
        assert!(!invocation.include_snippets);
        assert_eq!(invocation.plan.max_results, count.clamp(1, 20));
    }
}

#[test]
fn invalid_input_returns_only_closed_error_and_does_not_poison_configuration() {
    let tool = tool();
    let first = tool.prepare(&json!({"query":"good"})).expect("prepare");
    for input in [
        json!(null),
        json!({}),
        json!({"query":" "}),
        json!({"query":{"private":"secret"}}),
        json!({"query":"secret", "max_results":-1}),
        json!({"query":"secret", "include_snippets":"private"}),
        json!({"query":"private".repeat(4096)}),
    ] {
        let error = match tool.prepare_network_invocation(&input) {
            Err(error) => error,
            Ok(_) => panic!("invalid input accepted"),
        };
        assert_eq!(error, ToolNetworkPreparationError::Invalid);
        assert_eq!(error.to_string(), "network_preparation_invalid");
    }
    let next = tool.prepare(&json!({"query":"good"})).expect("prepare");
    assert_eq!(first.manifest().scope(), next.manifest().scope());
}

#[tokio::test]
async fn every_legacy_execution_surface_denies_even_with_path_authorization() {
    let tool = tool();
    let object: &dyn AgentTool = &tool;
    let input = json!({"query":"private-query", "max_results":1});
    let workspace = tempfile::tempdir().expect("workspace");
    let paths = [ToolExecutionAuthorization::for_path(
        "web_search",
        ToolNature::Network,
        workspace.path(),
        ".",
        ToolAuthorizationScope::Once,
    )
    .expect("path authorization")];
    for result in [
        object.execute(input.clone()).await,
        object.execute_authorized(input.clone(), &paths).await,
    ] {
        assert!(result.is_error);
        assert_eq!(result.content, "search_prepared_invocation_required");
    }
    for output in [
        object.execute_with_output(input.clone()).await,
        object.execute_authorized_with_output(input, &paths).await,
    ] {
        assert!(output.result.is_error);
        assert_eq!(output.result.content, "search_prepared_invocation_required");
        assert!(output.next_provider_parts.is_empty());
    }
}

#[tokio::test]
async fn actual_tool_trait_preparation_executes_captured_input_with_real_once_admission() {
    for snippets in [false, true] {
        let body =
            json!({"results":[{"title":"First","url":"https://first.invalid/","content":"one"},
            {"title":"Second","url":"https://second.invalid/","snippet":"two"}]})
            .to_string();
        let server = Server::new(vec![format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )]);
        let tool = local_tool(&server);
        let mut input =
            json!({"query":"captured +中", "max_results":0,"include_snippets":snippets});
        let object = tool
            .prepare_network_invocation(&input)
            .expect("prepare")
            .expect("owned");
        let state = Arc::new(PermissionSessionState::new(PermissionEngine::new()));
        let seal = authority(&state, object.as_ref(), &input);
        input["query"] = json!("replacement");
        input["max_results"] = json!(20);
        input["include_snippets"] = json!(!snippets);
        drop(tool);
        let output = object.execute(seal).await;
        assert!(!output.result.is_error, "{}", output.result.content);
        assert!(output.next_provider_parts.is_empty());
        assert_eq!(
            output.result.content,
            format!(
                "Searched: \"captured +中\"\nSource: SearXNG\nResults: 1\n\n1. First\n   https://first.invalid/\n{}\n",
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
async fn real_once_authority_revocation_or_mismatch_denies_before_any_connection() {
    for revoked in [false, true] {
        let server = Server::new(vec![]);
        let tool = local_tool(&server);
        let input = json!({"query":"private"});
        let first = tool
            .prepare_network_invocation(&input)
            .expect("prepare")
            .expect("owned");
        let second = tool
            .prepare_network_invocation(&input)
            .expect("prepare")
            .expect("owned");
        let state = Arc::new(PermissionSessionState::new(PermissionEngine::new()));
        let seal = authority(&state, first.as_ref(), &input);
        let object = if revoked {
            state.clear().expect("clear");
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
