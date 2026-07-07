//! MCP transport primitives.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use futures_util::StreamExt;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, oneshot};

use crate::error::{McpError, Result};

#[cfg(not(test))]
const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
#[cfg(test)]
const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(500);

/// Minimal JSON-RPC request envelope used by MCP.
#[derive(Debug, Serialize)]
struct RpcRequest<'a> {
    jsonrpc: &'static str,
    id: u64,
    method: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

/// Minimal JSON-RPC notification envelope used by MCP.
#[derive(Debug, Serialize)]
struct RpcNotification<'a> {
    jsonrpc: &'static str,
    method: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

/// Minimal JSON-RPC error payload.
#[derive(Debug, Clone, Deserialize)]
pub struct RpcErrorBody {
    /// JSON-RPC error code.
    pub code: i64,
    /// JSON-RPC error message.
    pub message: String,
    /// Optional error data.
    pub data: Option<Value>,
}

/// Minimal JSON-RPC response envelope used by MCP.
#[derive(Debug, Deserialize)]
pub struct RpcResponse {
    /// JSON-RPC protocol version.
    pub jsonrpc: String,
    /// Correlated request id.
    pub id: u64,
    /// Success result.
    pub result: Option<Value>,
    /// Error payload.
    pub error: Option<RpcErrorBody>,
}

struct LineTransportState {
    writer: Mutex<Box<dyn AsyncWrite + Send + Unpin>>,
    pending: Mutex<HashMap<u64, oneshot::Sender<Result<Value>>>>,
    next_id: AtomicU64,
    server: String,
}

struct StreamableHttpTransportState {
    client: reqwest::Client,
    url: String,
    headers: HeaderMap,
    next_id: AtomicU64,
    server: String,
}

struct LegacySseTransportState {
    client: reqwest::Client,
    post_url: Mutex<String>,
    headers: HeaderMap,
    pending: Mutex<HashMap<u64, oneshot::Sender<Result<Value>>>>,
    next_id: AtomicU64,
    server: String,
}

enum TransportKind {
    Line(Arc<LineTransportState>),
    StreamableHttp(Arc<StreamableHttpTransportState>),
    LegacySse(Arc<LegacySseTransportState>),
}

/// A line-delimited JSON-RPC transport used by the MCP client.
#[derive(Clone)]
pub struct McpTransport {
    kind: Arc<TransportKind>,
}

impl McpTransport {
    /// Creates a transport from arbitrary async reader/writer halves.
    pub fn from_io<R, W>(server: String, reader: R, writer: W) -> Self
    where
        R: AsyncRead + Send + Unpin + 'static,
        W: AsyncWrite + Send + Unpin + 'static,
    {
        let state = Arc::new(LineTransportState {
            writer: Mutex::new(Box::new(writer)),
            pending: Mutex::new(HashMap::new()),
            next_id: AtomicU64::new(1),
            server,
        });

        let transport = Self {
            kind: Arc::new(TransportKind::Line(state.clone())),
        };

        tokio::spawn(async move {
            let mut lines = BufReader::new(reader).lines();
            loop {
                match lines.next_line().await {
                    Ok(Some(line)) => {
                        let parsed: std::result::Result<RpcResponse, _> =
                            serde_json::from_str(&line);
                        match parsed {
                            Ok(resp) => {
                                let tx = {
                                    let mut pending = state.pending.lock().await;
                                    pending.remove(&resp.id)
                                };
                                if let Some(tx) = tx {
                                    if let Some(err) = resp.error {
                                        let _ = tx.send(Err(McpError::Rpc {
                                            server: state.server.clone(),
                                            method: "unknown".to_string(),
                                            message: format!(
                                                "code={} message={} data={:?}",
                                                err.code, err.message, err.data
                                            ),
                                        }));
                                    } else {
                                        let _ = tx.send(Ok(resp.result.unwrap_or(Value::Null)));
                                    }
                                }
                            }
                            Err(error) => {
                                tracing::warn!(server = %state.server, %error, "failed to parse MCP response line");
                            }
                        }
                    }
                    Ok(None) => {
                        Self::drain_pending_disconnected(&state).await;
                        break;
                    }
                    Err(error) => {
                        tracing::warn!(server = %state.server, %error, "MCP transport read failed");
                        Self::drain_pending_disconnected(&state).await;
                        break;
                    }
                }
            }
        });

        transport
    }

