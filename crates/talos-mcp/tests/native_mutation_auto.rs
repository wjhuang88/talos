//! Real hosted-native retained mutations through the MCP permission fence.

use async_trait::async_trait;
use rmcp::ServiceExt;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use talos_agent::auto_resolver::{
    AutoPermissionAssessor, AutoPermissionControl, AutoPermissionRequest, AutoPermissionResolver,
    FileMutationAssessmentRequest, ManagedWorkspaceLease,
};
use talos_agent::permission_pipeline::{
    ApprovalResolver, ApprovalResolverError, PermissionApprovalRequest, PermissionPipeline,
};
use talos_core::ApprovalChoice;
use talos_core::tool::ToolRegistry;
use talos_mcp::server::{McpPermissionGate, TalosMcpHandler};
use talos_permission::{
    InteractionCapability, PermissionContext, PermissionDecision, PermissionEngine, PermissionMode,
    PermissionRule, PermissionSessionState,
};
use talos_plugin::HookRegistry;

struct Assessor {
    calls: Arc<AtomicUsize>,
    verified: Arc<AtomicUsize>,
    scenario: &'static str,
    delete: bool,
    file_name: &'static str,
}

#[async_trait]
impl AutoPermissionAssessor for Assessor {
    async fn assess(&self, _: AutoPermissionRequest, _: Duration) -> Result<String, String> {
        panic!("retained mutations must use complete-image review")
    }

    async fn assess_file_mutation(
        &self,
        request: FileMutationAssessmentRequest,
        _: Duration,
    ) -> Result<String, String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(request.mutation.before, "hi");
        assert_eq!(
            request.mutation.after.as_deref(),
            if self.delete { None } else { Some("hello") }
        );
        assert_eq!(
            request.user_intent,
            format!("update {} with a retained backup", self.file_name)
        );
        self.verified.fetch_add(1, Ordering::SeqCst);
        if self.scenario == "changed" {
            std::fs::write(&request.mutation.path, "concurrent update").expect("concurrent writer");
        }
        if self.scenario == "error" {
            return Err("injected file mutation assessor failure".into());
        }
        Ok(
            json!({"schema":request.schema,"request_digest":request.request_digest,
            "decision":if self.scenario == "deny" {"human_required"} else {"allow_once"},
            "confidence":"high"})
            .to_string(),
        )
    }
}

struct Deny;
#[async_trait]
impl ApprovalResolver for Deny {
    async fn resolve(
        &self,
        _: PermissionApprovalRequest,
        _: Duration,
    ) -> Result<ApprovalChoice, ApprovalResolverError> {
        Ok(ApprovalChoice::Deny)
    }
}

struct ApproveRecovery;
#[async_trait]
impl ApprovalResolver for ApproveRecovery {
    async fn resolve(
        &self,
        request: PermissionApprovalRequest,
        _: Duration,
    ) -> Result<ApprovalChoice, ApprovalResolverError> {
        assert!(request.arguments.get("recovery_id").is_some());
        Ok(ApprovalChoice::ApproveOnce)
    }
}

