//! JSON-RPC server over stdio transport.
//!
//! The current implementation handles requests sequentially (MVP behavior).

use std::sync::Arc;

use anyhow::Result;
use tokio::io::{AsyncRead, AsyncWrite, BufReader, stdin, stdout};

use crate::cancel::CancelRegistry;
use crate::error::RpcError;
use crate::framing::{read_line, write_json_line};
use crate::methods::{MethodContext, dispatch_method, response_id_or_null};
use crate::protocol::{JSON_RPC_VERSION, JsonRpcRequest, JsonRpcResponse};
use crate::runtime::Runtime;

/// JSON-RPC server.
pub struct RpcServer {
    ctx: MethodContext,
}

impl RpcServer {
    /// Creates a new RPC server bound to a [`Runtime`].
    #[must_use]
    pub fn new(agent: Arc<dyn Runtime>) -> Self {
        Self {
            ctx: MethodContext {
                agent,
                cancel_registry: Arc::new(CancelRegistry::new()),
            },
        }
    }

    /// Runs the server using process stdio.
    pub async fn run_stdio(&self) -> Result<()> {
        let input = stdin();
        let output = stdout();
        self.run(input, output).await
    }

    /// Runs the server over arbitrary async input/output streams.
    pub async fn run<R, W>(&self, input: R, mut output: W) -> Result<()>
    where
        R: AsyncRead + Unpin,
        W: AsyncWrite + Unpin,
    {
        let mut reader = BufReader::new(input);

        while let Some(line) = read_line(&mut reader).await? {
            if line.trim().is_empty() {
                continue;
            }

            let parsed = parse_request_line(&line).and_then(validate_request);

            let (id, response) = match parsed {
                Ok(request) => {
                    if request.id.is_none() {
                        continue;
                    }
                    let id = response_id_or_null(request.id);
                    let result = dispatch_method(&self.ctx, &request.method, request.params).await;
                    match result {
                        Ok(result) => {
                            for notification in result.notifications {
                                write_json_line(&mut output, &notification).await?;
                            }
                            (id.clone(), JsonRpcResponse::success(id, result.result))
                        }
                        Err(error) => {
                            let rpc_error = error.to_json_rpc_error();
                            (id.clone(), JsonRpcResponse::failure(id, rpc_error))
                        }
                    }
                }
                Err(error) => {
                    let id = crate::protocol::JsonRpcId::Null;
                    let rpc_error = error.to_json_rpc_error();
                    (id.clone(), JsonRpcResponse::failure(id, rpc_error))
                }
            };

            tracing::debug!(?id, "rpc response");
            write_json_line(&mut output, &response).await?;
        }

        Ok(())
    }
}

fn parse_request_line(line: &str) -> Result<JsonRpcRequest, RpcError> {
    let value: serde_json::Value = serde_json::from_str(line).map_err(|_| RpcError::ParseError)?;
    serde_json::from_value(value).map_err(|e| RpcError::InvalidRequest(e.to_string()))
}

fn validate_request(request: JsonRpcRequest) -> Result<JsonRpcRequest, RpcError> {
    if request.jsonrpc != JSON_RPC_VERSION {
        return Err(RpcError::InvalidRequest(
            "jsonrpc field must be \"2.0\"".to_string(),
        ));
    }
    if request.method.trim().is_empty() {
        return Err(RpcError::InvalidRequest(
            "method must be non-empty".to_string(),
        ));
    }
    Ok(request)
}
