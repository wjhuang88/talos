# SCHED-001: Delayed and Scheduled Task Execution

**Status**: Planned — I028 superseded before implementation; selected into I124-I127
**Priority**: P2
**Source**: User request 2026-06-18
**Depends on**: None

I092 activation note (2026-07-04): selected for autonomy permission analysis only. v1 message
injection remains the safe shape; direct scheduled tool execution, persistence, and cron-style
expansion remain out of scope until a non-bypass permission matrix is recorded.

I092 A11 result (2026-07-04): `docs/reference/AUTONOMY-PERMISSION-MATRIX-2026-07-04.md` records
the scheduled-task boundary. Scheduled message injection does not grant permission to future tool
calls. Scheduled direct tool execution and persistent scheduler state remain denied/deferred until a
future ADR and fire-time permission re-evaluation tests exist.

2026-07-13 selection correction: I028's proposed read/auto-allow classification is rejected for
mutating schedule registration/cancellation. I124-I127 retain session-scoped message injection but
classify `delay`, `schedule`, and `cancel_scheduled_task` as Execute/Ask and listing as Read. No
permission-engine change, persistence, or direct scheduled tool execution is selected.

## Problem

The agent cannot schedule future actions. If the LLM wants to "check again in 60 seconds"
or "remind the user every 5 minutes", there is no mechanism to do so. All tool calls are
synchronous within a single turn.

## Scope

v1 (session-scoped, message injection):

- 4 built-in tools: `delay`, `schedule`, `cancel_scheduled_task`, `list_scheduled_tasks`.
- Trigger action is message injection via `SessionOp::Submit`. The LLM mediates any
  subsequent tool calls through the normal permission pipeline.
- Tasks are session-scoped: they die when the process exits. No persistence.
- No external scheduling crate. Raw `tokio::time::sleep` + `tokio::time::interval`.

Out of scope (v2+):

- Direct tool execution from scheduled tasks (bypasses LLM; needs ADR for permission model).
- Persistence across restarts (needs SQLite-backed scheduler state).
- Cron expression support (interval-based is sufficient for v1).

## Architecture

```
talos-core:    ScheduleCommand enum + ScheduledTaskInfo type
talos-tools:   4 tool structs (DelayTool, ScheduleTool, CancelScheduledTaskTool, ListScheduledTasksTool)
talos-agent:   SchedulerActor (tokio task, owns task HashMap, injects via sq_tx)
talos-cli:     Composition root (spawns actor, wires tools with sched_tx)
```

## Acceptance Criteria

- [x] `delay` tool accepts `{ message: string, delay_secs: u64 }` and returns a `task_id`.
- [x] `schedule` tool accepts `{ message: string, interval_secs: u64 }` and returns a `task_id`.
- [x] `cancel_scheduled_task` tool accepts `{ task_id: string }` and cancels the task.
- [x] `list_scheduled_tasks` tool returns all active scheduled tasks.
- [x] Scheduled message injection triggers a new turn via `SessionOp::Submit`.
- [x] Tasks are cancelled on session shutdown (CancellationToken).
- [x] `cargo check --workspace` passes.
- [x] `cargo test --workspace` passes.
- [x] `cargo clippy --workspace -- -D warnings` passes.

> I124 delivered the `delay` tool (one-shot). `schedule` (recurring), `cancel_scheduled_task`,
> and `list_scheduled_tasks` are owned by I125-I126.

2026-07-14 maintainer review: I124 remains Review. Repository validation is green, but the raw
`delay` registrations bypass the CLI/TUI approval wrappers, the claimed fixture-provider/session
proof is absent, and the scheduler introduced an unapproved public API. I125 remains blocked until
the findings in the I124 owner doc are fixed and re-reviewed.

2026-07-14 second review: the approval-wrapper bypass is fixed, but I124 remains Review. The real
Agent/session test does not prove a distinct fire-time Deny/Ask decision, public exports and a new
dev-dependency remain outside the baseline, the queue-wait limitation is misstated, and one
doctest is ignored without a tracking issue. I125 remains blocked.

2026-07-14 third review: commit `7fe1d17` closes the dev-dependency, queue wording, doctest, and
distinct-Deny setup findings, but I124 remains Review. The Deny test can still pass without the
scheduled turn occurring, and ADR-041 has not completed accepted baseline change control for the
two public exports. I125 remains blocked.

