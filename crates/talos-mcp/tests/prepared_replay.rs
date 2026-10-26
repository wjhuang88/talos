use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;

use async_trait::async_trait;
use rmcp::ServiceExt;
use serde_json::{Value, json};
use talos_core::tool::{
    AgentTool, PreparedNormalToolInvocation, ToolExecutionOutput, ToolNature, ToolPermissionFacet,
    ToolProvenance, ToolRegistry, ToolResult,
};
use talos_mcp::server::{McpPermissionGate, TalosMcpHandler};
use talos_permission::{AdmittedInvocation, PermissionEngine, PermissionRequest};
use talos_plugin::HookRegistry;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Notify;

// Turn a peer disconnect into an actual transport read error to exercise
// rmcp's error exit independently from its clean EOF exit.
struct DisconnectTransport {
    inner: tokio::io::DuplexStream,
    error_on_eof: bool,
}
impl tokio::io::AsyncRead for DisconnectTransport {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buffer: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let before = buffer.filled().len();
        let has_capacity = buffer.remaining() > 0;
        let result = std::pin::Pin::new(&mut self.inner).poll_read(cx, buffer);
        if self.error_on_eof
            && has_capacity
            && buffer.filled().len() == before
            && matches!(result, std::task::Poll::Ready(Ok(())))
        {
            return std::task::Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionReset,
                "fixture transport failure",
            )));
        }
        result
    }
}
impl tokio::io::AsyncWrite for DisconnectTransport {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        bytes: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        std::pin::Pin::new(&mut self.inner).poll_write(cx, bytes)
    }
    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.inner).poll_flush(cx)
    }
    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

struct PreparedCounter {
    expose_identity: bool,
    stages: usize,
    reuse_identity: bool,
    count: Arc<AtomicUsize>,
    started: Option<Arc<Notify>>,
    release: Option<Arc<Notify>>,
    dropped: Option<Arc<Notify>>,
    cleanup: Option<Arc<CleanupProbe>>,
}
struct CleanupProbe {
    dropped: Arc<Notify>,
    observed: Notify,
    release: Notify,
    completed: Notify,
}
#[async_trait]
impl talos_core::tool::ToolCleanupReceipt for CleanupProbe {
    async fn wait(&self, _: Duration) -> Result<(), talos_core::tool::ToolCleanupError> {
        self.dropped.notified().await;
        self.observed.notify_one();
        self.release.notified().await;
        self.completed.notify_one();
        Ok(())
    }
}

struct DropSignal(Option<Arc<Notify>>);
impl Drop for DropSignal {
    fn drop(&mut self) {
        if let Some(signal) = &self.0 {
            signal.notify_one();
        }
    }
}
struct Operation {
    identity: talos_core::tool::PreparedPlanIdentity,
    expose_identity: bool,
    stage: usize,
    stages: usize,
    reuse_identity: bool,
    provenance: ToolProvenance,
    count: Arc<AtomicUsize>,
    input: Value,
    facets: Vec<ToolPermissionFacet>,
    started: Option<Arc<Notify>>,
    release: Option<Arc<Notify>>,
    dropped: Option<Arc<Notify>>,
    cleanup: Option<Arc<CleanupProbe>>,
}

#[async_trait]
impl PreparedNormalToolInvocation for Operation {
    fn cleanup_receipt(&self) -> Option<Arc<dyn talos_core::tool::ToolCleanupReceipt>> {
        self.cleanup
            .clone()
            .map(|receipt| receipt as Arc<dyn talos_core::tool::ToolCleanupReceipt>)
    }

