use super::*;
use talos_core::tool::{
    PreparedCommandDescription, PreparedCommandEffect, PreparedCommandEvidence,
    PreparedPlanIdentity, ToolNature, ToolResourceKind,
};
use talos_permission::PermissionEngine;

struct CommandResolver;

struct LegacyAutomaticResolver;

#[async_trait]
impl ApprovalResolver for LegacyAutomaticResolver {
    async fn resolve(
        &self,
        _: PermissionApprovalRequest,
        _: Duration,
    ) -> Result<ApprovalChoice, ApprovalResolverError> {
        panic!("legacy automatic entrypoint must not regain automatic eligibility")
    }

    async fn resolve_with_auto_assessment(
        &self,
        _: PermissionApprovalRequest,
        _: Duration,
        auto_allowed: bool,
        intent: Option<&str>,
    ) -> Result<ApprovalChoice, ApprovalResolverError> {
        assert!(!auto_allowed);
        assert_eq!(intent, Some("update this repository"));
        Ok(ApprovalChoice::Deny)
    }
}

#[async_trait]
impl ApprovalResolver for CommandResolver {
    async fn resolve(
        &self,
        _: PermissionApprovalRequest,
        _: Duration,
    ) -> Result<ApprovalChoice, ApprovalResolverError> {
        panic!("command evidence must reach its dedicated resolver")
    }

    async fn resolve_prepared_command(
        &self,
        request: PermissionApprovalRequest,
        evidence: &PreparedCommandEvidence,
        _: Duration,
        auto_allowed: bool,
        intent: Option<&str>,
    ) -> Result<ApprovalChoice, ApprovalResolverError> {
        assert_eq!(
            request.arguments,
            serde_json::json!({"remote": "<redacted>"})
        );
        assert_eq!(evidence.description().operation, "git:pull-command/v1");
        assert!(auto_allowed);
        assert_eq!(intent, Some("update this repository"));
        Ok(ApprovalChoice::ApproveOnce)
    }
}

#[tokio::test]
async fn evidence_is_bound_before_resolution_and_preserves_one_use_admission() {
    let root = tempfile::tempdir().expect("workspace");
    let state = Arc::new(PermissionSessionState::new(
        PermissionEngine::with_workspace_root(root.path().to_owned()),
    ));
    let pipeline = PermissionPipeline::new(
        state.clone(),
        PermissionContext::new(
            PermissionMode::Interactive,
            InteractionCapability::Available,
        ),
        Some(Arc::new(CommandResolver)),
    );
    let plan = PreparedPlanIdentity::new();
    let evidence = PreparedCommandEvidence::new(
        plan.clone(),
        PreparedCommandDescription {
            operation: "git:pull-command/v1".into(),
            arguments: vec!["pull".into()],
            effects: vec![
                PreparedCommandEffect::Execute,
                PreparedCommandEffect::Network,
            ],
            confined: false,
            targets: talos_core::tool::PreparedCommandTargets {
                repository: ".".into(),
                metadata: vec![".git".into()],
                remote_selection: "configured upstream".into(),
                configured_remote_hosts: vec![],
                invocation_remote_host: None,
                configured_program_categories: vec![],
                dynamic_effects_unknown: true,
            },
        },
    );
    let facets = [ToolPermissionFacet::with_resource(
        ToolNature::Execute,
        "git pull origin",
        ToolResourceKind::Command,
    )];
    let input = serde_json::json!({"remote": "origin"});
    for matched in [false, true] {
        let other = PreparedPlanIdentity::new();
        let identity = if matched { &plan } else { &other };
        let result = pipeline
            .authorize_prepared_plan_with_evidence(
                PermissionAuthorizationRequest {
                    tool_name: "git_pull",
                    provenance: ToolProvenance::Native,
                    profile: &facets,
                    input: &input,
                    presentation_input: serde_json::json!({"remote": "<redacted>"}),
                    summary_fields: vec![],
                    deadline: Duration::from_secs(2),
                },
                Some("update this repository"),
                identity,
                Some(&evidence),
            )
            .await;
        if matched {
            result
                .expect("dedicated approval")
                .consume_for_prepared(
                    &PermissionRequest::native("git_pull", &facets, &input),
                    &plan,
                )
                .expect("bound admission");
        } else {
            assert!(
                matches!(result, Err(PermissionPipelineError::State(message))
                if message == "command evidence plan mismatch")
            );
        }
    }
    assert_eq!(state.grant_count().expect("grants"), 0);

    let legacy_pipeline = PermissionPipeline::new(
        state,
        PermissionContext::new(
            PermissionMode::Interactive,
            InteractionCapability::Available,
        ),
        Some(Arc::new(LegacyAutomaticResolver)),
    );
    let result = legacy_pipeline
        .authorize_prepared_plan_with_evidence(
            PermissionAuthorizationRequest {
                tool_name: "git_pull",
                provenance: ToolProvenance::Native,
                profile: &facets,
                input: &input,
                presentation_input: input.clone(),
                summary_fields: vec![],
                deadline: Duration::from_secs(2),
            },
            Some("update this repository"),
            &plan,
            Some(&evidence),
        )
        .await;
    assert!(matches!(result, Err(PermissionPipelineError::Denied(_))));
}
