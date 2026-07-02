//! Read-only loopback dashboard server for Talos (ADR-031).
//!
//! Binds to `127.0.0.1:0` (OS-assigned port) and serves GET-only routes from a
//! pre-computed [`DashboardSnapshot`]. By default (`loopback_only = true`) the
//! per-process bearer token is skipped and the loopback bind is the only access
//! control. Set `loopback_only = false` to require `Authorization: Bearer
//! <token>` on every request. No write, action, or tool-execution routes are
//! registered.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::extract::{Request, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Json, Response};
use axum::routing::get;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DashboardError {
    #[error("dashboard listener failed: {0}")]
    Bind(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardSnapshot {
    pub config_masked: String,
    pub status: Value,
    pub history: Value,
    pub governance: String,
}

#[derive(Clone)]
struct AppState {
    token: String,
    snapshot: Arc<DashboardSnapshot>,
    loopback_only: bool,
}

pub struct DashboardServer {
    state: AppState,
}

impl DashboardServer {
    pub fn new(snapshot: DashboardSnapshot) -> Self {
        Self::with_loopback_only(snapshot, false)
    }

    /// Create a dashboard server with explicit loopback-only control.
    ///
    /// When `loopback_only` is `true`, the bearer token middleware is skipped
    /// and the server relies on the `127.0.0.1` bind as the only access
    /// control. Callers should set this to `true` only when the user has
    /// explicitly opted in via `[dashboard] loopback_only = true` in config.
    pub fn with_loopback_only(snapshot: DashboardSnapshot, loopback_only: bool) -> Self {
        Self {
            state: AppState {
                token: Uuid::new_v4().simple().to_string(),
                snapshot: Arc::new(redact_snapshot(snapshot)),
                loopback_only,
            },
        }
    }

    pub fn token(&self) -> &str {
        &self.state.token
    }

    pub async fn serve(&self) -> Result<(SocketAddr, tokio::task::JoinHandle<()>), DashboardError> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        let app = self.build_router();
        let handle = tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        Ok((addr, handle))
    }

    fn build_router(&self) -> Router {
        let state = self.state.clone();
        let router = Router::new()
            .route("/status", get(status_handler))
            .route("/history", get(history_handler))
            .route("/governance", get(governance_handler))
            .route("/config", get(config_handler))
            .route("/", get(root_handler))
            .fallback(not_found_handler);
        if state.loopback_only {
            router.with_state(state)
        } else {
            router
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    auth_middleware,
                ))
                .with_state(state)
        }
    }
}

async fn auth_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let expected = format!("Bearer {}", state.token);
    let authorized = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v == expected);
    if authorized {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

fn apply_security_headers(resp: &mut Response, content_type: &'static str) {
    let headers = resp.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    if content_type.starts_with("text/html") {
        headers.insert(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'none'; style-src 'unsafe-inline'"),
        );
    }
}

async fn root_handler() -> Response {
    let body = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Talos Dashboard</title>
  <style>
    body { font-family: system-ui, sans-serif; margin: 2rem; line-height: 1.5; }
    a { display: block; margin: .5rem 0; }
  </style>
</head>
<body>
  <h1>Talos Dashboard</h1>
  <a href="/status">Status</a>
  <a href="/history">History</a>
  <a href="/governance">Governance</a>
  <a href="/config">Config</a>
</body>
</html>"#;
    let mut resp = body.into_response();
    apply_security_headers(&mut resp, "text/html; charset=utf-8");
    resp
}

async fn status_handler(State(state): State<AppState>) -> Response {
    let mut resp = Json(state.snapshot.status.clone()).into_response();
    apply_security_headers(&mut resp, "application/json; charset=utf-8");
    resp
}

async fn history_handler(State(state): State<AppState>) -> Response {
    let mut resp = Json(state.snapshot.history.clone()).into_response();
    apply_security_headers(&mut resp, "application/json; charset=utf-8");
    resp
}

async fn governance_handler(State(state): State<AppState>) -> Response {
    let mut resp = state.snapshot.governance.clone().into_response();
    apply_security_headers(&mut resp, "text/plain; charset=utf-8");
    resp
}

async fn config_handler(State(state): State<AppState>) -> Response {
    let mut resp = state.snapshot.config_masked.clone().into_response();
    apply_security_headers(&mut resp, "text/plain; charset=utf-8");
    resp
}

async fn not_found_handler() -> StatusCode {
    StatusCode::NOT_FOUND
}

fn redact_snapshot(snapshot: DashboardSnapshot) -> DashboardSnapshot {
    DashboardSnapshot {
        config_masked: redact_text(&snapshot.config_masked),
        status: redact_value(snapshot.status),
        history: redact_value(snapshot.history),
        governance: redact_text(&snapshot.governance),
    }
}

fn redact_value(value: Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(key, value)| {
                    if is_sensitive_key(&key) {
                        (key, Value::String("***".to_string()))
                    } else {
                        (key, redact_value(value))
                    }
                })
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.into_iter().map(redact_value).collect()),
        Value::String(value) => Value::String(redact_text(&value)),
        other => other,
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    key.contains("api_key")
        || key.contains("token")
        || key.contains("secret")
        || key.contains("password")
        || key.contains("authorization")
        || key.contains("credential")
        || key.contains("cookie")
        || key == "auth"
        || key == "key"
}

