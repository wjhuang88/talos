//! Model approval admits only one real scheduler invocation.
use super::*;
use crate::auto_resolver::BuiltinActionAssessmentRequest;
use std::sync::atomic::{AtomicUsize, Ordering};
use talos_core::tool::{ToolContribution, ToolContributionSource};

struct Assessor {
    calls: Arc<AtomicUsize>,
    scenario: &'static str,
    input: Value,
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
        panic!("scheduler must use exact action evidence")
    }
    async fn assess_builtin_action(
        &self,
        request: BuiltinActionAssessmentRequest,
        _: std::time::Duration,
    ) -> Result<String, String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(request.input, self.input);
        assert_eq!(
            request.user_intent,
            "perform this session scheduling action"
        );
        assert_eq!(
            request.facets,
            crate::scheduler::action_permission_profile(&request.tool, &request.input)
        );
        assert!(!request.description.is_empty());
        assert!(!request.session_binding.is_empty());
        match self.scenario {
            "stale" => {
                self.state.rebind_session().expect("rebind");
            }
            "disabled" => self.control.set_enabled(false),
            "error" => return Err("assessment unavailable".into()),
            "panic" => panic!("assessment failure"),
            "timeout" => std::future::pending::<()>().await,
            _ => {}
        }
        let mut response = serde_json::json!({"schema":request.schema,"request_digest":request.request_digest,"decision":"allow_once","confidence":"high"});
        match self.scenario {
            "uncertain" => response["decision"] = "human_required".into(),
            "low" => response["confidence"] = "low".into(),
            "digest" => response["request_digest"] = "wrong".into(),
            "schema" => response["schema"] = "wrong".into(),
            "extra" => response["grant"] = true.into(),
            "malformed" => return Ok("allow".into()),
            "large_response" => return Ok("x".repeat(64 * 1024)),
            _ => {}
        }
        Ok(response.to_string())
    }
}

#[tokio::test]
async fn scheduler_auto_reviews_exact_actions_and_preserves_policy() {
    for name in ["delay", "schedule", "cancel_scheduled_task"] {
        for scenario in [
            "allow",
            "deny",
            "human",
            "external",
            "uncertain",
            "low",
            "digest",
            "schema",
            "extra",
            "malformed",
            "large_response",
            "error",
            "panic",
            "timeout",
            "stale",
            "disabled",
            "oversized",
        ] {
            let root = tempfile::tempdir().expect("workspace");
            let (tools, pending) = crate::create_scheduler_tools();
            let list = tools
                .iter()
                .find(|tool| tool.name() == "list_scheduled_tasks")
                .expect("list")
                .clone();
            let delay = tools
                .iter()
                .find(|tool| tool.name() == "delay")
                .expect("delay")
                .clone();
            let cancel = tokio_util::sync::CancellationToken::new();
            let (sq_tx, _sq_rx) = mpsc::channel(4);
            let actor = pending.spawn(sq_tx, 0, cancel.clone());
            let message = if scenario == "oversized" {
                "x".repeat(20 * 1024)
            } else {
                "check progress".into()
            };
            let mut input = match name {
                "delay" => serde_json::json!({"message":message,"delay_secs":3600}),
                "schedule" => serde_json::json!({"message":message,"interval_secs":3600}),
                _ => {
                    let result = delay
                        .execute(serde_json::json!({"message":"existing","delay_secs":3600}))
                        .await;
                    assert!(!result.is_error, "{}", result.content);
                    let id = result
                        .content
                        .split_once("Task ID: ")
                        .expect("id")
                        .1
                        .lines()
                        .next()
                        .expect("id");
                    serde_json::json!({"task_id":id})
                }
            };
            if scenario == "oversized" && name == "cancel_scheduled_task" {
                input["extra"] = "x".repeat(20 * 1024).into();
            }
            let mut registry = ToolRegistry::new();
            if scenario == "external" {
                for tool in tools {
                    registry.register(tool);
                }
            } else {
                registry
                    .register_builtin_contributions(tools.into_iter().map(|tool| {
                        ToolContribution::new(
                            ToolContributionSource::new("talos-agent:scheduler"),
                            tool,
                        )
                    }))
                    .expect("registration");
            }
            let mut engine = if matches!(scenario, "deny" | "human") {
                let mut engine = PermissionEngine::empty();
                engine.set_workspace_root(root.path().into());
                engine
            } else {
                PermissionEngine::with_workspace_root(root.path().into())
            };
            if matches!(scenario, "deny" | "human") {
                engine.add_rule(talos_permission::PermissionRule::new_nature(
                    ToolNature::Execute,
                    None,
                    None,
                    if scenario == "human" {
                        PermissionDecision::Ask
                    } else {
                        PermissionDecision::Deny("denied".into())
                    },
                ));
            }
            let state = Arc::new(talos_permission::PermissionSessionState::new(engine));
            let calls = Arc::new(AtomicUsize::new(0));
            let control = AutoPermissionControl::new(true);
            let resolver = AutoPermissionResolver::new(
                Arc::new(Assessor {
                    calls: calls.clone(),
                    scenario,
                    input: input.clone(),
                    state: state.clone(),
                    control: control.clone(),
                }),
                Arc::new(AutoDenyFallback),
                ManagedWorkspaceLease::for_permission_session(root.path(), state.clone())
                    .expect("lease"),
                std::time::Duration::from_millis(100),
                control,
            );
            let model = Arc::new(MockModel::new(vec![
                vec![
                    AgentEvent::ToolCall {
                        call: ToolCall {
                            id: "action".into(),
                            name: name.into(),
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
            ]));
            let agent = Agent::with_security(model, registry, None, None, root.path().into())
                .with_permission_pipeline(
                    state.clone(),
                    talos_permission::PermissionContext::new(
                        talos_permission::PermissionMode::Interactive,
                        talos_permission::InteractionCapability::Available,
                    ),
                    Some(Arc::new(resolver)),
                );
            let (tx, mut rx) = mpsc::unbounded_channel();
            agent
                .run_streaming("perform this session scheduling action".into(), vec![], tx)
                .await
                .expect("run");
            let mut result_count = 0;
            while let Ok(event) = rx.try_recv() {
                if let AgentEvent::ToolResult { result, .. } = event {
                    result_count += 1;
                    assert_eq!(
                        result.is_error,
                        scenario != "allow",
                        "{name}/{scenario}: {} (assessments={})",
                        result.content,
                        calls.load(Ordering::SeqCst)
                    );
                }
            }
            assert_eq!(result_count, 1);
            let result = list.execute(serde_json::json!({})).await;
            let has_task = !result.content.contains("No active scheduled tasks.");
            assert_eq!(
                has_task,
                if name == "cancel_scheduled_task" {
                    scenario != "allow"
                } else {
                    scenario == "allow"
                },
                "{name}/{scenario}: {}",
                result.content
            );
            assert_eq!(
                calls.load(Ordering::SeqCst),
                usize::from(!matches!(
                    scenario,
                    "deny" | "human" | "external" | "oversized"
                )),
                "{name}/{scenario}"
            );
            assert_eq!(state.grant_count().expect("grants"), 0);
            cancel.cancel();
            actor.await.expect("actor");
        }
    }
}
