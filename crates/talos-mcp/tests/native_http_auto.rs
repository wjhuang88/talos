//! Real registered ordinary HTTP calls through rmcp and the production Auto gate.
use async_trait::async_trait;
use rmcp::ServiceExt;
use serde_json::json;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;
use talos_agent::auto_resolver::{
    AutoPermissionAssessor, AutoPermissionControl, AutoPermissionRequest, AutoPermissionResolver,
    ManagedWorkspaceLease, OrdinaryInvocationAssessmentRequest,
};
use talos_agent::permission_pipeline::{
    ApprovalResolver, ApprovalResolverError, PermissionApprovalRequest, PermissionPipeline,
};
use talos_core::{ApprovalChoice, tool::ToolRegistry};
use talos_mcp::server::{McpPermissionGate, TalosMcpHandler};
use talos_permission::{
    InteractionCapability, PermissionContext, PermissionEngine, PermissionMode,
    PermissionSessionState,
};
use talos_plugin::HookRegistry;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

const CHILD: &str = "TALOS_MCP_HTTP_AUTO_CHILD";
const URL: &str = "http://93.184.216.34/mcp-fixture";
const INTENT: &str = "send the fixture request to the supplied URL";

struct Assessor {
    calls: Arc<AtomicUsize>,
    state: Arc<PermissionSessionState>,
    scenario: String,
    tool: String,
}
#[async_trait]
impl AutoPermissionAssessor for Assessor {
    async fn assess(&self, _: AutoPermissionRequest, _: Duration) -> Result<String, String> {
        panic!("HTTP must use whole-invocation review")
    }
    async fn assess_ordinary_invocation(
        &self,
        request: OrdinaryInvocationAssessmentRequest,
        _: Duration,
    ) -> Result<String, String> {
        assert_eq!(request.action.tool, self.tool);
        assert_eq!(request.action.input["url"], URL);
        assert_eq!(request.action.user_intent, INTENT);
        assert_eq!(
            request.configuration["tool_configuration"]["destination_isolation"],
            false
        );
        if self.tool == "http_request" {
            assert_eq!(request.action.input["body"], "fixture-payload");
        }
        self.calls.fetch_add(1, Ordering::SeqCst);
        if self.scenario == "cancel" {
            std::future::pending::<()>().await;
        }
        if self.scenario == "error" {
            return Err("fixture unavailable".into());
        }
        if self.scenario == "stale" {
            self.state.rebind_session().expect("rebind");
        }
        Ok(
            json!({"schema":request.action.schema,"request_digest":request.action.request_digest,
            "decision":if self.scenario == "uncertain" {"human_required"} else {"allow_once"},
            "confidence":"high"})
            .to_string(),
        )
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

async fn child(scenario: String, name: String) {
    let root = tempfile::tempdir().expect("workspace");
    let mut registry = ToolRegistry::new();
    registry
        .register_builtin_contributions(talos_tools::network_tool_contributions())
        .expect("production registration");
    if scenario == "replacement" {
        let contributions = talos_tools::network_tool_contributions();
        registry.register(
            contributions
                .iter()
                .find(|entry| entry.name() == name)
                .expect("HTTP contribution")
                .tool()
                .clone(),
        );
    }
    let mut engine = if matches!(
        scenario.as_str(),
        "deterministic_allow" | "scoped_deny" | "scoped_human"
    ) {
        PermissionEngine::empty()
    } else {
        PermissionEngine::with_workspace_root(root.path().into())
    };
    if matches!(
        scenario.as_str(),
        "broad_allow" | "deterministic_allow" | "scoped_deny" | "scoped_human"
    ) {
        engine.add_rule(talos_permission::PermissionRule::new(
            &name,
            None,
            talos_permission::PermissionDecision::Allow,
        ));
    }
    if matches!(scenario.as_str(), "scoped_deny" | "scoped_human") {
        engine.add_rule(talos_permission::PermissionRule::new_nature(
            talos_core::tool::ToolNature::Network,
            Some("private.example.invalid".into()),
            Some(talos_core::tool::ToolResourceKind::Domain.into()),
            if scenario == "scoped_deny" {
                talos_permission::PermissionDecision::Deny(
                    "undisclosed destination restriction".into(),
                )
            } else {
                talos_permission::PermissionDecision::Ask
            },
        ));
    }
    if matches!(scenario.as_str(), "human" | "deny") {
        engine.add_rule(talos_permission::PermissionRule::new(
            &name,
            None,
            if scenario == "human" {
                talos_permission::PermissionDecision::Ask
            } else {
                talos_permission::PermissionDecision::Deny("explicit policy".into())
            },
        ));
    }
    let state = Arc::new(PermissionSessionState::new(engine));
    let calls = Arc::new(AtomicUsize::new(0));
    let resolver = AutoPermissionResolver::new(
        Arc::new(Assessor {
            calls: calls.clone(),
            state: state.clone(),
            scenario: scenario.clone(),
            tool: name.clone(),
        }),
        Arc::new(Deny),
        ManagedWorkspaceLease::for_permission_session(root.path(), state.clone()).expect("lease"),
        Duration::from_secs(5),
        AutoPermissionControl::new(true),
    );
    let pipeline = PermissionPipeline::new(
        state.clone(),
        PermissionContext::new(
            if scenario == "headless" {
                PermissionMode::Headless
            } else {
                PermissionMode::Interactive
            },
            if scenario == "headless" {
                InteractionCapability::Unavailable
            } else {
                InteractionCapability::Available
            },
        ),
        Some(Arc::new(resolver)),
    );
    let mut gate = McpPermissionGate::with_pipeline(pipeline, Arc::new(HookRegistry::new()));
    if scenario != "no_intent" {
        gate = gate.with_user_intent(INTENT.into());
    }
    let handler = TalosMcpHandler::new(Arc::new(registry), Arc::new(gate));
    let (client_io, server_io) = tokio::io::duplex(65536);
    let server = tokio::spawn(async move {
        handler
            .serve(server_io)
            .await
            .expect("server")
            .waiting()
            .await
            .expect("shutdown");
    });
    let input = if name == "http_request" {
        json!({"method":"POST","url":URL,"body":"fixture-payload","timeout_secs":3})
    } else {
        json!({"url":URL,"mode":"raw","timeout_secs":3})
    };
    if matches!(scenario.as_str(), "cancel" | "cancel_transport") {
        let (read, mut write) = tokio::io::split(client_io);
        let mut read = BufReader::new(read);
        let init = json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"http-cancel-test","version":"1"}}});
        write
            .write_all(format!("{init}\n").as_bytes())
            .await
            .expect("initialize");
        let mut line = String::new();
        read.read_line(&mut line)
            .await
            .expect("initialized response");
        assert!(
            serde_json::from_str::<serde_json::Value>(&line)
                .expect("JSON")
                .get("result")
                .is_some()
        );
        write
            .write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n")
            .await
            .expect("initialized");
        let call = json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":name,"arguments":input}});
        write
            .write_all(format!("{call}\n").as_bytes())
            .await
            .expect("call");
        tokio::time::timeout(Duration::from_secs(5), async {
            if scenario == "cancel_transport" {
                let marker = std::env::var("TALOS_MCP_HTTP_REQUEST_MARKER").expect("marker");
                while !tokio::fs::try_exists(&marker).await.expect("marker status") {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            } else {
                while calls.load(Ordering::SeqCst) == 0 {
                    tokio::task::yield_now().await;
                }
            }
        })
        .await
        .expect("review started");
        let cancel =
            json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":1}});
        write
            .write_all(format!("{cancel}\n").as_bytes())
            .await
            .expect("cancel");
        let ping = json!({"jsonrpc":"2.0","id":2,"method":"ping"});
        write
            .write_all(format!("{ping}\n").as_bytes())
            .await
            .expect("ping");
        line.clear();
        read.read_line(&mut line).await.expect("ping response");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&line).expect("JSON")["id"],
            2
        );
        write
            .write_all(format!("{call}\n").as_bytes())
            .await
            .expect("replay");
        line.clear();
        read.read_line(&mut line).await.expect("replay response");
        let replay: serde_json::Value = serde_json::from_str(&line).expect("JSON");
        assert!(replay.get("error").is_some(), "{replay}");
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(state.grant_count().expect("grants"), 0);
        drop(read);
        drop(write);
        server.await.expect("server task");
        return;
    }
    let client = ().serve(client_io).await.expect("client");
    let result = client
        .peer()
        .call_tool(
            rmcp::model::CallToolRequestParams::new(name)
                .with_arguments(input.as_object().expect("args").clone()),
        )
        .await;
    assert_eq!(
        calls.load(Ordering::SeqCst),
        usize::from(matches!(
            scenario.as_str(),
            "allow" | "uncertain" | "error" | "stale" | "broad_allow"
        )),
        "{scenario}"
    );
    if matches!(
        scenario.as_str(),
        "allow" | "broad_allow" | "deterministic_allow"
    ) {
        let result = result.expect("Auto admission");
        assert_ne!(result.is_error, Some(true), "{result:?}");
        assert!(format!("{result:?}").contains("fixture-response"));
    } else {
        assert!(
            result.is_err() || result.is_ok_and(|result| result.is_error == Some(true)),
            "{scenario}"
        );
    }
    assert_eq!(state.grant_count().expect("grants"), 0);
    client.cancel().await.expect("cancel");
    server.await.expect("server task");
}

