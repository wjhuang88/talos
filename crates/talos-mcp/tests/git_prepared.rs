#[path = "support/git_daemon.rs"]
mod git_daemon;

use rmcp::ServiceExt;
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::Duration;
use talos_core::tool::{ToolNature, ToolRegistry};
use talos_mcp::server::{McpPermissionGate, TalosMcpHandler};
use talos_permission::{PermissionDecision, PermissionEngine, PermissionRule};
use talos_plugin::HookRegistry;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[path = "support/git_checkout_cases.rs"]
mod checkout;

#[tokio::test]
async fn injected_auto_reviews_only_registry_owned_git_commands_with_user_intent() {
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use talos_agent::auto_resolver::{
        AutoPermissionAssessor, AutoPermissionControl, AutoPermissionRequest,
        AutoPermissionResolver, ManagedWorkspaceLease, PreparedCommandAssessmentRequest,
    };
    use talos_agent::permission_pipeline::{
        ApprovalResolver, ApprovalResolverError, PermissionApprovalRequest, PermissionPipeline,
    };
    use talos_core::ApprovalChoice;
    use talos_permission::{
        InteractionCapability, PermissionContext, PermissionMode, PermissionSessionState,
    };

    struct Assessor(Arc<AtomicUsize>);
    #[async_trait]
    impl AutoPermissionAssessor for Assessor {
        async fn assess(&self, _: AutoPermissionRequest, _: Duration) -> Result<String, String> {
            panic!("effectful command must not enter read-only assessment")
        }
        async fn assess_prepared_command(
            &self,
            request: PreparedCommandAssessmentRequest,
            _: Duration,
        ) -> Result<String, String> {
            self.0.fetch_add(1, Ordering::SeqCst);
            assert_eq!(request.tool, "git_add");
            assert_eq!(request.command.operation, "git:add-command/v1");
            assert_eq!(request.user_intent, "stage selected.txt in this repository");
            Ok(
                json!({"schema": request.schema, "request_digest": request.request_digest,
                "decision": "allow_once", "confidence": "high",
                "acknowledged_effects": request.command.effects,
                "accepts_unconfined_effects": true})
                .to_string(),
            )
        }
    }
    // Configuration discovery remains an explicit host approval. Never allow
    // the final command through fallback, so success proves model admission.
    struct DiscoveryThenDeny;
    #[async_trait]
    impl ApprovalResolver for DiscoveryThenDeny {
        async fn resolve(
            &self,
            request: PermissionApprovalRequest,
            _: Duration,
        ) -> Result<ApprovalChoice, ApprovalResolverError> {
            Ok(
                if request
                    .preview
                    .facets()
                    .iter()
                    .any(|facet| facet.normalized_scope == "git:add-command")
                {
                    ApprovalChoice::Deny
                } else {
                    ApprovalChoice::ApproveOnce
                },
            )
        }
    }

    for (builtin, intent, headless, policy) in [
        (true, true, false, None),
        (false, true, false, None),
        (true, false, false, None),
        (true, true, true, None),
        (true, true, false, Some(PermissionDecision::Ask)),
        (
            true,
            true,
            false,
            Some(PermissionDecision::Deny("explicit denial".into())),
        ),
    ] {
        let allowed = builtin && intent && !headless && policy.is_none();
        let root = tempfile::tempdir().expect("workspace");
        let git = |args: &[&str]| {
            std::process::Command::new("git")
                .args(args)
                .current_dir(root.path())
                .output()
                .expect("git fixture")
        };
        assert!(git(&["init"]).status.success());
        std::fs::write(root.path().join("selected.txt"), "selected\n").expect("fixture");
        let calls = Arc::new(AtomicUsize::new(0));
        let mut engine = PermissionEngine::with_workspace_root(root.path().into());
        if let Some(decision) = policy {
            engine.add_rule(PermissionRule::new_nature(
                ToolNature::Execute,
                Some("git:add-command".into()),
                None,
                decision,
            ));
        }
        let state = Arc::new(PermissionSessionState::new(engine));
        let resolver = AutoPermissionResolver::new(
            Arc::new(Assessor(calls.clone())),
            Arc::new(DiscoveryThenDeny),
            ManagedWorkspaceLease::for_permission_session(root.path(), state.clone())
                .expect("lease"),
            Duration::from_secs(5),
            AutoPermissionControl::new(true),
        );
        let pipeline = PermissionPipeline::new(
            state,
            PermissionContext::new(
                if headless {
                    PermissionMode::Headless
                } else {
                    PermissionMode::Interactive
                },
                InteractionCapability::Available,
            ),
            Some(Arc::new(resolver)),
        );
        let mut gate = McpPermissionGate::with_pipeline(pipeline, Arc::new(HookRegistry::new()));
        if intent {
            gate = gate.with_user_intent("stage selected.txt in this repository".into());
        }
        let mut registry = ToolRegistry::new();
        registry
            .register_builtin_contributions(talos_tools::git_mutation_tool_contributions(
                root.path().into(),
            ))
            .expect("native contributions");
        if !builtin {
            // Replacing even with the same implementation revokes factory identity.
            registry.register(Arc::new(talos_tools::git::GitAddTool::new(
                root.path().into(),
            )));
        }
        let handler = TalosMcpHandler::new(Arc::new(registry), Arc::new(gate));
        let (client_io, server_io) = tokio::io::duplex(65536);
        let task = tokio::spawn(async move {
            handler
                .serve(server_io)
                .await
                .expect("handshake")
                .waiting()
                .await
                .expect("shutdown");
        });
        let client = ().serve(client_io).await.expect("client");
        let result = client
            .peer()
            .call_tool(
                rmcp::model::CallToolRequestParams::new("git_add").with_arguments(
                    json!({"paths": ["selected.txt"]})
                        .as_object()
                        .expect("args")
                        .clone(),
                ),
            )
            .await;
        if allowed {
            let result = result.expect("Auto admission");
            assert_ne!(result.is_error, Some(true), "{result:?}");
            assert!(
                calls.load(Ordering::SeqCst) > 0,
                "real Auto assessor reached"
            );
            let staged = git(&["show", ":selected.txt"]);
            assert!(staged.status.success());
            assert_eq!(staged.stdout, b"selected\n");
        } else {
            assert!(
                result.is_err(),
                "untrusted or intentless call must fail closed"
            );
            assert_eq!(calls.load(Ordering::SeqCst), 0);
            assert!(!root.path().join(".git/index").exists());
        }
        let _ = client.cancel().await;
        task.await.expect("joined");
    }
}

