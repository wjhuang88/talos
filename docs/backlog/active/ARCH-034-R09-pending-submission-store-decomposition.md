# ARCH-034-R09: Pending Submission Store Decomposition

> Document status: Complete

| Field | Value |
|---|---|
| Parent | ARCH-034 |
| Finding | ARCH-034-F25 |
| Status | Complete |
| Priority | P2 |
| Selected Iteration | I178 (Complete; Implementation PR #165) |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 architecture session 2026-08-07 |
| Work Slice | Extract private schema/query/encoding and row-mapping helpers from `talos-session/src/pending_submission.rs` behind `PendingSubmissionStore`; preserve SQLite schema and SQL text, transaction modes, retry bounds, paths, identity/generation fencing, transition guards, recovery, cleanup, diagnostics, public methods, serialization, and dependency boundaries. |
| Claimed At | 2026-08-07 |
| Source Issue | None |
| Governance Claim PR | #164 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | No independent reviewer is currently available; exact-head CI, both governance validators, merge-time CAS, and no blocking review feedback are required. |
| Implementation PR | #165 |
| Last Updated | 2026-08-07 |
| Handoff / Release Condition | Closed after implementation PR #165 merged; any schema, transaction, state, recovery, identity, public API, dependency, or behavior change requires a separate story and migration/ADR review where applicable. |
| Preserved behavior | SQLite schema, transactions, transition guards, recovery, and identity fencing |

## Problem And Boundary

`talos-session/src/pending_submission.rs` combines the public transactional state machine with
schema creation, SQL row mapping, retry, identity, and encoding helpers in 1,097 production lines.

## Scope

- Extract private schema/query/encoding modules behind `PendingSubmissionStore`.
- Preserve SQL text, transaction modes, retry bounds, paths, and public methods.

## Exclusions

- No migration, schema version, state, timeout, durability, public API, or dependency change.

## Acceptance And Validation

- Before/after database schema and transition outcomes are byte/row equivalent in fixtures.
- Pending/restart/recovery/idempotency and session cleanup tests pass unchanged.
- Locked workspace, governance, and diff checks pass.

## Completion Evidence

- Completion Commit: `f92634803560dc50e0b15ca8d7d511e9928c983f`
- Implementation PR #165 squash-merged at `f92634803560dc50e0b15ca8d7d511e9928c983f` from source implementation `c662a7e6` and accepted exact Head `1fc4b761d38f6b2c35722da869f28fdd93a7b519`.
- Exact-head CI `31180591881` passed Unix/Windows workspace, governance, remote owner reconciliation, installer fixture, and rebuilt CLI smoke checks.
- Merge-time CAS confirmed base `7b87902dd215e03ac8a3f331b535d8af286c96d2`, head `1fc4b761d38f6b2c35722da869f28fdd93a7b519`, no blocking reviews/comments, and no overlapping claim or implementation PR.

## Rollback / Residual

Revert if SQL/state equivalence is not exact. Schema evolution requires a separate migration story.
