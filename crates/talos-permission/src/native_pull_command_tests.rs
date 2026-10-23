use super::*;
use crate::{
    GrantError, GrantScope, GrantSource, PermissionContext, PermissionInvocation,
    PermissionOutcome, PermissionRequest, PermissionRule, PermissionSessionState,
};
use talos_core::tool::PreparedPlanIdentity;

fn profile(root: &std::path::Path) -> Vec<ToolPermissionFacet> {
    vec![
        ToolPermissionFacet::with_resource(
            ToolNature::Read,
            root.to_string_lossy(),
            ToolResourceKind::Path,
        ),
        ToolPermissionFacet::with_resource(
            ToolNature::Execute,
            root.join("git").to_string_lossy(),
            ToolResourceKind::Command,
        ),
        ToolPermissionFacet::with_resource(
            ToolNature::Execute,
            CAPABILITY,
            ToolResourceKind::Remote,
        ),
    ]
}

#[test]
fn undiscovered_transitive_deny_wins_over_allow_and_workspace_trust() {
    let root = std::env::current_dir().expect("workspace");
    let input = serde_json::json!({"remote": "origin"});
    let facets = profile(&root);
    for nature in [
        ToolNature::Read,
        ToolNature::Write,
        ToolNature::Execute,
        ToolNature::Network,
    ] {
        let mut engine = PermissionEngine::with_workspace_root(root.clone());
        engine.set_trusted_workspace(true);
        engine.add_rule(PermissionRule::new_nature(
            ToolNature::Execute,
            None,
            None,
            PermissionDecision::Allow,
        ));
        engine.add_rule(PermissionRule::new_nature(
            nature,
            Some("not-a-known-resource/**".into()),
            None,
            PermissionDecision::Deny("sensitive restriction text".into()),
        ));
        // The ordinary explicit profile cannot see the undiscovered effect.
        assert!(!matches!(
            engine.evaluate_profile("git_pull", &facets[..2], &input),
            PermissionDecision::Deny(_)
        ));
        let report = engine.evaluate_request(
            &PermissionRequest::native("git_pull", &facets, &input),
            &PermissionContext::compatibility(),
        );
        assert_eq!(report.outcome(), PermissionOutcome::Deny, "{nature:?}");
        assert!(
            !serde_json::to_string(&report)
                .expect("report")
                .contains("sensitive")
        );
    }
}

#[test]
fn legacy_path_restriction_cannot_be_assumed_disjoint_from_helpers() {
    let root = std::env::current_dir().expect("workspace");
    let mut engine = PermissionEngine::with_workspace_root(root.clone());
    engine.add_rule(PermissionRule::new(
        "write_file",
        Some("private/**".into()),
        PermissionDecision::Deny("private".into()),
    ));
    assert!(matches!(
        engine.evaluate_profile("git_pull", &profile(&root), &serde_json::json!({})),
        PermissionDecision::Deny(_)
    ));
}

#[test]
fn explicit_ask_on_unknown_effect_retains_human_checkpoint_and_deny_precedence() {
    let root = std::env::current_dir().expect("workspace");
    let facets = profile(&root);
    let input = serde_json::json!({});
    let mut engine = PermissionEngine::with_workspace_root(root.clone());
    engine.add_rule(PermissionRule::new_nature(
        ToolNature::Write,
        Some("private/**".into()),
        None,
        PermissionDecision::Ask,
    ));
    let request = PermissionRequest::native("git_pull", &facets, &input);
    let report = engine.evaluate_request(&request, &PermissionContext::compatibility());
    let command = report.facets().last().expect("command report");
    assert_eq!(command.outcome(), PermissionOutcome::Ask);
    assert!(matches!(
        command.source(),
        PermissionDecisionSource::Rule {
            rule_source: PermissionRuleSource::Explicit,
            ..
        }
    ));
    engine.add_rule(PermissionRule::new_nature(
        ToolNature::Network,
        Some("unknown.example".into()),
        None,
        PermissionDecision::Deny("deny".into()),
    ));
    assert!(matches!(
        engine.evaluate_profile("git_pull", &facets, &input),
        PermissionDecision::Deny(_)
    ));
}

#[test]
fn default_and_configured_ask_do_not_become_explicit_human_only() {
    let root = std::env::current_dir().expect("workspace");
    let mut engine = PermissionEngine::with_workspace_root(root.clone());
    engine.load_from_config(&serde_json::json!({"rules": [{"nature": "Write", "resource": "private/**", "decision": "Ask"}]})).expect("configuration");
    let facets = profile(&root);
    let input = serde_json::json!({});
    let report = engine.evaluate_request(
        &PermissionRequest::native("git_pull", &facets, &input),
        &PermissionContext::compatibility(),
    );
    assert_eq!(report.outcome(), PermissionOutcome::Ask);
    assert!(!matches!(
        report.facets().last().expect("command").source(),
        PermissionDecisionSource::Rule {
            rule_source: PermissionRuleSource::Explicit,
            ..
        }
    ));
}

