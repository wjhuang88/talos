# Iteration I102: Provider Runtime Reliability Gate

> Document status: Planned
> Published plan date: 2026-07-07
> Planned objective: Execute Month 1 of the 2026-07-07 four-month developer operating plan by
> closing provider/tool-use stuck-processing risks before additional trial-facing work.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: malformed provider/tool-use streams become complete tool calls or visible
> terminal errors, with runtime evidence proving no silent processing tail.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| D100 | Developer operating plan | Planned | Current owner-doc inventory | Start-gate inventory and regression baseline are recorded. |
| D101 | PROVIDER-002/RUNTIME-002 | Planned | D100 | OpenAI-compatible SSE fixture matrix covers known malformed tool-call paths. |
| D102 | RUNTIME-002 | Planned | D101 | Agent turn-loop invariants reject malformed provider event sequences. |
| D103 | RUNTIME-002/TUI-028 | Planned | D102 | Runtime/TUI status distinguishes provider wait, tool wait, timeout, failed, cancelled. |
| D104 | Developer operating plan | Planned | D100-D103 | Month-1 closeout evidence and residuals are synchronized. |

### Scope

- Add deterministic provider fixture tests for OpenAI-compatible streaming edge cases.
- Add or verify agent invariants for malformed `ToolUse` sequences.
- Use existing conversation/TUI status plumbing for terminal failure visibility.
- Persist or document enough redacted evidence to debug malformed tool-use incidents.

### Non-Goals

- No provider schema redesign.
- No new credential flow.
- No permission-policy change.
- No background watchdog unless deterministic state transitions are proven insufficient and a
  separate review approves it.

### Acceptance

- Given a provider streams split or incomplete tool-call metadata, when parsing finishes, then Talos
  either emits complete `ToolCall` events or a terminal provider error.
- Given the agent receives `ToolUse` with zero or duplicate collected tool calls, when the turn loop
  processes it, then it fails explicitly instead of waiting forever.
- Given a provider/tool failure occurs after a turn starts, when the TUI receives status updates,
  then processing ends with a visible failed/timed-out/cancelled state.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo test -p talos-provider openai::tests::parse_sse_stream`
- `cargo test -p talos-agent tool_use`
- `cargo test -p talos-cli conversation_loop`
- `cargo test -p talos-tui processing`
- `cargo check --workspace`
- `cargo test --workspace` at closeout
- `cargo clippy --workspace -- -D warnings` at closeout
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- `docs/tasks/2026-07-07-four-month-developer-operating-plan.md`
- `docs/backlog/active/RUNTIME-002-turn-health-and-stuck-processing.md`
- `docs/backlog/active/PROVIDER-002-response-reliability-timeout-retry.md`
- `docs/BOARD.md` after owner docs

### Risks And Rollback

- Risk: fixture fixes accidentally reject valid text-based tool-call behavior.
- Rollback: keep native streaming invariants distinct from text-based tool-call parsing and retain
  regression tests for valid multi-tool turns.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-07 | Planning | Created as Month 1 shell for the four-month developer operating plan. |

## Verification Evidence

- Planned.

## Variance And Residuals

- Planned.

## Retrospective

- Pending.
