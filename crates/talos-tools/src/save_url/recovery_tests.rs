use super::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub(super) async fn tool_with_download(
    body: &'static [u8],
) -> (SaveUrlTool, tokio::task::JoinHandle<()>) {
    tool_with_download_action(body, || {}).await
}

async fn tool_with_download_action(
    body: &'static [u8],
    action: impl FnOnce() + Send + 'static,
) -> (SaveUrlTool, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind proxy");
    let address = listener.local_addr().expect("proxy address");
    let server = tokio::spawn(async move {
        tokio::time::timeout(Duration::from_secs(5), async move {
            let (mut stream, _) = listener.accept().await.expect("accept proxy");
            let mut headers = Vec::new();
            while !headers.ends_with(b"\r\n\r\n") {
                assert!(headers.len() < 16 * 1024);
                headers.push(stream.read_u8().await.expect("request"));
            }
            action();
            stream
                .write_all(
                    format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", body.len()).as_bytes(),
                )
                .await
                .expect("response headers");
            stream.write_all(body).await.expect("response body");
        })
        .await
        .expect("proxy deadline");
    });
    let tool = SaveUrlTool {
        client: reqwest::Client::builder()
            .no_proxy()
            .proxy(reqwest::Proxy::all(format!("http://{address}")).expect("proxy"))
            .timeout(Duration::from_secs(5))
            .build()
            .expect("client"),
        max_body_bytes: DEFAULT_MAX_BODY_BYTES,
        recovery: Default::default(),
    };
    (tool, server)
}

#[tokio::test]
async fn destination_modified_during_download_is_not_overwritten() {
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("download");
    std::fs::write(&path, b"original").expect("original");
    let changed = path.clone();
    let (tool, server) = tool_with_download_action(b"downloaded", move || {
        std::fs::write(changed, b"concurrent user edit").expect("concurrent edit");
    })
    .await;
    let result = tool.execute(serde_json::json!({"url":"http://93.184.216.34/fixture", "destination":path,"retain_backup":true})).await;
    server.await.expect("proxy completed");
    assert!(result.is_error, "{}", result.content);
    assert_eq!(
        std::fs::read(path).expect("preserved"),
        b"concurrent user edit"
    );
}

#[tokio::test]
async fn rejected_download_preserves_the_original_destination() {
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("download");
    std::fs::write(&path, b"original").expect("original");
    let (mut tool, server) = tool_with_download(b"downloaded").await;
    tool.max_body_bytes = 2;
    let result = tool.execute(serde_json::json!({"url":"http://93.184.216.34/fixture", "destination":path,"retain_backup":true})).await;
    server.await.expect("proxy completed");
    assert!(result.is_error, "{}", result.content);
    assert_eq!(std::fs::read(path).expect("preserved"), b"original");
}

#[tokio::test]
async fn downloaded_overwrite_has_one_use_restore_and_rejects_changed_content() {
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("download");
    std::fs::write(&path, b"original").expect("original");
    let (tool, server) = tool_with_download(b"downloaded").await;
    let input = serde_json::json!({"url":"http://93.184.216.34/fixture", "destination":path, "retain_backup":true});
    let profile = tool.permission_profile(&input);
    assert!(profile.iter().any(|facet| facet.nature == ToolNature::Read));
    assert!(
        profile
            .iter()
            .any(|facet| facet.nature == ToolNature::Write)
    );
    assert!(
        profile
            .iter()
            .any(|facet| facet.nature == ToolNature::Network)
    );
    let result = tool.execute(input).await;
    server.await.expect("proxy completed");
    assert!(!result.is_error, "{}", result.content);
    let id = result
        .content
        .split("recovery_id: ")
        .nth(1)
        .expect("receipt")
        .trim();
    assert_eq!(std::fs::read(&path).expect("download"), b"downloaded");
    let restore = serde_json::json!({"url":"ignored", "destination":path, "recovery_id":id});
    let profile = tool.permission_profile(&restore);
    assert_eq!(profile.len(), 2);
    assert!(
        profile
            .iter()
            .all(|facet| facet.nature != ToolNature::Network)
    );
    std::fs::write(&path, b"user edit").expect("user edit");
    assert!(tool.execute(restore.clone()).await.is_error);
    assert_eq!(std::fs::read(&path).expect("preserved"), b"user edit");
    std::fs::write(&path, b"downloaded").expect("restore expected state");
    let result = tool.execute(restore.clone()).await;
    assert!(!result.is_error, "{}", result.content);
    assert_eq!(std::fs::read(&path).expect("restored"), b"original");
    assert!(tool.execute(restore).await.is_error);
}

#[tokio::test]
async fn recovery_receipt_cannot_restore_another_destination_or_instance() {
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("download");
    let other = root.path().join("other");
    std::fs::write(&path, b"original").expect("original");
    std::fs::write(&other, b"downloaded").expect("other");
    let (tool, server) = tool_with_download(b"downloaded").await;
    let result = tool.execute(serde_json::json!({"url":"http://93.184.216.34/fixture", "destination":path,"retain_backup":true})).await;
    server.await.expect("proxy completed");
    assert!(!result.is_error, "{}", result.content);
    let id = result
        .content
        .split("recovery_id: ")
        .nth(1)
        .expect("receipt")
        .trim();
    let wrong_path = serde_json::json!({"url":"ignored","destination":other,"recovery_id":id});
    assert!(tool.execute(wrong_path).await.is_error);
    let restore = serde_json::json!({"url":"ignored","destination":path,"recovery_id":id});
    assert!(SaveUrlTool::new().execute(restore.clone()).await.is_error);
    assert!(!tool.execute(restore).await.is_error);
}

#[tokio::test]
async fn invalid_recovery_options_do_not_mutate_the_destination() {
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("download");
    std::fs::write(&path, b"original").expect("original");
    let tool = SaveUrlTool::new();
    for options in [
        serde_json::json!({"retain_backup":"yes"}),
        serde_json::json!({"recovery_id":true}),
        serde_json::json!({"retain_backup":true,"recovery_id":"id"}),
    ] {
        let mut input = serde_json::json!({"url":"invalid","destination":path});
        input
            .as_object_mut()
            .expect("object")
            .extend(options.as_object().expect("options").clone());
        assert!(tool.execute(input).await.is_error);
    }
    assert_eq!(std::fs::read(path).expect("unchanged"), b"original");
}
