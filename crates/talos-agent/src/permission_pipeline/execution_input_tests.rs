use super::*;
use talos_core::tool::{PreparedPlanIdentity, ToolNature, ToolResourceKind};
use talos_permission::PermissionEngine;

struct InputResolver;

// Models an existing adapter which implements only the exact-input extension.
struct LegacyInputResolver;

#[async_trait]
impl ApprovalResolver for LegacyInputResolver {
    async fn resolve(
        &self,
        request: PermissionApprovalRequest,
        remaining: Duration,
    ) -> Result<ApprovalChoice, ApprovalResolverError> {
        InputResolver.resolve(request, remaining).await
    }

    async fn resolve_with_execution_input(
        &self,
        request: PermissionApprovalRequest,
        input: &serde_json::Value,
        remaining: Duration,
        auto_allowed: bool,
        intent: Option<&str>,
    ) -> Result<ApprovalChoice, ApprovalResolverError> {
        InputResolver
            .resolve_with_execution_input(request, input, remaining, auto_allowed, intent)
            .await
    }
}

#[async_trait]
impl ApprovalResolver for InputResolver {
    async fn resolve_builtin_invocation(
        &self,
        request: PermissionApprovalRequest,
        evidence: BuiltinInvocationReviewEvidence<'_>,
        remaining: Duration,
        auto_assessment_allowed: bool,
        user_intent: Option<&str>,
    ) -> Result<ApprovalChoice, ApprovalResolverError> {
        assert_eq!(evidence.description(), "Update the current repository");
        assert_eq!(evidence.profile().len(), 1);
        assert_eq!(evidence.profile()[0].nature, ToolNature::Execute);
        assert!(evidence.working_directory().is_some());
        assert_eq!(
            evidence.configuration(),
            Some(&serde_json::json!({"backend": "test-retained"}))
        );
        self.resolve_with_execution_input(
            request,
            evidence.input(),
            remaining,
            auto_assessment_allowed,
            user_intent,
        )
        .await
    }
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
    for route in ["ordinary", "prepared", "registered", "registered_legacy"] {
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
            Some(if route == "registered_legacy" {
                Arc::new(LegacyInputResolver) as Arc<dyn ApprovalResolver>
            } else {
                Arc::new(InputResolver)
            }),
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
        if route == "prepared" {
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
        } else if route.starts_with("registered") {
            pipeline
                .authorize_registered_invocation(
                    authorization,
                    Some("update this repository"),
                    Some(root.path()),
                    "Update the current repository",
                    Some(&serde_json::json!({"backend": "test-retained"})),
                )
                .await
                .expect("builtin approval");
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

#[tokio::test]
async fn network_manifest_reaches_resolver_without_replacing_exact_input() {
    use talos_core::tool::{ToolNetworkEndpoint, ToolNetworkScope};

    struct NetworkResolver(ToolNetworkManifest);
    #[async_trait]
    impl ApprovalResolver for NetworkResolver {
        async fn resolve(
            &self,
            _: PermissionApprovalRequest,
            _: Duration,
        ) -> Result<ApprovalChoice, ApprovalResolverError> {
            panic!("network evidence must reach the dedicated resolver")
        }

        async fn resolve_network_invocation(
            &self,
            request: PermissionApprovalRequest,
            input: &serde_json::Value,
            manifest: &ToolNetworkManifest,
            _: Duration,
            auto_allowed: bool,
            intent: Option<&str>,
        ) -> Result<ApprovalChoice, ApprovalResolverError> {
            assert_eq!(manifest, &self.0);
            assert_eq!(input["query"], "private query");
            assert_eq!(request.arguments["query"], "<redacted>");
            assert!(auto_allowed);
            assert_eq!(intent, Some("search documentation"));
            Ok(ApprovalChoice::Deny)
        }
    }

    let endpoint =
        ToolNetworkEndpoint::new("https", "example.com", 443, "GET", "/search").expect("endpoint");
    let manifest = ToolNetworkManifest::new(
        ToolNetworkScope::new(uuid::Uuid::new_v4(), 0, vec![endpoint], None, vec![], 0)
            .expect("scope"),
    );
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
        Some(Arc::new(NetworkResolver(manifest.clone()))),
    );
    let input = serde_json::json!({"query": "private query"});
    let facets = [ToolPermissionFacet::with_resource(
        ToolNature::Network,
        "example.com",
        ToolResourceKind::Domain,
    )];
    let result = pipeline
        .authorize_network(
            PermissionAuthorizationRequest {
                tool_name: "web_search",
                provenance: ToolProvenance::Native,
                profile: &facets,
                input: &input,
                presentation_input: serde_json::json!({"query": "<redacted>"}),
                summary_fields: vec![],
                deadline: Duration::from_secs(2),
            },
            &manifest,
            Some("search documentation"),
        )
        .await;
    assert!(
        matches!(result, Err(PermissionPipelineError::Denied(_))),
        "{}",
        result.err().map(|e| e.to_string()).unwrap_or_default()
    );
    assert_eq!(state.grant_count().expect("grants"), 0);
}

#[tokio::test]
async fn legacy_resolver_preserves_exact_input_network_fallback() {
    use talos_core::tool::{ToolNetworkEndpoint, ToolNetworkScope};
    let endpoint =
        ToolNetworkEndpoint::new("https", "example.com", 443, "GET", "/").expect("endpoint");
    let manifest = ToolNetworkManifest::new(
        ToolNetworkScope::new(uuid::Uuid::new_v4(), 0, vec![endpoint], None, vec![], 0)
            .expect("scope"),
    );
    let root = tempfile::tempdir().expect("workspace");
    let state = Arc::new(PermissionSessionState::new(
        PermissionEngine::with_workspace_root(root.path().to_owned()),
    ));
    let pipeline = PermissionPipeline::new(
        state,
        PermissionContext::new(
            PermissionMode::Interactive,
            InteractionCapability::Available,
        ),
        Some(Arc::new(InputResolver)),
    );
    let input = serde_json::json!({"remote": "origin"});
    let facets = [ToolPermissionFacet::with_resource(
        ToolNature::Network,
        "example.com",
        ToolResourceKind::Domain,
    )];
    pipeline
        .authorize_network(
            PermissionAuthorizationRequest {
                tool_name: "legacy_network",
                provenance: ToolProvenance::Native,
                profile: &facets,
                input: &input,
                presentation_input: serde_json::json!({"remote": "<redacted>"}),
                summary_fields: vec![],
                deadline: Duration::from_secs(2),
            },
            &manifest,
            Some("update this repository"),
        )
        .await
        .expect("legacy adapter still approves through its original method");
}

#[tokio::test]
async fn ordinary_invocation_restrictions_precede_deterministic_allow() {
    use talos_permission::{PermissionDecision, PermissionRule};
    for tool in ["exec", "http_request", "bash", "powershell"] {
        for restriction in [
            PermissionDecision::Ask,
            PermissionDecision::Deny("private".into()),
        ] {
            let root = tempfile::tempdir().expect("workspace");
            let mut engine = PermissionEngine::with_workspace_root(root.path().into());
            engine.add_rule(PermissionRule::new_nature(
                ToolNature::Execute,
                None,
                None,
                PermissionDecision::Allow,
            ));
            engine.add_rule(PermissionRule::new_nature(
                ToolNature::Write,
                Some("undisclosed-output/**".into()),
                None,
                restriction,
            ));
            let state = Arc::new(PermissionSessionState::new(engine));
            let pipeline = PermissionPipeline::new(
                state.clone(),
                PermissionContext::new(
                    PermissionMode::Interactive,
                    InteractionCapability::Available,
                ),
                None,
            );
            let facets = [ToolPermissionFacet::with_resource(
                ToolNature::Execute,
                "test",
                ToolResourceKind::Command,
            )];
            let input = serde_json::json!({"command": "test"});
            let result = pipeline
                .authorize_registered_invocation(
                    PermissionAuthorizationRequest {
                        tool_name: tool,
                        provenance: ToolProvenance::Native,
                        profile: &facets,
                        input: &input,
                        presentation_input: input.clone(),
                        summary_fields: vec![],
                        deadline: Duration::from_secs(2),
                    },
                    Some("run the requested operation"),
                    Some(root.path()),
                    "test builtin",
                    None,
                )
                .await;
            assert!(
                matches!(result, Err(PermissionPipelineError::Denied(_))),
                "{tool}: {result:?}"
            );
            assert_eq!(state.grant_count().expect("grants"), 0);
        }
    }
}
