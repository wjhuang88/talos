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
    delete: bool,
    anchored: bool,
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
        if self.anchored {
            assert_eq!(request.input["operations"][0]["content"], "hello");
        } else if !self.delete {
            assert_eq!(request.input["new_string"], "hello");
        }
        assert_eq!(request.mutation.before, "hi");
        assert_eq!(
            request.mutation.after.as_deref(),
            if self.delete { None } else { Some("hello") }
        );
        assert_eq!(
            request.user_intent,
            if self.delete {
                "delete existing.txt with a retained backup"
            } else {
                "replace hi with hello in existing.txt"
            }
        );
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
    run_mutation_scenarios(false, false).await;
}

#[tokio::test]
async fn trusted_delete_reviews_complete_image_before_real_execution() {
    run_mutation_scenarios(true, false).await;
}

#[tokio::test]
async fn trusted_anchored_edit_reviews_complete_images_before_real_execution() {
    run_mutation_scenarios(false, true).await;
}

async fn run_mutation_scenarios(delete: bool, anchored: bool) {
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
        "nul_original",
        "nul_replacement",
    ] {
        if (delete && scenario == "nul_replacement") || (anchored && scenario == "nul_original") {
            continue;
        }
        let allow = scenario == "allow";
        let root = tempfile::tempdir().expect("workspace");
        std::fs::write(
            root.path().join("existing.txt"),
            if scenario == "nul_original" {
                "hi\0payload".into()
            } else if delete && scenario == "oversized" {
                "x".repeat(20 * 1024)
            } else {
                "hi".into()
            },
        )
        .expect("original");
        let mut registry = ToolRegistry::new();
        let mut anchor = None;
        let edit_tool: Arc<dyn AgentTool> = if anchored {
            let (read, _, edit, _) =
                talos_tools::file_tools::snapshot_aware_file_tools(root.path().to_owned());
            let result = read
                .execute(serde_json::json!({"path":"existing.txt"}))
                .await;
            assert!(!result.is_error, "{}", result.content);
            let mut lines = result.content.lines();
            let snapshot = lines
                .next()
                .expect("snapshot")
                .strip_prefix("[snapshot:")
                .and_then(|s| s.strip_suffix(']'))
                .expect("id")
                .to_owned();
            let target = lines
                .next()
                .expect("line")
                .split_once('|')
                .expect("anchor")
                .0
                .to_owned();
            anchor = Some((snapshot, target));
            Arc::new(edit)
        } else {
            Arc::new(talos_tools::file_tools::EditTool::new(
                root.path().to_owned(),
            ))
        };
        registry
            .register_builtin_contributions([ToolContribution::new(
                ToolContributionSource::new("talos-tools:file"),
                if delete {
                    Arc::new(talos_tools::file_tools::DeleteTool::new(
                        root.path().to_owned(),
                    )) as Arc<dyn AgentTool>
                } else {
                    edit_tool
                },
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
                delete,
                anchored,
            }),
            Arc::new(AutoDenyFallback),
            ManagedWorkspaceLease::for_permission_session(root.path(), state.clone())
                .expect("lease"),
            std::time::Duration::from_secs(2),
            control,
        );
        let content = if scenario == "nul_replacement" {
            "hello\0payload".into()
        } else if scenario == "oversized" {
            "x".repeat(20 * 1024)
        } else {
            "hello".into()
        };
        let responses = vec![
            vec![
                AgentEvent::ToolCall {
                    call: ToolCall {
                        id: "create".into(),
                        name: if delete { "delete" } else { "edit" }.into(),
                        input: if delete {
                            serde_json::json!({"path":"existing.txt","retain_backup":true})
                        } else if let Some((snapshot, target)) = &anchor {
                            serde_json::json!({"path":"existing.txt","snapshot_id":snapshot,"retain_backup":true,"operations":[{"op":"replace","target":target,"content":content}]})
                        } else {
                            serde_json::json!({"path":"existing.txt","old_string":"hi","new_string":content,"retain_backup":true})
                        },
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
            .run(
                if delete {
                    "delete existing.txt with a retained backup"
                } else {
                    "replace hi with hello in existing.txt"
                }
                .into(),
            )
            .await
            .expect("run");
        assert_eq!(
            calls.load(Ordering::SeqCst),
            usize::from(!matches!(
                scenario,
                "oversized" | "nul_original" | "nul_replacement"
            )),
            "{scenario}"
        );
        if delete && allow {
            assert!(!root.path().join("existing.txt").exists());
        } else {
            assert_eq!(
                std::fs::read_to_string(root.path().join("existing.txt")).expect("file"),
                if scenario == "nul_original" {
                    "hi\0payload".into()
                } else if delete && scenario == "oversized" {
                    "x".repeat(20 * 1024)
                } else if allow {
                    "hello".into()
                } else if scenario == "changed_file" {
                    "concurrent update".into()
                } else {
                    "hi".into()
                },
                "{scenario}"
            );
        }
        assert_eq!(state.grant_count().expect("grants"), 0);
    }
}
