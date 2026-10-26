//! Full print-mode Agent round against a controlled Responses endpoint, including image/tool replay.

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

#[test]
fn responses_image_and_native_read_complete_through_real_agent() {
    let temp = tempfile::tempdir().expect("isolated home and workspace");
    let config_dir = temp.path().join(".talos");
    std::fs::create_dir(&config_dir).expect("config directory");
    std::fs::write(temp.path().join("note.txt"), "RESPONSES-READ-RESULT").expect("read fixture");
    let image_path = temp.path().join("sample.png");
    image::RgbaImage::from_pixel(1, 1, image::Rgba([1, 2, 3, 255]))
        .save(&image_path)
        .expect("image fixture");
    let listener = TcpListener::bind("127.0.0.1:0").expect("controlled endpoint");
    listener.set_nonblocking(true).expect("bounded accept");
    let address = listener.local_addr().expect("endpoint address");
    std::fs::write(config_dir.join("config.toml"), format!(
        "provider = \"fixture\"\nmodel = \"test-model\"\n[providers.fixture]\nprotocol = \"openai-chat\"\nbase_url = \"http://{address}\"\napi_key = \"test-key\"\n[providers.fixture.models.test-model]\nprotocol = \"openai-responses\"\nimage_input = true\n"
    )).expect("config");
    let server = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(25);
        let mut requests = Vec::new();
        while requests.len() < 2 {
            let mut socket = loop {
                match listener.accept() {
                    Ok((socket, _)) => break socket,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(
                            Instant::now() < deadline,
                            "Agent did not finish its two requests"
                        );
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) => panic!("accept: {error}"),
                }
            };
            requests.push(read_request(&mut socket));
            let mut events = vec![serde_json::json!({"type": "response.created"})];
            if requests.len() == 1 {
                let item = serde_json::json!({
                    "id": "fc_read", "type": "function_call", "call_id": "call_read",
                    "name": "read", "arguments": "{\"path\":\"note.txt\"}",
                });
                events
                    .push(serde_json::json!({"type": "response.output_item.added", "item": item}));
                events.push(serde_json::json!({"type": "response.output_item.done", "item": item}));
            } else {
                events.push(serde_json::json!({"type": "response.output_text.delta", "delta": "RESPONSES-AGENT-COMPLETE"}));
            }
            events.push(serde_json::json!({"type": "response.completed", "response": {"usage": {"input_tokens": 10, "output_tokens": 2}}}));
            let body: String = events
                .iter()
                .map(|event| format!("data: {event}\n\n"))
                .collect();
            socket.write_all(format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()
            ).as_bytes()).expect("response");
        }
        requests
    });
    let mut child = Command::new(env!("CARGO_BIN_EXE_talos"))
        .args(["--print", "--no-context", "--workspace"])
        .arg(temp.path())
        .arg("--attach")
        .arg(&image_path)
        .arg("Read note.txt and describe the image")
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .env("USERPROFILE", temp.path())
        .env("NO_PROXY", "127.0.0.1,localhost")
        .env("no_proxy", "127.0.0.1,localhost")
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
    assert!(String::from_utf8_lossy(&output.stdout).contains("RESPONSES-AGENT-COMPLETE"));
    let requests = server.join().expect("controlled endpoint finished");
    for request in &requests {
        assert_eq!(request["store"], false);
        assert_eq!(request["stream"], true);
        assert!(request.get("previous_response_id").is_none());
        assert!(
            request["input"].to_string().contains("input_image"),
            "image continuation"
        );
    }
    assert!(
        requests[0]["tools"]
            .as_array()
            .expect("native tools")
            .iter()
            .any(|tool| tool["type"] == "function" && tool["name"] == "read")
    );
    assert!(
        requests[1]["input"]
            .as_array()
            .expect("input items")
            .iter()
            .any(|item| item["type"] == "function_call_output"
                && item["call_id"] == "call_read"
                && item["output"]
                    .as_str()
                    .is_some_and(|text| text.contains("RESPONSES-READ-RESULT")))
    );
}
