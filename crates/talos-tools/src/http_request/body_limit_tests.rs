use super::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn response(
    body: &'static [u8],
    declared_length: usize,
) -> (reqwest::Response, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let address = listener.local_addr().expect("address");
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept");
        let mut buffer = [0; 4096];
        stream.read(&mut buffer).await.expect("request");
        stream
            .write_all(
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    declared_length
                )
                .as_bytes(),
            )
            .await
            .expect("headers");
        stream.write_all(body).await.expect("body");
    });
    let response = reqwest::Client::builder()
        .no_proxy()
        .build()
        .expect("client")
        .get(format!("http://{address}/"))
        .send()
        .await
        .expect("response");
    (response, server)
}

#[tokio::test]
async fn body_retains_only_limit_plus_one_and_reports_actual_truncation() {
    for (body, limit, expected, truncated) in [
        (&b""[..], 0, &b""[..], false),
        (&b"abc"[..], 3, &b"abc"[..], false),
        (&b"abc"[..], 4, &b"abc"[..], false),
        (&b"abcdef"[..], 3, &b"abcd"[..], true),
        (&b"abc"[..], 0, &b"a"[..], true),
    ] {
        let (response, server) = response(body, body.len()).await;
        let (bytes, actual_truncated) = read_bounded_body(response, limit).await.expect("read");
        assert_eq!(bytes, expected);
        assert_eq!(actual_truncated, truncated);
        server.await.expect("server");
    }
}

#[tokio::test]
async fn incomplete_body_below_limit_remains_an_error() {
    let (response, server) = response(b"abc", 10).await;
    assert!(read_bounded_body(response, 20).await.is_err());
    server.await.expect("server");
}

#[tokio::test]
async fn oversized_chunked_body_does_not_wait_for_end_of_stream() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let address = listener.local_addr().expect("address");
    let (finish, finished) = tokio::sync::oneshot::channel::<()>();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept");
        let mut buffer = [0; 4096];
        stream.read(&mut buffer).await.expect("request");
        stream
            .write_all(
                b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nabc\r\n3\r\ndef\r\n",
            )
            .await
            .expect("chunks");
        let _ = finished.await;
    });
    let response = reqwest::Client::builder()
        .no_proxy()
        .build()
        .expect("client")
        .get(format!("http://{address}/"))
        .send()
        .await
        .expect("response");
    let result = tokio::time::timeout(Duration::from_secs(2), read_bounded_body(response, 3)).await;
    let _ = finish.send(());
    server.await.expect("server");
    let (bytes, truncated) = result.expect("must not wait for EOF").expect("read");
    assert_eq!(bytes, b"abcd");
    assert!(truncated);
}
