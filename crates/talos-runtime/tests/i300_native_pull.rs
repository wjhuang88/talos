//! Real Runtime -> Agent -> provider Auto -> supervised Git, using local repositories.
#![cfg(feature = "shared-composition")]

use async_trait::async_trait;
use serde_json::{Value, json};
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use talos_core::{message::ToolCall, provider::DecisionRequestLimits};
use talos_runtime::{
    AgentEvent, ApprovalChoice, ApprovalHandler, GrantPreview, LanguageModel, Message,
    PermissionDecision, PermissionMode, PermissionRule, ProviderResult, Receiver, RuntimeBuilder,
    StopReason, ToolNature, TurnCompletionStatus, Usage, collect_until_turn_completed,
};
use tokio::sync::mpsc;

fn git(root: &Path, args: &[&str]) -> Vec<u8> {
    let output = std::process::Command::new("git")
        .current_dir(root)
        .args([
            "-c",
            "maintenance.autoDetach=false",
            "-c",
            "gc.autoDetach=false",
            "-c",
            "core.autocrlf=false",
        ])
        .args(args)
        .output()
        .expect("fixture Git");
    assert!(output.status.success(), "{output:?}");
    output.stdout
}

struct Model {
    turns: AtomicUsize,
    reviews: Arc<AtomicUsize>,
    allowed: bool,
    assessor_failure: bool,
    tool_name: &'static str,
    operation: &'static str,
    intent_fragment: &'static str,
    input: Value,
}
#[async_trait]
impl LanguageModel for Model {
    async fn stream(&self, messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
        let first = self.turns.fetch_add(1, Ordering::SeqCst) == 0;
        let (tx, rx) = mpsc::channel(3);
        if first {
            tx.try_send(AgentEvent::ToolCall {
                call: ToolCall {
                    id: self.tool_name.into(),
                    name: self.tool_name.into(),
                    input: self.input.clone(),
                },
                provenance: Default::default(),
                summary_fields: vec![],
            })
            .expect("call");
        } else {
            let result = messages
                .iter()
                .find_map(|message| match message {
                    Message::Tool { result } if result.tool_use_id == self.tool_name => {
                        Some(result)
                    }
                    _ => None,
                })
                .expect("paired real tool result");
            assert_eq!(result.is_error, !self.allowed, "{}", result.content);
            tx.try_send(AgentEvent::TextDelta {
                delta: "done".into(),
            })
            .expect("text");
        }
        tx.try_send(AgentEvent::TurnEnd {
            stop_reason: if first {
                StopReason::ToolUse
            } else {
                StopReason::EndTurn
            },
            usage: Usage::default(),
        })
        .expect("end");
        Ok(rx)
    }
    async fn stream_auto_review(
        &self,
        messages: &[Message],
        limits: DecisionRequestLimits,
    ) -> ProviderResult<Receiver<AgentEvent>> {
        assert_eq!(limits.max_retries, 0);
        let payload = messages
            .iter()
            .find_map(|message| match message {
                Message::User { content } => Some(content),
                _ => None,
            })
            .expect("isolated review payload");
        let request: Value = serde_json::from_str(payload).expect("review JSON");
        assert_eq!(request["schema"], "prepared-command/v1");
        assert_eq!(request["tool"], self.tool_name);
        assert_eq!(request["command"]["operation"], self.operation);
        assert_eq!(
            request["command"]["effects"]
                .as_array()
                .expect("effects")
                .len(),
            6
        );
        assert_eq!(request["command"]["confined"], false);
        assert!(
            request["user_intent"]
                .as_str()
                .expect("trusted intent")
                .contains(self.intent_fragment)
        );
        self.reviews.fetch_add(1, Ordering::SeqCst);
        if self.assessor_failure {
            return Err(talos_core::provider::ProviderError::NetworkError(
                "fixture assessor unavailable".into(),
            ));
        }
        let response = json!({"schema":"prepared-command/v1", "request_digest":request["request_digest"],
            "decision":if self.allowed || self.tool_name == "git_pull" { "allow_once" } else { "human_required" }, "confidence":"high", "acknowledged_effects":request["command"]["effects"],
            "accepts_unconfined_effects":true});
        let (tx, rx) = mpsc::channel(2);
        tx.try_send(AgentEvent::TextDelta {
            delta: response.to_string(),
        })
        .expect("response");
        tx.try_send(AgentEvent::TurnEnd {
            stop_reason: StopReason::EndTurn,
            usage: Usage::default(),
        })
        .expect("end");
        Ok(rx)
    }
}
struct Human {
    final_approvals: Arc<AtomicUsize>,
    allow_final: bool,
    drift_repository: Option<std::path::PathBuf>,
}
#[async_trait]
impl ApprovalHandler for Human {
    async fn request_approval(&self, _: &str, _: &Value, _: &[String]) -> ApprovalChoice {
        panic!("scope-aware route required")
    }
    async fn request_scoped_approval(
        &self,
        _: &str,
        _: &Value,
        _: &[String],
        preview: &GrantPreview,
    ) -> ApprovalChoice {
        if preview.facets().iter().any(|facet| {
            matches!(
                facet.normalized_scope.as_str(),
                "git:pull-command"
                    | "git:add-command"
                    | "git:commit-command"
                    | "git:checkout-command"
                    | "git:push-command"
            )
        }) {
            if !self.allow_final
                && preview.facets().iter().any(|facet| {
                    facet.normalized_scope != "git:pull-command"
                        && facet.normalized_scope.starts_with("git:")
                        && facet.normalized_scope.ends_with("-command")
                })
            {
                self.final_approvals.fetch_add(1, Ordering::SeqCst);
                return ApprovalChoice::Deny;
            }
            assert!(
                self.allow_final,
                "Auto-only case must not fall back to human command approval"
            );
            self.final_approvals.fetch_add(1, Ordering::SeqCst);
            if let Some(root) = &self.drift_repository {
                git(root, &["config", "pull.rebase", "true"]);
            }
        }
        ApprovalChoice::ApproveOnce
    }
}

