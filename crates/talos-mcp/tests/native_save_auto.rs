//! Real hosted-native Save capture and retained-write admissions through MCP.

use async_trait::async_trait;
use rmcp::ServiceExt;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use talos_agent::auto_resolver::{
    AutoPermissionAssessor, AutoPermissionControl, AutoPermissionRequest, AutoPermissionResolver,
    FileMutationAssessmentRequest, ManagedWorkspaceLease,
};
use talos_agent::permission_pipeline::{
    ApprovalResolver, ApprovalResolverError, PermissionApprovalRequest, PermissionPipeline,
};
use talos_core::ApprovalChoice;
use talos_core::tool::ToolRegistry;
use talos_mcp::server::{McpPermissionGate, TalosMcpHandler};
use talos_permission::{
    InteractionCapability, PermissionContext, PermissionEngine, PermissionMode,
    PermissionSessionState,
};
use talos_plugin::HookRegistry;

use talos_agent::auto_resolver::OrdinaryInvocationAssessmentRequest;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
const CHILD: &str = "TALOS_MCP_SAVE_AUTO_CHILD";
struct Assessor {
    calls: Arc<AtomicUsize>,
    verified: Arc<AtomicUsize>,
    scenario: String,
}
#[async_trait]
impl AutoPermissionAssessor for Assessor {
    async fn assess(&self, _: AutoPermissionRequest, _: Duration) -> Result<String, String> {
        panic!("Save requires staged assessment")
    }
    async fn assess_ordinary_invocation(
        &self,
        request: OrdinaryInvocationAssessmentRequest,
        _: Duration,
    ) -> Result<String, String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(request.action.tool, "save_url");
        assert_eq!(
            request.configuration["stage"],
            "capture_existing_file_and_download_without_writes"
        );
        assert_eq!(request.configuration["destination_isolation"], false);
        assert_eq!(
            request.action.user_intent,
            "download replacement with a retained backup"
        );
        self.verified.fetch_add(1, Ordering::SeqCst);
        Ok(json!({"schema":request.action.schema,"request_digest":request.action.request_digest,"decision":if self.scenario == "capture_deny" {"human_required"} else {"allow_once"},"confidence":"high"}).to_string())
    }
    async fn assess_file_mutation(
        &self,
        request: FileMutationAssessmentRequest,
        _: Duration,
    ) -> Result<String, String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(request.mutation.before, "original");
        assert_eq!(request.mutation.after.as_deref(), Some("downloaded"));
        self.verified.fetch_add(1, Ordering::SeqCst);
        Ok(json!({"schema":request.schema,"request_digest":request.request_digest,"decision":if self.scenario == "write_deny" {"human_required"} else {"allow_once"},"confidence":"high"}).to_string())
    }
}
struct Deny;
#[async_trait]
impl ApprovalResolver for Deny {
    async fn resolve(
        &self,
        _: PermissionApprovalRequest,
        _: Duration,
    ) -> Result<ApprovalChoice, ApprovalResolverError> {
        Ok(ApprovalChoice::Deny)
    }
}
async fn child(scenario: String) {
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("saved.txt");
    std::fs::write(&path, "original").expect("fixture");
    let calls = Arc::new(AtomicUsize::new(0));
    let verified = Arc::new(AtomicUsize::new(0));
    let state = Arc::new(PermissionSessionState::new(
        PermissionEngine::with_workspace_root(root.path().into()),
    ));
    let resolver = AutoPermissionResolver::new(
        Arc::new(Assessor {
            calls: calls.clone(),
            verified: verified.clone(),
            scenario: scenario.clone(),
        }),
        Arc::new(Deny),
        ManagedWorkspaceLease::for_permission_session(root.path(), state.clone()).expect("lease"),
        Duration::from_secs(5),
        AutoPermissionControl::new(true),
    );
    let pipeline = PermissionPipeline::new(
        state,
        PermissionContext::new(
            PermissionMode::Interactive,
            InteractionCapability::Available,
        ),
        Some(Arc::new(resolver)),
    );
    let gate = McpPermissionGate::with_pipeline(pipeline, Arc::new(HookRegistry::new()));
    let gate = if scenario == "missing_intent" {
        gate
    } else {
        gate.with_user_intent("download replacement with a retained backup".into())
    };
    let mut registry = ToolRegistry::new();
    registry
        .register_builtin_contributions(talos_tools::network_tool_contributions())
        .expect("native registration");
    if scenario == "replacement" {
        registry.register(Arc::new(talos_tools::SaveUrlTool::new()));
    }
    let handler = TalosMcpHandler::new(Arc::new(registry), Arc::new(gate));
    let (client_io, server_io) = tokio::io::duplex(65536);
    let server = tokio::spawn(async move {
        let running = handler.serve(server_io).await.expect("server");
        let _ = running.waiting().await;
    });
    let client = ().serve(client_io).await.expect("client");
    let result = client.peer().call_tool(rmcp::model::CallToolRequestParams::new("save_url").with_arguments(json!({"url":"http://93.184.216.34/fixture","destination":path,"retain_backup":true}).as_object().expect("args").clone())).await;
    let expected = match scenario.as_str() {
        "replacement" | "missing_intent" => 0,
        "capture_deny" => 1,
        _ => 2,
    };
    assert_eq!(calls.load(Ordering::SeqCst), expected);
    assert_eq!(
        verified.load(Ordering::SeqCst),
        expected,
        "assessor assertions completed"
    );
    if scenario == "allow" {
        let result = result.expect("admitted");
        assert_ne!(result.is_error, Some(true), "{result:?}");
        assert!(
            serde_json::to_string(&result)
                .expect("result")
                .contains("recovery_id")
        );
    } else {
        assert!(
            result.as_ref().map_or(true, |r| r.is_error == Some(true)),
            "{result:?}"
        );
    }
    assert_eq!(
        std::fs::read_to_string(path).expect("contents"),
        if scenario == "allow" {
            "downloaded"
        } else {
            "original"
        }
    );
    let _ = client.cancel().await;
    server.await.expect("joined");
}
#[tokio::test]
async fn save_download_and_write_require_separate_native_auto_admissions() {
    if let Ok(scenario) = std::env::var(CHILD) {
        child(scenario).await;
        return;
    }
    for scenario in [
        "allow",
        "capture_deny",
        "write_deny",
        "replacement",
        "missing_intent",
    ] {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("proxy");
        let address = listener.local_addr().expect("address");
        let connections = Arc::new(AtomicUsize::new(0));
        let observed = connections.clone();
        let proxy = tokio::spawn(async move {
            loop {
                let (mut stream, _) = listener.accept().await.expect("accept");
                observed.fetch_add(1, Ordering::SeqCst);
                let mut headers = Vec::new();
                while !headers.ends_with(b"\r\n\r\n") {
                    assert!(headers.len() < 16384);
                    headers.push(stream.read_u8().await.expect("header"));
                }
                stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 10\r\nConnection: close\r\n\r\ndownloaded").await.expect("body");
            }
        });
        let mut command = tokio::process::Command::new(std::env::current_exe().expect("test exe"));
        command
            .args([
                "--exact",
                "save_download_and_write_require_separate_native_auto_admissions",
                "--nocapture",
            ])
            .env(CHILD, scenario);
        for key in [
            "HTTP_PROXY",
            "http_proxy",
            "HTTPS_PROXY",
            "https_proxy",
            "ALL_PROXY",
            "all_proxy",
        ] {
            command.env(key, format!("http://{address}"));
        }
        command
            .env("NO_PROXY", "")
            .env("no_proxy", "")
            .kill_on_drop(true);
        let output = tokio::time::timeout(Duration::from_secs(30), command.output())
            .await
            .expect("child deadline")
            .expect("spawn");
        assert!(
            output.status.success(),
            "{scenario}: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        // Child is terminal; no outstanding request can start after this observation.
        assert_eq!(
            connections.load(Ordering::SeqCst),
            usize::from(matches!(scenario, "allow" | "write_deny")),
            "{scenario}"
        );
        proxy.abort();
        let _ = proxy.await;
    }
}
