# Iteration I178: Pending Submission Store Decomposition

> Document status: Planned
> Published plan date: 2026-08-07
> Planned objective: decompose private schema, query, row-mapping, retry, identity, and encoding responsibilities from `talos-session/src/pending_submission.rs` without changing persistence or submission behavior.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: `PendingSubmissionStore` remains the public transactional state-machine facade while private persistence helpers have separate source ownership, with schema/transition equivalence and downstream API probes passing.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 architecture session 2026-08-07 |
| Work Slice | Extract private schema/query/encoding and row-mapping helpers from `talos-session/src/pending_submission.rs` behind `PendingSubmissionStore`; preserve SQLite schema and SQL text, transaction modes, retry bounds, paths, identity/generation fencing, transition guards, recovery, cleanup, diagnostics, public methods, serialization, and dependency boundaries. |
| Claimed At | 2026-08-07 |
| Source Issue | None |
| Governance Claim PR | #164 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | No independent reviewer is currently available; exact-head CI, both governance validators, merge-time CAS, and no blocking review feedback are required. |
| Implementation PR | Not started |
| Last Updated | 2026-08-07 |
| Handoff / Release Condition | Release if schema, transaction, state, recovery, identity, or public API equivalence cannot be proven; any schema evolution requires a separate migration story. |

Before activation, follow `docs/sop/AGENT-COLLABORATION.md`. The claim is ineffective until the
finalized `Claimed` record is merged into `main`.

## Non-Terminal Inventory At Selection

| Item | State | Disposition |
|---|---|---|
| I159-I162 | Blocked | Unchanged; existing dependency and publication gates remain. |
| I164 | Paused | Historical superseded target; no activation. |
| Recovery PRs #120/#121 | Open archival evidence | Immutable; no implementation authority. |
| ARCH-034-R04 | Refinement | Native/panic/unsafe boundary remains excluded pending independent security review. |
| I177 / ARCH-034-R08 | Complete | Implementation merge `f505eea8`; closeout merge `c4fbd02d`; no overlap with pending-store ownership. |
| ARCH-034-R10/R11 | Ready / unclaimed | Retained for later independent claims; no overlap with this SQLite source split. |

No other Active, Review, or Planned iteration overlaps this work. R09 is selected only after I177
closure and the current-state audit identified an independent persistence source-ownership seam.

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| ARCH-034-R09 | ARCH-034 | Ready | I171 architecture register, I169 durable-custody behavior, I177 session custody boundary, and existing pending-submission tests | One behavior-preserving private persistence source split behind `PendingSubmissionStore`. |

### Scope

- Keep `PendingSubmissionStore` as the public transactional state-machine facade.
- Move existing private schema creation, SQL/query, row mapping, retry, identity, and encoding helpers into focused private modules.
- Preserve explicit transaction boundaries, SQL text, retry bounds, file paths, state transitions, runtime-generation fencing, crash recovery, and cleanup behavior.
- Add schema/row equivalence fixtures and compile-time downstream API probes.

### Non-Goals

- No schema migration, state-machine redesign, timeout or durability change.
- No public path/name, serialization, dependency, or error-diagnostic change.
- No changes to R04, R10, or R11.

### Acceptance

- Existing callers and every public `PendingSubmissionStore` method compile unchanged.
- Existing pending/restart/recovery/idempotency/session-cleanup tests pass unchanged.
- Before/after schema definitions, SQL behavior, transaction outcomes, row mapping, and recovery transitions are equivalent in deterministic fixtures.
- Private persistence helpers have separate source ownership and cannot own an independent state machine.

### Planned Validation

- `cargo test -p talos-session --locked --no-fail-fast`
- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked --no-fail-fast`
- `./scripts/release_preflight.sh`
- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`
- `git diff --check`
- Mechanical source-body/schema equivalence check and downstream API probes.
- Exact-head Unix/Windows CI and rebuilt CLI smoke.

### Documentation To Update

- Synchronize ARCH-034-R09, I178, `docs/iterations/README.md`, `docs/backlog/PRODUCT-BACKLOG.md`, `docs/BOARD.md`, and the governance manifest.
- No user-facing behavior documentation change is expected because this is a private source decomposition.

### Risks And Rollback

- Risk: source movement changes SQL ordering, transaction scope, retry behavior, row decoding, identity fencing, or recovery transitions despite compiling.
- Rollback: revert the private module move if equivalence cannot be shown; schema evolution or behavior correction requires a separate story and migration/ADR review.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-07 | Planning | I178 selected after inventorying non-terminal work, confirming I177/R08 closure, and finding no overlapping effective claim or implementation PR. |

## Verification Evidence

- Claim-only preflight and current scale assessment are recorded in the session log; implementation evidence is intentionally absent until the claim becomes effective.

## Completion Evidence

- Completion Commit: not assigned; retain Planned until claim and implementation evidence exist.

## Variance And Residuals

- R04 remains Refinement pending independent security review.
- R10 and R11 remain separately owned and independently claimable after I178 closes.

## Retrospective

- Outcome: pending.
- Documentation: pending implementation result; no user-facing behavior documentation change is planned.
- Lessons: none recorded.