    /// Creates a Streamable HTTP transport.
    pub fn streamable_http(server: String, url: String, headers: HeaderMap) -> Result<Self> {
        validate_http_url(&url)?;
        let client = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(McpError::Http)?;
        Ok(Self {
            kind: Arc::new(TransportKind::StreamableHttp(Arc::new(
                StreamableHttpTransportState {
                    client,
                    url,
                    headers,
                    next_id: AtomicU64::new(1),
                    server,
                },
            ))),
        })
    }

    /// Connects to a legacy HTTP/SSE MCP transport.
    pub async fn connect_legacy_sse(
        server: String,
        url: String,
        post_url: Option<String>,
        headers: HeaderMap,
    ) -> Result<Self> {
        validate_http_url(&url)?;
        if let Some(explicit_post_url) = post_url.as_deref() {
            validate_http_url(explicit_post_url)?;
        }

        let client = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(McpError::Http)?;
        let response = client
            .get(&url)
            .headers(headers.clone())
            .header(ACCEPT, "text/event-stream")
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(McpError::InvalidConfig(format!(
                "server '{server}' SSE connect failed with HTTP {}",
                response.status()
            )));
        }

        let discover_endpoint = post_url.is_none();
        let (endpoint_tx, endpoint_rx) = oneshot::channel();
        let state = Arc::new(LegacySseTransportState {
            client,
            post_url: Mutex::new(post_url.unwrap_or_default()),
            headers,
            pending: Mutex::new(HashMap::new()),
            next_id: AtomicU64::new(1),
            server: server.clone(),
        });

        tokio::spawn(read_legacy_sse_stream(
            state.clone(),
            response.bytes_stream(),
            discover_endpoint.then_some(endpoint_tx),
            url,
        ));

        if state.post_url.lock().await.is_empty() {
            let discovered = tokio::time::timeout(REQUEST_TIMEOUT, endpoint_rx)
                .await
                .map_err(|_| McpError::Timeout {
                    server: server.clone(),
                    method: "sse/connect".to_string(),
                    timeout_secs: REQUEST_TIMEOUT.as_secs(),
                })?
                .map_err(|_| McpError::Disconnected(server.clone()))?;
            validate_http_url(&discovered)?;
        }