#[tokio::test]
async fn runtime_pull_executes_auto_or_human_fallback_and_preserves_explicit_policy() {
    for scenario in [
        "auto",
        "deny",
        "human_only",
        "assessor_failure",
        "conflict",
        "config_drift",
    ] {
        let conflict = scenario == "conflict";
        let denied = scenario == "deny";
        let drift = scenario == "config_drift";
        let no_fetch = denied || drift;
        let human_only = scenario == "human_only" || drift;
        let assessor_failure = scenario == "assessor_failure";
        let root = tempfile::tempdir().expect("fixture");
        let source = root.path().join("source");
        let work = root.path().join("work");
        std::fs::create_dir(&source).expect("source");
        git(&source, &["init", "-b", "main"]);
        for (key, value) in [
            ("user.name", "Fixture"),
            ("user.email", "fixture@example.invalid"),
            ("commit.gpgSign", "false"),
            ("core.autocrlf", "false"),
        ] {
            git(&source, &["config", key, value]);
        }
        git(&source, &["commit", "--allow-empty", "-m", "base"]);
        git(
            root.path(),
            &[
                "clone",
                source.to_str().expect("source path"),
                work.to_str().expect("work path"),
            ],
        );
        for (key, value) in [
            ("user.name", "Fixture"),
            ("user.email", "fixture@example.invalid"),
            ("commit.gpgSign", "false"),
            ("core.autocrlf", "false"),
            ("pull.rebase", "false"),
        ] {
            git(&work, &["config", key, value]);
        }
        if conflict {
            std::fs::write(work.join("upstream.txt"), "local content\n").expect("local write");
            git(&work, &["add", "upstream.txt"]);
            git(&work, &["commit", "-m", "local divergent addition"]);
        }
        let before = git(&work, &["rev-parse", "HEAD"]);
        std::fs::write(source.join("upstream.txt"), "new upstream content\n").expect("write");
        git(&source, &["add", "upstream.txt"]);
        git(&source, &["commit", "-m", "upstream"]);
        let after = git(&source, &["rev-parse", "HEAD"]);
        let reviews = Arc::new(AtomicUsize::new(0));
        let final_approvals = Arc::new(AtomicUsize::new(0));
        let mut builder = RuntimeBuilder::new()
            .workspace_root(&work)
            .provider(Arc::new(Model {
                turns: AtomicUsize::new(0),
                reviews: reviews.clone(),
                allowed: !no_fetch && !conflict,
                assessor_failure,
                tool_name: "git_pull",
                operation: "git:pull-command/v1",
                intent_fragment: "upstream",
                input: json!({}),
            }))
            .shared_tools()
            .approval_handler(Arc::new(Human {
                final_approvals: final_approvals.clone(),
                allow_final: human_only || assessor_failure,
                drift_repository: drift.then(|| work.clone()),
            }))
            .permission_mode(PermissionMode::Interactive)
            .auto_assistance(true);
        if denied {
            builder = builder.permission_rule(PermissionRule::new_nature(
                ToolNature::Execute,
                Some("git:pull-command".into()),
                None,
                PermissionDecision::Deny("fixture deny".into()),
            ));
        }
        if human_only {
            builder = builder.permission_rule(PermissionRule::new_nature(
                ToolNature::Write,
                Some("unseen-helper-output/**".into()),
                None,
                PermissionDecision::Ask,
            ));
        }
        let mut runtime = builder.build().expect("runtime");
        runtime
            .submit("update this repository from its configured upstream")
            .await
            .expect("submit");
        let terminal = tokio::time::timeout(
            Duration::from_secs(30),
            collect_until_turn_completed(&mut runtime),
        )
        .await
        .expect("bounded turn")
        .expect("terminal");
        assert!(matches!(terminal, TurnCompletionStatus::Success { .. }));
        assert_eq!(
            reviews.load(Ordering::SeqCst),
            usize::from(!denied && !human_only),
            "{scenario}"
        );
        assert_eq!(
            final_approvals.load(Ordering::SeqCst),
            usize::from(human_only || assessor_failure),
            "{scenario}"
        );
        assert_eq!(
            git(&work, &["rev-parse", "HEAD"]),
            if no_fetch || conflict {
                before
            } else {
                after.clone()
            }
        );
        assert_eq!(work.join(".git/FETCH_HEAD").exists(), !no_fetch);
        assert_eq!(work.join("upstream.txt").exists(), !no_fetch);
        if conflict {
            assert_eq!(
                std::fs::read(work.join(".git/MERGE_HEAD")).expect("merge state retained"),
                after
            );
            assert!(!git(&work, &["ls-files", "--unmerged"]).is_empty());
            let content = std::fs::read_to_string(work.join("upstream.txt")).expect("conflict");
            assert!(content.contains("<<<<<<<") && content.contains(">>>>>>>"));
        }
        runtime.shutdown().await.expect("shutdown");
    }
}

