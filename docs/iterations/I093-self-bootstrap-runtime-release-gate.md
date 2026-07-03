# Iteration I093: Self-Bootstrap, Runtime, And Release Gate

> Document status: Complete
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
| 2026-07-04 | Activation | Activated after I092 completed bash-only cache-stability/export evidence and the autonomy permission matrix. Non-terminal inventory disposition: I085 remains Paused with MC107 real terminal `/connect` walkthrough residual; I086-I089 remain planned product-hardening shells. I093 starts with readiness/reporting only: REL-002 remains not ready for `v1.0.0`, RUNTIME-001 is complete only as a pre-1.0 facade, GOV-003 remains Phase 1 partial, and ARCH-030 remains a residual register. No tag, publish, release action, or v1.0 claim is authorized. |
| 2026-07-04 | A13 execution | Updated REL-002 readiness with `docs/reference/REL-002-READINESS-REPORT-2026-07-04.md`. The verdict remains not ready for `v1.0.0`; RUNTIME-001 still needs Talos-primary edit/validation/git evidence and stable SDK surface classification, GOV-003 still needs mutating governance/gate enforcement/risk classification, and ARCH-030 identifies session SQLite and Git tool roots as the highest self-bootstrap risks before continuity/git workflows expand. |
| 2026-07-04 | A14 execution | Recorded `docs/tasks/2026-07-04-self-bootstrap-rehearsal-i093-a14-nonqualification.md` as non-qualifying REL-002 evidence. Talos proved only `talos 0.2.2` CLI version availability; Codex remained primary for planning, editing, validation orchestration, docs sync, commit, and push. |
| 2026-07-04 | A15 closeout | Added `docs/reference/I090-I093-HIGH-RISK-CLOSEOUT-2026-07-04.md`, synchronized release posture, and closed I093 without a v1.0 claim. |

## Verification Evidence

- `scripts/validate_project_governance.sh .`: passed, 0 warnings.
- `git diff --check`: clean.
- `cargo run -p talos-cli -- --version`: passed; output included `talos 0.2.2`.
- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace -- -D warnings`: passed.
- `cargo test --workspace`: passed.

## Variance And Residuals

- No release, publish, tag, or v1.0 posture changed at activation.
- A15 is complete; final phase commit/push remains the only session-end action.

## Retrospective

- Activation preserves pre-1.0 honesty. The next work should name concrete gaps rather than
  convert Codex-primary evidence into REL-002 qualification.
- A13 confirmed the core posture: improved prerequisites are not qualifying self-bootstrap
  evidence. The next highest-value work is a tightly scoped Talos-primary rehearsal.
- A14 confirmed that a version-runnable CLI is not self-bootstrap evidence. REL-002 needs a
  Talos-primary edit/validation/governance loop before the next rehearsal can be qualifying.
- I093 closed with explicit No-go release posture. The next work should not reopen this iteration;
  changed self-bootstrap objectives need a new iteration ID.
