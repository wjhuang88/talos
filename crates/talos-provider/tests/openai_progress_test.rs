use talos_config::ProviderTimeoutConfig;
use talos_core::message::Message;
use talos_core::provider::{LanguageModel, ProviderProgress};
use talos_provider::openai::OpenAIProvider;

#[tokio::test]
async fn protocol_dispatch_preserves_retry_progress_and_wire_tool_mode() {
    use serde_json::json;
    use talos_core::message::{AgentEvent, StopReason};
    use talos_core::provider::ToolDefinition;
    use talos_core::tool::ToolProtocol;

    for protocol in [
        ToolProtocol::Native,
        ToolProtocol::Compat,
        ToolProtocol::TalosStrict,
    ] {
        let mut server = mockito::Server::new_async().await;
        let tool = ToolDefinition::new("probe", "read-only fixture", json!({"type": "object"}));
        let native = protocol == ToolProtocol::Native;
        let retry = server
            .mock("POST", "/chat/completions")
            .match_request(move |request| {
                let body: serde_json::Value =
                    serde_json::from_slice(request.body().expect("body")).expect("JSON");
                if native {
                    body["tools"][0]["function"]["name"] == "probe"
                } else {
                    body.get("tools").is_none()
                }
            })
            .with_status(500)
            .with_body("transient fixture error")
            .expect(1)
            .create_async()
            .await;
        let success = server.mock("POST", "/chat/completions")
            .with_status(200).with_header("content-type", "text/event-stream")
            .with_body("data: {\"choices\":[{\"delta\":{\"content\":\"ok\"},\"finish_reason\":null}]}\n\ndata: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\ndata: [DONE]\n\n")
            .expect(1).create_async().await;
        let provider = OpenAIProvider::new("test-key", "test-model")
            .with_base_url(server.url())
            .with_timeout_config(timeout_config(1));
        let (tx, mut progress) = tokio::sync::mpsc::unbounded_channel();
        let mut events = provider
            .stream_with_protocol(&messages(), &[tool], protocol, tx)
            .await
            .expect("response");
        assert_eq!(
            collect_progress(&mut progress),
            vec![
                ProviderProgress::InitialDispatch {
                    attempt: 0,
                    max_attempts: 1
                },
                ProviderProgress::ScheduledBackoff {
                    attempt: 1,
                    max_attempts: 1,
                    delay_ms: 0
                },
                ProviderProgress::RetryDispatch {
                    attempt: 1,
                    max_attempts: 1
                },
                ProviderProgress::FirstPacketWait {
                    attempt: 1,
                    max_attempts: 1
                },
            ]
        );
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            let mut completed = false;
            while let Some(event) = events.recv().await {
                assert!(!matches!(event, AgentEvent::Error { .. }));
                completed |= matches!(
                    event,
                    AgentEvent::TurnEnd {
                        stop_reason: StopReason::EndTurn,
                        ..
                    }
                );
            }
            assert!(completed);
        })
        .await
        .expect("bounded response");
        retry.assert_async().await;
        success.assert_async().await;
    }
}

fn timeout_config(max_attempts: u32) -> ProviderTimeoutConfig {
    ProviderTimeoutConfig {
        max_attempts,
        backoff_base_ms: 0,
        backoff_max_ms: 0,
        ..ProviderTimeoutConfig::default()
    }
}

fn messages() -> Vec<Message> {
    vec![Message::User {
        content: "hello".into(),
    }]
}

fn collect_progress(
    progress_rx: &mut tokio::sync::mpsc::UnboundedReceiver<ProviderProgress>,
) -> Vec<ProviderProgress> {
    let mut progress = Vec::new();
    while let Ok(item) = progress_rx.try_recv() {
        progress.push(item);
    }
    progress
}

#[tokio::test]
async fn successful_initial_dispatch_reports_first_packet_wait() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body("data: [DONE]\n\n")
        .create_async()
        .await;
    let provider = OpenAIProvider::new("secret-key", "test-model")
        .with_base_url(server.url())
        .with_timeout_config(timeout_config(3));
    let (progress_tx, mut progress_rx) = tokio::sync::mpsc::unbounded_channel();

    let result = provider
        .stream_with_tools_and_progress(&messages(), &[], progress_tx)
        .await;

    assert!(result.is_ok());
    assert_eq!(
        collect_progress(&mut progress_rx),
        vec![
            ProviderProgress::InitialDispatch {
                attempt: 0,
                max_attempts: 3,
            },
            ProviderProgress::FirstPacketWait {
                attempt: 0,
                max_attempts: 3,
            },
        ]
    );
    mock.assert_async().await;
}

