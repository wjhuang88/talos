//! Fixed Git daemon fixture using the existing public background launcher seam.

use std::{future::Future, panic::AssertUnwindSafe, path::Path, time::Duration};

use async_trait::async_trait;
use futures_util::FutureExt;
use talos_core::{
    background_job::{
        BackgroundJobLauncher, BackgroundJobPermit, BackgroundProcessEvent, LaunchedBackgroundJob,
    },
    tool::{AgentTool, ToolResult},
};

struct FixturePermit(tokio::sync::oneshot::Sender<LaunchedBackgroundJob>);

#[async_trait]
impl BackgroundJobPermit for FixturePermit {
    async fn launch(self: Box<Self>, launcher: Box<dyn BackgroundJobLauncher>) -> ToolResult {
        match launcher.launch().await {
            Ok(job) => match self.0.send(job) {
                Ok(()) => ToolResult::success("fixture launched"),
                Err(job) => {
                    let _ = job.control.force_terminate().await;
                    ToolResult::error("fixture receiver closed")
                }
            },
            Err(error) => ToolResult::error(error),
        }
    }
}

// This permit launches only test-owned infrastructure. It does not authorize
// the MCP tool call under test or expose a new production permission path.
pub async fn with_daemon(root: &Path, port: u16, test: impl Future<Output = ()>) {
    let spelling = |path: &Path| {
        let text = path.to_str().expect("fixture path");
        if cfg!(windows) {
            text.replace('\\', "/")
        } else {
            text.to_owned()
        }
    };
    let (sender, receiver) = tokio::sync::oneshot::channel();
    let output = talos_tools::ExecTool::new(root.to_owned())
        .execute_background_authorized_with_output(
            serde_json::json!({
                "command": "git", "background": true,
                "args": ["daemon", "--reuseaddr", "--listen=127.0.0.1",
                    "--export-all", "--enable=receive-pack", format!("--port={port}"),
                    format!("--base-path={}", spelling(root.parent().expect("parent"))),
                    spelling(root)]
            }),
            Box::new(FixturePermit(sender)),
            &[],
        )
        .await;
    assert!(!output.result.is_error, "{}", output.result.content);
    let job = receiver.await.expect("launched daemon");
    let control = job.control;
    let mut events = job.events;
    // The pump never retains the Job control: unwinding closes the Windows Job.
    let drain = tokio::spawn(async move {
        let mut diagnostics = Vec::new();
        while let Some(event) = events.recv().await {
            match event {
                BackgroundProcessEvent::Output(chunk) => {
                    let remaining = 32_768_usize.saturating_sub(diagnostics.len());
                    diagnostics.extend_from_slice(&chunk.bytes[..remaining.min(chunk.bytes.len())]);
                }
                BackgroundProcessEvent::Exited(_) => return Ok(diagnostics),
                BackgroundProcessEvent::SupervisionFailed(error) => return Err(error),
            }
        }
        Err("daemon events ended without exit".to_owned())
    });
    // Dropping the caller (including cancellation) closes this channel. The
    // independent supervisor still owns the control and awaits cleanup.
    let drain_status = drain.abort_handle();
    let (cleanup_sender, cleanup_receiver) = tokio::sync::oneshot::channel::<()>();
    let cleanup = tokio::spawn(async move {
        let _ = cleanup_receiver.await;
        let stopped = control.force_terminate().await;
        let drained = tokio::time::timeout(Duration::from_secs(5), drain).await;
        (stopped, drained)
    });
    // Catch failures inside the fixture lifetime so timeout and assertion panic
    // both await tree termination/reaping before propagating to the test harness.
    let outcome = AssertUnwindSafe(async {
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if std::net::TcpStream::connect_timeout(
                    &([127, 0, 0, 1], port).into(),
                    Duration::from_millis(20),
                )
                .is_ok()
                {
                    break;
                }
                assert!(
                    !drain_status.is_finished(),
                    "daemon exited before listening"
                );
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("daemon ready");
        tokio::time::timeout(Duration::from_secs(30), test)
            .await
            .expect("MCP test deadline");
    })
    .catch_unwind()
    .await;
    drop(cleanup_sender);
    let cleanup_result = cleanup.await;
    if let Err(panic) = outcome {
        // Preserve the original failure even if cleanup also failed.
        eprintln!("daemon cleanup after test panic: {cleanup_result:?}");
        std::panic::resume_unwind(panic);
    }
    let (stopped, drained) = cleanup_result.expect("daemon cleanup supervisor");
    assert!(stopped.is_ok(), "daemon tree termination: {stopped:?}");
    assert!(
        matches!(&drained, Ok(Ok(Ok(_)))),
        "daemon tree drain: {drained:?}"
    );
}

#[tokio::test]
async fn cancellation_reaps_daemon() {
    let root = tempfile::tempdir().expect("fixture");
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("port");
    let port = listener.local_addr().expect("address").port();
    drop(listener);
    let (ready_sender, ready_receiver) = tokio::sync::oneshot::channel();
    let mut fixture = Box::pin(with_daemon(root.path(), port, async {
        ready_sender.send(()).expect("test ready receiver");
        std::future::pending::<()>().await;
    }));
    tokio::select! {
        result = ready_receiver => result.expect("fixture ready"),
        () = &mut fixture => panic!("fixture ended before cancellation"),
    }
    drop(fixture);
    tokio::time::timeout(Duration::from_secs(5), async {
        while std::net::TcpStream::connect_timeout(
            &([127, 0, 0, 1], port).into(),
            Duration::from_millis(20),
        )
        .is_ok()
        {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("cancelled daemon stops listening");
}

#[tokio::test]
async fn assertion_failure_reaps_daemon_before_propagating() {
    let root = tempfile::tempdir().expect("fixture");
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("port");
    let port = listener.local_addr().expect("address").port();
    drop(listener);
    let reached = std::sync::atomic::AtomicBool::new(false);
    let failure = AssertUnwindSafe(with_daemon(root.path(), port, async {
        reached.store(true, std::sync::atomic::Ordering::Relaxed);
        panic!("intentional fixture assertion failure");
    }))
    .catch_unwind()
    .await;
    assert!(reached.load(std::sync::atomic::Ordering::Relaxed));
    let panic = failure.expect_err("original assertion propagated");
    assert_eq!(
        panic.downcast_ref::<&str>(),
        Some(&"intentional fixture assertion failure")
    );
    assert!(
        std::net::TcpStream::connect_timeout(
            &([127, 0, 0, 1], port).into(),
            Duration::from_millis(100),
        )
        .is_err(),
        "daemon still listens after failed test cleanup"
    );
}
