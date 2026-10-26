use super::*;
use talos_core::tool::{PreparedPlanIdentity, ToolNature, ToolResourceKind};
use talos_permission::PermissionEngine;

struct InputResolver;

#[async_trait]
impl ApprovalResolver for InputResolver {
    async fn resolve(
        &self,
        request: PermissionApprovalRequest,
        _remaining: Duration,
    ) -> Result<ApprovalChoice, ApprovalResolverError> {
        assert_eq!(
            request.arguments,
            serde_json::json!({"remote": "<redacted>"})
        );
        Ok(ApprovalChoice::Deny)
    }

    async fn resolve_with_execution_input(
        &self,
        request: PermissionApprovalRequest,
        execution_input: &serde_json::Value,
        remaining: Duration,
        auto_assessment_allowed: bool,
        user_intent: Option<&str>,
    ) -> Result<ApprovalChoice, ApprovalResolverError> {
        assert_eq!(execution_input, &serde_json::json!({"remote": "origin"}));
        assert!(auto_assessment_allowed);
        assert_eq!(user_intent, Some("update this repository"));
        // A human fallback sees only presentation data, even after private assessment.
        self.resolve(request, remaining).await?;
        Ok(ApprovalChoice::ApproveOnce)
    }
}

#[tokio::test]
async fn native_and_prepared_requests_separate_execution_input_from_presentation() {
    for prepared in [false, true] {
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
            Some(Arc::new(InputResolver)),
        );
        let input = serde_json::json!({"remote": "origin"});
        let facets = [ToolPermissionFacet::with_resource(
            ToolNature::Execute,
            "git pull origin",
            ToolResourceKind::Command,
        )];
        let authorization = PermissionAuthorizationRequest {
            tool_name: "git_pull",
            provenance: ToolProvenance::Native,
            profile: &facets,
            input: &input,
            presentation_input: serde_json::json!({"remote": "<redacted>"}),
            summary_fields: vec!["remote".into()],
            deadline: Duration::from_secs(2),
        };
        if prepared {
            let plan = PreparedPlanIdentity::new();
            let admitted = pipeline
                .authorize_prepared_plan(authorization, Some("update this repository"), &plan)
                .await
                .expect("prepared approval");
            let request =
                PermissionRequest::new("git_pull", ToolProvenance::Native, &facets, &input);
            admitted
                .consume_for_prepared(&request, &plan)
                .expect("exact proof");
        } else {
            pipeline
                .authorize_with_user_intent(authorization, Some("update this repository"))
                .await
                .expect("native approval");
        }
        assert_eq!(state.grant_count().expect("grants"), 0);
    }
}

#[tokio::test]
async fn shortened_presentation_cannot_hide_oversized_write_from_auto() {
    use crate::auto_resolver::{
        AutoPermissionAssessor, AutoPermissionControl, AutoPermissionRequest,
        AutoPermissionResolver, ManagedWorkspaceLease,
    };
    struct UnexpectedAssessor(Arc<std::sync::atomic::AtomicUsize>);
    #[async_trait]
    impl AutoPermissionAssessor for UnexpectedAssessor {
        async fn assess(&self, _: AutoPermissionRequest, _: Duration) -> Result<String, String> {
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Err("unexpected model call".into())
        }
    }
    struct Fallback;
    #[async_trait]
    impl ApprovalResolver for Fallback {
        async fn resolve(
            &self,
            request: PermissionApprovalRequest,
            _: Duration,
        ) -> Result<ApprovalChoice, ApprovalResolverError> {
            assert_eq!(request.arguments["content"], "summary");
            Ok(ApprovalChoice::Deny)
        }
    }
    struct Capability;
    impl talos_core::tool::AtomicCreateCapability for Capability {
        fn create_new(&self, _: &std::path::Path, _: &[u8]) -> std::io::Result<()> {
            panic!("denied request cannot write")
        }
    }
    let root = tempfile::tempdir().expect("workspace");
    let state = Arc::new(PermissionSessionState::new(
        PermissionEngine::with_workspace_root(root.path().to_owned()),
    ));
    let lease = ManagedWorkspaceLease::for_permission_session(root.path(), state.clone())
        .expect("lease")
        .with_atomic_create_capability(Arc::new(Capability));
    let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let resolver = Arc::new(AutoPermissionResolver::new(
        Arc::new(UnexpectedAssessor(calls.clone())),
        Arc::new(Fallback),
        lease,
        Duration::from_secs(2),
        AutoPermissionControl::new(true),
    ));
    let pipeline = PermissionPipeline::new(
        state,
        PermissionContext::new(
            PermissionMode::Interactive,
            InteractionCapability::Available,
        ),
        Some(resolver),
    );
    let facets = [ToolPermissionFacet::with_resource(
        ToolNature::Write,
        root.path().join("new.txt").display().to_string(),
        ToolResourceKind::Path,
    )];
    let input = serde_json::json!({"path": "new.txt", "content": "x".repeat(65537)});
    let result = pipeline
        .authorize(PermissionAuthorizationRequest {
            tool_name: "write",
            provenance: ToolProvenance::Native,
            profile: &facets,
            input: &input,
            presentation_input: serde_json::json!({"path": "new.txt", "content": "summary"}),
            summary_fields: vec![],
            deadline: Duration::from_secs(2),
        })
        .await;
    assert!(matches!(result, Err(PermissionPipelineError::Denied(_))));
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);
    assert!(!root.path().join("new.txt").exists());
}
