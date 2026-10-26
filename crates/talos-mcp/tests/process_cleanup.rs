//! Real descendant cancellation across MCP transport and prepared invocation boundaries.
#![cfg(unix)]

use async_trait::async_trait;
use rmcp::ServiceExt;
use serde_json::{Value, json};
use std::{path::PathBuf, sync::Arc, time::Duration};
use talos_core::tool::*;
use talos_mcp::server::{McpPermissionGate, TalosMcpHandler};
use talos_permission::{AdmittedInvocation, PermissionEngine, PermissionRequest};
use talos_plugin::HookRegistry;
use talos_sandbox::{OwnedCommandExecutor, OwnedCommandOperation};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

struct ProcessTool {
    root: PathBuf,
    owner: Arc<OwnedCommandExecutor>,
}
struct ProcessPlan {
    root: PathBuf,
    operation: OwnedCommandOperation,
    identity: PreparedPlanIdentity,
    provenance: ToolProvenance,
    facets: Vec<ToolPermissionFacet>,
    input: Value,
}
#[async_trait]
impl AgentTool for ProcessTool {
    fn name(&self) -> &str {
        "process_fixture"
    }
    fn description(&self) -> &str {
        "owned process cancellation fixture"
    }
    fn parameters(&self) -> Value {
        json!({"type":"object"})
    }
    fn cleanup_receipt(&self) -> Option<Arc<dyn ToolCleanupOwner>> {
        Some(self.owner.clone())
    }
    async fn execute(&self, _: Value) -> ToolResult {
        panic!("prepared route required")
    }
    fn prepare_normalized_with_provenance(
        &self,
        input: &Value,
        provenance: ToolProvenance,
    ) -> Result<Option<Box<dyn PreparedNormalToolInvocation>>, String> {
        Ok(Some(Box::new(ProcessPlan {
            root: self.root.clone(),
            operation: self.owner.operation(),
            identity: PreparedPlanIdentity::new(),
            provenance,
            // Test-only auto-admission permits exercising the real execution path.
            facets: vec![ToolPermissionFacet::new(ToolNature::Read)],
            input: input.clone(),
        })))
    }
}
#[async_trait]
impl PreparedNormalToolInvocation for ProcessPlan {
    fn plan_identity(&self) -> Option<&PreparedPlanIdentity> {
        Some(&self.identity)
    }
    fn permission_profile(&self) -> &[ToolPermissionFacet] {
        &self.facets
    }
    fn cleanup_receipt(&self) -> Option<Arc<dyn ToolCleanupReceipt>> {
        Some(Arc::new(self.operation.cleanup_receipt()))
    }
    async fn execute(
        self: Box<Self>,
        admission: Box<dyn std::any::Any + Send>,
    ) -> ToolExecutionOutput {
        let proof = admission
            .downcast::<AdmittedInvocation>()
            .expect("real admission");
        proof
            .consume_for_prepared(
                &PermissionRequest::new(
                    "process_fixture",
                    self.provenance.clone(),
                    &self.facets,
                    &self.input,
                ),
                &self.identity,
            )
            .expect("bound proof");
        let mut command = std::process::Command::new("sh");
        command.current_dir(&self.root).args([
            "-c",
            "(sleep 2; printf forbidden > delayed) & printf ready > ready; wait",
        ]);
        let result = self
            .operation
            .execute(command, Duration::from_secs(20), 1024)
            .await;
        ToolExecutionOutput::error(format!("fixture result: {result:?}"))
    }
}

#[tokio::test]
async fn cancellation_reaps_descendants_and_preserves_shared_owner() {
    tokio::time::timeout(Duration::from_secs(15), async {
        let root = tempfile::tempdir().expect("tempdir");
        let owner = Arc::new(OwnedCommandExecutor::default());
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(ProcessTool { root: root.path().into(), owner: owner.clone() }));
        let registry = Arc::new(registry);
        let gate = Arc::new(McpPermissionGate::new(Arc::new(PermissionEngine::new()), Arc::new(HookRegistry::new())));
        let unrelated = TalosMcpHandler::new(registry.clone(), gate.clone());
        let handler = Arc::new(TalosMcpHandler::new(registry, gate));
        let (client, server) = tokio::io::duplex(65536);
        let serving_handler = handler.clone();
        let server_task = tokio::spawn(async move {
            let service = serving_handler.serve(server).await.expect("handshake");
            service.waiting().await.expect("transport stop");
        });
        let (read, mut write) = tokio::io::split(client);
        let mut read = BufReader::new(read);
        let init = json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"process-test","version":"1"}}});
        write.write_all(format!("{init}\n").as_bytes()).await.expect("initialize");
        let mut line = String::new();
        read.read_line(&mut line).await.expect("initialized response");
        write.write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n").await.expect("initialized");
        let call = json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"process_fixture","arguments":{}}});
        write.write_all(format!("{call}\n").as_bytes()).await.expect("call");
        while !root.path().join("ready").exists() { tokio::time::sleep(Duration::from_millis(5)).await; }
        unrelated.shutdown(Duration::from_millis(50)).await.expect("must not wait unrelated active process");
        let cancel = json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":1}});
        write.write_all(format!("{cancel}\n").as_bytes()).await.expect("cancel");
        handler.shutdown(Duration::from_secs(3)).await.expect("connection confirms cleanup");
        // A fresh operation proves neither connection shutdown sealed the shared owner.
        let mut command = std::process::Command::new("sh");
        command.args(["-c", "exit 0"]);
        assert!(owner.operation().execute(command, Duration::from_secs(2), 1024).await.expect("shared owner remains usable").status.success());
        tokio::time::sleep(Duration::from_millis(2100)).await;
        assert!(!root.path().join("delayed").exists(), "descendant survived confirmed cleanup");
        drop(read); drop(write);
        server_task.await.expect("server joined");
    }).await.expect("test deadline");
}
