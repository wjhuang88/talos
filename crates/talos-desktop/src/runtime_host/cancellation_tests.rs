//! Native sandbox integration: readiness comes from the actual descendant, not a delay.

use super::*;
use std::time::Duration;
use talos_provider::mock::MockProvider;

async fn running_shell_is_cleaned_up(shutdown: bool) {
    if !talos_sandbox::create_sandbox().is_available() {
        eprintln!("native sandbox unavailable; running-shell acceptance is not verified here");
        return;
    }
    let workspace = tests::TestWorkspace::new();
    let command = "echo $$ > leader.pid; sh -c 'echo $$ > descendant.pid; echo ready > ready; sleep 30; echo unexpected > late' & wait";
    let provider = MockProvider::new()
        .with_tool_call("bash", serde_json::json!({"command": command}))
        .with_response("Shell completed.");
    let mut host = RuntimeHost::start(Arc::new(provider), workspace.0.clone()).expect("host");
    host.try_send(RuntimeCommand::Submit("run the shell fixture".into()))
        .expect("submit");
    tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            match host.recv().await.expect("approval event") {
                RuntimeOutput::ApprovalRequested { request_id, .. } => {
                    host.try_send(RuntimeCommand::ApprovalResponse {
                        request_id,
                        choice: ApprovalChoice::ApproveOnce,
                    })
                    .expect("approve exact request");
                    break;
                }
                RuntimeOutput::Completed { status } => panic!("premature completion: {status:?}"),
                RuntimeOutput::Error(error) => panic!("host error: {error}"),
                _ => {}
            }
        }
        tokio::time::timeout(Duration::from_secs(4), async {
            while !workspace.0.join("ready").exists() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("actual descendant must announce readiness before cancellation");

        host.try_send(if shutdown {
            RuntimeCommand::Shutdown
        } else {
            RuntimeCommand::Interrupt
        })
        .expect("cancel running shell");
        loop {
            match host.recv().await.expect("cancellation terminal") {
                RuntimeOutput::Stopped if shutdown => break,
                RuntimeOutput::Completed { status } if !shutdown => {
                    assert_eq!(status, TerminalStatus::Cancelled);
                    break;
                }
                RuntimeOutput::Error(error) => panic!("cleanup not confirmed: {error}"),
                _ => {}
            }
        }
        // The receipt must precede the visible terminal. A zombie has stopped
        // executing; only its OS parent can reap an orphaned grandchild.
        for name in ["leader.pid", "descendant.pid"] {
            let pid = std::fs::read_to_string(workspace.0.join(name)).expect("fixture PID");
            let pid = pid.trim().parse::<u32>().expect("numeric PID");
            let state = std::process::Command::new("ps")
                .args(["-p", &pid.to_string(), "-o", "stat="])
                .output()
                .expect("native process state inspection");
            let state = String::from_utf8_lossy(&state.stdout);
            assert!(
                state.trim().is_empty() || state.trim().starts_with('Z'),
                "{name} still executing at terminal: {state}"
            );
        }
        assert!(!workspace.0.join("late").exists());
        if !shutdown {
            host.try_send(RuntimeCommand::Shutdown).expect("shutdown");
            loop {
                match host.recv().await.expect("shutdown result") {
                    RuntimeOutput::Stopped => break,
                    RuntimeOutput::Error(error) => panic!("shutdown failed: {error}"),
                    _ => {}
                }
            }
        }
    })
    .await
    .expect("bounded running-shell cancellation");
}

#[tokio::test(flavor = "current_thread")]
async fn running_shell_interrupt_waits_for_descendant_cleanup() {
    running_shell_is_cleaned_up(false).await;
}

#[tokio::test(flavor = "current_thread")]
async fn running_shell_shutdown_waits_for_descendant_cleanup() {
    running_shell_is_cleaned_up(true).await;
}
