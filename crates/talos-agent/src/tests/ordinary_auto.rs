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
    control: AutoPermissionControl,
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
        let previous_calls = self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(request.action.schema, "ordinary-invocation/v1");
        assert!(request.directory_identity.is_some());
        if previous_calls == 1 && self.scenario == "changed_input" {
            if request.action.tool == "exec" {
                assert_eq!(request.action.input["steps"][0]["command"], "echo");
            } else {
                assert_eq!(request.action.input["command"], "echo unexpected");
            }
        } else if request.action.tool == "exec" {
            assert_eq!(
                request.configuration["execution_directories_resolved"],
                previous_calls > 0
            );
            assert_eq!(request.action.input["steps"][1]["cwd"], "nested");
        } else {
            assert_eq!(request.action.tool, SHELL_NAME);
            assert_eq!(request.action.input["command"], SHELL_COMMAND);
        }
        self.reached.fetch_add(1, Ordering::SeqCst);
        match self.scenario {
            "cancel" => self.control.set_enabled(false),
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
            "cancel",
            "external",
            "replacement",
            "replay",
            "changed_input",
        ] {
            let root = tempfile::tempdir().expect("workspace");
            let mut registry = ToolRegistry::new();
            let contributions = talos_tools::shell_tool_contributions(root.path().to_owned());
            let replacement = talos_tools::shell_tool_contributions(root.path().to_owned());
            let same_name = replacement
                .iter()
                .find(|entry| entry.name() == tool_name)
                .expect("production tool")
                .tool()
                .clone();
            if scenario != "external" {
                registry
                    .register_builtin_contributions(contributions)
                    .expect("trusted registration");
            }
            if matches!(scenario, "external" | "replacement") {
                registry.register(same_name);
            }
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
            let control = AutoPermissionControl::new(true);
            let resolver = AutoPermissionResolver::new(
                Arc::new(Assessor {
                    scenario,
                    calls: calls.clone(),
                    reached: reached.clone(),
                    state: state.clone(),
                    control: control.clone(),
                }),
                Arc::new(AutoDenyFallback),
                ManagedWorkspaceLease::for_permission_session(root.path(), state.clone())
                    .expect("lease"),
                std::time::Duration::from_secs(2),
                control,
            );
            let input = if tool_name == SHELL_NAME {
                serde_json::json!({"command":SHELL_COMMAND})
            } else {
                exec_input()
            };
            let mut responses = vec![
                vec![
                    AgentEvent::ToolCall {
                        call: ToolCall {
                            id: "command".into(),
                            name: tool_name.into(),
                            input: input.clone(),
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
            if matches!(scenario, "replay" | "changed_input") {
                let mut repeated = responses[0].clone();
                if scenario == "changed_input" {
                    for event in &mut repeated {
                        if let AgentEvent::ToolCall { call, .. } = event {
                            // The final execution reservation must reject a changed payload with the same ID.
                            call.input = if tool_name == SHELL_NAME {
                                serde_json::json!({"command":"echo unexpected"})
                            } else {
                                serde_json::json!({"steps":[{"command":"echo","args":["unexpected"]}]})
                            };
                        }
                    }
                }
                responses.insert(1, repeated);
            }
            let (model, captured) = CapturingMockModel::new(responses);
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
            let expected_calls = if matches!(scenario, "replay" | "changed_input") {
                2
            } else {
                usize::from(!matches!(
                    scenario,
                    "deny" | "human" | "external" | "replacement"
                ))
            };
            assert_eq!(
                calls.load(Ordering::SeqCst),
                expected_calls,
                "{tool_name}/{scenario}"
            );
            assert_eq!(
                reached.load(Ordering::SeqCst),
                expected_calls,
                "scenario not reached: {tool_name}/{scenario}"
            );
            assert_eq!(
                root.path().join("nested/effect").exists(),
                matches!(scenario, "allow" | "replay" | "changed_input"),
                "{scenario}"
            );
            assert_eq!(
                root.path().join("nested").exists(),
                matches!(scenario, "allow" | "replay" | "changed_input"),
                "{scenario}"
            );
            if matches!(scenario, "replay" | "changed_input") {
                let messages = captured.lock().await;
                let results: Vec<_> = messages
                    .last()
                    .expect("continuation")
                    .iter()
                    .filter_map(|message| match message {
                        Message::Tool { result } if result.tool_use_id == "command" => Some(result),
                        _ => None,
                    })
                    .collect();
                assert_eq!(results.len(), 2, "{tool_name}/{scenario}");
                assert!(!results[0].is_error, "{}", results[0].content);
                assert!(results[1].is_error);
                let expected = if scenario == "replay" {
                    "refusing replay"
                } else {
                    "reused with different arguments"
                };
                assert!(
                    results[1].content.contains(expected),
                    "{}",
                    results[1].content
                );
            }
            assert_eq!(state.grant_count().expect("grants"), 0);
        }
    }
}
