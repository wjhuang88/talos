# PERM-006-B: Centralized Grant Compiler And Scoped Grant Store

| Field | Value |
|---|---|
| Story ID | PERM-006-B |
| Type | Permission / Technical Story |
| Priority | P0 |
| Status | Ready / Unclaimed — PERM-006-A completed through I189 |
| Source | [GitHub Issue #54](https://github.com/wjhuang88/talos/issues/54) |
| Selected Iteration | None |
| Depends On | PERM-006-A Complete; preserves PERM-004/PERM-005 and SEC-001 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #54 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-22 |
| Handoff / Release Condition | Select a runnable/testable iteration and establish an effective protected-scope claim before implementation. Readiness does not authorize a branch or code change. |

## Identity / Goal / Value

Separate configured policy from user-approved runtime grants and make one compiler define ApproveOnce and session-reusable scope across every surface.

## Scope

- Explicit grant identity, scope, provenance, compiler, in-memory session store, matching, and safe descriptions.
- Shared path, command, network, remote, and multi-facet grant compilation.
- Cross-surface structural-equivalence tests.

## Exclusions

- No persistent grants, task/scheduler inheritance, trusted-workspace broadening, or agent pipeline migration.

## Dependencies

PERM-006-A Complete; preserves PERM-004/PERM-005 and SEC-001

## Decision Links And Constraints

- [ADR-066](../../decisions/066-first-class-scoped-permission-grants.md) is the required
  first-class grant, session-lifetime, precedence and public API decision. Its Accepted status
  clears the decision gate only and authorizes no implementation.
- All effective policy Deny, including Configured and SDK/Runtime Explicit rules, plus hard
  boundaries override every grant.
- External paths never gain directory-wide reusable grants.
- Bash templates reuse the audited classifier; no second parser.

## Uncertainty And Validation Path

PERM-006-A now supplies authoritative requests and provenance. Refine the compiler/store lifetime,
scope-equivalence and approval-preview contract in the selected iteration before implementation.

## State / Status Owners

- Story status and acceptance: this file.
- Remote request state and discussion: GitHub Issue #54.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## User-Facing Documentation

Update user or SDK documentation only when observable behavior or a public integration contract changes.
Do not present this Story as shipped while it remains Ready/Unclaimed.

## Required Reads

- docs/backlog/active/PERM-006-A-structured-permission-decisions.md
- docs/backlog/active/PERM-004-workspace-trust-sandbox.md
- crates/talos-permission/
- crates/talos-cli/src/approval.rs
- crates/talos-runtime/src/

## Acceptance For Behavior / Technical Work

- Equivalent requests compile equivalent grants across CLI/TUI/runtime.
- Policy and grants have separate storage and provenance.
- Session grants expire with the session and never override Deny.
- Approval preview exactly describes the installed scope.

## Residual Destination

Persistent or task-scoped grants require a separate owner and ADR.

## 2026-08-22 Dependency Clearance Checkpoint

PERM-006-A/I189 is Complete/Closed at Completion Commit `6b577d6a`; implementation PR #356
merged as `54241bdd` after exact-head CI and independent permission/security/code review. This
clears PERM-006-B's dependency only. PERM-006-B is Ready/Unclaimed with Selected Iteration None;
no implementation branch or code change is authorized before a separate runnable iteration and
effective protected-scope Collaboration Claim reach `main`.

## 2026-08-22 ADR-066 Decision Checkpoint

Read-only assessment found that PERM-006-B changes security behavior and published SDK contracts:
it separates grants from ADR-065's compatibility rule vector, converges CLI parent-scope and
Runtime exact-scope behavior, and replaces legacy runtime-rule/lifetime APIs. ADR-066 records the
required decision, compatibility and rollback boundary. It remains Proposed until exact-head
independent permission/security/API review, CI, CAS and target-branch merge; this checkpoint does
not select an iteration, establish a claim or authorize implementation.

## 2026-08-22 ADR-066 Acceptance Checkpoint

ADR-066 decision content commit `17088d88` was independently approved at exact Proposed head
`33199bd8` in PR #358 comment `5376959300`, with CI run `32541156457`; repository-owner acceptance
is recorded in comment `5378407820`. The ADR is Accepted, clearing only this Story's decision gate.
PERM-006-B remains Ready/Unclaimed with Selected Iteration None until a separate runnable/testable
iteration and effective protected-scope Collaboration Claim reach `main`.
