//! Complete-image Auto review followed by recoverable file replacement.
use super::*;
use crate::auto_resolver::FileMutationAssessmentRequest;
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
        panic!("trusted edit must review complete images")
    }
    async fn assess_file_mutation(
        &self,
        request: FileMutationAssessmentRequest,
        _: std::time::Duration,
    ) -> Result<String, String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(request.input["new_string"], "hello");
        assert_eq!(request.mutation.before, "hi");
        assert_eq!(request.mutation.after.as_deref(), Some("hello"));
        assert_eq!(request.user_intent, "replace hi with hello in existing.txt");
        assert_eq!(request.facets.len(), 2);
        match self.scenario {
            "changed_file" => std::fs::write(&request.mutation.path, "concurrent update")
                .expect("concurrent writer"),
            "error" => return Err("fixture error".into()),
            "cancel" => self.control.set_enabled(false),
            "stale" => {
                self.state.rebind_session().expect("rebind");
            }
            _ => {}
        }
        let mut response = serde_json::json!({"schema":request.schema,"request_digest":request.request_digest,
            "decision":if self.scenario == "uncertain" {"human_required"} else {"allow_once"},"confidence":"high"});
        match self.scenario {
            "low_confidence" => response["confidence"] = "low".into(),
            "wrong_digest" => response["request_digest"] = "sha256:another-invocation".into(),
            "wrong_schema" => response["schema"] = "builtin-invocation/v2".into(),
            "extra_field" => response["persistent_grant"] = true.into(),
            "malformed" => return Ok("allow_once".into()),
            "oversized_response" => return Ok("x".repeat(64 * 1024)),
            _ => {}
        }
        Ok(response.to_string())
    }
}

#[tokio::test]
async fn trusted_edit_reviews_complete_images_before_real_execution() {
    for scenario in [
        "allow",
        "changed_file",
        "uncertain",
        "error",
        "cancel",
        "stale",
        "oversized",
        "low_confidence",
        "wrong_digest",
        "wrong_schema",
        "extra_field",
        "malformed",
        "oversized_response",
    ] {
        let allow = scenario == "allow";
        let root = tempfile::tempdir().expect("workspace");
        std::fs::write(root.path().join("existing.txt"), "hi").expect("original");
        let mut registry = ToolRegistry::new();
        registry
            .register_builtin_contributions([ToolContribution::new(
                ToolContributionSource::new("talos-tools:file"),
                Arc::new(talos_tools::file_tools::EditTool::new(
                    root.path().to_owned(),
                )) as Arc<dyn AgentTool>,
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
                .expect("lease"),
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
                        name: "edit".into(),
                        input: serde_json::json!({"path":"existing.txt","old_string":"hi","new_string":content,"retain_backup":true}),
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
            .run("replace hi with hello in existing.txt".into())
            .await
            .expect("run");
        assert_eq!(
            calls.load(Ordering::SeqCst),
            usize::from(scenario != "oversized"),
            "{scenario}"
        );
        assert_eq!(
            std::fs::read_to_string(root.path().join("existing.txt")).expect("file"),
            if allow {
                "hello"
            } else if scenario == "changed_file" {
                "concurrent update"
            } else {
                "hi"
            },
            "{scenario}"
        );
        assert_eq!(state.grant_count().expect("grants"), 0);
    }
}
