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

## Architecture

### Preferred Model: Relay + Peer-to-Peer (WireGuard-inspired)

Talos is open-source and should not require a mandatory central server. A WireGuard-style
architecture provides peer discovery and NAT traversal via a lightweight relay while session
data flows directly between peers.

```
        ┌─────────────────────┐
        │   Discovery Relay   │  (lightweight, optional)
        │  - Peer registry    │
        │  - NAT traversal    │
        │  - Public key store │
        └──┬──────────────┬───┘
           │              │
     "connect to      "here is
      desktop"         peer info"
           │              │
    ┌──────▼──────┐  ┌───▼──────────┐
    │ Mobile App  │  │ Desktop Talos │
    │ (read-only  │◄─┤ (full agent)  │
    │  client)    │  │               │
    └─────────────┘  └───────────────┘
          ▲                  ▲
          └── P2P encrypted ─┘
              (after relay handshake)
```

**How it works:**

1. Both peers register with the relay using public keys
2. Mobile client queries relay: "find peer with device ID `desktop-xyz`"
3. Relay facilitates NAT traversal (STUN/TURN-style) and key exchange
4. After handshake, peers communicate directly via encrypted P2P channel
5. Relay is not in the data path — session content never passes through it

**Relay is optional:**

```
Without relay:
  - Direct IP:port connection (LAN, VPN, known endpoints)
  - QR code pairing (exchange keys + endpoint out-of-band)

With relay:
  - Peer discovery across NATs
  - Fallback relayed transport if direct P2P fails
```

**Why this fits Talos:**

- Open-source product → no vendor lock-in to a central service
- Privacy-first → session data never touches a third party
- Works offline → LAN P2P without internet
- Scales horizontally → each desktop is its own "server"

### Transport Options

| Option | Role | Notes |
|--------|------|-------|
| **QUIC** | Peer-to-peer transport | Built-in encryption, NAT traversal friendly, multiplexed streams |
| **Noise Protocol** | Key exchange + encryption | Same framework as WireGuard, well-audited |
| **WebRTC** | NAT traversal | Mature STUN/TURN support, works in browsers |
| **mDNS / LAN discovery** | Local peer discovery | Zero-config on same network |
| **WebSocket** | Relay fallback | Simple, firewall-friendly, well-supported everywhere |

QUIC + Noise handshake is the preferred combination — same cryptographic foundation
as WireGuard, with QUIC providing the transport layer.

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
- `talos-agent`: dispatch remote commands via P2P channel
- `talos-remote` (new crate): peer discovery, NAT traversal, encrypted transport
- `talos-cli`: `talos remote serve` to start peer listener, `talos remote connect <peer>` to link

## Open Questions

1. Peer identity: public key fingerprint, device name, or human-readable alias?
2. Relay: self-hosted reference implementation, optional community relay, or both?
3. How does a mobile client authenticate to the desktop peer? QR code + key exchange?
4. Should P2P connections be persistent or on-demand per query?
5. How does this interact with the permission pipeline? Remote commands must go through
   the same approval flow as local commands.
6. QUIC vs WebRTC data channels for the P2P transport layer?
7. Should session history be push-synced (server → client) or pull-queried (client → server)?
8. What is the minimum viable slice: LAN-only P2P (mDNS + direct connect), relay-assisted, or full NAT traversal?
9. Mobile app: native (Swift/Kotlin) vs cross-platform (Flutter/React Native) vs PWA?
10. Can the same P2P protocol support multi-agent coordination (agent ↔ agent)?

## Constraints

- Rust-first. Server component must be Rust. Client SDKs may be in other languages.
- No secrets in transport without TLS.
- Remote commands must respect the same tool permission pipeline as local commands.
- Session data must remain JSONL-source-of-truth; remote protocol is a view layer.
- Must not introduce persistent background services without explicit user opt-in.

## Recommended Research Path

1. Spike a local P2P connection between two Talos instances on the same network
   (mDNS discovery + QUIC transport)
2. Implement Noise protocol key exchange for peer authentication
3. Design the session query/command protocol over the P2P channel
4. Add optional relay support for NAT traversal (reference implementation)
5. Build a minimal read-only mobile client (Flutter or React Native POC)

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
