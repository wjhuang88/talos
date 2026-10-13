//! ADR-090 public API and adversarial lifecycle fixtures; no HTTP/DNS.
use serde_json::{Value, json};
use std::sync::Arc;
use talos_core::tool::{
    ToolNature, ToolNetworkAuthorizationError, ToolNetworkEndpoint, ToolNetworkManifest,
    ToolNetworkScope, ToolPermissionFacet, ToolProvenance, ToolResourceKind,
};
use talos_permission::{
    GrantScope, GrantSource, InteractionCapability, PermissionContext, PermissionDecision,
    PermissionEngine, PermissionRequest, PermissionSessionState,
};
use uuid::Uuid;

fn context() -> PermissionContext {
    PermissionContext::new(
        talos_permission::PermissionMode::Interactive,
        InteractionCapability::Available,
    )
}

#[test]
fn parsed_http_url_ip_hosts_share_endpoint_and_policy_canonicalization() {
    for raw in [
        "https://[::1]:8443/search",
        "https://[2001:0db8::1]/search",
        "https://[::ffff:127.0.0.1]/search",
        "https://[::ffff:192.0.2.1]/search",
        "https://127.0.0.1/search",
    ] {
        let url = url::Url::parse(raw).expect("parsed URL");
        let destination = ToolNetworkEndpoint::new(
            url.scheme(),
            url.host_str().expect("host"),
            url.port_or_known_default().expect("port"),
            "GET",
            url.path(),
        )
        .expect("actual parsed destination");
        let facet = ToolPermissionFacet::with_resource(
            ToolNature::Network,
            destination.host(),
            ToolResourceKind::Domain,
        );
        let manifest = ToolNetworkManifest::new(
            ToolNetworkScope::new_configuration(vec![destination], None, vec![], 0).expect("scope"),
        );
        let input = json!({"query":"offline IP fixture"});
        let facets = [facet];
        let request = PermissionRequest::native("web_search", &facets, &input)
            .with_network_manifest(&manifest);
        let state = Arc::new(PermissionSessionState::new(PermissionEngine::new()));
        let proposal = state
            .propose(&request, &context(), GrantScope::Once)
            .expect("proposal");
        let pending = state
            .approve_once(proposal, &request, &context())
            .expect("approval");
        let authority = state
            .admit_network(pending, &request, &context())
            .expect("admit");
        authority
            .check(manifest.invocation_id(), &manifest.scope().endpoints()[0])
            .expect("exact IP scope");
    }
}
fn endpoint(port: u16, method: &str, path: &str) -> ToolNetworkEndpoint {
    ToolNetworkEndpoint::new("https", "search.example", port, method, path).expect("endpoint")
}
fn scope(id: Uuid, endpoint: ToolNetworkEndpoint) -> ToolNetworkScope {
    ToolNetworkScope::new(id, 0, vec![endpoint], None, vec![], 0).expect("scope")
}
fn profile() -> Vec<ToolPermissionFacet> {
    vec![ToolPermissionFacet::with_resource(
        ToolNature::Network,
        "search.example",
        ToolResourceKind::Domain,
    )]
}
fn request<'a>(
    input: &'a Value,
    facets: &'a [ToolPermissionFacet],
    manifest: &'a ToolNetworkManifest,
) -> PermissionRequest<'a> {
    PermissionRequest::native("web_search", facets, input).with_network_manifest(manifest)
}

