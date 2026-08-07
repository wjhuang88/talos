# ARCH-034-R07: CLI Session Handler Decomposition

> Document status: Complete

| Field | Value |
|---|---|
| Parent | ARCH-034 |
| Finding | ARCH-034-F23 |
| Status | Complete |
| Priority | P2 |
| Selected Iteration | I176 (Complete; Implementation PR #159) |
| Preserved behavior | Session lifecycle ordering, rollback, model activation, and UI diagnostics |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 architecture session 2026-08-07 |
| Work Slice | Move existing private provider/connect/model workflows and session delete/new/resume/fork workflows into private modules behind the current `session_handlers` facade; preserve handler paths/signatures, transition and UI channel ownership, CLI syntax, persistence, model identity, commit/rollback/publication ordering, exact diagnostics, and cleanup recovery behavior. |
| Claimed At | 2026-08-07 |
| Source Issue | None |
| Governance Claim PR | #158 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | No independent reviewer is currently available; exact-head CI, both governance validators, merge-time CAS, and no blocking review feedback are required. |
| Implementation PR | #159 |
| Last Updated | 2026-08-07 |
| Handoff / Release Condition | Closed after implementation PR #159 merged; any future handler path/signature, transition/UI ownership, CLI syntax, persistence, model identity, ordering, diagnostic, cleanup recovery, dependency, or behavior change requires a separate story. |

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

## Completion Evidence

- Completion Commit: `1de3243d`
- Implementation PR #159 merged at `37c557271b906664022476bd2775c5cd77f2b8ea`.
- Exact-head CI `31160309818` passed Unix/Windows workspace, governance, remote owner reconciliation, installer fixture, and rebuilt CLI smoke checks.
- Merge-time CAS confirmed base `97c252a1493e66cfb8ccbe5ff64d0643a92255d7`, head `643ecbc59869fe1d76159254a9835938ff63ae36`, no blocking reviews/comments, and no overlapping claim or implementation PR.

## Rollback / Residual

Revert any workflow extraction whose ordering equivalence is not proven. R04 remains Refinement
pending independent security review; R08-R11 remain Ready and separately claimable.