        Ok(Self {
            kind: Arc::new(TransportKind::LegacySse(state)),
        })
    }

    /// Sends a JSON-RPC request and waits for a matching response.
    pub async fn request(&self, method: &str, params: Option<Value>) -> Result<Value> {
        match &*self.kind {
            TransportKind::Line(state) => self.request_line(state, method, params).await,
            TransportKind::StreamableHttp(state) => {
                self.request_streamable_http(state, method, params).await
            }
            TransportKind::LegacySse(state) => self.request_legacy_sse(state, method, params).await,
        }
    }

    /// Sends a JSON-RPC notification without waiting for a response.
    pub async fn notify(&self, method: &str, params: Option<Value>) -> Result<()> {
        match &*self.kind {
            TransportKind::Line(state) => self.notify_line(state, method, params).await,
            TransportKind::StreamableHttp(state) => {
                self.notify_streamable_http(state, method, params).await
            }
            TransportKind::LegacySse(state) => self.notify_legacy_sse(state, method, params).await,
        }
    }

    async fn request_line(
        &self,
        state: &Arc<LineTransportState>,
        method: &str,
        params: Option<Value>,
    ) -> Result<Value> {
        let id = state.next_id.fetch_add(1, Ordering::Relaxed);
        let request = RpcRequest {
            jsonrpc: "2.0",
            id,
            method,
            params,
        };

        let serialized = serde_json::to_string(&request)?;
        let (tx, mut rx) = oneshot::channel();

        {
            let mut pending = state.pending.lock().await;
            pending.insert(id, tx);
        }

        {
            let mut writer = state.writer.lock().await;
            writer.write_all(serialized.as_bytes()).await?;
            writer.write_all(b"\n").await?;
            writer.flush().await?;
        }

        tokio::select! {
            result = &mut rx => match result {
                Ok(result) => result,
                Err(_) => Err(McpError::Disconnected(state.server.clone())),
            },
            _ = tokio::time::sleep(REQUEST_TIMEOUT) => {
                state.pending.lock().await.remove(&id);
                Err(McpError::Timeout {
                    server: state.server.clone(),
                    method: method.to_string(),
                    timeout_secs: REQUEST_TIMEOUT.as_secs(),
                })
            }
        }
    }

    async fn notify_line(
        &self,
        state: &Arc<LineTransportState>,
        method: &str,
        params: Option<Value>,
    ) -> Result<()> {
        let notification = RpcNotification {
            jsonrpc: "2.0",
            method,
            params,
        };
        let serialized = serde_json::to_string(&notification)?;
        let mut writer = state.writer.lock().await;
        writer.write_all(serialized.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
        Ok(())
    }

    async fn request_streamable_http(
        &self,
        state: &Arc<StreamableHttpTransportState>,
        method: &str,
        params: Option<Value>,
    ) -> Result<Value> {
        let id = state.next_id.fetch_add(1, Ordering::Relaxed);
        let request = RpcRequest {
            jsonrpc: "2.0",
            id,
            method,
            params,
        };
        let response = state
            .client
            .post(&state.url)
            .headers(state.headers.clone())
            .header(ACCEPT, "application/json, text/event-stream")
            .header(CONTENT_TYPE, "application/json")
            .json(&request)
            .send()
            .await?;
        let status = response.status();
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_string();
        let body = response.text().await?;
        if !status.is_success() {
            return Err(McpError::InvalidConfig(format!(
                "server '{}' HTTP request failed with status {}: {}",
                state.server, status, body
            )));
        }

        if content_type.contains("text/event-stream") {
            for event in parse_sse_events(&body) {
                if let Ok(resp) = serde_json::from_str::<RpcResponse>(&event.data)
                    && resp.id == id
                {
                    return rpc_response_result(&state.server, method, resp);
                }
            }
            return Err(McpError::Protocol {
                code: -32000,
                message: format!(
                    "server '{}' streamable HTTP response did not include request id {}",
                    state.server, id
                ),
            });
        }

        let resp: RpcResponse = serde_json::from_str(&body)?;
        if resp.id != id {
            return Err(McpError::Protocol {
                code: -32000,
                message: format!(
                    "server '{}' returned mismatched response id {} for request {}",
                    state.server, resp.id, id
                ),
            });
        }
        rpc_response_result(&state.server, method, resp)
    }

    async fn notify_streamable_http(
        &self,
        state: &Arc<StreamableHttpTransportState>,
        method: &str,
        params: Option<Value>,
    ) -> Result<()> {
        let notification = RpcNotification {
            jsonrpc: "2.0",
            method,
            params,
        };
        let response = state
            .client
            .post(&state.url)
            .headers(state.headers.clone())
            .header(ACCEPT, "application/json, text/event-stream")
            .header(CONTENT_TYPE, "application/json")
            .json(&notification)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(McpError::InvalidConfig(format!(
                "server '{}' HTTP notification '{}' failed with status {}",
                state.server,
                method,
                response.status()
            )));
        }
        Ok(())
    }

    async fn request_legacy_sse(
        &self,
        state: &Arc<LegacySseTransportState>,
        method: &str,
        params: Option<Value>,
    ) -> Result<Value> {
        let id = state.next_id.fetch_add(1, Ordering::Relaxed);
        let request = RpcRequest {
            jsonrpc: "2.0",
            id,
            method,
            params,
        };
        let (tx, mut rx) = oneshot::channel();
        {
            let mut pending = state.pending.lock().await;
            pending.insert(id, tx);
        }

        let post_url = state.post_url.lock().await.clone();
        if post_url.trim().is_empty() {
            return Err(McpError::InvalidConfig(format!(
                "server '{}' legacy SSE transport did not discover a POST endpoint",
                state.server
            )));
        }

        let response = state
            .client
            .post(post_url)
            .headers(state.headers.clone())
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .json(&request)
            .send()
            .await;
        match response {
            Ok(response) if response.status().is_success() => {}
            Ok(response) => {
                state.pending.lock().await.remove(&id);
                return Err(McpError::InvalidConfig(format!(
                    "server '{}' SSE POST failed with HTTP {}",
                    state.server,
                    response.status()
                )));
            }
            Err(error) => {
                state.pending.lock().await.remove(&id);
                return Err(McpError::Http(error));
            }
        }

        tokio::select! {
            result = &mut rx => match result {
                Ok(result) => result,
                Err(_) => Err(McpError::Disconnected(state.server.clone())),
            },
            _ = tokio::time::sleep(REQUEST_TIMEOUT) => {
                state.pending.lock().await.remove(&id);
                Err(McpError::Timeout {
                    server: state.server.clone(),
                    method: method.to_string(),
                    timeout_secs: REQUEST_TIMEOUT.as_secs(),
                })
            }
        }
    }

    async fn notify_legacy_sse(
        &self,
        state: &Arc<LegacySseTransportState>,
        method: &str,
        params: Option<Value>,
    ) -> Result<()> {
        let post_url = state.post_url.lock().await.clone();
        if post_url.trim().is_empty() {
            return Err(McpError::InvalidConfig(format!(
                "server '{}' legacy SSE transport did not discover a POST endpoint",
                state.server
            )));
        }
        let notification = RpcNotification {
            jsonrpc: "2.0",
            method,
            params,
        };
        let response = state
            .client
            .post(post_url)
            .headers(state.headers.clone())
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .json(&notification)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(McpError::InvalidConfig(format!(
                "server '{}' SSE notification '{}' failed with HTTP {}",
                state.server,
                method,
                response.status()
            )));
        }
        Ok(())
    }

    async fn drain_pending_disconnected(state: &Arc<LineTransportState>) {
        let pending = {
            let mut map = state.pending.lock().await;
            std::mem::take(&mut *map)
        };
        for (_, tx) in pending {
            let _ = tx.send(Err(McpError::Disconnected(state.server.clone())));
        }
    }

    #[cfg(test)]
    async fn pending_len_for_test(&self) -> usize {
        match &*self.kind {
            TransportKind::Line(state) => state.pending.lock().await.len(),
            TransportKind::LegacySse(state) => state.pending.lock().await.len(),
            TransportKind::StreamableHttp(_) => 0,
        }
    }
}

