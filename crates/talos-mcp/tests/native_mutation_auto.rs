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
            "update selected.txt with a retained backup"
        );
        self.verified.fetch_add(1, Ordering::SeqCst);
        if self.scenario == "changed" {
            std::fs::write(&request.mutation.path, "concurrent update").expect("concurrent writer");
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

#[tokio::test]
async fn retained_edit_and_delete_require_trusted_complete_image_auto_review() {
    for delete in [false, true] {
        for scenario in [
            "allow",
            "deny",
            "replacement",
            "changed",
            "policy_write_deny",
            "headless",
        ] {
            let root = tempfile::tempdir().expect("workspace");
            let path = root.path().join("selected.txt");
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
                }),
                Arc::new(Deny),
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
                .with_user_intent("update selected.txt with a retained backup".into());
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
                json!({"path":"selected.txt", "retain_backup":true})
            } else {
                json!({"path":"selected.txt", "old_string":"hi", "new_string":"hello", "retain_backup":true})
            };
            let result = client
                .peer()
                .call_tool(
                    rmcp::model::CallToolRequestParams::new(if delete { "delete" } else { "edit" })
                        .with_arguments(input.as_object().expect("args").clone()),
                )
                .await;
            let expected_reviews =
                usize::from(!matches!(scenario, "replacement" | "policy_write_deny"));
            assert_eq!(calls.load(Ordering::SeqCst), expected_reviews, "{scenario}");
            assert_eq!(
                verified.load(Ordering::SeqCst),
                expected_reviews,
                "assessment evidence assertions must complete: {scenario}"
            );
            if matches!(scenario, "allow" | "headless") {
                let result = result.expect("Auto admission");
                assert_ne!(result.is_error, Some(true), "{result:?}");
                if delete {
                    assert!(!path.exists());
                } else {
                    assert_eq!(std::fs::read_to_string(&path).expect("edited"), "hello");
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
