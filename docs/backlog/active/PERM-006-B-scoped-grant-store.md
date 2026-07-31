# PERM-006-B: Centralized Grant Compiler And Scoped Grant Store

| Field | Value |
|---|---|
| Story ID | PERM-006-B |
| Type | Permission / Technical Story |
| Priority | P0 |
| Status | Blocked — PERM-006-A must be accepted and complete |
| Source | [GitHub Issue #54](https://github.com/wjhuang88/talos/issues/54) |
| Selected Iteration | None |
| Depends On | Blocked by PERM-006-A; preserves PERM-004/PERM-005 and SEC-001 |

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
| Last Updated | 2026-08-01 |
| Handoff / Release Condition | Establish an effective claim and select an iteration before implementation. |

## Identity / Goal / Value

Separate configured policy from user-approved runtime grants and make one compiler define ApproveOnce and session-reusable scope across every surface.

## Scope

- Explicit grant identity, scope, provenance, compiler, in-memory session store, matching, and safe descriptions.
- Shared path, command, network, remote, and multi-facet grant compilation.
- Cross-surface structural-equivalence tests.

## Exclusions

- No persistent grants, task/scheduler inheritance, trusted-workspace broadening, or agent pipeline migration.

## Dependencies

Blocked by PERM-006-A; preserves PERM-004/PERM-005 and SEC-001

## Decision Links And Constraints

- Configured Deny and hard boundaries override every grant.
- External paths never gain directory-wide reusable grants.
- Bash templates reuse the audited classifier; no second parser.

## Uncertainty And Validation Path

Activate only after PERM-006-A supplies authoritative requests and provenance.

## State / Status Owners

- Story status and acceptance: this file.
- Remote request state and discussion: GitHub Issue #54.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## User-Facing Documentation

Update user or SDK documentation only when observable behavior or a public integration contract changes.
Do not present this Story as shipped while it remains Blocked.

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
