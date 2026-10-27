use tokio::io::AsyncReadExt;

pub(super) async fn read_request_headers(stream: &mut tokio::net::TcpStream) {
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        let mut headers = Vec::new();
        let mut buffer = [0; 1024];
        loop {
            let count = stream.read(&mut buffer).await.expect("request headers");
            assert_ne!(count, 0, "connection closed before complete headers");
            headers.extend_from_slice(&buffer[..count]);
            assert!(headers.len() <= 16 * 1024, "request headers too large");
            if headers.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
        }
    })
    .await
    .expect("request headers timeout");
}