fn redact_text(input: &str) -> String {
    const KEYS: &[&str] = &[
        "api_key",
        "access_token",
        "refresh_token",
        "token",
        "secret",
        "password",
        "auth",
        "sig",
        "signature",
        "key",
    ];

    let mut output = input.to_string();
    for key in KEYS {
        output = redact_assignment_values(&output, key);
    }
    output
}

fn redact_assignment_values(input: &str, key: &str) -> String {
    let lower = input.to_ascii_lowercase();
    let mut output = String::with_capacity(input.len());
    let mut cursor = 0;

    while let Some(offset) = lower[cursor..].find(key) {
        let start = cursor + offset;
        let key_start_ok = start == 0
            || matches!(
                input.as_bytes().get(start - 1),
                Some(b'?' | b'&' | b';' | b' ' | b'\n' | b'\t' | b'"' | b'\'')
            );
        let key_end = start + key.len();
        let Some(eq_relative) = input[key_end..].find('=') else {
            output.push_str(&input[cursor..key_end]);
            cursor = key_end;
            continue;
        };
        let eq_pos = key_end + eq_relative;
        let only_space_before_equals = input[key_end..eq_pos]
            .chars()
            .all(|c| matches!(c, ' ' | '\t'));

        if !key_start_ok {
            output.push_str(&input[cursor..key_end]);
            cursor = key_end;
            continue;
        }
        if !only_space_before_equals {
            output.push_str(&input[cursor..eq_pos + 1]);
            cursor = eq_pos + 1;
            continue;
        }

        let value_prefix_start = eq_pos + 1;
        let value_start = value_prefix_start
            + input[value_prefix_start..]
                .find(|c: char| !matches!(c, ' ' | '\t'))
                .unwrap_or(0);
        let value_mask_start = if matches!(input.as_bytes().get(value_start), Some(b'"' | b'\'')) {
            value_start + 1
        } else {
            value_start
        };
        let value_end = input[value_mask_start..]
            .find(['&', ';', '"', '\'', '\n', '\r', '\t', ' '])
            .map(|end| value_mask_start + end)
            .unwrap_or(input.len());

        output.push_str(&input[cursor..value_mask_start]);
        output.push_str("***");
        cursor = value_end;
    }

    output.push_str(&input[cursor..]);
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Method;

    fn test_snapshot() -> DashboardSnapshot {
        DashboardSnapshot {
            config_masked: "provider = \"anthropic\"\napi_key = \"***\"".to_string(),
            status: serde_json::json!({"model": "test-model", "sessions": 3}),
            history: serde_json::json!([{"id": "abc", "preview": "hello"}]),
            governance: "Now: test item".to_string(),
        }
    }

    fn build_test_app() -> (Router, String) {
        let server = DashboardServer::new(test_snapshot());
        let token = server.token().to_string();
        (server.build_router(), token)
    }

    async fn request(
        app: &Router,
        method: Method,
        path: &str,
        token: Option<&str>,
    ) -> (StatusCode, String) {
        let mut req = Request::builder()
            .method(method)
            .uri(path)
            .body(axum::body::Body::empty())
            .expect("failed to build request");
        if let Some(t) = token {
            req.headers_mut().insert(
                header::AUTHORIZATION,
                format!("Bearer {t}").parse().expect("valid header value"),
            );
        }
        let response = tower::ServiceExt::oneshot(app.clone(), req)
            .await
            .expect("request failed");
        let status = response.status();
        let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap_or_default();
        (status, String::from_utf8_lossy(&bytes).to_string())
    }

    #[tokio::test]
    async fn token_rejection_no_auth_header() {
        let (app, _token) = build_test_app();
        let (status, _) = request(&app, Method::GET, "/status", None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn token_rejection_wrong_token() {
        let (app, _token) = build_test_app();
        let (status, _) = request(&app, Method::GET, "/status", Some("wrong")).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn valid_token_returns_status() {
        let (app, token) = build_test_app();
        let (status, body) = request(&app, Method::GET, "/status", Some(&token)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("test-model"));
    }

    #[tokio::test]
    async fn valid_token_returns_config_masked() {
        let (app, token) = build_test_app();
        let (status, body) = request(&app, Method::GET, "/config", Some(&token)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("***"));
        assert!(!body.contains("sk-ant-"));
    }

    #[tokio::test]
    async fn valid_token_returns_root_index() {
        let (app, token) = build_test_app();
        let (status, body) = request(&app, Method::GET, "/", Some(&token)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("Talos Dashboard"));
        assert!(body.contains("/governance"));
    }

    #[tokio::test]
    async fn root_index_requires_token() {
        let (app, _token) = build_test_app();
        let (status, _) = request(&app, Method::GET, "/", None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn valid_token_returns_governance() {
        let (app, token) = build_test_app();
        let (status, body) = request(&app, Method::GET, "/governance", Some(&token)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("Now: test item"));
    }

    #[tokio::test]
    async fn valid_token_returns_history() {
        let (app, token) = build_test_app();
        let (status, body) = request(&app, Method::GET, "/history", Some(&token)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("abc"));
    }

    #[tokio::test]
    async fn no_write_routes_registered() {
        let (app, token) = build_test_app();
        for method in [Method::POST, Method::PUT, Method::DELETE, Method::PATCH] {
            for path in ["/", "/status", "/history", "/governance", "/config"] {
                let (status, _) = request(&app, method.clone(), path, Some(&token)).await;
                assert_eq!(
                    status,
                    StatusCode::METHOD_NOT_ALLOWED,
                    "{method} {path} should be rejected"
                );
            }
        }
    }

    #[tokio::test]
    async fn unknown_path_returns_404_even_with_valid_token() {
        let (app, token) = build_test_app();
        let (status, _) = request(&app, Method::GET, "/admin", Some(&token)).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn unknown_path_without_token_is_rejected() {
        let (app, _token) = build_test_app();
        let (status, _) = request(&app, Method::GET, "/admin", None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn snapshot_outputs_are_redacted_at_boundary() {
        let snapshot = DashboardSnapshot {
            config_masked: "api_key = \"sk-live\"\ntoken=abc".to_string(),
            status: serde_json::json!({
                "model": "test",
                "api_key": "sk-live",
                "url": "https://example.com/?token=abc&ok=1",
            }),
            history: serde_json::json!([
                {
                    "tool": "http_request",
                    "headers": {
                        "Authorization": "Bearer secret",
                        "Cookie": "sid=secret"
                    },
                    "url": "https://example.com/?api_key=sk-live&ok=1"
                }
            ]),
            governance: "refresh_token=abc status=ok".to_string(),
        };
        let server = DashboardServer::new(snapshot);
        let token = server.token().to_string();
        let app = server.build_router();

        for path in ["/status", "/history", "/governance", "/config"] {
            let (status, body) = request(&app, Method::GET, path, Some(&token)).await;
            assert_eq!(status, StatusCode::OK);
            assert!(!body.contains("sk-live"), "{path} leaked api key: {body}");
            assert!(
                !body.contains("Bearer secret"),
                "{path} leaked bearer: {body}"
            );
            assert!(!body.contains("sid=secret"), "{path} leaked cookie: {body}");
            assert!(
                !body.contains("token=abc"),
                "{path} leaked token query: {body}"
            );
            assert!(body.contains("***"), "{path} did not redact: {body}");
        }
    }

    #[tokio::test]
    async fn binds_to_loopback_only() {
        let server = DashboardServer::new(test_snapshot());
        let (addr, handle) = server.serve().await.unwrap();
        assert_eq!(addr.ip().to_string(), "127.0.0.1");
        handle.abort();
    }

    #[test]
    fn token_is_crypto_random_per_instance() {
        let s1 = DashboardServer::new(test_snapshot());
        let s2 = DashboardServer::new(test_snapshot());
        assert_ne!(s1.token(), s2.token());
        assert_eq!(s1.token().len(), 32);
    }

    fn build_loopback_only_app() -> Router {
        let server = DashboardServer::with_loopback_only(test_snapshot(), true);
        server.build_router()
    }

    #[tokio::test]
    async fn loopback_only_no_token_required() {
        let app = build_loopback_only_app();
        for path in ["/status", "/history", "/governance", "/config", "/"] {
            let (status, _) = request(&app, Method::GET, path, None).await;
            assert_eq!(
                status,
                StatusCode::OK,
                "GET {path} should succeed without token"
            );
        }
    }

    #[tokio::test]
    async fn loopback_only_token_header_ignored() {
        let app = build_loopback_only_app();
        let (status, body) = request(&app, Method::GET, "/status", Some("any-value")).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("test-model"));
    }

    #[tokio::test]
    async fn loopback_only_still_serves_governance() {
        let app = build_loopback_only_app();
        let (status, body) = request(&app, Method::GET, "/governance", None).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("Now: test item"));
    }

    #[tokio::test]
    async fn loopback_only_binds_loopback() {
        let server = DashboardServer::with_loopback_only(test_snapshot(), true);
        let (addr, handle) = server.serve().await.unwrap();
        assert_eq!(addr.ip().to_string(), "127.0.0.1");
        handle.abort();
    }

    #[tokio::test]
    async fn token_mode_still_rejects_without_token() {
        let (app, _token) = build_test_app();
        let (status, _) = request(&app, Method::GET, "/status", None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
}
