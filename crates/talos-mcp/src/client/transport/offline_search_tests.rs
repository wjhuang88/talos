use super::*;

use crate::client::dispatcher::McpDispatcher;
use crate::types::McpToolDescriptor;
use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

const MAX_FIXTURE_BODY: usize = 64 * 1024;
const MAX_FIXTURE_REQUESTS: usize = 4;
const SYNTHETIC_QUERY: &str = "I337 synthetic search fixture";
const SYNTHETIC_RESULT_URL: &str = "https://synthetic.invalid/fixture";
const STEPS: [&str; 4] = [
    "initialize",
    "notifications/initialized",
    "tools/list",
    "tools/call",
];

#[derive(Clone, Copy)]
enum ListShape {
    Pinned,
    ExtraTools,
    Duplicate,
    MissingPinned,
    SchemaDrift,
}

#[derive(Clone, Copy)]
enum Reply {
    Json,
    Sse,
    MalformedJson,
    MalformedSse,
    WrongId,
    RpcError,
    Http(u16),
    Redirect,
    Partial,
    Oversized,
    ToolError,
    InvalidResultUrl,
    Delay(Duration),
}

#[derive(Clone, Copy)]
struct Fixture {
    tool: &'static str,
    list_shape: ListShape,
    list_reply: Reply,
    call_reply: Reply,
}

fn pinned_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "query": { "type": "string" }
        },
        "required": ["query"],
        "additionalProperties": false
    })
}

fn offline_transport(url: String) -> McpTransport {
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(REQUEST_TIMEOUT)
        .build()
        .expect("build isolated offline HTTP client");
    let state = StreamableHttpTransportState {
        client,
        url,
        headers: HeaderMap::new(),
        next_id: AtomicU64::new(1),
        server: "i337-synthetic".to_string(),
    };
    McpTransport {
        kind: Arc::new(TransportKind::StreamableHttp(Arc::new(state))),
    }
}

async fn spawn_fixture(
    fixture: Fixture,
    request_count: usize,
) -> (String, JoinHandle<Vec<String>>) {
    spawn_fixture_with_control(fixture, request_count, None).await
}

async fn spawn_fixture_with_control(
    fixture: Fixture,
    request_count: usize,
    cancellation: Option<(oneshot::Sender<()>, oneshot::Receiver<()>)>,
) -> (String, JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
        .await
        .expect("bind IPv4 loopback fixture");
    let address = listener.local_addr().expect("read local fixture address");
    let task = tokio::spawn(async move {
        let mut methods = Vec::new();
        let mut cancellation = cancellation;
        for _ in 0..request_count {
            let (mut socket, _) = tokio::time::timeout(Duration::from_secs(2), listener.accept())
                .await
                .expect("local MCP request deadline")
                .expect("accept local MCP request");
            let (request, has_credentials) =
                tokio::time::timeout(Duration::from_secs(2), read_fixture_request(&mut socket))
                    .await
                    .expect("local MCP request body deadline")
                    .expect("read bounded fixture request");
            assert_eq!(request["jsonrpc"], "2.0");
            assert!(
                !has_credentials,
                "fixture requests must not contain credentials"
            );
            let method = request["method"].as_str().expect("MCP method").to_string();
            methods.push(method.clone());

            match method.as_str() {
                "initialize" => {
                    assert_eq!(request["id"], 1);
                    assert_eq!(request["params"]["protocolVersion"], "2024-11-05");
                }
                "notifications/initialized" => {
                    assert!(request.get("id").is_none());
                }
                "tools/list" => {
                    assert_eq!(request["id"], 2);
                    assert_eq!(request["params"], serde_json::json!({}));
                }
                "tools/call" => {
                    assert_eq!(request["id"], 3);
                    assert_eq!(
                        request["params"],
                        serde_json::json!({
                            "name": fixture.tool,
                            "arguments": { "query": SYNTHETIC_QUERY }
                        })
                    );
                }
                _ => panic!("unexpected local MCP method: {method}"),
            }

            if method == "tools/list"
                && let Some((received, release)) = cancellation.take()
            {
                received.send(()).expect("signal parsed tools/list request");
                tokio::time::timeout(Duration::from_secs(2), release)
                    .await
                    .expect("bounded cancellation fixture release")
                    .expect("release cancellation fixture response");
            }

            if method == "notifications/initialized" {
                write_http_response(&mut socket, 202, "application/json", b"", None, None).await;
                continue;
            }

            let reply = match method.as_str() {
                "initialize" => Reply::Json,
                "tools/list" => fixture.list_reply,
                "tools/call" => fixture.call_reply,
                _ => unreachable!(),
            };
            if let Reply::Delay(delay) = reply {
                tokio::time::sleep(delay).await;
            }
            let result = match method.as_str() {
                "initialize" => serde_json::json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": { "tools": {} },
                    "serverInfo": { "name": "synthetic", "version": "1" }
                }),
                "tools/list" => tool_list(fixture.tool, fixture.list_shape),
                "tools/call" => tool_result(reply),
                _ => unreachable!(),
            };
            let id = request["id"].as_u64().expect("numeric request id");
            let local_redirect = format!("http://{address}/sink");
            write_rpc_response(&mut socket, id, result, reply, Some(&local_redirect)).await;
        }

        if matches!(fixture.list_reply, Reply::Redirect)
            && let Ok(Ok((mut socket, _))) =
                tokio::time::timeout(Duration::from_millis(100), listener.accept()).await
        {
            let (request, _) = read_fixture_request(&mut socket)
                .await
                .expect("read local redirect-sink request");
            methods.push(format!(
                "redirect-followed:{}",
                request["method"].as_str().unwrap_or("unknown")
            ));
        }
        methods
    });
    (format!("http://{address}/mcp"), task)
}