    fn plan_identity(&self) -> Option<&talos_core::tool::PreparedPlanIdentity> {
        self.expose_identity.then_some(&self.identity)
    }
    fn permission_profile(&self) -> &[ToolPermissionFacet] {
        &self.facets
    }
    async fn execute_stage(
        self: Box<Self>,
        admission: Box<dyn std::any::Any + Send>,
    ) -> talos_core::tool::PreparedNormalStage {
        use talos_core::tool::PreparedNormalStage;
        if self.stage >= self.stages {
            return PreparedNormalStage::Complete(self.execute(admission).await);
        }
        let proof = admission
            .downcast::<AdmittedInvocation>()
            .expect("real discovery admission");
        let request = PermissionRequest::new(
            "prepared_counter",
            self.provenance.clone(),
            &self.facets,
            &self.input,
        );
        proof
            .consume_for_prepared(&request, &self.identity)
            .expect("exact discovery request admitted");
        self.count.fetch_add(1, Ordering::SeqCst);
        let mut next = *self;
        next.stage += 1;
        if !next.reuse_identity {
            next.identity = talos_core::tool::PreparedPlanIdentity::new();
        }
        PreparedNormalStage::Continue(Box::new(next))
    }

    async fn execute(
        self: Box<Self>,
        admission: Box<dyn std::any::Any + Send>,
    ) -> ToolExecutionOutput {
        let _drop_signal = DropSignal(self.dropped.clone());
        let proof = admission
            .downcast::<AdmittedInvocation>()
            .expect("real admission");
        let request = PermissionRequest::new(
            "prepared_counter",
            self.provenance.clone(),
            &self.facets,
            &self.input,
        );
        proof
            .consume_for_prepared(&request, &self.identity)
            .expect("exact request admitted");
        self.count.fetch_add(1, Ordering::SeqCst);
        if let Some(started) = &self.started {
            started.notify_one();
        }
        if let Some(release) = &self.release {
            release.notified().await;
        }
        // An error may follow a side effect; it must not reopen the request ID.
        ToolExecutionOutput::error("fixture indeterminate result")
    }
}

#[async_trait]
impl AgentTool for PreparedCounter {
    fn name(&self) -> &str {
        "prepared_counter"
    }
    fn description(&self) -> &str {
        "prepared admission fixture"
    }
    fn parameters(&self) -> Value {
        json!({"type":"object"})
    }
    async fn execute(&self, _: Value) -> ToolResult {
        panic!("legacy route")
    }
    fn prepare_normalized_with_provenance(
        &self,
        input: &Value,
        provenance: ToolProvenance,
    ) -> Result<Option<Box<dyn PreparedNormalToolInvocation>>, String> {
        Ok(Some(Box::new(Operation {
            identity: talos_core::tool::PreparedPlanIdentity::new(),
            expose_identity: self.expose_identity,
            stage: 1,
            stages: self.stages,
            reuse_identity: self.reuse_identity,
            provenance,
            count: self.count.clone(),
            input: input.clone(),
            facets: vec![ToolPermissionFacet::new(ToolNature::Read).with_description(
                "x".repeat(input["profile_bytes"].as_u64().unwrap_or(0) as usize),
            )],
            started: self.started.clone(),
            release: self.release.clone(),
            dropped: self.dropped.clone(),
            cleanup: self.cleanup.clone(),
        })))
    }
}

#[tokio::test]
async fn transport_duplicate_and_changed_arguments_do_not_reexecute_after_error() {
    for expose_identity in [true, false] {
        tokio::time::timeout(Duration::from_secs(10), async {
        let count = Arc::new(AtomicUsize::new(0));
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(PreparedCounter {
            expose_identity,
            stages: 1,
            reuse_identity: false,
            count: count.clone(),
            started: None,
            release: None,
            dropped: None,
            cleanup: None,
        }));
        let handler = TalosMcpHandler::new(Arc::new(registry), Arc::new(McpPermissionGate::new(Arc::new(PermissionEngine::new()), Arc::new(HookRegistry::new()))));
        let (client, server) = tokio::io::duplex(65536);
        let server_task = tokio::spawn(async move {
            let service = handler.serve(server).await.expect("server handshake");
            service.waiting().await.expect("server shutdown");
        });
        let (read, mut write) = tokio::io::split(client);
        let mut read = BufReader::new(read);
        let init = json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"replay-test","version":"1"}}});
        write.write_all(format!("{init}\n").as_bytes()).await.expect("init");
        let mut line = String::new();
        read.read_line(&mut line).await.expect("init response");
        assert!(serde_json::from_str::<Value>(&line).expect("json").get("result").is_some(), "{line}");
        write.write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n").await.expect("initialized");
        for (id, value, expected_count, duplicate) in [(1, 1, 1, false), (1, 1, 1, true), (1, 2, 1, true), (2, 1, 2, false)] {
            let call = json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":"prepared_counter","arguments":{"value":value}}});
            write.write_all(format!("{call}\n").as_bytes()).await.expect("request");
            line.clear();
            read.read_line(&mut line).await.expect("response");
            let response: Value = serde_json::from_str(&line).expect("response json");
            if !expose_identity {
                assert!(response["error"]["message"].as_str().expect("error").contains("prepared plan identity required"), "{response}");
            } else if duplicate {
                assert!(response["error"]["message"].as_str().expect("error").contains("already used"), "{response}");
            } else {
                assert_eq!(response["result"]["isError"], true, "{response}");
            }
            assert_eq!(count.load(Ordering::SeqCst), if expose_identity { expected_count } else { 0 });
        }
        drop(read);
        drop(write);
        server_task.await.expect("server task");
    }).await.expect("transport test deadline");
    }
}

