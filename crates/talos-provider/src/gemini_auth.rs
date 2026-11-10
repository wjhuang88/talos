//! Vertex credential admission. Network discovery belongs to explicit Vertex dispatch only.

use serde_json::Value;
use talos_core::provider::{ProviderError, ProviderResult};

const MAX_CREDENTIAL_BYTES: usize = 1024 * 1024;

/// Admit supported credential files before constructing any signer or HTTP request.
/// Error messages intentionally exclude source JSON, paths and upstream errors.
pub(crate) fn parse_credentials(bytes: &[u8]) -> ProviderResult<Value> {
    let invalid = || ProviderError::InvalidResponse("invalid or unsupported Vertex credentials".into());
    if bytes.len() > MAX_CREDENTIAL_BYTES {
        return Err(invalid());
    }
    let value: Value = serde_json::from_slice(bytes).map_err(|_| invalid())?;
    if !matches!(value.get("type").and_then(Value::as_str), Some("service_account" | "authorized_user"))
        || value.get("credential_source").is_some()
        || value.get("service_account_impersonation_url").is_some()
        || value.get("universe_domain").is_some_and(|v| v.as_str() != Some("googleapis.com"))
        || value.get("token_uri").is_some_and(|v| !matches!(v.as_str(),
            Some("https://oauth2.googleapis.com/token" | "https://accounts.google.com/o/oauth2/token")))
    {
        return Err(invalid());
    }
    let fields: &[&str] = if value["type"] == "service_account" {
        &["client_email", "private_key_id", "private_key", "project_id"]
    } else {
        &["client_id", "client_secret", "refresh_token"]
    };
    if fields.iter().any(|name| value.get(*name).and_then(Value::as_str).is_none_or(str::is_empty)) {
        return Err(invalid());
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn admits_user_refresh_credentials() {
        let v = json!({"type":"authorized_user", "client_id":"fixture-client",
            "client_secret":"fixture-secret", "refresh_token":"fixture-refresh"});
        assert_eq!(parse_credentials(v.to_string().as_bytes()).expect("valid"), v);
    }

    #[test]
    fn rejects_execution_relocation_and_oversized_input_without_disclosure() {
        for v in [
            json!({"type":"external_account", "credential_source":{"executable":{"command":"fixture"}}}),
            json!({"type":"authorized_user", "token_uri":"https://example.invalid/fixture-secret"}),
            json!({"type":"service_account", "universe_domain":"example.invalid"}),
        ] {
            let error = parse_credentials(v.to_string().as_bytes()).expect_err("reject");
            assert!(!error.to_string().contains("fixture-secret"));
        }
        assert!(parse_credentials(&vec![b' '; MAX_CREDENTIAL_BYTES + 1]).is_err());
    }
}

/// Request-owned authorized-user refresh. Service-account signing is integrated separately.
#[derive(Clone)]
pub(crate) struct UserTokenSource {
    client: reqwest::Client,
    credentials: std::sync::Arc<Value>,
    cache: std::sync::Arc<tokio::sync::Mutex<Option<CachedToken>>>,
    endpoint: String,
}

struct CachedToken {
    token: String,
    refresh_at: std::time::Instant,
}

impl std::fmt::Debug for UserTokenSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserTokenSource").finish_non_exhaustive()
    }
}

fn auth_error(message: &'static str) -> ProviderError {
    ProviderError::InvalidResponse(message.into())
}

impl UserTokenSource {
    pub(crate) fn from_bytes(bytes: &[u8]) -> ProviderResult<Self> {
        let credentials = parse_credentials(bytes)?;
        if credentials["type"] != "authorized_user" {
            return Err(auth_error("expected Vertex authorized-user credentials"));
        }
        Ok(Self {
            client: reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build().map_err(|_| auth_error("Vertex auth client initialization failed"))?,
            credentials: std::sync::Arc::new(credentials),
            cache: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
            endpoint: "https://oauth2.googleapis.com/token".into(),
        })
    }

    /// Lock acquisition and refresh share the caller's deadline. No background refresh is spawned.
    pub(crate) async fn token(&self, deadline: std::time::Instant) -> ProviderResult<String> {
        tokio::time::timeout_at(deadline.into(), self.refresh()).await
            .map_err(|_| auth_error("Vertex credential acquisition timed out"))?
    }

