# MCP Remote HTTP/SSE Transport Fast-Track

Status: Complete
Owner: Codex
Date: 2026-07-07
Priority: Immediate escalation

## Scope

Implement remote MCP client transport support so Talos can connect to:

- Legacy HTTP/SSE MCP servers (`transport = "sse"`)
- Streamable HTTP MCP servers (`transport = "streamable_http"`)
- `transport = "http"` as a compatibility alias for Streamable HTTP

`stdio` transport remains supported and unchanged.

## Acceptance Criteria

- MCP config accepts remote endpoint fields without breaking existing stdio config.
- Remote auth can be injected from environment variables rather than requiring secrets in config.
- Streamable HTTP supports JSON responses and `text/event-stream` responses for JSON-RPC requests.
- Legacy SSE connects to the SSE endpoint, discovers the POST endpoint from the `endpoint` event when needed, then routes response events back to pending requests.
- MCP startup performs `initialize` and `notifications/initialized` before tool discovery, while preserving compatibility with older fixtures that do not implement `initialize`.
- Per-server remote startup failures are reported through existing MCP diagnostics and do not abort Talos startup.
- Local tests cover stdio preservation, Streamable HTTP JSON response, Streamable HTTP SSE response, legacy SSE endpoint discovery, and missing remote URL diagnostics.

## Implementation Notes

- Added `url`, `sse_post_url`, `headers`, `auth_token_env`, and `authorization_env` to MCP server config.
- Added environment-backed HTTP authorization:
  - `auth_token_env` sends `Authorization: Bearer <token>`.
  - `authorization_env` sends the full `Authorization` header value.
- `headers` is intended for non-secret HTTP headers.
- Legacy SSE respects an explicitly configured `sse_post_url`; endpoint auto-discovery only runs when it is unset.
- The first implementation covers request/response tool discovery and tool calls over remote transports. Streamable HTTP resumable sessions and long-lived server-to-client notification channels remain outside this fast-track scope.

## Validation

- `cargo fmt --all`
- `cargo test -p talos-mcp`
- `cargo check -p talos-cli`
- `cargo fmt --all -- --check`
- `cargo check --workspace`
