# A2A-001: Multi-Instance Discovery And Communication Spike

| Field | Value |
|---|---|
| Story ID | A2A-001 |
| Type | Architecture and security spike |
| Priority | P3 |
| Status | Deferred (2026-07-16, I133) — ADR-044: no product need; five threat-model risks unresolved; all implementation paths violate P140 non-goals |
| Source | [GitHub Issue #40](https://github.com/wjhuang88/talos/issues/40) |
| Depends On | `RUNTIME-001`, `REMOTE-001` |

## Goal

Determine whether Talos should support any bounded communication or discovery between independent instances before a network protocol is considered.

## Scope

- Produce a threat model for discovery, identity, transport authentication, authorization, capability advertisement, data retention, and revocation.
- Compare an explicit host-managed connection boundary with automatic discovery.
- Record an ADR-ready recommendation, deferment, or rejection with dependencies and migration implications.

## Acceptance

- A reviewable ADR or explicit defer/reject record defines the decision and its credential/transcript exposure boundary.
- The result preserves the no-global-event-bus boundary and introduces no implicit authority between instances.
- Protocol work is split into separately reviewed stories only after acceptance.

## Non-Goals

- No automatic discovery, network service, protocol, credential transfer, or multi-agent orchestration.
- No new runtime dependency solely for this exploratory work.

## Required Reads

- `docs/backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md`
- `docs/backlog/active/REMOTE-001-remote-session-protocol.md`
- `docs/decisions/006-event-architecture-boundary.md`
- `docs/decisions/021-tool-call-protocol-architecture.md`
