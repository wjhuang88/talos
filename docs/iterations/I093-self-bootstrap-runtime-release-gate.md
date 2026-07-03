# Iteration I093: Self-Bootstrap, Runtime, And Release Gate

> Document status: Planned
> Published plan date: 2026-07-04
> Planned objective: update the self-bootstrap readiness boundary, runtime SDK residuals, and
> release posture without making a v1.0 claim prematurely.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a test-backed readiness report and at least one recorded Talos-on-Talos
> rehearsal or explicit non-qualification record.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `REL-002` | v1.0 self-bootstrap gate | Planned | Runtime/governance maturity | Readiness evidence updated honestly. |
| `RUNTIME-001` residual | Embeddable runtime | Complete pre-1.0 facade | ADR-024 | SDK residuals audited for self-bootstrap needs. |
| `GOV-003` residual | Built-in governance | Phase 1 complete | Governance docs | Governance capability gaps named. |
| `ARCH-030` watchlist | Architecture residuals | Tracking | Current source audit | Remaining roots classified for release risk. |

### Scope

- Record self-bootstrap evidence honestly.
- Keep Codex-primary sessions marked as non-qualifying for REL-002.
- Audit runtime/governance gaps needed for Talos-primary development.
- Produce final four-month closeout and next handoff.

### Non-Goals

- No v1.0 tag or claim.
- No publish or release action.
- No lowering gates to make self-bootstrap appear complete.

### Acceptance

- Given a Talos-on-Talos rehearsal is attempted,
  When evidence is recorded,
  Then the primary executor boundary is explicit and REL-002 qualification is honest.
- Given runtime/governance residuals exist,
  When readiness is reported,
  Then each residual has an owner or explicit deferral.
- Given final closeout runs,
  When docs are synchronized,
  Then Board, backlog, iterations, README status, and release posture agree.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `scripts/validate_project_governance.sh .`
- `git diff --check`

### Documentation To Update

- `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`
- `docs/reference/REL-002-READINESS-REPORT-*.md`
- `docs/backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md`
- `docs/backlog/active/GOV-003-builtin-project-governance.md` if touched
- `docs/BOARD.md`
- README only if public release posture changes

### Risks And Rollback

- Risk: overstating self-bootstrap readiness.
- Rollback: record non-qualifying evidence and keep pre-1.0 posture.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|

## Verification Evidence

- Pending.

## Variance And Residuals

- Pending.

## Retrospective

- Pending.
