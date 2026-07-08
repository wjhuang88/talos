# Iteration I106: Self-Bootstrap Control Plane

> Document status: Planned
> Published plan date: 2026-07-08
> Planned objective: establish the Talos-primary execution contract, runtime smoke harness, and
> evidence classification needed before another REL-002 self-bootstrap attempt.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a repeatable Talos-primary rehearsal that records whether the session qualifies,
> partially qualifies, or does not qualify for REL-002.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `SBT100` | 2026-07-08 self-bootstrap plan | Planned | Clean worktree and current owner docs | Talos-primary execution contract and inventory recorded. |
| `SBT101` | 2026-07-08 self-bootstrap plan | Planned | SBT100 | Evidence schema distinguishes qualifying, partial, and non-qualifying sessions. |
| `SBT102` | 2026-07-08 self-bootstrap plan | Planned | SBT101 | Talos runtime smoke harness is repeatable. |
| `SBT103` | 2026-07-08 self-bootstrap plan | Planned | SBT102 | Talos performs a bounded governance rehearsal with rollback evidence. |
| `SBT104` | 2026-07-08 self-bootstrap plan | Planned | SBT103 | Month-1 result is classified in REL-002. |

### Scope

- Define the execution evidence that makes a Talos-primary session auditable.
- Verify Talos can run baseline validation and bounded owner-doc mutation paths.
- Record honest REL-002 qualification state after the rehearsal.

### Non-Goals

- No product feature implementation.
- No release, tag, publish, deployment, or external trial invitation.
- No permission, sandbox, credential, dependency, or session-storage default change.

### Acceptance

- Given a self-bootstrap session is assigned to Talos
  When the session records checkpoints and closeout evidence
  Then reviewers can tell whether Talos or an external runtime was the primary executor.
- Given Talos runs the smoke harness
  When provider, validation, governance, and resume paths are exercised
  Then each result is recorded with commands, outcomes, and residuals.
- Given external assistance occurs
  When REL-002 evidence is updated
  Then the affected session is classified as partial or non-qualifying instead of overclaimed.

### Planned Validation

- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `scripts/validate_project_governance.sh .`
- Real `talos` binary smoke commands recorded by SBT102/SBT103.

### Documentation To Update

- `docs/tasks/2026-07-08-four-month-talos-self-bootstrap-plan.md`
- `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`
- `docs/iterations/README.md`
- `docs/BOARD.md`

### Risks And Rollback

- Risk: the rehearsal proves Talos cannot yet act as primary executor.
- Rollback: close I106 as partial or blocked, preserve evidence, and do not activate I107 until the
  blocker has an owner.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-08 | Planning | Created as Month 1 of the 2026-07-08 Talos-primary self-bootstrap plan. |

## Verification Evidence

- Planned.

## Variance And Residuals

- Planned.

## Retrospective

- Planned.
