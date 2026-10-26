use super::*;

#[test]
fn request_timeout_is_clamped_without_rebuilding_client() {
    let tool = HttpRequestTool::with_config(1024, 0);
    for (configured, expected) in [(0, 1), (30, 30), (100, 60)] {
        let input: HttpRequestInput = serde_json::from_value(serde_json::json!({
            "url": "https://example.com/", "timeout_secs": configured
        }))
        .expect("input");
        let request = tool
            .build_request(&tool.client, &input)
            .expect("builder")
            .build()
            .expect("request");
        assert_eq!(request.timeout(), Some(&Duration::from_secs(expected)));
    }
}

#[tokio::test]
async fn timeout_override_preserves_disabled_redirects() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener");
    let address = listener.local_addr().expect("address");
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("connection");
        let mut buffer = [0; 4096];
        stream.read(&mut buffer).await.expect("request");
        stream.write_all(b"HTTP/1.1 302 Found\r\nLocation: /redirected\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
            .await.expect("response");
    });
    let mut tool = HttpRequestTool::with_config(1024, 0);
    // Isolate this transport regression from ambient proxy configuration and the
    // public execute path's intentional loopback SSRF rejection.
    tool.client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::limited(0))
        .build()
        .expect("client");
    let input = serde_json::from_value(serde_json::json!({
        "url": format!("http://{address}/"), "timeout_secs": 2
    }))
    .expect("input");
    let error = tool
        .build_request(&tool.client, &input)
        .expect("request")
        .send()
        .await
        .expect_err("zero redirect budget must reject the redirect");
    assert!(error.is_redirect());
    server.await.expect("server");
}
