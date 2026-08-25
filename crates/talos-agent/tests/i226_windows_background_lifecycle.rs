#![cfg(windows)]

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use talos_agent::Agent;
use talos_agent::permission_pipeline::{
    ApprovalResolver, ApprovalResolverError, PermissionApprovalRequest,
};
use talos_agent::session::AppServerSession;
use talos_core::ApprovalChoice;
use talos_core::background_job::BackgroundJobState;
use talos_core::message::{AgentEvent, Message, StopReason, ToolCall, Usage};
use talos_core::provider::{LanguageModel, ProviderResult};
use talos_core::session::{RuntimePolicy, SessionConfig, SessionEvent, SessionOp};
use talos_core::tool::ToolRegistry;
use talos_permission::PermissionEngine;
use talos_tools::BashTool;
use tokio::sync::mpsc;

#[derive(Clone, Copy)]
enum LifecycleCase {
    Timeout,
    Cancel,
    Shutdown,
}

struct LifecycleModel {
    case: LifecycleCase,
    marker: PathBuf,
    calls: AtomicUsize,
}

struct ApproveOnceResolver;

#[async_trait]
impl ApprovalResolver for ApproveOnceResolver {
    async fn resolve(
        &self,
        _request: PermissionApprovalRequest,
        _remaining: Duration,
    ) -> Result<ApprovalChoice, ApprovalResolverError> {
        Ok(ApprovalChoice::ApproveOnce)
    }
}

#[async_trait]
impl LanguageModel for LifecycleModel {
    async fn stream(&self, messages: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        let call = self.calls.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = mpsc::channel(8);
        let events = if call == 0 {
            let script = format!(
                "$child = Start-Process powershell.exe -ArgumentList '-NoLogo','-NoProfile','-NonInteractive','-Command','Start-Sleep -Seconds 30' -PassThru; Set-Content -LiteralPath '{}' -Value $child.Id; Start-Sleep -Seconds 30",
                self.marker.display()
            );
            vec![
                AgentEvent::TurnStart,
                AgentEvent::ToolCall {
                    call: ToolCall {
                        id: "windows-background".into(),
                        name: "powershell".into(),
                        input: serde_json::json!({
                            "command": script,
                            "background": true,
                            // Leave enough startup margin for a cold Windows runner while
                            // still timing out long before the fixture's 30-second children.
                            "timeout_secs": if matches!(self.case, LifecycleCase::Timeout) { 5 } else { 30 },
                        }),
                    },
                    provenance: Default::default(),
                    summary_fields: Vec::new(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::ToolUse,
                    usage: Usage::default(),
                },
            ]
        } else if call == 1 && matches!(self.case, LifecycleCase::Cancel) {
            wait_for_marker(&self.marker).await;
            let job_id = messages
                .iter()
                .rev()
                .find_map(|message| match message {
                    Message::Tool { result } => {
                        serde_json::from_str::<serde_json::Value>(&result.content)
                            .ok()
                            .and_then(|value| value["job_id"].as_str().map(str::to_owned))
                    }
                    _ => None,
                })
                .expect("background receipt must expose a job id");
            vec![
                AgentEvent::TurnStart,
                AgentEvent::ToolCall {
                    call: ToolCall {
                        id: "windows-background-cancel".into(),
                        name: "process".into(),
                        input: serde_json::json!({"action": "cancel", "job_id": job_id}),
                    },
                    provenance: Default::default(),
                    summary_fields: Vec::new(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::ToolUse,
                    usage: Usage::default(),
                },
            ]
        } else {
            vec![
                AgentEvent::TurnStart,
                AgentEvent::TextDelta {
                    delta: "background lifecycle observed".into(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: Usage::default(),
                },
            ]
        };
        tokio::spawn(async move {
            for event in events {
                if tx.send(event).await.is_err() {
                    break;
                }
            }
        });
        Ok(rx)
    }
}

async fn wait_for_marker(marker: &Path) -> u32 {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            if let Ok(value) = std::fs::read_to_string(marker)
                && let Ok(pid) = value.trim().parse()
            {
                return pid;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("grandchild marker must arrive")
}

fn process_exists(pid: u32) -> bool {
    std::process::Command::new("powershell.exe")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &format!(
                "if (Get-Process -Id {pid} -ErrorAction SilentlyContinue) {{ exit 0 }} else {{ exit 1 }}"
            ),
        ])
        .status()
        .is_ok_and(|status| status.success())
}

async fn assert_process_gone(pid: u32) {
    for _ in 0..40 {
        if !process_exists(pid) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("Windows Job Object left grandchild process {pid} alive");
}

async fn run_case(case: LifecycleCase, expected: BackgroundJobState) {
    let workspace = tempfile::tempdir().expect("workspace");
    let marker = workspace.path().join("grandchild.pid");
    let mut tools = ToolRegistry::new();
    tools.register(Arc::new(BashTool::new(workspace.path().to_path_buf())));
    let agent = Agent::with_security(
        Arc::new(LifecycleModel {
            case,
            marker: marker.clone(),
            calls: AtomicUsize::new(0),
        }),
        tools,
        Some(Arc::new(PermissionEngine::with_workspace_root(
            workspace.path().to_path_buf(),
        ))),
        None,
        workspace.path().to_path_buf(),
    )
    .with_approval_resolver(Arc::new(ApproveOnceResolver));
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: workspace.path().to_path_buf(),
        initial_history: Vec::new(),
        model_context_limit: 128_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);
    let sq_tx = handle.sq_tx;
    let mut eq_rx = handle.eq_rx;
    let actor_task = tokio::spawn(async move { actor.run().await });

    sq_tx
        .send(SessionOp::Submit {
            message: "exercise the Windows background lifecycle".into(),
        })
        .await
        .expect("submit succeeds");
    let grandchild_pid = wait_for_marker(&marker).await;
    if matches!(case, LifecycleCase::Shutdown) {
        sq_tx
            .send(SessionOp::Shutdown)
            .await
            .expect("shutdown submits");
    }

    let summary = tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            if let Some(SessionEvent::BackgroundJobTerminal { summary, .. }) = eq_rx.recv().await {
                break summary;
            }
        }
    })
    .await
    .expect("session must emit one terminal background summary");
    assert_eq!(summary.state, expected);
    assert!(summary.cleanup_error.is_none());
    assert_process_gone(grandchild_pid).await;

    let _ = sq_tx.send(SessionOp::Shutdown).await;
    tokio::time::timeout(Duration::from_secs(10), actor_task)
        .await
        .expect("session actor shuts down")
        .expect("session actor joins");
}

#[tokio::test]
async fn windows_agent_supervisor_timeout_cleans_the_job_tree() {
    run_case(LifecycleCase::Timeout, BackgroundJobState::TimedOut).await;
}

#[tokio::test]
async fn windows_agent_process_cancel_cleans_the_job_tree() {
    run_case(LifecycleCase::Cancel, BackgroundJobState::Cancelled).await;
}

#[tokio::test]
async fn windows_session_shutdown_cleans_the_job_tree() {
    run_case(LifecycleCase::Shutdown, BackgroundJobState::Cancelled).await;
}
