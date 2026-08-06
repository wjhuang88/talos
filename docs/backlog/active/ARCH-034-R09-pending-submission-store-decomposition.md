# ARCH-034-R09: Pending Submission Store Decomposition

| Field | Value |
|---|---|
| Parent | ARCH-034 |
| Finding | ARCH-034-F25 |
| Status | Ready |
| Priority | P2 |
| Selected Iteration | Not selected |
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

## Rollback / Residual

Revert if SQL/state equivalence is not exact. Schema evolution requires a separate migration story.
