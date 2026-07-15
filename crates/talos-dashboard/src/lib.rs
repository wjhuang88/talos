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
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
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
    pub extensions: Value,
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
        Self::with_loopback_only(snapshot, true)
    }

    /// Create a dashboard server with explicit loopback-only control.
    ///
    /// When `loopback_only` is `true`, the bearer token middleware is skipped
    /// and the server relies on the `127.0.0.1` bind as the only access
    /// control. Set this to `false` to require a per-process bearer token.
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
            .route("/extensions", get(extensions_handler))
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
  <a href="/extensions">Extensions</a>
</body>
</html>"#;
    let mut resp = body.into_response();
    apply_security_headers(&mut resp, "text/html; charset=utf-8");
    resp
}

async fn status_handler(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if accepts_html(&headers) {
        let mut resp = render_status_html(&state.snapshot).into_response();
        apply_security_headers(&mut resp, "text/html; charset=utf-8");
        resp
    } else {
        let mut resp = Json(state.snapshot.status.clone()).into_response();
        apply_security_headers(&mut resp, "application/json; charset=utf-8");
        resp
    }
}

async fn history_handler(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if accepts_html(&headers) {
        let mut resp = render_history_html(&state.snapshot).into_response();
        apply_security_headers(&mut resp, "text/html; charset=utf-8");
        resp
    } else {
        let mut resp = Json(state.snapshot.history.clone()).into_response();
        apply_security_headers(&mut resp, "application/json; charset=utf-8");
        resp
    }
}

async fn governance_handler(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if accepts_html(&headers) {
        let mut resp = render_governance_html(&state.snapshot).into_response();
        apply_security_headers(&mut resp, "text/html; charset=utf-8");
        resp
    } else {
        let mut resp = state.snapshot.governance.clone().into_response();
        apply_security_headers(&mut resp, "text/plain; charset=utf-8");
        resp
    }
}

async fn config_handler(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if accepts_html(&headers) {
        let mut resp = render_config_html(&state.snapshot).into_response();
        apply_security_headers(&mut resp, "text/html; charset=utf-8");
        resp
    } else {
        let mut resp = state.snapshot.config_masked.clone().into_response();
        apply_security_headers(&mut resp, "text/plain; charset=utf-8");
        resp
    }
}

async fn extensions_handler(State(state): State<AppState>) -> Response {
    let mut resp = state.snapshot.extensions.to_string().into_response();
    apply_security_headers(&mut resp, "application/json");
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
        extensions: redact_value(snapshot.extensions),
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

// ── HTML rendering helpers (I129) ──────────────────────────────────────────

/// Returns true only when the request's `Accept` header explicitly names
/// `text/html`. Requests with `*/*`, no `Accept`, or `application/json`
/// return false — preserving the existing JSON/plain-text API.
fn accepts_html(headers: &HeaderMap) -> bool {
    headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| {
            v.split(',')
                .any(|part| part.trim().starts_with("text/html"))
        })
}

/// Escape dynamic content for safe embedding in HTML text nodes and
/// attribute values. Every value rendered into an HTML page passes through
/// this function.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Render a `serde_json::Value` recursively as safe HTML.
fn render_value_html(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            if map.is_empty() {
                return r#"<span class="empty">(empty)</span>"#.to_string();
            }
            let mut rows = String::new();
            for (k, v) in map {
                rows.push_str(&format!(
                    "<tr><th>{}</th><td>{}</td></tr>",
                    html_escape(k),
                    render_value_html(v)
                ));
            }
            format!("<table>{rows}</table>")
        }
        Value::Array(items) => {
            if items.is_empty() {
                return r#"<span class="empty">(empty)</span>"#.to_string();
            }
            let mut lis = String::new();
            for item in items {
                lis.push_str(&format!("<li>{}</li>", render_value_html(item)));
            }
            format!("<ul>{lis}</ul>")
        }
        Value::String(s) => html_escape(s),
        Value::Null => r#"<em>null</em>"#.to_string(),
        other => html_escape(&other.to_string()),
    }
}

