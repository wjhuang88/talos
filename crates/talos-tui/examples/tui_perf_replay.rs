//! Deterministic offline TUI performance replay; no provider or tools execute.
//! Arguments: scenario (idle/animation/stream/approval/workspace-switch), history
//! count, duration seconds, optional workspace and alternate workspace.
//! Drive scrolling/resize through the PTY harness.
use std::{io, time::Duration};
use talos_conversation::{ContentOutput, MessageSource, StatusSnapshot, TurnPhase, UiOutput};
use talos_core::message::Message;
use talos_tui::Tui;
use tokio::sync::mpsc;

#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let scenario = args.get(1).map(String::as_str).unwrap_or("idle");
    if !matches!(
        scenario,
        "idle" | "animation" | "stream" | "approval" | "workspace-switch"
    ) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "unknown scenario",
        ));
    }
    let count = args
        .get(2)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(100);
    let seconds = args
        .get(3)
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(35);
    if count > 10_000 || !(1..=3600).contains(&seconds) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "replay bounds exceeded",
        ));
    }
    let stream_hz = std::env::var("TALOS_REPLAY_STREAM_HZ")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(20);
    if !(1..=1000).contains(&stream_hz) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "stream rate must be 1..=1000",
        ));
    }
    let workspace = args.get(4).cloned().unwrap_or_default();
    let alternate_workspace = args.get(5).cloned().unwrap_or_default();
    if scenario == "workspace-switch"
        && (workspace.is_empty()
            || alternate_workspace.is_empty()
            || workspace == alternate_workspace)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "workspace-switch requires two distinct nonempty workspace paths",
        ));
    }
    let history: Vec<Message> = (0..count).map(|index| Message::Assistant {
        content: format!("Replay {index:05}: deterministic ASCII 中文 emoji 🦀 e\u{301}.\nSecond line for layout."),
        tool_calls: vec![], reasoning: None,
    }).collect();
    let mut tui = Tui::new()?;
    tui.hydrate_history(&history);
    tui.set_workspace_path(workspace.clone());
    if scenario == "approval" {
        tui.show_approval("offline_fixture", r#"{"display_only":true}"#);
    }
    let (tx, rx) = mpsc::unbounded_channel();
    tui.set_ui_output_rx(rx);
    let drive = async {
        if let Ok(path) = std::env::var("TALOS_REPLAY_READY") {
            std::fs::write(path, b"ready\n")?;
        }
        let status = StatusSnapshot {
            model_name: "offline-replay".into(),
            provider: "fixture".into(),
            workspace_path: workspace,
            is_processing: matches!(scenario, "animation" | "stream"),
            phase: matches!(scenario, "animation" | "stream").then_some(TurnPhase::Thinking),
            ..Default::default()
        };
        let _ = tx.send(UiOutput::Status(status.clone()));
        if scenario == "stream" {
            let _ = tx.send(UiOutput::Content(ContentOutput::Start {
                source: MessageSource::Assistant,
            }));
            let mut interval =
                tokio::time::interval(Duration::from_nanos(1_000_000_000 / stream_hz));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            for index in 0..seconds * stream_hz {
                interval.tick().await;
                if tx
                    .send(UiOutput::Content(ContentOutput::Delta {
                        text: format!("token{index:05} 中文 "),
                    }))
                    .is_err()
                {
                    return Ok::<(), io::Error>(());
                }
            }
            let _ = tx.send(UiOutput::Content(ContentOutput::End));
            // Allow the final stream frame to be observed before fixture shutdown.
            tokio::time::sleep(Duration::from_millis(200)).await;
        } else if scenario == "workspace-switch" {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            interval.tick().await;
            for index in 0..seconds {
                interval.tick().await;
                let mut switched = status.clone();
                if index % 2 == 0 {
                    switched.workspace_path.clone_from(&alternate_workspace);
                }
                if tx.send(UiOutput::Status(switched)).is_err() {
                    return Ok(());
                }
            }
        } else {
            tokio::time::sleep(Duration::from_secs(seconds)).await;
        }
        if scenario == "approval" {
            return Ok(());
        }
        let _ = tx.send(UiOutput::Exit);
        std::future::pending::<io::Result<()>>().await
    };
    tokio::select! { result = tui.run() => result, result = drive => result }
}
