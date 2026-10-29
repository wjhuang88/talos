//! Real registered Pull execution through model command admission.
use super::*;
use crate::auto_resolver::PreparedCommandAssessmentRequest;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

fn git(root: &Path, args: &[&str]) -> Vec<u8> {
    let output = std::process::Command::new("git")
        .env_clear()
        .env("PATH", std::env::var_os("PATH").expect("PATH"))
        .env("HOME", root)
        .env("TEMP", root)
        .env("TMP", root)
        .env(
            "SystemRoot",
            std::env::var_os("SystemRoot").unwrap_or_default(),
        )
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .current_dir(root)
        .args(args)
        .output()
        .expect("fixture Git");
    assert!(
        output.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

fn commit(root: &Path, message: &str) {
    git(root, &["add", "tracked.txt"]);
    git(
        root,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgSign=false",
            "-c",
            "maintenance.auto=false",
            "commit",
            "-m",
            message,
        ],
    );
}

struct Assessor {
    scenario: &'static str,
    tool: &'static str,
    operation: &'static str,
    intent: &'static str,
    calls: Arc<AtomicUsize>,
    checked: Arc<AtomicUsize>,
    state: Arc<talos_permission::PermissionSessionState>,
}
#[async_trait]
impl AutoPermissionAssessor for Assessor {
    async fn assess(
        &self,
        _: AutoPermissionRequest,
        _: std::time::Duration,
    ) -> Result<String, String> {
        panic!("effectful Pull must not use legacy read-only assessment")
    }
    async fn assess_prepared_command(
        &self,
        request: PreparedCommandAssessmentRequest,
        _: std::time::Duration,
    ) -> Result<String, String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(request.tool, self.tool);
        assert_eq!(request.command.operation, self.operation);
        assert_eq!(request.command.effects.len(), 6);
        assert!(!request.command.confined);
        assert!(request.command.targets.dynamic_effects_unknown);
        assert_eq!(request.user_intent, self.intent);
        self.checked.fetch_add(1, Ordering::SeqCst);
        if self.scenario == "error" {
            return Err("assessor unavailable".into());
        }
        if self.scenario == "stale" {
            self.state.rebind_session().expect("rebind");
        }
        Ok(
            serde_json::json!({"schema":request.schema,"request_digest":request.request_digest,
            "decision":match self.scenario { "human" => "human_required", "model_deny" => "deny", _ => "allow_once" },
            "confidence":"high","acknowledged_effects":request.command.effects,
            "accepts_unconfined_effects":true})
            .to_string(),
        )
    }
}

// Explicitly approve only configuration discovery. A later fallback cannot
// accidentally turn a failed/unsupported model decision into successful Pull.
struct DiscoveryOnly(Arc<AtomicUsize>, bool);
#[async_trait]
impl ApprovalResolver for DiscoveryOnly {
    async fn resolve(
        &self,
        _: PermissionApprovalRequest,
        _: std::time::Duration,
    ) -> Result<talos_core::ApprovalChoice, ApprovalResolverError> {
        Ok(if self.0.fetch_add(1, Ordering::SeqCst) == 0 || self.1 {
            talos_core::ApprovalChoice::ApproveOnce
        } else {
            talos_core::ApprovalChoice::Deny
        })
    }
}

struct DiscoveryThenDenyCommand(Arc<AtomicUsize>);
#[async_trait]
impl ApprovalResolver for DiscoveryThenDenyCommand {
    async fn resolve(
        &self,
        request: PermissionApprovalRequest,
        _: std::time::Duration,
    ) -> Result<talos_core::ApprovalChoice, ApprovalResolverError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(
            if request.preview.facets().iter().any(|facet| {
                matches!(
                    facet.normalized_scope.as_str(),
                    "git:add-command"
                        | "git:commit-command"
                        | "git:checkout-command"
                        | "git:push-command"
                )
            }) {
                talos_core::ApprovalChoice::Deny
            } else {
                talos_core::ApprovalChoice::ApproveOnce
            },
        )
    }

    async fn resolve_with_explanation(
        &self,
        request: PermissionApprovalRequest,
        remaining: std::time::Duration,
        explanation: &str,
    ) -> Result<talos_core::ApprovalChoice, ApprovalResolverError> {
        if explanation.contains("automatic review did not authorize it") {
            Ok(talos_core::ApprovalChoice::Deny)
        } else {
            self.resolve(request, remaining).await
        }
    }
}

