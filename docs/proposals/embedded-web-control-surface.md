# Embedded Web Control Surface Proposal

> Status: Research proposal (far-future)
> Created: 2026-06-17
> Backlog: WEB-001

## Context

Talos today exposes control only through the TUI (and CLI). Complex interactions — browsing live
status, tailing logs, inspecting tool calls, editing config, reviewing long history — are cramped
in a terminal. The goal is to embed a **local web server + embedded static UI inside the Talos
runtime**, started alongside the TUI, so the user can open `http://localhost:<port>` in a browser
for a richer control surface.

This is a **far-future** goal, not a near-term target. It adds a web framework dependency and
raises architecture questions (concurrency with the TUI loop, security, asset/toolchain, and how it
shares the session/agent/event layer). This proposal captures the design space and open questions.

It is distinct from, but adjacent to, **REMOTE-001** (Remote Session Protocol): REMOTE-001 is about
remote/P2P access across machines; WEB-001 is the **local** browser surface on the same machine.
The two likely share a common session-query/command handler backbone.

## Use Cases

1. **Status dashboard** — current session, active turn, model, token/cost usage, sandbox/permission
   state, shown live in a browser tab.
2. **Log viewer** — tail the observability log (OBS-001) with filtering/search, replacing `tail -f`.
3. **Config editor** — view and edit `~/.talos/config.toml` through a form, reusing CONF-001
   primitives (never echoing `api_key`).
4. **History inspector** — paginated, searchable conversation + tool-call history with provenance.
5. **Approval / interaction** — approve tool calls, submit prompts, fork sessions from the browser
   (a second control surface alongside the TUI, not a replacement).

## Design Space

### Preferred Model: In-process HTTP/WebSocket server + embedded assets

```
   Browser (localhost)  ◄──── HTTP / WS ────►  ┌──────────────────────────┐
                                              │  Talos runtime (tokio)   │
                                              │  ┌────────────────────┐  │
                                              │  │ WebServer task      │  │
                                              │  │ (axum + rust-embed) │  │
                                              │  └─────────┬──────────┘  │
                                              │            │ reads/subscribes
                                              │  ┌─────────▼──────────┐  │
                                              │  │ Session / Agent /   │  │
                                              │  │ Config / Logs       │  │
                                              │  └────────────────────┘  │
                                              │  TUI event loop (unchanged)│
                                              └──────────────────────────┘
```

- **Server**: Rust-native HTTP framework (`axum`/`hyper`). No external web server process; runs as
  a tokio task inside the existing runtime.
- **Assets**: static UI (HTML/CSS/JS) embedded into the binary via `rust-embed` / `include_dir` —
  "rust-embedded-web": the binary is self-contained, no file IO for assets, no install step.
- **Realtime**: Server-Sent Events or WebSocket for live status / log tail.
- **Lifecycle**: started alongside the TUI (default loopback-only), or behind a flag / `talos web`.

### Security

- Bind to **loopback only** (`127.0.0.1`) by default; never expose to the network in the first
  slice (network exposure is REMOTE-001).
- Optional localhost auth token (printed in the TUI / written to a known file), since other local
  users could otherwise reach the port.
- Web-driven actions go through the **same permission pipeline** as TUI actions (no bypass).
- Secrets (e.g. `api_key`) never leave the runtime via the web API; config editing reuses CONF-001
  masking.

### Integration Points

- `talos-rpc` — reuse the existing JSON-RPC handler shapes for session query / command.
- `talos-session`, `talos-agent` — state sources for status/history/approval.
- `talos-config` — config read/write via CONF-001 primitives.
- Observability log sink (OBS-001) — the log viewer is a consumer.
- Must respect **ADR-006** (no global event bus): the web server subscribes through the proper
  existing channels, it does not introduce a new pub/sub backplane.

## Open Questions

1. Web framework choice — `axum` (tokio-native, matches the runtime) vs alternatives? Needs an ADR.
2. UI tech — vanilla HTML/JS/Alpine/Preact bundled at build time, or a heavier framework? See the
   Node.js toolchain constraint below.
3. Realtime transport — SSE vs WebSocket for live tail?
4. Lifecycle — always-on with the TUI, opt-in flag, or a separate `talos web` subcommand?
5. Auth — loopback-only trust vs per-startup token? How is the token surfaced to the user?
6. Convergence with REMOTE-001 — share one handler backbone for local-web and remote-P2P, or keep
   WEB-001 fully separate?
7. Approval UX — how do web-initiated approvals interleave with TUI approvals without races?
8. State sharing — how does the web task observe live state without violating ADR-006?

## Constraints

- **Rust-first.** The server component must be Rust (HTTP framework). The hard constraint forbids
  a Node.js *runtime*; if the UI requires a Node.js *build* toolchain, that is a **Soft-constraint
  tradeoff** to record in an ADR (preferred: keep the UI vanilla / single-file to avoid it).
- Adding any web-framework dependency requires an ADR per dependency discipline (ADR-010).
- Web-driven actions are not privileged — they go through the same permission pipeline as the TUI.
- Localhost-only by default; no background network listener without explicit opt-in.
- No new global event bus (ADR-006); the web surface is a subscriber/view, not a new backplane.

## Recommended Research Path

1. Spike an in-process `axum` server bound to loopback serving one embedded static page.
2. Wire a read-only `/status` (session/model/usage) and a log-tail SSE endpoint off the OBS-001 sink.
3. Add config read/edit via CONF-001 primitives (with secret masking).
4. Evaluate the Node-build-toolchain question and record the ADR (framework + asset embedding).
5. Decide WEB-001 vs REMOTE-001 handler convergence.

## Non-Goals (First Slice)

- No network/remote exposure (that is REMOTE-001).
- No replacement of the TUI; the web is a parallel control surface.
- No multi-instance coordination.
- No mobile client.
- No new global pub/sub (ADR-006).

## Required Reads

- `crates/talos-rpc/src/` (existing JSON-RPC infrastructure)
- `crates/talos-session/`; `crates/talos-agent/` (state sources)
- `docs/decisions/006-event-architecture-boundary.md` (ADR-006 — no global bus)
- `docs/decisions/010-...` (ADR-010 — dependency discipline; web framework needs ADR)
- `docs/backlog/active/OBS-001-observability-prompt-assets.md` (log surface)
- `docs/backlog/active/CONF-001-config-editing.md` (config primitives)
- `docs/backlog/active/REMOTE-001-remote-session-protocol.md` (adjacent remote surface)