#[tokio::test]
async fn registered_http_auto_requires_exact_trusted_admission() {
    if let Ok(scenario) = std::env::var(CHILD) {
        child(
            scenario,
            std::env::var("TALOS_MCP_HTTP_AUTO_TOOL").expect("tool"),
        )
        .await;
        return;
    }
    for name in ["http_request", "fetch_url"] {
        for scenario in [
            "allow",
            "broad_allow",
            "deterministic_allow",
            "scoped_deny",
            "scoped_human",
            "uncertain",
            "error",
            "stale",
            "human",
            "deny",
            "headless",
            "no_intent",
            "replacement",
            "cancel",
            "cancel_transport",
        ] {
            let listener = Arc::new(
                tokio::net::TcpListener::bind("127.0.0.1:0")
                    .await
                    .expect("proxy"),
            );
            let address = listener.local_addr().expect("address");
            let marker_dir = tempfile::tempdir().expect("marker directory");
            let marker = marker_dir.path().join("request-received");
            let request_marker = marker.clone();
            let connections = Arc::new(AtomicUsize::new(0));
            let observed = connections.clone();
            let accepting = listener.clone();
            let server = tokio::spawn(async move {
                let (mut stream, _) = accepting.accept().await.expect("connection");
                observed.fetch_add(1, Ordering::SeqCst);
                let mut bytes = Vec::new();
                loop {
                    let mut buffer = [0; 1024];
                    let count = stream.read(&mut buffer).await.expect("request");
                    assert!(count > 0);
                    bytes.extend_from_slice(&buffer[..count]);
                    assert!(bytes.len() <= 8192);
                    if let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
                        let headers = String::from_utf8_lossy(&bytes[..end]);
                        let length = headers
                            .lines()
                            .find_map(|line| {
                                line.split_once(':')
                                    .filter(|(key, _)| key.eq_ignore_ascii_case("content-length"))
                                    .map(|(_, value)| {
                                        value.trim().parse::<usize>().expect("length")
                                    })
                            })
                            .unwrap_or(0);
                        if bytes.len() >= end + 4 + length {
                            break;
                        }
                    }
                }
                let request = String::from_utf8(bytes).expect("UTF8");
                assert!(
                    request.starts_with(&format!(
                        "{} {URL} HTTP/1.1\r\n",
                        if name == "http_request" {
                            "POST"
                        } else {
                            "GET"
                        }
                    )),
                    "{request}"
                );
                if name == "http_request" {
                    assert!(request.ends_with("fixture-payload"));
                }
                if scenario == "cancel_transport" {
                    tokio::fs::write(request_marker, b"received")
                        .await
                        .expect("signal request");
                    let mut byte = [0u8; 1];
                    let remaining =
                        tokio::time::timeout(Duration::from_secs(5), stream.read(&mut byte))
                            .await
                            .expect("cancel closes transport")
                            .expect("transport read");
                    assert_eq!(remaining, 0, "cancelled HTTP transport remained open");
                    return;
                }
                stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 16\r\nConnection: close\r\n\r\nfixture-response")
                    .await.expect("response");
            });
            let mut command =
                tokio::process::Command::new(std::env::current_exe().expect("test executable"));
            command
                .args([
                    "--exact",
                    "registered_http_auto_requires_exact_trusted_admission",
                    "--nocapture",
                ])
                .env(CHILD, scenario)
                .env("TALOS_MCP_HTTP_REQUEST_MARKER", &marker)
                .env("TALOS_MCP_HTTP_AUTO_TOOL", name)
                .env("HTTP_PROXY", format!("http://{address}"))
                .env("http_proxy", format!("http://{address}"))
                .env("NO_PROXY", "")
                .env("no_proxy", "")
                .env_remove("ALL_PROXY")
                .env_remove("all_proxy")
                .env_remove("HTTPS_PROXY")
                .env_remove("https_proxy")
                .env_remove("REQUEST_METHOD")
                .kill_on_drop(true);
            let output = tokio::time::timeout(Duration::from_secs(20), command.output())
                .await
                .expect("bounded child")
                .expect("child");
            assert!(
                output.status.success(),
                "{name}/{scenario}: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            if matches!(
                scenario,
                "allow" | "broad_allow" | "deterministic_allow" | "cancel_transport"
            ) {
                tokio::time::timeout(Duration::from_secs(5), server)
                    .await
                    .expect("proxy finished")
                    .expect("proxy task");
                assert_eq!(connections.load(Ordering::SeqCst), 1);
            } else {
                server.abort();
                let _ = server.await;
                assert_eq!(connections.load(Ordering::SeqCst), 0, "{name}/{scenario}");
                assert!(
                    tokio::time::timeout(Duration::from_millis(50), listener.accept())
                        .await
                        .is_err(),
                    "{name}/{scenario}: queued unauthorized request"
                );
            }
        }
    }
}
