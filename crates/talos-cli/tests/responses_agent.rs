//! Full print-mode Agent round against a controlled Responses endpoint, including image/tool replay.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn read_request(socket: &mut TcpStream) -> serde_json::Value {
    read_request_at(socket, "/responses")
}

fn read_request_at(socket: &mut TcpStream, path: &str) -> serde_json::Value {
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
            assert!(
                headers.starts_with(&format!("POST {path} HTTP/1.1")),
                "{headers}"
            );
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

#[test]
fn responses_image_and_native_read_complete_through_real_agent() {
    image_and_native_read_round(ImageFixture::Configured);
}

#[test]
fn neon_packaged_model_endpoint_drives_a_real_agent_round() {
    image_and_native_read_round(ImageFixture::Packaged);
}

#[test]
fn custom_catalog_image_and_output_complete_through_real_agent() {
    image_and_native_read_round(ImageFixture::CustomInferred);
}

#[test]
fn configured_gateway_endpoint_preserves_catalog_variant_through_real_agent() {
    image_and_native_read_round(ImageFixture::CatalogVariant);
}

#[test]
fn custom_catalog_reasoning_and_image_complete_through_real_agent() {
    image_and_native_read_round(ImageFixture::CustomReasoning);
}

#[derive(Clone, Copy)]
enum ImageFixture {
    Configured,
    Packaged,
    CustomInferred,
    CustomReasoning,
    CatalogVariant,
}

fn image_and_native_read_round(fixture: ImageFixture) {
    let custom_reasoning = matches!(fixture, ImageFixture::CustomReasoning);
    let inferred = matches!(
        fixture,
        ImageFixture::CustomInferred | ImageFixture::CustomReasoning
    );
    let catalog = matches!(fixture, ImageFixture::Packaged);
    let variant = matches!(fixture, ImageFixture::CatalogVariant);
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
    let models = talos_config::model::builtin_models();
    let inferred_model = models
        .iter()
        .find(|model| {
            model.capabilities.image_input
                && model.capabilities.reasoning
                && model.output_limit.is_some()
                && models
                    .iter()
                    .filter(|candidate| candidate.id == model.id)
                    .count()
                    == 1
        })
        .expect("unique vision/reasoning catalog fixture");
    let inferred_id = format!("proxy/{}", inferred_model.id);
    let (expected_path, expected_model) = if variant {
        std::fs::write(
            config_dir.join("config.toml"),
            format!(
                r#"provider = "openai"
model = "o3"
variant = "high-reasoning"
[providers.openai]
protocol = "openai-responses"
base_url = "http://{address}/model-wire"
api_key = "test-key"
[providers.openai.models.o3.reasoning]
effort = "low"
replay = false
"#
            ),
        )
        .expect("gateway endpoint with catalog-scoped variant");
        ("/model-wire/responses", "o3")
    } else if inferred {
        std::fs::write(
            config_dir.join("config.toml"),
            format!(
                r#"provider = "fixture"
model = {inferred_id:?}
[providers.fixture]
protocol = "openai-responses"
base_url = "http://{address}/model-wire"
api_key = "test-key"
[providers.fixture.models.{inferred_id:?}]
"#,
            ),
        )
        .expect("inferred config without capability or limit overrides");
        ("/model-wire/responses", inferred_id.as_str())
    } else if catalog {
        std::fs::write(config_dir.join("config.toml"), "provider = \"neon\"\nmodel = \"gpt-5-2\"\n[providers.neon]\nbase_url = \"${NEON_AI_GATEWAY_BASE_URL}/v1\"\napi_key_env = \"NEON_AI_GATEWAY_TOKEN\"\n").expect("catalog-routed config without model protocol/endpoint overrides");
        ("/openai/v1/responses", "gpt-5-2")
    } else {
        std::fs::write(config_dir.join("config.toml"),
            "provider = \"fixture\"\nmodel = \"test-model\"\n[providers.fixture]\nprotocol = \"openai-chat\"\nbase_url = \"http://127.0.0.1:9/provider\"\napi_key = \"test-key\"\n[providers.fixture.models.test-model]\nbase_url = \"${TALOS_MODEL_ENDPOINT_TEST}/model-wire\"\nprotocol = \"openai-responses\"\nimage_input = true\n"
        ).expect("config");
        ("/model-wire/responses", "test-model")
    };
    if custom_reasoning {
        use std::io::Write;
        let mut config = std::fs::OpenOptions::new()
            .append(true)
            .open(config_dir.join("config.toml"))
            .expect("existing custom config");
        writeln!(config, "[providers.fixture.models.{inferred_id:?}.reasoning]\neffort = \"high\"\nreplay = false")
            .expect("explicit effort without capability override");
    }
    let config_before = std::fs::read(config_dir.join("config.toml")).expect("config bytes");
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
            requests.push(read_request_at(&mut socket, expected_path));
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
        .env("TALOS_MODEL_ENDPOINT_TEST", format!("http://{address}"))
        .env("NEON_AI_GATEWAY_BASE_URL", format!("http://{address}"))
        .env("NEON_AI_GATEWAY_TOKEN", "test-key")
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
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("RESPONSES-AGENT-COMPLETE"),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let requests = server.join().expect("controlled endpoint finished");
    assert_eq!(
        std::fs::read(config_dir.join("config.toml")).expect("config bytes"),
        config_before
    );
    for request in &requests {
        assert_eq!(request["model"], expected_model);
        if variant {
            assert_eq!(
                request["reasoning"]["effort"], "high",
                "selected variant overrides explicit effort"
            );
        }
        if inferred {
            assert_eq!(
                request["max_output_tokens"],
                inferred_model.output_limit.expect("output")
            );
            if custom_reasoning {
                assert_eq!(request["reasoning"]["effort"], "high");
            } else {
                assert!(
                    request.get("reasoning").is_none(),
                    "capability alone must not invent a preset"
                );
            }
        }
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

#[test]
fn responses_tool_image_is_sent_once_through_real_agent() {
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
        while requests.len() < 3 {
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
            if requests.len() <= 2 {
                let (name, path, id) = if requests.len() == 1 {
                    ("read_image", "sample.png", "call_image")
                } else {
                    ("read", "note.txt", "call_read")
                };
                let item = serde_json::json!({
                    "id": format!("fc_{id}"), "type": "function_call", "call_id": id,
                    "name": name, "arguments": serde_json::json!({"path":path}).to_string(),
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
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("RESPONSES-AGENT-COMPLETE"),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let requests = server.join().expect("controlled endpoint finished");
    for request in &requests {
        assert_eq!(request["store"], false);
        assert_eq!(request["stream"], true);
        assert!(request.get("previous_response_id").is_none());
    }
    assert!(
        requests[0]["tools"]
            .as_array()
            .expect("native tools")
            .iter()
            .any(|tool| tool["type"] == "function" && tool["name"] == "read")
    );
    for (index, request) in requests.iter().enumerate() {
        assert_eq!(
            request["input"]
                .to_string()
                .matches("\"type\":\"input_image\"")
                .count(),
            usize::from(index == 1),
            "tool image must appear exactly once in the next request"
        );
        assert!(!request.to_string().contains("data:image") || index == 1);
    }
    assert!(
        requests[1]["input"]
            .as_array()
            .expect("Responses input items")
            .iter()
            .any(|item| item["type"] == "function_call_output" && item["call_id"] == "call_image")
    );
    assert!(
        requests[2]["input"]
            .as_array()
            .expect("Responses input items")
            .iter()
            .any(|item| {
                item["type"] == "function_call_output"
                    && item["call_id"] == "call_read"
                    && item["output"]
                        .as_str()
                        .is_some_and(|text| text.contains("RESPONSES-READ-RESULT"))
            })
    );
}

#[test]
fn responses_compat_tool_round_through_real_agent() {
    textual_tool_round_through_real_agent("compat");
}
#[test]
fn responses_strict_tool_round_through_real_agent() {
    textual_tool_round_through_real_agent("talos-strict");
}

fn textual_tool_round_through_real_agent(mode: &str) {
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
        "provider = \"fixture\"\nmodel = \"test-model\"\n[providers.fixture]\nprotocol = \"openai-chat\"\ntool_protocol = \"{mode}\"\nbase_url = \"http://{address}\"\napi_key = \"test-key\"\n[providers.fixture.models.test-model]\nprotocol = \"openai-responses\"\nimage_input = true\n"
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
                let text = r#"<tool_call>{"id":"call_read","name":"read","args":{"path":"note.txt"}}</tool_call>"#;
                events.push(serde_json::json!({"type":"response.output_text.delta","delta":text}));
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
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("RESPONSES-AGENT-COMPLETE"),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let requests = server.join().expect("controlled endpoint finished");
    for request in &requests {
        assert_eq!(request["store"], false);
        assert_eq!(request["stream"], true);
        assert!(request.get("previous_response_id").is_none());
    }
    assert!(requests[0]["tools"].as_array().is_none_or(Vec::is_empty));
    let replay = requests[1]["input"].to_string();
    assert!(replay.contains("<tool_result id=call_read"));
    assert!(replay.contains("RESPONSES-READ-RESULT"));
    assert!(!replay.contains("function_call_output"));
}
