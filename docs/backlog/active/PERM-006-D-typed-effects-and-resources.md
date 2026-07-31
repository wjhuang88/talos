# PERM-006-D: Typed Permission Effects And Resources

| Field | Value |
|---|---|
| Story ID | PERM-006-D |
| Type | Permission / Public API Story |
| Priority | P0 |
| Status | Blocked — PERM-006-C pipeline must complete first |
| Source | [GitHub Issue #56](https://github.com/wjhuang88/talos/issues/56) |
| Selected Iteration | None |
| Depends On | Blocked by PERM-006-C; feeds PERM-006-E |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #56 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-01 |
| Handoff / Release Condition | Establish an effective claim and select an iteration before implementation. |

## Identity / Goal / Value

Replace in-tree tool-name and JSON-field inference with explicit typed effects and normalized resources declared by each concrete invocation.

## Scope

- ADR-backed effect/resource model and compatibility adapter.
- Migration inventory for every built-in tool.
- Typed path, command, endpoint, and remote-object matching.
- Conservative extension/MCP/plugin fallback.

## Exclusions

- No workspace-trust broadening, persistent grants, new tools, or transport-security replacement.

## Dependencies

Blocked by PERM-006-C; feeds PERM-006-E

## Decision Links And Constraints

- Effect and resource are separate dimensions.
- Normalization matches exact execution authorization.
- Same textual value in different resource variants cannot cross-match.
- Unknown consequential metadata fails closed.

## Uncertainty And Validation Path

Accept an ADR and migration plan before changing public pre-1.0 types or configuration.

## State / Status Owners

- Story status and acceptance: this file.
- Remote request state and discussion: GitHub Issue #56.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## User-Facing Documentation

Update user or SDK documentation only when observable behavior or a public integration contract changes.
Do not present this Story as shipped while it remains Blocked.

## Required Reads

- docs/backlog/active/PERM-006-C-agent-owned-permission-pipeline.md
- crates/talos-core/src/tool.rs
- crates/talos-permission/src/resource.rs
- all in-tree AgentTool permission descriptors

## Acceptance For Behavior / Technical Work

- All built-in tools emit complete typed facets without generic field inference.
- Remote read and external mutation are distinguishable.
- Path/command/endpoint/remote variants and normalization pass the security matrix.
- Legacy external callers have a documented conservative compatibility path.

## Residual Destination

Future scope/effect additions use additive variants or a new ADR, not implicit string inference.
