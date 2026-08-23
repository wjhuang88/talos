# PERM-006: Permission Decision, Approval, Grant, And Execution Convergence

| Field | Value |
|---|---|
| Story ID | PERM-006 |
| Type | Architecture / Permission Epic |
| Priority | P0 |
| Status | In Progress — A/I189 and B/I219 Complete/Closed; C-E remain blocked/unclaimed |
| Source | [GitHub Issue #52](https://github.com/wjhuang88/talos/issues/52) |
| Selected Iteration | None; completed children A/I189 and B/I219; parent Epic remains unclaimed |
| Depends On | PERM-004/PERM-005 security boundaries; child order A → B → C → D → E |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #52 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-01 |
| Handoff / Release Condition | Establish an effective claim and select an iteration before implementation. |

## Identity / Goal / Value

Converge Talos permission handling into one agent-owned evaluate → approve → grant → authorize → execute pipeline while preserving multi-facet safety, deny precedence, workspace boundaries, and cross-surface behavior.

## Scope

- PERM-006-A structured requests/contexts/decision reports.
- PERM-006-B grant compiler and scoped grant stores.
- PERM-006-C agent-owned execution pipeline.
- PERM-006-D typed effects/resources and in-tree migration.
- PERM-006-E cross-surface conformance gate.

## Exclusions

- No sandbox-policy broadening, persistent background-task grants, ACP implementation, or global event bus.
- Do not merge the pipeline-convergence and typed-resource migrations into one implementation iteration.

## Dependencies

PERM-004/PERM-005 security boundaries; child order A → B → C → D → E

## Decision Links And Constraints

- Configured Deny and hard boundaries always dominate grants or approval.
- Headless unresolved Ask is Deny.
- Each invocation has one authoritative permission evaluation.
- Public API changes follow pre-1.0 ADR/migration rules.

## Uncertainty And Validation Path

Refine each child independently, accept any required ADR, and select only one bounded child at a time.

## State / Status Owners

- Story status and acceptance: this file.
- Remote request state and discussion: GitHub Issue #52.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## User-Facing Documentation

Update user or SDK documentation only when observable behavior or a public integration contract changes.
Do not present this Epic as shipped before every child reaches reviewed completion.

## Required Reads

- docs/backlog/active/PERM-004-workspace-trust-sandbox.md
- docs/backlog/active/PERM-005-logical-tool-sandbox-enforcement.md
- crates/talos-permission/
- crates/talos-agent/src/tool_execution.rs
- crates/talos-runtime/src/

## Acceptance For Behavior / Technical Work

- All five children are Complete with implementation and conformance evidence.
- CLI, TUI, headless, embedded runtime, RPC/MCP and extension paths use equivalent semantics.
- Duplicate permission engines/wrappers are removed or have an explicit removal gate.
- Architecture and SDK permission documentation are synchronized.

## Residual Destination

Epic completion requires all children; partial implementation remains owned by the corresponding child.

## 2026-08-22 I189 API Decision Checkpoint

- PR #351 merged as `20cfcce4`; I189/PERM-006-A is the sole Active child under its effective claim.
- ADR-065 is an in-scope public API/migration prerequisite for truthful rule/grant provenance. It
  changes no decision behavior or child ordering. PERM-006-B/C/D/E remain blocked/unclaimed.

## 2026-08-22 I189 Implementation Review Checkpoint

- ADR-065 was Accepted through PR #355 merge `9579df7a`. I189 implementation commit `6b577d6a`
  is in Review through PR #356 pending fresh exact-head CI, independent security/code review,
  merge-time CAS and owner-first closeout.
- PERM-006-B/C/D/E remain blocked/unclaimed. This Review candidate transfers no implementation
  authority to them, PERM-007 behavior or TOOL-024.

## 2026-08-22 PERM-006-A Completion Checkpoint

- PERM-006-A/I189 completed at implementation commit `6b577d6a`; PR #356 merged as `54241bdd`
  after exact-head CI `32511672926`, independent Agent-role permission/security/code review
  `5376591491` and merge-time CAS.
- PERM-006-B is now Ready/Unclaimed. C/D/E remain blocked in order, and no later child may start
  without its own runnable iteration and effective protected-scope claim.

## 2026-08-22 PERM-006-B / I219 Claim Proposal

- ADR-066 is Accepted through PR #358 merge `17e0b648`. PR #359 atomically proposes I219 and
  PERM-006-B as Claimed/Active for the bounded first-class grant/compiler/store delivery.
- The parent Epic remains Unclaimed. The open proposal is ineffective before merge; PERM-006-C/D/E,
  PERM-007 behavior, TOOL-024, release and publication remain blocked or separately unauthorized.

## 2026-08-22 PERM-006-B / I219 Claim Effectiveness

- PR #359 candidate `96816eb9` passed exact-head CI `32558607899` and independent Agent-role
  permission/security/API review `5378949775`, then merged as `781bb112`.
- B/I219 is the effective Active/Claimed permission child. C-E remain blocked/unclaimed; the parent
  Epic remains unclaimed. Parallel I213 retains only its separate Dashboard authority.

## 2026-08-22 PERM-006-B / I219 Local Convergence

- B/I219 reached Review/Claimed after its bounded implementation, official adapters, public API
  migration, documentation and local validation converged from claim merge `781bb112`.
- The first stable candidate has not been pushed and has no implementation PR or Completion Commit.
  Exact-head CI, Windows validation, independent protected-scope review and merge-time CAS remain.
  C-E stay blocked/unclaimed and I213 retains only its separate Dashboard authority.

## 2026-08-22 PERM-006-B / I219 Completion

- B/I219 completed through pre-existing implementation commits `56436027` and `d0c96048`.
  Corrected exact head `97028ac0` passed CI `32579790496` and independent Agent-role delta review
  `5381051760`, then PR #368 merged as `de79ad46` after merge-time CAS.
- The parent Epic remains In Progress/Unclaimed. C-E remain blocked/unclaimed and receive no
  implementation authority; I213 retains only its separate Dashboard lane.

## 2026-08-23 PERM-006-C Decision-Prerequisite Checkpoint

- A/I189 and B/I219 are Complete, so C is no longer blocked by those dependencies.
- C is still Blocked/Unclaimed because ADR-065 deferred the final hook transport/version contract
  and current cross-surface paths retain multiple permission authorities. Planned/unclaimed I220
  owns only the ADR-067 decision and current-path/migration matrix.
- I220 cannot activate alongside Active I213 until a new explicit non-overlap authorization is
  recorded; the earlier I219/I213 exception is not reusable. I221 will separately own C
  implementation after ADR-067 acceptance and its own effective claim.

## 2026-08-23 I220 Claim And Decision Work Activation

- I220 claim PR #370 merged to `main` as `5b8b1488`; its decision-only scope is now effective.
- I220 owns ADR-067 and the current-path/migration matrix. PERM-006-C remains Blocked/Unclaimed
  until ADR-067 is Accepted and I221 obtains a separate implementation claim.
- The explicit I213/I220 decision-only non-overlap is effective; no permission implementation,
  Runtime/MCP behavior, wrapper or hook change is authorized by I220.

## 2026-08-23 ADR-067 Accepted / I221 Handoff

- ADR-067 and the current-path/migration matrix are Accepted through PR #373 merge `5d2d2dcf`.
- I220 is Complete/Closed with decision commits `c21bb7f3` and `820586ea`.
- PERM-006-C is Ready/Unclaimed. I221 must establish a separate effective implementation claim;
  no Rust, Runtime, MCP, wrapper or hook implementation is authorized by this closeout.
