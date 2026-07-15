# Iteration I127: Scheduler Reliability Closeout

> Document status: Complete
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
| 2026-07-15 | Maintainer acceptance (`6cfc19c`) | Promotion rejected; I127 remains Review. Focused SF130/SF131 tests, fmt, workspace Clippy, governance, and diff checks pass, but the acceptance evidence is not discriminating: the claimed full-channel test never fills a channel, the recurring stress bound allows Burst behavior, clean-HOME is a proposed command without a trial record, and no second-operator replay record exists. Board, SCHED-001, and the execution package also remain at the pre-I127 state. |
| 2026-07-15 | Maintainer re-review (`43662e7`) | Promotion rejected again; I127 remains Review. The exact-10 assertion now discriminates Delay from Burst, and governance review records are present. However, the full-channel test exercises only a raw `mpsc::try_send`; production `SchedulerHandle::send` and tools still use unbounded-wait `send().await`, so it does not prove a bounded tool outcome under queue pressure. The recorded clean-HOME command only confirms process exit/system-prompt schemas, not register/fire/list/cancel/shutdown, and the claimed second-operator replay is an unattributed test-count statement rather than an independent replay record. |
| 2026-07-15 | Maintainer closure (`b15a33c`) | Production `SchedulerHandle::send` now uses immediate bounded `try_send` behavior, mapping full and closed channels to distinct actionable tool errors. `sf130_full_command_queue_returns_bounded_tool_errors` saturates the real handle and exercises delay/schedule/list/cancel. `scripts/replay_i127_scheduler.sh` was replayed by the maintainer in a disposable HOME: clean-HOME CLI composition plus fixture-provider register/fire/list/cancel/shutdown all passed with no credential. All acceptance criteria are met; I127 is Complete. |

## Verification Evidence

- **SF130**: shutdown no-leak, cancel-token completion, real full-queue tool errors across delay/schedule/list/cancel, and receiver-gone tool error. `b15a33c` makes the production send path immediate and bounded; no command awaits capacity.
- **SF131**: 3 stress tests: 20-round rapid register/cancel, 10-recurring bounded fires (Delay), 50-task mass cancel. All paused time. Commit `f2cc7b7`.
- **SF132**: `scripts/replay_i127_scheduler.sh` creates a disposable HOME/TALOS_HOME while preserving the pinned Rust toolchain roots, verifies clean-HOME CLI composition with the mock provider, then runs real fixture-provider lifecycle tests.
- **SF133**: independent maintainer replay on 2026-07-15: the script passed its clean-HOME preflight, `fixture_provider_delay_fires_and_follow_up_gets_fresh_deny` (register/fire), `fixture_provider_list_cancel_full_lifecycle` (list/cancel), and `sf130_shutdown_leaves_no_leaked_tasks` (shutdown). No credentials or network provider call were used. REL-002/release and durable scheduling remain out of scope.
- **Full scheduler test suite**: 62 scheduler tests are included in the 277 passing `talos-agent` tests, covering types, actor behavior, tool execution, permission regression, fixture-provider, hardening, and stress.
- **Iteration validation ladder** (2026-07-15):
  - `cargo fmt --all -- --check` — pass
  - `cargo clippy --workspace --locked -- -D warnings` — pass
  - `cargo test --workspace --locked` — pass
  - `./scripts/release_preflight.sh` — pass
  - `scripts/validate_project_governance.sh .` — 0 warnings
  - `git diff --check` — clean

## Maintainer Closure Findings

The two remaining re-review findings are resolved by `b15a33c`:

1. **Production queue pressure is bounded.** `SchedulerHandle::send` uses `try_send`, returning
   `Full` or `Closed` immediately. The four production scheduling tools map `Full` to
   `scheduler is busy; try again` and preserve an actionable unavailable error for `Closed`.
   `sf130_full_command_queue_returns_bounded_tool_errors` saturates a real handle and asserts
   those results through delay, schedule, list, and cancel.

2. **Clean-HOME lifecycle replay is executable and recorded.**
   `scripts/replay_i127_scheduler.sh` isolates Talos state, retains the repository-pinned Rust
   toolchain, runs the no-credential mock composition check, and runs the real Agent/session
   fixture matrix for fire, list/cancel, and shutdown. The maintainer executed it successfully on
   2026-07-15; the script's explicit scope note prevents the CLI request preview from being
   mistaken for a synthetic tool invocation.

The earlier exact-10 Delay assertion and governance synchronization remain accepted.

## Historical Maintainer Re-review Findings

1. **Queue pressure is still unbounded at the production seam.** The new full-channel test fills a
   standalone capacity-4 Tokio channel and tests `try_send`, but every scheduler tool calls
   `SchedulerHandle::send`, which still uses `send().await` without a timeout. It neither
   constructs a scheduler tool nor proves a bounded `ToolResult` when the real command queue is
   full. Implement a bounded production policy (for example `try_send` mapped to an actionable
   error, or an explicit timeout) and test it through the tools. The documented no-timeout
   limitation cannot satisfy SF130's published bounded-failure acceptance.

2. **SF132/SF133 runtime evidence remains absent.** The recorded clean-HOME command only confirms
   a successful mock exit and schema disclosure; it does not exercise registration, fire, list,
   cancel, or shutdown. Record a replay packet with isolated-HOME setup, exact commands and
   outputs, cleanup, and recovery behavior. A second operator must independently record the
   commit, environment description, commands, and results without credentials; a bare test-count
   assertion is not independent replay evidence.

Resolved by `b15a33c` and the recorded replay above. Focused tests, the full locked validation
ladder, governance validation, and diff check pass. I127 is Complete; this does not authorize
release activity.

## Variance And Residuals

- Persistent/calendar scheduling requires a new requirement and ADR.
- **REL-002/release**: the scheduler is a session-scoped capability. It does not qualify REL-002
  and does not trigger any release, tag, or publish action.
- **Durable scheduling**: not implemented. Scheduled tasks die when the process exits. A future
  iteration would need an ADR for persistence (SQLite-backed scheduler state) and recovery tests.
- **Model-switch task persistence**: switching models recreates the scheduler, losing active tasks.
  This is accepted for v1.

## Retrospective

- All 4 stories delivered. The 4-month program (I124-I127) implementation is complete; I127 is
  Complete after production queue-pressure and independent replay closure. No release action is
  authorized.
- The scheduler lifecycle (register → fire → list → cancel → shutdown) is tested with
  paused time, real Agent/session fixture paths, deterministic stress, and a clean-HOME trial.
- The `MissedTickBehavior::Delay` choice is discriminated: stress asserts exactly 10 fires
  for 10 tasks (Burst would produce 20).
- Queue-full is bounded at the production seam: all four scheduling tools return immediately with
  `scheduler is busy; try again` when their real `SchedulerHandle` queue is saturated.
- Honest closeout: no REL-002 qualification, no release action, no persistence.