#[tokio::test]
async fn transport_prepared_continuations_are_bounded_and_replay_safe() {
    for (stages, reuse_identity, profile_bytes, expected_effects, expected_limit) in [
        (2, false, 0, 2, false),
        (8, false, 0, 8, false),
        (9, false, 0, 8, true),
        (2, true, 0, 1, true),
        (1, false, 16 * 1024 * 1024 + 1, 0, true),
        (3, false, 8 * 1024 * 1024, 2, true),
    ] {
        tokio::time::timeout(Duration::from_secs(10), async {
            let count = Arc::new(AtomicUsize::new(0));
            let mut registry = ToolRegistry::new();
            registry.register(Arc::new(PreparedCounter {
                expose_identity: true,
                stages,
                reuse_identity,
                count: count.clone(),
                started: None,
                release: None,
                dropped: None,
                cleanup: None,
            }));
            let handler = TalosMcpHandler::new(
                Arc::new(registry),
                Arc::new(McpPermissionGate::new(
                    Arc::new(PermissionEngine::new()),
                    Arc::new(HookRegistry::new()),
                )),
            );
            let (client, server) = tokio::io::duplex(65536);
            let server_task = tokio::spawn(async move {
                let service = handler.serve(server).await.expect("server handshake");
                service.waiting().await.expect("server shutdown");
            });
            let (read, mut write) = tokio::io::split(client);
            let mut read = BufReader::new(read);
            let init = json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"stage-test","version":"1"}}});
            write.write_all(format!("{init}\n").as_bytes()).await.expect("init");
            let mut line = String::new();
            read.read_line(&mut line).await.expect("init response");
            assert!(serde_json::from_str::<Value>(&line).expect("init JSON").get("result").is_some());
            write.write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n").await.expect("initialized");
            let call = json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"prepared_counter","arguments":{"profile_bytes":profile_bytes}}});
            write.write_all(format!("{call}\n").as_bytes()).await.expect("call");
            line.clear();
            read.read_line(&mut line).await.expect("call response");
            let response: Value = serde_json::from_str(&line).expect("call JSON");
            if expected_limit {
                assert!(response["error"]["message"].as_str().expect("limit error").contains("prepared stage limit or reused identity"), "{response}");
            } else {
                assert_eq!(response["result"]["isError"], true, "{response}");
            }
            assert_eq!(count.load(Ordering::SeqCst), expected_effects);
            write.write_all(format!("{call}\n").as_bytes()).await.expect("replay");
            line.clear();
            read.read_line(&mut line).await.expect("replay response");
            let replay: Value = serde_json::from_str(&line).expect("replay JSON");
            assert!(replay["error"]["message"].as_str().expect("replay error").contains("already used"), "{replay}");
            assert_eq!(count.load(Ordering::SeqCst), expected_effects);
            drop(read);
            drop(write);
            server_task.await.expect("server task");
        }).await.expect("stage test deadline");
    }
}

