//! Recovery receipts confer no authority over a later Agent invocation.
use super::*;
use talos_core::tool::{ToolContribution, ToolContributionSource};

struct PendingRecoveryApproval(Arc<tokio::sync::Notify>);

#[async_trait]
impl ApprovalResolver for PendingRecoveryApproval {
    async fn resolve(
        &self,
        _: PermissionApprovalRequest,
        _: std::time::Duration,
    ) -> Result<talos_core::ApprovalChoice, ApprovalResolverError> {
        self.0.notify_one();
        std::future::pending().await
    }
}

#[tokio::test]
async fn recovery_dispatch_rechecks_policy_and_preserves_concurrent_changes() {
    for delete in [false, true] {
        for scenario in ["allow", "deny", "ask", "changed", "cancel"] {
            let root = tempfile::tempdir().expect("workspace");
            let path = root.path().join("file.txt");
            std::fs::write(&path, "before").expect("fixture");
            let tool: Arc<dyn AgentTool> = if delete {
                Arc::new(talos_tools::file_tools::DeleteTool::new(root.path().into()))
            } else {
                Arc::new(talos_tools::file_tools::EditTool::new(root.path().into()))
            };
            // Prepare a genuine instance-local receipt; only restoration is under test.
            let mutation = if delete {
                serde_json::json!({"path":"file.txt","retain_backup":true})
            } else {
                serde_json::json!({"path":"file.txt","old_string":"before","new_string":"after","retain_backup":true})
            };
            let result = tool.execute(mutation).await;
            assert!(!result.is_error, "{}", result.content);
            let id = result
                .content
                .split_once("recovery_id: ")
                .map(|(_, rest)| rest.split([';', '\n']).next().expect("id"))
                .expect("receipt")
                .to_owned();
            if scenario == "changed" {
                std::fs::write(&path, "concurrent content").expect("concurrent writer");
            }
            let input = serde_json::json!({"path":"file.txt","recovery_id":id});
            let mut registry = ToolRegistry::new();
            registry
                .register_builtin_contributions([ToolContribution::new(
                    ToolContributionSource::new("talos-tools:file"),
                    tool.clone(),
                )])
                .expect("registration");
            let mut engine = PermissionEngine::empty();
            engine.set_workspace_root(root.path().into());
            engine.add_rule(talos_permission::PermissionRule::new_nature(
                talos_core::tool::ToolNature::Write,
                None,
                None,
                match scenario {
                    "deny" => PermissionDecision::Deny("recovery denied".into()),
                    "ask" | "cancel" => PermissionDecision::Ask,
                    _ => PermissionDecision::Allow,
                },
            ));
            let state = Arc::new(talos_permission::PermissionSessionState::new(engine));
            let responses = vec![
                vec![
                    AgentEvent::ToolCall {
                        call: ToolCall {
                            id: "restore".into(),
                            name: tool.name().into(),
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
            let approval_started = Arc::new(tokio::sync::Notify::new());
            let resolver: Arc<dyn ApprovalResolver> = if scenario == "cancel" {
                Arc::new(PendingRecoveryApproval(approval_started.clone()))
            } else {
                Arc::new(AutoDenyFallback)
            };
            let agent = Agent::with_security(
                Arc::new(MockModel::new(responses)),
                registry,
                None,
                None,
                root.path().into(),
            )
            .with_permission_pipeline(
                state.clone(),
                talos_permission::PermissionContext::new(
                    talos_permission::PermissionMode::Interactive,
                    talos_permission::InteractionCapability::Available,
                ),
                Some(resolver),
            );
            let (tx, mut rx) = mpsc::unbounded_channel();
            if scenario == "cancel" {
                let task = tokio::spawn(async move {
                    agent
                        .run_streaming("restore the retained backup".into(), vec![], tx)
                        .await
                });
                tokio::time::timeout(
                    std::time::Duration::from_secs(2),
                    approval_started.notified(),
                )
                .await
                .expect("restore reaches approval before cancellation");
                task.abort();
                assert!(task.await.expect_err("cancelled caller").is_cancelled());
            } else {
                agent
                    .run_streaming("restore the retained backup".into(), vec![], tx)
                    .await
                    .expect("run");
            }
            let mut results = 0;
            while let Ok(event) = rx.try_recv() {
                if let AgentEvent::ToolResult { result, .. } = event {
                    results += 1;
                    assert_eq!(
                        result.is_error,
                        scenario != "allow",
                        "delete={delete}, {scenario}: {}",
                        result.content
                    );
                }
            }
            assert_eq!(results, usize::from(scenario != "cancel"));
            assert_eq!(state.grant_count().expect("grants"), 0);
            match scenario {
                "allow" => {
                    assert_eq!(std::fs::read_to_string(&path).expect("restored"), "before");
                    assert!(tool.execute(input).await.is_error, "receipt consumed");
                }
                "changed" => {
                    assert_eq!(
                        std::fs::read_to_string(&path).expect("current"),
                        "concurrent content"
                    );
                }
                _ => {
                    if delete {
                        assert!(!path.exists());
                    } else {
                        assert_eq!(std::fs::read_to_string(&path).expect("edited"), "after");
                    }
                    // Denied or cancelled dispatch cannot consume the receipt. An explicitly
                    // invoked tool restore proves it remains usable afterwards.
                    let retry = tool.execute(input).await;
                    assert!(!retry.is_error, "{}", retry.content);
                    assert_eq!(std::fs::read_to_string(&path).expect("retry"), "before");
                }
            }
        }
    }
}
