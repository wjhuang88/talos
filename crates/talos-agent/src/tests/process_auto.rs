//! Production process cancellation admission, with a supervised pending job.
use super::*;
use crate::auto_resolver::BuiltinActionAssessmentRequest;
use std::sync::atomic::{AtomicUsize, Ordering};
use talos_core::background_job::{BackgroundJobHost, BackgroundJobRequest};

struct Assessor {
    scenario: &'static str,
    calls: Arc<AtomicUsize>,
    verified: Arc<AtomicUsize>,
    state: Arc<talos_permission::PermissionSessionState>,
}
#[async_trait]
impl AutoPermissionAssessor for Assessor {
    async fn assess(
        &self,
        _: AutoPermissionRequest,
        _: std::time::Duration,
    ) -> Result<String, String> {
        Err("wrong assessor route".into())
    }
    async fn assess_builtin_action(
        &self,
        request: BuiltinActionAssessmentRequest,
        _: std::time::Duration,
    ) -> Result<String, String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(request.tool, "process");
        assert!(crate::process_tool::auto_cancel_profile(
            &request.input,
            &request.facets
        ));
        assert!(
            request
                .user_intent
                .contains(request.input["job_id"].as_str().expect("id"))
        );
        self.verified.fetch_add(1, Ordering::SeqCst);
        match self.scenario {
            "error" => return Err("unavailable".into()),
            "stale" => {
                self.state.rebind_session().expect("rebind");
            }
            _ => {}
        }
        Ok(
            serde_json::json!({"schema":request.schema,"request_digest":request.request_digest,
            "decision":if self.scenario == "uncertain" {"human_required"} else {"allow_once"},
            "confidence":"high"})
            .to_string(),
        )
    }
}

#[tokio::test]
async fn process_auto_cancellation_uses_owned_instance_and_exact_job() {
    for scenario in [
        "allow",
        "uncertain",
        "error",
        "stale",
        "deny",
        "foreign",
        "replacement",
    ] {
        let root = tempfile::tempdir().expect("workspace");
        let (events, _) = mpsc::unbounded_channel();
        let supervisor =
            crate::background_jobs::BackgroundJobSupervisor::new(events, "fixture".into(), 0);
        let permit = supervisor
            .reserve(BackgroundJobRequest {
                tool_name: "exec".into(),
                timeout: std::time::Duration::from_secs(30),
            })
            .await
            .expect("reserve");
        let owned = supervisor.job_ids().remove(0);
        #[cfg(unix)]
        let permit = {
            let launched = talos_tools::ExecTool::new(root.path().into())
                .execute_background_authorized_with_output(
                    serde_json::json!({"command":"/bin/sleep", "args":["30"], "background":true, "timeout_secs":30}),
                    permit,
                    &[],
                ).await;
            assert!(!launched.result.is_error, "{}", launched.result.content);
            None::<Box<dyn talos_core::background_job::BackgroundJobPermit>>
        };
        #[cfg(not(unix))]
        let permit = Some(permit);
        let id = if scenario == "foreign" {
            format!("job_{}", uuid::Uuid::new_v4())
        } else {
            owned.clone()
        };
        let mut engine = PermissionEngine::with_workspace_root(root.path().into());
        if scenario == "deny" {
            engine = PermissionEngine::empty();
            engine.set_workspace_root(root.path().into());
            engine.add_rule(talos_permission::PermissionRule::new_nature(
                ToolNature::Execute,
                None,
                None,
                PermissionDecision::Deny("explicit deny".into()),
            ));
        }
        let state = Arc::new(talos_permission::PermissionSessionState::new(engine));
        let calls = Arc::new(AtomicUsize::new(0));
        let verified = Arc::new(AtomicUsize::new(0));
        let resolver = AutoPermissionResolver::new(
            Arc::new(Assessor {
                scenario,
                calls: calls.clone(),
                verified: verified.clone(),
                state: state.clone(),
            }),
            Arc::new(AutoDenyFallback),
            ManagedWorkspaceLease::for_permission_session(root.path(), state.clone())
                .expect("lease"),
            std::time::Duration::from_secs(1),
            AutoPermissionControl::new(true),
        );
        let model = MockModel::new(vec![
            vec![
                AgentEvent::ToolCall {
                    call: ToolCall {
                        id: "cancel".into(),
                        name: "process".into(),
                        input: serde_json::json!({"action":"cancel","job_id":id,"wait_ms":5000}),
                    },
                    provenance: Default::default(),
                    summary_fields: vec![],
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::ToolUse,
                    usage: Default::default(),
                },
            ],
            text_done_events("done"),
        ]);
        let mut agent = Agent::with_security(
            Arc::new(model),
            ToolRegistry::new(),
            None,
            None,
            root.path().into(),
        )
        .with_permission_pipeline(
            state,
            talos_permission::PermissionContext::new(
                talos_permission::PermissionMode::Interactive,
                talos_permission::InteractionCapability::Available,
            ),
            Some(Arc::new(resolver)),
        );
        agent.register_process_tool(supervisor.clone());
        if scenario == "replacement" {
            agent
                .tools
                .register(Arc::new(crate::process_tool::ProcessTool::new(
                    supervisor.clone(),
                )));
        }
        let (tx, mut rx) = mpsc::unbounded_channel();
        agent
            .run_streaming(
                format!("Stop exactly background job {id}; I accept interruption of its effects."),
                vec![],
                tx,
            )
            .await
            .expect("run");
        let mut results = 0;
        while let Ok(event) = rx.try_recv() {
            if let AgentEvent::ToolResult { result, .. } = event {
                results += 1;
                assert_eq!(
                    result.is_error,
                    scenario != "allow",
                    "{scenario}: {}",
                    result.content
                );
                if scenario == "allow" {
                    #[cfg(not(unix))]
                    assert!(result.content.contains("cancellation_requested"));
                    #[cfg(unix)]
                    {
                        let status: Value =
                            serde_json::from_str(&result.content).expect("cancel status");
                        assert_eq!(status["state"], "cancelled");
                        assert_eq!(status["cleanup_outcome"], "terminated");
                    }
                }
            }
        }
        assert_eq!(results, 1);
        let expected = usize::from(!matches!(scenario, "deny" | "foreign" | "replacement"));
        assert_eq!(calls.load(Ordering::SeqCst), expected, "{scenario}");
        assert_eq!(verified.load(Ordering::SeqCst), expected, "{scenario}");
        #[cfg(unix)]
        {
            let status = crate::process_tool::ProcessTool::new(supervisor.clone())
                .execute(serde_json::json!({"action":"status", "job_id":owned}))
                .await;
            assert!(!status.is_error, "{}", status.content);
            let status: Value = serde_json::from_str(&status.content).expect("job status");
            assert_eq!(
                status["state"],
                if scenario == "allow" {
                    "cancelled"
                } else {
                    "running"
                },
                "{scenario}"
            );
        }
        drop(permit);
        supervisor.finalize().await.expect("finalize");
    }
}
