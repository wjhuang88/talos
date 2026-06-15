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

## Required Reads

- `docs/proposals/remote-session-protocol.md`
- `crates/talos-rpc/src/`
- `docs/backlog/active/MEM-004-workspace-session-topology.md`
