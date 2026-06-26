# REMOTE-002: iroh P2P Transport Evaluation

**Status**: Research
**Priority**: P4
**Source**: User request 2026-06-26; analysis of [n0-computer/iroh](https://github.com/n0-computer/iroh) v1.0.0
**Iteration**: None yet

## Problem

REMOTE-001 envisions remote session query/control over HTTPS + token auth. For cross-network P2P connectivity (mobile app, cross-device continuity), we need NAT traversal, key-based authentication, and relay fallback. iroh provides all three as a pure-Rust, v1.0 GA library.

## Scope

Evaluate iroh as the transport layer for REMOTE-001's full bidirectional control phase.

### iroh capabilities (from v1.0.0 analysis)

- **NAT traversal**: Hole-punching (~90% success) + stateless relay fallback. No STUN/TURN infrastructure needed.
- **Authentication**: QUIC + TLS 1.3 with raw Ed25519 public key certificates (RFC 7250). Mutual auth by endpoint identity.
- **Protocol dispatch**: ALPN-based Router — custom protocols share one endpoint. Maps directly to talos-rpc method handlers.
- **irpc**: Type-safe RPC over QUIC with streaming support.
- **Pure Rust**: Zero native/C dependencies. Fully ADR-010 compliant. No ADR exception needed.
- **License**: MIT OR Apache-2.0 — compatible with Talos Apache-2.0.

### Evaluation questions

1. **MVP fit**: REMOTE-001's MVP is read-only HTTP. Is iroh overkill for this phase? (Likely yes — use axum + rustls for MVP, iroh for P2P phase.)
2. **Dependency weight**: iroh adds ~50 crates. Is this acceptable for a P4 feature? Can it be feature-gated?
3. **Binary size impact**: QUIC + TLS + relay client increases binary. Measure before committing.
4. **Mobile/Web bindings**: iroh has Swift/Kotlin/JS/WASM bindings. Do they cover our target clients?
5. **Token-to-key mapping**: iroh authenticates by Ed25519 key, not session token. How do we map session tokens to allowed EndpointIds?

### Non-goals

- No immediate implementation (REMOTE-001 is P4).
- No custom relay server deployment.
- No post-quantum cryptography evaluation.

## Acceptance

- [ ] iroh dependency footprint and binary size impact measured on a minimal test binary.
- [ ] Architecture sketch: iroh Endpoint → ALPN Router → talos-rpc ProtocolHandler mapping documented.
- [ ] Decision recorded: pursue (when REMOTE-001 advances to P2P) / monitor / reject.

## Dependencies

- REMOTE-001 (Remote Session Protocol) — parent story.
- talos-rpc infrastructure.

## Required Reads

- `docs/backlog/active/REMOTE-001-remote-session-protocol.md`
- `docs/proposals/remote-session-protocol.md`
- `crates/talos-rpc/src/`
- [iroh docs](https://docs.iroh.computer)
- [iroh repo](https://github.com/n0-computer/iroh)
