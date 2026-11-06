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

/// Read an explicitly selected credential file. The caller owns the dispatch deadline.
/// Never fall back to metadata or another account when a selected file is invalid.
pub(crate) async fn read_credentials(
    path: &std::path::Path,
    deadline: std::time::Instant,
) -> ProviderResult<Value> {
    use tokio::io::AsyncReadExt;
    tokio::time::timeout_at(deadline.into(), async {
        let file = tokio::fs::File::open(path).await
            .map_err(|_| auth_error("cannot open Vertex credential file"))?;
        let mut bytes = Vec::new();
        file.take((MAX_CREDENTIAL_BYTES + 1) as u64).read_to_end(&mut bytes).await
            .map_err(|_| auth_error("cannot read Vertex credential file"))?;
        parse_credentials(&bytes)
    }).await.map_err(|_| auth_error("Vertex credential file read timed out"))?
}

/// Load local ADC only after explicit Vertex dispatch. The caller resolves platform paths.
/// Only absence of the well-known file permits a subsequent metadata decision; an explicitly
/// selected file, permission failure, or invalid file must never switch accounts silently.
pub(crate) async fn read_local_adc(
    explicit: Option<&std::path::Path>,
    well_known: &std::path::Path,
    deadline: std::time::Instant,
) -> ProviderResult<Option<Value>> {
    if deadline <= std::time::Instant::now() {
        return Err(auth_error("Vertex credential file read timed out"));
    }
    if let Some(path) = explicit {
        return read_credentials(path, deadline).await.map(Some);
    }
    use tokio::io::AsyncReadExt;
    tokio::time::timeout_at(deadline.into(), async {
        let file = match tokio::fs::File::open(well_known).await {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err(auth_error("cannot open Vertex credential file")),
        };
        let mut bytes = Vec::new();
        file.take((MAX_CREDENTIAL_BYTES + 1) as u64).read_to_end(&mut bytes).await
            .map_err(|_| auth_error("cannot read Vertex credential file"))?;
        parse_credentials(&bytes).map(Some)
    }).await.map_err(|_| auth_error("Vertex credential file read timed out"))?
}

#[cfg(test)]
mod file_tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[tokio::test]
    async fn local_adc_only_falls_through_for_absent_well_known_file() {
        let dir = tempfile::tempdir().expect("directory");
        let path = dir.path().join("adc.json");
        let deadline = || Instant::now() + Duration::from_secs(2);
        assert!(read_local_adc(None, &path, deadline()).await.expect("absent").is_none());
        assert!(read_local_adc(Some(&path), &path, deadline()).await.is_err());
        std::fs::write(&path, b"invalid JSON").expect("write");
        assert!(read_local_adc(None, &path, deadline()).await.is_err());
        let valid = br#"{"type":"authorized_user","client_id":"fixture-client","client_secret":"fixture-secret","refresh_token":"fixture-refresh"}"#;
        std::fs::write(&path, valid).expect("write");
        assert_eq!(read_local_adc(None, &path, deadline()).await.expect("valid"),
            Some(parse_credentials(valid).expect("credentials")));
        let missing = dir.path().join("explicit-missing.json");
        assert!(read_local_adc(Some(&missing), &path, deadline()).await.is_err());
        assert!(read_local_adc(None, &path, Instant::now() - Duration::from_secs(1)).await.is_err());
    }

    #[tokio::test]
    async fn explicit_invalid_file_fails_without_account_fallback_or_path_disclosure() {
        let dir = tempfile::tempdir().expect("directory");
        let path = dir.path().join("private-account-name.json");
        std::fs::write(&path, b"not credential JSON").expect("write");
        let error = read_credentials(&path, Instant::now() + Duration::from_secs(2))
            .await.expect_err("invalid file").to_string();
        assert!(!error.contains("private-account-name"));
        std::fs::write(&path, vec![b' '; MAX_CREDENTIAL_BYTES + 1]).expect("large file");
        assert!(read_credentials(&path, Instant::now() + Duration::from_secs(2)).await.is_err());
    }
}

