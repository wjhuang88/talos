# REMOTE-001: Remote Session Protocol

| Field | Value |
|-------|-------|
| Story ID | REMOTE-001 |
| Priority | P4 (Future Research) |
| Status | Research |
| Depends On | MEM-004 (workspace topology); talos-rpc infrastructure |
| Blocks | Mobile app; web dashboard; cross-device continuity |
| Origin | User request on 2026-06-15 |

## Outcome

Talos exposes a remote protocol allowing authorized clients to query session state and issue
instructions. Initial target: read-only session monitoring from a mobile app. Full bidirectional
control requires design decisions about networking, authentication, and permission gating.

## Target Model

```
Mobile App / Web Dashboard
        │
   HTTPS + Token Auth
        │
   ┌────▼──────────────────────┐
   │  Talos Server (talos-rpc) │
   │  ┌──────────────────────┐ │
   │  │ RemoteSessionHandler │ │
   │  └──────────┬───────────┘ │
   │             │              │
   │  ┌──────────▼───────────┐ │
   │  │  Session / Agent     │ │
   │  └──────────────────────┘ │
   └───────────────────────────┘
```

## Minimum Viable Slice

- Read-only HTTP endpoint: `GET /sessions/:id` → session state + recent history
- Session-level access token (not global API key)
- Stateless, no persistent daemon (started with `talos serve`)

## Open Questions

See `docs/proposals/remote-session-protocol.md` for full design space discussion.

## Architecture Reservation Points

These are decisions in current/near-term work that could constrain REMOTE-001
if not designed with remote access in mind. None require implementation now,
but all should be validated before closing their respective stories.

### 1. `talos-rpc` Method Dispatch (already covered by design)

**Current state**: `talos-rpc` has typed method handlers (`methods/agent.rs`,
`methods/system.rs`). The `Runtime` trait decouples method handlers from
concrete implementations.

**Reservation**: The I036 convergence decision already records that
REMOTE-001 and WEB-001 should share the same handler backbone in `talos-rpc`.
When WEB-001 implements its loopback HTTP server, the session query methods
it exposes should be designed as `talos-rpc` methods first, with the HTTP
layer as a thin transport wrapper. This ensures REMOTE-001 can consume the
same handlers through a different transport (P2P/QUIC) without rewriting
session access logic.

### 2. Session State Access (gate on SESSION-001)

**Current state**: Session state lives in `talos-session` (SQLite + JSONL).
Access is through local file I/O and an in-process session manager.

**Reservation**: SESSION-001 (Interactive Session Lifecycle) should ensure
that session state queries (list, search, status, history) are callable
through a typed API, not just through CLI flags. The existing
`talos_session::SessionManager` already has `list_recent()`, `search()`,
`get_session()` — these should be treated as the canonical session query
surface. No new API needed, but the existing one must remain the sole
access path (no ad-hoc SQLite queries scattered in CLI code).

### 3. Permission Pipeline (already covered)

**Current state**: Permission checks work in all modes (TUI, print, MCP).
The pipeline is already transport-agnostic — it doesn't care whether the
tool call originated from a local TUI or a remote client.

**Reservation**: No action needed. The existing `PermissionEngine` +
`ToolRegistry` boundary is already REMOTE-001-ready.

### 4. Event Observation (future WEB-001 decision)

**Current state**: Agent events flow through typed `mpsc` channels.
There is no push-based event observation for external consumers.

**Reservation**: When WEB-001 implements SSE log-tail for the web dashboard,
it should emit events through a reusable observation interface, not as a
one-off HTTP handler. REMOTE-001 will need the same event stream for
real-time session monitoring over P2P.

### 5. State Serialization (validate before CMD-001 closes)

**Current state**: `StatusSnapshot`, `ChatMessage`, `SessionInfo` types
exist but are not designed for remote serialization.

**Reservation**: Before marking any of these types as "stable API", verify
that their serialized forms are suitable for transmission over a network
boundary (no `Instant` timestamps in JSON, no `PathBuf` leaking host
filesystem paths, no unvalidated sizes that could DoS a remote client).

- `docs/proposals/remote-session-protocol.md`
- `crates/talos-rpc/src/`
- `docs/backlog/active/MEM-004-workspace-session-topology.md`