/// Builds remote HTTP headers from non-secret config and environment-backed auth.
pub fn build_remote_headers(
    headers: &HashMap<String, String>,
    auth_token_env: Option<&str>,
    authorization_env: Option<&str>,
) -> Result<HeaderMap> {
    let mut map = HeaderMap::new();
    for (name, value) in headers {
        let name = HeaderName::from_bytes(name.as_bytes()).map_err(|source| {
            McpError::InvalidConfig(format!("invalid MCP HTTP header name '{name}': {source}"))
        })?;
        let value = HeaderValue::from_str(value).map_err(|source| {
            McpError::InvalidConfig(format!(
                "invalid MCP HTTP header value for '{name}': {source}"
            ))
        })?;
        map.insert(name, value);
    }

    if let Some(env_name) = auth_token_env.filter(|name| !name.trim().is_empty()) {
        let token = std::env::var(env_name).map_err(|_| {
            McpError::InvalidConfig(format!(
                "MCP auth_token_env '{env_name}' is set but the environment variable is missing"
            ))
        })?;
        let value = HeaderValue::from_str(&format!("Bearer {token}")).map_err(|source| {
            McpError::InvalidConfig(format!("invalid bearer token from '{env_name}': {source}"))
        })?;
        map.insert(AUTHORIZATION, value);
    }

    if let Some(env_name) = authorization_env.filter(|name| !name.trim().is_empty()) {
        let authorization = std::env::var(env_name).map_err(|_| {
            McpError::InvalidConfig(format!(
                "MCP authorization_env '{env_name}' is set but the environment variable is missing"
            ))
        })?;
        let value = HeaderValue::from_str(&authorization).map_err(|source| {
            McpError::InvalidConfig(format!(
                "invalid authorization header from '{env_name}': {source}"
            ))
        })?;
        map.insert(AUTHORIZATION, value);
    }

    Ok(map)
}

