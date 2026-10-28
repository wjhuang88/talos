//! Concrete legacy WebSearch fails closed without a typed network resource.
use super::*;
use crate::auto_resolver::{BuiltinNetworkAssessmentRequest, OrdinaryInvocationAssessmentRequest};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Assessor(Arc<AtomicUsize>);
#[async_trait]
impl AutoPermissionAssessor for Assessor {
    async fn assess(
        &self,
        _: AutoPermissionRequest,
        _: std::time::Duration,
    ) -> Result<String, String> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Err("search must not use legacy read-only review".into())
    }
    async fn assess_ordinary_invocation(
        &self,
        _: OrdinaryInvocationAssessmentRequest,
        _: std::time::Duration,
    ) -> Result<String, String> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Err("search must not use ordinary HTTP review".into())
    }
    async fn assess_builtin_network(
        &self,
        _: BuiltinNetworkAssessmentRequest,
        _: std::time::Duration,
    ) -> Result<String, String> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Err("legacy search has no prepared network evidence".into())
    }
}
struct Human(Arc<AtomicUsize>);
#[async_trait]
impl ApprovalResolver for Human {
    async fn resolve(
        &self,
        _: PermissionApprovalRequest,
        _: std::time::Duration,
    ) -> Result<talos_core::ApprovalChoice, ApprovalResolverError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(talos_core::ApprovalChoice::ApproveOnce)
    }
}

#[tokio::test]
async fn concrete_legacy_web_search_cannot_downgrade_to_http_auto_or_manual_admission() {
    for scenario in [
        "auto_fallback",
        "manual",
        "deny",
        "human_policy",
        "policy_allow",
        "headless",
    ] {
        let root = tempfile::tempdir().expect("workspace");
        let mut registry = ToolRegistry::new();
        registry
            .register_builtin_contributions(talos_tools::network_tool_contributions())
            .expect("production registration");
        let search = registry.get("web_search").expect("concrete WebSearch");
        assert!(registry.is_builtin_instance("web_search"));
        assert!(
            !search.requires_network_invocation(),
            "legacy search contract changed: reconcile ADR-090 coverage"
        );
        let mut engine = PermissionEngine::with_workspace_root(root.path().into());
        if matches!(scenario, "deny" | "human_policy" | "policy_allow") {
            engine.add_rule(talos_permission::PermissionRule::new(
                "web_search",
                None,
                match scenario {
                    "deny" => PermissionDecision::Deny("explicit search denial".into()),
                    "policy_allow" => PermissionDecision::Allow,
                    _ => PermissionDecision::Ask,
                },
            ));
        }
        let state = Arc::new(talos_permission::PermissionSessionState::new(engine));
        let assessed = Arc::new(AtomicUsize::new(0));
        let humans = Arc::new(AtomicUsize::new(0));
        let resolver = AutoPermissionResolver::new(
            Arc::new(Assessor(assessed.clone())),
            Arc::new(Human(humans.clone())),
            ManagedWorkspaceLease::for_permission_session(root.path(), state.clone())
                .expect("lease"),
            std::time::Duration::from_secs(2),
            AutoPermissionControl::new(scenario != "manual"),
        );
        // Empty query prevents external I/O even if admission unexpectedly opens.
        // It must not reach implementation validation under this legacy contract.
        let model = MockModel::new(vec![
            vec![
                AgentEvent::ToolCall {
                    call: ToolCall {
                        id: "search".into(),
                        name: "web_search".into(),
                        input: serde_json::json!({"query":""}),
                    },
                    provenance: Default::default(),
                    summary_fields: vec![],
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::ToolUse,
                    usage: Usage::default(),
                },
            ],
            text_done_events("done"),
        ]);
        let headless = scenario == "headless";
        let mut agent =
            Agent::with_security(Arc::new(model), registry, None, None, root.path().into())
                .with_permission_pipeline(
                    state.clone(),
                    talos_permission::PermissionContext::new(
                        if headless {
                            talos_permission::PermissionMode::Headless
                        } else {
                            talos_permission::PermissionMode::Interactive
                        },
                        if headless {
                            talos_permission::InteractionCapability::Unavailable
                        } else {
                            talos_permission::InteractionCapability::Available
                        },
                    ),
                    Some(Arc::new(resolver)),
                );
        agent.set_tool_presentation_policy(ToolPresentationPolicy::with_families([
            ToolFamily::Network,
            ToolFamily::AdvancedNetwork,
        ]));
        let (tx, mut rx) = mpsc::unbounded_channel();
        agent
            .run_streaming("search current Rust documentation".into(), vec![], tx)
            .await
            .expect("run");
        let mut results = 0;
        while let Ok(event) = rx.try_recv() {
            if let AgentEvent::ToolResult { result, .. } = event {
                results += 1;
                assert!(result.is_error, "{scenario}: {}", result.content);
                assert!(
                    result.content.contains("Permission denied"),
                    "{scenario}: {}",
                    result.content
                );
                assert!(!result.content.contains("query must not be empty"));
                if scenario != "deny" {
                    assert!(
                        result.content.contains("no safe typed resource"),
                        "{scenario}: {}",
                        result.content
                    );
                }
            }
        }
        assert_eq!(results, 1, "{scenario}");
        assert_eq!(assessed.load(Ordering::SeqCst), 0, "{scenario}");
        assert_eq!(humans.load(Ordering::SeqCst), 0, "{scenario}");
        assert_eq!(state.grant_count().expect("grants"), 0, "{scenario}");
    }
}
