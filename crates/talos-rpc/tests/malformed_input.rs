use std::sync::Arc;

use talos_agent::Agent;
use talos_core::tool::ToolRegistry;
use talos_provider::mock::MockProvider;
use talos_rpc::server::RpcServer;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn malformed_frames_return_expected_codes() {
    #[allow(deprecated)]
    let agent = Agent::new(Arc::new(MockProvider::new()), ToolRegistry::new());
    let server = RpcServer::new(Arc::new(agent));

    let (mut client_in, server_in) = tokio::io::duplex(8 * 1024);
    let (server_out, mut client_out) = tokio::io::duplex(8 * 1024);

    let server_task = tokio::spawn(async move { server.run(server_in, server_out).await });

    client_in
        .write_all(b"not json\n{\"foo\":\"bar\"}\n{\"jsonrpc\":\"1.0\",\"method\":\"x\"}\n")
        .await
        .expect("write malformed frames");
    client_in.shutdown().await.expect("close input");

    let mut output = String::new();
    client_out
        .read_to_string(&mut output)
        .await
        .expect("read server output");

    server_task
        .await
        .expect("join server task")
        .expect("server result");

    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 3, "output: {output}");

    let r0: serde_json::Value = serde_json::from_str(lines[0]).expect("json line 0");
    let r1: serde_json::Value = serde_json::from_str(lines[1]).expect("json line 1");
    let r2: serde_json::Value = serde_json::from_str(lines[2]).expect("json line 2");

    assert_eq!(r0["error"]["code"], -32700);
    assert_eq!(r1["error"]["code"], -32600);
    assert_eq!(r2["error"]["code"], -32600);
}