#[tokio::test]
async fn registered_mcp_git_add_commit_checkout_and_push_execute_after_bound_auto_review() {
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use talos_agent::auto_resolver::{
        AutoPermissionAssessor, AutoPermissionControl, AutoPermissionRequest,
        AutoPermissionResolver, ManagedWorkspaceLease, PreparedCommandAssessmentRequest,
    };
    use talos_agent::permission_pipeline::{
        ApprovalResolver, ApprovalResolverError, PermissionApprovalRequest, PermissionPipeline,
    };
    use talos_core::ApprovalChoice;
    use talos_permission::{
        InteractionCapability, PermissionContext, PermissionMode, PermissionSessionState,
    };

    struct Assessor {
        calls: Arc<AtomicUsize>,
        tool: &'static str,
        operation: &'static str,
        intent: &'static str,
        fail: bool,
    }
    #[async_trait]
    impl AutoPermissionAssessor for Assessor {
        async fn assess(&self, _: AutoPermissionRequest, _: Duration) -> Result<String, String> {
            panic!("Git mutation must use operation-bound review")
        }
        async fn assess_prepared_command(
            &self,
            request: PreparedCommandAssessmentRequest,
            _: Duration,
        ) -> Result<String, String> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            assert_eq!(request.tool, self.tool);
            assert_eq!(request.command.operation, self.operation);
            assert_eq!(request.user_intent, self.intent);
            if self.fail {
                return Err("injected operation assessor failure".into());
            }
            Ok(json!({
                "schema": request.schema,
                "request_digest": request.request_digest,
                "decision": "allow_once",
                "confidence": "high",
                "acknowledged_effects": request.command.effects,
                "accepts_unconfined_effects": true
            })
            .to_string())
        }
    }
    struct ApproveDiscoveryDenyCommand(&'static str);
    #[async_trait]
    impl ApprovalResolver for ApproveDiscoveryDenyCommand {
        async fn resolve(
            &self,
            request: PermissionApprovalRequest,
            _: Duration,
        ) -> Result<ApprovalChoice, ApprovalResolverError> {
            Ok(
                if request
                    .preview
                    .facets()
                    .iter()
                    .any(|facet| facet.normalized_scope.contains(self.0))
                {
                    ApprovalChoice::Deny
                } else {
                    ApprovalChoice::ApproveOnce
                },
            )
        }
    }

    tokio::time::timeout(Duration::from_secs(120), async {
        for (name, operation, scope, intent) in [
            (
                "git_add",
                "git:add-command/v1",
                "git:add-command",
                "stage selected file in this repository",
            ),
            (
                "git_commit",
                "git:commit-command/v1",
                "git:commit-command",
                "commit staged change in this repository",
            ),
            (
                "git_checkout",
                "git:checkout-command/v1",
                "git:checkout-command",
                "switch this repository to the target branch",
            ),
            (
                "git_push",
                "git:push-command/v1",
                "git:push-command",
                "push this branch to the configured receiver",
            ),
        ] {
            for (explicit_deny, assessor_failure) in [(false, false), (true, false), (false, true)]
            {
                let temp = tempfile::tempdir().expect("fixture");
                let repository = temp.path().join("work");
                std::fs::create_dir(&repository).expect("repository");
                let git = |args: &[&str]| {
                    let output = std::process::Command::new("git")
                        .args(args)
                        .current_dir(&repository)
                        .output()
                        .expect("fixture git");
                    assert!(
                        output.status.success(),
                        "{args:?}: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                    output.stdout
                };
                git(&["init", "--template=", "--initial-branch=main"]);
                git(&["config", "user.name", "Fixture"]);
                git(&["config", "user.email", "fixture@example.invalid"]);
                git(&["config", "commit.gpgSign", "false"]);
                git(&["config", "maintenance.auto", "false"]);
                std::fs::write(repository.join("tracked.txt"), "initial\n").expect("tracked");
                git(&["add", "tracked.txt"]);
                git(&["commit", "-m", "initial"]);

                let input = match name {
                    "git_add" => {
                        std::fs::write(repository.join("selected.txt"), "selected\n")
                            .expect("selected");
                        json!({"paths": ["selected.txt"]})
                    }
                    "git_commit" => {
                        std::fs::write(repository.join("selected.txt"), "committed\n")
                            .expect("selected");
                        git(&["add", "selected.txt"]);
                        json!({"message": "operation-bound MCP commit"})
                    }
                    "git_checkout" => {
                        git(&["branch", "target"]);
                        json!({"branch": "target"})
                    }
                    "git_push" => {
                        let receiver = temp.path().join("receiver.git");
                        let output = std::process::Command::new("git")
                            .args(["init", "--bare", receiver.to_str().expect("receiver")])
                            .current_dir(temp.path())
                            .output()
                            .expect("bare receiver");
                        assert!(output.status.success(), "{:?}", output);
                        git(&[
                            "remote",
                            "add",
                            "origin",
                            receiver.to_str().expect("receiver"),
                        ]);
                        json!({"branch": "main"})
                    }
                    _ => unreachable!(),
                };

                let before_head = git(&["rev-parse", "HEAD"]);
                let before_index = git(&["diff", "--cached", "--binary"]);
                let before_branch = git(&["branch", "--show-current"]);
                let receiver = temp.path().join("receiver.git");
                let mut engine = PermissionEngine::with_workspace_root(repository.clone());
                if explicit_deny {
                    engine.add_rule(PermissionRule::new_nature(
                        ToolNature::Execute,
                        Some(scope.into()),
                        None,
                        PermissionDecision::Deny("explicit operation denial".into()),
                    ));
                }
                let state = Arc::new(PermissionSessionState::new(engine));
                let calls = Arc::new(AtomicUsize::new(0));
                let resolver = AutoPermissionResolver::new(
                    Arc::new(Assessor {
                        calls: calls.clone(),
                        tool: name,
                        operation,
                        intent,
                        fail: assessor_failure,
                    }),
                    Arc::new(ApproveDiscoveryDenyCommand(scope)),
                    ManagedWorkspaceLease::for_permission_session(&repository, state.clone())
                        .expect("lease"),
                    Duration::from_secs(10),
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
                let gate =
                    McpPermissionGate::with_pipeline(pipeline, Arc::new(HookRegistry::new()))
                        .with_user_intent(intent.into());
                let mut registry = ToolRegistry::new();
                registry
                    .register_builtin_contributions(talos_tools::git_mutation_tool_contributions(
                        repository.clone(),
                    ))
                    .expect("native Git registration");
                assert!(registry.is_builtin_instance(name));
                let handler = TalosMcpHandler::new(Arc::new(registry), Arc::new(gate));
                let (client_io, server_io) = tokio::io::duplex(65536);
                let server = tokio::spawn(async move {
                    let running = handler.serve(server_io).await.expect("server");
                    let _ = running.waiting().await;
                });
                let client = ().serve(client_io).await.expect("client");
                let args = input.as_object().expect("arguments").clone();
                let result = client
                    .peer()
                    .call_tool(rmcp::model::CallToolRequestParams::new(name).with_arguments(args))
                    .await;
                if explicit_deny || assessor_failure {
                    let error = result.expect_err("denied admission must reject the operation");
                    assert!(
                        error.to_string().contains("permission denied"),
                        "{name}: {error}"
                    );
                    assert_eq!(
                        calls.load(Ordering::SeqCst),
                        usize::from(assessor_failure),
                        "{name}: unexpected assessor calls"
                    );
                    assert_eq!(git(&["rev-parse", "HEAD"]), before_head, "{name}");
                    assert_eq!(
                        git(&["diff", "--cached", "--binary"]),
                        before_index,
                        "{name}"
                    );
                    assert_eq!(git(&["branch", "--show-current"]), before_branch, "{name}");
                    if name == "git_push" {
                        assert!(
                            !receiver.join("refs/heads/main").exists(),
                            "push changed remote ref"
                        );
                    }
                } else {
                    let result = result.expect("transport");
                    assert_ne!(result.is_error, Some(true), "{name}: {result:?}");
                    assert!(
                        calls.load(Ordering::SeqCst) > 0,
                        "{name}: Auto assessor not reached"
                    );
                    match name {
                        "git_add" => assert_eq!(git(&["show", ":selected.txt"]), b"selected\n"),
                        "git_commit" => assert_eq!(
                            String::from_utf8(git(&["log", "-1", "--format=%s"]))
                                .expect("subject")
                                .trim(),
                            "operation-bound MCP commit",
                        ),
                        "git_checkout" => assert_eq!(
                            String::from_utf8(git(&["branch", "--show-current"]))
                                .expect("branch")
                                .trim(),
                            "target",
                        ),
                        "git_push" => {
                            let output = std::process::Command::new("git")
                                .args([
                                    "--git-dir",
                                    receiver.to_str().expect("receiver"),
                                    "rev-parse",
                                    "refs/heads/main",
                                ])
                                .output()
                                .expect("remote ref");
                            assert!(output.status.success(), "{:?}", output);
                            assert_eq!(output.stdout, git(&["rev-parse", "HEAD"]));
                        }
                        _ => unreachable!(),
                    }
                }
                assert_eq!(state.grant_count().expect("grant count"), 0, "{name}");
                let _ = client.cancel().await;
                server.await.expect("server joined");
            }
        }
    })
    .await
    .expect("Git MCP test deadline");
}

#[tokio::test]
async fn real_git_transport_write_denial_and_replay_preserve_index() {
    tokio::time::timeout(Duration::from_secs(30), async {
        for allow_write in [false, true] {
            let root = tempfile::tempdir().expect("workspace");
            let git = |args: &[&str]| {
                let out = std::process::Command::new("git").args(args)
                    .current_dir(root.path()).output().expect("Git fixture");
                assert!(out.status.success(), "{out:?}");
                out.stdout
            };
            git(&["init"]);
            std::fs::write(root.path().join("unrelated.txt"), "staged before\n").expect("fixture");
            git(&["add", "unrelated.txt"]);
            let before = std::fs::read(root.path().join(".git/index")).expect("index");
            std::fs::write(root.path().join("unrelated.txt"), "unstaged after\n").expect("fixture");
            std::fs::write(root.path().join("selected.txt"), "first\n").expect("fixture");
            let mut engine = PermissionEngine::empty();
            engine.set_workspace_root(root.path().to_path_buf());
            for nature in [ToolNature::Read, ToolNature::Execute, ToolNature::Write] {
                engine.add_rule(PermissionRule::new_nature(nature, Some("*".into()), None,
                    if nature == ToolNature::Write && !allow_write { PermissionDecision::Deny("fixture write denial".into()) }
                    else { PermissionDecision::Allow }));
            }
            let mut registry = ToolRegistry::new();
            registry.register(Arc::new(talos_tools::git::GitAddTool::new(root.path().to_path_buf())));
            let handler = TalosMcpHandler::new(Arc::new(registry), Arc::new(McpPermissionGate::new(
                Arc::new(engine), Arc::new(HookRegistry::new()))));
            let (client, server) = tokio::io::duplex(65536);
            let task = tokio::spawn(async move {
                handler.serve(server).await.expect("handshake").waiting().await.expect("shutdown");
            });
            let (read, mut write) = tokio::io::split(client);
            let mut read = BufReader::new(read);
            let init = json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{
                "protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"git-test","version":"1"}}});
            write.write_all(format!("{init}\n").as_bytes()).await.expect("init");
            let mut line = String::new();
            read.read_line(&mut line).await.expect("initialize response");
            assert!(serde_json::from_str::<Value>(&line).expect("json").get("result").is_some());
            write.write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n").await.expect("initialized");
            for (attempt, id) in [1, 1, 2].into_iter().enumerate() {
                let call = json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{
                    "name":"git_add","arguments":{"paths":["selected.txt"]}}});
                write.write_all(format!("{call}\n").as_bytes()).await.expect("call");
                line.clear();
                read.read_line(&mut line).await.expect("response");
                let response: Value = serde_json::from_str(&line).expect("json");
                assert_eq!(response["id"], id);
                if allow_write && attempt == 0 {
                    assert!(response.get("result").is_some(), "{response}");
                    assert_ne!(response["result"]["isError"], true, "{response}");
                    assert_eq!(git(&["show", ":selected.txt"]), b"first\n");
                    std::fs::write(root.path().join("selected.txt"), "second\n").expect("edit");
                } else if allow_write && id == 1 {
                    assert!(response["error"]["message"].as_str().expect("error").contains("already used"));
                    assert_eq!(git(&["show", ":selected.txt"]), b"first\n");
                } else if allow_write {
                    assert!(response.get("result").is_some(), "{response}");
                    assert_ne!(response["result"]["isError"], true, "{response}");
                    assert_eq!(git(&["show", ":selected.txt"]), b"second\n");
                } else {
                    assert_eq!(response["id"], id);
                    let message = response["error"]["message"].as_str().expect("permission error");
                    assert!(message.contains(if attempt == 1 { "already used" } else { "permission denied: approval denied" }), "{response}");
                    assert_eq!(std::fs::read(root.path().join(".git/index")).expect("index"), before);
                }
                assert_eq!(git(&["show", ":unrelated.txt"]), b"staged before\n");
            }
            drop(read);
            drop(write);
            task.await.expect("server task");
        }
    }).await.expect("deadline");
}

#[tokio::test]
async fn real_commit_transport_requires_write_admission_and_never_replays() {
    tokio::time::timeout(Duration::from_secs(30), async {
        for allowed in [false, true] {
            let root = tempfile::tempdir().expect("workspace");
            let git = |args: &[&str]| {
                std::process::Command::new("git").args(args).current_dir(root.path()).output().expect("git")
            };
            for args in [vec!["init"], vec!["config", "user.name", "Fixture"], vec!["config", "user.email", "fixture@example.invalid"], vec!["config", "commit.gpgSign", "false"]] {
                assert!(git(&args).status.success());
            }
            std::fs::write(root.path().join("file"), "staged").expect("file");
            assert!(git(&["add", "file"]).status.success());
            let index = std::fs::read(root.path().join(".git/index")).expect("index");
            std::fs::write(root.path().join("file"), "unstaged").expect("edit");
            let mut engine = PermissionEngine::empty();
            engine.set_workspace_root(root.path().into());
            for nature in [ToolNature::Read, ToolNature::Execute, ToolNature::Write] {
                engine.add_rule(PermissionRule::new_nature(nature, Some("*".into()), None,
                    if nature == ToolNature::Write && !allowed { PermissionDecision::Deny("fixture denial".into()) } else { PermissionDecision::Allow }));
            }
            let mut registry = ToolRegistry::new();
            registry.register(Arc::new(talos_tools::git::GitCommitTool::new(root.path().into())));
            let handler = Arc::new(TalosMcpHandler::new_with_owner_lifecycle(Arc::new(registry), Arc::new(McpPermissionGate::new(Arc::new(engine), Arc::new(HookRegistry::new()))), true));
            let (client, server) = tokio::io::duplex(65536);
            let task = tokio::spawn(async move {
                handler.clone().serve(server).await.expect("handshake").waiting().await.expect("transport");
                handler.shutdown(Duration::from_secs(5)).await.expect("cleanup");
            });
            let (read, mut write) = tokio::io::split(client);
            let mut read = BufReader::new(read);
            let init = json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"commit-test","version":"1"}}});
            write.write_all(format!("{init}\n").as_bytes()).await.expect("init");
            let mut line = String::new();
            read.read_line(&mut line).await.expect("response");
            assert!(serde_json::from_str::<Value>(&line).expect("json").get("result").is_some());
            write.write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n").await.expect("initialized");
            let mut first_head = None;
            for attempt in 0..2 {
                let call = json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"git_commit","arguments":{"message":"MCP commit"}}});
                write.write_all(format!("{call}\n").as_bytes()).await.expect("call");
                line.clear();
                read.read_line(&mut line).await.expect("response");
                let response: Value = serde_json::from_str(&line).expect("json");
                if attempt == 1 {
                    assert!(response["error"]["message"].as_str().expect("replay error").contains("already used"), "{response}");
                } else if allowed {
                    assert!(response.get("result").is_some(), "{response}");
                    assert_ne!(response["result"]["isError"], true, "{response}");
                    let head = git(&["rev-parse", "HEAD"]);
                    assert!(head.status.success());
                    first_head = Some(head.stdout);
                } else {
                    assert!(response["error"]["message"].as_str().expect("denial").contains("permission denied"), "{response}");
                }
                if allowed {
                    assert_eq!(git(&["rev-parse", "HEAD"]).stdout, *first_head.as_ref().expect("head"));
                    assert_eq!(git(&["show", "HEAD:file"]).stdout, b"staged");
                } else {
                    assert!(!git(&["rev-parse", "--verify", "HEAD"]).status.success());
                    assert_eq!(std::fs::read(root.path().join(".git/index")).expect("index"), index);
                    assert!(!root.path().join(".git/COMMIT_EDITMSG").exists());
                }
                assert_eq!(std::fs::read_to_string(root.path().join("file")).expect("worktree"), "unstaged");
            }
            drop(read); drop(write);
            task.await.expect("joined");
        }
    }).await.expect("test deadline");
}

