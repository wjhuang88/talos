# ARCH-034-R03: Todo Module Decomposition

| Field | Value |
|---|---|
| Parent | ARCH-034 |
| Finding | ARCH-034-F06 |
| Status | Ready |
| Priority | P2 |
| Selected Iteration | Not selected |
| Preserved behavior | Todo schema, SQL, idempotency, dependency validation, tools, and permissions |

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
