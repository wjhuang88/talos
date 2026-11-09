//! Exact-input Auto review followed by real capability-bound file creation.
use super::*;
use crate::auto_resolver::BuiltinInvocationAssessmentRequest;
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
        panic!("trusted creation must review exact input")
    }
    async fn assess_builtin_invocation(
        &self,
        request: BuiltinInvocationAssessmentRequest,
        _: std::time::Duration,
    ) -> Result<String, String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(
            request.input,
            serde_json::json!({"path":"new.txt","content":"hello"})
        );
        assert_eq!(request.user_intent, "create new.txt containing hello");
        assert_eq!(request.facets.len(), 1);
        match self.scenario {
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
async fn trusted_creation_reviews_content_and_intent_before_real_execution() {
    for scenario in [
        "allow",
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
        let capability = Arc::new(
            talos_tools::file_tools::CapStdAtomicCreateCapability::open(root.path())
                .expect("capability"),
        );
        let mut registry = ToolRegistry::new();
        registry
            .register_builtin_contributions([ToolContribution::new(
                ToolContributionSource::new("talos-tools:file"),
                Arc::new(
                    talos_tools::file_tools::WriteTool::new(root.path().to_owned())
                        .with_atomic_create_capability(capability.clone()),
                ) as Arc<dyn AgentTool>,
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
                .expect("lease")
                .with_atomic_create_capability(capability),
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
                        name: "write".into(),
                        input: serde_json::json!({"path":"new.txt","content":content}),
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
            .run("create new.txt containing hello".into())
            .await
            .expect("run");
        assert_eq!(
            calls.load(Ordering::SeqCst),
            usize::from(scenario != "oversized"),
            "{scenario}"
        );
        assert_eq!(root.path().join("new.txt").exists(), allow);
        if allow {
            assert_eq!(
                std::fs::read(root.path().join("new.txt")).expect("file"),
                b"hello"
            );
        }
        assert_eq!(state.grant_count().expect("grants"), 0);
    }
}

struct ExternalApproval {
    calls: Arc<AtomicUsize>,
    approve: bool,
}
#[async_trait]
impl crate::permission_pipeline::ApprovalResolver for ExternalApproval {
    async fn resolve(
        &self,
        _: crate::permission_pipeline::PermissionApprovalRequest,
        _: std::time::Duration,
    ) -> Result<talos_core::ApprovalChoice, crate::permission_pipeline::ApprovalResolverError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(if self.approve {
            talos_core::ApprovalChoice::ApproveOnce
        } else {
            talos_core::ApprovalChoice::Deny
        })
    }
}

#[tokio::test]
async fn workspace_defaults_and_external_auto_fallback_reach_real_write() {
    for (external, approve) in [(false, false), (true, true), (true, false)] {
        let root = tempfile::tempdir().expect("workspace");
        let outside = tempfile::tempdir().expect("outside");
        let target = if external {
            outside.path()
        } else {
            root.path()
        }
        .join("nested/new.txt");
        let capability = Arc::new(
            talos_tools::file_tools::CapStdAtomicCreateCapability::open(root.path())
                .expect("capability"),
        );
        let mut registry = ToolRegistry::new();
        registry
            .register_builtin_contributions([ToolContribution::new(
                ToolContributionSource::new("talos-tools:file"),
                Arc::new(
                    talos_tools::WriteTool::new(root.path().to_owned())
                        .with_atomic_create_capability(capability.clone()),
                ) as Arc<dyn AgentTool>,
            )])
            .expect("register");
        let mut engine = PermissionEngine::with_workspace_root(root.path().to_owned());
        engine.set_ordinary_workspace_files(true);
        let state = Arc::new(talos_permission::PermissionSessionState::new(engine));
        let calls = Arc::new(AtomicUsize::new(0));
        let assessor_calls = Arc::new(AtomicUsize::new(0));
        let control = AutoPermissionControl::new(true);
        let resolver = AutoPermissionResolver::new(
            Arc::new(Assessor {
                calls: assessor_calls.clone(),
                scenario: "allow",
                state: state.clone(),
                control: control.clone(),
            }),
            Arc::new(ExternalApproval {
                calls: calls.clone(),
                approve,
            }),
            ManagedWorkspaceLease::for_permission_session(root.path(), state.clone())
                .expect("lease")
                .with_atomic_create_capability(capability),
            std::time::Duration::from_secs(2),
            control,
        );
        let model = MockModel::new(vec![
            vec![
                AgentEvent::ToolCall {
                    call: ToolCall {
                        id: "external".into(),
                        name: "write".into(),
                        input: serde_json::json!({"path":target,"content":"approved"}),
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
        ]);
        let agent = Agent::with_security(
            Arc::new(model),
            registry,
            None,
            None,
            root.path().to_owned(),
        )
        .with_permission_pipeline(
            state,
            talos_permission::PermissionContext::new(
                talos_permission::PermissionMode::Interactive,
                talos_permission::InteractionCapability::Available,
            ),
            Some(Arc::new(resolver)),
        );
        agent
            .run("create the requested file".into())
            .await
            .expect("run");
        assert_eq!(calls.load(Ordering::SeqCst), usize::from(external));
        assert_eq!(assessor_calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            target.exists(),
            !external || approve,
            "external={external}, approve={approve}"
        );
    }
}

#[tokio::test]
async fn approved_shell_worktree_creation_admits_next_native_write() {
    let temp = tempfile::tempdir().expect("temp");
    let root = temp.path().join("repo");
    std::fs::create_dir(&root).expect("root");
    for args in [
        vec!["init"],
        vec![
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "commit",
            "--allow-empty",
            "-m",
            "initial",
        ],
    ] {
        let output = std::process::Command::new("git")
            .current_dir(&root)
            .args(args)
            .output()
            .expect("git");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let mut engine = PermissionEngine::with_workspace_root(root.clone());
    engine.set_ordinary_workspace_files(true);
    let state = Arc::new(talos_permission::PermissionSessionState::new(engine));
    let host = talos_tools::session_worktrees::SessionWorktrees::open(
        &root,
        "agent-session",
        &temp.path().join("receipts"),
        state.clone(),
    )
    .expect("host");
    let shell = talos_tools::BashTool::new(root.clone()).with_session_worktrees(host);
    let shell_name = shell.name().to_owned();
    let mut registry = ToolRegistry::new();
    registry
        .register_builtin_contributions([
            ToolContribution::new(
                ToolContributionSource::new("talos-tools:shell"),
                Arc::new(shell) as Arc<dyn AgentTool>,
            ),
            ToolContribution::new(
                ToolContributionSource::new("talos-tools:file"),
                Arc::new(talos_tools::WriteTool::new(root.clone())) as Arc<dyn AgentTool>,
            ),
        ])
        .expect("registry");
    let target = temp.path().join("linked/result.txt");
    let turn = |id: &str, name: String, input| {
        vec![
            AgentEvent::ToolCall {
                call: ToolCall {
                    id: id.into(),
                    name,
                    input,
                },
                provenance: Default::default(),
                summary_fields: vec![],
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::ToolUse,
                usage: Usage::default(),
            },
        ]
    };
    let model = MockModel::new(vec![
        turn(
            "create",
            shell_name,
            serde_json::json!({"command":"git worktree add -b linked ../linked"}),
        ),
        turn(
            "write",
            "write".into(),
            serde_json::json!({"path":target,"content":"registered"}),
        ),
        text_done_events("done"),
    ]);
    let approvals = Arc::new(AtomicUsize::new(0));
    let agent = Agent::with_security(Arc::new(model), registry, None, None, root)
        .with_permission_pipeline(
            state,
            talos_permission::PermissionContext::new(
                talos_permission::PermissionMode::Interactive,
                talos_permission::InteractionCapability::Available,
            ),
            Some(Arc::new(ExternalApproval {
                calls: approvals.clone(),
                approve: true,
            })),
        );
    agent
        .run("create a worktree and write result.txt".into())
        .await
        .expect("run");
    assert_eq!(
        std::fs::read_to_string(&target).expect("created file"),
        "registered"
    );
    assert_eq!(
        approvals.load(Ordering::SeqCst),
        1,
        "only the creating invocation requires approval"
    );
}
