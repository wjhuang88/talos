# Iteration I107: Talos-Primary Feature Polish

> Document status: Planned
> Published plan date: 2026-07-08
> Planned objective: have Talos complete one low-risk user-facing feature or polish change as the
> primary development executor.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a tested user-visible improvement with Talos-primary implementation, validation,
> documentation, and REL-002 classification.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `SBT110` | 2026-07-08 self-bootstrap plan | Planned | I106 closeout | Low-risk feature/polish owner selected and activated. |
| `SBT111` | 2026-07-08 self-bootstrap plan | Planned | SBT110 | Talos implements the selected change through permission-gated tools. |
| `SBT112` | 2026-07-08 self-bootstrap plan | Planned | SBT111 | User docs, backlog, iteration, and board are synchronized owner-first. |
| `SBT113` | 2026-07-08 self-bootstrap plan | Planned | SBT112 | Session is classified against REL-002. |

### Scope

- Select one existing low-risk story, preferring `TOOL-020` or the I085 MC107 walkthrough residual
  if still relevant.
- Require real runtime evidence for any behavior change.
- Preserve permission, sandbox, credential, dependency, and storage boundaries.

### Non-Goals

- No new feature outside an existing owner doc.
- No broad refactor, dependency addition, permission-default change, or release action.
- No claim that external implementation qualifies for REL-002.

### Acceptance

- Given a low-risk user-facing story is selected
  When Talos implements and validates it as primary executor
  Then the owner docs identify the changed behavior and runtime evidence.
- Given external review finds issues
  When remediation is needed
  Then Talos fixes them or the evidence is downgraded honestly.
- Given the session closes
  When REL-002 is updated
  Then it records whether this is a qualifying, partial, or non-qualifying feature/polish session.

### Planned Validation

- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- Real binary/runtime scenario for the selected story.
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- Selected backlog owner doc.
- `docs/tasks/2026-07-08-four-month-talos-self-bootstrap-plan.md`
- `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`
- `docs/iterations/README.md`
- `docs/BOARD.md`

### Risks And Rollback

- Risk: the selected story turns out to require high-risk boundaries or external implementation.
- Rollback: defer that story, record why, and select a lower-risk owner with the same monthly goal.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-08 | Planning | Created as Month 2 of the 2026-07-08 Talos-primary self-bootstrap plan. |

## Verification Evidence

- Planned.

## Variance And Residuals

- Planned.

## Retrospective

- Planned.
