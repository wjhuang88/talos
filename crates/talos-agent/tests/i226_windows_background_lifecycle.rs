#![cfg(windows)]

#[path = "support/lifecycle_clock.rs"]
mod lifecycle_clock;

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
use talos_core::provider::{LanguageModel, ProviderError, ProviderResult};
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
                "$ErrorActionPreference = 'Stop'; $child = Start-Process powershell.exe -ArgumentList '-NoLogo','-NoProfile','-NonInteractive','-Command','Start-Sleep -Seconds 300' -PassThru; Set-Content -LiteralPath '{}' -Value $child.Id; Start-Sleep -Seconds 300",
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
                            "timeout_secs": if matches!(self.case, LifecycleCase::Timeout) { 12 } else { 30 },
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
            wait_for_marker(&self.marker)
                .await
                .map_err(ProviderError::ServerError)?;
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

fn marker_pid(marker: &Path) -> Option<u32> {
    std::fs::read_to_string(marker).ok()?.trim().parse().ok()
}

async fn wait_for_marker(marker: &Path) -> Result<u32, String> {
    let marker = marker.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let start = std::time::Instant::now();
        loop {
            if let Some(pid) = marker_pid(&marker) {
                return Ok(pid);
            }
            if start.elapsed() >= Duration::from_secs(25) {
                return Err(format!(
                    "OS readiness deadline exceeded; marker {}: {:?}",
                    marker.display(),
                    std::fs::read_to_string(&marker)
                ));
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    })
    .await
    .map_err(|error| format!("marker worker failed: {error}"))?
}

async fn process_exists(pid: u32) -> Result<bool, String> {
    let mut child = tokio::process::Command::new("powershell.exe")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &format!(
                "if (Get-Process -Id {pid} -ErrorAction SilentlyContinue) {{ exit 0 }} else {{ exit 1 }}"
            ),
        ])
        .kill_on_drop(true)
        .spawn()
        .map_err(|error| format!("process query could not start: {error}"))?;
    let status = tokio::time::timeout(Duration::from_secs(5), child.wait())
        .await
        .map_err(|_| "process query timed out".to_owned())?
        .map_err(|error| format!("process query failed: {error}"))?;
    match status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Err(format!("unexpected process query status: {status}")),
    }
}

async fn process_gone(pid: u32) -> Result<(), String> {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            if !process_exists(pid).await? {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .map_err(|_| format!("Windows Job Object left grandchild process {pid} alive"))?
}

async fn run_case(case: LifecycleCase, expected: BackgroundJobState) {
    tokio::time::pause();
    let clock = lifecycle_clock::ReadinessClock::hold().await;
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
    let mut actor_task = tokio::spawn(async move { actor.run().await });

    sq_tx
        .send(SessionOp::Submit {
            message: "exercise the Windows background lifecycle".into(),
        })
        .await
        .expect("submit succeeds");
    let mut observations = Vec::new();
    let mut early_terminal = None;
    let readiness = wait_for_marker(&marker);
    tokio::pin!(readiness);
    let readiness_result = loop {
        tokio::select! {
            result = &mut readiness => break result,
            event = eq_rx.recv() => {
                let Some(event) = event else {
                    break Err("session event channel closed before readiness".into());
                };
                observations.push(format!("{event:?}"));
                if let SessionEvent::BackgroundJobTerminal { summary, .. } = event {
                    if let Some(pid) = marker_pid(&marker) {
                        early_terminal = Some(summary);
                        break Ok(pid);
                    }
                    break Err(format!("process terminated before readiness: {summary:?}"));
                }
            }
        }
    };
    if readiness_result.is_ok() && matches!(case, LifecycleCase::Timeout) {
        // Trigger the real supervisor deadline only after the OS child exists.
        clock
            .advance_past_execution_deadline(Duration::from_secs(12))
            .await;
    }
    tokio::time::resume();
    clock.release().await;
    let grandchild_pid = match readiness_result {
        Ok(pid) => pid,
        Err(error) => {
            let _ = sq_tx.send(SessionOp::Shutdown).await;
            let shutdown = tokio::time::timeout(Duration::from_secs(10), &mut actor_task).await;
            if shutdown.is_err() {
                actor_task.abort();
                let _ = actor_task.await;
            }
            panic!("{error}; session events: {observations:?}; shutdown: {shutdown:?}");
        }
    };
    if matches!(case, LifecycleCase::Shutdown) {
        sq_tx
            .send(SessionOp::Shutdown)
            .await
            .expect("shutdown submits");
    }

    let summary = tokio::time::timeout(Duration::from_secs(15), async {
        if let Some(summary) = early_terminal {
            return Ok(summary);
        }
        loop {
            match eq_rx.recv().await {
                Some(SessionEvent::BackgroundJobTerminal { summary, .. }) => break Ok(summary),
                Some(event) => observations.push(format!("{event:?}")),
                None => break Err("session closed without a terminal summary"),
            }
        }
    })
    .await;
    // Observe cleanup before fallback shutdown so shutdown cannot mask a broken timeout/cancel.
    let cleanup = process_gone(grandchild_pid).await;
    let _ = sq_tx.send(SessionOp::Shutdown).await;
    let shutdown = tokio::time::timeout(Duration::from_secs(10), &mut actor_task).await;
    if shutdown.is_err() {
        actor_task.abort();
        let _ = actor_task.await;
    }
    cleanup.expect("terminal lifecycle cleans the real grandchild before fallback shutdown");
    shutdown
        .expect("session actor shuts down")
        .expect("session actor joins");
    let summary = summary
        .unwrap_or_else(|error| {
            panic!("terminal deadline failed: {error}; events: {observations:?}")
        })
        .unwrap_or_else(|error| panic!("{error}; events: {observations:?}"));
    assert_eq!(summary.state, expected);
    assert!(summary.cleanup_error.is_none(), "{summary:?}");
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
