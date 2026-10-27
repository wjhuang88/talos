//! Actual command effects after whole-invocation model admission.
use super::*;
use crate::auto_resolver::OrdinaryInvocationAssessmentRequest;
use std::sync::atomic::{AtomicUsize, Ordering};

struct Assessor {
    scenario: &'static str,
    calls: Arc<AtomicUsize>,
    state: Arc<talos_permission::PermissionSessionState>,
}

#[async_trait]
impl AutoPermissionAssessor for Assessor {
    async fn assess(
        &self,
        _: AutoPermissionRequest,
        _: std::time::Duration,
    ) -> Result<String, String> {
        panic!("trusted command must use whole-invocation review")
    }

    async fn assess_ordinary_invocation(
        &self,
        request: OrdinaryInvocationAssessmentRequest,
        _: std::time::Duration,
    ) -> Result<String, String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(request.action.schema, "ordinary-invocation/v1");
        assert!(request.directory_identity.is_some());
        if request.action.tool == "exec" {
            assert_eq!(
                request.configuration["execution_directories_resolved"],
                false
            );
            assert_eq!(request.action.input["steps"][1]["cwd"], "nested");
        } else {
            assert_eq!(request.action.tool, "bash");
            assert_eq!(
                request.action.input["command"],
                "mkdir nested && touch nested/effect"
            );
        }
        match self.scenario {
            "error" => return Err("unavailable".into()),
            "stale" => self.state.rebind_session().expect("rebind"),
            _ => {}
        }
        Ok(serde_json::json!({
            "schema":request.action.schema,
            "request_digest":request.action.request_digest,
            "decision":if self.scenario == "uncertain" { "human_required" } else { "allow_once" },
            "confidence":"high"
        })
        .to_string())
    }
}

#[cfg(unix)]
#[tokio::test]
async fn real_shell_and_exec_auto_control_effects() {
    for tool_name in ["exec", "bash"] {
        for scenario in ["allow", "uncertain", "error", "stale"] {
            let root = tempfile::tempdir().expect("workspace");
            let mut registry = ToolRegistry::new();
            registry
                .register_builtin_contributions(talos_tools::shell_tool_contributions(
                    root.path().to_owned(),
                ))
                .expect("trusted registration");
            let state = Arc::new(talos_permission::PermissionSessionState::new(
                PermissionEngine::with_workspace_root(root.path().to_owned()),
            ));
            let calls = Arc::new(AtomicUsize::new(0));
            let resolver = AutoPermissionResolver::new(
                Arc::new(Assessor {
                    scenario,
                    calls: calls.clone(),
                    state: state.clone(),
                }),
                Arc::new(AutoDenyFallback),
                ManagedWorkspaceLease::for_permission_session(root.path(), state.clone())
                    .expect("lease"),
                std::time::Duration::from_secs(2),
                AutoPermissionControl::new(true),
            );
            let input = if tool_name == "bash" {
                serde_json::json!({"command":"mkdir nested && touch nested/effect"})
            } else {
                serde_json::json!({"steps":[
                    {"command":"mkdir", "args":["nested"]},
                    {"command":"touch", "args":["effect"], "cwd":"nested"}
                ]})
            };
            let model = MockModel::new(vec![
                vec![
                    AgentEvent::ToolCall {
                        call: ToolCall {
                            id: "command".into(),
                            name: tool_name.into(),
                            input,
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
            let agent = Agent::with_security(
                Arc::new(model),
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
                .run("create nested and then create an empty effect file inside it".into())
                .await
                .expect("run");
            assert_eq!(calls.load(Ordering::SeqCst), 1, "{scenario}");
            assert_eq!(
                root.path().join("nested/effect").exists(),
                scenario == "allow",
                "{scenario}"
            );
            assert_eq!(
                root.path().join("nested").exists(),
                scenario == "allow",
                "{scenario}"
            );
            assert_eq!(state.grant_count().expect("grants"), 0);
        }
    }
}
