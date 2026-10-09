use super::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn ephemeral_deadline_cancels_stalled_error_body_and_retry_backoff() {
    for anthropic in [false, true] {
        for stalled_body in [false, true] {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .expect("listener");
            let endpoint = format!("http://{}", listener.local_addr().expect("address"));
            let (sent_tx, sent_rx) = tokio::sync::oneshot::channel();
            let server = tokio::spawn(async move {
                let (mut socket, _) = listener.accept().await.expect("accept");
                let mut buffer = [0; 8192];
                assert!(socket.read(&mut buffer).await.expect("request") > 0);
                let response = if stalled_body {
                    b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 1024\r\n\r\nx".as_slice()
                } else {
                    b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 1\r\n\r\nx".as_slice()
                };
                socket.write_all(response).await.expect("response");
                let _ = sent_tx.send(());
                std::future::pending::<()>().await;
            });
            let timeout = ProviderTimeoutConfig {
                backoff_base_ms: 5000,
                backoff_max_ms: 5000,
                max_attempts: 2,
                ..ProviderTimeoutConfig::default()
            };
            let provider: Box<dyn LanguageModel> = if anthropic {
                Box::new(
                    AnthropicProvider::new("fixture", "fixture")
                        .with_base_url(endpoint)
                        .with_timeout_config(timeout),
                )
            } else {
                Box::new(
                    openai::OpenAIProvider::new("fixture", "fixture")
                        .with_base_url(endpoint)
                        .with_timeout_config(timeout),
                )
            };
            let image = talos_core::provider::EphemeralImage::png(
                b"\x89PNG\r\n\x1a\nfixture".to_vec(),
                std::time::Instant::now() + Duration::from_millis(500),
            )
            .expect("image");
            let (progress, _) = mpsc::unbounded_channel();
            let result = tokio::time::timeout(
                Duration::from_secs(2),
                provider.stream_with_ephemeral_images(
                    &[Message::User {
                        content: "fixture".into(),
                    }],
                    &[],
                    talos_core::tool::ToolProtocol::Native,
                    progress,
                    &[],
                    vec![image],
                ),
            )
            .await;
            server.abort();
            let _ = server.await;
            if sent_rx.await.is_err() {
                match &result {
                    Ok(Err(error)) => panic!("fixture was not reached: {error}"),
                    _ => panic!("fixture was not reached"),
                }
            }
            assert!(
                matches!(result, Ok(Err(ProviderError::InvalidResponse(message)))
                if message.contains("ephemeral image expired"))
            );
        }
    }
}
