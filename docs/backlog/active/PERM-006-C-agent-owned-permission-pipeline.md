# PERM-006-C: Agent-Owned Permission, Approval, Authorization, And Execution Pipeline

| Field | Value |
|---|---|
| Story ID | PERM-006-C |
| Type | Agent / Architecture Story |
| Priority | P0 |
| Status | Blocked / Unclaimed — A/B gates satisfied; ADR-067 Proposed through effective I220, awaiting acceptance and separate I221 implementation claim |
| Source | [GitHub Issue #55](https://github.com/wjhuang88/talos/issues/55) |
| Selected Iteration | None for implementation; I220 Active/Claimed owns the decision prerequisite |
| Depends On | PERM-006-A/I189 and B/I219 Complete; blocked on Accepted ADR-067 and later I221 claim; prerequisite for PERM-006-D/E |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #55 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-23 |
| Handoff / Release Condition | Accept ADR-067 from I220, then establish a separate effective I221 claim before implementation. I213/I220 decision-only non-overlap is effective through the recorded I220 claim merge. |

## Identity / Goal / Value

Move permission evaluation, Ask resolution, grant installation, exact authorization, and tool execution into one authoritative agent-owned pipeline; product surfaces provide only approval adapters.

## Scope

- ApprovalResolver abstraction and terminal/TUI/headless/runtime/RPC-MCP adapters.
- One evaluation, final decision hooks, exact authorization, original tool execution.
- Remove duplicate permission-aware wrappers and runtime engines.

## Exclusions

- No typed effect/resource migration, persistent grants, ACP implementation, or sandbox replacement.

## Dependencies

PERM-006-A/I189 and PERM-006-B/I219 are Complete. Implementation remains blocked on Accepted
ADR-067 and a separate effective I221 claim; C remains the prerequisite for PERM-006-D/E.

## Decision Links And Constraints

- Approval failure, channel closure, cancellation, timeout, or poisoned state fails closed.
- Permission-relevant normalization happens before approval and the approved value is executed.
- Private input projection remains separate from authoritative evaluation/execution.
- ADR-065 deferred final hook transport/version semantics to this child; ADR-067 must resolve them
  before code changes.

## Uncertainty And Validation Path

Characterize current cross-surface behavior before migration; retain compatibility adapters only with an explicit removal gate.

## State / Status Owners

- Story status and acceptance: this file.
- Remote request state and discussion: GitHub Issue #55.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## User-Facing Documentation

Update user or SDK documentation only when observable behavior or a public integration contract changes.
Do not present this Story as shipped while it remains Blocked.

## Required Reads

- docs/backlog/active/PERM-006-A-structured-permission-decisions.md
- docs/backlog/active/PERM-006-B-scoped-grant-store.md
- crates/talos-agent/src/tool_execution.rs
- crates/talos-cli/src/approval.rs
- crates/talos-runtime/src/

## Acceptance For Behavior / Technical Work

- One invocation performs exactly one authoritative evaluation.
- Ask is resolved inside the agent pipeline through a surface adapter.
- Hooks observe the final decision that gates execution.
- Closed/non-interactive approval paths deny and execute zero tools.

## Residual Destination

Retained wrappers must be documented as policy-free compatibility adapters with a removal issue.

## 2026-08-23 Dependency And Decision Checkpoint

PERM-006-A/I189 and PERM-006-B/I219 are Complete/Closed through their recorded implementation
evidence. That clears the old dependency blocker but does not make C implementation-ready.

Read-only cross-surface analysis found two unresolved architecture/security contracts: current
composition roots still own multiple evaluation/wrapper paths, and `AfterPermissionCheck` does not
yet represent the final execution-gating decision. ADR-065 explicitly deferred that public hook
migration. Planned decision-only I220 therefore owns ADR-067 and the current-path/migration matrix;
PERM-006-C remains Blocked/Unclaimed, and later I221 must obtain its own claim before any Rust,
Cargo, wrapper, hook, Runtime, MCP or behavior change.
