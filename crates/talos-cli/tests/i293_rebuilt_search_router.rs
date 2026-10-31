//! Rebuilt-binary evidence for SEARCH-001-C / I293 C-V11.
//!
//! Drives the real `talos` binary against a loopback Responses provider. The provider asks the
//! production registry to call `web_search`; headless permission denial stops before any external
//! search send while still proving that the rebuilt binary advertised and entered the protected
//! prepared-network SearchRouter path.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn read_request(socket: &mut TcpStream) -> serde_json::Value {
    socket.set_nonblocking(false).expect("blocking socket");
    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("read deadline");
    let mut bytes = Vec::new();
    let mut buffer = [0; 8192];
    loop {
        let count = socket.read(&mut buffer).expect("request bytes");
        assert!(count > 0, "incomplete request");
        bytes.extend_from_slice(&buffer[..count]);
        assert!(bytes.len() < 4 * 1024 * 1024, "bounded request");
        if let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
            let headers = String::from_utf8_lossy(&bytes[..end]);
            assert!(headers.starts_with("POST /responses HTTP/1.1"), "{headers}");
            assert!(
                headers.lines().any(|line| line
                    .split_once(':')
                    .is_some_and(|(name, value)| name.eq_ignore_ascii_case("authorization")
                        && value.trim() == "Bearer test-key")),
                "configured API credential must be used"
            );
            let length = headers
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().expect("content length"))
                })
                .expect("content length header");
            if bytes.len() >= end + 4 + length {
                return serde_json::from_slice(&bytes[end + 4..end + 4 + length])
                    .expect("request JSON");
            }
        }
    }
}

fn write_events(socket: &mut TcpStream, events: &[serde_json::Value]) {
    let body: String = events
        .iter()
        .map(|event| format!("data: {event}\n\n"))
        .collect();
    socket
        .write_all(
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .as_bytes(),
        )
        .expect("fixture response");
}

#[test]
fn rebuilt_binary_registers_and_enters_protected_web_search_path() {
    let temp = tempfile::tempdir().expect("isolated home and workspace");
    let config_dir = temp.path().join(".talos");
    std::fs::create_dir(&config_dir).expect("config directory");

    let listener = TcpListener::bind("127.0.0.1:0").expect("controlled Responses endpoint");
    listener.set_nonblocking(true).expect("bounded accept");
    let address = listener.local_addr().expect("endpoint address");
    std::fs::write(
        config_dir.join("config.toml"),
        format!(
            r#"provider = "fixture"
model = "test-model"
[providers.fixture]
protocol = "openai-responses"
base_url = "http://{address}"
api_key = "test-key"
[providers.fixture.models.test-model]
"#
        ),
    )
    .expect("fixture config");

    let server = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(25);
        let mut requests = Vec::new();
        while requests.len() < 2 {
            let mut socket = loop {
                match listener.accept() {
                    Ok((socket, _)) => break socket,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(Instant::now() < deadline, "rebuilt Agent round timed out");
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) => panic!("accept: {error}"),
                }
            };
            requests.push(read_request(&mut socket));
            if requests.len() == 1 {
                let item = serde_json::json!({
                    "id": "fc_search",
                    "type": "function_call",
                    "call_id": "call_search",
                    "name": "web_search",
                    "arguments": "{\"query\":\"private rebuilt probe\",\"max_results\":1,\"include_snippets\":true}"
                });
                write_events(
                    &mut socket,
                    &[
                        serde_json::json!({"type": "response.created"}),
                        serde_json::json!({"type": "response.output_item.added", "item": item}),
                        serde_json::json!({"type": "response.output_item.done", "item": item}),
                        serde_json::json!({
                            "type": "response.completed",
                            "response": {"usage": {"input_tokens": 10, "output_tokens": 2}}
                        }),
                    ],
                );
            } else {
                write_events(
                    &mut socket,
                    &[
                        serde_json::json!({"type": "response.created"}),
                        serde_json::json!({
                            "type": "response.output_text.delta",
                            "delta": "I293-REBUILT-SEARCH-COMPLETE"
                        }),
                        serde_json::json!({
                            "type": "response.completed",
                            "response": {"usage": {"input_tokens": 10, "output_tokens": 2}}
                        }),
                    ],
                );
            }
        }
        requests
    });

    let mut child = Command::new(env!("CARGO_BIN_EXE_talos"))
        .args(["--print", "--no-context", "--no-init", "--workspace"])
        .arg(temp.path())
        .arg("Use web_search to investigate the rebuilt SearchRouter fixture")
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .env("USERPROFILE", temp.path())
        .env("NO_PROXY", "127.0.0.1,localhost")
        .env("no_proxy", "127.0.0.1,localhost")
        .env_remove("HTTPS_PROXY")
        .env_remove("https_proxy")
        .env_remove("ALL_PROXY")
        .env_remove("all_proxy")
        .env_remove("REQUEST_METHOD")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("real talos binary");

    let deadline = Instant::now() + Duration::from_secs(30);
    while child.try_wait().expect("poll binary").is_none() {
        if Instant::now() >= deadline {
            child.kill().expect("terminate timed-out fixture");
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
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("I293-REBUILT-SEARCH-COMPLETE"),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let requests = server.join().expect("controlled endpoint finished");
    let tools = requests[0]["tools"].as_array().expect("advertised tools");
    let search = tools
        .iter()
        .find(|tool| tool["type"] == "function" && tool["name"] == "web_search")
        .expect("production web_search registration");
    assert!(
        search["description"]
            .as_str()
            .is_some_and(|description| description.contains("Talos-owned routed candidate set")
                && description.contains("complete prepared network plan")),
        "protected SearchRouter description: {search}"
    );
    let parameters = &search["parameters"];
    assert_eq!(parameters["type"], "object");
    let properties = parameters["properties"].as_object().expect("schema properties");
    assert!(properties.contains_key("query"));
    assert!(properties.contains_key("max_results"));
    assert!(properties.contains_key("include_snippets"));
    assert!(
        parameters["required"]
            .as_array()
            .is_some_and(|required| required.iter().any(|field| field == "query")),
        "query remains required: {parameters}"
    );

    let continuation = requests[1]["input"]
        .as_array()
        .expect("continuation input")
        .iter()
        .find(|item| item["type"] == "function_call_output" && item["call_id"] == "call_search")
        .expect("web_search tool result returned to provider");
    assert!(
        continuation["output"]
            .as_str()
            .is_some_and(|text| text.contains("Permission denied")),
        "headless permission gate must stop before external search execution: {continuation}"
    );
}