#[tokio::test]
async fn real_push_transport_network_denial_never_connects_or_replays() {
    tokio::time::timeout(Duration::from_secs(30), async {
        let root = tempfile::tempdir().expect("workspace");
        assert!(std::process::Command::new("git").arg("init").current_dir(root.path()).output().expect("git").status.success());
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("listener");
        listener.set_nonblocking(true).expect("nonblocking");
        let remote = format!("git://{}/repository", listener.local_addr().expect("address"));
        let mut engine = PermissionEngine::empty();
        engine.set_workspace_root(root.path().into());
        for nature in [ToolNature::Read, ToolNature::Execute, ToolNature::Write, ToolNature::Network] {
            engine.add_rule(PermissionRule::new_nature(nature, Some("*".into()), None,
                if nature == ToolNature::Network { PermissionDecision::Deny("network fixture denial".into()) } else { PermissionDecision::Allow }));
        }
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(talos_tools::git::GitPushTool::new(root.path().into())));
        let handler = Arc::new(TalosMcpHandler::new_with_owner_lifecycle(Arc::new(registry), Arc::new(McpPermissionGate::new(Arc::new(engine), Arc::new(HookRegistry::new()))), true));
        let (client, server) = tokio::io::duplex(65536);
        let task = tokio::spawn(async move {
            handler.clone().serve(server).await.expect("handshake").waiting().await.expect("transport");
            handler.shutdown(Duration::from_secs(5)).await.expect("cleanup");
        });
        let (read, mut write) = tokio::io::split(client);
        let mut read = BufReader::new(read);
        let init = json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"push-test","version":"1"}}});
        write.write_all(format!("{init}\n").as_bytes()).await.expect("init");
        let mut line = String::new();
        read.read_line(&mut line).await.expect("response");
        assert!(serde_json::from_str::<Value>(&line).expect("json").get("result").is_some());
        write.write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n").await.expect("initialized");
        for attempt in 0..2 {
            let call = json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"git_push","arguments":{"remote":remote}}});
            write.write_all(format!("{call}\n").as_bytes()).await.expect("call");
            line.clear();
            read.read_line(&mut line).await.expect("response");
            let response: Value = serde_json::from_str(&line).expect("json");
            let message = response["error"]["message"].as_str().expect("denied");
            assert!(message.contains(if attempt == 0 { "permission denied" } else { "already used" }), "{response}");
            assert_eq!(listener.accept().expect_err("no network connection").kind(), std::io::ErrorKind::WouldBlock);
            assert!(!root.path().join(".git/index").exists());
        }
        drop(read); drop(write);
        task.await.expect("joined");
    }).await.expect("deadline");
}