#[tokio::test]
async fn retryable_failures_report_exact_ordinals_and_exhaust_without_secrets() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/chat/completions")
        .with_status(429)
        .with_body("secret-key provider response")
        .expect(3)
        .create_async()
        .await;
    let provider = OpenAIProvider::new("secret-key", "test-model")
        .with_base_url(server.url())
        .with_timeout_config(timeout_config(2));
    let (progress_tx, mut progress_rx) = tokio::sync::mpsc::unbounded_channel();

    let result = provider
        .stream_with_tools_and_progress(&messages(), &[], progress_tx)
        .await;

    assert!(result.is_err());
    let progress = collect_progress(&mut progress_rx);
    assert_eq!(
        progress,
        vec![
            ProviderProgress::InitialDispatch {
                attempt: 0,
                max_attempts: 2,
            },
            ProviderProgress::ScheduledBackoff {
                attempt: 1,
                max_attempts: 2,
                delay_ms: 0,
            },
            ProviderProgress::RetryDispatch {
                attempt: 1,
                max_attempts: 2,
            },
            ProviderProgress::ScheduledBackoff {
                attempt: 2,
                max_attempts: 2,
                delay_ms: 0,
            },
            ProviderProgress::RetryDispatch {
                attempt: 2,
                max_attempts: 2,
            },
        ]
    );
    let encoded = serde_json::to_string(&progress).expect("serialize progress");
    assert!(!encoded.contains("secret-key"));
    assert!(!encoded.contains("provider response"));
    mock.assert_async().await;
}

#[tokio::test]
async fn retryable_failure_then_success_preserves_progress_order() {
    let mut server = mockito::Server::new_async().await;
    let retry = server
        .mock("POST", "/chat/completions")
        .with_status(500)
        .with_body("retry once")
        .expect(1)
        .create_async()
        .await;
    let success = server
        .mock("POST", "/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body("data: [DONE]\n\n")
        .expect(1)
        .create_async()
        .await;
    let provider = OpenAIProvider::new("secret-key", "test-model")
        .with_base_url(server.url())
        .with_timeout_config(timeout_config(2));
    let (progress_tx, mut progress_rx) = tokio::sync::mpsc::unbounded_channel();

    let result = provider
        .stream_with_tools_and_progress(&messages(), &[], progress_tx)
        .await;

    assert!(result.is_ok());
    assert_eq!(
        collect_progress(&mut progress_rx),
        vec![
            ProviderProgress::InitialDispatch {
                attempt: 0,
                max_attempts: 2,
            },
            ProviderProgress::ScheduledBackoff {
                attempt: 1,
                max_attempts: 2,
                delay_ms: 0,
            },
            ProviderProgress::RetryDispatch {
                attempt: 1,
                max_attempts: 2,
            },
            ProviderProgress::FirstPacketWait {
                attempt: 1,
                max_attempts: 2,
            },
        ]
    );
    retry.assert_async().await;
    success.assert_async().await;
}

#[tokio::test]
async fn dispatch_timeout_reports_retry_progress_from_existing_policy() {
    let config = ProviderTimeoutConfig {
        dispatch_timeout_secs: 0,
        max_attempts: 1,
        backoff_base_ms: 0,
        backoff_max_ms: 0,
        ..ProviderTimeoutConfig::default()
    };
    let provider = OpenAIProvider::new("secret-key", "test-model")
        .with_base_url("http://127.0.0.1:9")
        .with_timeout_config(config);
    let (progress_tx, mut progress_rx) = tokio::sync::mpsc::unbounded_channel();

    let result = provider
        .stream_with_tools_and_progress(&messages(), &[], progress_tx)
        .await;

    assert!(result.is_err());
    assert_eq!(
        collect_progress(&mut progress_rx),
        vec![
            ProviderProgress::InitialDispatch {
                attempt: 0,
                max_attempts: 1,
            },
            ProviderProgress::ScheduledBackoff {
                attempt: 1,
                max_attempts: 1,
                delay_ms: 0,
            },
            ProviderProgress::RetryDispatch {
                attempt: 1,
                max_attempts: 1,
            },
        ]
    );
}
