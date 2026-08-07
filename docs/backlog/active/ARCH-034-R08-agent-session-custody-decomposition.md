# ARCH-034-R08: Agent Session Custody Decomposition

> Document status: Complete

| Field | Value |
|---|---|
| Parent | ARCH-034 |
| Finding | ARCH-034-F24 |
| Status | Complete |
| Priority | P2 |
| Selected Iteration | I177 (Complete; Implementation PR #162) |
| Preserved behavior | Actor ordering, generation fences, receipts, recovery, pause/cancel, and archive |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 architecture session 2026-08-07 |
| Work Slice | Extract private durable custody/reconciliation, admission/rejection/receipt projection, pending-shutdown release, structured-turn finish, and pause/cancel helpers from `talos-agent/src/session.rs` while keeping `AppServerSession` as the sole actor and mutable state owner; preserve actor ordering, generation fences, receipts, recovery, pause/cancel, archive, diagnostics, event order, persistence protocol, and public API. |
| Claimed At | 2026-08-07 |
| Source Issue | None |
| Governance Claim PR | #161 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | No independent reviewer is currently available; exact-head CI, both governance validators, merge-time CAS, and no blocking review feedback are required. |
| Implementation PR | #162 |
| Last Updated | 2026-08-07 |
| Handoff / Release Condition | Closed after implementation PR #162 merged; any actor redesign, state ownership, persistence/event/diagnostic, public API, dependency, or behavior change requires a separate story and ADR where applicable. |

## Problem And Boundary

`talos-agent/src/session.rs` is a 1,386-line, high-change actor that combines run-loop dispatch,
durable custody/reconciliation, rejection/receipt projection, turn start/finish, and archiving.

## Scope

- Extract private custody/reconciliation helpers while keeping `AppServerSession` as the actor.
- Preserve the one-writer state machine, queue ordering, and generation checks.

## Exclusions

- No channel, persistence protocol, scheduler, retry, receipt, or public API change.

## Acceptance And Validation

- Extracted helpers cannot independently mutate actor state outside explicit parameters/results.
- Structured submission, recovery, pause/cancel, generation, and shutdown tests remain identical.
- Locked workspace, governance, and diff checks pass.

## Completion Evidence

- Completion Commit: `f505eea8`
- Implementation PR #162 squash-merged at `f505eea8`.
- Exact-head CI `31166594367` passed Unix/Windows workspace, governance, remote owner reconciliation, installer fixture, and rebuilt CLI smoke checks.
- Merge-time CAS confirmed base `9bc6012c`, head `97536674`, no blocking reviews/comments, and no overlapping claim or implementation PR.

## Rollback / Residual

Revert if actor ordering/custody equivalence cannot be proven. Actor redesign requires an ADR.
