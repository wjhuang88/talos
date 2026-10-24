//! Running native Pull cleanup at the MCP connection boundary.

use super::*;
use std::os::unix::fs::PermissionsExt;
use std::{
    pin::Pin,
    sync::atomic::{AtomicBool, Ordering},
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, ReadBuf};

struct FailingReader<R> {
    inner: R,
    fail: Arc<AtomicBool>,
    observed: Arc<AtomicBool>,
}

impl<R: AsyncRead + Unpin> AsyncRead for FailingReader<R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        if self.fail.load(Ordering::SeqCst) {
            self.observed.store(true, Ordering::SeqCst);
            return Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionReset,
                "fixture transport reset",
            )));
        }
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

#[tokio::test]
async fn disconnect_reaps_running_pull_and_preserves_completed_integration() {
    running_pull_connection_failure(false).await;
}

#[tokio::test]
async fn transport_error_reaps_running_pull_and_preserves_completed_integration() {
    running_pull_connection_failure(true).await;
}

async fn running_pull_connection_failure(transport_error: bool) {
    tokio::time::timeout(Duration::from_secs(20), async {
        let root = tempfile::tempdir().expect("workspace");
        git(root.path(), &["init", "-b", "main"]);
        for (key, value) in [
            ("user.name", "Fixture"),
            ("user.email", "fixture@example.invalid"),
            ("commit.gpgSign", "false"),
            ("core.autocrlf", "false"),
            ("pull.rebase", "false"),
            ("remote.origin.url", "."),
            ("remote.origin.fetch", "+refs/heads/*:refs/remotes/origin/*"),
            ("branch.main.remote", "origin"),
            ("branch.main.merge", "refs/heads/upstream"),
        ] {
            git(root.path(), &["config", key, value]);
        }
        std::fs::write(root.path().join("file"), "before\n").expect("base");
        git(root.path(), &["add", "file"]);
        git(root.path(), &["commit", "-m", "base"]);
        git(root.path(), &["checkout", "-b", "upstream"]);
        std::fs::write(root.path().join("file"), "after\n").expect("update");
        git(root.path(), &["commit", "-am", "upstream"]);
        let expected_head = git(root.path(), &["rev-parse", "HEAD"]);
        git(root.path(), &["checkout", "main"]);
        let hook = root.path().join(".git/hooks/post-merge");
        std::fs::write(
            &hook,
            "#!/bin/sh\n(while test ! -f release-effect; do sleep 0.01; done; printf forbidden > delayed-effect) &\nprintf observed > hook-started\nwait\n",
        )
        .expect("hook");
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755))
            .expect("executable hook");
        git(root.path(), &["config", "core.hooksPath", ".git/hooks"]);

        let mut engine = PermissionEngine::empty();
        engine.set_workspace_root(root.path().into());
        for nature in [ToolNature::Read, ToolNature::Execute, ToolNature::Write, ToolNature::Network] {
            engine.add_rule(PermissionRule::new_nature(
                nature, Some("*".into()), None, PermissionDecision::Allow,
            ));
        }
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(talos_tools::git::GitPullTool::new(root.path().into())));
        let handler = Arc::new(TalosMcpHandler::new_with_owner_lifecycle(
            Arc::new(registry),
            Arc::new(McpPermissionGate::new(Arc::new(engine), Arc::new(HookRegistry::new()))),
            true,
        ));
        let (client, server) = tokio::io::duplex(65536);
        let fail = Arc::new(AtomicBool::new(false));
        let observed = Arc::new(AtomicBool::new(false));
        let (server_read, server_write) = tokio::io::split(server);
        let server_read = FailingReader { inner: server_read, fail: fail.clone(), observed: observed.clone() };
        let task = tokio::spawn(async move {
            let service = handler.clone().serve((server_read, server_write)).await.expect("handshake");
            let _transport_result = service.waiting().await;
            handler.shutdown(Duration::from_secs(5)).await.expect("owned Git cleanup");
        });
        let (read, mut write) = tokio::io::split(client);
        let mut read = BufReader::new(read);
        let init = json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{
            "protocolVersion":"2025-03-26","capabilities":{},
            "clientInfo":{"name":"pull-disconnect-test","version":"1"}
        }});
        write.write_all(format!("{init}\n").as_bytes()).await.expect("initialize");
        let mut line = String::new();
        read.read_line(&mut line).await.expect("initialize response");
        assert!(serde_json::from_str::<Value>(&line).expect("JSON").get("result").is_some());
        write.write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n")
            .await.expect("initialized");
        let call = json!({"jsonrpc":"2.0","id":1,"method":"tools/call",
            "params":{"name":"git_pull","arguments":{}}});
        write.write_all(format!("{call}\n").as_bytes()).await.expect("Pull");
        while !root.path().join("hook-started").exists() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        if transport_error {
            fail.store(true, Ordering::SeqCst);
            // Wake the pending read so the injected I/O error, rather than EOF,
            // reaches the server while the client remains connected.
            write.write_all(b"\n").await.expect("wake failed transport");
        } else {
            drop(read);
            drop(write);
        }
        task.await.expect("connection cleanup joined");
        assert_eq!(observed.load(Ordering::SeqCst), transport_error);
        std::fs::write(root.path().join("release-effect"), "release\n").expect("release barrier");
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert!(!root.path().join("delayed-effect").exists());
        assert_eq!(git(root.path(), &["rev-parse", "HEAD"]), expected_head);
        assert_eq!(std::fs::read_to_string(root.path().join("file")).expect("integrated file"), "after\n");
    }).await.expect("running Pull disconnect deadline");
}
