# WEB-001: Embedded Web Control Surface

| Field | Value |
|-------|-------|
| Story ID | WEB-001 |
| Priority | P2 (elevated 2026-06-27 — product differentiation track; informed by EXT-002/omp.sh reference) |
| Status | Partial — loopback-only read-only snapshot API, security hardening, and an HTML link landing page are implemented. No rendered dashboard pages, SSE log viewer, web config editor, or web actions exist. |
| Depends On | talos-rpc infrastructure; OBS-001 (logs); CONF-001 (config primitives) |
| Relates To | REMOTE-001 (remote/P2P surface — may share a handler backbone); OBS-001; CONF-001 |
| Blocks | Browser dashboard; live log viewer; web config editor |
| Origin | User request 2026-06-17; reprioritized 2026-06-27 as a Talos特色优势 candidate, with EXT-002/omp.sh as reference implementation research input |

## Outcome

Embed a **local web server + embedded static UI** ("rust-embedded-web": Rust HTTP framework +
assets embedded via `rust-embed`/`include_dir`) inside the Talos runtime, started alongside the
TUI. Opening `http://localhost:<port>` gives a richer control surface for status, live logs,
config editing, history inspection, and approvals/interaction — a parallel surface to the TUI, not
a replacement.

This is no longer treated as a distant novelty. WEB-001 is a product differentiation research track:
Talos should study existing browser control surfaces such as the omp.sh/EXT-002 reference, then
define a narrow loopback-only MVP that preserves the same permission, config, session, and
governance boundaries as the TUI.

## Gate Status

ADR-031 accepted the WEB-001 MVP boundary on 2026-07-01 and was amended on 2026-07-02 for a
default-on TUI lifecycle with config opt-out. The first implementation slice is now unblocked for
a loopback-only, read-only dashboard. The per-process bearer token is opt-in via
`[dashboard] loopback_only = false`; the default is loopback-bind-only for the common single-user
case.
Remote access, web approvals, config writes, browser automation, WebSocket control, and any
write/session-mutating route remain out of scope until a later security review.

T112/T113 security review update (2026-07-02): `docs/reference/WEB-DASHBOARD-BROWSER-SECURITY-REVIEW-2026-07-02.md`
recorded the dashboard/browser boundary review. T113 added dashboard output-boundary redaction for
snapshot data and regression coverage proving that, when `loopback_only = false`, unknown paths
without a token are rejected before returning route information. These slices are complete; the
prior “in Review” label did not reflect their completed evidence.

## Current Implementation Boundary

`talos-dashboard` serves a loopback-only, read-only snapshot API at `/status`, `/history`,
`/governance`, `/config`, and `/extensions`. `/` is an inline HTML landing page containing links to
those representations. The routes return JSON or plain text; they do not render a dashboard UI.

Not implemented: status/history/config pages, a live log/SSE view, governance tables or Kanban,
client-side navigation/state, config editing, approvals, session actions, WebSocket control, or
remote/LAN access. The shipped capability is therefore an API foundation, not the product web
control surface described by this story.

## Target Model (Not Current Implementation)

```
   Browser (loopback)  ◄── HTTP / WS ──►  Talos runtime (tokio)
                                             │  WebServer task (axum + static assets)
                                             │  reads/subscribes via proper channels
                                             ▼
                                  Session / Agent / Config / Logs
```

## Product MVP Target (Not Complete)

- In-process loopback-only HTTP server serving a rendered static page.
- Read-only status/history/governance/config presentation plus a log-tail (SSE) view.
- Config editing via CONF-001 primitives (secret masking), only after an explicit write/permission
  design is accepted.
- Any web-driven action uses the same permission pipeline as the TUI.

## Open Questions

See `docs/proposals/embedded-web-control-surface.md` for the full design space (framework choice,
UI toolchain vs Node.js build constraint, realtime transport, lifecycle, auth, and WEB-001 vs
REMOTE-001 handler convergence).

## Project Management UI (GOV-003 integration)

Beyond the status/log/config pages, the web dashboard should expose project
management views backed by GOV-003's built-in governance logic:

- **Iteration Board**: Kanban-style view of active iteration stories
  (columns: Planned / In Progress / Review / Complete)
- **Product Backlog**: Filterable table with priority, status, dependencies
- **ADR Index**: Decision records with status and dates
- **Validation Status**: Governance harness check results

These pages read from the same `docs/` sources as GOV-003's context injection
layer — single source of truth, no duplication. See GOV-003 for the
governance state model and data sources.

## Required Reads

- `docs/decisions/031-web-loopback-dashboard-boundary.md`
- `docs/proposals/embedded-web-control-surface.md`
- `docs/backlog/active/EXT-002-oh-my-pi-feature-analysis.md`
- `crates/talos-rpc/src/` (existing JSON-RPC infrastructure)
- `docs/backlog/active/REMOTE-001-remote-session-protocol.md`
- `docs/backlog/active/OBS-001-observability-prompt-assets.md`
- `docs/backlog/active/CONF-001-config-editing.md`
- `docs/decisions/006-event-architecture-boundary.md` (ADR-006 — no global bus)