#[tokio::test]
async fn admitted_execution_outlives_permission_deadline() {
    tokio::time::timeout(Duration::from_secs(10), async {
        let count = Arc::new(AtomicUsize::new(0));
        let started = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(PreparedCounter {
            expose_identity: true,
            stages: 1,
            reuse_identity: false,
            count: count.clone(),
            started: Some(started.clone()),
            release: Some(release.clone()),
            dropped: None,
            cleanup: None,
        }));
        let handler = TalosMcpHandler::new(
            Arc::new(registry),
            Arc::new(
                McpPermissionGate::new(
                    Arc::new(PermissionEngine::new()),
                    Arc::new(HookRegistry::new()),
                )
                .with_deadline(Duration::from_millis(300)),
            ),
        );
        let (client, server) = tokio::io::duplex(65536);
        let server_task = tokio::spawn(async move {
            let service = handler.serve(server).await.expect("server handshake");
            service.waiting().await.expect("server shutdown");
        });
        let (read, mut write) = tokio::io::split(client);
        let mut read = BufReader::new(read);
        let init = json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"deadline-test","version":"1"}}});
        write.write_all(format!("{init}\n").as_bytes()).await.expect("init");
        let mut line = String::new();
        read.read_line(&mut line).await.expect("init response");
        assert!(serde_json::from_str::<Value>(&line).expect("init JSON").get("result").is_some());
        write.write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n").await.expect("initialized");
        let call = json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"prepared_counter","arguments":{}}});
        let started_wait = started.notified();
        write.write_all(format!("{call}\n").as_bytes()).await.expect("call");
        started_wait.await;
        tokio::time::sleep(Duration::from_millis(400)).await;
        assert_eq!(count.load(Ordering::SeqCst), 1);
        release.notify_one();
        line.clear();
        read.read_line(&mut line).await.expect("call response");
        let response: Value = serde_json::from_str(&line).expect("call JSON");
        assert_eq!(response["result"]["isError"], true, "{response}");
        drop(read);
        drop(write);
        server_task.await.expect("server task");
    }).await.expect("transport deadline");
}

#[tokio::test]
async fn cancelled_prepared_request_keeps_its_id_reserved() {
    tokio::time::timeout(Duration::from_secs(10), async {
        let count = Arc::new(AtomicUsize::new(0));
        let started = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());
        let dropped = Arc::new(Notify::new());
        let cleanup = Arc::new(CleanupProbe {
            dropped: dropped.clone(), observed: Notify::new(),
            release: Notify::new(), completed: Notify::new(),
        });
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(PreparedCounter {
            expose_identity: true,
            stages: 1,
            reuse_identity: false,
            count: count.clone(),
            started: Some(started.clone()),
            release: Some(release),
            dropped: Some(dropped.clone()),
            cleanup: Some(cleanup.clone()),
        }));
        let handler = TalosMcpHandler::new(
            Arc::new(registry),
            Arc::new(McpPermissionGate::new(
                Arc::new(PermissionEngine::new()),
                Arc::new(HookRegistry::new()),
            )),
        );
        let (client, server) = tokio::io::duplex(65536);
        let server_task = tokio::spawn(async move {
            let service = handler.serve(server).await.expect("server handshake");
            service.waiting().await.expect("server shutdown");
        });
        let (read, mut write) = tokio::io::split(client);
        let mut read = BufReader::new(read);
        let init = json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"cancel-test","version":"1"}}});
        write.write_all(format!("{init}\n").as_bytes()).await.expect("init");
        let mut line = String::new();
        read.read_line(&mut line).await.expect("init response");
        assert!(serde_json::from_str::<Value>(&line).expect("json").get("result").is_some(), "{line}");
        write.write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n").await.expect("initialized");
        let call = json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"prepared_counter","arguments":{"value":1}}});
        let started_wait = started.notified();
        write.write_all(format!("{call}\n").as_bytes()).await.expect("first request");
        started_wait.await;
        assert_eq!(count.load(Ordering::SeqCst), 1);
        let cancel = json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":1,"reason":"fixture cancellation"}});
        write.write_all(format!("{cancel}\n").as_bytes()).await.expect("cancel");
        cleanup.observed.notified().await;
        assert!(tokio::time::timeout(Duration::from_millis(20), cleanup.completed.notified()).await.is_err(), "cleanup must still be waiting");
        cleanup.release.notify_one();
        cleanup.completed.notified().await;
        let ping = json!({"jsonrpc":"2.0","id":2,"method":"ping"});
        write.write_all(format!("{ping}\n").as_bytes()).await.expect("ping");
        line.clear();
        read.read_line(&mut line).await.expect("ping response");
        let response: Value = serde_json::from_str(&line).expect("response json");
        assert_eq!(response["id"], 2, "{response}");
        assert!(response.get("result").is_some(), "{response}");
        write.write_all(format!("{call}\n").as_bytes()).await.expect("replay");
        line.clear();
        read.read_line(&mut line).await.expect("replay response");
        let response: Value = serde_json::from_str(&line).expect("response json");
        assert!(response["error"]["message"].as_str().expect("replay error").contains("already used"), "{response}");
        assert_eq!(count.load(Ordering::SeqCst), 1);
        drop(read);
        drop(write);
        server_task.await.expect("server task");
    }).await.expect("transport test deadline");
}

