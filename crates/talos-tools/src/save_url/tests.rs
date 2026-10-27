use super::*;

async fn response(body: &'static [u8]) -> reqwest::Response {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind fixture");
    let address = listener.local_addr().expect("fixture address");
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept fixture");
        let mut headers = Vec::new();
        while !headers.ends_with(b"\r\n\r\n") {
            assert!(headers.len() < 16 * 1024);
            headers.push(stream.read_u8().await.expect("request header"));
        }
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n")
            .await
            .expect("response header");
        stream.write_all(body).await.expect("response body");
    });
    reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("fixture client")
        .get(format!("http://{address}"))
        .send()
        .await
        .expect("fixture response")
}

#[tokio::test]
async fn download_without_content_length_accepts_exact_limit() {
    let body = read_bounded_body(response(b"12345").await, 5)
        .await
        .expect("exact limit");
    assert_eq!(body, b"12345");
}

#[tokio::test]
async fn download_without_content_length_rejects_over_limit() {
    let error = read_bounded_body(response(b"123456").await, 5)
        .await
        .expect_err("body exceeds limit");
    assert!(error.contains("response too large"));
}

#[tokio::test]
async fn oversized_download_is_rejected_before_the_server_finishes() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind fixture");
    let address = listener.local_addr().expect("fixture address");
    let (release, released) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept fixture");
        let mut headers = Vec::new();
        while !headers.ends_with(b"\r\n\r\n") {
            assert!(headers.len() < 16 * 1024);
            headers.push(stream.read_u8().await.expect("request header"));
        }
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\n123456")
            .await
            .expect("partial oversized response");
        let _ = released.await;
    });
    let response = reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("fixture client")
        .get(format!("http://{address}"))
        .send()
        .await
        .expect("fixture response");
    let result = tokio::time::timeout(Duration::from_secs(2), read_bounded_body(response, 5)).await;
    let _ = release.send(());
    server.await.expect("fixture completed");
    assert!(
        result
            .expect("reject without waiting for EOF")
            .expect_err("oversized body")
            .contains("response too large")
    );
}

#[test]
fn test_tool_name() {
    let tool = SaveUrlTool::new();
    assert_eq!(tool.name(), "save_url");
}

#[test]
fn test_tool_not_read_only() {
    let tool = SaveUrlTool::new();
    assert!(!tool.is_read_only());
}

#[test]
fn test_tool_nature_is_write() {
    let tool = SaveUrlTool::new();
    assert!(matches!(tool.nature(), ToolNature::Write));
}

#[test]
fn test_permission_profile_includes_network_and_write_facets() {
    let tool = SaveUrlTool::new();
    let profile = tool.permission_profile(&serde_json::json!({
        "url": "https://Example.com/archive.zip",
        "destination": "downloads/archive.zip"
    }));

    assert_eq!(profile.len(), 2);
    assert_eq!(profile[0].nature, ToolNature::Network);
    assert_eq!(profile[0].resource.as_deref(), Some("example.com"));
    assert_eq!(profile[0].resource_kind, Some(ToolResourceKind::Domain));
    assert_eq!(profile[1].nature, ToolNature::Write);
    assert_eq!(
        profile[1].resource.as_deref(),
        Some("downloads/archive.zip")
    );
    assert_eq!(profile[1].resource_kind, Some(ToolResourceKind::Path));
}

#[test]
fn test_tool_summary_fields() {
    let tool = SaveUrlTool::new();
    assert_eq!(tool.summary_fields(), &["url", "destination"]);
}

#[test]
fn test_tool_emits_parameters_schema() {
    let tool = SaveUrlTool::new();
    let schema = tool.parameters();
    assert!(schema.is_object());
    assert!(schema.get("properties").is_some());
}

#[test]
fn test_deserialize_input() {
    let json = r#"{"url": "https://example.com/file.zip", "destination": "/tmp/file.zip"}"#;
    let input: SaveUrlInput = serde_json::from_str(json).expect("operation should succeed");
    assert_eq!(input.url, "https://example.com/file.zip");
    assert_eq!(input.destination, "/tmp/file.zip");
}