#[derive(Debug, Clone)]
struct SseEvent {
    event: Option<String>,
    data: String,
}

fn parse_sse_events(input: &str) -> Vec<SseEvent> {
    input
        .split("\n\n")
        .filter_map(|chunk| {
            let mut event = None;
            let mut data = Vec::new();
            for raw_line in chunk.lines() {
                let line = raw_line.trim_end_matches('\r');
                if let Some(value) = line.strip_prefix("event:") {
                    event = Some(value.trim().to_string());
                } else if let Some(value) = line.strip_prefix("data:") {
                    data.push(value.trim_start().to_string());
                }
            }
            if data.is_empty() {
                None
            } else {
                Some(SseEvent {
                    event,
                    data: data.join("\n"),
                })
            }
        })
        .collect()
}

async fn read_legacy_sse_stream<S>(
    state: Arc<LegacySseTransportState>,
    mut stream: S,
    mut endpoint_tx: Option<oneshot::Sender<String>>,
    sse_url: String,
) where
    S: futures_util::Stream<Item = std::result::Result<bytes::Bytes, reqwest::Error>>
        + Unpin
        + Send
        + 'static,
{
    let mut buffer = String::new();
    while let Some(chunk) = stream.next().await {
        let chunk = match chunk {
            Ok(chunk) => chunk,
            Err(error) => {
                tracing::warn!(server = %state.server, %error, "MCP SSE stream read failed");
                drain_legacy_pending(&state).await;
                return;
            }
        };
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(boundary) = buffer.find("\n\n") {
            let event_text = buffer[..boundary + 2].to_string();
            buffer.replace_range(..boundary + 2, "");
            for event in parse_sse_events(&event_text) {
                if event.event.as_deref() == Some("endpoint") {
                    if endpoint_tx.is_some()
                        && let Some(resolved) = resolve_sse_post_url(&sse_url, &event.data)
                    {
                        *state.post_url.lock().await = resolved.clone();
                        if let Some(tx) = endpoint_tx.take() {
                            let _ = tx.send(resolved);
                        }
                    }
                    continue;
                }
                if let Ok(resp) = serde_json::from_str::<RpcResponse>(&event.data) {
                    let tx = {
                        let mut pending = state.pending.lock().await;
                        pending.remove(&resp.id)
                    };
                    if let Some(tx) = tx {
                        let _ = tx.send(rpc_response_result(&state.server, "unknown", resp));
                    }
                }
            }
        }
    }
    drain_legacy_pending(&state).await;
}

async fn drain_legacy_pending(state: &Arc<LegacySseTransportState>) {
    let pending = {
        let mut map = state.pending.lock().await;
        std::mem::take(&mut *map)
    };
    for (_, tx) in pending {
        let _ = tx.send(Err(McpError::Disconnected(state.server.clone())));
    }
}

