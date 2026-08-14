# SESSION-009: Multi-Client Session Architecture

| Field | Value |
|---|---|
| Story ID | SESSION-009 |
| Type | Architecture / Session Story |
| Priority | P1 |
| Status | Refinement — ADR and iteration selection required |
| Source | [GitHub Issue #46](https://github.com/wjhuang88/talos/issues/46) |
| Selected Iteration | None |
| Depends On | ADR-042 durable sessions; ADR-052 SDK boundary; blocks ACP-001 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #46 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-01 |
| Handoff / Release Condition | Establish an effective claim and select an iteration before implementation. |

## Identity / Goal / Value

Define a session-owned runtime that supports attach/detach, replay, event fan-out, and one active controller without coupling an agent instance to one transport connection.

## Scope

- Define Session Runtime and Client Attachment identities and lifecycle.
- Define ordered replay/fan-out and client-specific cursor/viewport state.
- Define one-controller/many-observer command ownership and takeover behavior.
- Define reconnect and durable-session compatibility.

## Exclusions

- No ACP implementation, collaborative multi-controller editing, or multi-user authorization.
- No renderer state in durable session facts and no new global event bus.

## Dependencies

ADR-042 durable sessions; ADR-052 SDK boundary; blocks ACP-001

## Decision Links And Constraints

- Transcript/execution/approval facts are session-owned; viewport/layout/cursor state is client-owned.
- Event ordering and replay use one canonical sequence contract.
- Breaking public runtime/session changes require ADR and migration guidance.

## Uncertainty And Validation Path

Inventory current `talos-runtime`, durable session, TUI bridge, RPC, and MCP attachment assumptions before choosing the public contract.

## State / Status Owners

- Story status and acceptance: this file.
- Remote request state and discussion: GitHub Issue #46.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## User-Facing Documentation

Update user or SDK documentation only when observable behavior or a public integration contract changes.
Do not present this Story as shipped while it remains Refinement.

## Required Reads

- docs/decisions/042-embedded-durable-runtime-session-boundary.md
- docs/decisions/052-sdk-publication-and-composition-boundary.md
- docs/backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md
- crates/talos-runtime/
- crates/talos-session/

## Acceptance For Behavior / Technical Work

- Session and Client Attachment concepts and ownership are documented in an accepted ADR.
- One active controller and observer semantics are deterministic.
- Replay, reconnect, cancellation, approval, and detach races have tests in the selected implementation Story.
- ACP-001 references this accepted contract rather than connection-owned agents.

## Residual Destination

Implementation slices must be selected after ADR acceptance; ACP remains blocked until then.

## Downstream Work-Domain Dependency

WORK-001 defines the separately governed shared work/evaluation chain. WORK-001-A / I196 records
only the P0 canonical work-domain, identity/revision and migration contract. It must preserve this
Story's ownership of attach/detach, reconnect, replay and multi-client semantics, must not select or
implement SESSION-009, and must leave later shared-domain APIs compatible with a future
session-owned attachment model.
