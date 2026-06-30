# WEB-001 Embedded Web Control Surface — MVP Design Proposal

Created: 2026-06-30 (T22+T23 of the four-month plan)
Status: Design (pre-ADR); no implementation without WEB-001 ADR gate

## Scope

This proposal defines the WEB-001 MVP: a loopback-only read-only dashboard for status, history,
governance, and config surfaces (T22), and the local web auth boundary (T23). It is a design
artifact — implementation requires the WEB-001 ADR to pass and is gated by the Month-2 closeout
(T39).

Source backlog: `docs/backlog/active/WEB-001-embedded-web-control-surface.md`.

## Architecture

### Loopback-only HTTP server

```
Talos runtime
  ├── Agent turn loop (existing)
  └── Loopback dashboard server (NEW, optional)
        ├── Bind: 127.0.0.1:{random_port}
        ├── Auth: startup token in Authorization header
        ├── Routes: /status, /history, /governance, /config (read-only)
        └── No outbound connections, no remote access
```

- The dashboard server binds to `127.0.0.1` only. It is not reachable from other machines.
- The port is random (OS-assigned) to avoid port-conflict attacks and predictable endpoints.
- The server starts only when explicitly enabled (config flag or CLI flag `--dashboard`).
- The server has no write endpoints. All routes are GET-only.

### Dashboard surfaces

| Route | Content | Data source | Secret-safe? |
|---|---|---|---|
| `/status` | Session status: model, turn count, token usage, duration, active tools | `RuntimeHandle` event stream | Yes (usage counts only, no API keys) |
| `/history` | Conversation turn list: role, timestamp, tool calls, token delta | Session JSONL log | Yes (tool arguments scrubbed of secrets) |
| `/governance` | Board/backlog/iteration summary: active items, blockers, planned | `docs/BOARD.md` + governance state | Yes (derived view only) |
| `/config` | Configuration view: provider, model, feature flags (masked) | `talos-config` | Yes (`api_key` masked as `***` per ADR-023) |

All surfaces are read-only. No POST/PUT/DELETE routes. No configuration editing through the web.

## Auth Boundary (T23)

### Startup token

- When the dashboard server starts, it generates a cryptographically random token (32 bytes,
  hex-encoded).
- The token is printed once to stdout/stderr: `Dashboard: http://127.0.0.1:{port}?token={token}`.
- Every request must include `Authorization: Bearer {token}`.
- Requests without or with an incorrect token receive `401 Unauthorized`.
- The token is NOT stored in any file, config, or environment variable. It exists only in the
  running process memory.

### No secret echo

- The `/config` surface masks all credentials per ADR-023: `api_key` → `***`, `api_key_env` → shown
  (env var name only, not the value).
- The `/history` surface scrubs tool arguments that may contain secrets (e.g., `http_request`
  headers, `bash` commands with env vars). The scrubbing uses the same logic as the CLI
  `--config-list` masking.
- Error messages never include secrets, tokens, or internal paths.

### No permission bypass

- The dashboard does NOT grant any tool execution capability. There are no write routes.
- The dashboard does NOT bypass the permission pipeline. It only reads state that the runtime
  already exposes.
- The dashboard does NOT expose the model's system prompt, skill bodies, or memory content in
  raw form. These are summarized (counts, metadata) not dumped.
- The dashboard process is the same Talos process — it does not create a new privilege context.

### Loopback constraints

| Constraint | Enforcement |
|---|---|
| Bind address | `127.0.0.1` hardcoded; no `0.0.0.0` option |
| CORS | Disabled (no `Access-Control-Allow-Origin` header) |
| CSP | `default-src 'self'; script-src 'none'` (static HTML only, no JS from server) |
| TLS | Not required (loopback only); no certificate management |
| Rate limit | 10 requests/second per route (prevents resource exhaustion) |
| Request size | 1 MB max body (all routes are GET, so this is defensive only) |

## Threat Model

### What this protects against

1. **Remote access**: loopback-only binding prevents access from other machines.
2. **Token guessing**: 32-byte crypto-random token is infeasible to brute-force.
3. **Secret leakage**: all config/history surfaces mask credentials.
4. **Privilege escalation**: no write routes means no way to execute tools or change config.
5. **CSRF**: no cookies, no session storage, token-based auth only.

### What this does NOT protect against

1. **Local process compromise**: if malware runs on the same machine, it can read the token from
   process memory or sniff loopback traffic. This is outside Talos's trust boundary.
2. **Browser tab hijacking**: if the user opens the dashboard in a browser tab, a malicious page
  on another tab could theoretically send requests (if CORS were enabled — it is NOT). The CSP
  and CORS-disabled headers mitigate this.
3. **Dashboard content injection**: if the dashboard displays user-controlled content (e.g., tool
   output in history), XSS is a risk. Mitigation: all content is HTML-escaped; no inline scripts.

## Implementation Phases (reference only)

Phase 0 (this design): architecture and auth boundary defined.
Phase 1 (T28, gated): prototype read-only status/history subset if Month-2 gate passes.
Phase 2 (future): governance/config surfaces, full dashboard.
Phase 3 (future): interactive features (if ever approved — requires full security review).

## Non-goals

- No remote access, no VPN, no tunneling.
- No write/execute capability through the web.
- No WebSocket streaming (polling only for MVP).
- No JavaScript framework (static HTML + CSS, minimal JS for polling).
- No authentication beyond the startup token (no OAuth, no passwords).
- No integration with WEB-005 browser-page reads.
