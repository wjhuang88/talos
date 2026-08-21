# Iteration I{NNN}: {Title}

> Document status: Planned
> Published plan date: YYYY-MM-DD
> Planned objective: {preserved objective}
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: {runnable and testable user- or operator-visible result}

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | None |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | YYYY-MM-DD |
| Handoff / Release Condition | None |

Before implementation, follow `docs/sop/AGENT-COLLABORATION.md`. One governance-only PR proposes
both `Claimed` and `Active`; both are ineffective until the finalized record reaches the target
branch. Implementation then converges locally before the first stable stage candidate is pushed.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `{ID}` | `{Epic or none}` | Ready | `{specific prerequisite}` | `{one verifiable result}` |

### Scope

- {authorized behavior or technical result}

### Non-Goals

- {explicit exclusion}

### Acceptance

- Given {precondition}
  When {actor action}
  Then {observable result}

### Planned Validation

- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- {binary/runtime scenario proving the MVP deliverable}

### Documentation To Update

- `README.md` or another user-facing owner affected by the deliverable
- Backlog parent/child status and `docs/BOARD.md`

### Risks And Rollback

- Risk: {failure mode}
- Rollback: {how the previous runnable state is preserved or restored}

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| YYYY-MM-DD | Atomic claim+activation | {dependency inventory, merge-time CAS result, and statement that both states become effective only on merge} |

## Verification Evidence

- {actual command}: {actual result}
- Runtime evidence: {binary command/test and observed result}

## Completion Evidence

- Completion Commit: `{already-existing implementation SHA}`
- Status-only documentation commits must not cite themselves. If implementation or maintainer
  acceptance is still pending, retain `Review`, `Partial`, or `Blocked` instead of `Complete`.

## Variance And Residuals

- {difference from baseline, deferred work, blocker, or none}

## Retrospective

- Outcome: {met, partial, blocked}
- Documentation: {updated paths or residual owner}
- Lessons: {EVOLUTION.md entry or none}
