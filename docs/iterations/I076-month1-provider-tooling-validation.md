# Iteration I076: Month 1 — Provider, Tooling, And Validation Loop

> Document status: Active (2026-07-01)
> Published plan date: 2026-07-01
> Planned objective: Execute weeks 1-4 of the 2026-07-01 four-month replan: provider usage
> accounting, status-bar correctness, write/edit output visibility, model-switch context markers,
> and autonomous validation-loop design.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: accurate OpenAI-compatible usage accounting plus a validated self-bootstrap
> validation-loop decision.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| T100 | Replan | Ready | Board/backlog inventory | Replan and I076-I079 published |
| T101 | PROVIDER-001 | Planned | Issue #12 | Streaming usage parsed |
| T102 | TUI-018 | Planned | TUI status bar | Million-unit context limit display |
| T103 | TUI-017 | Planned | T101 | Context usage percentage |
| T104 | TOOL-015 | Planned | TOOL-003 | Bounded write/edit result visibility |
| T105 | TUI-019 | Planned | T104 | Primary/secondary tool output style |
| T106 | SESSION-003 | Planned | SESSION-001 | Model-switch context marker |
| T107 | REL-002 | Review | T52 evidence | Validation-loop design |
| T108 | REL-002 | Review | T107 | First safe validation surface if approved |
| T109 | Replan | Planned | T100-T108 | Month-1 closeout |

### Scope

- Provider usage accounting and dependent status bar display.
- Tool result transparency for write/edit.
- Model-switch context marker persistence.
- Design and optional first implementation of autonomous validation loop.

### Non-Goals

- No plugin execution work.
- No direct exec tool.
- No release or publish action.

### Acceptance

- Given an OpenAI-compatible streaming usage-only chunk, when parsed, then token usage is retained.
- Given a known context limit, when status renders, then usage percentage and million-unit format are correct.
- Given write/edit completes, when displayed, then bounded verification output is visible.
- Given `/model` switches, when later context is built, then a persisted switch marker is visible.
- Given validation loop design clears, when a safe validation surface is implemented, then it cannot bypass permissions or hide failures.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- Targeted provider/TUI/tools/session tests
- `cargo test --workspace` at T109
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- `docs/tasks/2026-07-01-four-month-self-bootstrap-replan.md`
- Issue comments for #9-#14 as statuses change
- `docs/BOARD.md`

### Risks And Rollback

- Risk: provider-specific usage formats vary. Rollback: keep request change isolated and tolerate missing usage.
- Risk: validation loop becomes an execution bypass. Rollback: keep design-only and require explicit user-run validation.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-01 | Planning | Created as Month 1 shell for the replan. |
| 2026-07-01 | Activation | Activated for unattended execution. First packet is T100-T103: replan activation, OpenAI-compatible streaming usage, context million-unit display, and context usage percentage. |
| 2026-07-01 | Review | T100 complete; T101-T103 implemented and moved to Review after provider/TUI/check/clippy/governance validation. |
| 2026-07-01 | Review | T104-T105 implemented and moved to Review after file-tool and TUI result-rendering validation. |
| 2026-07-01 | Review | T106 implemented and moved to Review after model-switch marker, JSONL round-trip, and request-preview validation. |
| 2026-07-01 | Review | T107 design completed as `docs/proposals/autonomous-validation-loop.md`; T108 is constrained to a read-only validation plan/report surface. |
| 2026-07-01 | Review | T108 implemented `talos validate plan` as a read-only profile report with text/JSON output and no command execution. |

## Verification Evidence

- 2026-07-01: `cargo fmt --all -- --check` passed.
- 2026-07-01: `cargo test -p talos-provider` passed: 48 unit tests, 4 integration tests, 2 doc tests.
- 2026-07-01: `cargo test -p talos-tui status_bar` passed: 14 status-bar tests.
- 2026-07-01: `cargo test -p talos-tui` passed: 180 unit tests, 2 doc tests.
- 2026-07-01: `cargo check --workspace` passed.
- 2026-07-01: `cargo clippy -p talos-provider -p talos-tui -- -D warnings` passed.
- 2026-07-01: `scripts/validate_project_governance.sh .` passed with 0 warnings.
- 2026-07-01: `cargo test -p talos-tools file_tool_tests` passed: 22 tests.
- 2026-07-01: `cargo test -p talos-tui tool_result` passed: 4 tests.
- 2026-07-01: `cargo test -p talos-tools` passed: 200 unit tests, 15 document-boundary tests, 3 integration-hardening tests.
- 2026-07-01: `cargo test -p talos-tui` passed after T104-T105: 182 unit tests, 2 doc tests.
- 2026-07-01: `cargo clippy -p talos-tools -p talos-tui -- -D warnings` passed.
- 2026-07-01: `cargo test -p talos-cli model_switch_marker` passed: 3 tests.
- 2026-07-01: `cargo test -p talos-cli` passed: 95 unit tests and 8 integration tests.
- 2026-07-01: `cargo clippy -p talos-cli -- -D warnings` passed.
- 2026-07-01: `scripts/validate_project_governance.sh .` passed after T107 design sync with 0 warnings.
- 2026-07-01: `cargo test -p talos-cli validation` passed: 4 validation-plan tests.
- 2026-07-01: `cargo test -p talos-cli` passed after T108: 99 unit tests and 8 integration tests.
- 2026-07-01: `cargo clippy -p talos-cli -- -D warnings` passed after T108.
- 2026-07-01: `cargo run -p talos-cli -- validate plan --profile i076` printed the read-only I076 validation matrix without executing checks.
- 2026-07-01: `cargo check --workspace` passed after T108.
- 2026-07-01: `cargo run -p talos-cli -- validate plan --profile governance --json` printed structured read-only profile output.
- 2026-07-01: `scripts/validate_project_governance.sh .` passed after T108 with 0 warnings.

## Variance And Residuals

- T107 selected a Phase 1 read-only validation plan/report before any validation execution surface.
- T108 implemented Phase 1 only; validation execution remains out of scope until a separate permission-gated decision.
- T109 remains planned next for Month-1 closeout and full workspace validation.

## Retrospective

- Pending.
