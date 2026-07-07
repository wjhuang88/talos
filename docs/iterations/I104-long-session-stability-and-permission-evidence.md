# Iteration I104: Long-Session Stability And Permission Evidence

> Document status: Planned
> Published plan date: 2026-07-07
> Planned objective: Execute Month 3 of the 2026-07-07 four-month developer operating plan by
> improving long-session ergonomics without weakening permission or validation boundaries.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: long development sessions have bounded approval noise evidence, readable tool
> output, and validation routing coverage without security-policy drift.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| D120 | PERM-003/PERM-002 | Planned | I103 closeout or explicit activation | Repeated-approval traces identify noise while preserving deny precedence. |
| D121 | VALIDATION-001 | Planned | D120 | Internal validation/project detection covers Rust and one non-Rust fixture. |
| D122 | TUI-015/TUI-019/TUI-025 | Planned | D120 | Long output and tool arguments render readably without changing model-visible payloads. |
| D123 | Developer operating plan | Planned | D120-D122 | Long-session stability evidence and residuals are synchronized. |

### Scope

- Collect and test permission-noise evidence before any policy change.
- Preserve deny precedence and write-tool gates.
- Exercise validation routing through existing internal service boundaries.
- Improve display-only tool-output ergonomics where owner docs already authorize it.

### Non-Goals

- No Guardian auto-approval.
- No exec DSL implementation.
- No sandbox/process-hardening change.
- No model-facing compression policy change.
- No new global scheduler, background watchdog, or event bus.

### Acceptance

- Given repeated low-risk development actions, when permission prompts occur, then traces identify
  the repeated decision scope without weakening write or deny behavior.
- Given a Rust and a non-Rust fixture project, when validation routing runs, then adapter selection
  is explicit and Cargo guidance is not injected for unrelated project types.
- Given long tool output or long arguments, when rendered in TUI, then display stays bounded while
  export/model payload semantics remain unchanged.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo test -p talos-permission`
- `cargo test -p talos-tools`
- `cargo test -p talos-tui tool_display`
- `cargo test -p talos-cli validation`
- `cargo check --workspace`
- `cargo test --workspace` at closeout
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- `docs/backlog/active/PERM-003-permission-experience-reference-study.md`
- `docs/backlog/active/VALIDATION-001-internal-validation-service.md`
- `docs/backlog/active/TUI-015-head-tail-truncation.md`
- `docs/backlog/active/TUI-019-tool-output-visual-hierarchy.md`
- `docs/backlog/active/TUI-025-tool-argument-line-fit-display.md`
- `docs/BOARD.md` after owner docs

### Risks And Rollback

- Risk: permission-noise fixes accidentally broaden allowed write execution.
- Rollback: treat policy changes as out of scope; collect traces and tests first, then escalate any
  required policy change to a separate senior-reviewed iteration.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-07 | Planning | Created as Month 3 shell for the four-month developer operating plan. |

## Verification Evidence

- Planned.

## Variance And Residuals

- Planned.

## Retrospective

- Pending.
