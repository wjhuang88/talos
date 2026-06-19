# Iteration I{NNN}: {Title}

> Document status: Planned
> Published plan date: YYYY-MM-DD
> Planned objective: {preserved objective}
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: {runnable and testable user- or operator-visible result}

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
| YYYY-MM-DD | Activation | {dependency inventory and activation decision} |

## Verification Evidence

- {actual command}: {actual result}
- Runtime evidence: {binary command/test and observed result}

## Variance And Residuals

- {difference from baseline, deferred work, blocker, or none}

## Retrospective

- Outcome: {met, partial, blocked}
- Documentation: {updated paths or residual owner}
- Lessons: {EVOLUTION.md entry or none}
