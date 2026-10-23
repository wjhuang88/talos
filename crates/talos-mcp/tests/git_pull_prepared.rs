//! Actual MCP transport coverage for the native Pull command-authority route.

use rmcp::ServiceExt;
use serde_json::{Value, json};
use std::{path::Path, sync::Arc, time::Duration};
use talos_core::tool::{ToolNature, ToolRegistry};
use talos_mcp::server::{McpPermissionGate, TalosMcpHandler};
use talos_permission::{PermissionDecision, PermissionEngine, PermissionRule};
use talos_plugin::HookRegistry;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

fn git(root: &Path, args: &[&str]) -> Vec<u8> {
    let output = std::process::Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .expect("fixture Git");
    assert!(output.status.success(), "{output:?}");
    output.stdout
}

#[tokio::test]
async fn native_pull_mcp_command_allows_real_fetch_and_denies_or_rejects_replay() {
    tokio::time::timeout(Duration::from_secs(30), async {
        for (denied, conflict) in [(true, false), (false, false), (false, true)] {
            let root = tempfile::tempdir().expect("workspace");
            git(root.path(), &["init", "-b", "main"]);
            for (key, value) in [
                ("user.name", "Fixture"),
                ("user.email", "fixture@example.invalid"),
                ("commit.gpgSign", "false"),
                ("core.autocrlf", "false"),
                ("pull.rebase", "false"),
                ("remote.origin.url", "."),
                ("remote.origin.fetch", "+refs/heads/*:refs/remotes/origin/*"),
                ("branch.main.remote", "origin"),
                ("branch.main.merge", "refs/heads/main"),
            ] {
                git(root.path(), &["config", key, value]);
            }
            git(root.path(), &["commit", "--allow-empty", "-m", "base"]);
            if conflict {
                std::fs::write(root.path().join("shared.txt"), "base\n").expect("base file");
                git(root.path(), &["add", "shared.txt"]);
                git(root.path(), &["commit", "-m", "shared base"]);
                git(root.path(), &["checkout", "-b", "upstream"]);
                std::fs::write(root.path().join("shared.txt"), "upstream\n").expect("upstream");
                git(root.path(), &["commit", "-am", "upstream change"]);
                git(root.path(), &["checkout", "main"]);
                std::fs::write(root.path().join("shared.txt"), "local\n").expect("local");
                git(root.path(), &["commit", "-am", "local change"]);
                git(
                    root.path(),
                    &["config", "branch.main.merge", "refs/heads/upstream"],
                );
            }
            let head = git(root.path(), &["rev-parse", "HEAD"]);
            let config = std::fs::read(root.path().join(".git/config")).expect("config");
            let mut engine = PermissionEngine::empty();
            engine.set_workspace_root(root.path().into());
            if denied {
                engine.add_rule(PermissionRule::new_nature(
                    ToolNature::Execute,
                    Some("git:pull-command".into()),
                    None,
                    PermissionDecision::Deny("fixture command denial".into()),
                ));
            }
            for nature in [
                ToolNature::Read,
                ToolNature::Execute,
                ToolNature::Write,
                ToolNature::Network,
            ] {
                engine.add_rule(PermissionRule::new_nature(
                    nature,
                    Some("*".into()),
                    None,
                    PermissionDecision::Allow,
                ));
            }
            let mut registry = ToolRegistry::new();
            registry.register(Arc::new(talos_tools::git::GitPullTool::new(
                root.path().into(),
            )));
            let handler = Arc::new(TalosMcpHandler::new_with_owner_lifecycle(
                Arc::new(registry),
                Arc::new(McpPermissionGate::new(
                    Arc::new(engine),
                    Arc::new(HookRegistry::new()),
                )),
                true,
            ));
            let (client, server) = tokio::io::duplex(65536);
            let task = tokio::spawn(async move {
                handler
                    .clone()
                    .serve(server)
                    .await
                    .expect("handshake")
                    .waiting()
                    .await
                    .expect("transport");
                handler
                    .shutdown(Duration::from_secs(5))
                    .await
                    .expect("cleanup");
            });
            let (read, mut write) = tokio::io::split(client);
            let mut read = BufReader::new(read);
            let init = json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{
                "protocolVersion":"2025-03-26","capabilities":{},
                "clientInfo":{"name":"pull-command-test","version":"1"}
            }});
            write
                .write_all(format!("{init}\n").as_bytes())
                .await
                .expect("init");
            let mut line = String::new();
            read.read_line(&mut line)
                .await
                .expect("initialize response");
            assert!(
                serde_json::from_str::<Value>(&line)
                    .expect("JSON")
                    .get("result")
                    .is_some()
            );
            write
                .write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n")
                .await
                .expect("initialized");
            for attempt in 0..2 {
                let call = json!({"jsonrpc":"2.0","id":1,"method":"tools/call",
                    "params":{"name":"git_pull","arguments":{}}});
                write
                    .write_all(format!("{call}\n").as_bytes())
                    .await
                    .expect("call");
                line.clear();
                read.read_line(&mut line).await.expect("Pull response");
                let response: Value = serde_json::from_str(&line).expect("JSON");
                if attempt > 0 {
                    assert!(
                        response["error"]["message"]
                            .as_str()
                            .expect("replay refusal")
                            .contains("already used"),
                        "{response}"
                    );
                } else if denied {
                    assert!(
                        response["error"]["message"]
                            .as_str()
                            .expect("permission refusal")
                            .contains("permission denied"),
                        "{response}"
                    );
                } else if conflict {
                    assert_eq!(response["result"]["isError"], true, "{response}");
                    let upstream = git(root.path(), &["rev-parse", "upstream"]);
                    let fetched = std::fs::read_to_string(root.path().join(".git/FETCH_HEAD"))
                        .expect("failed integration retains fetch effects");
                    assert!(fetched.contains(std::str::from_utf8(&upstream).expect("OID").trim()));
                    assert_eq!(
                        std::fs::read(root.path().join(".git/MERGE_HEAD")).expect("merge state"),
                        upstream
                    );
                    let unmerged = git(root.path(), &["ls-files", "--unmerged"]);
                    assert!(!unmerged.is_empty(), "conflict index must remain");
                    let content = std::fs::read_to_string(root.path().join("shared.txt"))
                        .expect("conflicted worktree");
                    assert!(content.contains("<<<<<<<") && content.contains(">>>>>>>"));
                } else {
                    assert_eq!(response["result"]["isError"], false, "{response}");
                    let fetched = std::fs::read_to_string(root.path().join(".git/FETCH_HEAD"))
                        .expect("real fetch writes FETCH_HEAD");
                    assert!(
                        fetched.starts_with(std::str::from_utf8(&head).expect("head").trim()),
                        "{fetched}"
                    );
                }
                assert_eq!(git(root.path(), &["rev-parse", "HEAD"]), head);
                assert_eq!(
                    std::fs::read(root.path().join(".git/config")).expect("config"),
                    config
                );
                assert_eq!(root.path().join(".git/FETCH_HEAD").exists(), !denied);
            }
            drop(read);
            drop(write);
            task.await.expect("joined");
        }
    })
    .await
    .expect("transport deadline");
}
