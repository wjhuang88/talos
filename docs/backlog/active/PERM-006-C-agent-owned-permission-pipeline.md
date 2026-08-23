# PERM-006-C: Agent-Owned Permission, Approval, Authorization, And Execution Pipeline

> Document status: Complete

| Field | Value |
|---|---|
| Story ID | PERM-006-C |
| Type | Agent / Architecture Story |
| Priority | P0 |
| Status | Complete |
| Source | [GitHub Issue #55](https://github.com/wjhuang88/talos/issues/55) |
| Selected Iteration | I221 Complete / Closed |
| Depends On | PERM-006-A/I189, B/I219 and ADR-067/I220 Complete; prerequisite for PERM-006-D/E satisfied |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 mainline implementation session 2026-08-23 |
| Work Slice | PERM-006-C implementation under Accepted ADR-067; see I221 owner for exact boundaries |
| Claimed At | 2026-08-23 |
| Source Issue | #55 |
| Governance Claim PR | #375 |
| Authorization Mode | Independent review |
| Authorization Evidence | ADR-067 Accepted through PR #373 merge `5d2d2dcf`; claim #375 merged as `d662501c`; implementation PR #376 exact head `aed71fb4` passed CI `32640691772`, independent review `5386153429`, merge-time CAS and merge `f9e6706d`. |
| Implementation PR | #376 |
| Last Updated | 2026-08-23 |
| Handoff / Release Condition | Closed; PERM-006-D/E and TOOL-024 require separate owners and effective claims. |

## Identity / Goal / Value

Move permission evaluation, Ask resolution, grant installation, exact authorization, and tool execution into one authoritative agent-owned pipeline; product surfaces provide only approval adapters.

## Scope

- ApprovalResolver abstraction and terminal/TUI/headless/runtime/RPC-MCP adapters.
- One evaluation, final decision hooks, exact authorization, original tool execution.
- Remove duplicate permission-aware wrappers and runtime engines.

## Exclusions

- No typed effect/resource migration, persistent grants, ACP implementation, or sandbox replacement.

## Dependencies

PERM-006-A/I189, PERM-006-B/I219, ADR-067/I220 and implementation I221 are Complete. C's
prerequisite gate for PERM-006-D/E is satisfied; those children remain separately governed.

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
Present this Story as shipped only with implementation commit `49d1546c` and merge `f9e6706d`.

## Required Reads

- docs/backlog/active/PERM-006-A-structured-permission-decisions.md
- docs/backlog/active/PERM-006-B-scoped-grant-store.md
- crates/talos-agent/src/tool_execution.rs
- crates/talos-cli/src/approval.rs
- crates/talos-cli/src/event_loop.rs
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

## 2026-08-23 I221 Activation Checkpoint

I221 claim #375 is effective at `main@d662501c` after exact head `de99de1c`, base `055e5c6b`, CI
`32620749103`, independent review `5384445091`, successful merge-time CAS and merge `d662501c`;
implementation is now authorized only within the I221 slice. The owner is in Review/Claimed while
the local candidate is converged. The non-TUI interactive resolver now delegates approval input to
the existing event-loop stdin reader, so timeout or cancellation closes the pending response
without leaving a competing reader that can consume later input. Completion remains pending a
pre-existing implementation commit, exact-head CI, independent
permission/security/API review and merge-time CAS.

## 2026-08-23 I221 Completion Checkpoint

Completion Commit: `49d1546c3748930177655dbedc7f3665780d92ab`.

I221 implementation commit `49d1546c3748930177655dbedc7f3665780d92ab` reached `main` through
PR #376 merge `f9e6706d39a3c612061c6a1fb68e31bd24c29904`. Final exact head
`aed71fb432086a20d1fdf2a927e0d7bf7b1f672c` passed CI `32640691772`, independent
permission/security/API approval `5386153429` and merge-time CAS. The implementation satisfies
the exactly-once evaluator, bounded resolver, final gate, fail-closed concurrency/deadline and
cross-surface compatibility acceptance without changing Dashboard/I213 or excluded product lanes.

The non-blocking third-party projection contract is assigned to PERM-006-E: tools whose arguments
contain secrets must override the default `AgentTool::project_input()` before that conformance
child can close. This status closeout cites the pre-existing implementation commit and does not
self-certify completion.
