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

#[tokio::test]
async fn real_checkout_transport_denial_success_and_replay() {
    tokio::time::timeout(Duration::from_secs(30), async {
        for (allowed, kind) in [(false, 0), (true, 0), (false, 1), (true, 1), (false, 2), (true, 2), (false, 3), (true, 3), (false, 4), (true, 4), (false, 5), (true, 5), (true, 6), (false, 7), (true, 7), (false, 8), (true, 8), (false, 9), (true, 9), (false, 10), (true, 10), (false, 11), (true, 11), (false, 12), (false, 13), (true, 13), (false, 14), (true, 14), (false, 15), (true, 15), (false, 16), (false, 17), (true, 17), (false, 18), (true, 18), (false, 19), (true, 19), (false, 20), (false, 21), (true, 21)] {
            let initialized = kind >= 18;
            let nonempty = (14..18).contains(&kind);
            if (nonempty || initialized) && !cfg!(unix) { continue; }
            let kind = if initialized { kind - 8 } else if nonempty { kind - 4 } else { kind };
            if kind == 3 && !cfg!(unix) { continue; }
            let case_started = std::time::Instant::now();
            eprintln!("checkout case start: allowed={allowed}, kind={kind}, initialized={initialized}, nonempty={nonempty}");
            let gitlink = !matches!(kind, 0 | 7 | 8);
            let reverse = matches!(kind, 2 | 4 | 5 | 6 | 9);
            let new_directory = kind >= 10;
            let stage_limited = kind == 6 || (initialized && kind == 13);
            let succeeds = allowed && !stage_limited;
            let root = tempfile::tempdir().expect("workspace");
            let external = tempfile::tempdir().expect("external referent");
            let referent = external.path().join("sentinel");
            std::fs::write(&referent, "external sentinel").expect("sentinel");
            let git = |args: &[&str]| {
                let output = std::process::Command::new("git").args(args).current_dir(root.path()).output().expect("git");
                assert!(output.status.success(), "{output:?}");
                output.stdout
            };
            git(&["init", "-b", "base"]);
            git(&["config", "user.name", "Fixture"]);
            git(&["config", "user.email", "fixture@example.invalid"]);
            git(&["config", "commit.gpgSign", "false"]);
            if kind == 3 {
                #[cfg(unix)]
                std::os::unix::fs::symlink(&referent, root.path().join("file")).expect("link");
            } else if !new_directory {
                std::fs::write(root.path().join("file"), "base").expect("file");
            }
            if !new_directory { git(&["add", "file"]); }
            git(&["commit", "--allow-empty", "-m", "base"]);
            if kind == 13 {
                let oid = String::from_utf8(git(&["rev-parse", "HEAD"])).expect("base OID");
                git(&["update-index", "--add", "--cacheinfo", "160000", oid.trim(), "old"]);
                git(&["commit", "-m", "old gitlink"]);
            }
            git(&["checkout", "-b", "target"]);
            if gitlink {
                let oid = String::from_utf8(git(&["rev-parse", "HEAD"])).expect("oid");
                git(&["update-index", "--add", "--cacheinfo", "160000", oid.trim(), "file"]);
                if kind == 13 {
                    git(&["update-index", "--force-remove", "old"]);
                    std::fs::write(root.path().join("old"), "restored ordinary file").expect("target old file");
                    git(&["add", "old"]);
                }
                git(&["commit", "-m", "gitlink target"]);
            } else {
                std::fs::write(root.path().join("file"), "target").expect("target file");
                git(&["commit", "-am", "target"]);
            }
            git(&["checkout", "base"]);
            if new_directory { std::fs::create_dir_all(root.path().join("file")).expect("new empty directory"); }
            if nonempty {
                std::fs::write(root.path().join("file/sentinel"), "retained descendant").expect("nonempty child");
                std::fs::create_dir_all(root.path().join("file/deep/nested")).expect("deep descendant");
            }
            let child_head = if initialized {
                git(&["-C", "file", "init"]);
                git(&["-C", "file", "config", "user.name", "Child"]);
                git(&["-C", "file", "config", "user.email", "child@example.invalid"]);
                std::fs::write(root.path().join("file/tracked"), "retained tracked child").expect("child source");
                git(&["-C", "file", "add", "tracked"]);
                git(&["-C", "file", "-c", "commit.gpgSign=false", "commit", "-m", "child"]);
                Some(std::fs::read(root.path().join("file/.git/HEAD")).expect("child HEAD"))
            } else { None };
            if kind == 13 {
                std::fs::create_dir_all(root.path().join("old")).expect("old gitlink directory");
                std::fs::write(root.path().join("old/sentinel"), "explicit deletion").expect("old child");
            }
            if matches!(kind, 7 | 11) {
                for branch in ["base", "target"] {
                    let config = external.path().join(format!("{branch}.config"));
                    std::fs::write(&config, format!("[fixture]\n marker = {branch}\n")).expect("conditional config");
                    git(&["config", &format!("includeIf.onbranch:{branch}.path"), config.to_str().expect("config path")]);
                }
            }
            #[cfg(unix)]
            let mut _socket_fixture = None;
            if reverse {
                git(&["checkout", "target"]);
                if kind == 2 {
                    std::fs::remove_dir(root.path().join("file")).expect("missing uninitialized gitlink");
                } else {
                    if kind == 5 {
                        git(&["-C", "file", "init", "--separate-git-dir", external.path().join("child.git").to_str().expect("metadata path")]);
                        git(&["-C", "file", "config", "user.name", "Child"]);
                        git(&["-C", "file", "config", "user.email", "child@example.invalid"]);
                        git(&["-C", "file", "-c", "commit.gpgSign=false", "commit", "--allow-empty", "-m", "child"]);
                    }
                    std::fs::write(root.path().join("file/untracked"), "explicit deletion").expect("child content");
                    #[cfg(unix)]
                    if kind == 4 {
                        _socket_fixture = Some(std::os::unix::net::UnixListener::bind(root.path().join("file/socket")).expect("socket fixture"));
                        assert!(std::process::Command::new("mkfifo")
                            .arg(root.path().join("file/pipe"))
                            .status().expect("FIFO fixture").success());
                    }
                    if kind == 6 {
                        let deep = root.path().join("file/a/b/c/d/e/f/g/h/i");
                        std::fs::create_dir_all(&deep).expect("deep child");
                        std::fs::write(deep.join("sentinel"), "preserved deep child").expect("deep sentinel");
                    }
                }
            }
            if matches!(kind, 8 | 9) {
                git(&["config", "remote.origin.url", "."]);
                git(&["config", "remote.origin.fetch", "+refs/heads/*:refs/remotes/origin/*"]);
                let oid = String::from_utf8(git(&["rev-parse", if reverse { "base" } else { "target" }])).expect("target OID");
                git(&["update-ref", "refs/remotes/origin/topic", oid.trim()]);
                git(&["config", "includeIf.onbranch:topic.path", "target.conf"]);
                std::fs::write(root.path().join(".git/target.conf"), "[fixture]\nmarker=topic\n").expect("target config");
            }
            let target_branch = if matches!(kind, 8 | 9) { "topic" } else if reverse { "base" } else { "target" };
            let before = std::fs::read(root.path().join(".git/index")).expect("index");
            let mut engine = PermissionEngine::empty();
            engine.set_workspace_root(root.path().into());
            if kind == 12 {
                engine.add_rule(PermissionRule::new_nature(ToolNature::Read,
                    Some(std::fs::canonicalize(root.path().join("file")).expect("directory identity").to_str().expect("directory path").into()),
                    None, PermissionDecision::Deny("directory Read denied".into())));
            }
            for nature in [ToolNature::Read, ToolNature::Execute, ToolNature::Write] {
                engine.add_rule(PermissionRule::new_nature(nature, Some("*".into()), None,
                    if nature == ToolNature::Write && !allowed && kind != 12 { PermissionDecision::Deny("fixture denial".into()) } else { PermissionDecision::Allow }));
            }
            let mut registry = ToolRegistry::new();
            registry.register(Arc::new(talos_tools::git::GitCheckoutTool::new(root.path().into())));
            let handler = Arc::new(TalosMcpHandler::new_with_owner_lifecycle(Arc::new(registry), Arc::new(McpPermissionGate::new(Arc::new(engine), Arc::new(HookRegistry::new()))), true));
            let (client, server) = tokio::io::duplex(65536);
            let task = tokio::spawn(async move {
                handler.clone().serve(server).await.expect("handshake").waiting().await.expect("transport");
                handler.shutdown(Duration::from_secs(5)).await.expect("cleanup");
            });
            let (read, mut write) = tokio::io::split(client);
            let mut read = BufReader::new(read);
            let init = json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"checkout-test","version":"1"}}});
            write.write_all(format!("{init}\n").as_bytes()).await.expect("init");
            let mut line = String::new();
            read.read_line(&mut line).await.expect("response");
            assert!(serde_json::from_str::<Value>(&line).expect("json").get("result").is_some());
            write.write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n").await.expect("initialized");
            for attempt in 0..2 {
                let call = json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"git_checkout","arguments":{"branch":target_branch}}});
                write.write_all(format!("{call}\n").as_bytes()).await.expect("call");
                line.clear();
                read.read_line(&mut line).await.expect("response");
                let response: Value = serde_json::from_str(&line).expect("json");
                if attempt == 1 {
                    assert!(response["error"]["message"].as_str().expect("replay").contains("already used"), "{response}");
                } else if stage_limited {
                    assert!(response["error"]["message"].as_str().expect("budget refusal").contains("prepared stage limit"), "{response}");
                } else if allowed {
                    assert!(response.get("result").is_some() && response["result"]["isError"] != true, "kind={kind}, initialized={initialized}: {response}");
                } else {
                    assert!(response["error"]["message"].as_str().is_some_and(|message| message.contains("permission denied")), "kind={kind}, initialized={initialized}, nonempty={nonempty}: {response}");
                }
                let expected = if succeeds { target_branch } else if reverse { "target" } else { "base" };
                assert_eq!(git(&["branch", "--show-current"]), format!("{expected}\n").as_bytes());
                if matches!(kind, 8 | 9) {
                    assert_eq!(root.path().join(".git/refs/heads/topic").exists(), succeeds);
                    assert_eq!(std::fs::read_to_string(root.path().join(".git/target.conf")).expect("target config retained"), "[fixture]\nmarker=topic\n");
                    if succeeds {
                        assert_eq!(git(&["config", "fixture.marker"]), b"topic\n");
                    }
                }
                if matches!(kind, 7 | 11) {
                    assert_eq!(git(&["config", "fixture.marker"]), format!("{expected}\n").as_bytes());
                    for branch in ["base", "target"] {
                        assert_eq!(std::fs::read_to_string(external.path().join(format!("{branch}.config"))).expect("config retained"), format!("[fixture]\n marker = {branch}\n"));
                    }
                }
                if reverse {
                    if succeeds {
                        assert_eq!(std::fs::read(root.path().join("file")).expect("restored file"), b"base");
                        assert!(git(&["ls-files", "--stage", "file"]).starts_with(b"100644 "));
                    } else if kind == 2 {
                        assert!(!root.path().join("file").exists());
                    } else {
                        assert_eq!(std::fs::read(root.path().join("file/untracked")).expect("preserved untracked child"), b"explicit deletion");
                        #[cfg(unix)]
                        if kind == 4 {
                            use std::os::unix::fs::FileTypeExt;
                            assert!(std::fs::symlink_metadata(root.path().join("file/pipe"))
                                .expect("denied FIFO retained").file_type().is_fifo());
                            assert!(std::fs::symlink_metadata(root.path().join("file/socket"))
                                .expect("denied socket retained").file_type().is_socket());
                        }
                    }
                } else if new_directory {
                    if let Some(head) = &child_head {
                        assert_eq!(std::fs::read(root.path().join("file/.git/HEAD")).expect("preserved child HEAD"), *head);
                        assert_eq!(std::fs::read(root.path().join("file/tracked")).expect("preserved child source"), b"retained tracked child");
                    } else if nonempty {
                        assert_eq!(std::fs::read(root.path().join("file/sentinel")).expect("preserved descendant"), b"retained descendant");
                        assert!(root.path().join("file/deep/nested").is_dir());
                        assert_eq!(std::fs::read_dir(root.path().join("file")).expect("directory retained").count(), 2);
                    } else {
                        assert_eq!(std::fs::read_dir(root.path().join("file")).expect("empty directory retained").count(), 0);
                    }
                    let staged = git(&["ls-files", "--stage", "file"]);
                    if succeeds { assert!(staged.starts_with(b"160000 ")); }
                    else { assert!(staged.is_empty()); }
                } else if allowed && gitlink {
                    assert!(root.path().join("file").is_dir());
                    assert!(!root.path().join("file/.git").exists());
                    assert!(git(&["ls-files", "--stage", "file"]).starts_with(b"160000 "));
                } else if kind == 3 {
                    assert_eq!(std::fs::read_link(root.path().join("file")).expect("retained link"), referent);
                } else {
                    assert_eq!(std::fs::read_to_string(root.path().join("file")).expect("file"), if kind == 8 && succeeds { "target" } else { expected });
                }
                assert_eq!(std::fs::read(&referent).expect("preserved referent"), b"external sentinel");
                if kind == 13 {
                    if succeeds { assert_eq!(std::fs::read(root.path().join("old")).expect("restored old file"), b"restored ordinary file"); }
                    else { assert_eq!(std::fs::read(root.path().join("old/sentinel")).expect("old child retained"), b"explicit deletion"); }
                }
                if kind == 5 {
                    assert!(external.path().join("child.git/HEAD").is_file());
                    assert!(external.path().join("child.git/objects").is_dir());
                }
                if kind == 6 {
                    assert_eq!(std::fs::read(root.path().join("file/a/b/c/d/e/f/g/h/i/sentinel")).expect("deep sentinel retained"), b"preserved deep child");
                }
                if !succeeds { assert_eq!(std::fs::read(root.path().join(".git/index")).expect("index"), before); }
            }
            drop(read); drop(write);
            task.await.expect("joined");
            eprintln!("checkout case complete: allowed={allowed}, kind={kind}, elapsed={:?}", case_started.elapsed());
        }
    }).await.expect("test deadline");
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
