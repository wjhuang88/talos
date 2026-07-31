# AGENT-003: Multi-Agent Architecture And Delegation Model

| Field | Value |
|---|---|
| Story ID | AGENT-003 |
| Type | Architecture Spike |
| Priority | P3 |
| Status | Deferred — proposal retained; runtime and permission foundations not selected |
| Source | [GitHub Issue #30](https://github.com/wjhuang88/talos/issues/30) |
| Selected Iteration | None |
| Depends On | PERM-006 permission convergence; SESSION-009 client/session ownership; TASK-001 remains Deferred |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #30 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-01 |
| Handoff / Release Condition | Establish an effective claim and select an iteration before implementation. |

## Identity / Goal / Value

Define how Talos could delegate bounded work to subordinate agents while preserving one understandable user session, explicit authority, cancellation, and cost visibility.

## Scope

- Compare orchestrator/worker, peer, blackboard, and hybrid collaboration models.
- Define sub-agent lifecycle, result ownership, cancellation propagation, budgets, and artifact exchange.
- Define how tool presentation and permissions are reduced for each subordinate agent.

## Exclusions

- No multi-agent runtime implementation, autonomous recursive delegation, or distributed swarm.
- No implicit inheritance of parent grants or unbounded token/tool budgets.

## Dependencies

PERM-006 permission convergence; SESSION-009 client/session ownership; TASK-001 remains Deferred

## Decision Links And Constraints

- Sub-agents must consume the canonical agent execution and permission pipeline.
- No global event bus or duplicate durable-session authority.
- Any public orchestration API requires ADR and compatibility review.

## Uncertainty And Validation Path

Resume only after the single-agent permission/session contracts are stable and a concrete first delegation use case is selected.

## State / Status Owners

- Story status and acceptance: this file.
- Remote request state and discussion: GitHub Issue #30.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## User-Facing Documentation

Update user or SDK documentation only when observable behavior or a public integration contract changes.
Do not present this Story as shipped while it remains Deferred.

## Required Reads

- docs/backlog/active/PERM-006-permission-pipeline-convergence.md
- docs/backlog/active/SESSION-009-multi-client-session-architecture.md
- docs/backlog/active/TASK-001-persistent-task-runtime-spike.md
- docs/decisions/006-event-architecture-boundary.md

## Acceptance For Behavior / Technical Work

- An ADR selects one bounded initial orchestration model and states recursion/concurrency limits.
- Permission, cancellation, durable evidence, and user-visible cost ownership are explicit.
- No implementation begins without a dedicated iteration and claim.

## Residual Destination

A future selected delegation Story should be split from this Deferred architecture owner.
