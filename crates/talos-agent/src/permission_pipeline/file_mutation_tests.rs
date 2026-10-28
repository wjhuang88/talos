use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use talos_core::tool::{
    PreparedFileMutationDescription, PreparedFileMutationEvidence, PreparedPlanIdentity,
    ToolNature, ToolResourceKind,
};
use talos_permission::PermissionEngine;

struct MutationResolver {
    calls: Arc<AtomicUsize>,
    allow: bool,
}

#[async_trait]
impl ApprovalResolver for MutationResolver {
    async fn resolve(
        &self,
        _: PermissionApprovalRequest,
        _: Duration,
    ) -> Result<ApprovalChoice, ApprovalResolverError> {
        Ok(ApprovalChoice::Deny)
    }

    async fn resolve_builtin_file_mutation(
        &self,
        request: PermissionApprovalRequest,
        evidence: BuiltinFileMutationReviewEvidence<'_>,
        _: Duration,
        auto_allowed: bool,
        intent: Option<&str>,
    ) -> Result<ApprovalChoice, ApprovalResolverError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert!(auto_allowed);
        assert_eq!(intent, Some("replace the old greeting"));
        assert_eq!(request.arguments, serde_json::json!({"path":"<redacted>"}));
        assert_eq!(evidence.input()["new_string"], "hello");
        assert_eq!(evidence.profile().len(), 1);
        assert_eq!(evidence.mutation().description().before, "hi");
        assert_eq!(
            evidence.mutation().description().after.as_deref(),
            Some("hello")
        );
        Ok(if self.allow {
            ApprovalChoice::ApproveOnce
        } else {
            ApprovalChoice::Deny
        })
    }
}

#[tokio::test]
async fn mutation_admission_requires_instance_ownership_and_matching_plan() {
    for (builtin, matching, allow) in [
        (true, true, true),
        (true, true, false),
        (false, true, true),
        (true, false, true),
    ] {
        let root = tempfile::tempdir().expect("workspace");
        let path = root.path().join("greeting.txt");
        let state = Arc::new(PermissionSessionState::new(
            PermissionEngine::with_workspace_root(root.path().to_owned()),
        ));
        let calls = Arc::new(AtomicUsize::new(0));
        let pipeline = PermissionPipeline::new(
            state.clone(),
            PermissionContext::new(
                PermissionMode::Interactive,
                InteractionCapability::Available,
            ),
            Some(Arc::new(MutationResolver {
                calls: calls.clone(),
                allow,
            })),
        );
        let plan = PreparedPlanIdentity::new();
        let evidence = PreparedFileMutationEvidence::new(
            if matching {
                plan.clone()
            } else {
                PreparedPlanIdentity::new()
            },
            PreparedFileMutationDescription {
                path: path.clone(),
                before: "hi".into(),
                after: Some("hello".into()),
            },
        )
        .expect("evidence");
        let input = serde_json::json!({"path":path,"old_string":"hi","new_string":"hello"});
        let facets = [ToolPermissionFacet::with_resource(
            ToolNature::Write,
            path.to_string_lossy(),
            ToolResourceKind::Path,
        )];
        let authorization = PermissionAuthorizationRequest {
            tool_name: "edit",
            provenance: ToolProvenance::Native,
            profile: &facets,
            input: &input,
            presentation_input: serde_json::json!({"path":"<redacted>"}),
            summary_fields: vec![],
            deadline: Duration::from_secs(2),
        };
        let result = if builtin {
            pipeline
                .authorize_prepared_file_mutation(
                    authorization,
                    Some("replace the old greeting"),
                    &plan,
                    &evidence,
                )
                .await
        } else {
            pipeline
                .authorize_registered_file_mutation(
                    authorization,
                    Some("replace the old greeting"),
                    &plan,
                    &evidence,
                    false,
                )
                .await
        };
        assert_eq!(
            calls.load(Ordering::SeqCst),
            usize::from(builtin && matching)
        );
        if builtin && matching && allow {
            let request = PermissionRequest::new("edit", ToolProvenance::Native, &facets, &input);
            result
                .expect("approved plan")
                .consume_for_prepared(&request, &plan)
                .expect("exact proof");
        } else {
            assert!(
                result.is_err(),
                "untrusted, stale or denied change cannot be admitted"
            );
        }
        assert_eq!(state.grant_count().expect("no persistent grants"), 0);
    }
}
