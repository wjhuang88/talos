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
    let mut resp = render_root_html().into_response();
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

async fn extensions_handler(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if accepts_html(&headers) {
        let mut resp = render_extensions_html(&state.snapshot).into_response();
        apply_security_headers(&mut resp, "text/html; charset=utf-8");
        resp
    } else {
        let mut resp = state.snapshot.extensions.to_string().into_response();
        apply_security_headers(&mut resp, "application/json");
        resp
    }
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

// ── HTML rendering helpers (I129 + I195) ────────────────────────────────────

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
/// attribute values. Every dynamic value rendered into a page passes through
/// this function.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Render a `serde_json::Value` recursively as safe, read-only HTML.
fn render_value_html(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            if map.is_empty() {
                return r#"<p class="empty">No values available.</p>"#.to_string();
            }
            let mut facts = String::new();
            for (key, value) in map {
                facts.push_str(&format!(
                    "<div class=\"fact-row\"><dt>{}</dt><dd>{}</dd></div>",
                    html_escape(key),
                    render_value_html(value)
                ));
            }
            format!("<dl class=\"facts\">{facts}</dl>")
        }
        Value::Array(items) => {
            if items.is_empty() {
                return r#"<p class="empty">No items available.</p>"#.to_string();
            }
            let mut items_html = String::new();
            for item in items {
                items_html.push_str(&format!(
                    "<article class=\"collection-item\">{}</article>",
                    render_value_html(item)
                ));
            }
            format!("<div class=\"collection\">{items_html}</div>")
        }
        Value::String(value) => format!(r#"<span class="value">{}</span>"#, html_escape(value)),
        Value::Null => r#"<span class="muted">null</span>"#.to_string(),
        other => format!(
            r#"<span class="machine-value">{}</span>"#,
            html_escape(&other.to_string())
        ),
    }
}

const DASHBOARD_NAV: &[(&str, &str)] = &[
    ("/", "Overview"),
    ("/status", "Status"),
    ("/history", "History"),
    ("/governance", "Governance"),
    ("/config", "Config"),
    ("/extensions", "Extensions"),
];

fn nav_icon(path: &str) -> &'static str {
    match path {
        "/" => {
            r#"<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 11.5 12 5l8 6.5v7a1.5 1.5 0 0 1-1.5 1.5h-13A1.5 1.5 0 0 1 4 18.5Z"/><path d="M9.5 20v-5h5v5"/></svg>"#
        }
        "/status" => {
            r#"<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M3 12h4l2-5 4 10 2-5h6"/></svg>"#
        }
        "/history" => {
            r#"<svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="8"/><path d="M12 8v4l3 2"/></svg>"#
        }
        "/governance" => {
            r#"<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 3 19 6v5c0 4.7-2.7 7.7-7 10-4.3-2.3-7-5.3-7-10V6Z"/><path d="m9 12 2 2 4-4"/></svg>"#
        }
        "/config" => {
            r#"<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 7h10M18 7h2M4 12h4M12 12h8M4 17h12M20 17h0"/><circle cx="16" cy="7" r="2"/><circle cx="10" cy="12" r="2"/><circle cx="18" cy="17" r="2"/></svg>"#
        }
        "/extensions" => {
            r#"<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M9 4h3a2 2 0 1 1 4 0h3v5a2 2 0 1 0 0 4v5h-5a2 2 0 1 1-4 0H5v-5a2 2 0 1 1 0-4V4Z"/></svg>"#
        }
        _ => "",
    }
}

fn render_navigation(active_path: &str) -> String {
    let mut links = String::new();
    for (path, label) in DASHBOARD_NAV {
        let icon = nav_icon(path);
        if *path == active_path {
            links.push_str(&format!(
                r#"<a href="{path}" aria-current="page"><span class="nav-icon">{icon}</span><span>{label}</span></a>"#
            ));
        } else {
            links.push_str(&format!(
                r#"<a href="{path}"><span class="nav-icon">{icon}</span><span>{label}</span></a>"#
            ));
        }
    }
    links
}

