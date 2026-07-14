# Iteration I125: Recurring Scheduled Follow-Ups

> Document status: Active
> Published plan date: 2026-07-13
> Planned objective: add bounded recurring follow-ups without catch-up bursts or permission reuse
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a user can request a session-scoped recurring follow-up and observe bounded fires

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| SF110 | SCHED-001 | Ready | I124 Complete | bounded recurring actor behavior |
| SF111 | SCHED-001 | Ready | SF110 | Execute/Ask `schedule` tool |
| SF112 | SCHED-001 | Ready | SF111 | missed-tick/race/permission regressions |
| SF113 | SCHED-001 | Ready | SF112 | docs and configured-provider walkthrough |

### Scope

- interval-in-seconds recurrence with documented bounds and `MissedTickBehavior::Delay`;
- no immediate surprise tick unless the user-visible contract explicitly says so;
- each fired turn follows the existing queue and permission pipeline.

### Non-Goals

- persistence, cron/calendar syntax, direct tools, background daemon, permission redesign.

### Acceptance

- A valid approved recurrence fires at the documented cadence without catch-up bursts.
- Cancellation/shutdown races produce no duplicate post-cancel turn.
- Approval of recurrence never becomes approval for later write/network/execute tools.

### Planned Validation

- paused-time cadence and missed-tick tests;
- fixture-provider recurring-turn test;
- standard workspace/preflight/governance ladder.

### Documentation To Update

- README scheduling behavior/limits; SCHED-001; index/backlog/Board.

### Risks And Rollback

- Risk: interval first-tick semantics or queue pressure creates unexpected repeated work.
- Rollback: keep `schedule` unregistered while retaining I124 one-shot behavior.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-13 | Planning | Blocked on I124 Complete; no activation claimed. |
| 2026-07-14 | Dependency update | I124 reached Complete. I125 is unblocked and remains Planned; no activation or implementation is claimed. Run the required activation inventory and gates before starting work. |
| 2026-07-14 | Activation | Gate 0 passed (workspace clean, rustc 1.97.0, cargo metadata OK, governance 0 warnings, release preflight passed). No other Active iteration exists (I124 Complete). Security boundary unchanged from I124 — `schedule` tool is `ToolNature::Execute` (default Ask), same permission architecture. I125 activated; SF110-SF113 ready for implementation. |

## Verification Evidence

- Pending SF110-SF113 implementation.

## Variance And Residuals

- List/cancel UX is owned by I126.

## Retrospective

- Pending.
