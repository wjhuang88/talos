//! Production Save URL staged Auto admission, real network and recoverable writes.
use super::*;
use crate::auto_resolver::{FileMutationAssessmentRequest, OrdinaryInvocationAssessmentRequest};
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const CHILD_SCENARIO: &str = "TALOS_I308_SAVE_AUTO_CHILD";
const URL: &str = "http://93.184.216.34/start";

fn user_intent(_: &str) -> &'static str {
    "replace file.txt with the supplied URL content and retain an overwrite backup"
}

struct Assessor {
    scenario: String,
    tool: String,
    calls: Arc<AtomicUsize>,
    reached: Arc<AtomicUsize>,
    state: Arc<talos_permission::PermissionSessionState>,
}
#[async_trait]
impl AutoPermissionAssessor for Assessor {
    async fn assess(
        &self,
        _: AutoPermissionRequest,
        _: std::time::Duration,
    ) -> Result<String, String> {
        panic!("Save capture must use whole-invocation assessment")
    }
    async fn assess_file_mutation(
        &self,
        request: FileMutationAssessmentRequest,
        _: std::time::Duration,
    ) -> Result<String, String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(request.input["destination"], "file.txt");
        assert_eq!(request.mutation.before, "original");
        assert_eq!(request.mutation.after.as_deref(), Some("fixture"));
        assert_eq!(request.facets.len(), 2);
        self.reached.fetch_add(1, Ordering::SeqCst);
        match self.scenario.as_str() {
            "write_error" => return Err("unavailable".into()),
            "write_stale" => self.state.rebind_session().expect("rebind"),
            "changed" => std::fs::write(&request.mutation.path, "concurrent").expect("writer"),
            _ => {}
        }
        Ok(serde_json::json!({"schema":request.schema,"request_digest":request.request_digest,
            "decision":if self.scenario == "write_uncertain" {"human_required"} else {"allow_once"},"confidence":"high"}).to_string())
    }
    async fn assess_ordinary_invocation(
        &self,
        request: OrdinaryInvocationAssessmentRequest,
        _: std::time::Duration,
    ) -> Result<String, String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(request.action.tool, self.tool);
        assert_eq!(request.action.input["url"], URL);
        assert_eq!(request.action.user_intent, user_intent(&self.tool));
        assert_eq!(request.action.input["retain_backup"], true);
        assert_eq!(request.action.input["destination"], "file.txt");
        assert_eq!(
            request.configuration["stage"],
            "capture_existing_file_and_download_without_writes"
        );
        assert_eq!(
            request.configuration["maximum_download_bytes"],
            2 * 1024 * 1024
        );
        assert_eq!(request.configuration["backend"], "reqwest");
        assert_eq!(request.configuration["destination_isolation"], false);
        self.reached.fetch_add(1, Ordering::SeqCst);
        match self.scenario.as_str() {
            "error" => return Err("unavailable".into()),
            "stale" => self.state.rebind_session().expect("rebind"),
            _ => {}
        }
        Ok(serde_json::json!({"schema":request.action.schema,"request_digest":request.action.request_digest,
            "decision":if self.scenario == "uncertain" {"human_required"} else {"allow_once"},"confidence":"high"}).to_string())
    }
}

struct ManualApproval;
#[async_trait]
impl ApprovalResolver for ManualApproval {
    async fn resolve(
        &self,
        _: PermissionApprovalRequest,
        _: std::time::Duration,
    ) -> Result<talos_core::ApprovalChoice, ApprovalResolverError> {
        Ok(talos_core::ApprovalChoice::ApproveOnce)
    }
}

struct SaveModel {
    inner: MockModel,
    allow: bool,
    results: Arc<AtomicUsize>,
}
#[async_trait]
impl LanguageModel for SaveModel {
    async fn stream(&self, messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
        for message in messages {
            if let Message::Tool { result } = message {
                if result.tool_use_id == "http" {
                    self.results.fetch_add(1, Ordering::SeqCst);
                    assert_eq!(!result.is_error, self.allow, "{}", result.content);
                    if self.allow {
                        assert!(
                            result.content.contains("recovery_id: "),
                            "{}",
                            result.content
                        );
                    }
                }
            }
        }
        self.inner.stream(messages).await
    }
}