#[tokio::test]
async fn runtime_git_add_commit_checkout_and_push_use_operation_specific_auto_review() {
    for (tool_name, operation, intent_fragment) in [
        ("git_add", "git:add-command/v1", "stage"),
        ("git_commit", "git:commit-command/v1", "commit"),
        ("git_checkout", "git:checkout-command/v1", "switch"),
        ("git_push", "git:push-command/v1", "push"),
    ] {
        for allowed in [false, true] {
            let root = tempfile::tempdir().expect("fixture");
            let work = root.path().join("work");
            std::fs::create_dir(&work).expect("work");
            git(&work, &["init", "-b", "main"]);
            for (key, value) in [
                ("user.name", "Fixture"),
                ("user.email", "fixture@example.invalid"),
                ("commit.gpgSign", "false"),
                ("core.autocrlf", "false"),
            ] {
                git(&work, &["config", key, value]);
            }
            std::fs::write(work.join("tracked.txt"), "initial\n").expect("initial");
            git(&work, &["add", "tracked.txt"]);
            git(&work, &["commit", "-m", "initial"]);
            let input = match tool_name {
                "git_add" => {
                    std::fs::write(work.join("added.txt"), "added\n").expect("add input");
                    json!({"paths":["added.txt"]})
                }
                "git_commit" => {
                    std::fs::write(work.join("committed.txt"), "committed\n")
                        .expect("commit input");
                    git(&work, &["add", "committed.txt"]);
                    json!({"message":"runtime operation commit"})
                }
                "git_checkout" => {
                    git(&work, &["branch", "target"]);
                    json!({"branch":"target"})
                }
                "git_push" => {
                    let receiver = root.path().join("receiver.git");
                    git(
                        root.path(),
                        &["init", "--bare", receiver.to_str().expect("receiver path")],
                    );
                    git(
                        &work,
                        &[
                            "remote",
                            "add",
                            "origin",
                            receiver.to_str().expect("receiver path"),
                        ],
                    );
                    json!({"branch":"main"})
                }
                _ => unreachable!(),
            };
            let reviews = Arc::new(AtomicUsize::new(0));
            let final_approvals = Arc::new(AtomicUsize::new(0));
            let mut runtime = RuntimeBuilder::new()
                .workspace_root(&work)
                .provider(Arc::new(Model {
                    turns: AtomicUsize::new(0),
                    reviews: reviews.clone(),
                    allowed,
                    assessor_failure: false,
                    tool_name,
                    operation,
                    intent_fragment,
                    input,
                }))
                .shared_tools()
                .approval_handler(Arc::new(Human {
                    final_approvals: final_approvals.clone(),
                    allow_final: false,
                    drift_repository: None,
                }))
                .permission_mode(PermissionMode::Interactive)
                .auto_assistance(true)
                .build()
                .expect("runtime");
            runtime
                .submit(format!("{intent_fragment} this change in this repository"))
                .await
                .expect("submit");
            let terminal = tokio::time::timeout(
                Duration::from_secs(30),
                collect_until_turn_completed(&mut runtime),
            )
            .await
            .expect("bounded turn")
            .expect("terminal");
            assert!(matches!(terminal, TurnCompletionStatus::Success { .. }));
            assert_eq!(reviews.load(Ordering::SeqCst), 1);
            assert_eq!(
                final_approvals.load(Ordering::SeqCst),
                usize::from(!allowed)
            );
            match tool_name {
                "git_add" => assert_eq!(
                    !git(&work, &["diff", "--cached", "--name-only"]).is_empty(),
                    allowed
                ),
                "git_commit" => assert_eq!(
                    String::from_utf8(git(&work, &["log", "-1", "--format=%s"]))
                        .expect("subject")
                        .trim(),
                    if allowed {
                        "runtime operation commit"
                    } else {
                        "initial"
                    }
                ),
                "git_checkout" => assert_eq!(
                    String::from_utf8(git(&work, &["branch", "--show-current"]))
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
            runtime.shutdown().await.expect("shutdown");
        }
    }
}

#[path = "support/i300_native_pull_recursion.rs"]
mod recursion;

#[path = "support/i300_native_pull_cancellation.rs"]
mod cancellation;