fn tool_list(tool: &str, shape: ListShape) -> Value {
    let pinned = serde_json::json!({
        "name": tool,
        "description": "Synthetic search tool",
        "inputSchema": pinned_schema()
    });
    let extra = ["web_research", "web_fetch", "image_search"]
        .into_iter()
        .map(|name| serde_json::json!({ "name": name, "inputSchema": pinned_schema() }));
    let tools = match shape {
        ListShape::Pinned => vec![pinned],
        ListShape::ExtraTools => std::iter::once(pinned).chain(extra).collect(),
        ListShape::Duplicate => vec![pinned.clone(), pinned],
        ListShape::MissingPinned => vec![serde_json::json!({
            "name": "web_research",
            "inputSchema": pinned_schema()
        })],
        ListShape::SchemaDrift => vec![serde_json::json!({
            "name": tool,
            "inputSchema": { "type": "object" }
        })],
    };
    serde_json::json!({ "tools": tools })
}

fn tool_result(reply: Reply) -> Value {
    match reply {
        Reply::ToolError => serde_json::json!({
            "isError": true,
            "content": [{ "type": "text", "text": "synthetic tool error" }]
        }),
        Reply::InvalidResultUrl => serde_json::json!({
            "structuredContent": {
                "results": [{ "url": "file:///synthetic/not-fetched", "title": "Synthetic title" }]
            }
        }),
        _ => serde_json::json!({
            "structuredContent": {
                "results": [{ "url": SYNTHETIC_RESULT_URL, "title": "Synthetic title" }]
            }
        }),
    }
}

async fn write_rpc_response(
    socket: &mut TcpStream,
    id: u64,
    result: Value,
    reply: Reply,
    redirect: Option<&str>,
) {
    if let Reply::Http(status) = reply {
        write_http_response(
            socket,
            status,
            "application/json",
            br#"{"error":"synthetic private body"}"#,
            None,
            None,
        )
        .await;
        return;
    }
    if matches!(reply, Reply::Redirect) {
        write_http_response(socket, 302, "application/json", b"", None, redirect).await;
        return;
    }
    if matches!(reply, Reply::Oversized) {
        write_http_response(
            socket,
            200,
            "application/json",
            &vec![b'x'; MAX_FIXTURE_BODY + 1],
            None,
            None,
        )
        .await;
        return;
    }

    let response_id = if matches!(reply, Reply::WrongId) {
        id + 1
    } else {
        id
    };
    let (content_type, body) = match reply {
        Reply::MalformedJson => ("application/json", b"{".to_vec()),
        Reply::MalformedSse => ("text/event-stream", b"event: message\ndata: {\n\n".to_vec()),
        Reply::Partial => {
            let body = serde_json::to_vec(&serde_json::json!({
                "jsonrpc": "2.0", "id": response_id, "result": result
            }))
            .expect("serialize partial fixture");
            write_http_response(
                socket,
                200,
                "application/json",
                &body,
                Some(body.len() + 8),
                None,
            )
            .await;
            return;
        }
        Reply::RpcError => {
            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "id": response_id,
                "error": { "code": -32000, "message": "synthetic RPC error" }
            });
            (
                "application/json",
                serde_json::to_vec(&response).expect("serialize RPC error"),
            )
        }
        _ => {
            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "id": response_id,
                "result": result
            });
            let json = serde_json::to_vec(&response).expect("serialize fixture response");
            if matches!(reply, Reply::Sse) {
                (
                    "text/event-stream",
                    format!(
                        "event: message\ndata: {}\n\n",
                        String::from_utf8_lossy(&json)
                    )
                    .into_bytes(),
                )
            } else {
                ("application/json", json)
            }
        }
    };
    write_http_response(socket, 200, content_type, &body, None, None).await;
}

