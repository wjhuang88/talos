//! Production Agent dispatch with an offline prepared-transport fixture.
use super::*;
use crate::auto_resolver::BuiltinNetworkAssessmentRequest;
use talos_core::tool::{ToolContribution, ToolContributionSource};

struct NetworkAssessor(Arc<AtomicUsize>);

#[async_trait]
impl AutoPermissionAssessor for NetworkAssessor {
    async fn assess(
        &self,
        _: AutoPermissionRequest,
        _: std::time::Duration,
    ) -> Result<String, String> {
        panic!("network execution must use retained evidence")
    }

    async fn assess_builtin_network(
        &self,
        request: BuiltinNetworkAssessmentRequest,
        _: std::time::Duration,
    ) -> Result<String, String> {
        self.0.fetch_add(1, Ordering::SeqCst);
        assert_eq!(request.input["query"], "rust documentation");
        assert_eq!(request.endpoints[0].host, "search.example");
        assert_eq!(request.user_intent, "search rust documentation");
        Ok(serde_json::json!({
            "schema": request.schema,
            "request_digest": request.request_digest,
            "decision": "allow_once",
            "confidence": "high"
        })
        .to_string())
    }
}

#[tokio::test]
async fn auto_network_dispatch_requires_builtin_ownership_and_preserves_deny() {
    for scenario in ["builtin", "external", "replacement", "deny", "policy_allow"] {
        let root = tempfile::tempdir().expect("workspace");
        let mut engine = PermissionEngine::with_workspace_root(root.path().to_owned());
        if matches!(scenario, "deny" | "policy_allow") {
            engine = PermissionEngine::empty();
            engine.set_workspace_root(root.path().to_owned());
            engine.add_rule(talos_permission::PermissionRule::new_nature(
                ToolNature::Network,
                None,
                None,
                if scenario == "deny" {
                    PermissionDecision::Deny("explicit policy".into())
                } else {
                    PermissionDecision::Allow
                },
            ));
        }
        let state = Arc::new(PermissionSessionState::new(engine));
        let prepared = Arc::new(AtomicUsize::new(0));
        let executed = Arc::new(Mutex::new(Vec::new()));
        let tool: Arc<dyn AgentTool> = Arc::new(NetworkFixture {
            configuration_id: uuid::Uuid::new_v4(),
            prepared: prepared.clone(),
            executed: executed.clone(),
            missing: false,
            revoke: None,
        });
        let mut registry = ToolRegistry::new();
        if scenario == "external" {
            registry.register(tool.clone());
        } else {
            registry
                .register_builtin_contributions([ToolContribution::new(
                    ToolContributionSource::new("talos-tools:network-fixture"),
                    tool.clone(),
                )])
                .expect("builtin registration");
        }
        if scenario == "replacement" {
            registry.register(tool);
        }
        let assessments = Arc::new(AtomicUsize::new(0));
        let resolver = AutoPermissionResolver::new(
            Arc::new(NetworkAssessor(assessments.clone())),
            Arc::new(AutoDenyFallback),
            ManagedWorkspaceLease::for_permission_session(root.path(), state.clone())
                .expect("lease"),
            std::time::Duration::from_secs(2),
            AutoPermissionControl::new(true),
        );
        let input = serde_json::json!({"query": "rust documentation"});
        let agent = Agent::with_security(
            Arc::new(MockModel::new(events(input.clone()))),
            registry,
            None,
            None,
            root.path().to_owned(),
        )
        .with_permission_pipeline(
            state.clone(),
            PermissionContext::new(
                PermissionMode::Interactive,
                InteractionCapability::Available,
            ),
            Some(Arc::new(resolver)),
        );
        agent
            .run("search rust documentation".into())
            .await
            .expect("Agent run");
        assert_eq!(prepared.load(Ordering::SeqCst), 1, "{scenario}");
        assert_eq!(
            assessments.load(Ordering::SeqCst),
            usize::from(scenario == "builtin"),
            "{scenario}"
        );
        let expected = if matches!(scenario, "builtin" | "policy_allow") {
            vec![input]
        } else {
            vec![]
        };
        assert_eq!(*executed.lock().await, expected, "{scenario}");
        assert_eq!(state.grant_count().expect("grants"), 0, "{scenario}");
    }
}