2026-07-14 closure: the fixture-provider test now positively proves the labeled scheduled turn ran
and separately proves the follow-up resource was denied. ADR-041 is Accepted and indexed for the
two minimal composition exports. The full validation ladder passes; I124 is Complete. I125 is
unblocked and remains Planned, not activated.

2026-07-14 I125 maintainer review: I125 remains Review despite a green validation ladder. Required
corrections are recorded in the I125 owner doc: preserve or formally migrate the ADR-041 public
API, add discriminating Delay/no-burst and cancellation/shutdown-race tests, prove recurring
fire-time permission isolation, supply the configured-provider walkthrough or approved variance,
and correct user/governance documentation. I126 remains blocked.

2026-07-14 I125 maintainer re-review (`20e782e`): validation, the discriminating Delay test,
recurring fresh-Deny proof, and cancel-result text pass. I125 remains Review because the breaking
ADR-041 API amendment lacks explicit maintainer authority or a compatibility entry point, the new
cancel/shutdown tests do not exercise a competing timer boundary, and SF113 still has neither the
published real configured-provider walkthrough nor an accepted maintainer variance. Minor
root-count and stale-symbol documentation drift also remain. I126 stays blocked.

2026-07-14 I125 closure: the I124 factory is restored as a tested compatibility entry point;
Cancel/Shutdown tests now exercise a competing-ready timer boundary; a real `alibaba-cn` /
`glm-5.2` binary walkthrough registered `sched_1` and persisted eight labeled recurring turns;
all documentation and full validation gates pass. I125 is Complete. I126 is unblocked but remains
Planned until its activation inventory and Gate 0 are recorded.

2026-07-15 I126 maintainer review: I126 remains Review despite a green validation ladder.
Recurring list timing becomes stale after the first fire; total list output and sensitive previews
are not safely bounded; SF123 lacks fixture-provider list/cancel proof; narrow-width tests do not
exercise semantic ratatui buffers or all required fields; Deny/outcome evidence and governance
sync are incomplete. Corrections are detailed in the I126 owner doc. I127 remains blocked.

2026-07-15 I126 maintainer re-review (`7b5a0ab`): I126 remains Review. The remediation fixes
first-tick metadata and user-visible message exposure and adds a happy-path Agent/session fixture,
but it does not prove denied cancellation leaves a real task unchanged or that cancellation
prevents a later fire. Multi-task cap, multiple-tick timing, and actual tool-result narrow-buffer
proof also remain incomplete; owner evidence is stale. I127 remains blocked.

2026-07-15 I126 closure: list output now hides message content, caps at 20 rows with an omitted
count, and has a 21-task regression. Three paused recurring ticks refresh `next` timing. Real
Agent/session fixtures prove approved cancellation produces no later submission and configured
Deny leaves a task visible before and after a later list. Scrollback-derived ratatui Buffer tests
cover 40/60/80/120 widths. Full validation passes; I126 is Complete. I127 is unblocked and remains
Planned until its activation gate.

2026-07-15 I127 maintainer acceptance (`6cfc19c`): I127 remains Review. The claimed queue-full
test never fills a channel and the recurring stress bound accepts a catch-up Burst. No isolated
clean-HOME or independent second-operator replay record exists, and Board/SCHED/execution-package
status is stale. Corrections are recorded in the I127 owner doc; no release action is authorized.

2026-07-15 I127 maintainer re-review (`43662e7`): I127 remains Review. The exact Delay assertion
and review-document sync are accepted, but a raw `try_send` test does not bound the production
tools' `send().await` full-queue wait. The clean-HOME command only shows schemas and a successful
exit; it does not record the lifecycle or independent operator replay required by SF132/SF133.

2026-07-15 I127 closure (`b15a33c`): `SchedulerHandle::send` now returns immediate Full/Closed
results through the real delay/schedule/list/cancel tools, with a saturated-handle regression for
all four. `scripts/replay_i127_scheduler.sh` was independently replayed in a disposable HOME and
proved clean-HOME composition plus fixture-provider register/fire/list/cancel/shutdown. Full locked
validation and governance pass. I127 and the I124-I127 delivery sequence are Complete; durable
scheduling and REL-002/release remain out of scope.