fn rpc_response_result(server: &str, method: &str, resp: RpcResponse) -> Result<Value> {
    if let Some(err) = resp.error {
        Err(McpError::Rpc {
            server: server.to_string(),
            method: method.to_string(),
            message: format!(
                "code={} message={} data={:?}",
                err.code, err.message, err.data
            ),
        })
    } else {
        Ok(resp.result.unwrap_or(Value::Null))
    }
}

fn validate_http_url(url: &str) -> Result<()> {
    let parsed = reqwest::Url::parse(url).map_err(|source| {
        McpError::InvalidConfig(format!("invalid MCP HTTP URL '{url}': {source}"))
    })?;
    match parsed.scheme() {
        "http" | "https" => Ok(()),
        scheme => Err(McpError::InvalidConfig(format!(
            "invalid MCP HTTP URL '{url}': unsupported scheme '{scheme}'"
        ))),
    }
}

fn resolve_sse_post_url(sse_url: &str, endpoint: &str) -> Option<String> {
    if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
        return Some(endpoint.to_string());
    }
    reqwest::Url::parse(sse_url)
        .ok()
        .and_then(|base| base.join(endpoint).ok())
        .map(|url| url.to_string())
}

/// Spawns a stdio child process and wires it to an MCP transport.
pub async fn spawn_stdio_transport(
    server: &str,
    command: &str,
    args: &[String],
    env: &HashMap<String, String>,
    cwd: Option<&Path>,
) -> Result<(McpTransport, Child)> {
    let mut cmd = Command::new(command);
    cmd.args(args);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    for (key, value) in env {
        cmd.env(key, value);
    }
    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);

    let mut child = cmd.spawn().map_err(|source| McpError::Spawn {
        server: server.to_string(),
        source,
    })?;

    let child_stdout = child.stdout.take().ok_or_else(|| {
        McpError::InvalidConfig(format!("MCP server '{server}' missing stdout pipe"))
    })?;
    let child_stdin = child.stdin.take().ok_or_else(|| {
        McpError::InvalidConfig(format!("MCP server '{server}' missing stdin pipe"))
    })?;

    let transport = McpTransport::from_io(server.to_string(), child_stdout, child_stdin);
    Ok((transport, child))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt;

    #[tokio::test]
    async fn silent_server_times_out_and_removes_pending_request() {
        let (client, _server) = tokio::io::duplex(256);
        let (reader, writer) = tokio::io::split(client);
        let transport = McpTransport::from_io("silent".to_string(), reader, writer);

        let error = transport
            .request("tools/list", Some(serde_json::json!({})))
            .await
            .expect_err("silent server should time out");

        assert!(matches!(error, McpError::Timeout { .. }));
        assert_eq!(transport.pending_len_for_test().await, 0);
    }

    #[tokio::test]
    async fn streamable_http_json_response_roundtrips() {
        let server = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind local test server");
        let url = format!("http://{}/mcp", server.local_addr().expect("local addr"));
        tokio::spawn(async move {
            let (mut socket, _) = server.accept().await.expect("accept request");
            let request = read_http_request(&mut socket).await;
            let id = request_body_id(&request);
            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": { "ok": true }
            });
            write_http_response(&mut socket, "application/json", &response.to_string()).await;
        });

        let transport = McpTransport::streamable_http("remote".to_string(), url, HeaderMap::new())
            .expect("create transport");
        let result = transport
            .request("tools/list", Some(serde_json::json!({})))
            .await
            .expect("request should succeed");

        assert_eq!(result["ok"], true);
    }

    #[tokio::test]
    async fn streamable_http_sse_response_roundtrips() {
        let server = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind local test server");
        let url = format!("http://{}/mcp", server.local_addr().expect("local addr"));
        tokio::spawn(async move {
            let (mut socket, _) = server.accept().await.expect("accept request");
            let request = read_http_request(&mut socket).await;
            let id = request_body_id(&request);
            let event = format!(
                "event: message\ndata: {}\n\n",
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": { "from_sse": true }
                })
            );
            write_http_response(&mut socket, "text/event-stream", &event).await;
        });

        let transport = McpTransport::streamable_http("remote".to_string(), url, HeaderMap::new())
            .expect("create transport");
        let result = transport
            .request("tools/list", Some(serde_json::json!({})))
            .await
            .expect("request should succeed");

        assert_eq!(result["from_sse"], true);
    }

    #[tokio::test]
    async fn legacy_sse_endpoint_and_post_roundtrip() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind local test server");
        let base = format!("http://{}", listener.local_addr().expect("local addr"));
        let sse_url = format!("{base}/sse");
        let (response_tx, mut response_rx) = tokio::sync::mpsc::channel::<String>(1);

        tokio::spawn(async move {
            let (mut sse_socket, _) = listener.accept().await.expect("accept sse");
            let _ = read_http_request(&mut sse_socket).await;
            let headers = "HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\n\r\n";
            sse_socket
                .write_all(headers.as_bytes())
                .await
                .expect("write sse headers");
            sse_socket
                .write_all(b"event: endpoint\ndata: /message\n\n")
                .await
                .expect("write endpoint event");

            let (mut post_socket, _) = listener.accept().await.expect("accept post");
            let request = read_http_request(&mut post_socket).await;
            let id = request_body_id(&request);
            write_http_response(&mut post_socket, "application/json", "{}").await;
            let event = format!(
                "event: message\ndata: {}\n\n",
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": { "legacy": true }
                })
            );
            response_tx.send(event).await.expect("queue sse response");
            if let Some(event) = response_rx.recv().await {
                sse_socket
                    .write_all(event.as_bytes())
                    .await
                    .expect("write response event");
            }
        });

        let transport =
            McpTransport::connect_legacy_sse("legacy".to_string(), sse_url, None, HeaderMap::new())
                .await
                .expect("connect legacy sse");
        let result = transport
            .request("tools/list", Some(serde_json::json!({})))
            .await
            .expect("request should succeed");

        assert_eq!(result["legacy"], true);
    }

    async fn read_http_request(socket: &mut tokio::net::TcpStream) -> String {
        let mut bytes = Vec::new();
        let header_end = loop {
            let mut byte = [0_u8; 1];
            socket.read_exact(&mut byte).await.expect("read request");
            bytes.push(byte[0]);
            if bytes.ends_with(b"\r\n\r\n") {
                break bytes.len();
            }
        };
        let header_text = String::from_utf8_lossy(&bytes).to_string();
        let content_length = header_text
            .lines()
            .find_map(|line| {
                line.strip_prefix("content-length:")
                    .or_else(|| line.strip_prefix("Content-Length:"))
            })
            .and_then(|value| value.trim().parse::<usize>().ok())
            .unwrap_or(0);
        let mut body = vec![0_u8; content_length];
        socket.read_exact(&mut body).await.expect("read body");
        bytes.extend_from_slice(&body);
        let mut request = String::from_utf8_lossy(&bytes[..header_end]).to_string();
        request.push_str(&String::from_utf8_lossy(&body));
        request
    }

    fn request_body_id(request: &str) -> u64 {
        let body_start = request.find("\r\n\r\n").map(|idx| idx + 4).unwrap_or(0);
        serde_json::from_str::<Value>(&request[body_start..])
            .expect("parse JSON request")
            .get("id")
            .and_then(Value::as_u64)
            .expect("request id")
    }

    async fn write_http_response(
        socket: &mut tokio::net::TcpStream,
        content_type: &str,
        body: &str,
    ) {
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: {content_type}\r\ncontent-length: {}\r\n\r\n{body}",
            body.len()
        );
        socket
            .write_all(response.as_bytes())
            .await
            .expect("write response");
    }
}
