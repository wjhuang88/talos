use std::process::Stdio;

use rmcp::ServiceExt;
use rmcp::model::{CallToolRequestParams, PaginatedRequestParams};
use rmcp::transport::{ConfigureCommandExt, TokioChildProcess};
use tokio::process::Command;

use rmcp::transport::Transport;

/// Transport wrapper that captures all outbound JSON-RPC frames.
struct CapturingTransport {
    inner: TokioChildProcess,
    sent_frames: std::sync::Arc<tokio::sync::Mutex<Vec<String>>>,
}

impl CapturingTransport {
    fn new(
        inner: TokioChildProcess,
        sent_frames: std::sync::Arc<tokio::sync::Mutex<Vec<String>>>,
    ) -> Self {
        Self { inner, sent_frames }
    }
}

impl Transport<rmcp::RoleClient> for CapturingTransport {
    type Error = std::io::Error;

    fn send(
        &mut self,
        item: rmcp::service::TxJsonRpcMessage<rmcp::RoleClient>,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send + 'static {
        let raw = serde_json::to_string(&item).unwrap_or_else(|_| "{}".to_string());
        let frames = self.sent_frames.clone();
        let fut = self.inner.send(item);
        async move {
            frames.lock().await.push(raw);
            fut.await
        }
    }

    fn receive(
        &mut self,
    ) -> impl std::future::Future<Output = Option<rmcp::service::RxJsonRpcMessage<rmcp::RoleClient>>>
    + Send {
        self.inner.receive()
    }

    fn close(&mut self) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send {
        self.inner.close()
    }
}

#[tokio::test]
async fn mcp_server_e2e() {
    let child = Command::new(env!("CARGO_BIN_EXE_talos")).configure(|cmd| {
        cmd.args(["--mode", "mcp-server"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
    });

    let transport = TokioChildProcess::new(child).expect("spawn talos mcp server");
    let frames = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let transport = CapturingTransport::new(transport, frames.clone());
    let client = ().serve(transport).await.expect("connect rmcp client to talos");

    let tools = client
        .peer()
        .list_tools(Some(PaginatedRequestParams::default()))
        .await
        .expect("tools/list succeeds");
    assert!(
        tools.tools.len() >= 5,
        "expected >=5 tools, got {}",
        tools.tools.len()
    );

    let denied = client
        .peer()
        .call_tool(CallToolRequestParams {
            meta: None,
            name: "bash".into(),
            arguments: Some(
                serde_json::json!({"command": "echo denied"})
                    .as_object()
                    .cloned()
                    .expect("object"),
            ),
            task: None,
        })
        .await;
    assert!(denied.is_err(), "expected denied tool call to return error");

    let sent = frames.lock().await;
    assert!(!sent.is_empty(), "expected outbound JSON-RPC frames");
    for frame in sent.iter() {
        assert!(
            serde_json::from_str::<serde_json::Value>(frame).is_ok(),
            "non-json frame: {frame}"
        );
    }

    let _ = client.cancel().await;
}
