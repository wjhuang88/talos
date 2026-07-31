# PERM-006-A: Structured Permission Requests, Contexts, And Decision Reports

| Field | Value |
|---|---|
| Story ID | PERM-006-A |
| Type | Permission / Technical Story |
| Priority | P0 |
| Status | Refinement — additive API and compatibility shape require design |
| Source | [GitHub Issue #53](https://github.com/wjhuang88/talos/issues/53) |
| Selected Iteration | None |
| Depends On | Parent PERM-006; foundational dependency for PERM-006-B/C |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #53 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-01 |
| Handoff / Release Condition | Establish an effective claim and select an iteration before implementation. |

## Identity / Goal / Value

Add one structured permission request/context/report contract that explains per-facet outcomes and becomes the implementation source for existing evaluation entrypoints without changing shipped decisions.

## Scope

- Structured request, execution context, per-facet report, rule/grant provenance, and safe reasons.
- Compatibility projection to current `PermissionDecision`.
- Order-independent conservative aggregation tests.

## Exclusions

- No approval routing, wrapper removal, AlwaysApprove scope change, or typed-resource migration.

## Dependencies

Parent PERM-006; foundational dependency for PERM-006-B/C

## Decision Links And Constraints

- Any Deny denies; otherwise any Ask asks; otherwise Allow.
- Observer-facing reports exclude private projected fields and secrets.
- Current serialized configuration remains compatible.

## Uncertainty And Validation Path

Decide public type ownership and compatibility in an ADR or explicit pre-1.0 API note before Ready.

## State / Status Owners

- Story status and acceptance: this file.
- Remote request state and discussion: GitHub Issue #53.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## User-Facing Documentation

Update user or SDK documentation only when observable behavior or a public integration contract changes.
Do not present this Story as shipped while it remains Refinement.

## Required Reads

- docs/backlog/active/PERM-006-permission-pipeline-convergence.md
- crates/talos-core/src/tool.rs
- crates/talos-permission/src/lib.rs
- crates/talos-agent/src/tool_execution.rs

## Acceptance For Behavior / Technical Work

- One request-evaluation entrypoint is the source of truth for compatibility methods.
- Reports identify explicit rule, grant, workspace, mode, and default decisions.
- Current behavior matrix remains identical and invalid consequential resources fail closed.

## Residual Destination

Any behavior correction discovered here must become a separately reviewed security fix.
