# ARCH-034-R07: CLI Session Handler Decomposition

| Field | Value |
|---|---|
| Parent | ARCH-034 |
| Finding | ARCH-034-F23 |
| Status | Ready |
| Priority | P2 |
| Selected Iteration | Not selected |
| Preserved behavior | Session lifecycle ordering, rollback, model activation, and UI diagnostics |

## Problem And Boundary

`talos-cli/src/session_handlers.rs` combines delete, provider connection, model switching,
new/resume/fork, rollback, and cleanup recovery workflows in 1,329 production lines.

## Scope

- Split private provider/model and session-lifecycle workflow modules.
- Retain current handler signatures and shared transition/UI channel ownership.

## Exclusions

- No CLI syntax, persistence, model identity, publication order, diagnostic, or API change.

## Acceptance And Validation

- Each workflow retains current commit/rollback ordering and failure diagnostics.
- Model activation, session fork/resume/delete, parser, and cleanup recovery tests pass unchanged.
- Locked workspace, CLI e2e, governance, and diff checks pass.

## Rollback / Residual

Revert any workflow extraction whose ordering equivalence is not proven.
