//! Exact-input Auto review followed by real capability-bound file creation.
use super::*;
use crate::auto_resolver::BuiltinInvocationAssessmentRequest;
use std::sync::atomic::{AtomicUsize, Ordering};
use talos_core::tool::{ToolContribution, ToolContributionSource};

struct Assessor {
    calls: Arc<AtomicUsize>,
    scenario: &'static str,
    state: Arc<talos_permission::PermissionSessionState>,
    control: AutoPermissionControl,
}
#[async_trait]
impl AutoPermissionAssessor for Assessor {
    async fn assess(
        &self,
        _: AutoPermissionRequest,
        _: std::time::Duration,
    ) -> Result<String, String> {
        panic!("trusted creation must review exact input")
    }
    async fn assess_builtin_invocation(
        &self,
        request: BuiltinInvocationAssessmentRequest,
        _: std::time::Duration,
    ) -> Result<String, String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(
            request.input,
            serde_json::json!({"path":"new.txt","content":"hello"})
        );
        assert_eq!(request.user_intent, "create new.txt containing hello");
        assert_eq!(request.facets.len(), 1);
        match self.scenario {
            "error" => return Err("fixture error".into()),
            "cancel" => self.control.set_enabled(false),
            "stale" => {
                self.state.rebind_session().expect("rebind");
            }
            _ => {}
        }
        Ok(
            serde_json::json!({"schema":request.schema,"request_digest":request.request_digest,
            "decision":if self.scenario == "uncertain" {"human_required"} else {"allow_once"},"confidence":"high"})
            .to_string(),
        )
    }
}

#[tokio::test]
async fn trusted_creation_reviews_content_and_intent_before_real_execution() {
    for scenario in [
        "allow",
        "uncertain",
        "error",
        "cancel",
        "stale",
        "oversized",
    ] {
        let allow = scenario == "allow";
        let root = tempfile::tempdir().expect("workspace");
        let capability = Arc::new(
            talos_tools::file_tools::CapStdAtomicCreateCapability::open(root.path())
                .expect("capability"),
        );
        let mut registry = ToolRegistry::new();
        registry
            .register_builtin_contributions([ToolContribution::new(
                ToolContributionSource::new("talos-tools:file"),
                Arc::new(
                    talos_tools::file_tools::WriteTool::new(root.path().to_owned())
                        .with_atomic_create_capability(capability.clone()),
                ) as Arc<dyn AgentTool>,
            )])
            .expect("registration");
        let state = Arc::new(talos_permission::PermissionSessionState::new(
            PermissionEngine::with_workspace_root(root.path().to_owned()),
        ));
        let calls = Arc::new(AtomicUsize::new(0));
        let control = AutoPermissionControl::new(true);
        let resolver = AutoPermissionResolver::new(
            Arc::new(Assessor {
                calls: calls.clone(),
                scenario,
                state: state.clone(),
                control: control.clone(),
            }),
            Arc::new(AutoDenyFallback),
            ManagedWorkspaceLease::for_permission_session(root.path(), state.clone())
                .expect("lease")
                .with_atomic_create_capability(capability),
            std::time::Duration::from_secs(2),
            control,
        );
        let content = if scenario == "oversized" {
            "x".repeat(20 * 1024)
        } else {
            "hello".into()
        };
        let responses = vec![
            vec![
                AgentEvent::ToolCall {
                    call: ToolCall {
                        id: "create".into(),
                        name: "write".into(),
                        input: serde_json::json!({"path":"new.txt","content":content}),
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
        ];
        let agent = Agent::with_security(
            Arc::new(MockModel::new(responses)),
            registry,
            None,
            None,
            root.path().to_owned(),
        )
        .with_permission_pipeline(
            state.clone(),
            talos_permission::PermissionContext::new(
                talos_permission::PermissionMode::Interactive,
                talos_permission::InteractionCapability::Available,
            ),
            Some(Arc::new(resolver)),
        );
        agent
            .run("create new.txt containing hello".into())
            .await
            .expect("run");
        assert_eq!(
            calls.load(Ordering::SeqCst),
            usize::from(scenario != "oversized"),
            "{scenario}"
        );
        assert_eq!(root.path().join("new.txt").exists(), allow);
        if allow {
            assert_eq!(
                std::fs::read(root.path().join("new.txt")).expect("file"),
                b"hello"
            );
        }
        assert_eq!(state.grant_count().expect("grants"), 0);
    }
}
