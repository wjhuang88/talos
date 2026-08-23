# PERM-006-C: Agent-Owned Permission, Approval, Authorization, And Execution Pipeline

| Field | Value |
|---|---|
| Story ID | PERM-006-C |
| Type | Agent / Architecture Story |
| Priority | P0 |
| Status | Active / Claimed proposed by PR #375; ineffective until target-branch merge |
| Source | [GitHub Issue #55](https://github.com/wjhuang88/talos/issues/55) |
| Selected Iteration | I221 implementation claim proposed; ineffective until merge |
| Depends On | PERM-006-A/I189, B/I219 and ADR-067/I220 Complete; awaiting I221 claim; prerequisite for PERM-006-D/E |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 mainline implementation session 2026-08-23 |
| Work Slice | PERM-006-C implementation under Accepted ADR-067; see I221 owner for exact boundaries |
| Claimed At | 2026-08-23 |
| Source Issue | #55 |
| Governance Claim PR | #375 |
| Authorization Mode | Independent review |
| Authorization Evidence | ADR-067 Accepted through PR #373 merge `5d2d2dcf`; fresh exact-head claim CI/review/CAS required for I221. |
| Implementation PR | Not started |
| Last Updated | 2026-08-23 |
| Handoff / Release Condition | Claim #375 is ineffective until merge. After merge, implement only I221 from the claim merge or later `main`; I213/I220 boundaries remain protected. |

## Identity / Goal / Value

Move permission evaluation, Ask resolution, grant installation, exact authorization, and tool execution into one authoritative agent-owned pipeline; product surfaces provide only approval adapters.

## Scope

- ApprovalResolver abstraction and terminal/TUI/headless/runtime/RPC-MCP adapters.
- One evaluation, final decision hooks, exact authorization, original tool execution.
- Remove duplicate permission-aware wrappers and runtime engines.

## Exclusions

- No typed effect/resource migration, persistent grants, ACP implementation, or sandbox replacement.

## Dependencies

PERM-006-A/I189, PERM-006-B/I219 and ADR-067/I220 are Complete. Implementation remains gated by
a separate effective I221 claim; C remains the prerequisite for PERM-006-D/E.

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

## 2026-08-23 ADR-067 Acceptance And I221 Readiness

ADR-067 and its current-path/migration matrix were Accepted through PR #373 merge `5d2d2dcf`.
PERM-006-C is now Ready/Unclaimed; no implementation authority exists until a separate I221 claim
is effective. The historical checkpoint above is retained unchanged.
