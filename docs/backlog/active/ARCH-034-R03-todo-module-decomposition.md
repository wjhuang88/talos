# ARCH-034-R03: Todo Module Decomposition

**Status**: Complete

| Field | Value |
|---|---|
| Parent | ARCH-034 |
| Finding | ARCH-034-F06 |
| Status | Complete |
| Priority | P2 |
| Selected Iteration | I173 (Complete; PR #149 merged as `506311dcb6db18a2cbe1602c8dae69f780f4416d`) |
| Preserved behavior | Todo schema, SQL, idempotency, dependency validation, tools, and permissions |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 architecture session 2026-08-07 |
| Work Slice | Private model, repository, formatting, and nine-tool-adapter decomposition behind the current `todo` facade with exact public-path, SQL, schema, permission, contribution-order, and output preservation. |
| Claimed At | 2026-08-07 |
| Source Issue | None |
| Governance Claim PR | #148 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Exact-head CI `31143057387`, both governance validators, merge-time CAS, and no blocking review feedback passed before merge. |
| Implementation PR | #149 (merged as `506311dcb6db18a2cbe1602c8dae69f780f4416d`) |
| Last Updated | 2026-08-07 |
| Handoff / Release Condition | Release if the split requires any schema, SQL, permission, public-path, or output change. |

## Problem And Boundary

`talos-session/src/todo.rs` combines domain types, SQLite repository operations, dependency graph
checks, display formatting, and nine `AgentTool` adapters in 1,653 production lines. All belong to
the session domain, but they have independent reasons to change.

## Scope

- Split private model, repository, formatting, and tool-adapter modules.
- Preserve the existing `talos_session` re-exports and all public type/function paths.
- Keep one shared Todo permission facet and the existing contribution source.

## Exclusions

- No schema migration, query change, tool rename, permission change, new abstraction, or crate split.

## Acceptance And Validation

- Repository code does not import tool adapter types; adapters consume the repository facade.
- Public API compile paths and serialized schemas remain identical.
- Todo repository, batch/idempotency, dependency, contribution, and permission tests pass.
- Locked workspace and governance validation pass.

## Rollback / Residual

Revert the module move if API or SQL equivalence is not exact. New Todo behavior requires a product
story.

## Completion Evidence

- Completion Commit: `e4818e34c1e047c41d41abc1f7859c7984008e83`
- Exact-head CI: `31143057387` passed Unix and Windows workspace validation, governance checks, and rebuilt CLI smoke.
- Implementation PR: #149 merged as `506311dcb6db18a2cbe1602c8dae69f780f4416d`.
