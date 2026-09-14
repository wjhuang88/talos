//! Cancelling a decision must release the network response, not leave an idle parser.
use talos_core::provider::{DecisionRequestLimits, LanguageModel};
use talos_provider::{AnthropicProvider, openai::OpenAIProvider};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn dropping_decision_receiver_closes_idle_http_stream() {
    for (anthropic, after_first_packet) in
        [(false, false), (false, true), (true, false), (true, true)]
    {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let url = format!("http://{}", listener.local_addr().expect("address"));
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("accept");
            let mut bytes = Vec::new();
            let header_end = loop {
                let mut byte = [0];
                socket.read_exact(&mut byte).await.expect("headers");
                bytes.push(byte[0]);
                assert!(bytes.len() < 16384);
                if bytes.ends_with(b"\r\n\r\n") {
                    break bytes.len();
                }
            };
            let headers = String::from_utf8(bytes[..header_end].to_vec()).expect("UTF8 headers");
            let length: usize = headers
                .lines()
                .find_map(|line| {
                    let (key, value) = line.split_once(':')?;
                    key.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse().expect("length"))
                })
                .expect("content length");
            assert!(length < 16384);
            let mut body = vec![0; length];
            socket.read_exact(&mut body).await.expect("request body");
            socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: 999999\r\nConnection: close\r\n\r\n").await.expect("response headers");
            if after_first_packet {
                let packet = if anthropic {
                    "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"partial\"}}\n\n"
                } else {
                    "data: {\"choices\":[{\"delta\":{\"content\":\"partial\"},\"finish_reason\":null}]}\n\n"
                };
                socket
                    .write_all(packet.as_bytes())
                    .await
                    .expect("first packet");
            }
            let mut byte = [0];
            socket.read(&mut byte).await.expect("peer closure")
        });
        let provider: Box<dyn LanguageModel> = if anthropic {
            Box::new(AnthropicProvider::new("test", "fixture").with_base_url(url))
        } else {
            Box::new(OpenAIProvider::new("test", "fixture").with_base_url(url))
        };
        let mut response = tokio::time::timeout(
            std::time::Duration::from_secs(3),
            provider.stream_decision(
                &[],
                DecisionRequestLimits {
                    max_output_tokens: 32,
                    max_retries: 0,
                },
            ),
        )
        .await
        .expect("dispatch deadline")
        .expect("response");
        if after_first_packet {
            tokio::time::timeout(std::time::Duration::from_secs(2), async {
                loop {
                    let event = response.recv().await.expect("partial response");
                    if matches!(event, talos_core::message::AgentEvent::TextDelta { .. }) {
                        break;
                    }
                }
            })
            .await
            .expect("first packet deadline");
        }
        drop(response);
        let mut server = server;
        let closed = tokio::time::timeout(std::time::Duration::from_secs(2), &mut server).await;
        if closed.is_err() {
            server.abort();
        }
        assert_eq!(
            closed
                .expect("parser must release response on cancellation")
                .expect("server task"),
            0
        );
    }
}
