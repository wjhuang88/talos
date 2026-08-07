# ARCH-034-R07: CLI Session Handler Decomposition

| Field | Value |
|---|---|
| Parent | ARCH-034 |
| Finding | ARCH-034-F23 |
| Status | Ready |
| Priority | P2 |
| Selected Iteration | I176 (Planned; claim PR pending) |
| Preserved behavior | Session lifecycle ordering, rollback, model activation, and UI diagnostics |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 architecture session 2026-08-07 |
| Work Slice | Move existing private provider/connect/model workflows and session delete/new/resume/fork workflows into private modules behind the current `session_handlers` facade; preserve handler paths/signatures, transition and UI channel ownership, CLI syntax, persistence, model identity, commit/rollback/publication ordering, exact diagnostics, and cleanup recovery behavior. |
| Claimed At | 2026-08-07 |
| Source Issue | None |
| Governance Claim PR | Pending |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | No independent reviewer is currently available; exact-head CI, both governance validators, merge-time CAS, and no blocking review feedback are required. |
| Implementation PR | Not started |
| Last Updated | 2026-08-07 |
| Handoff / Release Condition | Release if the split requires any handler path/signature, transition/UI ownership, CLI syntax, persistence, model identity, ordering, diagnostic, cleanup recovery, dependency, or behavior change. |

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
