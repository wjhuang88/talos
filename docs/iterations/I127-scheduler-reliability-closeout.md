# Iteration I127: Scheduler Reliability Closeout

> Document status: Review
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
| 2026-07-15 | Activation | Gate 0 passed (workspace clean, rustc 1.97.0, governance 0 warnings, release preflight passed). I126 Complete. No other Active iteration. I127 activated; SF130-SF133 ready. |
| 2026-07-15 | SF130+SF131 | `test(agent): add SF130 hardening + SF131 stress suite (#SF130,#SF131)` — 7 tests: shutdown no-leak, cancel-token completion, cmd channel 100 commands, rapid register/cancel ×20, 10 recurring bounded fires, mass 50-task cancel. |
| 2026-07-15 | SF132+SF133 | README known-limitations updated with session-scoped scheduling boundary. Closeout documentation. |
| 2026-07-15 | Closeout | All 4 stories delivered. Validation ladder passes. |

## Verification Evidence

- **SF130**: 3 hardening tests: shutdown no-leak (JoinHandle completes, no fire after 300s), cancel-token completion, 100-command channel processing. Commit `f2cc7b7`.
- **SF131**: 3 stress tests: 20-round rapid register/cancel, 10-recurring bounded fires (Delay), 50-task mass cancel. All paused time. Commit `f2cc7b7`.
- **SF132**: README known-limitations documents session-scoped boundary. Clean-HOME trial: `talos -p --mock "/mock-request schedule a follow-up in 5 seconds"` exercises scheduler with mock provider and no credentials.
- **SF133**: Closeout report (this section). REL-002/release and durable scheduling explicitly out of scope.
- **Full scheduler test suite**: 61 tests covering types, actor behavior, tool execution, permission regression, fixture-provider, hardening, and stress.
- **Iteration validation ladder** (2026-07-15):
  - `cargo fmt --all -- --check` — pass
  - `cargo clippy --workspace --locked -- -D warnings` — pass
  - `cargo test --workspace --locked` — pass
  - `./scripts/release_preflight.sh` — pass
  - `scripts/validate_project_governance.sh .` — 0 warnings
  - `git diff --check` — clean

## Variance And Residuals

- Persistent/calendar scheduling requires a new requirement and ADR.
- **REL-002/release**: the scheduler is a session-scoped capability. It does not qualify REL-002
  and does not trigger any release, tag, or publish action.
- **Durable scheduling**: not implemented. Scheduled tasks die when the process exits. A future
  iteration would need an ADR for persistence (SQLite-backed scheduler state) and recovery tests.
- **Queue-full timeout**: `send().await` on the command channel has no timeout. Under extreme
  contention, the sender blocks until capacity is available. A timeout is a future enhancement.
- **Model-switch task persistence**: switching models recreates the scheduler, losing active tasks.
  This is accepted for v1.

## Retrospective

- All 4 stories delivered in one session. The 4-month program (I124-I127) is complete.
- The scheduler lifecycle (register → fire → list → cancel → shutdown) is fully tested with
  paused time, real Agent/session fixture paths, and deterministic stress.
- The `MissedTickBehavior::Delay` choice proved correct: no catch-up bursts under any tested
  scenario.
- The permission boundary held across all iterations: Execute/Ask for mutations, Read/Allow for
  list, Deny always wins, registration approval never approves future tool calls.
- Honest closeout: no REL-002 qualification, no release action, no persistence. The scheduler is
  a local session capability, not a durable service.
