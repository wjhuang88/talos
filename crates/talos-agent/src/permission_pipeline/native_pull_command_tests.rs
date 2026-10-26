use super::*;
use talos_core::tool::{PreparedPlanIdentity, ToolNature, ToolResourceKind};
use talos_permission::{PermissionDecision, PermissionEngine, PermissionRule};

struct Resolver {
    human_only: bool,
}

#[async_trait]
impl ApprovalResolver for Resolver {
    async fn resolve(
        &self,
        _: PermissionApprovalRequest,
        _: Duration,
    ) -> Result<ApprovalChoice, ApprovalResolverError> {
        panic!("prepared authorization must preserve execution input")
    }

    async fn resolve_with_execution_input(
        &self,
        _: PermissionApprovalRequest,
        execution_input: &serde_json::Value,
        _: Duration,
        auto_assessment_allowed: bool,
        _: Option<&str>,
    ) -> Result<ApprovalChoice, ApprovalResolverError> {
        assert_eq!(execution_input["remote"], "origin");
        assert_eq!(auto_assessment_allowed, !self.human_only);
        Ok(ApprovalChoice::AlwaysApprove)
    }
}

#[tokio::test]
async fn command_authority_preserves_human_policy_and_never_becomes_session_authority() {
    for restriction in [
        None,
        Some(PermissionDecision::Ask),
        Some(PermissionDecision::Deny("private".into())),
    ] {
        let root = tempfile::tempdir().expect("workspace");
        let mut engine = PermissionEngine::with_workspace_root(root.path().to_owned());
        let human_only = matches!(restriction, Some(PermissionDecision::Ask));
        let denied = matches!(restriction, Some(PermissionDecision::Deny(_)));
        if let Some(decision) = restriction {
            engine.add_rule(PermissionRule::new_nature(
                ToolNature::Write,
                Some("unseen-helper-output/**".into()),
                None,
                decision,
            ));
        }
        let state = Arc::new(PermissionSessionState::new(engine));
        let pipeline = PermissionPipeline::new(
            state.clone(),
            PermissionContext::new(
                PermissionMode::Interactive,
                InteractionCapability::Available,
            ),
            // A denial must return before consulting any resolver.
            if denied {
                None
            } else {
                Some(Arc::new(Resolver { human_only }))
            },
        );
        let facets = [
            ToolPermissionFacet::with_resource(
                ToolNature::Read,
                root.path().to_string_lossy(),
                ToolResourceKind::Path,
            ),
            ToolPermissionFacet::with_resource(
                ToolNature::Execute,
                root.path().join("git").to_string_lossy(),
                ToolResourceKind::Command,
            ),
            ToolPermissionFacet::with_resource(
                ToolNature::Execute,
                "git:pull-command",
                ToolResourceKind::Remote,
            ),
        ];
        let input = serde_json::json!({"remote": "origin"});
        let plan = PreparedPlanIdentity::new();
        let result = pipeline
            .authorize_prepared_plan(
                PermissionAuthorizationRequest {
                    tool_name: "git_pull",
                    provenance: ToolProvenance::Native,
                    profile: &facets,
                    input: &input,
                    presentation_input: input.clone(),
                    summary_fields: vec![],
                    deadline: Duration::from_secs(2),
                },
                None,
                &plan,
            )
            .await;
        if denied {
            assert!(matches!(result, Err(PermissionPipelineError::Denied(_))));
        } else {
            result
                .expect("approved command")
                .consume_for_prepared(
                    &PermissionRequest::native("git_pull", &facets, &input),
                    &plan,
                )
                .expect("one-use prepared proof");
        }
        assert_eq!(state.grant_count().expect("session state"), 0);
    }
}
