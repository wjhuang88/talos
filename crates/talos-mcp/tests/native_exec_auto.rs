//! Registered MCP Exec whole-invocation Auto admission and execution.

use async_trait::async_trait;
use rmcp::ServiceExt;
use serde_json::json;
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
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
    InteractionCapability, PermissionContext, PermissionDecision, PermissionEngine, PermissionMode,
    PermissionRule, PermissionSessionState,
};
use talos_plugin::HookRegistry;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

struct Assessor {
    calls: Arc<AtomicUsize>,
    root: String,
    tool: &'static str,
    outcome: AssessorOutcome,
    started: Option<Arc<tokio::sync::Notify>>,
}

#[derive(Clone, Copy)]
enum AssessorOutcome {
    Allow,
    Uncertain,
    Error,
}

#[async_trait]
impl AutoPermissionAssessor for Assessor {
    async fn assess(&self, _: AutoPermissionRequest, _: Duration) -> Result<String, String> {
        panic!("Exec must use whole-invocation review")
    }

    async fn assess_ordinary_invocation(
        &self,
        request: OrdinaryInvocationAssessmentRequest,
        _: Duration,
    ) -> Result<String, String> {
        assert_eq!(request.action.tool, self.tool);
        assert_eq!(
            request.action.user_intent,
            "record the fixture identity in Git config"
        );
        if self.tool == "exec" {
            assert_eq!(request.action.input["command"], "git");
            assert_eq!(request.action.input["args"][0], "config");
            assert_eq!(
                request.configuration["tool_configuration"]["backend"],
                "direct-argv"
            );
        } else {
            assert!(
                request.action.input["command"]
                    .as_str()
                    .is_some_and(|value| value.contains("git config"))
            );
            assert_eq!(
                request.configuration["tool_configuration"]["backend"],
                if cfg!(windows) {
                    "powershell.exe"
                } else {
                    "sh -c"
                }
            );
        }
        assert_eq!(
            request.working_directory.as_deref(),
            Some(self.root.as_str())
        );
        assert!(request.directory_identity.is_some());
        assert_eq!(request.configuration["execution_directories"][0], self.root);
        assert_eq!(
            request.configuration["execution_directories_resolved"],
            true
        );
        self.calls.fetch_add(1, Ordering::SeqCst);
        if let Some(started) = &self.started {
            started.notify_one();
            return std::future::pending().await;
        }
        match self.outcome {
            AssessorOutcome::Error => Err("injected assessor failure".into()),
            AssessorOutcome::Allow | AssessorOutcome::Uncertain => Ok(json!({
            "schema": request.action.schema,
            "request_digest": request.action.request_digest,
            "decision": if matches!(self.outcome, AssessorOutcome::Allow) { "allow_once" } else { "human_required" },
            "confidence": if matches!(self.outcome, AssessorOutcome::Allow) { "high" } else { "low" },
        })
            .to_string()),
        }
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

#[tokio::test]
async fn registered_mcp_exec_requires_bound_auto_review_before_git_config_mutation() {
    let shell_tool = if cfg!(windows) { "powershell" } else { "bash" };
    for tool in ["exec", shell_tool] {
        for (explicit_deny, outcome) in [
            (false, AssessorOutcome::Allow),
            (true, AssessorOutcome::Allow),
            (false, AssessorOutcome::Uncertain),
            (false, AssessorOutcome::Error),
        ] {
            let root = tempfile::tempdir().expect("workspace");
            let root_path = root.path().canonicalize().expect("canonical root");
            assert!(
                std::process::Command::new("git")
                    .current_dir(&root_path)
                    .args(["init", "--quiet"])
                    .status()
                    .expect("initialize Git fixture")
                    .success()
            );
            let mut engine = PermissionEngine::with_workspace_root(root_path.clone());
            if explicit_deny {
                engine.add_rule(PermissionRule::new_nature(
                    talos_core::tool::ToolNature::Execute,
                    None,
                    None,
                    PermissionDecision::Deny("explicit Execute Deny".into()),
                ));
            }
            let state = Arc::new(PermissionSessionState::new(engine));
            let calls = Arc::new(AtomicUsize::new(0));
            let resolver = AutoPermissionResolver::new(
                Arc::new(Assessor {
                    calls: calls.clone(),
                    root: root_path.to_string_lossy().into_owned(),
                    tool,
                    outcome,
                    started: None,
                }),
                Arc::new(Deny),
                ManagedWorkspaceLease::for_permission_session(&root_path, state.clone())
                    .expect("lease"),
                Duration::from_secs(5),
                AutoPermissionControl::new(true),
            );
            let pipeline = PermissionPipeline::new(
                state.clone(),
                PermissionContext::new(
                    PermissionMode::Interactive,
                    InteractionCapability::Available,
                ),
                Some(Arc::new(resolver)),
            );
            let gate = McpPermissionGate::with_pipeline(pipeline, Arc::new(HookRegistry::new()))
                .with_user_intent("record the fixture identity in Git config".into());
            let mut registry = ToolRegistry::new();
            registry
                .register_builtin_contributions(talos_tools::shell_tool_contributions(
                    root_path.clone(),
                ))
                .expect("registered native shell tools");
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
            let client = ().serve(client_io).await.expect("client");
            let arguments = if tool == "exec" {
                json!({
                    "command": "git",
                    "args": ["config", "--local", "user.name", "Fixture"],
                    "cwd": ".",
                })
            } else {
                json!({"command":"git config --local user.name Fixture"})
            };
            let result = client
                .peer()
                .call_tool(
                    rmcp::model::CallToolRequestParams::new(tool)
                        .with_arguments(arguments.as_object().expect("arguments").clone()),
                )
                .await;
            assert_eq!(calls.load(Ordering::SeqCst), usize::from(!explicit_deny));
            let admitted = !explicit_deny && matches!(outcome, AssessorOutcome::Allow);
            assert_eq!(
                std::process::Command::new("git")
                    .current_dir(&root_path)
                    .args(["config", "--local", "--get", "user.name"])
                    .output()
                    .expect("read Git config")
                    .status
                    .success(),
                admitted,
                "{result:?}"
            );
            if admitted {
                let result = result.expect("Auto admission");
                assert_ne!(result.is_error, Some(true), "{result:?}");
            } else {
                assert!(result.is_err(), "non-allow outcomes must deny the MCP call");
            }
            assert_eq!(state.grant_count().expect("grants"), 0);
            client.cancel().await.expect("cancel");
            server.await.expect("server joined");
        }
    }
}

#[tokio::test]
async fn cancelled_registered_exec_auto_review_cannot_mutate_or_replay() {
    tokio::time::timeout(Duration::from_secs(15), cancelled_exec_review())
        .await
        .expect("cancelled MCP review deadline");
}

async fn cancelled_exec_review() {
    let root = tempfile::tempdir().expect("workspace");
    let root_path = root.path().canonicalize().expect("canonical root");
    assert!(
        std::process::Command::new("git")
            .current_dir(&root_path)
            .args(["init", "--quiet"])
            .status()
            .expect("initialize Git fixture")
            .success()
    );

    let state = Arc::new(PermissionSessionState::new(
        PermissionEngine::with_workspace_root(root_path.clone()),
    ));
    let calls = Arc::new(AtomicUsize::new(0));
    let started = Arc::new(tokio::sync::Notify::new());
    let resolver = AutoPermissionResolver::new(
        Arc::new(Assessor {
            calls: calls.clone(),
            root: root_path.to_string_lossy().into_owned(),
            tool: "exec",
            outcome: AssessorOutcome::Allow,
            started: Some(started.clone()),
        }),
        Arc::new(Deny),
        ManagedWorkspaceLease::for_permission_session(&root_path, state.clone()).expect("lease"),
        Duration::from_secs(5),
        AutoPermissionControl::new(true),
    );
    let pipeline = PermissionPipeline::new(
        state.clone(),
        PermissionContext::new(
            PermissionMode::Interactive,
            InteractionCapability::Available,
        ),
        Some(Arc::new(resolver)),
    );
    let gate = McpPermissionGate::with_pipeline(pipeline, Arc::new(HookRegistry::new()))
        .with_user_intent("record the fixture identity in Git config".into());
    let mut registry = ToolRegistry::new();
    registry
        .register_builtin_contributions(talos_tools::shell_tool_contributions(root_path.clone()))
        .expect("registered native shell tools");
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
    let (read, mut write) = tokio::io::split(client_io);
    let mut read = BufReader::new(read);
    let initialize = json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"exec-cancel-test","version":"1"}}});
    write
        .write_all(format!("{initialize}\n").as_bytes())
        .await
        .expect("initialize");
    let mut line = String::new();
    read.read_line(&mut line)
        .await
        .expect("initialize response");
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

    let call = json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"exec","arguments":{"command":"git","args":["config","--local","user.name","Fixture"],"cwd":"."}}});
    write
        .write_all(format!("{call}\n").as_bytes())
        .await
        .expect("call");
    tokio::time::timeout(Duration::from_secs(5), started.notified())
        .await
        .expect("assessor started");
    let cancelled = json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":1,"reason":"fixture cancellation"}});
    write
        .write_all(format!("{cancelled}\n").as_bytes())
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
    assert!(
        replay["error"]["message"]
            .as_str()
            .expect("error")
            .contains("already used")
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(state.grant_count().expect("grants"), 0);
    assert!(
        !std::process::Command::new("git")
            .current_dir(&root_path)
            .args(["config", "--local", "--get", "user.name"])
            .output()
            .expect("read Git config")
            .status
            .success()
    );
    drop(read);
    drop(write);
    server.await.expect("server joined");
}
