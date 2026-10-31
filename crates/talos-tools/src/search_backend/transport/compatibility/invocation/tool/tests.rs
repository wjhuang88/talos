use super::*;
use crate::search_backend::transport::{
    compatibility::{CompatibilityProtocol, execution_tests, plan::tests::controlled},
    tests::Server,
};
use serde_json::json;
use std::{
    sync::{Arc, atomic::Ordering},
    time::Duration,
};
use talos_core::tool::{ToolAuthorizationScope, ToolExecutionAuthorization, ToolNature};
use talos_permission::{
    GrantScope, InteractionCapability, PermissionContext, PermissionEngine, PermissionMode,
    PermissionRequest, PermissionSessionState,
};

fn tool() -> GuardedCompatibilitySearchTool {
    GuardedCompatibilitySearchTool::new(
        TransportLimits::new(Duration::from_secs(5), 4, 1_048_576).expect("limits"),
    )
    .expect("tool")
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
    assert_eq!(
        format!("{tool:?}"),
        "GuardedCompatibilitySearchTool(<redacted>)"
    );
    assert!(object.description().contains("Bing"));
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
    assert_eq!(first.permission_profile().len(), 6);
    assert_eq!(first.manifest().scope().endpoints().len(), 9);
    let replacement = self::tool()
        .prepare_network_invocation(&input)
        .expect("prepare")
        .expect("owned");
    assert_ne!(first.manifest().scope(), replacement.manifest().scope());
    assert!(!format!("{:?}", first.manifest()).contains("private-one"));
}

#[test]
fn defaults_and_direct_clamp_match_existing_input_semantics() {
    let tool = tool();
    let default = tool.prepare(&json!({"query":"q"})).expect("defaults");
    assert!(default.include_snippets);
    for count in [0, 1, 20, 21, u32::MAX] {
        let invocation = tool
            .prepare(&json!({"query":"q", "max_results":count, "include_snippets":false}))
            .expect("existing clamp");
        assert!(!invocation.include_snippets);
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
async fn actual_tool_preparation_executes_retained_input_with_real_once_admission() {
    for snippets in [false, true] {
        let tool = tool();
        let mut input = json!({"query":"captured +&", "max_results":1,"include_snippets":snippets});
        // This is the same parsing/preparation method used by the AgentTool implementation.
        let mut invocation = tool.prepare(&input).expect("prepare");
        let body = execution_tests::body(CompatibilityProtocol::Lite);
        let server = Server::new(vec![format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            String::from_utf8(body).expect("body")
        )]);
        let (plan, effects) = controlled(invocation.plan, &server, usize::MAX, false);
        invocation.plan = plan;
        let object: Box<dyn PreparedNetworkToolInvocation> = Box::new(invocation);
        let state = Arc::new(PermissionSessionState::new(PermissionEngine::new()));
        let context = PermissionContext::new(
            PermissionMode::Interactive,
            InteractionCapability::Available,
        );
        let request = PermissionRequest::native(tool.name(), object.permission_profile(), &input)
            .with_network_manifest(object.manifest());
        let proposal = state
            .propose(&request, &context, GrantScope::Once)
            .expect("proposal");
        let pending = state
            .approve_once(proposal, &request, &context)
            .expect("once");
        let authority = state
            .admit_network(pending, &request, &context)
            .expect("admit");
        input["query"] = json!("replacement");
        drop(tool);
        let output = object.execute(authority).await;
        assert!(!output.result.is_error);
        assert!(output.next_provider_parts.is_empty());
        assert_eq!(
            output.result.content,
            format!(
                "Searched: \"captured +&\"\nSource: DuckDuckGo\nResults: 1\n\n1. First\n   https://first.invalid/\n{}\n",
                if snippets { "   one\n" } else { "" }
            )
        );
        assert_eq!(effects.dns.load(Ordering::Relaxed), 1);
        assert_eq!(server.count(), 1);
        assert!(server.request(0).ends_with("q=captured+%2B%26&kl=wt-wt"));
    }
}
