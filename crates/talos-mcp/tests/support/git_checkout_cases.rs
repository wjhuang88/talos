//! Independent MCP Checkout cases; each keeps a bounded operation deadline.
use super::*;

macro_rules! checkout_case {
    ($name:ident, $allowed:expr, $kind:expr) => {
        #[tokio::test]
        async fn $name() {
            run_case($allowed, $kind).await;
        }
    };
}

checkout_case!(case_00_denied, false, 0);
checkout_case!(case_00_approved, true, 0);
checkout_case!(case_01_denied, false, 1);
checkout_case!(case_01_approved, true, 1);
checkout_case!(case_02_denied, false, 2);
checkout_case!(case_02_approved, true, 2);
#[cfg(unix)]
checkout_case!(case_03_denied, false, 3);
#[cfg(unix)]
checkout_case!(case_03_approved, true, 3);
checkout_case!(case_04_denied, false, 4);
checkout_case!(case_04_approved, true, 4);
checkout_case!(case_05_denied, false, 5);
checkout_case!(case_05_approved, true, 5);
checkout_case!(case_06_approved, true, 6);
checkout_case!(case_07_denied, false, 7);
checkout_case!(case_07_approved, true, 7);
checkout_case!(case_08_denied, false, 8);
checkout_case!(case_08_approved, true, 8);
checkout_case!(case_09_denied, false, 9);
checkout_case!(case_09_approved, true, 9);
checkout_case!(case_10_denied, false, 10);
checkout_case!(case_10_approved, true, 10);
checkout_case!(case_11_denied, false, 11);
checkout_case!(case_11_approved, true, 11);
checkout_case!(case_12_denied, false, 12);
checkout_case!(case_13_denied, false, 13);
checkout_case!(case_13_approved, true, 13);
#[cfg(unix)]
checkout_case!(case_14_denied, false, 14);
#[cfg(unix)]
checkout_case!(case_14_approved, true, 14);
#[cfg(unix)]
checkout_case!(case_15_denied, false, 15);
#[cfg(unix)]
checkout_case!(case_15_approved, true, 15);
#[cfg(unix)]
checkout_case!(case_16_denied, false, 16);
#[cfg(unix)]
checkout_case!(case_17_denied, false, 17);
#[cfg(unix)]
checkout_case!(case_17_approved, true, 17);
#[cfg(unix)]
checkout_case!(case_18_denied, false, 18);
#[cfg(unix)]
checkout_case!(case_18_approved, true, 18);
#[cfg(unix)]
checkout_case!(case_19_denied, false, 19);
#[cfg(unix)]
checkout_case!(case_19_approved, true, 19);
#[cfg(unix)]
checkout_case!(case_20_denied, false, 20);
#[cfg(unix)]
checkout_case!(case_21_denied, false, 21);
#[cfg(unix)]
checkout_case!(case_21_approved, true, 21);

async fn run_case(allowed: bool, kind: u8) {
    tokio::time::timeout(Duration::from_secs(30), async {
        let initialized = kind >= 18;
        let nonempty = (14..18).contains(&kind);
        let kind = if initialized { kind - 8 } else if nonempty { kind - 4 } else { kind };
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
    }).await.expect("Checkout case deadline");
}
