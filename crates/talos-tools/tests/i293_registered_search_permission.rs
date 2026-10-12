//! Real network contribution and permission-state characterization, without execution.
//! Constructors may read ambient configuration; no tool execute method is called.
//! This does not prove CLI/SDK transport behavior or authorize a permission-policy repair.
#![cfg(feature = "network")]

use serde_json::json;
use talos_core::tool::{ToolNature, ToolRegistry};
use talos_permission::{
    GrantError, GrantScope, InteractionCapability, PermissionContext, PermissionDecision,
    PermissionEngine, PermissionInvocation, PermissionMode, PermissionRequest,
    PermissionSessionState, ResourceExtractor,
};
use talos_tools::network_tool_contributions;

fn registered_search() -> ToolRegistry {
    let contributions = network_tool_contributions();
    let search = contributions
        .iter()
        .find(|contribution| contribution.name() == "web_search")
        .expect("real network group contains web_search");
    assert_eq!(search.source().as_str(), "talos-tools:network");
    let mut registry = ToolRegistry::new();
    registry
        .register_contributions(contributions)
        .expect("network contributions register without collisions");
    registry
}

#[test]
fn registered_search_keeps_query_contract_and_untyped_network_profile() {
    let registry = registered_search();
    let tool = registry.get("web_search").expect("registered search");
    let input = json!({"query": "offline characterization", "max_results": 2,
                       "include_snippets": true});
    jsonschema::validate(&tool.parameters(), &input).expect("public query input is valid");
    assert!(tool.parameters()["properties"].get("url").is_none());
    assert_eq!(tool.nature(), ToolNature::Network);
    let facets = tool.permission_profile(&input);
    assert_eq!(facets.len(), 1);
    assert_eq!(facets[0].nature, ToolNature::Network);
    assert!(facets[0].resource.is_none());
    assert!(facets[0].resource_kind.is_none());
    assert!(ResourceExtractor::extract(ToolNature::Network, &input).is_none());
}

#[test]
fn registered_search_default_ask_cannot_compile_once_or_session_proposal() {
    let registry = registered_search();
    let tool = registry.get("web_search").expect("registered search");
    let input = json!({"query": "offline characterization"});
    let facets = tool.permission_profile(&input);
    let request = PermissionRequest::new(tool.name(), tool.provenance(), &facets, &input);
    for context in [
        PermissionContext::new(
            PermissionMode::Interactive,
            InteractionCapability::Available,
        ),
        PermissionContext::new(PermissionMode::Headless, InteractionCapability::Unavailable),
    ] {
        let state = PermissionSessionState::new(PermissionEngine::new());
        assert_eq!(
            state
                .evaluate(&request, &context)
                .expect("evaluate")
                .decision(),
            PermissionDecision::Ask
        );
        assert!(matches!(
            state.begin_invocation(&request, &context),
            Err(GrantError::MissingResource(0))
        ));
        for scope in [GrantScope::Once, GrantScope::Session] {
            assert!(matches!(
                state.propose(&request, &context, scope),
                Err(GrantError::MissingResource(0))
            ));
        }
        assert_eq!(state.grant_count().expect("grant count"), 0);
    }
}

#[test]
fn registered_search_explicit_policy_allow_admits_without_an_approval_grant() {
    let registry = registered_search();
    let tool = registry.get("web_search").expect("registered search");
    let input = json!({"query": "offline characterization"});
    let facets = tool.permission_profile(&input);
    let request = PermissionRequest::new(tool.name(), tool.provenance(), &facets, &input);
    let mut engine = PermissionEngine::new();
    engine
        .load_from_config(&json!({"rules": [{"tool_name": "web_search", "decision": "Allow"}]}))
        .expect("explicit fixture policy");
    let state = PermissionSessionState::new(engine);
    let context =
        PermissionContext::new(PermissionMode::Headless, InteractionCapability::Unavailable);
    let PermissionInvocation::Allow(pending) = state
        .begin_invocation(&request, &context)
        .expect("explicit policy invocation")
    else {
        panic!("explicit policy should allow this fixture request");
    };
    state
        .admit(*pending, &request, &context)
        .expect("policy admission");
    assert_eq!(state.grant_count().expect("grant count"), 0);
    // Admission is observed only: do not execute the real search or any adapter.
}
