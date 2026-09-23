use super::*;
use std::sync::atomic::AtomicUsize;
use std::time::Duration;
use talos_provider::mock::MockProvider;
use talos_runtime::{DecisionRequestLimits, Message, ProviderResult, Receiver};

struct AssessmentProvider {
    conversation: MockProvider,
    assessments: AtomicUsize,
    allow: bool,
}

#[async_trait]
impl LanguageModel for AssessmentProvider {
    async fn stream(&self, messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
        self.conversation.stream(messages).await
    }

    async fn stream_auto_review(
        &self,
        messages: &[Message],
        limits: DecisionRequestLimits,
    ) -> ProviderResult<Receiver<AgentEvent>> {
        self.assessments.fetch_add(1, Ordering::SeqCst);
        assert_eq!(messages.len(), 2, "review must use isolated context");
        assert_eq!(limits.max_retries, 0);
        assert!(limits.max_output_tokens > 0);
        let Message::System { content, .. } = &messages[0] else {
            panic!("review system instruction expected");
        };
        assert!(content.contains("permission risk assessor"));
        let Message::User { content } = &messages[1] else {
            panic!("review payload expected");
        };
        let payload: serde_json::Value =
            serde_json::from_str(content.split_once('\n').expect("review JSON delimiter").1)
                .expect("valid assessment payload");
        let digest = payload["request_digest"]
            .as_str()
            .expect("request-bound digest");
        assert!(!digest.is_empty());
        let response = serde_json::json!({
            "schema_version": 1,
            "request_digest": digest,
            "decision": if self.allow { "allow_once" } else { "human_required" },
            "effect": if self.allow { "read_only" } else { "unknown" },
            "reason_code": if self.allow { "bounded_read_only_command" } else { "uncertain" },
            "confidence": if self.allow { "high" } else { "low" },
            "effect_summary": "Inspect the current directory.",
            "decision_points": ["Should this inspection run now?"]
        });
        MockProvider::new()
            .with_response(response.to_string())
            .stream(&[])
            .await
    }
}

struct Workspace(PathBuf);

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

async fn exercise_auto(enabled: bool, allow: bool) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let workspace = Workspace(std::env::temp_dir().join(format!(
        "talos-desktop-auto-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    )));
    std::fs::create_dir(&workspace.0).expect("unique workspace");
    let shell = if cfg!(windows) { "powershell" } else { "bash" };
    let provider = Arc::new(AssessmentProvider {
        conversation: MockProvider::new()
            .with_tool_call(shell, serde_json::json!({"command": "pwd"}))
            .with_response("Inspection request finished."),
        assessments: AtomicUsize::new(0),
        allow,
    });
    let configured = provider.clone();
    let mut host = RuntimeHost::start_with(
        move || Ok((configured, 128_000, enabled)),
        workspace.0.clone(),
    )
    .expect("host starts");
    host.try_send(RuntimeCommand::Submit(
        "Inspect the working directory.".into(),
    ))
    .expect("submit");
    let mut reports = Vec::new();
    let mut approvals = 0;
    let mut results = Vec::new();
    let mut explanations = Vec::new();
    let outcome = tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            match host.recv().await.expect("host event") {
                RuntimeOutput::AutoDecision {
                    outcome, evaluator, ..
                } => {
                    reports.push((outcome, evaluator));
                }
                RuntimeOutput::ApprovalRequested {
                    request_id,
                    explanation,
                    ..
                } => {
                    approvals += 1;
                    explanations.push(explanation);
                    host.try_send(RuntimeCommand::ApprovalResponse {
                        request_id,
                        choice: ApprovalChoice::Deny,
                    })
                    .expect("human denial");
                }
                RuntimeOutput::ToolResult {
                    is_error, content, ..
                } => results.push((is_error, content)),
                RuntimeOutput::Completed { .. } => break,
                RuntimeOutput::Error(error) => panic!("host failed: {error}"),
                _ => {}
            }
        }
    })
    .await;
    host.try_send(RuntimeCommand::Shutdown).expect("shutdown");
    tokio::time::timeout(Duration::from_secs(10), async {
        while !matches!(host.recv().await, Some(RuntimeOutput::Stopped) | None) {}
    })
    .await
    .expect("host shutdown completes");
    outcome.expect("approval flow completes");
    assert_eq!(results.len(), 1, "one authoritative tool result");
    assert_eq!(approvals, usize::from(!(enabled && allow)));
    if !allow {
        assert!(results[0].0, "manual Deny must reach the tool result");
        assert!(results[0].1.to_ascii_lowercase().contains("denied"));
    } else if results[0].0 {
        assert!(
            results[0].1.to_ascii_lowercase().contains("sandbox"),
            "Auto allowance must not mask an unrelated execution failure: {:?}",
            results[0]
        );
    } else {
        let expected = workspace.0.canonicalize().expect("canonical workspace");
        assert!(
            results[0].1.contains(expected.to_string_lossy().as_ref()),
            "command must execute in the assessed workspace: {:?}",
            results[0]
        );
    }
    if enabled && !allow {
        assert!(
            explanations[0].contains("Should this inspection run now?"),
            "explanations={explanations:?}, reports={reports:?}"
        );
    }
    assert_eq!(
        provider.assessments.load(Ordering::SeqCst),
        usize::from(enabled)
    );
    if enabled {
        assert_eq!(
            reports,
            [(
                if allow {
                    "allow_once"
                } else {
                    "human_required"
                }
                .into(),
                "configured-model".into()
            )]
        );
    } else {
        assert!(
            reports.is_empty(),
            "disabled Auto must not invent an assessment"
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn enabled_auto_consults_provider_reports_and_falls_back_to_human() {
    exercise_auto(true, false).await;
}

#[tokio::test(flavor = "current_thread")]
async fn disabled_auto_uses_human_approval_without_assessment() {
    exercise_auto(false, false).await;
}

#[tokio::test(flavor = "current_thread")]
async fn enabled_auto_allow_once_reports_without_manual_approval() {
    // Execution still passes through the real platform sandbox after approval;
    // this test asserts permission behavior, not host sandbox availability.
    exercise_auto(true, true).await;
}