#[test]
fn internal_restrictions_do_not_describe_process_effects() {
    let root = std::env::current_dir().expect("workspace");
    let mut engine = PermissionEngine::with_workspace_root(root.clone());
    engine.add_rule(PermissionRule::new_nature(
        ToolNature::Internal,
        None,
        None,
        PermissionDecision::Deny("internal".into()),
    ));
    assert_eq!(
        engine.evaluate_profile("git_pull", &profile(&root), &serde_json::json!({})),
        PermissionDecision::Ask
    );
}

#[test]
fn capability_is_closed_to_pull_execute_remote_with_bound_root_and_program() {
    let root = std::env::current_dir().expect("workspace");
    let input = serde_json::json!({});
    let base = profile(&root);
    let valid = |tool, facets: &[ToolPermissionFacet]| {
        crate::grant::compiled_identity(
            &PermissionRequest::native(tool, facets, &input),
            Some(&root),
        )
        .is_ok()
    };
    assert!(valid("git_pull", &base));
    assert!(!valid("git_push", &base));
    assert!(!valid("bash", &base));
    for missing in [0, 1] {
        let mut facets = base.clone();
        facets.remove(missing);
        assert!(!valid("git_pull", &facets));
    }
    for nature in [
        ToolNature::Read,
        ToolNature::Write,
        ToolNature::Network,
        ToolNature::Internal,
    ] {
        let mut facets = base.clone();
        facets[2].nature = nature;
        assert!(!valid("git_pull", &facets));
    }
    for kind in [
        None,
        Some(ToolResourceKind::Path),
        Some(ToolResourceKind::Command),
    ] {
        let mut facets = base.clone();
        facets[2].resource_kind = kind;
        assert!(!valid("git_pull", &facets));
    }
}

#[test]
fn generic_session_approval_is_once_only_and_cannot_use_legacy_proof() {
    let root = std::env::current_dir().expect("workspace");
    let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(root.clone()));
    let mut facets = profile(&root);
    facets[2].description = Some("frozen native Pull and transitive configured helpers".into());
    let input = serde_json::json!({"remote": "origin"});
    let request = PermissionRequest::native("git_pull", &facets, &input);
    let context = PermissionContext::compatibility();
    let PermissionInvocation::Ask { session, .. } =
        state.begin_invocation(&request, &context).expect("begin")
    else {
        panic!("approval required")
    };
    assert_eq!(session.preview().scope(), GrantScope::Once);
    assert_eq!(
        session.preview().notes(),
        &["frozen native Pull and transitive configured helpers"]
    );
    let pending = state
        .approve_session(*session, &request, &context, GrantSource::SdkHostApproval)
        .expect("one invocation");
    assert_eq!(state.grant_count().expect("store"), 0);
    let proof = state
        .try_admit_invocation(pending, &request, &context)
        .expect("proof");
    assert!(matches!(
        proof.consume_for(&request),
        Err(GrantError::AdmissionInvalidated)
    ));
    assert!(matches!(
        state
            .begin_invocation(&request, &context)
            .expect("next call"),
        PermissionInvocation::Ask { .. }
    ));
}

#[test]
fn prepared_command_proof_binds_plan_input_and_policy_generation() {
    let root = std::env::current_dir().expect("workspace");
    let facets = profile(&root);
    let input = serde_json::json!({"remote": "origin"});
    let changed = serde_json::json!({"remote": "other"});
    let request = PermissionRequest::native("git_pull", &facets, &input);
    let context = PermissionContext::compatibility();
    for mutation in 0..4 {
        let state =
            PermissionSessionState::new(PermissionEngine::with_workspace_root(root.clone()));
        let proposal = state
            .propose(&request, &context, GrantScope::Once)
            .expect("proposal");
        let pending = state
            .approve_once(proposal, &request, &context)
            .expect("approve");
        let plan = PreparedPlanIdentity::new();
        let proof = state
            .try_admit_prepared_invocation(pending, &request, &context, &plan)
            .expect("admit");
        match mutation {
            0 => {
                proof
                    .consume_for_prepared(&request, &plan)
                    .expect("exact command proof");
            }
            1 => assert!(matches!(
                proof.consume_for_prepared(&request, &PreparedPlanIdentity::new()),
                Err(GrantError::AdmissionInvalidated)
            )),
            2 => assert!(matches!(
                proof.consume_for_prepared(
                    &PermissionRequest::native("git_pull", &facets, &changed),
                    &plan
                ),
                Err(GrantError::AdmissionInvalidated)
            )),
            _ => {
                let mut engine = PermissionEngine::with_workspace_root(root.clone());
                engine.add_rule(PermissionRule::new_nature(
                    ToolNature::Write,
                    Some("private/**".into()),
                    None,
                    PermissionDecision::Deny("new policy".into()),
                ));
                state.replace_policy(engine).expect("publish restriction");
                assert!(matches!(
                    proof.consume_for_prepared(&request, &plan),
                    Err(GrantError::AdmissionInvalidated)
                ));
            }
        }
    }
}
