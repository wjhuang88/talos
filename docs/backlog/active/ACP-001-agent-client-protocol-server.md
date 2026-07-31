# ACP-001: ACP-Compatible Agent Server

| Field | Value |
|---|---|
| Story ID | ACP-001 |
| Type | Protocol / Runtime Epic |
| Priority | P1 |
| Status | Blocked — SESSION-009 architecture must be accepted first |
| Source | [GitHub Issue #47](https://github.com/wjhuang88/talos/issues/47) |
| Selected Iteration | None |
| Depends On | Blocked by SESSION-009; also consumes PERM-006 and RUNTIME-001 public contracts |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #47 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-01 |
| Handoff / Release Condition | Establish an effective claim and select an iteration before implementation. |

## Identity / Goal / Value

Expose Talos as an ACP-compatible agent through a protocol adapter above `talos-runtime`, preserving the existing execution, permission, session, and cancellation authorities.

## Scope

- ACP initialization and capability negotiation.
- Session create/attach mapping, prompt execution, streaming events, tool events, approval bridge, and cancellation.
- Interoperability fixtures with representative ACP clients.

## Exclusions

- No replacement of MCP, CLI/TUI, or the Talos runtime.
- No connection-owned agent loop, permission bypass, or collaborative multi-controller behavior.

## Dependencies

Blocked by SESSION-009; also consumes PERM-006 and RUNTIME-001 public contracts

## Decision Links And Constraints

- `talos-acp` depends on `talos-runtime`, not CLI/TUI internals.
- Session ownership and event sequencing follow SESSION-009.
- Unresolved Ask in non-interactive contexts fails closed.

## Uncertainty And Validation Path

After SESSION-009 is accepted, pin the ACP protocol/version, compare available Rust implementations, and select a minimal server interoperability slice.

## State / Status Owners

- Story status and acceptance: this file.
- Remote request state and discussion: GitHub Issue #47.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## User-Facing Documentation

Update user or SDK documentation only when observable behavior or a public integration contract changes.
Do not present this Story as shipped while it remains Blocked.

## Required Reads

- docs/backlog/active/SESSION-009-multi-client-session-architecture.md
- docs/backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md
- docs/backlog/active/PERM-006-permission-pipeline-convergence.md
- docs/decisions/052-sdk-publication-and-composition-boundary.md

## Acceptance For Behavior / Technical Work

- A supported ACP client can initialize and create/attach according to SESSION-009.
- Streaming, tool, approval, cancellation, and error semantics map to canonical runtime events.
- Existing CLI/TUI behavior and MCP boundaries remain unchanged.
- Protocol conformance tests require no live provider credentials.

## Residual Destination

Remain Blocked until SESSION-009 is accepted and a protocol-version decision is recorded.
