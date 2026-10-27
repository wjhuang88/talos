use super::*;

fn invalidate(state: &PermissionSessionState, mutation: usize) {
    match mutation {
        0 => state.replace_policy(PermissionEngine::new()),
        1 => state.bump_mode_generation(),
        2 => state.rebind_session(),
        _ => state.clear(),
    }
    .expect("invalidate");
}

#[test]
fn broad_authority_rechecks_every_admission_boundary() {
    let input = serde_json::json!({"command": "program"});
    let facets = [ToolPermissionFacet::with_resource(
        ToolNature::Execute,
        "program",
        ToolResourceKind::Command,
    )];
    let context = PermissionContext::compatibility();
    let request = PermissionRequest::native("exec", &facets, &input).with_transitive_effects();
    for mutation in 0..4 {
        for boundary in 0..3 {
            let state = PermissionSessionState::new(PermissionEngine::new());
            let proposal = state
                .propose(&request, &context, GrantScope::Once)
                .expect("proposal");
            if boundary == 0 {
                invalidate(&state, mutation);
                assert!(state.approve_once(proposal, &request, &context).is_err());
                continue;
            }
            let pending = state
                .approve_once(proposal, &request, &context)
                .expect("approve");
            if boundary == 1 {
                invalidate(&state, mutation);
                assert!(
                    state
                        .try_admit_invocation(pending, &request, &context)
                        .is_err()
                );
                continue;
            }
            let proof = state
                .try_admit_invocation(pending, &request, &context)
                .expect("admit");
            invalidate(&state, mutation);
            assert!(proof.consume_for(&request).is_err());
        }
    }
}

#[test]
fn prior_narrow_session_grant_cannot_authorize_broad_effects() {
    let state = PermissionSessionState::new(PermissionEngine::new());
    let input = serde_json::json!({"command": "program"});
    let facets = [ToolPermissionFacet::with_resource(
        ToolNature::Execute,
        "program",
        ToolResourceKind::Command,
    )];
    let context = PermissionContext::compatibility();
    let narrow = PermissionRequest::native("exec", &facets, &input);
    let PermissionInvocation::Ask { session, .. } =
        state.begin_invocation(&narrow, &context).expect("begin")
    else {
        panic!("ask")
    };
    state
        .approve_session(*session, &narrow, &context, GrantSource::SdkHostApproval)
        .expect("session approval");
    assert_eq!(state.grant_count().expect("store"), 1);
    assert!(matches!(
        state.begin_invocation(&narrow, &context).expect("narrow"),
        PermissionInvocation::Allow(_)
    ));
    let broad = narrow.with_transitive_effects();
    assert!(matches!(
        state.begin_invocation(&broad, &context).expect("broad"),
        PermissionInvocation::Ask { .. }
    ));
}

#[test]
fn unknown_effect_denies_precede_deterministic_allow() {
    let input = serde_json::json!({"command": "known-program"});
    let facets = [ToolPermissionFacet::with_resource(
        ToolNature::Execute,
        "known-program",
        ToolResourceKind::Command,
    )];
    for nature in [
        ToolNature::Read,
        ToolNature::Write,
        ToolNature::Execute,
        ToolNature::Network,
    ] {
        let mut engine = PermissionEngine::empty();
        engine.add_rule(PermissionRule::new_nature(
            ToolNature::Execute,
            None,
            None,
            PermissionDecision::Allow,
        ));
        engine.add_rule(PermissionRule::new_nature(
            nature,
            Some("undisclosed-target".into()),
            None,
            PermissionDecision::Deny("private policy text".into()),
        ));
        let narrow = PermissionRequest::native("exec", &facets, &input);
        assert_eq!(
            engine
                .evaluate_request(&narrow, &PermissionContext::compatibility())
                .outcome(),
            PermissionOutcome::Allow
        );
        let broad = narrow.with_transitive_effects();
        let report = engine.evaluate_request(&broad, &PermissionContext::compatibility());
        assert_eq!(report.outcome(), PermissionOutcome::Deny, "{nature:?}");
        assert!(
            !serde_json::to_string(&report)
                .expect("report")
                .contains("private policy text")
        );
    }
}

#[test]
fn explicit_unknown_effect_ask_remains_human_only() {
    let mut engine = PermissionEngine::empty();
    engine.add_rule(PermissionRule::new_nature(
        ToolNature::Execute,
        None,
        None,
        PermissionDecision::Allow,
    ));
    engine.add_rule(PermissionRule::new_nature(
        ToolNature::Write,
        Some("private/**".into()),
        None,
        PermissionDecision::Ask,
    ));
    let input = serde_json::json!({});
    let facets = [ToolPermissionFacet::new(ToolNature::Execute)];
    let request = PermissionRequest::native("exec", &facets, &input).with_transitive_effects();
    let report = engine.evaluate_request(&request, &PermissionContext::compatibility());
    assert_eq!(report.outcome(), PermissionOutcome::Ask);
    assert!(matches!(
        report.facets()[0].source(),
        PermissionDecisionSource::Rule {
            rule_source: PermissionRuleSource::Explicit,
            ..
        }
    ));
}

#[test]
fn broad_approval_is_once_only_and_cannot_be_reused_without_effect_binding() {
    let input = serde_json::json!({"command": "program"});
    let facets = [ToolPermissionFacet::with_resource(
        ToolNature::Execute,
        "program",
        ToolResourceKind::Command,
    )];
    let context = PermissionContext::compatibility();
    for remove_binding in [false, true] {
        let state = PermissionSessionState::new(PermissionEngine::new());
        let request = PermissionRequest::native("exec", &facets, &input).with_transitive_effects();
        let PermissionInvocation::Ask { session, .. } =
            state.begin_invocation(&request, &context).expect("begin")
        else {
            panic!("ask")
        };
        assert_eq!(session.preview().scope(), GrantScope::Once);
        let pending = state
            .approve_session(*session, &request, &context, GrantSource::SdkHostApproval)
            .expect("approve once");
        assert_eq!(state.grant_count().expect("store"), 0);
        let proof = state
            .try_admit_invocation(pending, &request, &context)
            .expect("proof");
        if remove_binding {
            assert!(
                proof
                    .consume_for(&PermissionRequest::native("exec", &facets, &input))
                    .is_err()
            );
        } else {
            proof.consume_for(&request).expect("exact binding");
        }
        assert!(matches!(
            state.begin_invocation(&request, &context).expect("next"),
            PermissionInvocation::Ask { .. }
        ));
    }
}
