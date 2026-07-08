# Iteration I108: Architecture-Sensitive Self-Bootstrap

> Document status: Planned
> Published plan date: 2026-07-08
> Planned objective: have Talos route and complete one bounded architecture-sensitive session as
> the primary executor without bypassing review gates.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: an architecture-sensitive audit, diagnostic, or bounded change with risk
> classification, evidence, review outcome, and REL-002 classification.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `SBT120` | 2026-07-08 self-bootstrap plan | Planned | I107 closeout | Architecture-sensitive owner selected and risk-classified. |
| `SBT121` | 2026-07-08 self-bootstrap plan | Planned | SBT120 | Talos performs the bounded architecture work. |
| `SBT122` | 2026-07-08 self-bootstrap plan | Planned | SBT121 | External review checks claims without taking over implementation. |
| `SBT123` | 2026-07-08 self-bootstrap plan | Planned | SBT122 | Session is classified against REL-002. |

### Scope

- Prefer `ARCH-032` Single Data Flow Audit unless a different bounded owner is explicitly selected.
- If code changes are selected, keep them narrow and inside existing architecture boundaries.
- Record review findings and residuals without hiding external intervention.

### Non-Goals

- No session-storage default migration unless a separate `SESSION-004` gate is explicitly activated.
- No permission policy, sandbox, credential, dependency, release, or broad orchestration change.
- No new ADR unless the selected work actually requires a decision record.

### Acceptance

- Given architecture-sensitive work is selected
  When Talos classifies the risk
  Then the correct owner docs and review gates are named before implementation.
- Given Talos completes the work
  When evidence is reviewed
  Then claims are backed by source, tests, runtime evidence, or explicit audit artifacts.
- Given external review changes the result
  When REL-002 is updated
  Then the qualification status reflects the real executor boundary.

### Planned Validation

- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- Architecture evidence or runtime scenario required by the selected owner.
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- Selected architecture/backlog owner doc.
- `docs/tasks/2026-07-08-four-month-talos-self-bootstrap-plan.md`
- `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`
- `docs/iterations/README.md`
- `docs/BOARD.md`

### Risks And Rollback

- Risk: Talos misclassifies high-risk work or drifts into a forbidden boundary.
- Rollback: stop the iteration, record the risk-routing defect, and require maintainer review before
  resuming.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-08 | Planning | Created as Month 3 of the 2026-07-08 Talos-primary self-bootstrap plan. |

## Verification Evidence

- Planned.

## Variance And Residuals

- Planned.

## Retrospective

- Planned.