#[tokio::test]
async fn real_pull_auto_admission_updates_only_after_complete_model_approval() {
    for scenario in [
        "allow",
        "human",
        "error",
        "stale",
        "deny",
        "mandatory_human",
        "external",
        "replacement",
        "manual_external",
        "manual_replacement",
    ] {
        let root = tempfile::tempdir().expect("fixture");
        let source = root.path().join("source");
        let target = root.path().join("target");
        std::fs::create_dir(&source).expect("source");
        git(&source, &["init", "--template=", "--initial-branch=main"]);
        git(&source, &["config", "core.autocrlf", "false"]);
        std::fs::write(source.join("tracked.txt"), "before\n").expect("before");
        commit(&source, "before");
        git(
            root.path(),
            &[
                "clone",
                "--template=",
                "--config=core.autocrlf=false",
                source.to_str().expect("source path"),
                target.to_str().expect("target path"),
            ],
        );
        std::fs::write(source.join("tracked.txt"), "after\n").expect("after");
        commit(&source, "after");
        let before = git(&target, &["rev-parse", "HEAD"]);
        let after = git(&source, &["rev-parse", "HEAD"]);
        let mut registry = ToolRegistry::new();
        if !matches!(scenario, "external" | "manual_external") {
            registry
                .register_builtin_contributions(talos_tools::git_mutation_tool_contributions(
                    target.clone(),
                ))
                .expect("builtins");
        }
        if matches!(
            scenario,
            "external" | "replacement" | "manual_external" | "manual_replacement"
        ) {
            registry.register(Arc::new(talos_tools::git::GitPullTool::new(target.clone())));
        }
        let mut engine = PermissionEngine::with_workspace_root(target.clone());
        if matches!(scenario, "deny" | "mandatory_human") {
            engine.add_rule(talos_permission::PermissionRule::new(
                "git_pull",
                None,
                if scenario == "deny" {
                    PermissionDecision::Deny("test policy".into())
                } else {
                    PermissionDecision::Ask
                },
            ));
        }
        let state = Arc::new(talos_permission::PermissionSessionState::new(engine));
        let calls = Arc::new(AtomicUsize::new(0));
        let checked = Arc::new(AtomicUsize::new(0));
        let manual = Arc::new(AtomicUsize::new(0));
        let resolver = AutoPermissionResolver::new(
            Arc::new(Assessor {
                scenario,
                tool: "git_pull",
                operation: "git:pull-command/v1",
                intent: "pull the configured upstream into this repository",
                calls: calls.clone(),
                checked: checked.clone(),
                state: state.clone(),
            }),
            Arc::new(DiscoveryOnly(
                manual.clone(),
                scenario.starts_with("manual_"),
            )),
            ManagedWorkspaceLease::for_permission_session(&target, state.clone()).expect("lease"),
            std::time::Duration::from_secs(5),
            AutoPermissionControl::new(true),
        );
        let model = MockModel::new(vec![
            vec![
                AgentEvent::ToolCall {
                    call: ToolCall {
                        id: "pull".into(),
                        name: "git_pull".into(),
                        input: serde_json::json!({}),
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
        let agent = Agent::with_security(Arc::new(model), registry, None, None, target.clone())
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
            .run_streaming(
                "pull the configured upstream into this repository".into(),
                vec![],
                tx,
            )
            .await
            .expect("turn");
        let mut result = None;
        while let Ok(event) = rx.try_recv() {
            if let AgentEvent::ToolResult { result: value, .. } = event {
                if value.tool_use_id == "pull" {
                    result = Some(value);
                }
            }
        }
        let result = result.expect("real Pull result");
        let allowed = matches!(scenario, "allow" | "manual_external" | "manual_replacement");
        assert_eq!(!result.is_error, allowed, "{scenario}: {}", result.content);
        assert_eq!(
            calls.load(Ordering::SeqCst),
            usize::from(matches!(scenario, "allow" | "human" | "error" | "stale")),
            "{scenario}"
        );
        assert_eq!(
            checked.load(Ordering::SeqCst),
            calls.load(Ordering::SeqCst),
            "{scenario}: assessor assertions must complete"
        );
        assert_eq!(
            manual.load(Ordering::SeqCst),
            if scenario == "deny" {
                0
            } else if scenario == "allow" {
                1
            } else {
                2
            },
            "{scenario}"
        );
        assert_eq!(
            git(&target, &["rev-parse", "HEAD"]),
            if allowed { after } else { before },
            "{scenario}"
        );
        assert_eq!(
            std::fs::read_to_string(target.join("tracked.txt")).expect("worktree"),
            if allowed { "after\n" } else { "before\n" },
            "{scenario}"
        );
        assert_eq!(state.grant_count().expect("grants"), 0, "{scenario}");
    }
}

#[tokio::test]
async fn real_registered_git_mutations_execute_only_after_operation_specific_auto_review() {
    for (tool_name, operation, intent) in [
        (
            "git_add",
            "git:add-command/v1",
            "stage this file in this repository",
        ),
        (
            "git_commit",
            "git:commit-command/v1",
            "commit the staged change in this repository",
        ),
        (
            "git_checkout",
            "git:checkout-command/v1",
            "switch this repository to the target branch",
        ),
        (
            "git_push",
            "git:push-command/v1",
            "push this branch to the configured receiver",
        ),
    ] {
        for scenario in [
            "allow",
            "human",
            "model_deny",
            "error",
            "stale",
            "policy_deny",
            "scoped_ask",
            "external",
            "replacement",
        ] {
            let root = tempfile::tempdir().expect("fixture");
            git(
                root.path(),
                &["init", "--template=", "--initial-branch=main"],
            );
            git(root.path(), &["config", "user.name", "Fixture"]);
            git(
                root.path(),
                &["config", "user.email", "fixture@example.invalid"],
            );
            git(root.path(), &["config", "commit.gpgSign", "false"]);
            git(root.path(), &["config", "maintenance.auto", "false"]);
            std::fs::write(root.path().join("tracked.txt"), "initial\n").expect("initial file");
            commit(root.path(), "initial");
            let input = match tool_name {
                "git_add" => {
                    std::fs::write(root.path().join("added.txt"), "staged\n").expect("add file");
                    serde_json::json!({"paths":["added.txt"]})
                }
                "git_commit" => {
                    std::fs::write(root.path().join("committed.txt"), "committed\n")
                        .expect("commit file");
                    git(root.path(), &["add", "committed.txt"]);
                    serde_json::json!({"message":"operation specific commit"})
                }
                "git_checkout" => {
                    git(root.path(), &["branch", "target"]);
                    serde_json::json!({"branch":"target"})
                }
                "git_push" => {
                    let receiver = root.path().join("receiver.git");
                    git(
                        root.path(),
                        &["init", "--bare", receiver.to_str().expect("receiver")],
                    );
                    git(
                        root.path(),
                        &[
                            "remote",
                            "add",
                            "origin",
                            receiver.to_str().expect("receiver"),
                        ],
                    );
                    serde_json::json!({"branch":"main"})
                }
                _ => unreachable!(),
            };
            let mut registry = ToolRegistry::new();
            if scenario != "external" {
                registry
                    .register_builtin_contributions(talos_tools::git_mutation_tool_contributions(
                        root.path().to_owned(),
                    ))
                    .expect("registered Git builtins");
            }
            if matches!(scenario, "external" | "replacement") {
                let tool: Arc<dyn talos_core::tool::AgentTool> = match tool_name {
                    "git_add" => {
                        Arc::new(talos_tools::git::GitAddTool::new(root.path().to_owned()))
                    }
                    "git_commit" => {
                        Arc::new(talos_tools::git::GitCommitTool::new(root.path().to_owned()))
                    }
                    "git_checkout" => Arc::new(talos_tools::git::GitCheckoutTool::new(
                        root.path().to_owned(),
                    )),
                    "git_push" => {
                        Arc::new(talos_tools::git::GitPushTool::new(root.path().to_owned()))
                    }
                    _ => unreachable!(),
                };
                registry.register(tool);
            }
            let mut engine = PermissionEngine::with_workspace_root(root.path().to_owned());
            if matches!(scenario, "policy_deny" | "scoped_ask") {
                engine.add_rule(talos_permission::PermissionRule::new_nature(
                    talos_core::tool::ToolNature::Write,
                    Some("unseen-helper-output/**".into()),
                    None,
                    if scenario == "policy_deny" {
                        PermissionDecision::Deny("test transitive restriction".into())
                    } else {
                        PermissionDecision::Ask
                    },
                ));
            }
            let state = Arc::new(talos_permission::PermissionSessionState::new(engine));
            let calls = Arc::new(AtomicUsize::new(0));
            let checked = Arc::new(AtomicUsize::new(0));
            let manual = Arc::new(AtomicUsize::new(0));
            let resolver = AutoPermissionResolver::new(
                Arc::new(Assessor {
                    scenario,
                    tool: tool_name,
                    operation,
                    intent,
                    calls: calls.clone(),
                    checked: checked.clone(),
                    state: state.clone(),
                }),
                Arc::new(DiscoveryThenDenyCommand(manual.clone())),
                ManagedWorkspaceLease::for_permission_session(root.path(), state.clone())
                    .expect("lease"),
                std::time::Duration::from_secs(5),
                AutoPermissionControl::new(true),
            );
            let model = MockModel::new(vec![
                vec![
                    AgentEvent::ToolCall {
                        call: ToolCall {
                            id: tool_name.into(),
                            name: tool_name.into(),
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
            ]);
            let agent = Agent::with_security(
                Arc::new(model),
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
            let (tx, mut rx) = mpsc::unbounded_channel();
            agent
                .run_streaming(intent.into(), vec![], tx)
                .await
                .expect("turn");
            let mut result = None;
            while let Ok(event) = rx.try_recv() {
                if let AgentEvent::ToolResult { result: value, .. } = event
                    && value.tool_use_id == tool_name
                {
                    result = Some(value);
                }
            }
            let result = result.expect("tool result");
            let allowed = scenario == "allow";
            assert_eq!(
                !result.is_error, allowed,
                "{tool_name}/{scenario}: {}",
                result.content
            );
            assert_eq!(
                calls.load(Ordering::SeqCst),
                usize::from(!matches!(
                    scenario,
                    "policy_deny" | "scoped_ask" | "external" | "replacement"
                )),
                "{tool_name}: model review count"
            );
            assert_eq!(
                checked.load(Ordering::SeqCst),
                usize::from(!matches!(
                    scenario,
                    "policy_deny" | "scoped_ask" | "external" | "replacement"
                )),
                "{tool_name}: evidence checks"
            );
            assert_eq!(
                state.grant_count().expect("grants"),
                0,
                "{tool_name}: no persistent grant"
            );
            match tool_name {
                "git_add" => {
                    assert_eq!(
                        !git(root.path(), &["diff", "--cached", "--name-only"]).is_empty(),
                        allowed
                    )
                }
                "git_commit" => assert_eq!(
                    String::from_utf8(git(root.path(), &["log", "-1", "--format=%s"]))
                        .expect("subject")
                        .trim(),
                    if allowed {
                        "operation specific commit"
                    } else {
                        "initial"
                    }
                ),
                "git_checkout" => assert_eq!(
                    String::from_utf8(git(root.path(), &["branch", "--show-current"]))
                        .expect("branch")
                        .trim(),
                    if allowed { "target" } else { "main" }
                ),
                "git_push" => assert_eq!(
                    root.path().join("receiver.git/refs/heads/main").exists(),
                    allowed
                ),
                _ => unreachable!(),
            }
        }
    }
}