#[test]
fn session_query_reuse_requires_complete_scope_and_fresh_invocation_authority() {
    let state = Arc::new(PermissionSessionState::new(PermissionEngine::new()));
    let id = Uuid::new_v4();
    let destination = endpoint(443, "GET", "/search");
    let original_scope = scope(id, destination.clone());
    let first = ToolNetworkManifest::new(original_scope.clone());
    let facets = profile();
    let input = json!({"query":"first-private-query"});
    let req = request(&input, &facets, &first);
    let proposal = state
        .propose(&req, &context(), GrantScope::Session)
        .expect("proposal");
    assert_eq!(proposal.preview().network_scope(), Some(&original_scope));
    assert!(!format!("{proposal:?}").contains("private-query"));
    let pending = state
        .approve_session(proposal, &req, &context(), GrantSource::SdkHostApproval)
        .expect("approve");
    let first_authority = state
        .admit_network(pending, &req, &context())
        .expect("admit");
    first_authority
        .check(first.invocation_id(), &destination)
        .expect("initial send");
    let second = ToolNetworkManifest::new(original_scope.clone());
    let next_input = json!({"query":"different-private-query"});
    let next = request(&next_input, &facets, &second);
    let pending = state
        .prepare_authorized(&next, &context())
        .expect("prepare")
        .expect("same scope grant");
    let second_authority = state
        .try_admit_network(pending, &next, &context())
        .expect("admit fresh");
    second_authority
        .check(second.invocation_id(), &destination)
        .expect("fresh send");
    assert_eq!(
        first_authority.check(second.invocation_id(), &destination),
        Err(ToolNetworkAuthorizationError::Mismatch)
    );
    assert_eq!(
        second_authority.check(first.invocation_id(), &destination),
        Err(ToolNetworkAuthorizationError::Mismatch)
    );
    let changed = [
        scope(id, endpoint(8443, "GET", "/search")),
        scope(id, endpoint(443, "POST", "/search")),
        scope(id, endpoint(443, "GET", "/other")),
        scope(Uuid::new_v4(), destination.clone()),
        ToolNetworkScope::new(id, 1, vec![destination.clone()], None, vec![], 0).expect("revision"),
        ToolNetworkScope::new(id, 0, vec![destination.clone()], None, vec![], 1).expect("redirect"),
        ToolNetworkScope::new(
            id,
            0,
            vec![destination.clone()],
            None,
            vec!["127.0.0.1".parse().expect("ip")],
            0,
        )
        .expect("address"),
        ToolNetworkScope::new(
            id,
            0,
            vec![destination.clone(), endpoint(443, "HEAD", "/probe")],
            None,
            vec![],
            0,
        )
        .expect("probe"),
        ToolNetworkScope::new(
            id,
            0,
            vec![destination.clone()],
            Some(endpoint(443, "CONNECT", "/")),
            vec![],
            0,
        )
        .expect("proxy"),
    ];
    for changed_scope in changed {
        let manifest = ToolNetworkManifest::new(changed_scope);
        let changed_request = request(&next_input, &facets, &manifest);
        assert_eq!(
            state
                .evaluate(&changed_request, &context())
                .expect("evaluate")
                .decision(),
            PermissionDecision::Ask
        );
        assert!(
            state
                .prepare_authorized(&changed_request, &context())
                .expect("prepare changed")
                .is_none()
        );
    }
    for changed_endpoint in [
        endpoint(8443, "GET", "/search"),
        endpoint(443, "POST", "/search"),
        endpoint(443, "GET", "/other"),
    ] {
        assert_eq!(
            first_authority.check(first.invocation_id(), &changed_endpoint),
            Err(ToolNetworkAuthorizationError::Mismatch)
        );
    }
}

#[test]
fn legacy_host_grants_never_upgrade_and_network_grants_never_downgrade() {
    for legacy_first in [true, false] {
        let state = Arc::new(PermissionSessionState::new(PermissionEngine::new()));
        let input = json!({"query":"private"});
        let facets = profile();
        let manifest =
            ToolNetworkManifest::new(scope(Uuid::new_v4(), endpoint(443, "GET", "/search")));
        let legacy = PermissionRequest::native("web_search", &facets, &input);
        let network = request(&input, &facets, &manifest);
        let (approved, other) = if legacy_first {
            (&legacy, &network)
        } else {
            (&network, &legacy)
        };
        let proposal = state
            .propose(approved, &context(), GrantScope::Session)
            .expect("proposal");
        state
            .approve_session(proposal, approved, &context(), GrantSource::SdkHostApproval)
            .expect("approve");
        assert_eq!(
            state
                .evaluate(other, &context())
                .expect("evaluate")
                .decision(),
            PermissionDecision::Ask
        );
    }
}

