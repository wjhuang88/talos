# Iteration I126: Schedule Inspection And Control

> Document status: Active
> Published plan date: 2026-07-13
> Planned objective: make active schedules inspectable, cancellable, and readable in narrow TUI views
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a user can list and cancel session schedules and verify the result on screen

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| SF120 | SCHED-001 | Ready | I125 Complete | bounded read-only schedule snapshots |
| SF121 | SCHED-001 | Ready | SF120 | Execute/Ask cancellation with safe races |
| SF122 | SCHED-001 | Ready | SF121 | consistent TUI/tool/transcript-safe display |
| SF123 | SCHED-001 | Ready | SF122 | fixture-provider and narrow-terminal proof |

### Scope

- stable task IDs, kind, interval/delay state, and bounded message preview;
- read-only list and mutating cancel with explicit permission classification;
- clear unknown, already-fired, cancelled, and shutting-down outcomes.

### Non-Goals

- editing/rescheduling an existing task, history database, remote/dashboard controls.

### Acceptance

- Listing never mutates tasks and never exposes unbounded/sensitive message content.
- Approved cancellation prevents future fires; Deny leaves the task unchanged.
- 40/60/80/120-column buffers retain task ID, state, and actionable result without panic.

### Planned Validation

- actor/tool permission and race tests;
- ratatui semantic buffer tests plus fixture-provider runtime flow;
- standard workspace/preflight/governance ladder.

### Documentation To Update

- README examples and privacy note; SCHED-001; index/backlog/Board.

### Risks And Rollback

- Risk: stale snapshots or cancellation races mislead the operator.
- Rollback: unregister list/cancel together; retain I124-I125 scheduling behavior.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-13 | Planning | Blocked on I125 Complete; no activation claimed. |
| 2026-07-14 | Activation | Gate 0 passed (workspace clean, rustc 1.97.0, governance 0 warnings, release preflight passed). I125 Complete. No other Active iteration. Security boundary unchanged — `list_scheduled_tasks` is Read/Allow, `cancel_scheduled_task` is Execute/Ask. I126 activated; SF120-SF123 ready. |

## Verification Evidence

- Pending SF120-SF123 implementation.

## Variance And Residuals

- Stress, recovery, and trial closeout are owned by I127.

## Retrospective

- Pending.