#[tokio::test]
async fn eof_cancels_active_invocation_before_connection_cleanup() {
    disconnect_cleanup(false).await;
}

#[tokio::test]
async fn transport_error_cancels_active_invocation_before_connection_cleanup() {
    disconnect_cleanup(true).await;
}

async fn disconnect_cleanup(error_on_eof: bool) {
    tokio::time::timeout(Duration::from_secs(15), async {
        let count = Arc::new(AtomicUsize::new(0));
        let started = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());
        let dropped = Arc::new(Notify::new());
        let cleanup = Arc::new(CleanupProbe {
            dropped: dropped.clone(), observed: Notify::new(),
            release: Notify::new(), completed: Notify::new(),
        });
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(PreparedCounter {
            expose_identity: true,
            stages: 1,
            reuse_identity: false,
            count: count.clone(),
            started: Some(started.clone()),
            release: Some(release),
            dropped: Some(dropped.clone()),
            cleanup: Some(cleanup.clone()),
        }));
        let handler = TalosMcpHandler::new(
            Arc::new(registry),
            Arc::new(McpPermissionGate::new(
                Arc::new(PermissionEngine::new()),
                Arc::new(HookRegistry::new()),
            )),
        );
        let handler = Arc::new(handler);
        let (client, server) = tokio::io::duplex(65536);
        let server = DisconnectTransport { inner: server, error_on_eof };
        let server_task = tokio::spawn(async move {
            let service = handler.clone().serve(server).await.expect("server handshake");
            let _transport_result = service.waiting().await;
            handler.shutdown(Duration::from_secs(5)).await.expect("connection cleanup");
        });
        let (read, mut write) = tokio::io::split(client);
        let mut read = BufReader::new(read);
        let init = json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"cancel-test","version":"1"}}});
        write.write_all(format!("{init}\n").as_bytes()).await.expect("init");
        let mut line = String::new();
        read.read_line(&mut line).await.expect("init response");
        assert!(serde_json::from_str::<Value>(&line).expect("json").get("result").is_some(), "{line}");
        write.write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n").await.expect("initialized");
        let call = json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"prepared_counter","arguments":{"value":1}}});
        let started_wait = started.notified();
        write.write_all(format!("{call}\n").as_bytes()).await.expect("first request");
        started_wait.await;
        assert_eq!(count.load(Ordering::SeqCst), 1);
        drop(read);
        drop(write);
        cleanup.observed.notified().await;
        assert!(tokio::time::timeout(Duration::from_millis(20), cleanup.completed.notified()).await.is_err(), "cleanup must still be waiting");
        cleanup.release.notify_one();
        cleanup.completed.notified().await;
        server_task.await.expect("server task");
    }).await.expect("transport test deadline");
}