/// Request-owned OAuth refresh for admitted user and service-account credentials.
#[derive(Clone)]
pub(crate) struct VertexTokenSource {
    client: reqwest::Client,
    credentials: std::sync::Arc<Value>,
    cache: std::sync::Arc<tokio::sync::Mutex<Option<CachedToken>>>,
    endpoint: String,
    metadata: bool,
}

struct CachedToken {
    token: String,
    refresh_at: std::time::Instant,
}

impl std::fmt::Debug for VertexTokenSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VertexTokenSource").finish_non_exhaustive()
    }
}

fn auth_error(message: &'static str) -> ProviderError {
    ProviderError::InvalidResponse(message.into())
}

/// Use only the official local service-account signer, never its token cache or ADC builder.
async fn service_account_assertion(credentials: &Value) -> ProviderResult<String> {
    use base64::Engine;
    use futures_util::FutureExt;
    use std::time::{SystemTime, UNIX_EPOCH};
    let key = credentials["private_key"].as_str()
        .ok_or_else(|| auth_error("invalid Vertex signing key"))?;
    if key.len() > 16 * 1024 {
        return Err(auth_error("Vertex signing key exceeds limit"));
    }
    let issued = SystemTime::now().duration_since(UNIX_EPOCH)
        .map_err(|_| auth_error("invalid Vertex signing clock"))?.as_secs();
    let expires = issued.checked_add(3600)
        .ok_or_else(|| auth_error("invalid Vertex signing clock"))?;
    let encode = |v: Value| base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(v.to_string());
    let header = encode(serde_json::json!({"alg":"RS256", "typ":"JWT", "kid":credentials["private_key_id"]}));
    let claims = encode(serde_json::json!({
        "iss":credentials["client_email"],
        "scope":"https://www.googleapis.com/auth/cloud-platform",
        "aud":"https://oauth2.googleapis.com/token", "iat":issued, "exp":expires,
    }));
    let input = format!("{header}.{claims}");
    let signer = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        google_cloud_auth::credentials::service_account::Builder::new(credentials.clone()).build_signer()
    })).map_err(|_| auth_error("Vertex signer initialization failed"))?
        .map_err(|_| auth_error("Vertex signer initialization failed"))?;
    let signature = std::panic::AssertUnwindSafe(signer.sign(input.as_bytes())).catch_unwind().await
        .map_err(|_| auth_error("Vertex signing failed"))?
        .map_err(|_| auth_error("Vertex signing failed"))?;
    Ok(format!("{input}.{}", base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(signature)))
}

impl VertexTokenSource {
    pub(crate) fn from_bytes(bytes: &[u8]) -> ProviderResult<Self> {
        let credentials = parse_credentials(bytes)?;
        Ok(Self {
            client: reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build().map_err(|_| auth_error("Vertex auth client initialization failed"))?,
            credentials: std::sync::Arc::new(credentials),
            cache: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
            endpoint: "https://oauth2.googleapis.com/token".into(),
            metadata: false,
        })
    }

    /// Construct only after explicit Vertex ADC selection and absent local credentials.
    /// Ignore metadata host environment overrides and never send this request through proxies.
    pub(crate) fn metadata() -> ProviderResult<Self> {
        Ok(Self {
            client: reqwest::Client::builder().no_proxy()
                .redirect(reqwest::redirect::Policy::none())
                .build().map_err(|_| auth_error("Vertex metadata client initialization failed"))?,
            credentials: std::sync::Arc::new(Value::Null),
            cache: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
            endpoint: "http://169.254.169.254/computeMetadata/v1/instance/service-accounts/default/token".into(),
            metadata: true,
        })
    }

