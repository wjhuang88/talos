use std::sync::Arc;

use serde_json::{Value, json};
use talos_mcp::client::dispatcher::McpDispatcher;
use talos_mcp::client::transport::McpTransport;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

#[tokio::test]
async fn mock_stdio_roundtrip_tools_list_and_call() {
    let (client_io, server_io) = tokio::io::duplex(16 * 1024);
    let (client_reader, client_writer) = tokio::io::split(client_io);
    let (server_reader, mut server_writer) = tokio::io::split(server_io);

    let server_task = tokio::spawn(async move {
        let mut lines = tokio::io::BufReader::new(server_reader).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let request: Value = match serde_json::from_str(&line) {
                Ok(value) => value,
                Err(_) => continue,
            };
            let id = request.get("id").and_then(Value::as_u64).unwrap_or(0);
            let method = request
                .get("method")
                .and_then(Value::as_str)
                .unwrap_or_default();

            let response = match method {
                "tools/list" => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "tools": [
                            {
                                "name": "echo",
                                "description": "Echo text",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": { "text": { "type": "string" } },
                                    "required": ["text"]
                                },
                                "annotations": { "readOnlyHint": true }
                            }
                        ]
                    }
                }),
                "tools/call" => {
                    let text = request
                        .get("params")
                        .and_then(|p| p.get("arguments"))
                        .and_then(|a| a.get("text"))
                        .and_then(Value::as_str)
                        .unwrap_or_default();
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [
                                { "type": "text", "text": format!("echo:{text}") }
                            ]
                        }
                    })
                }
                _ => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32601, "message": "unknown method" }
                }),
            };

            let serialized = serde_json::to_vec(&response).expect("serialize response");
            if server_writer.write_all(&serialized).await.is_err() {
                break;
            }
            if server_writer.write_all(b"\n").await.is_err() {
                break;
            }
        }
    });

    let transport = McpTransport::from_io("mock".to_string(), client_reader, client_writer);
    let dispatcher = Arc::new(McpDispatcher::new("mock".to_string(), transport));

    let tools = dispatcher
        .list_tools()
        .await
        .expect("list tools should succeed");
    assert_eq!(tools.len(), 1);

    let called = dispatcher
        .call_tool("echo", json!({ "text": "hello" }))
        .await
        .expect("call tool should succeed");
    let text = called["content"][0]["text"]
        .as_str()
        .expect("text field present");
    assert_eq!(text, "echo:hello");

    server_task.abort();
}
