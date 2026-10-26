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
        std::fs::write(
            root.path().join("fixture.rs"),
            "use std::path::PathBuf;\npub fn auto_fixture() -> PathBuf { PathBuf::new() }\nfn caller() { let _ = auto_fixture(); }\n",
        ).expect("source fixture");
        // Git is used only to build an isolated fixture; product reads use the actual tools.
        for args in [
            vec!["init", "--template=", "--initial-branch=auto-proof"],
            vec!["add", "evidence.txt"],
            vec![
                "-c",
                "user.name=Auto fixture",
                "-c",
                "user.email=auto@example.invalid",
                "-c",
                "commit.gpgSign=false",
                "-c",
                "maintenance.auto=false",
                "commit",
                "-m",
                "auto-read-commit-evidence",
            ],
        ] {
            let output = std::process::Command::new("git")
                .env_clear()
                .env("PATH", std::env::var_os("PATH").expect("PATH"))
                .env("HOME", root.path())
                .env("TEMP", root.path())
                .env("TMP", root.path())
                .env(
                    "SystemRoot",
                    std::env::var_os("SystemRoot").unwrap_or_default(),
                )
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .current_dir(root.path())
                .args(args)
                .output()
                .expect("fixture Git");
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::fs::write(
            root.path().join("evidence.txt"),
            "read-fast-path-evidence\nmodified",
        )
        .expect("modified tracked fixture");
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
            (
                Arc::new(talos_tools::git::GitStatusTool::new(root.path().into())),
                serde_json::json!({}),
            ),
            (
                Arc::new(talos_tools::git::GitLogTool::new(root.path().into())),
                serde_json::json!({}),
            ),
            (
                Arc::new(talos_tools::git::GitBranchListTool::new(root.path().into())),
                serde_json::json!({}),
            ),
            (
                Arc::new(talos_tools::git::GitDiffTool::new(root.path().into())),
                serde_json::json!({}),
            ),
            (
                Arc::new(talos_tools::git::GitShowTool::new(root.path().into())),
                serde_json::json!({"revision":"HEAD"}),
            ),
            (
                Arc::new(talos_tools::document_extract::DocumentExtractTool::new(
                    root.path().into(),
                )),
                serde_json::json!({"path":"evidence.txt"}),
            ),
            (
                Arc::new(talos_tools::symbol::FindSymbolTool::new(root.path().into())),
                serde_json::json!({"name":"auto_fixture","path":"."}),
            ),
            (
                Arc::new(talos_tools::symbol::FindReferencesTool::new(
                    root.path().into(),
                )),
                serde_json::json!({"name":"auto_fixture","file":"fixture.rs"}),
            ),
            (
                Arc::new(talos_tools::symbol::ListSymbolsTool::new(
                    root.path().into(),
                )),
                serde_json::json!({"path":"fixture.rs"}),
            ),
            (
                Arc::new(talos_tools::symbol::ListImportsTool::new(
                    root.path().into(),
                )),
                serde_json::json!({"file":"fixture.rs"}),
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
                            result.content.contains(match name.as_str() {
                                "read" | "document_extract" => "read-fast-path-evidence",
                                "find_symbol" | "list_symbols" => "auto_fixture",
                                "find_references" => "fixture.rs",
                                "list_imports" => "PathBuf",
                                "git_log" | "git_show" => "auto-read-commit-evidence",
                                "git_branch_list" => "auto-proof",
                                _ => "evidence.txt",
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
