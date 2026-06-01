use std::sync::Arc;

use talos_agent::Agent;
use talos_core::tool::ToolRegistry;
use talos_provider::mock::MockProvider;
use talos_rpc::server::RpcServer;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn non_streaming_agent_run_returns_final_result() {
    #[allow(deprecated)]
    let agent = Agent::new(
        Arc::new(MockProvider::new().with_response("hi")),
        ToolRegistry::new(),
    );
    let server = RpcServer::new(Arc::new(agent));

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