#[test]
fn once_is_bound_to_input_and_invocation_and_cannot_use_path_admission() {
    let state = Arc::new(PermissionSessionState::new(PermissionEngine::new()));
    let manifest = ToolNetworkManifest::new(scope(Uuid::new_v4(), endpoint(443, "GET", "/search")));
    let facets = profile();
    let input = json!({"query":"private"});
    let req = request(&input, &facets, &manifest);
    let proposal = state
        .propose(&req, &context(), GrantScope::Once)
        .expect("proposal");
    let pending = state
        .approve_once(proposal, &req, &context())
        .expect("once");
    let changed = json!({"query":"other"});
    assert!(
        state
            .admit_network(pending, &request(&changed, &facets, &manifest), &context())
            .is_err()
    );
    let proposal = state
        .propose(&req, &context(), GrantScope::Once)
        .expect("proposal");
    let another = ToolNetworkManifest::new(manifest.scope().clone());
    assert!(
        state
            .approve_once(proposal, &request(&input, &facets, &another), &context())
            .is_err()
    );
    let proposal = state
        .propose(&req, &context(), GrantScope::Once)
        .expect("proposal");
    let pending = state
        .approve_once(proposal, &req, &context())
        .expect("once");
    assert!(state.admit(pending, &req, &context()).is_err());
    let proposal = state
        .propose(&req, &context(), GrantScope::Once)
        .expect("proposal");
    let pending = state
        .approve_once(proposal, &req, &context())
        .expect("once");
    let authority = state
        .admit_network(pending, &req, &context())
        .expect("once admit");
    authority
        .check(manifest.invocation_id(), &endpoint(443, "GET", "/search"))
        .expect("once request");
    state.clear().expect("revoke");
    assert_eq!(
        authority.check(manifest.invocation_id(), &endpoint(443, "GET", "/search")),
        Err(ToolNetworkAuthorizationError::Invalidated)
    );
}

#[test]
fn unrelated_grant_creation_does_not_revoke_started_authority_but_clear_does() {
    let state = Arc::new(PermissionSessionState::new(PermissionEngine::new()));
    let manifest = ToolNetworkManifest::new(scope(Uuid::new_v4(), endpoint(443, "GET", "/search")));
    let facets = profile();
    let input = json!({"query":"private"});
    let req = request(&input, &facets, &manifest);
    let proposal = state
        .propose(&req, &context(), GrantScope::Session)
        .expect("proposal");
    let pending = state
        .approve_session(proposal, &req, &context(), GrantSource::SdkHostApproval)
        .expect("approve");
    let authority = state
        .admit_network(pending, &req, &context())
        .expect("admit");
    let other_facets = [ToolPermissionFacet::with_resource(
        ToolNature::Network,
        "unrelated.example",
        ToolResourceKind::Domain,
    )];
    let other = PermissionRequest::native("other", &other_facets, &input);
    let proposal = state
        .propose(&other, &context(), GrantScope::Session)
        .expect("unrelated");
    state
        .approve_session(proposal, &other, &context(), GrantSource::SdkHostApproval)
        .expect("other approval");
    authority
        .check(manifest.invocation_id(), &endpoint(443, "GET", "/search"))
        .expect("unrelated grant doesn't revoke");
    state.clear().expect("clear");
    assert_eq!(
        authority.check(manifest.invocation_id(), &endpoint(443, "GET", "/search")),
        Err(ToolNetworkAuthorizationError::Invalidated)
    );
}

