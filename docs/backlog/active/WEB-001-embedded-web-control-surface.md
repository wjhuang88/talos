# WEB-001: Embedded Web Control Surface

| Field | Value |
|-------|-------|
| Story ID | WEB-001 |
| Priority | P2 (elevated 2026-06-27 — product differentiation track; informed by EXT-002/omp.sh reference) |
| Status | Partial — I129 rendered pages and WEB-001-A/I195 cohesive read-only visual shell Complete (2026-08-18). Residuals: SSE log view, config editor, web approvals, session actions, remote/LAN — all remain separately governed. |
| Depends On | talos-rpc infrastructure; OBS-001 (logs); CONF-001 (config primitives) |
| Relates To | REMOTE-001 (remote/P2P surface — may share a handler backbone); OBS-001; CONF-001 |
| Blocks | live log viewer; web config editor; later write/control surfaces |
| Origin | User request 2026-06-17; reprioritized 2026-06-27 as a Talos特色优势 candidate, with EXT-002/omp.sh as reference implementation research input |

## Outcome

Embed a **local web server + embedded static UI** inside the Talos runtime, started alongside the
TUI. The delivered boundary is currently a loopback-only, read-only browser surface; richer control
capabilities remain future separately governed work rather than being implied by the completed shell.

WEB-001 remains Partial because the completed read-only shell does not authorize or complete live
logs, configuration writes, approvals, session actions, remote/LAN access or other control-plane
behavior.

## Gate Status

ADR-031 accepted the WEB-001 MVP boundary on 2026-07-01 and was amended on 2026-07-02 for a
default-on TUI lifecycle with config opt-out. The per-process bearer token is opt-in via
`[dashboard] loopback_only = false`; the default is loopback-bind-only for the common single-user
case. Remote access, web approvals, config writes, browser automation, WebSocket control, and any
write/session-mutating route remain out of scope until later separately governed security/design work.

T112/T113 security review update (2026-07-02): `docs/reference/WEB-DASHBOARD-BROWSER-SECURITY-REVIEW-2026-07-02.md`
recorded the dashboard/browser boundary review. T113 added dashboard output-boundary redaction for
snapshot data and regression coverage proving that, when `loopback_only = false`, unknown paths
without a token are rejected before returning route information.

## Current Implementation Boundary

`talos-dashboard` serves the loopback-only, read-only snapshot surfaces at `/status`, `/history`,
`/governance`, `/config`, and `/extensions`. Root plus all five data pages now share the completed
WEB-001-A/I195 light, compact Nord-derived visual shell when HTML is explicitly requested.

Existing JSON/plain-text representations remain authoritative for non-HTML requests, including
default JSON for `/extensions`; config masking, output-boundary redaction, HTML escaping and the
existing CSP are preserved. No write/action route, remote/LAN path, new dependency, durable state or
alternate runtime/domain/session authority was introduced.

WEB-001-A/I195 completed through PR #233 merge
`490503db905bcd2eb2ab5e3b5487b1f542873d63` from reviewed exact head
`1ee4aa3786785473069c735e1985c9d720b82e2f`, with exact-head CI `32087223234`, independent AI
technical review `5323625004`, human browser acceptance `5323801564`, and maintainer review-policy
override `5326076971`.

Still not implemented: a live log/SSE view, config editing, approvals, session actions, WebSocket
control, or remote/LAN access.

## Governed Dashboard Child Outcome

[WEB-001-A](WEB-001-A-dashboard-read-only-visual-shell.md) and
[I195](../../iterations/I195-dashboard-read-only-visual-shell.md) are Complete/Closed. Their bounded
read-only visual-shell outcome does **not** reopen or satisfy WEB-001's separately gated live/write/
control/remote residual acceptance.

WEB-001-A did not reuse I129 acceptance as authorization. It consumed the existing GET-only loopback
snapshot surfaces and added cohesive navigation, visual hierarchy, responsive rendering, accessible
keyboard/focus behavior, useful empty states, and HTML presentation parity for `/extensions` while
preserving JSON/plain-text negotiation, config masking, output redaction, and HTML escaping.

## Opt-In Token Delivery Security Residual

[SEC-002](SEC-002-dashboard-token-delivery-boundary.md) separately owns the pre-existing ADR-031
gap where `[dashboard] loopback_only = false` generates and enforces a memory-only token but defines
no compliant operator-delivery channel. SEC-002 is Refinement / Unclaimed / Selected Iteration None;
it must choose an ADR-backed delivery, authentication-redesign or mode-deprecation contract before
implementation. It does not alter WEB-001-A/I195 completion or reuse their authorization.

## Target Model (Not Current Implementation)

```
   Browser (loopback)  ◄── HTTP / WS ──►  Talos runtime (tokio)
                                             │  WebServer task (axum + static assets)
                                             │  reads/subscribes via proper channels
                                             ▼
                                  Session / Agent / Config / Logs
```

The diagram remains a target-space sketch only; WebSocket/control behavior is not implemented or
authorized by I195.

## Product MVP Target (Not Complete)

- In-process loopback-only HTTP server serving the completed read-only visual shell.
- Read-only status/history/governance/config/extensions presentation is delivered; a log-tail (SSE)
  view remains future work.
- Config editing via CONF-001 primitives (secret masking), only after an explicit write/permission
  design is accepted.
- Any future web-driven action must use the same permission pipeline as the TUI and receive separate
  authorization.

## Open Questions

See `docs/proposals/embedded-web-control-surface.md` for the remaining design space (realtime
transport, lifecycle, auth, and WEB-001 vs REMOTE-001 handler convergence). The completed I195 shell
does not settle those residual questions.

## Project Management UI (GOV-003 integration)

Potential future project-management views remain proposal-level unless separately selected and
claimed. If implemented, they must read the same `docs/` sources as GOV-003's built-in governance
logic and avoid a second source of truth.

## Required Reads

- `docs/decisions/031-web-loopback-dashboard-boundary.md`
- `docs/proposals/embedded-web-control-surface.md`
- `docs/backlog/active/WEB-001-A-dashboard-read-only-visual-shell.md`
- `docs/backlog/active/SEC-002-dashboard-token-delivery-boundary.md`
- `docs/iterations/I195-dashboard-read-only-visual-shell.md`
- `docs/backlog/active/EXT-002-oh-my-pi-feature-analysis.md`
- `crates/talos-rpc/src/` (existing JSON-RPC infrastructure)
- `docs/backlog/active/REMOTE-001-remote-session-protocol.md`
- `docs/backlog/active/OBS-001-observability-prompt-assets.md`
- `docs/backlog/active/CONF-001-config-editing.md`
- `docs/decisions/006-event-architecture-boundary.md` (ADR-006 — no global bus)