    async fn refresh(&self) -> ProviderResult<String> {
        use std::time::{Duration, Instant};
        let mut cached = self.cache.lock().await;
        if let Some(token) = cached.as_ref().filter(|t| Instant::now() < t.refresh_at) {
            return Ok(token.token.clone());
        }
        let started = Instant::now();
        let body = serde_json::json!({
            "grant_type": "refresh_token",
            "client_id": self.credentials["client_id"],
            "client_secret": self.credentials["client_secret"],
            "refresh_token": self.credentials["refresh_token"],
        });
        let mut response = self.client.post(&self.endpoint).json(&body).send().await
            .map_err(|_| auth_error("Vertex token request failed"))?;
        if !response.status().is_success() {
            return Err(auth_error("Vertex token request rejected"));
        }
        const MAX_RESPONSE: usize = 64 * 1024;
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await
            .map_err(|_| auth_error("Vertex token response failed"))? {
            if chunk.len() > MAX_RESPONSE.saturating_sub(bytes.len()) {
                return Err(auth_error("Vertex token response exceeds limit"));
            }
            bytes.extend_from_slice(&chunk);
        }
        let response: Value = serde_json::from_slice(&bytes)
            .map_err(|_| auth_error("invalid Vertex token response"))?;
        let token = response["access_token"].as_str().filter(|s| !s.is_empty()
            && !s.bytes().any(|b| b.is_ascii_whitespace())
            && reqwest::header::HeaderValue::from_str(s).is_ok())
            .ok_or_else(|| auth_error("invalid Vertex access token"))?;
        if response["token_type"].as_str().is_none_or(|s| !s.eq_ignore_ascii_case("Bearer")) {
            return Err(auth_error("invalid Vertex token type"));
        }
        let lifetime = response["expires_in"].as_u64().filter(|n| *n > 0 && *n <= 86400)
            .ok_or_else(|| auth_error("invalid Vertex token lifetime"))?;
        if started + Duration::from_secs(lifetime) <= Instant::now() {
            return Err(auth_error("Vertex token expired during acquisition"));
        }
        *cached = Some(CachedToken {
            token: token.into(),
            refresh_at: started + Duration::from_secs(lifetime.saturating_sub(60)),
        });
        Ok(token.into())
    }
}

#[cfg(test)]
mod refresh_tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn source() -> UserTokenSource {
        UserTokenSource::from_bytes(br#"{"type":"authorized_user","client_id":"fixture-client","client_secret":"fixture-secret","refresh_token":"fixture-refresh"}"#).expect("credentials")
    }
    fn deadline() -> Instant { Instant::now() + Duration::from_secs(5) }

    #[tokio::test]
    async fn cache_reuses_valid_token_and_short_lifetime_refreshes() {
        let mut server = mockito::Server::new_async().await;
        let first = server.mock("POST", "/").with_body(r#"{"access_token":"first","token_type":"Bearer","expires_in":1}"#).expect(1).create_async().await;
        let mut auth = source();
        auth.endpoint = server.url();
        assert_eq!(auth.token(deadline()).await.expect("first"), "first");
        first.assert_async().await;
        first.remove_async().await;
        let next = server.mock("POST", "/").with_body(r#"{"access_token":"next","token_type":"Bearer","expires_in":3600}"#).expect(1).create_async().await;
        assert_eq!(auth.token(deadline()).await.expect("refresh"), "next");
        assert_eq!(auth.token(deadline()).await.expect("cached"), "next");
        next.assert_async().await;
        assert!(!format!("{auth:?}").contains("fixture-secret"));
    }

    #[tokio::test]
    async fn redirects_do_not_forward_credentials() {
        let mut server = mockito::Server::new_async().await;
        let mut target = mockito::Server::new_async().await;
        let forbidden = target.mock("POST", "/").expect(0).create_async().await;
        let redirect = server.mock("POST", "/").with_status(307)
            .with_header("Location", &target.url()).with_body("fixture-secret").create_async().await;
        let mut auth = source();
        auth.endpoint = server.url();
        let error = auth.token(deadline()).await.expect_err("redirect").to_string();
        assert!(!error.contains("fixture-secret"));
        redirect.assert_async().await;
        forbidden.assert_async().await;
    }

    #[tokio::test]
    async fn dropping_inflight_refresh_closes_connection_and_releases_lock() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let mut auth = source();
        auth.endpoint = format!("http://{}", listener.local_addr().expect("address"));
        let (entered_tx, entered_rx) = tokio::sync::oneshot::channel();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("accept");
            // Consume the complete request before testing EOF; unread request bytes are not EOF.
            let mut request = Vec::new();
            let mut byte = [0];
            while !request.ends_with(b"\r\n\r\n") {
                socket.read_exact(&mut byte).await.expect("request header");
                request.push(byte[0]);
                assert!(request.len() < 8192);
            }
            let headers = String::from_utf8(request).expect("headers");
            let length: usize = headers.lines().find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length").then(|| value.trim().parse().expect("length"))
            }).expect("content length");
            let mut body = vec![0; length];
            socket.read_exact(&mut body).await.expect("request body");
            socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\n").await.expect("response headers");
            entered_tx.send(()).expect("notify");
            tokio::time::timeout(Duration::from_secs(2), socket.read(&mut byte)).await
                .expect("connection closed").expect("EOF")
        });
        let mut refresh = Box::pin(auth.token(deadline()));
        tokio::select! {
            result = entered_rx => { result.expect("server ready"); },
            _ = &mut refresh => panic!("refresh ended before cancellation"),
        }
        drop(refresh);
        assert!(auth.cache.try_lock().is_ok());
        assert_eq!(server.await.expect("server"), 0);
        assert!(auth.cache.lock().await.is_none());
    }

    #[tokio::test]
    async fn deadline_bounds_cache_lock_wait() {
        let auth = source();
        let guard = auth.cache.lock().await;
        assert!(auth.token(Instant::now() + Duration::from_millis(5)).await.is_err());
        drop(guard);
        assert!(auth.cache.try_lock().is_ok());
    }
}
