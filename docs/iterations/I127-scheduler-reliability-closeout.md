# Iteration I127: Scheduler Reliability Closeout

> Document status: Planned
> Published plan date: 2026-07-13
> Planned objective: prove scheduler cleanup, recovery, and operator documentation under failure
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: another operator can replay a clean-HOME scheduled-follow-up trial safely

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| SF130 | SCHED-001 | Ready | I126 Complete | shutdown/channel/backpressure hardening |
| SF131 | SCHED-001 | Ready | SF130 | deterministic lifecycle stress suite |
| SF132 | SCHED-001 | Ready | SF131 | clean-HOME trial and recovery packet |
| SF133 | SCHED-001 | Ready | SF132 | second-operator evidence and honest closeout |

### Scope

- bounded failure behavior for full/closed queues, cancellation, actor exit, and completed tasks;
- replayable fixture-provider trial requiring no real credential;
- residual and unsupported-context documentation.

### Non-Goals

- durable scheduler, daemon/service integration, release qualification or release action.

### Acceptance

- Shutdown leaves no scheduled fire or leaked task; channel failures produce bounded outcomes.
- Deterministic stress tests pass without long wall-clock sleeps or flaky tolerance widening.
- A second operator reproduces register/fire/list/cancel/shutdown from the written packet.
- Closeout explicitly keeps REL-002/release and durable scheduling out of scope.

### Planned Validation

- lifecycle stress and clean-HOME fixture-provider trial;
- full standard validation ladder;
- independent replay record without secrets.

### Documentation To Update

- README user guide/troubleshooting; SCHED-001; iteration index/backlog/Board; closeout report.

### Risks And Rollback

- Risk: tests pass while actor tasks leak or real composition roots differ.
- Rollback: disable scheduler registration as one composition-root change; preserve session runtime.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-13 | Planning | Blocked on I126 Complete; no activation claimed. |

## Verification Evidence

- Pending activation.

## Variance And Residuals

- Persistent/calendar scheduling requires a new requirement and ADR.

## Retrospective

- Pending.
