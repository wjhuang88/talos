# Iteration I086: Experience Polish And Retry Visibility

> Document status: Planned
> Published plan date: 2026-07-03
> Planned objective: Execute weeks 3-4 of the 2026-07-03 four-month hardening plan: close the
> most visible I084 follow-ups without expanding model/provider scope.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: retry attempts and transient thinking behavior are visible, bounded, documented,
> and covered by tests.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| H104 | PROVIDER-002/UX-001 | Complete with residual | I084 | Retry attempt status events flow through conversation/TUI events instead of tracing-only logs |
| H105 | MODEL-003/TUI-020 | Complete with residual | I084 | Thinking preview refinements and replay/compaction policy documented |
| H106 | Hardening plan | Planned | H104-H105 | Month-1 closeout with validation and residuals |

### Scope

- Surface provider retry attempts through the same event path that drives TUI status states.
- Preserve the existing timeout/retry policy from I084 unless a defect is proven.
- Keep thinking content transient unless explicitly persisted as structured reasoning blocks.
- Document replay, compaction, and visibility boundaries for thinking/reasoning states.

### Non-Goals

- No new provider protocol.
- No Gemini/OpenAI Responses implementation.
- No change to `/model` or `/connect`; those belong to I085.
- No permission, dashboard, or plugin behavior changes.

### Acceptance

- Given a retryable provider failure, when a retry begins, then the TUI can display a retrying
  state without relying on tracing logs.
- Given thinking preview text is streaming, when it is displayed, then transient content remains
  bounded and does not pollute finalized history.
- Given closeout completes, then I084 residuals are either closed or explicitly retained with owner
  docs updated.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test -p talos-provider`
- `cargo test -p talos-conversation`
- `cargo test -p talos-tui`
- `cargo test --workspace` at closeout
- `cargo clippy --workspace -- -D warnings` at closeout
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- `docs/backlog/active/PROVIDER-002-response-reliability-timeout-retry.md`
- `docs/backlog/active/UX-001-experience-reliability-program.md`
- `docs/backlog/active/MODEL-003-reasoning-thinking-support.md`
- `docs/tasks/2026-07-03-four-month-product-hardening-plan.md`
- `docs/BOARD.md` after owner docs

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-03 | Planning | Created as the I086 shell for the 2026-07-03 four-month hardening plan. |

## Verification Evidence

- Planned.

## Variance And Residuals

- Planned.

## Retrospective

- Pending.