#[test]
fn incomplete_wildcard_or_remote_profiles_cannot_compile_network_authority() {
    let state = PermissionSessionState::new(PermissionEngine::new());
    let input = json!({"query":"private"});
    let manifest = ToolNetworkManifest::new(scope(Uuid::new_v4(), endpoint(443, "GET", "/search")));
    for facets in [
        vec![],
        vec![ToolPermissionFacet::new(ToolNature::Network)],
        vec![ToolPermissionFacet::with_resource(
            ToolNature::Network,
            "*.example",
            ToolResourceKind::Domain,
        )],
        vec![ToolPermissionFacet::with_resource(
            ToolNature::Network,
            "search.example",
            ToolResourceKind::Remote,
        )],
    ] {
        assert!(
            state
                .begin_invocation(&request(&input, &facets, &manifest), &context())
                .is_err()
        );
    }
    let facets = profile();
    let remote = PermissionRequest::new(
        "web_search",
        ToolProvenance::McpRemote {
            server: "remote".into(),
        },
        &facets,
        &input,
    )
    .with_network_manifest(&manifest);
    assert!(state.begin_invocation(&remote, &context()).is_err());
}

#[test]
fn policy_and_registration_generations_invalidate_future_logical_starts() {
    let changes: [fn(&PermissionSessionState); 6] = [
        |s| s.bump_mode_generation().expect("mode"),
        |s| s.bump_workspace_generation().expect("workspace"),
        |s| s.bump_registration_generation().expect("registration"),
        |s| s.bump_restriction_generation().expect("restriction"),
        |s| s.rebind_session().expect("session"),
        |s| s.replace_policy(PermissionEngine::new()).expect("policy"),
    ];
    for change in changes {
        let state = Arc::new(PermissionSessionState::new(PermissionEngine::new()));
        let manifest =
            ToolNetworkManifest::new(scope(Uuid::new_v4(), endpoint(443, "GET", "/search")));
        let facets = profile();
        let input = json!({"query":"private"});
        let req = request(&input, &facets, &manifest);
        let proposal = state
            .propose(&req, &context(), GrantScope::Once)
            .expect("proposal");
        let pending = state
            .approve_once(proposal, &req, &context())
            .expect("approve");
        let authority = state
            .admit_network(pending, &req, &context())
            .expect("admit");
        change(&state);
        assert_eq!(
            authority.check(manifest.invocation_id(), &endpoint(443, "GET", "/search")),
            Err(ToolNetworkAuthorizationError::Invalidated)
        );
    }
}

#[test]
fn existing_deny_dominance_and_complete_fallback_coverage_are_preserved() {
    use talos_permission::{PermissionRule, ResourceKind};
    let allow =
        PermissionRule::new_nature(ToolNature::Network, None, None, PermissionDecision::Allow);
    let deny = PermissionRule::new_nature(
        ToolNature::Network,
        Some("fallback.example".into()),
        Some(ResourceKind::Domain),
        PermissionDecision::Deny("fallback denied".into()),
    );
    let state = PermissionSessionState::new(PermissionEngine::from_rules(vec![allow, deny]));
    let primary = endpoint(443, "GET", "/search");
    let fallback = ToolNetworkEndpoint::new("https", "fallback.example", 443, "GET", "/fallback")
        .expect("fallback");
    let manifest = ToolNetworkManifest::new(
        ToolNetworkScope::new(Uuid::new_v4(), 0, vec![primary, fallback], None, vec![], 0)
            .expect("scope"),
    );
    let mut facets = profile();
    facets.push(ToolPermissionFacet::with_resource(
        ToolNature::Network,
        "fallback.example",
        ToolResourceKind::Domain,
    ));
    let input = json!({"query":"private"});
    let req = request(&input, &facets, &manifest);
    assert!(matches!(
        state
            .evaluate(&req, &context())
            .expect("evaluate")
            .decision(),
        PermissionDecision::Deny(_)
    ));
    assert!(
        state
            .propose(&req, &context(), GrantScope::Session)
            .is_err()
    );
    assert!(
        state
            .prepare_authorized(&req, &context())
            .expect("denied")
            .is_none()
    );
    assert!(
        state
            .begin_invocation(&request(&input, &facets[..1], &manifest), &context())
            .is_err(),
        "cannot strip a denied fallback"
    );
}
