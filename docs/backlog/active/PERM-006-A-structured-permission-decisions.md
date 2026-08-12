# PERM-006-A: Structured Permission Requests, Contexts, And Decision Reports

| Field | Value |
|---|---|
| Story ID | PERM-006-A |
| Type | Permission / Technical Story |
| Priority | P0 |
| Status | Planned — claim PR #197 pending independent security review |
| Source | [GitHub Issue #53](https://github.com/wjhuang88/talos/issues/53) |
| Selected Iteration | I189 (proposed; no authority before target-branch claim merge) |
| Depends On | Parent PERM-006; foundational dependency for PERM-006-B/C |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 implementation session 2026-08-11 |
| Work Slice | Implement only PERM-006-A / I189: add one structured permission request/context/per-facet decision-report evaluator, delegate existing permission entrypoints to it, preserve current Deny/Ask/Allow outcomes and compatibility-visible Deny messages, and add provenance, redaction, fail-closed and order-independence tests. No approval routing, wrapper removal, grant/store, AlwaysApprove, typed-resource, policy, sandbox, PERM-006-B/C/D/E, PERM-007, TOOL-024, ACP or release change. |
| Claimed At | 2026-08-11 |
| Source Issue | #53 |
| Governance Claim PR | #197 |
| Authorization Mode | Independent review |
| Authorization Evidence | Independent security review is mandatory on the finalized exact head before merge; no approval exists yet. This proposed claim remains ineffective until target-branch merge. |
| Implementation PR | Not started |
| Last Updated | 2026-08-11 |
| Handoff / Release Condition | Obtain independent exact-head security review, pass CI and merge-time CAS, and merge PR #197 before implementation. |

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
