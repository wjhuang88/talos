use std::sync::Arc;

use async_trait::async_trait;
use talos_core::message::{AgentEvent, Message, StopReason, Usage};
use talos_rpc::{RpcServer, Runtime, RuntimeError};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;

struct MockRuntime;

#[async_trait]
impl Runtime for MockRuntime {
    async fn run(&self, _user_message: String) -> Result<String, RuntimeError> {
        Ok("hi".to_string())
    }

    async fn run_streaming(
        &self,
        _user_message: String,
        _history: Vec<Message>,
        event_tx: mpsc::UnboundedSender<AgentEvent>,
    ) -> Result<String, RuntimeError> {
        let _ = event_tx.send(AgentEvent::TurnStart);
        let _ = event_tx.send(AgentEvent::TextDelta {
            delta: "hi".to_string(),
        });
        let _ = event_tx.send(AgentEvent::TurnEnd {
            stop_reason: StopReason::EndTurn,
            usage: Usage::default(),
        });
        Ok("hi".to_string())
    }
}

#[tokio::test]
async fn non_streaming_agent_run_returns_final_result() {
    let server = RpcServer::new(Arc::new(MockRuntime));

    let (mut client_in, server_in) = tokio::io::duplex(8 * 1024);
    let (server_out, mut client_out) = tokio::io::duplex(8 * 1024);

    let server_task = tokio::spawn(async move { server.run(server_in, server_out).await });

    client_in
        .write_all(
            b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"agent.run\",\"params\":{\"prompt\":\"echo hi\",\"stream\":false}}\n",
        )
        .await
        .expect("write request");
    client_in.shutdown().await.expect("close input");

    let mut output = String::new();
    client_out
        .read_to_string(&mut output)
        .await
        .expect("read response");

    server_task
        .await
        .expect("join server task")
        .expect("server result");

    let response_line = output.lines().next().expect("response line");
    let response: serde_json::Value = serde_json::from_str(response_line).expect("parse response");
    let final_result = response["result"]["result"]
        .as_str()
        .expect("result string");
    assert!(final_result.contains("hi"), "response: {response}");
}
