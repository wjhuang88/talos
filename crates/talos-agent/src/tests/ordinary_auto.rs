//! Actual command effects after whole-invocation model admission.
use super::*;
use crate::auto_resolver::OrdinaryInvocationAssessmentRequest;
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(unix)]
const SHELL_NAME: &str = "bash";
#[cfg(windows)]
const SHELL_NAME: &str = "powershell";
#[cfg(unix)]
const SHELL_COMMAND: &str = "mkdir nested && touch nested/effect";
#[cfg(windows)]
const SHELL_COMMAND: &str =
    "New-Item -ItemType Directory nested; New-Item -ItemType File nested/effect";

fn exec_input() -> serde_json::Value {
    #[cfg(unix)]
    return serde_json::json!({"steps":[
        {"command":"mkdir", "args":["nested"]},
        {"command":"touch", "args":["effect"], "cwd":"nested"}
    ]});
    #[cfg(windows)]
    return serde_json::json!({"steps":[
        {"command":"cmd.exe", "args":["/D", "/C", "mkdir nested"]},
        {"command":"cmd.exe", "args":["/D", "/C", "type nul > effect"], "cwd":"nested"}
    ]});
}

struct Assessor {
    scenario: &'static str,
    calls: Arc<AtomicUsize>,
    reached: Arc<AtomicUsize>,
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
            assert_eq!(request.action.tool, SHELL_NAME);
            assert_eq!(request.action.input["command"], SHELL_COMMAND);
        }
        self.reached.fetch_add(1, Ordering::SeqCst);
        match self.scenario {
            "error" => return Err("unavailable".into()),
            "stale" => self.state.rebind_session().expect("rebind"),
            "panic" => panic!("fixture assessor panic"),
            "timeout" => tokio::time::sleep(std::time::Duration::from_secs(3)).await,
            _ => {}
        }
        Ok(serde_json::json!({
            "schema":request.action.schema,
            "request_digest":if self.scenario == "digest" { "mismatched".into() } else { request.action.request_digest },
            "decision":if self.scenario == "uncertain" { "human_required" } else { "allow_once" },
            "confidence":"high"
        })
        .to_string())
    }
}

#[tokio::test]
async fn real_shell_and_exec_auto_control_effects() {
    for tool_name in ["exec", SHELL_NAME] {
        for scenario in [
            "allow",
            "uncertain",
            "error",
            "stale",
            "deny",
            "human",
            "panic",
            "timeout",
            "digest",
        ] {
            let root = tempfile::tempdir().expect("workspace");
            let mut registry = ToolRegistry::new();
            registry
                .register_builtin_contributions(talos_tools::shell_tool_contributions(
                    root.path().to_owned(),
                ))
                .expect("trusted registration");
            let mut engine = PermissionEngine::with_workspace_root(root.path().to_owned());
            if matches!(scenario, "deny" | "human") {
                engine.add_rule(talos_permission::PermissionRule::new(
                    tool_name,
                    None,
                    if scenario == "deny" {
                        PermissionDecision::Deny("explicit test policy".into())
                    } else {
                        PermissionDecision::Ask
                    },
                ));
            }
            let state = Arc::new(talos_permission::PermissionSessionState::new(engine));
            let calls = Arc::new(AtomicUsize::new(0));
            let reached = Arc::new(AtomicUsize::new(0));
            let resolver = AutoPermissionResolver::new(
                Arc::new(Assessor {
                    scenario,
                    calls: calls.clone(),
                    reached: reached.clone(),
                    state: state.clone(),
                }),
                Arc::new(AutoDenyFallback),
                ManagedWorkspaceLease::for_permission_session(root.path(), state.clone())
                    .expect("lease"),
                std::time::Duration::from_secs(2),
                AutoPermissionControl::new(true),
            );
            let input = if tool_name == SHELL_NAME {
                serde_json::json!({"command":SHELL_COMMAND})
            } else {
                exec_input()
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
            assert_eq!(
                calls.load(Ordering::SeqCst),
                usize::from(!matches!(scenario, "deny" | "human")),
                "{tool_name}/{scenario}"
            );
            assert_eq!(
                reached.load(Ordering::SeqCst),
                usize::from(!matches!(scenario, "deny" | "human")),
                "scenario not reached: {tool_name}/{scenario}"
            );
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
