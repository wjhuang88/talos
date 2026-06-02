use serde_json::json;
use talos_core::message::{AgentEvent, Message, StopReason};
use talos_core::provider::LanguageModel;
use talos_provider::AnthropicProvider;

fn sse_event(event_type: &str, data: &str) -> String {
    format!("event: {event_type}\ndata: {data}\n\n")
}

fn successful_stream_body() -> String {
    let mut body = String::new();
    body.push_str(&sse_event(
        "message_start",
        &json!({
            "message": {
                "id": "msg_123",
                "type": "message",
                "role": "assistant",
                "model": "claude-sonnet-4-20250514",
                "usage": {
                    "input_tokens": 10,
                    "output_tokens": 0,
                    "cache_read_input_tokens": 5,
                    "cache_creation_input_tokens": 3
                }
            }
        })
        .to_string(),
    ));
    body.push_str(&sse_event(
        "content_block_start",
        &json!({
            "index": 0,
            "content_block": { "type": "text", "text": "" }
        })
        .to_string(),
    ));
    body.push_str(&sse_event(
        "content_block_delta",
        &json!({
            "index": 0,
            "delta": { "type": "text_delta", "text": "Hello" }
        })
        .to_string(),
    ));
    body.push_str(&sse_event(
        "content_block_delta",
        &json!({
            "index": 0,
            "delta": { "type": "text_delta", "text": ", world!" }
        })
        .to_string(),
    ));
    body.push_str(&sse_event(
        "content_block_stop",
        &json!({
            "index": 0
        })
        .to_string(),
    ));
    body.push_str(&sse_event(
        "message_delta",
        &json!({
            "delta": { "stop_reason": "end_turn", "stop_sequence": null },
            "usage": { "output_tokens": 5 }
        })
        .to_string(),
    ));
    body.push_str(&sse_event(
        "message_stop",
        &json!({
            "type": "message_stop"
        })
        .to_string(),
    ));
    body
}

#[tokio::test]
async fn test_successful_streaming_response() {
    let mut server = mockito::Server::new_async().await;
    let body = successful_stream_body();

    let mock = server
        .mock("POST", "/")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(body)
        .create();

    let provider =
        AnthropicProvider::new("test-key", "claude-sonnet-4-20250514").with_base_url(server.url());

    let messages = vec![Message::User {
        content: "Hello".into(),
    }];

    let mut rx = provider
        .stream(&messages)
        .await
        .expect("stream should succeed");

    let mut events = Vec::new();
    while let Some(event) = rx.recv().await {
        events.push(event);
    }

    assert!(events.iter().any(|e| matches!(e, AgentEvent::TurnStart)));

    let text_deltas: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            AgentEvent::TextDelta { delta } => Some(delta.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(text_deltas, vec!["Hello", ", world!"]);

    let turn_end = events.iter().find_map(|e| match e {
        AgentEvent::TurnEnd { stop_reason, usage } => Some((stop_reason, usage)),
        _ => None,
    });
    assert!(turn_end.is_some());
    let (stop_reason, usage) = turn_end.unwrap();
    assert_eq!(stop_reason, &StopReason::EndTurn);
    assert_eq!(usage.input_tokens, 10);
    assert_eq!(usage.output_tokens, 5);
    assert_eq!(usage.cache_read_tokens, 5);
    assert_eq!(usage.cache_write_tokens, 3);

    mock.assert();
}

#[tokio::test]
async fn test_authentication_error() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/")
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error":{"message":"invalid api key"}}"#)
        .create();

    let provider =
        AnthropicProvider::new("bad-key", "claude-sonnet-4-20250514").with_base_url(server.url());

    let messages = vec![Message::User {
        content: "Hello".into(),
    }];

    let result = provider.stream(&messages).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("authentication failed"));

    mock.assert();
}

#[tokio::test]
async fn test_rate_limit_error() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/")
        .with_status(429)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error":{"message":"rate limited"}}"#)
        .expect_at_least(1)
        .create();

    let provider =
        AnthropicProvider::new("test-key", "claude-sonnet-4-20250514").with_base_url(server.url());

    let messages = vec![Message::User {
        content: "Hello".into(),
    }];

    let result = provider.stream(&messages).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("rate limited"));

    mock.assert();
}

#[tokio::test]
async fn test_server_error() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error":{"message":"internal error"}}"#)
        .expect_at_least(1)
        .create();

    let provider =
        AnthropicProvider::new("test-key", "claude-sonnet-4-20250514").with_base_url(server.url());

    let messages = vec![Message::User {
        content: "Hello".into(),
    }];

    let result = provider.stream(&messages).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("server error"));

    mock.assert();
}