    /// Lock acquisition and refresh share the caller's deadline. No background refresh is spawned.
    pub(crate) async fn token(&self, deadline: std::time::Instant) -> ProviderResult<String> {
        if deadline <= std::time::Instant::now() {
            return Err(auth_error("Vertex credential acquisition timed out"));
        }
        tokio::time::timeout_at(deadline.into(), self.refresh(deadline)).await
            .map_err(|_| auth_error("Vertex credential acquisition timed out"))?
    }

    async fn refresh(&self, deadline: std::time::Instant) -> ProviderResult<String> {
        use std::time::{Duration, Instant};
        let mut cached = self.cache.lock().await;
        if let Some(token) = cached.as_ref().filter(|t| Instant::now() < t.refresh_at) {
            return Ok(token.token.clone());
        }
        let started = Instant::now();
        let body = if self.metadata {
            Value::Null
        } else if self.credentials["type"] == "service_account" {
            serde_json::json!({
                "grant_type": "urn:ietf:params:oauth:grant-type:jwt-bearer",
                "assertion": service_account_assertion(&self.credentials).await?,
            })
        } else {
            serde_json::json!({
                "grant_type": "refresh_token",
                "client_id": self.credentials["client_id"],
                "client_secret": self.credentials["client_secret"],
                "refresh_token": self.credentials["refresh_token"],
            })
        };
        if std::time::Instant::now() >= deadline {
            return Err(auth_error("Vertex credential acquisition timed out"));
        }
        let request = if self.metadata {
            self.client.get(&self.endpoint).header("Metadata-Flavor", "Google")
        } else {
            self.client.post(&self.endpoint).json(&body)
        };
        let mut response = request.send().await
            .map_err(|_| auth_error("Vertex token request failed"))?;
        if self.metadata && response.headers().get("Metadata-Flavor")
            .is_none_or(|value| value.as_bytes() != b"Google") {
            return Err(auth_error("invalid Vertex metadata response origin"));
        }
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

    fn source() -> VertexTokenSource {
        VertexTokenSource::from_bytes(br#"{"type":"authorized_user","client_id":"fixture-client","client_secret":"fixture-secret","refresh_token":"fixture-refresh"}"#).expect("credentials")
    }
    fn deadline() -> Instant { Instant::now() + Duration::from_secs(5) }

    #[tokio::test]
    async fn metadata_requires_flavor_and_caches_only_valid_response() {
        for flavor in [None, Some("Other"), Some("Google")] {
            let mut server = mockito::Server::new_async().await;
            let mut mock = server.mock("GET", "/").match_header("Metadata-Flavor", "Google")
                .match_header("authorization", mockito::Matcher::Missing)
                .with_body(r#"{"access_token":"fixture-metadata","token_type":"Bearer","expires_in":3600}"#)
                .expect(1);
            if let Some(value) = flavor { mock = mock.with_header("Metadata-Flavor", value); }
            let mock = mock.create_async().await;
            let mut auth = VertexTokenSource::metadata().expect("metadata source");
            assert!(auth.endpoint.starts_with("http://169.254.169.254/"));
            auth.endpoint = server.url();
            let result = auth.token(deadline()).await;
            if flavor == Some("Google") {
                assert_eq!(result.expect("token"), "fixture-metadata");
                assert_eq!(auth.token(deadline()).await.expect("cached"), "fixture-metadata");
            } else {
                assert!(result.is_err());
                assert!(auth.cache.lock().await.is_none());
            }
            mock.assert_async().await;
        }
    }

    #[tokio::test]
    async fn metadata_redirect_cannot_relocate_request() {
        let mut server = mockito::Server::new_async().await;
        let mut target = mockito::Server::new_async().await;
        let forbidden = target.mock("GET", "/").expect(0).create_async().await;
        let redirect = server.mock("GET", "/").with_status(302)
            .with_header("Metadata-Flavor", "Google").with_header("Location", &target.url())
            .create_async().await;
        let mut auth = VertexTokenSource::metadata().expect("source");
        auth.endpoint = server.url();
        assert!(auth.token(deadline()).await.is_err());
        assert!(auth.cache.lock().await.is_none());
        redirect.assert_async().await;
        forbidden.assert_async().await;
    }

    #[tokio::test]
    async fn service_account_exchanges_signed_assertion_and_caches_token() {
        use base64::Engine;
        let credentials = serde_json::json!({
            "type":"service_account", "client_email":"fixture@example.invalid",
            "private_key_id":"fixture-key", "project_id":"fixture-project",
            "private_key":include_str!("../tests/fixtures/gemini/test-only-service-account.pem"),
        });
        let assertion = service_account_assertion(&credentials).await.expect("sign");
        let parts: Vec<_> = assertion.split('.').collect();
        assert_eq!(parts.len(), 3);
        let decode = |s: &str| -> Value {
            serde_json::from_slice(&base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(s).expect("base64")).expect("json")
        };
        assert_eq!(decode(parts[0])["alg"], "RS256");
        let claims = decode(parts[1]);
        assert_eq!(claims["aud"], "https://oauth2.googleapis.com/token");
        assert_eq!(claims["exp"].as_u64().expect("expiry") - claims["iat"].as_u64().expect("issued"), 3600);
        assert_eq!(base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(parts[2]).expect("signature").len(), 256);
        let mut server = mockito::Server::new_async().await;
        let mock = server.mock("POST", "/")
            .match_body(mockito::Matcher::PartialJson(serde_json::json!({"grant_type":"urn:ietf:params:oauth:grant-type:jwt-bearer"})))
            .with_body(r#"{"access_token":"fixture-token","token_type":"Bearer","expires_in":3600}"#)
            .expect(1).create_async().await;
        let mut auth = VertexTokenSource::from_bytes(credentials.to_string().as_bytes()).expect("source");
        auth.endpoint = server.url();
        assert_eq!(auth.token(deadline()).await.expect("exchange"), "fixture-token");
        assert_eq!(auth.token(deadline()).await.expect("cached"), "fixture-token");
        mock.assert_async().await;
        let mut invalid = credentials;
        invalid["private_key"] = "fixture-invalid-private-key".into();
        let error = service_account_assertion(&invalid).await.expect_err("invalid key").to_string();
        assert!(!error.contains("fixture-invalid-private-key"));
    }

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
    async fn invalid_token_responses_never_enter_cache_or_errors() {
        for body in [
            r#"{"access_token":"fixture-secret","token_type":"Bearer","expires_in":0}"#.to_owned(),
            r#"{"access_token":"fixture-secret","token_type":"Other","expires_in":3600}"#.to_owned(),
            r#"{"access_token":"bad\r\nheader","token_type":"Bearer","expires_in":3600}"#.to_owned(),
            r#"{"access_token":"fixture-secret","token_type":"Bearer","expires_in":86401}"#.to_owned(),
            "x".repeat(64 * 1024 + 1),
            "not JSON fixture-secret".into(),
        ] {
            let mut server = mockito::Server::new_async().await;
            let mock = server.mock("POST", "/").with_body(body).create_async().await;
            let mut auth = source();
            auth.endpoint = server.url();
            let error = auth.token(deadline()).await.expect_err("invalid response").to_string();
            assert!(!error.contains("fixture-secret"));
            assert!(auth.cache.lock().await.is_none());
            mock.assert_async().await;
        }
    }

    #[tokio::test]
    async fn expired_deadline_cannot_return_cached_credentials() {
        let auth = source();
        *auth.cache.lock().await = Some(CachedToken {
            token: "fixture-cached-token".into(),
            refresh_at: Instant::now() + Duration::from_secs(3600),
        });
        let error = auth.token(Instant::now() - Duration::from_secs(1))
            .await.expect_err("expired caller deadline");
        assert!(!error.to_string().contains("fixture-cached-token"));
        assert!(auth.cache.try_lock().is_ok());
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
