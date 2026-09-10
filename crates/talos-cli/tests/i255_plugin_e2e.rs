//! Real binary execution against an isolated local model endpoint, no external API.
use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[test]
fn explicit_capability_package_executes_through_print_permission_pipeline() {
    run_fixture(true);
}

#[test]
fn capability_registration_does_not_bypass_outside_workspace_permission() {
    run_fixture(false);
}

fn run_fixture(allowed: bool) {
    let temp = tempfile::tempdir().expect("isolated home");
    let config = temp.path().join(".talos");
    std::fs::create_dir(&config).expect("config directory");
    let listener = TcpListener::bind("127.0.0.1:0").expect("local fixture endpoint");
    listener.set_nonblocking(true).expect("nonblocking accept");
    let address = listener.local_addr().expect("address");
    std::fs::write(
        config.join("config.toml"),
        format!(
            r#"
provider = "i255-local"
model = "fixture"
[providers.i255-local]
protocol = "openai-chat"
base_url = "http://{address}/v1"
api_key = "local-test-not-a-secret"
"#
        ),
    )
    .expect("local config");
    let server = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(20);
        let mut requests = Vec::new();
        while requests.len() < 2 {
            let mut socket = loop {
                match listener.accept() {
                    Ok((socket, _)) => break socket,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(
                            Instant::now() < deadline,
                            "model fixture did not receive request {}",
                            requests.len()
                        );
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) => panic!("accept: {error}"),
                }
            };
            // Accepted sockets can inherit nonblocking mode on macOS. Restore
            // blocking reads explicitly; a read timeout still bounds bad clients.
            socket
                .set_nonblocking(false)
                .expect("blocking accepted socket");
            socket
                .set_read_timeout(Some(Duration::from_secs(5)))
                .expect("read deadline");
            let mut bytes = Vec::new();
            let mut buffer = [0u8; 8192];
            let request = loop {
                let count = socket.read(&mut buffer).expect("request bytes");
                assert!(count > 0, "incomplete HTTP request");
                bytes.extend_from_slice(&buffer[..count]);
                assert!(bytes.len() < 4 * 1024 * 1024, "bounded fixture request");
                if let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&bytes[..end]);
                    let length = headers
                        .lines()
                        .find_map(|line| {
                            line.split_once(':')
                                .filter(|(key, _)| key.eq_ignore_ascii_case("content-length"))
                                .map(|(_, value)| {
                                    value.trim().parse::<usize>().expect("content length")
                                })
                        })
                        .expect("content length header");
                    if bytes.len() >= end + 4 + length {
                        break serde_json::from_slice::<serde_json::Value>(
                            &bytes[end + 4..end + 4 + length],
                        )
                        .expect("JSON request");
                    }
                }
            };
            let delta = if requests.is_empty() {
                serde_json::json!({"tool_calls":[{"index":0,"id":"i255-call","type":"function","function":{"name":"capability-demo.answer","arguments":"{}"}}]})
            } else {
                serde_json::json!({"content":"I255 binary lifecycle complete"})
            };
            let reason = if requests.is_empty() {
                "tool_calls"
            } else {
                "stop"
            };
            let body = format!(
                "data: {}\n\ndata: {}\n\ndata: [DONE]\n\n",
                serde_json::json!({"choices":[{"index":0,"delta":delta,"finish_reason":null}]}),
                serde_json::json!({"choices":[{"index":0,"delta":{},"finish_reason":reason}]})
            );
            socket.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).as_bytes()).expect("model response");
            requests.push(request);
        }
        requests
    });
    let workspace = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates")
        .parent()
        .expect("workspace")
        .to_owned();
    let package = workspace.join("crates/talos-plugin/tests/fixtures/capability-demo");
    let mut child = Command::new(env!("CARGO_BIN_EXE_talos"))
        .args(["--print", "--no-context", "--workspace"])
        .arg(if allowed {
            workspace.as_path()
        } else {
            temp.path()
        })
        .arg("--plugin")
        .arg(package)
        .arg("Call capability-demo.answer")
        .env("HOME", temp.path())
        .env("USERPROFILE", temp.path())
        .env("XDG_CONFIG_HOME", "")
        .env("NO_PROXY", "127.0.0.1,localhost")
        .env("no_proxy", "127.0.0.1,localhost")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start real binary");
    let deadline = Instant::now() + Duration::from_secs(25);
    while child.try_wait().expect("poll binary").is_none() {
        if Instant::now() >= deadline {
            child.kill().expect("kill timed out test binary");
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let output = child.wait_with_output().expect("binary output");
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("I255 binary lifecycle complete"));
    let requests = server.join().expect("fixture completed");
    assert!(
        requests[0]["tools"]
            .to_string()
            .contains("capability-demo.answer")
    );
    assert!(
        requests[1]["messages"]
            .as_array()
            .expect("messages")
            .iter()
            .any(|message| message["role"] == "tool"
                && message["content"]
                    .as_str()
                    .is_some_and(|content| if allowed {
                        content.contains("returned 7")
                    } else {
                        content.to_lowercase().contains("permission")
                            && !content.contains("returned 7")
                    })),
        "no actual plugin execution: {}",
        requests[1]
    );
}