async fn write_http_response(
    socket: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
    declared_length: Option<usize>,
    location: Option<&str>,
) {
    let reason = match status {
        200 => "OK",
        202 => "Accepted",
        301 => "Moved Permanently",
        302 => "Found",
        401 => "Unauthorized",
        403 => "Forbidden",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        _ => "Fixture Response",
    };
    let mut headers = format!(
        "HTTP/1.1 {status} {reason}\r\ncontent-type: {content_type}\r\ncontent-length: {}\r\nconnection: close\r\n",
        declared_length.unwrap_or(body.len())
    );
    if let Some(location) = location {
        headers.push_str(&format!("location: {location}\r\n"));
    }
    headers.push_str("\r\n");
    let _ = socket.write_all(headers.as_bytes()).await;
    let _ = socket.write_all(body).await;
}

async fn read_fixture_request(socket: &mut TcpStream) -> std::io::Result<(Value, bool)> {
    let mut reader = BufReader::new(socket);
    let mut content_length = 0_usize;
    let mut header_bytes = 0_usize;
    let mut has_credentials = false;
    loop {
        let mut line = Vec::new();
        let read = reader.read_until(b'\n', &mut line).await?;
        if read == 0 {
            return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof));
        }
        header_bytes += read;
        if header_bytes > 8 * 1024 {
            return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
        }
        let lower = String::from_utf8_lossy(&line).to_ascii_lowercase();
        if lower.starts_with("content-length:") {
            content_length = lower
                .trim()
                .strip_prefix("content-length:")
                .and_then(|value| value.trim().parse().ok())
                .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidData))?;
        }
        if lower.starts_with("authorization:") || lower.starts_with("cookie:") {
            has_credentials = true;
        }
        if line == b"\r\n" {
            break;
        }
    }
    if content_length == 0 || content_length > MAX_FIXTURE_BODY {
        return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
    }
    let mut body = vec![0; content_length];
    reader.read_exact(&mut body).await?;
    let request = serde_json::from_slice(&body)
        .map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidData))?;
    Ok((request, has_credentials))
}

fn only_pinned_search_tool(tools: &[McpToolDescriptor], expected_name: &str) -> bool {
    tools.len() == 1 && tools[0].name == expected_name && tools[0].input_schema == pinned_schema()
}

async fn call_admitted_tool(
    dispatcher: &McpDispatcher,
    tools: &[McpToolDescriptor],
    expected_name: &str,
    requested_name: &str,
) -> Option<crate::error::Result<Value>> {
    if requested_name != expected_name || !only_pinned_search_tool(tools, expected_name) {
        return None;
    }
    Some(
        dispatcher
            .call_tool(
                requested_name,
                serde_json::json!({ "query": SYNTHETIC_QUERY }),
            )
            .await,
    )
}

fn sanitized_result_count(result: &Value) -> Option<usize> {
    if result.get("isError").and_then(Value::as_bool) == Some(true) {
        return None;
    }
    let rows = result
        .get("structuredContent")?
        .get("results")?
        .as_array()?;
    rows.iter()
        .all(|row| {
            row.as_object().is_some_and(|object| {
                object.len() == 2
                    && object.get("url").and_then(Value::as_str) == Some(SYNTHETIC_RESULT_URL)
                    && object.get("title").and_then(Value::as_str).is_some()
            })
        })
        .then_some(rows.len())
}

fn fixture(tool: &'static str) -> Fixture {
    Fixture {
        tool,
        list_shape: ListShape::Pinned,
        list_reply: Reply::Json,
        call_reply: Reply::Json,
    }
}

fn expected_steps(count: usize) -> Vec<String> {
    STEPS[..count]
        .iter()
        .map(|method| (*method).to_string())
        .collect()
}

