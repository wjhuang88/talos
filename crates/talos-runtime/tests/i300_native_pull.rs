//! Real Runtime -> Agent -> provider Auto -> supervised Git, using local repositories.

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
}
#[async_trait]
impl LanguageModel for Model {
    async fn stream(&self, messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
        let first = self.turns.fetch_add(1, Ordering::SeqCst) == 0;
        let (tx, rx) = mpsc::channel(3);
        if first {
            tx.try_send(AgentEvent::ToolCall {
                call: ToolCall {
                    id: "pull".into(),
                    name: "git_pull".into(),
                    input: json!({}),
                },
                provenance: Default::default(),
                summary_fields: vec![],
            })
            .expect("call");
        } else {
            let result = messages
                .iter()
                .find_map(|message| match message {
                    Message::Tool { result } if result.tool_use_id == "pull" => Some(result),
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
                .contains("upstream")
        );
        self.reviews.fetch_add(1, Ordering::SeqCst);
        if self.assessor_failure {
            return Err(talos_core::provider::ProviderError::NetworkError(
                "fixture assessor unavailable".into(),
            ));
        }
        let response = json!({"schema":"prepared-command/v1", "request_digest":request["request_digest"],
            "decision":"allow_once", "confidence":"high", "acknowledged_effects":request["command"]["effects"],
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
        if preview
            .facets()
            .iter()
            .any(|facet| facet.normalized_scope == "git:pull-command")
        {
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
            }))
            .tool(Arc::new(talos_tools::git::GitPullTool::new(work.clone())))
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

#[path = "support/i300_native_pull_recursion.rs"]
mod recursion;

#[cfg(unix)]
#[path = "support/i300_native_pull_cancellation.rs"]
mod cancellation;
