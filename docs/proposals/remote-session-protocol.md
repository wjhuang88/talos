# Remote Session Protocol Proposal

> Status: Research proposal
> Created: 2026-06-15
> Backlog: REMOTE-001

## Context

Talos currently runs as a local CLI/TUI agent tied to a specific workspace. Users have asked
for the ability to interact with active sessions from remote clients — for example, checking
session status from a mobile app, sending instructions to a running agent, or monitoring
conversation history without being at the terminal.

This is not a near-term implementation target. It requires significant architecture decisions
about networking, authentication, concurrency, and data synchronization. This proposal captures
the design space and research questions.

## Use Cases

1. **Session monitoring**: View active session state (current turn, model, token usage) from
   a mobile app or web dashboard.
2. **Remote instruction**: Send a prompt to a running Talos instance from a remote client.
3. **Cross-device continuity**: Start a session on desktop, continue on mobile, review history
   on either device.
4. **Multi-agent coordination**: One Talos instance queries another's session state.

## Design Space

### Transport Options

| Option | Pros | Cons |
|--------|------|------|
| **JSON-RPC over WebSocket** | Bidirectional, already in `talos-rpc` patterns | Requires persistent connection, port management |
| **HTTP REST + SSE** | Simple, firewall-friendly, stateless queries | SSE for push only, no bidirectional commands |
| **gRPC** | Strong typing, streaming, good tooling | Heavier dependency, protobuf build step |
| **Unix domain socket + JSON** | Local-first, no network exposure, simple | Remote requires tunneling |

### Authentication & Security

- Session-level access tokens (not global API keys)
- Read-only vs read-write permission per client
- TLS for remote connections
- Rate limiting per client

### Data Model

```
RemoteSession
  id: UUID
  workspace_root: String
  status: Idle | Processing | Error
  current_turn: Option<TurnInfo>
  history: Vec<Message> (paginated)
  token_usage: UsageStats
  model: String

RemoteCommand
  session_id: UUID
  action: SendPrompt | Cancel | Fork | ListSessions
  payload: String (prompt text)
```

### Integration Points

- `talos-session`: query active session state
- `talos-agent`: dispatch remote commands
- `talos-rpc`: existing JSON-RPC infrastructure (potential reuse)
- `talos-cli`: optional server mode (`talos serve`)

## Open Questions

1. Should Talos run a persistent daemon/server, or only expose sessions on-demand?
2. Should remote access be a separate crate (`talos-remote`) or integrated into `talos-rpc`?
3. How does this interact with the permission pipeline? Remote commands must go through
   the same approval flow as local commands.
4. Should session history be push-synced (server → client) or pull-queried (client → server)?
5. What is the minimum viable slice: read-only session query, or full bidirectional control?
6. How does this interact with workspace-scoped session topology (MEM-004)?
7. Mobile app: native (Swift/Kotlin) vs cross-platform (Flutter/React Native) vs PWA?

## Constraints

- Rust-first. Server component must be Rust. Client SDKs may be in other languages.
- No secrets in transport without TLS.
- Remote commands must respect the same tool permission pipeline as local commands.
- Session data must remain JSONL-source-of-truth; remote protocol is a view layer.
- Must not introduce persistent background services without explicit user opt-in.

## Recommended Research Path

1. Prototype a minimal read-only HTTP endpoint (`GET /sessions/:id`) in `talos-rpc`
2. Evaluate WebSocket vs SSE for push events
3. Design the authentication model (session tokens)
4. Decide mobile client technology
5. Spike a two-way command flow

## Non-Goals (First Slice)

- No persistent daemon mode
- No mobile app implementation
- No multi-instance clustering
- No cross-workspace session federation
- No end-to-end encryption beyond TLS

## Required Reads

- `crates/talos-rpc/src/` (existing JSON-RPC infrastructure)
- `crates/talos-session/src/lib.rs` (session storage)
- `crates/talos-agent/src/session.rs` (AppServerSession)
- `docs/backlog/active/MEM-004-workspace-session-topology.md`
- `docs/decisions/006-event-architecture-boundary.md`