#[tokio::test]
async fn retained_edit_and_delete_require_trusted_complete_image_auto_review() {
    for delete in [false, true] {
        for scenario in [
            "allow",
            "restore",
            "restore_changed",
            "deny",
            "error",
            "sensitive_path",
            "credential_path",
            "replacement",
            "changed",
            "policy_write_deny",
            "headless",
        ] {
            let root = tempfile::tempdir().expect("workspace");
            let file_name = match scenario {
                "sensitive_path" => ".env",
                "credential_path" => ".aws/credentials",
                _ => "selected.txt",
            };
            let path = root.path().join(file_name);
            std::fs::create_dir_all(path.parent().expect("parent")).expect("fixture directory");
            std::fs::write(&path, "hi").expect("fixture");
            let calls = Arc::new(AtomicUsize::new(0));
            let verified = Arc::new(AtomicUsize::new(0));
            let mut engine = PermissionEngine::with_workspace_root(root.path().into());
            if scenario == "policy_write_deny" {
                engine.add_rule(PermissionRule::new_nature(
                    talos_core::tool::ToolNature::Write,
                    None,
                    None,
                    PermissionDecision::Deny("explicit write denial".into()),
                ));
            }
            let state = Arc::new(PermissionSessionState::new(engine));
            let resolver = AutoPermissionResolver::new(
                Arc::new(Assessor {
                    calls: calls.clone(),
                    verified: verified.clone(),
                    scenario,
                    delete,
                    file_name,
                }),
                if matches!(scenario, "restore" | "restore_changed") {
                    Arc::new(ApproveRecovery) as Arc<dyn ApprovalResolver>
                } else {
                    Arc::new(Deny)
                },
                ManagedWorkspaceLease::for_permission_session(root.path(), state.clone())
                    .expect("lease"),
                Duration::from_secs(5),
                AutoPermissionControl::new(true),
            );
            let pipeline = PermissionPipeline::new(
                state.clone(),
                PermissionContext::new(
                    if scenario == "headless" {
                        PermissionMode::Headless
                    } else {
                        PermissionMode::Interactive
                    },
                    if scenario == "headless" {
                        InteractionCapability::Unavailable
                    } else {
                        InteractionCapability::Available
                    },
                ),
                Some(Arc::new(resolver)),
            );
            let gate = McpPermissionGate::with_pipeline(pipeline, Arc::new(HookRegistry::new()))
                .with_user_intent(format!("update {file_name} with a retained backup"));
            let mut registry = ToolRegistry::new();
            registry
                .register_builtin_contributions(talos_tools::ordinary_file_tool_contributions(
                    root.path().into(),
                ))
                .expect("native registration");
            if scenario == "replacement" {
                if delete {
                    registry.register(Arc::new(talos_tools::DeleteTool::new(root.path().into())));
                } else {
                    registry.register(Arc::new(talos_tools::EditTool::new(root.path().into())));
                }
            }
            let handler = TalosMcpHandler::new(Arc::new(registry), Arc::new(gate));
            let (client_io, server_io) = tokio::io::duplex(65536);
            let task = tokio::spawn(async move {
                handler
                    .serve(server_io)
                    .await
                    .expect("server")
                    .waiting()
                    .await
                    .expect("shutdown");
            });
            let client = ().serve(client_io).await.expect("client");
            let input = if delete {
                json!({"path":file_name, "retain_backup":true})
            } else {
                json!({"path":file_name, "old_string":"hi", "new_string":"hello", "retain_backup":true})
            };
            let result = client
                .peer()
                .call_tool(
                    rmcp::model::CallToolRequestParams::new(if delete { "delete" } else { "edit" })
                        .with_arguments(input.as_object().expect("args").clone()),
                )
                .await;
            let expected_reviews = usize::from(!matches!(
                scenario,
                "replacement" | "policy_write_deny" | "sensitive_path" | "credential_path"
            ));
            assert_eq!(calls.load(Ordering::SeqCst), expected_reviews, "{scenario}");
            assert_eq!(
                verified.load(Ordering::SeqCst),
                expected_reviews,
                "assessment evidence assertions must complete: {scenario}"
            );
            if matches!(
                scenario,
                "allow" | "restore" | "restore_changed" | "headless"
            ) {
                let result = result.expect("Auto admission");
                assert_ne!(result.is_error, Some(true), "{result:?}");
                if delete {
                    assert!(!path.exists());
                } else {
                    assert_eq!(std::fs::read_to_string(&path).expect("edited"), "hello");
                }
                if matches!(scenario, "restore" | "restore_changed") {
                    let text = result
                        .content
                        .iter()
                        .filter_map(|part| part.as_text())
                        .map(|part| part.text.as_str())
                        .collect::<Vec<_>>()
                        .join("\n");
                    let id = text
                        .split("recovery_id: ")
                        .nth(1)
                        .expect("receipt")
                        .split([';', '\n'])
                        .next()
                        .expect("recovery id");
                    if scenario == "restore_changed" {
                        std::fs::write(&path, "new user content").expect("concurrent replacement");
                    }
                    let input = json!({"path":"selected.txt", "recovery_id":id});
                    for replay in [false, true] {
                        let restored = client
                            .peer()
                            .call_tool(
                                rmcp::model::CallToolRequestParams::new(if delete {
                                    "delete"
                                } else {
                                    "edit"
                                })
                                .with_arguments(input.as_object().expect("args").clone()),
                            )
                            .await
                            .expect("recovery transport");
                        assert_eq!(
                            restored.is_error == Some(true),
                            replay || scenario == "restore_changed",
                            "{restored:?}"
                        );
                        assert_eq!(
                            std::fs::read_to_string(&path).expect("restored or preserved"),
                            if scenario == "restore_changed" {
                                "new user content"
                            } else {
                                "hi"
                            }
                        );
                        assert_eq!(
                            calls.load(Ordering::SeqCst),
                            1,
                            "restore must not reuse Auto"
                        );
                    }
                }
            } else {
                assert!(
                    result
                        .as_ref()
                        .map_or(true, |result| result.is_error == Some(true)),
                    "{scenario}: {result:?}"
                );
                assert_eq!(
                    std::fs::read_to_string(&path).expect("preserved"),
                    if scenario == "changed" {
                        "concurrent update"
                    } else {
                        "hi"
                    }
                );
            }
            assert_eq!(state.grant_count().expect("grant count"), 0, "{scenario}");
            let _ = client.cancel().await;
            task.await.expect("joined");
        }
    }
}
