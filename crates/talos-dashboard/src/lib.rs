//! Read-only loopback dashboard server for Talos (ADR-031).
//!
//! Binds to `127.0.0.1:0` (OS-assigned port), authenticates every request with
//! a per-process bearer token, and serves four GET-only routes from a
//! pre-computed [`DashboardSnapshot`]. No write, action, or tool-execution
//! routes are registered.

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
}

pub struct DashboardServer {
    state: AppState,
}

impl DashboardServer {
    pub fn new(snapshot: DashboardSnapshot) -> Self {
        Self {
            state: AppState {
                token: Uuid::new_v4().simple().to_string(),
                snapshot: Arc::new(snapshot),
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
        Router::new()
            .route("/status", get(status_handler))
            .route("/history", get(history_handler))
            .route("/governance", get(governance_handler))
            .route("/config", get(config_handler))
            .route("/", get(root_handler))
            .route_layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ))
            .with_state(state)
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
}
