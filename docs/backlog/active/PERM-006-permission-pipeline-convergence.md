# PERM-006: Permission Decision, Approval, Grant, And Execution Convergence

| Field | Value |
|---|---|
| Story ID | PERM-006 |
| Type | Architecture / Permission Epic |
| Priority | P0 |
| Status | In Progress — activation PR #351 proposes PERM-006-A/I189 as the only Active child; B-E remain blocked/unclaimed |
| Source | [GitHub Issue #52](https://github.com/wjhuang88/talos/issues/52) |
| Selected Iteration | I189 for PERM-006-A only; activation is ineffective until PR #351 reaches `main` |
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
Do not present this Story as shipped while it remains Refinement.

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