/// Shared HTML page wrapper with inline CSS and navigation.
fn render_html_page(title: &str, content: &str) -> String {
    let title = html_escape(title);
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title} — Talos Dashboard</title>
<style>
body {{ font-family: system-ui, sans-serif; margin: 1.5rem; line-height: 1.5; max-width: 64rem; }}
nav {{ margin-bottom: 1.5rem; padding-bottom: .5rem; border-bottom: 1px solid #ddd; }}
nav a {{ margin-right: 1rem; }}
h1 {{ margin-bottom: 1rem; }}
pre {{ background: #f4f4f4; padding: 1rem; overflow-x: auto; border-radius: .25rem; }}
table {{ border-collapse: collapse; width: 100%; }}
th {{ text-align: left; vertical-align: top; padding: .4rem .6rem; background: #f9f9f9; white-space: nowrap; }}
td {{ padding: .4rem .6rem; vertical-align: top; }}
tr {{ border-bottom: 1px solid #eee; }}
ul {{ padding-left: 1.5rem; }}
.empty {{ color: #999; font-style: italic; }}
</style>
</head>
<body>
<nav>
<a href="/">Home</a>
<a href="/status">Status</a>
<a href="/history">History</a>
<a href="/governance">Governance</a>
<a href="/config">Config</a>
</nav>
<h1>{title}</h1>
{content}
</body>
</html>"#
    )
}

fn render_status_html(snapshot: &DashboardSnapshot) -> String {
    let content = if snapshot.status.is_null()
        || (snapshot.status.is_object()
            && snapshot.status.as_object().is_some_and(|m| m.is_empty()))
    {
        r#"<p class="empty">No status data available.</p>"#.to_string()
    } else {
        render_value_html(&snapshot.status)
    };
    render_html_page("Status", &content)
}

fn render_history_html(snapshot: &DashboardSnapshot) -> String {
    let content = if snapshot.history.is_array()
        && snapshot.history.as_array().is_some_and(|a| a.is_empty())
    {
        r#"<p class="empty">No session history.</p>"#.to_string()
    } else {
        render_value_html(&snapshot.history)
    };
    render_html_page("History", &content)
}

fn render_governance_html(snapshot: &DashboardSnapshot) -> String {
    let content = if snapshot.governance.trim().is_empty() {
        r#"<p class="empty">No governance data found.</p>"#.to_string()
    } else {
        format!("<pre>{}</pre>", html_escape(&snapshot.governance))
    };
    render_html_page("Governance", &content)
}

fn render_config_html(snapshot: &DashboardSnapshot) -> String {
    let content = if snapshot.config_masked.trim().is_empty() {
        r#"<p class="empty">No configuration data.</p>"#.to_string()
    } else {
        format!("<pre>{}</pre>", html_escape(&snapshot.config_masked))
    };
    render_html_page("Config", &content)
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
            extensions: serde_json::json!({
                "mcp_servers": [{"name": "test-server", "connected": true, "tool_count": 2}],
                "collisions": []
            }),
        }
    }

    fn build_test_app() -> (Router, String) {
        let server = DashboardServer::with_loopback_only(test_snapshot(), false);
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

    async fn request_with_accept(
        app: &Router,
        method: Method,
        path: &str,
        token: Option<&str>,
        accept: Option<&str>,
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
        if let Some(a) = accept {
            req.headers_mut()
                .insert(header::ACCEPT, a.parse().expect("valid accept header"));
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
            extensions: serde_json::json!({
                "mcp_servers": [{"name": "server", "error": "api_key=sk-live"}],
            }),
        };
        let server = DashboardServer::with_loopback_only(snapshot, false);
        let token = server.token().to_string();
        let app = server.build_router();

        for path in [
            "/status",
            "/history",
            "/governance",
            "/config",
            "/extensions",
        ] {
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
        let s1 = DashboardServer::with_loopback_only(test_snapshot(), false);
        let s2 = DashboardServer::with_loopback_only(test_snapshot(), false);
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

    #[tokio::test]
    async fn default_server_is_loopback_only() {
        let server = DashboardServer::new(test_snapshot());
        let app = server.build_router();
        let (status, body) = request(&app, Method::GET, "/status", None).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("test-model"));
    }

    #[tokio::test]
    async fn extensions_route_returns_json() {
        let (app, token) = build_test_app();
        let (status, body) = request(&app, Method::GET, "/extensions", Some(&token)).await;
        assert_eq!(status, StatusCode::OK);
        let value: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");
        assert!(value["mcp_servers"].is_array());
        assert!(
            value["mcp_servers"]
                .as_array()
                .unwrap()
                .iter()
                .any(|s| { s["name"] == "test-server" && s["connected"] == true }),
            "extensions should include test-server: {body}"
        );
    }

    #[tokio::test]
    async fn extensions_route_redacts_sensitive_data() {
        let (app, token) = build_test_app();
        let (status, body) = request(&app, Method::GET, "/extensions", Some(&token)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(
            !body.contains("sk-live"),
            "extensions leaked api key: {body}"
        );
        assert!(!body.contains("secret"), "extensions leaked secret: {body}");
    }

    #[tokio::test]
    async fn extensions_route_is_get_only() {
        let (app, token) = build_test_app();
        for method in [Method::POST, Method::PUT, Method::DELETE, Method::PATCH] {
            let (status, _) = request(&app, method, "/extensions", Some(&token)).await;
            assert_eq!(
                status,
                StatusCode::METHOD_NOT_ALLOWED,
                "extensions route must be GET-only"
            );
        }
    }

    // ── I129 content negotiation tests ─────────────────────────────────────

    #[tokio::test]
    async fn html_accept_returns_html_status() {
        let (app, token) = build_test_app();
        let (status, body) = request_with_accept(
            &app,
            Method::GET,
            "/status",
            Some(&token),
            Some("text/html"),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("<!doctype html>"), "expected HTML: {body}");
        assert!(body.contains("<title>Status"), "expected title");
        assert!(body.contains("test-model"), "expected data in HTML");
        assert!(body.contains("<nav>"), "expected navigation");
    }

    #[tokio::test]
    async fn html_accept_returns_html_history() {
        let (app, token) = build_test_app();
        let (status, body) = request_with_accept(
            &app,
            Method::GET,
            "/history",
            Some(&token),
            Some("text/html"),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("<!doctype html>"));
        assert!(body.contains("abc"));
    }

    #[tokio::test]
    async fn html_accept_returns_html_governance() {
        let (app, token) = build_test_app();
        let (status, body) = request_with_accept(
            &app,
            Method::GET,
            "/governance",
            Some(&token),
            Some("text/html"),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("<!doctype html>"));
        assert!(body.contains("Now: test item"));
    }

    #[tokio::test]
    async fn html_accept_returns_html_config() {
        let (app, token) = build_test_app();
        let (status, body) = request_with_accept(
            &app,
            Method::GET,
            "/config",
            Some(&token),
            Some("text/html"),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("<!doctype html>"));
        assert!(body.contains("***"));
    }

    #[tokio::test]
    async fn no_accept_returns_json_status() {
        let (app, token) = build_test_app();
        let (status, body) =
            request_with_accept(&app, Method::GET, "/status", Some(&token), None).await;
        assert_eq!(status, StatusCode::OK);
        let _: serde_json::Value =
            serde_json::from_str(&body).expect("should be valid JSON, not HTML");
    }

    #[tokio::test]
    async fn wildcard_accept_returns_json_status() {
        let (app, token) = build_test_app();
        let (status, body) =
            request_with_accept(&app, Method::GET, "/status", Some(&token), Some("*/*")).await;
        assert_eq!(status, StatusCode::OK);
        let _: serde_json::Value =
            serde_json::from_str(&body).expect("*/* should return JSON, not HTML");
    }

    #[tokio::test]
    async fn json_accept_returns_json_status() {
        let (app, token) = build_test_app();
        let (status, body) = request_with_accept(
            &app,
            Method::GET,
            "/status",
            Some(&token),
            Some("application/json"),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let _: serde_json::Value =
            serde_json::from_str(&body).expect("application/json should return JSON");
    }

    #[tokio::test]
    async fn complex_accept_with_html_returns_html() {
        let (app, token) = build_test_app();
        let (status, body) = request_with_accept(
            &app,
            Method::GET,
            "/status",
            Some(&token),
            Some("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert!(
            body.contains("<!doctype html>"),
            "complex Accept with text/html should return HTML"
        );
    }

    #[tokio::test]
    async fn html_mode_redacts_secrets() {
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
            extensions: serde_json::json!({}),
        };
        let server = DashboardServer::with_loopback_only(snapshot, false);
        let token = server.token().to_string();
        let app = server.build_router();

        for path in ["/status", "/history", "/governance", "/config"] {
            let (status_code, body) =
                request_with_accept(&app, Method::GET, path, Some(&token), Some("text/html")).await;
            assert_eq!(status_code, StatusCode::OK, "{path} should be OK");
            assert!(
                !body.contains("sk-live"),
                "{path} leaked api key in HTML: {body}"
            );
            assert!(
                !body.contains("Bearer secret"),
                "{path} leaked bearer in HTML: {body}"
            );
            assert!(
                !body.contains("sid=secret"),
                "{path} leaked cookie in HTML: {body}"
            );
            assert!(
                !body.contains("token=abc"),
                "{path} leaked token query in HTML: {body}"
            );
        }
    }

    #[tokio::test]
    async fn html_mode_escapes_xss_payloads() {
        let snapshot = DashboardSnapshot {
            config_masked: "<script>alert(1)</script>".to_string(),
            status: serde_json::json!({"model": "<img onerror=alert(1)>"}),
            history: serde_json::json!([{"id": "\"><script>alert('xss')</script>"}]),
            governance: "<b>bold</b>&amp;".to_string(),
            extensions: serde_json::json!({}),
        };
        let server = DashboardServer::with_loopback_only(snapshot, false);
        let token = server.token().to_string();
        let app = server.build_router();

        for path in ["/status", "/history", "/governance", "/config"] {
            let (_, body) =
                request_with_accept(&app, Method::GET, path, Some(&token), Some("text/html")).await;
            assert!(
                !body.contains("<script>"),
                "{path} HTML leaked unescaped <script>: {body}"
            );
            assert!(
                !body.contains("<img "),
                "{path} HTML leaked unescaped <img> tag: {body}"
            );
        }
    }

    #[tokio::test]
    async fn empty_snapshot_renders_empty_states() {
        let snapshot = DashboardSnapshot {
            config_masked: "".to_string(),
            status: serde_json::json!({}),
            history: serde_json::json!([]),
            governance: "".to_string(),
            extensions: serde_json::json!({}),
        };
        let server = DashboardServer::with_loopback_only(snapshot, true);
        let app = server.build_router();

        let (_, status_body) =
            request_with_accept(&app, Method::GET, "/status", None, Some("text/html")).await;
        assert!(
            status_body.contains("No status data available."),
            "expected empty state: {status_body}"
        );

        let (_, history_body) =
            request_with_accept(&app, Method::GET, "/history", None, Some("text/html")).await;
        assert!(
            history_body.contains("No session history."),
            "expected empty state: {history_body}"
        );

        let (_, gov_body) =
            request_with_accept(&app, Method::GET, "/governance", None, Some("text/html")).await;
        assert!(
            gov_body.contains("No governance data found."),
            "expected empty state: {gov_body}"
        );

        let (_, config_body) =
            request_with_accept(&app, Method::GET, "/config", None, Some("text/html")).await;
        assert!(
            config_body.contains("No configuration data."),
            "expected empty state: {config_body}"
        );
    }

    #[tokio::test]
    async fn extensions_ignores_accept_html() {
        let (app, token) = build_test_app();
        let (status_code, body) = request_with_accept(
            &app,
            Method::GET,
            "/extensions",
            Some(&token),
            Some("text/html"),
        )
        .await;
        assert_eq!(status_code, StatusCode::OK);
        let _: serde_json::Value =
            serde_json::from_str(&body).expect("extensions must return JSON even with text/html");
    }

    #[tokio::test]
    async fn html_mode_get_only() {
        let (app, token) = build_test_app();
        for method in [Method::POST, Method::PUT, Method::DELETE, Method::PATCH] {
            let (status_code, _) = request_with_accept(
                &app,
                method.clone(),
                "/status",
                Some(&token),
                Some("text/html"),
            )
            .await;
            assert_eq!(
                status_code,
                StatusCode::METHOD_NOT_ALLOWED,
                "{method} /status with text/html should be 405"
            );
        }
    }

    #[tokio::test]
    async fn html_mode_loopback_only_no_token() {
        let app = build_loopback_only_app();
        let (status_code, body) =
            request_with_accept(&app, Method::GET, "/status", None, Some("text/html")).await;
        assert_eq!(status_code, StatusCode::OK);
        assert!(body.contains("<!doctype html>"));
    }

    #[tokio::test]
    async fn html_mode_token_required_in_token_mode() {
        let (app, _token) = build_test_app();
        let (status_code, _) =
            request_with_accept(&app, Method::GET, "/status", None, Some("text/html")).await;
        assert_eq!(status_code, StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn html_escape_covers_all_special_chars() {
        assert_eq!(html_escape("<>&\"'"), "&lt;&gt;&amp;&quot;&#x27;");
    }

    #[test]
    fn accepts_html_matching() {
        let mut headers = HeaderMap::new();
        assert!(!accepts_html(&headers)); // no Accept header

        headers.insert(header::ACCEPT, "*/*".parse().unwrap());
        assert!(!accepts_html(&headers));

        headers.insert(header::ACCEPT, "application/json".parse().unwrap());
        assert!(!accepts_html(&headers));

        headers.insert(header::ACCEPT, "text/html".parse().unwrap());
        assert!(accepts_html(&headers));

        headers.insert(
            header::ACCEPT,
            "text/html,application/xhtml+xml,*/*;q=0.8".parse().unwrap(),
        );
        assert!(accepts_html(&headers));

        headers.insert(header::ACCEPT, "text/html;charset=utf-8".parse().unwrap());
        assert!(accepts_html(&headers));
    }
}