async fn child(scenario: String, tool: String) {
    let allow = matches!(
        scenario.as_str(),
        "allow" | "manual_external" | "manual_replacement"
    );
    let results = Arc::new(AtomicUsize::new(0));
    let root = tempfile::tempdir().expect("workspace");
    std::fs::write(root.path().join("file.txt"), "original").expect("original");
    let mut registry = ToolRegistry::new();
    if !matches!(scenario.as_str(), "external" | "manual_external") {
        registry
            .register_builtin_contributions(talos_tools::network_tool_contributions())
            .expect("production contributions");
    }
    if matches!(
        scenario.as_str(),
        "external" | "replacement" | "manual_external" | "manual_replacement"
    ) {
        let contributions = talos_tools::network_tool_contributions();
        let tool = contributions
            .iter()
            .find(|entry| entry.name() == tool)
            .expect("production Save")
            .tool()
            .clone();
        registry.register(tool);
    }
    let mut engine = PermissionEngine::with_workspace_root(root.path().to_owned());
    if matches!(scenario.as_str(), "deny" | "human") {
        engine.add_rule(talos_permission::PermissionRule::new(
            &tool,
            None,
            if scenario == "deny" {
                PermissionDecision::Deny("explicit test policy".into())
            } else {
                PermissionDecision::Ask
            },
        ));
    }
    let expected_calls = if matches!(
        scenario.as_str(),
        "allow" | "write_error" | "write_stale" | "write_uncertain" | "changed"
    ) {
        2
    } else {
        usize::from(!matches!(
            scenario.as_str(),
            "deny"
                | "human"
                | "external"
                | "replacement"
                | "manual_external"
                | "manual_replacement"
                | "no_backup"
        ))
    };
    let state = Arc::new(talos_permission::PermissionSessionState::new(engine));
    let calls = Arc::new(AtomicUsize::new(0));
    let reached = Arc::new(AtomicUsize::new(0));
    let resolver = AutoPermissionResolver::new(
        Arc::new(Assessor {
            scenario: scenario.clone(),
            tool: tool.clone(),
            calls: calls.clone(),
            reached: reached.clone(),
            state: state.clone(),
        }),
        if scenario.starts_with("manual_") {
            Arc::new(ManualApproval) as Arc<dyn ApprovalResolver>
        } else {
            Arc::new(AutoDenyFallback) as Arc<dyn ApprovalResolver>
        },
        ManagedWorkspaceLease::for_permission_session(root.path(), state.clone()).expect("lease"),
        std::time::Duration::from_secs(2),
        AutoPermissionControl::new(true),
    );
    let model = MockModel::new(vec![
        vec![
            AgentEvent::ToolCall {
                call: ToolCall {
                    id: "http".into(),
                    name: tool.clone(),
                    input: serde_json::json!({"url":URL,"destination":"file.txt","retain_backup":scenario != "no_backup"}),
                },
                provenance: Default::default(),
                summary_fields: vec![],
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::ToolUse,
                usage: Usage::default(),
            },
        ],
        text_done_events("done"),
    ]);
    let mut agent = Agent::with_security(
        Arc::new(SaveModel {
            inner: model,
            allow,
            results: results.clone(),
        }),
        registry,
        None,
        None,
        root.path().to_owned(),
    )
    .with_permission_pipeline(
        state.clone(),
        talos_permission::PermissionContext::new(
            talos_permission::PermissionMode::Interactive,
            talos_permission::InteractionCapability::Available,
        ),
        Some(Arc::new(resolver)),
    );
    agent.set_tool_presentation_policy(ToolPresentationPolicy::with_families([
        ToolFamily::AdvancedNetwork,
        ToolFamily::Network,
    ]));
    agent.run(user_intent(&tool).into()).await.expect("run");
    assert_eq!(calls.load(Ordering::SeqCst), expected_calls);
    assert_eq!(reached.load(Ordering::SeqCst), expected_calls);
    assert_eq!(results.load(Ordering::SeqCst), 1);
    assert_eq!(state.grant_count().expect("grants"), 0);
    assert_eq!(
        std::fs::read_to_string(root.path().join("file.txt")).expect("result"),
        if allow {
            "fixture"
        } else if scenario == "changed" {
            "concurrent"
        } else {
            "original"
        }
    );
}