#[tokio::test]
async fn pinned_exa_and_parallel_tools_roundtrip_over_json_and_sse() {
    for (tool, requested_name) in [
        ("web_search_exa", "web_search_exa"),
        ("web_search", "web_search"),
    ] {
        for reply in [Reply::Json, Reply::Sse] {
            let mut fixture = fixture(tool);
            fixture.call_reply = reply;
            let (url, server) = spawn_fixture(fixture, MAX_FIXTURE_REQUESTS).await;
            let dispatcher =
                McpDispatcher::new("i337-synthetic".to_string(), offline_transport(url));
            dispatcher
                .initialize()
                .await
                .expect("initialize local MCP fixture");
            let tools = dispatcher.list_tools().await.expect("list synthetic tools");
            assert!(only_pinned_search_tool(&tools, tool));

            let forbidden_names = match tool {
                "web_search_exa" => ["web_research", "web_fetch", "image_search", "web_search"],
                "web_search" => [
                    "web_research",
                    "web_fetch",
                    "image_search",
                    "web_search_exa",
                ],
                _ => unreachable!(),
            };
            for forbidden_name in forbidden_names {
                assert!(
                    call_admitted_tool(&dispatcher, &tools, tool, forbidden_name)
                        .await
                        .is_none(),
                    "unexpectedly admitted {forbidden_name} for {tool}"
                );
            }

            let result = call_admitted_tool(&dispatcher, &tools, tool, requested_name)
                .await
                .expect("pinned call is admitted")
                .expect("synthetic call succeeds");
            assert_eq!(sanitized_result_count(&result), Some(1));

            let methods = server.await.expect("fixture task");
            assert_eq!(methods, expected_steps(MAX_FIXTURE_REQUESTS));
            assert!(methods.len() <= MAX_FIXTURE_REQUESTS);
        }
    }
}

#[tokio::test]
async fn unexpected_or_drifted_tools_never_reach_the_call_path() {
    for tool in ["web_search_exa", "web_search"] {
        for shape in [
            ListShape::ExtraTools,
            ListShape::Duplicate,
            ListShape::MissingPinned,
            ListShape::SchemaDrift,
        ] {
            let mut fixture = fixture(tool);
            fixture.list_shape = shape;
            let (url, server) = spawn_fixture(fixture, 3).await;
            let dispatcher =
                McpDispatcher::new("i337-synthetic".to_string(), offline_transport(url));
            dispatcher
                .initialize()
                .await
                .expect("initialize local MCP fixture");
            let tools = dispatcher
                .list_tools()
                .await
                .expect("decode local tool list");
            assert!(!only_pinned_search_tool(&tools, tool));
            assert!(
                call_admitted_tool(&dispatcher, &tools, tool, tool)
                    .await
                    .is_none()
            );
            assert!(
                call_admitted_tool(&dispatcher, &tools, tool, "web_research")
                    .await
                    .is_none()
            );
            assert!(
                call_admitted_tool(&dispatcher, &tools, tool, "web_fetch")
                    .await
                    .is_none()
            );
            assert!(
                call_admitted_tool(&dispatcher, &tools, tool, "image_search")
                    .await
                    .is_none()
            );
            assert_eq!(server.await.expect("fixture task"), expected_steps(3));
        }
    }
}

#[tokio::test]
async fn malformed_responses_ids_http_errors_redirects_and_budgets_are_bounded() {
    for reply in [
        Reply::MalformedJson,
        Reply::MalformedSse,
        Reply::WrongId,
        Reply::RpcError,
        Reply::Http(401),
        Reply::Http(403),
        Reply::Http(429),
        Reply::Http(500),
        Reply::Redirect,
        Reply::Partial,
        Reply::Oversized,
    ] {
        let mut fixture = fixture("web_search_exa");
        fixture.list_reply = reply;
        let (url, server) = spawn_fixture(fixture, 3).await;
        let dispatcher = McpDispatcher::new("i337-synthetic".to_string(), offline_transport(url));
        dispatcher
            .initialize()
            .await
            .expect("initialize local MCP fixture");
        let error = dispatcher
            .list_tools()
            .await
            .expect_err("malformed fixture response must fail")
            .to_string();
        match reply {
            Reply::MalformedJson | Reply::Oversized => {
                assert!(error.contains("MCP JSON error"), "{error}");
            }
            Reply::MalformedSse => {
                assert!(error.contains("did not include request id"), "{error}");
            }
            Reply::WrongId => assert!(error.contains("mismatched response id"), "{error}"),
            Reply::RpcError => assert!(error.contains("synthetic RPC error"), "{error}"),
            Reply::Http(status) => {
                assert!(error.contains(&format!("status {status}")), "{error}");
            }
            Reply::Redirect => assert!(error.contains("status 302"), "{error}"),
            Reply::Partial => assert!(error.contains("MCP HTTP error"), "{error}"),
            _ => unreachable!(),
        }
        let methods = server.await.expect("fixture task");
        assert_eq!(methods, expected_steps(3));
        assert!(methods.len() <= 3, "failed listing must not call any tool");
        if matches!(reply, Reply::Redirect) {
            assert!(
                !methods
                    .iter()
                    .any(|method| method.starts_with("redirect-followed:"))
            );
        }
    }
}