/// Shared Dashboard document shell. It deliberately has no script, external
/// font, image, or remote asset dependency so it continues to satisfy the
/// existing CSP (`default-src 'none'; style-src 'unsafe-inline'`).
fn render_html_page(title: &str, active_path: &str, description: &str, content: &str) -> String {
    let title = html_escape(title);
    let description = html_escape(description);
    let navigation = render_navigation(active_path);
    format!(
        r##"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title} — Talos Dashboard</title>
<style>
:root {{
  color-scheme: light;
  font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  --nord0: #2e3440;
  --nord1: #3b4252;
  --nord3: #4c566a;
  --nord4: #d8dee9;
  --nord5: #e5e9f0;
  --nord6: #eceff4;
  --nord9: #81a1c1;
  --nord10: #5e81ac;
  --nord11: #bf616a;
  --nord13: #ebcb8b;
  --nord14: #a3be8c;
  --canvas: #fbfcfe;
  --surface: rgba(255,255,255,.82);
  --surface-strong: #ffffff;
  --muted: #687386;
  --focus: #5e81ac;
}}
* {{ box-sizing: border-box; }}
html {{ min-width: 0; background: var(--nord6); color: var(--nord0); }}
body {{ margin: 0; min-width: 0; min-height: 100vh; line-height: 1.5; background: var(--canvas); }}
a {{ color: var(--nord10); text-underline-offset: .18em; }}
a:focus-visible, [tabindex="0"]:focus-visible {{ outline: 3px solid var(--focus); outline-offset: 3px; border-radius: .45rem; }}
.skip-link {{ position: fixed; left: 1rem; top: -5rem; z-index: 20; padding: .65rem .85rem; color: var(--nord0); background: #fff; border: 1px solid var(--nord4); border-radius: .7rem; box-shadow: 0 8px 22px rgba(46,52,64,.12); }}
.skip-link:focus {{ top: 1rem; }}
.app-shell {{ min-height: 100vh; display: grid; grid-template-columns: 13.5rem minmax(0, 1fr); }}
.rail {{ position: sticky; top: 0; align-self: start; height: 100vh; display: flex; flex-direction: column; padding: 1.7rem 1rem 1.25rem; background: rgba(245,247,250,.9); border-right: 1px solid rgba(216,222,233,.86); }}
.brand {{ display: flex; align-items: center; gap: .72rem; min-height: 2.5rem; padding: 0 .55rem; color: var(--nord0); text-decoration: none; font-weight: 720; letter-spacing: -.018em; }}
.brand-mark {{ display: grid; place-items: center; width: 2rem; height: 2rem; border-radius: .62rem; color: #fff; background: var(--nord10); font-size: .92rem; font-weight: 780; box-shadow: inset 0 0 0 1px rgba(46,52,64,.08); }}
.brand-copy {{ display: grid; gap: .04rem; line-height: 1.15; }}
.brand-copy small {{ color: var(--muted); font-size: .67rem; font-weight: 560; letter-spacing: .02em; }}
nav {{ display: grid; gap: .28rem; margin-top: 2rem; }}
nav a {{ display: flex; align-items: center; gap: .72rem; min-height: 2.6rem; padding: .55rem .72rem; color: var(--nord3); border-radius: .72rem; text-decoration: none; font-size: .91rem; font-weight: 590; }}
nav a:hover {{ color: var(--nord0); background: rgba(229,233,240,.72); }}
nav a[aria-current="page"] {{ color: #365f90; background: rgba(129,161,193,.13); font-weight: 690; }}
.nav-icon {{ flex: 0 0 auto; width: 1.18rem; height: 1.18rem; display: grid; place-items: center; }}
.nav-icon svg {{ width: 100%; height: 100%; fill: none; stroke: currentColor; stroke-width: 1.75; stroke-linecap: round; stroke-linejoin: round; }}
.rail-meta {{ margin-top: auto; padding: 1rem .58rem 0; border-top: 1px solid rgba(216,222,233,.9); color: var(--muted); font-size: .74rem; line-height: 1.55; }}
.rail-meta strong {{ display: flex; align-items: center; gap: .42rem; color: var(--nord3); font-size: .76rem; font-weight: 650; }}
.rail-meta strong::before {{ content: ""; width: .45rem; height: .45rem; border-radius: 50%; background: #66835b; box-shadow: 0 0 0 3px rgba(163,190,140,.16); }}
.canvas {{ min-width: 0; background: var(--canvas); }}
main {{ width: min(74rem, calc(100% - clamp(2rem, 7vw, 7rem))); margin: 0 auto; padding: clamp(2.4rem, 6vw, 5.1rem) 0 4rem; }}
.page-header {{ max-width: 50rem; margin-bottom: clamp(2.1rem, 5vw, 3.7rem); }}
.eyebrow {{ margin: 0 0 .72rem; color: var(--nord10); font-size: .72rem; font-weight: 720; letter-spacing: .095em; text-transform: uppercase; }}
h1 {{ margin: 0; color: var(--nord0); font-size: clamp(2rem, 4.8vw, 3rem); line-height: 1.06; letter-spacing: -.042em; font-weight: 735; }}
.lede {{ max-width: 44rem; margin: .72rem 0 0; color: var(--nord3); font-size: clamp(.96rem, 1.7vw, 1.06rem); line-height: 1.58; }}
.context-line {{ display: inline-flex; align-items: center; gap: .42rem; margin-top: 1rem; color: var(--muted); font-size: .76rem; }}
.context-line::before {{ content: ""; width: .44rem; height: .44rem; border-radius: 50%; background: #66835b; }}
.content-flow {{ min-width: 0; display: grid; gap: 2rem; }}
.section-heading {{ margin: 0 0 1rem; color: var(--nord10); font-size: .72rem; font-weight: 720; letter-spacing: .09em; text-transform: uppercase; }}
.focus-surface {{ max-width: 56rem; min-width: 0; padding: 1.35rem 1.45rem; border: 1px solid rgba(216,222,233,.86); border-radius: 1.2rem; background: var(--surface); box-shadow: 0 8px 30px rgba(46,52,64,.035); }}
.material-surface {{ min-width: 0; padding: 1rem 1.15rem; border: 1px solid rgba(216,222,233,.9); border-radius: 1rem; background: var(--surface); }}
.facts {{ margin: 0; }}
.fact-row {{ display: grid; grid-template-columns: minmax(8rem, .34fr) minmax(0, 1fr); gap: 1.2rem; padding: .72rem 0; border-bottom: 1px solid rgba(229,233,240,.92); }}
.fact-row:first-child {{ padding-top: 0; }}
.fact-row:last-child {{ padding-bottom: 0; border-bottom: 0; }}
dt {{ color: var(--nord3); font-size: .82rem; font-weight: 620; overflow-wrap: anywhere; }}
dd {{ margin: 0; min-width: 0; color: var(--nord0); overflow-wrap: anywhere; }}
.value {{ color: var(--nord0); }}
.machine-value {{ color: var(--nord1); font: 560 .88rem/1.5 ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; }}
.collection {{ display: grid; gap: 0; }}
.collection-item {{ min-width: 0; padding: 1rem 0; border-bottom: 1px solid rgba(229,233,240,.94); }}
.collection-item:first-child {{ padding-top: 0; }}
.collection-item:last-child {{ padding-bottom: 0; border-bottom: 0; }}
.timeline {{ max-width: 58rem; position: relative; padding-left: 1.3rem; }}
.timeline::before {{ content: ""; position: absolute; left: .24rem; top: .55rem; bottom: .55rem; width: 1px; background: var(--nord4); }}
.timeline-item {{ position: relative; padding: 0 0 1.45rem 1.15rem; }}
.timeline-item:last-child {{ padding-bottom: 0; }}
.timeline-marker {{ position: absolute; left: -1.34rem; top: .5rem; width: .56rem; height: .56rem; border-radius: 50%; background: var(--nord5); border: 1px solid #c6cfdb; }}
.timeline-item:first-child .timeline-marker {{ background: var(--nord10); border-color: var(--nord10); box-shadow: 0 0 0 4px rgba(94,129,172,.1); }}
.timeline .facts {{ max-width: 48rem; }}
.timeline .fact-row {{ grid-template-columns: minmax(6.5rem, .28fr) minmax(0,1fr); padding: .38rem 0; border-bottom: 0; }}
.section-links {{ max-width: 56rem; border-top: 1px solid var(--nord5); }}
.section-link {{ display: grid; grid-template-columns: minmax(0,1fr) auto; gap: 1rem; align-items: center; padding: 1.2rem .25rem; border-bottom: 1px solid var(--nord5); color: inherit; text-decoration: none; }}
.section-link:hover .link-title {{ color: var(--nord10); }}
.link-title {{ display: block; color: var(--nord0); font-size: 1rem; font-weight: 690; transition: color .16s ease; }}
.section-link p {{ margin: .2rem 0 0; color: var(--muted); font-size: .86rem; }}
.section-link .arrow {{ color: var(--nord10); font-size: 1.08rem; }}
pre {{ margin: 0; max-width: 100%; overflow: auto; padding: .9rem 1rem; border: 0; border-radius: .8rem; background: rgba(236,239,244,.58); color: var(--nord1); font: 520 .86rem/1.62 ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; white-space: pre; }}
.empty {{ margin: 0; color: var(--muted); font-style: italic; }}
.muted {{ color: var(--muted); }}
.footer-note {{ max-width: 56rem; margin: 3.2rem 0 0; padding-top: 1rem; border-top: 1px solid var(--nord5); color: #778294; font-size: .75rem; }}
@media (max-width: 54rem) {{
  .app-shell {{ grid-template-columns: 11.5rem minmax(0,1fr); }}
  main {{ width: min(100% - 2rem, 74rem); }}
  .fact-row {{ grid-template-columns: minmax(7rem, .32fr) minmax(0,1fr); }}
}}
@media (max-width: 42rem) {{
  .app-shell {{ display: block; }}
  .rail {{ position: static; height: auto; padding: .8rem .7rem .65rem; border-right: 0; border-bottom: 1px solid var(--nord4); }}
  .brand {{ min-height: 2.2rem; padding: 0 .35rem; }}
  .brand-mark {{ width: 1.75rem; height: 1.75rem; border-radius: .52rem; }}
  .brand-copy small {{ display: none; }}
  nav {{ grid-template-columns: repeat(3, minmax(0,1fr)); gap: .25rem; margin-top: .75rem; }}
  nav a {{ justify-content: center; min-height: 2.35rem; padding: .45rem .35rem; gap: .42rem; font-size: .78rem; }}
  .nav-icon {{ width: 1rem; height: 1rem; }}
  .rail-meta {{ display: none; }}
  main {{ width: min(100% - 1.35rem, 74rem); padding: 2rem 0 3rem; }}
  .page-header {{ margin-bottom: 2rem; }}
  h1 {{ font-size: clamp(1.85rem, 10vw, 2.35rem); }}
  .focus-surface, .material-surface {{ padding: 1rem; border-radius: .9rem; }}
  .fact-row, .timeline .fact-row {{ grid-template-columns: 1fr; gap: .22rem; }}
  .timeline {{ padding-left: 1.15rem; }}
  .section-link {{ padding: 1rem .1rem; }}
}}
@media (prefers-reduced-motion: reduce) {{
  *, *::before, *::after {{ scroll-behavior: auto !important; transition-duration: 0.01ms !important; }}
}}
</style>
</head>
<body>
<a class="skip-link" href="#main-content">Skip to content</a>
<div class="app-shell">
  <aside class="rail" aria-label="Dashboard navigation">
    <a class="brand" href="/"><span class="brand-mark" aria-hidden="true">T</span><span class="brand-copy">Talos Dashboard<small>Local state surface</small></span></a>
    <nav aria-label="Dashboard sections">{navigation}</nav>
    <div class="rail-meta"><strong>Read-only</strong>127.0.0.1 · local loopback</div>
  </aside>
  <div class="canvas">
    <main id="main-content">
      <header class="page-header">
        <p class="eyebrow">Talos Dashboard</p>
        <h1>{title}</h1>
        <p class="lede">{description}</p>
        <span class="context-line">Local · read-only · 127.0.0.1</span>
      </header>
      {content}
      <p class="footer-note">This Dashboard presents existing Talos state. It does not provide write, approval, tool-execution, or session-mutation controls.</p>
    </main>
  </div>
</div>
</body>
</html>"##
    )
}

fn render_root_html() -> String {
    let links = r#"<section class="content-flow" aria-label="Dashboard sections">
<div>
  <p class="section-heading">Read-only surfaces</p>
  <div class="section-links">
    <a class="section-link" href="/status"><span><span class="link-title">Status</span><p>Current runtime and session state.</p></span><span class="arrow" aria-hidden="true">→</span></a>
    <a class="section-link" href="/history"><span><span class="link-title">History</span><p>Recent session history from existing Talos state.</p></span><span class="arrow" aria-hidden="true">→</span></a>
    <a class="section-link" href="/governance"><span><span class="link-title">Governance</span><p>Current project governance summary.</p></span><span class="arrow" aria-hidden="true">→</span></a>
    <a class="section-link" href="/config"><span><span class="link-title">Config</span><p>Masked configuration with secrets withheld.</p></span><span class="arrow" aria-hidden="true">→</span></a>
    <a class="section-link" href="/extensions"><span><span class="link-title">Extensions</span><p>Observed extension and MCP state.</p></span><span class="arrow" aria-hidden="true">→</span></a>
  </div>
</div>
</section>"#;
    render_html_page(
        "Local state, without controls.",
        "/",
        "Inspect Talos through five existing loopback surfaces. The Dashboard stays presentation-only and keeps the underlying runtime, configuration, and governance sources authoritative.",
        links,
    )
}

fn render_focus_section(label: &str, content: &str) -> String {
    format!(
        r#"<section class="content-flow"><div><p class="section-heading">{}</p><div class="focus-surface">{content}</div></div></section>"#,
        html_escape(label)
    )
}

fn render_material_section(label: &str, content: &str) -> String {
    format!(
        r#"<section class="content-flow"><div><p class="section-heading">{}</p><div class="material-surface">{content}</div></div></section>"#,
        html_escape(label)
    )
}

fn render_history_items(value: &Value) -> String {
    match value.as_array() {
        Some(items) if items.is_empty() => {
            r#"<p class="empty">No session history.</p>"#.to_string()
        }
        Some(items) => {
            let mut timeline = String::new();
            for item in items {
                timeline.push_str(&format!(
                    r#"<article class="timeline-item"><span class="timeline-marker" aria-hidden="true"></span>{}</article>"#,
                    render_value_html(item)
                ));
            }
            format!(r#"<div class="timeline">{timeline}</div>"#)
        }
        None => render_value_html(value),
    }
}

fn render_status_html(snapshot: &DashboardSnapshot) -> String {
    let content = if snapshot.status.is_null()
        || (snapshot.status.is_object()
            && snapshot
                .status
                .as_object()
                .is_some_and(|map| map.is_empty()))
    {
        r#"<p class="empty">No status data available.</p>"#.to_string()
    } else {
        render_value_html(&snapshot.status)
    };
    render_html_page(
        "Status",
        "/status",
        "What is Talos doing now? This is the current runtime and session snapshot, presented without inventing progress or control state.",
        &render_focus_section("Current snapshot", &content),
    )
}

fn render_history_html(snapshot: &DashboardSnapshot) -> String {
    let content = render_history_items(&snapshot.history);
    render_html_page(
        "History",
        "/history",
        "Recent session history, presented as a quiet activity stream rather than a control log.",
        &format!(
            r#"<section class="content-flow"><div><p class="section-heading">Recent activity</p>{content}</div></section>"#
        ),
    )
}

fn render_governance_html(snapshot: &DashboardSnapshot) -> String {
    let content = if snapshot.governance.trim().is_empty() {
        r#"<p class="empty">No governance data found.</p>"#.to_string()
    } else {
        format!("<pre>{}</pre>", html_escape(&snapshot.governance))
    };
    render_html_page(
        "Governance",
        "/governance",
        "Ownership, state, and policy facts from the existing governance summary. No claim or decision is changed here.",
        &render_material_section("Current governance", &content),
    )
}

fn render_config_html(snapshot: &DashboardSnapshot) -> String {
    let content = if snapshot.config_masked.trim().is_empty() {
        r#"<p class="empty">No configuration data.</p>"#.to_string()
    } else {
        format!("<pre>{}</pre>", html_escape(&snapshot.config_masked))
    };
    render_html_page(
        "Config",
        "/config",
        "Read-only configuration with the existing masking and output-redaction boundary kept intact.",
        &render_material_section("Masked configuration", &content),
    )
}

fn render_extensions_html(snapshot: &DashboardSnapshot) -> String {
    let content = if snapshot.extensions.is_null()
        || (snapshot.extensions.is_object()
            && snapshot
                .extensions
                .as_object()
                .is_some_and(|map| map.is_empty()))
    {
        r#"<p class="empty">No extension data available.</p>"#.to_string()
    } else {
        render_value_html(&snapshot.extensions)
    };
    render_html_page(
        "Extensions",
        "/extensions",
        "Installed and observed extension/MCP state from the existing snapshot. This view does not install, enable, disable, or execute anything.",
        &render_material_section("Observed extensions", &content),
    )
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
            for path in [
                "/",
                "/status",
                "/history",
                "/governance",
                "/config",
                "/extensions",
            ] {
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
        let (addr, handle) = server.serve().await.expect("test operation should succeed");
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
        for path in [
            "/status",
            "/history",
            "/governance",
            "/config",
            "/extensions",
            "/",
        ] {
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
        let (addr, handle) = server.serve().await.expect("test operation should succeed");
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
                .expect("test operation should succeed")
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
        assert!(
            body.contains(r#"<nav aria-label="Dashboard sections">"#),
            "expected navigation"
        );
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
            extensions: serde_json::json!({
                "adapter": "<svg onload=alert(1)>",
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
            assert!(
                !body.contains("<svg onload="),
                "{path} HTML leaked injected SVG attributes: {body}"
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

        let (_, extensions_body) =
            request_with_accept(&app, Method::GET, "/extensions", None, Some("text/html")).await;
        assert!(
            extensions_body.contains("No extension data available."),
            "expected empty state: {extensions_body}"
        );
    }

    #[tokio::test]
    async fn history_html_preserves_snapshot_order() {
        let snapshot = DashboardSnapshot {
            config_masked: String::new(),
            status: serde_json::json!({}),
            history: serde_json::json!([
                {"id": "first-session"},
                {"id": "second-session"}
            ]),
            governance: String::new(),
            extensions: serde_json::json!({}),
        };
        let server = DashboardServer::with_loopback_only(snapshot, true);
        let app = server.build_router();
        let (_, body) =
            request_with_accept(&app, Method::GET, "/history", None, Some("text/html")).await;
        let first = body
            .find("first-session")
            .expect("first history item missing");
        let second = body
            .find("second-session")
            .expect("second history item missing");
        assert!(
            first < second,
            "HTML history reordered the snapshot: {body}"
        );
    }

    #[tokio::test]
    async fn extensions_non_html_accept_preserves_json() {
        let (app, token) = build_test_app();
        for accept in [None, Some("*/*"), Some("application/json")] {
            let (status_code, body) =
                request_with_accept(&app, Method::GET, "/extensions", Some(&token), accept).await;
            assert_eq!(status_code, StatusCode::OK);
            let value: Value =
                serde_json::from_str(&body).expect("non-HTML extensions response must stay JSON");
            assert_eq!(value["mcp_servers"][0]["name"], "test-server");
        }
    }

    #[tokio::test]
    async fn html_accept_returns_html_extensions() {
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
        assert!(body.contains("<!doctype html>"), "expected HTML: {body}");
        assert!(body.contains("<title>Extensions"), "expected title");
        assert!(body.contains("test-server"), "expected extension data");
        assert!(
            body.contains(r#"href="/extensions" aria-current="page""#),
            "expected active Extensions navigation"
        );
    }

    #[tokio::test]
    async fn html_mode_get_only() {
        let (app, token) = build_test_app();
        for method in [Method::POST, Method::PUT, Method::DELETE, Method::PATCH] {
            for path in [
                "/status",
                "/history",
                "/governance",
                "/config",
                "/extensions",
            ] {
                let (status_code, _) = request_with_accept(
                    &app,
                    method.clone(),
                    path,
                    Some(&token),
                    Some("text/html"),
                )
                .await;
                assert_eq!(
                    status_code,
                    StatusCode::METHOD_NOT_ALLOWED,
                    "{method} {path} with text/html should be 405"
                );
            }
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

    #[tokio::test]
    async fn root_and_all_pages_share_accessible_navigation_shell() {
        let (app, token) = build_test_app();
        let pages = [
            ("/", "Overview"),
            ("/status", "Status"),
            ("/history", "History"),
            ("/governance", "Governance"),
            ("/config", "Config"),
            ("/extensions", "Extensions"),
        ];
        for (path, label) in pages {
            let (_, body) =
                request_with_accept(&app, Method::GET, path, Some(&token), Some("text/html")).await;
            assert!(body.contains(r#"<html lang="en">"#), "{path} lang metadata");
            assert!(body.contains(r#"<meta name="viewport""#), "{path} viewport");
            assert!(
                body.contains(r#"<nav aria-label="Dashboard sections">"#),
                "{path} semantic navigation"
            );
            assert!(
                body.contains(r#"<main id="main-content">"#),
                "{path} main landmark"
            );
            assert!(body.contains(r#"class="skip-link""#), "{path} skip link");
            assert!(
                body.contains(r#"href="/extensions""#),
                "{path} extensions link"
            );
            assert!(
                body.contains(&format!(r#"href="{path}" aria-current="page""#))
                    && body.contains(&format!("<span>{label}</span></a>")),
                "{path} current page marker"
            );
            assert!(
                body.contains(":focus-visible"),
                "{path} visible focus styling"
            );
            assert!(
                body.contains("@media (max-width: 42rem)"),
                "{path} narrow layout"
            );
            assert!(!body.contains("<script"), "{path} must not require script");
        }
    }

    #[tokio::test]
    async fn html_pages_keep_existing_security_headers() {
        let (app, token) = build_test_app();
        for path in [
            "/",
            "/status",
            "/history",
            "/governance",
            "/config",
            "/extensions",
        ] {
            let mut req = Request::builder()
                .method(Method::GET)
                .uri(path)
                .body(axum::body::Body::empty())
                .expect("failed to build request");
            req.headers_mut().insert(
                header::AUTHORIZATION,
                format!("Bearer {token}")
                    .parse()
                    .expect("valid auth header"),
            );
            req.headers_mut().insert(
                header::ACCEPT,
                "text/html".parse().expect("valid accept header"),
            );
            let response = tower::ServiceExt::oneshot(app.clone(), req)
                .await
                .expect("request failed");
            assert_eq!(response.status(), StatusCode::OK, "{path}");
            assert_eq!(
                response.headers().get(header::CONTENT_SECURITY_POLICY),
                Some(&HeaderValue::from_static(
                    "default-src 'none'; style-src 'unsafe-inline'"
                )),
                "{path} CSP changed"
            );
            assert_eq!(
                response.headers().get(header::X_CONTENT_TYPE_OPTIONS),
                Some(&HeaderValue::from_static("nosniff")),
                "{path} nosniff missing"
            );
            assert_eq!(
                response.headers().get(header::CACHE_CONTROL),
                Some(&HeaderValue::from_static("no-store")),
                "{path} cache policy changed"
            );
        }
    }

    fn srgb_channel(value: u8) -> f64 {
        let value = f64::from(value) / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }

    fn relative_luminance(rgb: [u8; 3]) -> f64 {
        0.2126 * srgb_channel(rgb[0])
            + 0.7152 * srgb_channel(rgb[1])
            + 0.0722 * srgb_channel(rgb[2])
    }

    fn contrast_ratio(a: [u8; 3], b: [u8; 3]) -> f64 {
        let a = relative_luminance(a);
        let b = relative_luminance(b);
        let (lighter, darker) = if a >= b { (a, b) } else { (b, a) };
        (lighter + 0.05) / (darker + 0.05)
    }

    #[test]
    fn dashboard_palette_meets_recorded_wcag_contrast_thresholds() {
        let canvas = [0xfb, 0xfc, 0xfe];
        let rail = [0xf5, 0xf7, 0xfa];
        let primary_text = [0x2e, 0x34, 0x40];
        let secondary_text = [0x4c, 0x56, 0x6a];
        let muted_text = [0x68, 0x73, 0x86];
        let current_text = [0x36, 0x5f, 0x90];
        let focus = [0x5e, 0x81, 0xac];

        assert!(contrast_ratio(primary_text, canvas) >= 4.5);
        assert!(contrast_ratio(secondary_text, canvas) >= 4.5);
        assert!(contrast_ratio(muted_text, canvas) >= 4.5);
        assert!(contrast_ratio(current_text, canvas) >= 4.5);
        assert!(contrast_ratio(focus, canvas) >= 3.0);
        assert!(contrast_ratio(focus, rail) >= 3.0);
    }

    #[test]
    fn html_escape_covers_all_special_chars() {
        assert_eq!(html_escape("<>&\"'"), "&lt;&gt;&amp;&quot;&#x27;");
    }

    #[test]
    fn accepts_html_matching() {
        let mut headers = HeaderMap::new();
        assert!(!accepts_html(&headers)); // no Accept header

        headers.insert(
            header::ACCEPT,
            "*/*".parse().expect("test operation should succeed"),
        );
        assert!(!accepts_html(&headers));

        headers.insert(
            header::ACCEPT,
            "application/json"
                .parse()
                .expect("test operation should succeed"),
        );
        assert!(!accepts_html(&headers));

        headers.insert(
            header::ACCEPT,
            "text/html".parse().expect("test operation should succeed"),
        );
        assert!(accepts_html(&headers));

        headers.insert(
            header::ACCEPT,
            "text/html,application/xhtml+xml,*/*;q=0.8"
                .parse()
                .expect("test operation should succeed"),
        );
        assert!(accepts_html(&headers));

        headers.insert(
            header::ACCEPT,
            "text/html;charset=utf-8"
                .parse()
                .expect("test operation should succeed"),
        );
        assert!(accepts_html(&headers));
    }
}
