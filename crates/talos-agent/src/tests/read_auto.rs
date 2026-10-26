//! Real read-only dispatch must retain policy fast paths without model review.
use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use talos_core::tool::{ToolContribution, ToolContributionSource};

struct UnexpectedAssessor(Arc<AtomicUsize>);

#[async_trait]
impl AutoPermissionAssessor for UnexpectedAssessor {
    async fn assess(
        &self,
        _: AutoPermissionRequest,
        _: std::time::Duration,
    ) -> Result<String, String> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Err("read policy must resolve without model review".into())
    }
}

#[tokio::test]
async fn real_read_tools_preserve_policy_allow_and_deny_without_assessment() {
    for denied in [false, true] {
        let root = tempfile::tempdir().expect("workspace");
        std::fs::write(root.path().join("evidence.txt"), "read-fast-path-evidence")
            .expect("fixture");
        std::fs::write(root.path().join("changed.txt"), "changed-evidence").expect("fixture");
        let tools: Vec<(Arc<dyn AgentTool>, Value)> = vec![
            (
                Arc::new(talos_tools::file_tools::ReadTool::new(root.path().into())),
                serde_json::json!({"path":"evidence.txt"}),
            ),
            (
                Arc::new(talos_tools::file_tools::LsTool::new(root.path().into())),
                serde_json::json!({"path":"."}),
            ),
            (
                Arc::new(talos_tools::diff_stat::StatTool::new(root.path().into())),
                serde_json::json!({"path":"evidence.txt"}),
            ),
            (
                Arc::new(talos_tools::tree::TreeTool::new(root.path().into())),
                serde_json::json!({"path":"."}),
            ),
            (
                Arc::new(talos_tools::search_tools::GrepTool::new(root.path().into())),
                serde_json::json!({"path":".","pattern":"read-fast-path-evidence"}),
            ),
            (
                Arc::new(talos_tools::search_tools::GlobTool::new(root.path().into())),
                serde_json::json!({"pattern":"*.txt"}),
            ),
            (
                Arc::new(talos_tools::diff_stat::DiffTool::new(root.path().into())),
                serde_json::json!({"old_path":"evidence.txt","new_path":"changed.txt"}),
            ),
        ];
        for (tool, input) in tools {
            let name = tool.name().to_owned();
            let mut registry = ToolRegistry::new();
            registry
                .register_builtin_contributions([ToolContribution::new(
                    ToolContributionSource::new("talos-tools:read-fixture"),
                    tool,
                )])
                .expect("registration");
            let mut engine = PermissionEngine::with_workspace_root(root.path().into());
            if denied {
                engine = PermissionEngine::empty();
                engine.set_workspace_root(root.path().into());
                engine.add_rule(talos_permission::PermissionRule::new_nature(
                    ToolNature::Read,
                    None,
                    None,
                    PermissionDecision::Deny("explicit read denial".into()),
                ));
            }
            let state = Arc::new(talos_permission::PermissionSessionState::new(engine));
            let assessments = Arc::new(AtomicUsize::new(0));
            let resolver = AutoPermissionResolver::new(
                Arc::new(UnexpectedAssessor(assessments.clone())),
                Arc::new(AutoDenyFallback),
                ManagedWorkspaceLease::for_permission_session(root.path(), state.clone())
                    .expect("lease"),
                std::time::Duration::from_secs(2),
                AutoPermissionControl::new(true),
            );
            let responses = vec![
                vec![
                    AgentEvent::ToolCall {
                        call: ToolCall {
                            id: "read-proof".into(),
                            name: name.clone(),
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
            ];
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
                Some(Arc::new(resolver)),
            );
            let (tx, mut rx) = mpsc::unbounded_channel();
            agent
                .run_streaming("inspect evidence.txt".into(), vec![], tx)
                .await
                .expect("run");
            let mut results = 0;
            while let Ok(event) = rx.try_recv() {
                if let AgentEvent::ToolResult { result, .. } = event {
                    results += 1;
                    assert_eq!(result.is_error, denied, "{name}: {}", result.content);
                    if !denied {
                        assert!(
                            result.content.contains(if name == "read" {
                                "read-fast-path-evidence"
                            } else {
                                "evidence.txt"
                            }),
                            "{name}: {}",
                            result.content
                        );
                    }
                }
            }
            assert_eq!(results, 1, "{name}");
            assert_eq!(assessments.load(Ordering::SeqCst), 0, "{name}");
            assert_eq!(state.grant_count().expect("grants"), 0, "{name}");
        }
    }
}