#[tokio::test]
async fn result_url_filtering_and_tool_errors_are_local_and_sanitized() {
    for (reply, accepted) in [
        (Reply::Json, Some(1)),
        (Reply::InvalidResultUrl, None),
        (Reply::ToolError, None),
    ] {
        let mut fixture = fixture("web_search");
        fixture.call_reply = reply;
        let (url, server) = spawn_fixture(fixture, MAX_FIXTURE_REQUESTS).await;
        let dispatcher = McpDispatcher::new("i337-synthetic".to_string(), offline_transport(url));
        dispatcher
            .initialize()
            .await
            .expect("initialize local MCP fixture");
        let tools = dispatcher.list_tools().await.expect("list synthetic tools");
        let result = call_admitted_tool(&dispatcher, &tools, "web_search", "web_search")
            .await
            .expect("pinned call is admitted")
            .expect("fixture response received");
        assert_eq!(sanitized_result_count(&result), accepted);
        assert_eq!(
            server.await.expect("fixture task"),
            expected_steps(MAX_FIXTURE_REQUESTS)
        );
    }
}

#[tokio::test]
async fn streamable_http_deadline_and_caller_cancellation_are_in_flight() {
    let mut deadline_fixture = fixture("web_search_exa");
    deadline_fixture.list_reply = Reply::Delay(Duration::from_millis(650));
    let (url, server) = spawn_fixture(deadline_fixture, 3).await;
    let transport = offline_transport(url);
    let dispatcher = McpDispatcher::new("i337-synthetic".to_string(), transport.clone());
    dispatcher
        .initialize()
        .await
        .expect("initialize local MCP fixture");
    let timeout_error = transport
        .request("tools/list", Some(serde_json::json!({})))
        .await
        .expect_err("HTTP deadline should expire");
    assert!(matches!(timeout_error, McpError::Http(ref error) if error.is_timeout()));
    assert_eq!(server.await.expect("fixture task"), expected_steps(3));

    let cancellation_fixture = fixture("web_search");
    let (received, request_received) = oneshot::channel();
    let (release_response, response_released) = oneshot::channel();
    let (url, server) =
        spawn_fixture_with_control(cancellation_fixture, 3, Some((received, response_released)))
            .await;
    let transport = offline_transport(url);
    McpDispatcher::new("i337-synthetic".to_string(), transport.clone())
        .initialize()
        .await
        .expect("initialize local MCP fixture");
    let request_transport = transport.clone();
    let request = tokio::spawn(async move {
        request_transport
            .request("tools/list", Some(serde_json::json!({})))
            .await
    });
    tokio::time::timeout(Duration::from_secs(2), request_received)
        .await
        .expect("bounded wait for parsed tools/list request")
        .expect("fixture signaled tools/list receipt");
    request.abort();
    assert!(
        matches!(request.await, Err(error) if error.is_cancelled()),
        "request future was not cancelled while awaiting its response"
    );
    release_response
        .send(())
        .expect("release fixture after caller cancellation");
    assert_eq!(
        tokio::time::timeout(Duration::from_secs(3), server)
            .await
            .expect("bounded cancellation fixture cleanup")
            .expect("fixture task"),
        expected_steps(3)
    );
}

#[tokio::test]
async fn tls_style_failure_is_injected_only_on_loopback() {
    let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
        .await
        .expect("bind local TLS-failure fixture");
    let address = listener.local_addr().expect("local fixture address");
    let sink = tokio::spawn(async move {
        let (mut socket, _) = tokio::time::timeout(Duration::from_secs(2), listener.accept())
            .await
            .expect("TLS probe deadline")
            .expect("accept TLS probe");
        let mut bytes = [0; 512];
        let _ = socket.read(&mut bytes).await;
    });
    let transport = offline_transport(format!("https://{address}/mcp"));
    let error = transport
        .request("tools/list", Some(serde_json::json!({})))
        .await
        .expect_err("plain local sink is not TLS");
    assert!(matches!(error, McpError::Http(_)));
    sink.await.expect("TLS sink task");
}
