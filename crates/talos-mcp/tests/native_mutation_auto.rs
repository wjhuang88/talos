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
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

struct Assessor {
    calls: Arc<AtomicUsize>,
    verified: Arc<AtomicUsize>,
    scenario: &'static str,
    delete: bool,
    file_name: &'static str,
    dropped: Arc<tokio::sync::Notify>,
}

struct AssessmentDrop(Arc<tokio::sync::Notify>);

impl Drop for AssessmentDrop {
    fn drop(&mut self) {
        self.0.notify_one();
    }
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
        if self.scenario == "cancel" {
            let _guard = AssessmentDrop(self.dropped.clone());
            std::future::pending::<()>().await;
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
            "restore_denied",
            "restore_changed",
            "deny",
            "error",
            "cancel",
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
            let dropped = Arc::new(tokio::sync::Notify::new());
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
                    dropped: dropped.clone(),
                }),
                if matches!(scenario, "restore" | "restore_denied" | "restore_changed") {
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
            let input = if delete {
                json!({"path":file_name, "retain_backup":true})
            } else {
                json!({"path":file_name, "old_string":"hi", "new_string":"hello", "retain_backup":true})
            };
            if scenario == "cancel" {
                let (read, mut write) = tokio::io::split(client_io);
                let mut read = BufReader::new(read);
                let initialize = json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"mutation-cancel-test","version":"1"}}});
                write
                    .write_all(format!("{initialize}\n").as_bytes())
                    .await
                    .expect("initialize");
                let mut line = String::new();
                read.read_line(&mut line)
                    .await
                    .expect("initialized response");
                assert!(
                    serde_json::from_str::<serde_json::Value>(&line)
                        .expect("JSON")
                        .get("result")
                        .is_some()
                );
                write
                    .write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n")
                    .await
                    .expect("initialized");
                let call = json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":if delete {"delete"} else {"edit"},"arguments":input}});
                write
                    .write_all(format!("{call}\n").as_bytes())
                    .await
                    .expect("call");
                tokio::time::timeout(Duration::from_secs(5), async {
                    while calls.load(Ordering::SeqCst) == 0 {
                        tokio::task::yield_now().await;
                    }
                })
                .await
                .expect("assessment started");
                let cancelled = json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":1}});
                write
                    .write_all(format!("{cancelled}\n").as_bytes())
                    .await
                    .expect("cancel");
                tokio::time::timeout(Duration::from_secs(1), dropped.notified())
                    .await
                    .expect("cancel must drop assessment before its deadline or transport close");
                let ping = json!({"jsonrpc":"2.0","id":2,"method":"ping"});
                write
                    .write_all(format!("{ping}\n").as_bytes())
                    .await
                    .expect("ping");
                line.clear();
                read.read_line(&mut line).await.expect("ping response");
                assert_eq!(
                    serde_json::from_str::<serde_json::Value>(&line).expect("JSON")["id"],
                    2
                );
                write
                    .write_all(format!("{call}\n").as_bytes())
                    .await
                    .expect("replay");
                line.clear();
                read.read_line(&mut line).await.expect("replay response");
                let replay: serde_json::Value = serde_json::from_str(&line).expect("JSON");
                assert!(replay.get("error").is_some(), "{replay}");
                assert_eq!(calls.load(Ordering::SeqCst), 1);
                assert_eq!(state.grant_count().expect("grant count"), 0);
                assert_eq!(std::fs::read_to_string(&path).expect("preserved"), "hi");
                drop(read);
                drop(write);
                task.await.expect("joined");
                continue;
            }
            let client = ().serve(client_io).await.expect("client");
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
                "allow" | "restore" | "restore_denied" | "restore_changed" | "headless"
            ) {
                let result = result.expect("Auto admission");
                assert_ne!(result.is_error, Some(true), "{result:?}");
                if delete {
                    assert!(!path.exists());
                } else {
                    assert_eq!(std::fs::read_to_string(&path).expect("edited"), "hello");
                }
                if matches!(scenario, "restore" | "restore_denied" | "restore_changed") {
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
                    if scenario == "restore_denied" {
                        let mut denied_engine =
                            PermissionEngine::with_workspace_root(root.path().into());
                        denied_engine.add_rule(PermissionRule::new_nature(
                            talos_core::tool::ToolNature::Write,
                            None,
                            None,
                            PermissionDecision::Deny("recovery denied by policy".into()),
                        ));
                        state
                            .replace_policy(denied_engine)
                            .expect("install recovery denial");
                        let denied = client
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
                            .expect_err("denied recovery must reject at MCP admission");
                        assert!(denied.to_string().contains("permission denied"), "{denied}");
                        assert_eq!(calls.load(Ordering::SeqCst), 1);
                        if delete {
                            assert!(!path.exists(), "denied restore must preserve deletion");
                        } else {
                            assert_eq!(std::fs::read_to_string(&path).expect("edited"), "hello");
                        }
                        assert_eq!(state.grant_count().expect("grant count"), 0);
                        continue;
                    }
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