async fn read_request(stream: &mut tokio::net::TcpStream) -> String {
    let mut bytes = Vec::new();
    loop {
        let mut buffer = [0; 1024];
        let count = stream.read(&mut buffer).await.expect("request read");
        assert!(count > 0, "request ended early");
        bytes.extend_from_slice(&buffer[..count]);
        assert!(bytes.len() <= 8192, "bounded fixture request");
        if let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
            let headers = String::from_utf8_lossy(&bytes[..end]);
            let length = headers
                .lines()
                .find_map(|line| {
                    line.split_once(':')
                        .filter(|(key, _)| key.eq_ignore_ascii_case("content-length"))
                        .map(|(_, value)| value.trim().parse::<usize>().expect("length"))
                })
                .unwrap_or(0);
            if bytes.len() >= end + 4 + length {
                return String::from_utf8(bytes).expect("UTF8 request");
            }
        }
    }
}

#[tokio::test]
async fn real_save_auto_controls_download_and_recoverable_write() {
    if let Ok(scenario) = std::env::var(CHILD_SCENARIO) {
        child(
            scenario,
            std::env::var("TALOS_I308_SAVE_AUTO_TOOL").expect("tool"),
        )
        .await;
        return;
    }
    for tool in ["save_url"] {
        for scenario in [
            "allow",
            "uncertain",
            "error",
            "stale",
            "deny",
            "human",
            "external",
            "replacement",
            "write_uncertain",
            "write_error",
            "write_stale",
            "changed",
            "manual_external",
            "manual_replacement",
            "no_backup",
            "binary",
        ] {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .expect("proxy");
            let address = listener.local_addr().expect("address");
            let listener = Arc::new(listener);
            let server_listener = listener.clone();
            let connections = Arc::new(AtomicUsize::new(0));
            let observed = connections.clone();
            let server = tokio::spawn(async move {
                for index in 0..2 {
                    let (mut stream, _) = server_listener.accept().await.expect("accept");
                    observed.fetch_add(1, Ordering::SeqCst);
                    let request = read_request(&mut stream).await;
                    if index == 0 {
                        assert!(
                            request.starts_with("GET http://93.184.216.34/start HTTP/1.1\r\n"),
                            "{request}"
                        );
                        stream.write_all(b"HTTP/1.1 302 Found\r\nLocation: http://93.184.216.34/finish\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").await.expect("redirect");
                    } else {
                        assert!(
                            request.starts_with("GET http://93.184.216.34/finish HTTP/1.1\r\n"),
                            "{request}"
                        );
                        let response = if scenario == "binary" {
                            &b"HTTP/1.1 200 OK\r\nContent-Length: 7\r\nConnection: close\r\n\r\n\xffixture"[..]
                        } else {
                            &b"HTTP/1.1 200 OK\r\nContent-Length: 7\r\nConnection: close\r\n\r\nfixture"[..]
                        };
                        stream.write_all(response).await.expect("response");
                    }
                }
            });
            let mut command =
                tokio::process::Command::new(std::env::current_exe().expect("test executable"));
            command
                .args([
                    "--exact",
                    "tests::save_url_auto::real_save_auto_controls_download_and_recoverable_write",
                    "--nocapture",
                ])
                .env(CHILD_SCENARIO, scenario)
                .env("TALOS_I308_SAVE_AUTO_TOOL", tool)
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
            let output = tokio::time::timeout(std::time::Duration::from_secs(20), command.output())
                .await
                .expect("bounded child")
                .expect("child");
            assert!(
                output.status.success(),
                "{scenario}: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            if matches!(
                scenario,
                "allow"
                    | "write_uncertain"
                    | "write_error"
                    | "write_stale"
                    | "changed"
                    | "manual_external"
                    | "manual_replacement"
                    | "binary"
            ) {
                tokio::time::timeout(std::time::Duration::from_secs(5), server)
                    .await
                    .expect("proxy completed")
                    .expect("proxy task");
                assert_eq!(connections.load(Ordering::SeqCst), 2);
            } else {
                server.abort();
                let _ = server.await;
                assert_eq!(connections.load(Ordering::SeqCst), 0, "{scenario}");
                assert!(
                    tokio::time::timeout(std::time::Duration::from_millis(50), listener.accept())
                        .await
                        .is_err(),
                    "{scenario}: unexpected queued proxy connection"
                );
            }
        }
    }
}