#[tokio::test]
async fn real_push_transport_success_and_replay_preserve_remote_head() {
    async {
        let root = tempfile::tempdir().expect("workspace");
        let receiver = tempfile::tempdir().expect("receiver");
        let git = |cwd: &std::path::Path, args: &[&str]| {
            let output = std::process::Command::new("git").current_dir(cwd).args(args).output().expect("git");
            assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
            output.stdout
        };
        git(root.path(), &["init", "-b", "main"]);
        #[cfg(windows)]
        git(root.path(), &["config", "sendpack.sideband", "false"]);
        git(receiver.path(), &["init", "--bare"]);
        for (key, value) in [("user.name", "Fixture"), ("user.email", "fixture@example.invalid"), ("commit.gpgSign", "false")] {
            git(root.path(), &["config", key, value]);
        }
        git(root.path(), &["commit", "--allow-empty", "-m", "first"]);
        let expected = git(root.path(), &["rev-parse", "HEAD"]);
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("port");
        let port = listener.local_addr().expect("address").port();
        drop(listener);
        git_daemon::with_daemon(receiver.path(), port, async {
        let remote = format!("git://127.0.0.1:{port}/{}", receiver.path().file_name().expect("name").to_string_lossy());
        let mut engine = PermissionEngine::empty();
        engine.set_workspace_root(root.path().into());
        for nature in [ToolNature::Read, ToolNature::Execute, ToolNature::Write, ToolNature::Network] {
            engine.add_rule(PermissionRule::new_nature(nature, Some("*".into()), None, PermissionDecision::Allow));
        }
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(talos_tools::git::GitPushTool::new(root.path().into())));
        let handler = Arc::new(TalosMcpHandler::new_with_owner_lifecycle(Arc::new(registry), Arc::new(McpPermissionGate::new(Arc::new(engine), Arc::new(HookRegistry::new()))), true));
        let (client, server) = tokio::io::duplex(65536);
        let task = tokio::spawn(async move {
            handler.clone().serve(server).await.expect("handshake").waiting().await.expect("transport");
            handler.shutdown(Duration::from_secs(5)).await.expect("cleanup");
        });
        let (read, mut write) = tokio::io::split(client);
        let mut read = BufReader::new(read);
        let init = json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"push-success","version":"1"}}});
        write.write_all(format!("{init}\n").as_bytes()).await.expect("init");
        let mut line = String::new();
        read.read_line(&mut line).await.expect("response");
        assert!(serde_json::from_str::<Value>(&line).expect("json").get("result").is_some());
        write.write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n").await.expect("initialized");
        for attempt in 0..2 {
            let call = json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"git_push","arguments":{"remote":remote,"branch":"HEAD:refs/heads/main"}}});
            write.write_all(format!("{call}\n").as_bytes()).await.expect("call");
            line.clear();
            read.read_line(&mut line).await.expect("response");
            let response: Value = serde_json::from_str(&line).expect("json");
            if attempt == 0 {
                assert!(response.get("result").is_some() && response["result"]["isError"] != true, "{response}");
                git(root.path(), &["commit", "--allow-empty", "-m", "must not replay"]);
            } else {
                assert!(response["error"]["message"].as_str().expect("replay").contains("already used"), "{response}");
            }
            assert_eq!(git(receiver.path(), &["rev-parse", "refs/heads/main"]), expected);
        }
        drop(read); drop(write);
        task.await.expect("joined");
        }).await;
    }.await;
}

