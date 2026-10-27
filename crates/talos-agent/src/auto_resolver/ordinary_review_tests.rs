use super::*;
use crate::permission_pipeline::{PermissionAuthorizationRequest, PermissionPipeline};
use std::sync::atomic::AtomicUsize;
use talos_permission::{PermissionContext, PermissionEngine, PermissionSessionState};

struct Assessor {
    scenario: &'static str,
    calls: Arc<AtomicUsize>,
    state: Arc<PermissionSessionState>,
}

#[async_trait]
impl AutoPermissionAssessor for Assessor {
    async fn assess(&self, _: AutoPermissionRequest, _: Duration) -> Result<String, String> {
        panic!("ordinary review must not downgrade")
    }
    async fn assess_ordinary_invocation(
        &self,
        request: OrdinaryInvocationAssessmentRequest,
        _: Duration,
    ) -> Result<String, String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(request.action.input["command"], "program");
        assert_eq!(request.configuration["backend"], "test");
        assert!(request.working_directory.is_some());
        assert!(request.directory_identity.is_some());
        let mut copy = request.clone();
        copy.action.request_digest.clear();
        let hash: String = Sha256::digest(bounded_json(&copy).expect("bounded").as_bytes())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        assert_eq!(request.action.request_digest, format!("sha256:{hash}"));
        match self.scenario {
            "stale" => self.state.rebind_session().expect("rebind"),
            "error" => return Err("unavailable".into()),
            "panic" => panic!("assessor panic"),
            "timeout" => std::future::pending::<()>().await,
            _ => {}
        }
        Ok(serde_json::json!({
            "schema": request.action.schema,
            "request_digest": if self.scenario == "digest" { "wrong" } else { &request.action.request_digest },
            "decision": if self.scenario == "uncertain" { "human_required" } else { "allow_once" },
            "confidence": "high",
        }).to_string())
    }
}

struct Deny;
#[async_trait]
impl ApprovalResolver for Deny {
    async fn resolve(
        &self,
        _: PermissionApprovalRequest,
        _: Duration,
    ) -> Result<ApprovalChoice, ApprovalResolverError> {
        Ok(ApprovalChoice::Deny)
    }
}

struct ReviewAdapter(AutoPermissionResolver);
#[async_trait]
impl ApprovalResolver for ReviewAdapter {
    async fn resolve(
        &self,
        _: PermissionApprovalRequest,
        _: Duration,
    ) -> Result<ApprovalChoice, ApprovalResolverError> {
        Ok(ApprovalChoice::Deny)
    }
    async fn resolve_builtin_invocation(
        &self,
        request: PermissionApprovalRequest,
        evidence: BuiltinInvocationReviewEvidence<'_>,
        remaining: Duration,
        allowed: bool,
        intent: Option<&str>,
    ) -> Result<ApprovalChoice, ApprovalResolverError> {
        self.0
            .resolve_action_review(request, evidence, remaining, allowed, intent)
            .await
    }
}

#[tokio::test]
async fn ordinary_review_binds_complete_evidence_and_fails_closed() {
    for scenario in [
        "allow",
        "uncertain",
        "digest",
        "stale",
        "error",
        "panic",
        "timeout",
        "missing_configuration",
        "unresolved_directories",
    ] {
        let root = tempfile::tempdir().expect("workspace");
        let state = Arc::new(PermissionSessionState::new(
            PermissionEngine::with_workspace_root(root.path().into()),
        ));
        let calls = Arc::new(AtomicUsize::new(0));
        let resolver = AutoPermissionResolver::new(
            Arc::new(Assessor {
                scenario,
                calls: calls.clone(),
                state: state.clone(),
            }),
            Arc::new(Deny),
            ManagedWorkspaceLease::for_permission_session(root.path(), state.clone())
                .expect("lease"),
            Duration::from_millis(20),
            AutoPermissionControl::new(true),
        );
        let pipeline = PermissionPipeline::new(
            state.clone(),
            PermissionContext::new(
                PermissionMode::Interactive,
                InteractionCapability::Available,
            ),
            Some(Arc::new(ReviewAdapter(resolver))),
        );
        let input = serde_json::json!({"command": "program"});
        let facets = [ToolPermissionFacet::with_resource(
            ToolNature::Execute,
            "program",
            ToolResourceKind::Command,
        )];
        let configuration = if scenario == "unresolved_directories" {
            serde_json::json!({"backend":"test", "execution_directories_resolved":false})
        } else {
            serde_json::json!({"backend": "test"})
        };
        let result = pipeline
            .authorize_registered_invocation(
                PermissionAuthorizationRequest {
                    tool_name: "exec",
                    provenance: ToolProvenance::Native,
                    profile: &facets,
                    input: &input,
                    presentation_input: input.clone(),
                    summary_fields: vec![],
                    deadline: Duration::from_secs(2),
                },
                Some("run program"),
                Some(root.path()),
                "direct program",
                if scenario == "missing_configuration" {
                    None
                } else {
                    Some(&configuration)
                },
            )
            .await;
        assert_eq!(
            result.is_ok(),
            scenario == "allow",
            "{scenario}: {result:?}"
        );
        assert_eq!(
            calls.load(Ordering::SeqCst),
            usize::from(!matches!(
                scenario,
                "missing_configuration" | "unresolved_directories"
            ))
        );
        assert_eq!(state.grant_count().expect("grants"), 0);
    }
}
