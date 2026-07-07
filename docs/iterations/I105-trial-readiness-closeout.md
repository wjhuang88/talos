# Iteration I105: Trial Readiness Closeout

> Document status: Planned
> Published plan date: 2026-07-07
> Planned objective: Execute Month 4 of the 2026-07-07 four-month developer operating plan by
> producing trial documentation, smoke evidence, REL-002 classification, and a maintainer go/no-go
> package.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a market-trial readiness report with repeatable smoke evidence, known limits,
> rollback instructions, and honest release/self-bootstrap status.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| D130 | Developer operating plan | Planned | I104 closeout or explicit activation | Trial docs cover install, first run, providers, permissions, local data, and bug reports. |
| D131 | Developer operating plan | Planned | D130 | Smoke checklist exercises first run, `/connect`, `/model`, tool use, provider failure, resume, exit. |
| D132 | REL-002 | Planned | D131 | One Talos-primary attempt is recorded as qualifying or non-qualifying evidence. |
| D133 | Developer operating plan | Planned | D130-D132 | Final readiness report gives go/no-go, residual risks, rollback, and next owners. |

### Scope

- Create trial-facing docs and smoke checklist.
- Run and record repeatable smoke evidence.
- Update REL-002 evidence honestly without claiming readiness if criteria are not met.
- Produce final closeout and recovery handoff.

### Non-Goals

- No external trial invitation.
- No `v1.0` claim.
- No release tag, GitHub Release, crates.io publish, or installer signing.
- No release gate lowering.

### Acceptance

- Given a trial candidate build, when smoke validation runs, then the same checklist proves first
  run, provider setup, model selection, tool use, provider failure visibility, session resume, and
  exit summary.
- Given a self-bootstrap attempt, when it is evaluated against REL-002, then the owner doc records
  whether it qualifies and why.
- Given the month closes, then the maintainer has a go/no-go report with residual risks and rollback
  instructions.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `scripts/validate_project_governance.sh .`
- `git diff --check`
- Recorded smoke checklist evidence

### Documentation To Update

- `README.md`
- `README.zh-CN.md` if user-facing setup text changes
- `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`
- `docs/tasks/2026-07-07-four-month-developer-operating-plan.md`
- `docs/BOARD.md` after owner docs
- A final readiness report under `docs/reference/` if D133 completes

### Risks And Rollback

- Risk: smoke passes are mistaken for release qualification.
- Rollback: separate trial-readiness evidence from release authorization and keep release actions
  explicitly out of scope.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-07 | Planning | Created as Month 4 shell for the four-month developer operating plan. |

## Verification Evidence

- Planned.

## Variance And Residuals

- Planned.

## Retrospective

- Pending.