#[cfg(unix)]
#[tokio::test]
async fn real_local_push_transport_success_denial_and_replay() {
    tokio::time::timeout(Duration::from_secs(30), async {
        for allowed in [false, true] {
        let root = tempfile::tempdir().expect("workspace");
        let receiver = tempfile::tempdir().expect("receiver");
        let git = |cwd: &std::path::Path, args: &[&str]| {
            let output = std::process::Command::new("git").current_dir(cwd).args(args).output().expect("git");
            assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
            output.stdout
        };
        git(root.path(), &["init", "-b", "main"]);
        git(receiver.path(), &["init", "--bare"]);
        for (key, value) in [("user.name", "Fixture"), ("user.email", "fixture@example.invalid"), ("commit.gpgSign", "false")] {
            git(root.path(), &["config", key, value]);
        }
        git(root.path(), &["commit", "--allow-empty", "-m", "first"]);
        let expected = git(root.path(), &["rev-parse", "HEAD"]);
        let remote = receiver.path().to_string_lossy().into_owned();
        let mut engine = PermissionEngine::empty();
        engine.set_workspace_root(root.path().into());
        for nature in [ToolNature::Read, ToolNature::Execute, ToolNature::Write, ToolNature::Network] {
            engine.add_rule(PermissionRule::new_nature(nature, Some("*".into()), None, if nature == ToolNature::Write && !allowed { PermissionDecision::Deny("fixture receiver write denial".into()) } else { PermissionDecision::Allow }));
        }
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(talos_tools::git::GitPushTool::new(root.path().into())));
        let handler = Arc::new(TalosMcpHandler::new_with_owner_lifecycle(Arc::new(registry), Arc::new(McpPermissionGate::new(Arc::new(engine), Arc::new(HookRegistry::new()))), true));
        let (client, server) = tokio::io::duplex(65536);
        let task = tokio::spawn(async move {
            handler.clone().serve(server).await.expect("handshake").waiting().await.expect("transport");
            handler.shutdown(Duration::from_secs(5)).await.expect("cleanup");
        });
        let (read, mut write) = tokio::io::split(client);
        let mut read = BufReader::new(read);
        let init = json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"push-success","version":"1"}}});
        write.write_all(format!("{init}\n").as_bytes()).await.expect("init");
        let mut line = String::new();
        read.read_line(&mut line).await.expect("response");
        assert!(serde_json::from_str::<Value>(&line).expect("json").get("result").is_some());
        write.write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n").await.expect("initialized");
        for attempt in 0..2 {
            let call = json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"git_push","arguments":{"remote":remote,"branch":"HEAD:refs/heads/main"}}});
            write.write_all(format!("{call}\n").as_bytes()).await.expect("call");
            line.clear();
            read.read_line(&mut line).await.expect("response");
            let response: Value = serde_json::from_str(&line).expect("json");
            if attempt == 0 && allowed {
                assert!(response.get("result").is_some() && response["result"]["isError"] != true, "{response}");
                git(root.path(), &["commit", "--allow-empty", "-m", "must not replay"]);
            } else {
                assert!(response["error"]["message"].as_str().expect("denial or replay").contains(if attempt == 0 { "permission denied" } else { "already used" }), "{response}");
            }
            if allowed { assert_eq!(git(receiver.path(), &["rev-parse", "refs/heads/main"]), expected); }
            else { assert!(!receiver.path().join("refs/heads/main").exists()); assert!(!receiver.path().join("objects/pack").read_dir().expect("pack").any(|entry| entry.is_ok())); }
        }
        drop(read); drop(write);
        task.await.expect("joined");
        }
    }).await.expect("deadline");
}
