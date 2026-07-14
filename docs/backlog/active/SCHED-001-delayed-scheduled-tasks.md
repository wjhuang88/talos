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
- [ ] `schedule` tool accepts `{ message: string, interval_secs: u64 }` and returns a `task_id`.
- [ ] `cancel_scheduled_task` tool accepts `{ task_id: string }` and cancels the task.
- [ ] `list_scheduled_tasks` tool returns all active scheduled tasks.
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
