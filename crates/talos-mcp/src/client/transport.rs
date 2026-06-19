//! MCP transport primitives.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, oneshot};

use crate::error::{McpError, Result};

#[cfg(not(test))]
const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
#[cfg(test)]
const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(50);

/// Minimal JSON-RPC request envelope used by MCP.
#[derive(Debug, Serialize)]
struct RpcRequest<'a> {
    jsonrpc: &'static str,
    id: u64,
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

struct TransportState {
    writer: Mutex<Box<dyn AsyncWrite + Send + Unpin>>,
    pending: Mutex<HashMap<u64, oneshot::Sender<Result<Value>>>>,
    next_id: AtomicU64,
    server: String,
}

/// A line-delimited JSON-RPC transport used by the MCP client.
#[derive(Clone)]
pub struct McpTransport {
    state: Arc<TransportState>,
}

impl McpTransport {
    /// Creates a transport from arbitrary async reader/writer halves.
    pub fn from_io<R, W>(server: String, reader: R, writer: W) -> Self
    where
        R: AsyncRead + Send + Unpin + 'static,
        W: AsyncWrite + Send + Unpin + 'static,
    {
        let state = Arc::new(TransportState {
            writer: Mutex::new(Box::new(writer)),
            pending: Mutex::new(HashMap::new()),
            next_id: AtomicU64::new(1),
            server,
        });

        let transport = Self {
            state: state.clone(),
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

    /// Sends a JSON-RPC request and waits for a matching response.
    pub async fn request(&self, method: &str, params: Option<Value>) -> Result<Value> {
        let id = self.state.next_id.fetch_add(1, Ordering::Relaxed);
        let request = RpcRequest {
            jsonrpc: "2.0",
            id,
            method,
            params,
        };

        let serialized = serde_json::to_string(&request)?;
        let (tx, mut rx) = oneshot::channel();

        {
            let mut pending = self.state.pending.lock().await;
            pending.insert(id, tx);
        }

        {
            let mut writer = self.state.writer.lock().await;
            writer.write_all(serialized.as_bytes()).await?;
            writer.write_all(b"\n").await?;
            writer.flush().await?;
        }

        tokio::select! {
            result = &mut rx => match result {
                Ok(result) => result,
                Err(_) => Err(McpError::Disconnected(self.state.server.clone())),
            },
            _ = tokio::time::sleep(REQUEST_TIMEOUT) => {
                self.state.pending.lock().await.remove(&id);
                Err(McpError::Timeout {
                    server: self.state.server.clone(),
                    method: method.to_string(),
                    timeout_secs: REQUEST_TIMEOUT.as_secs(),
                })
            }
        }
    }

    async fn drain_pending_disconnected(state: &Arc<TransportState>) {
        let pending = {
            let mut map = state.pending.lock().await;
            std::mem::take(&mut *map)
        };
        for (_, tx) in pending {
            let _ = tx.send(Err(McpError::Disconnected(state.server.clone())));
        }
    }
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
        assert!(transport.state.pending.lock().await.is_empty());
    }
}
