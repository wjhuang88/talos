# SCHED-001: Delayed and Scheduled Task Execution

**Status**: In Progress (→ I028, activated 2026-06-18)
**Priority**: P2
**Source**: User request 2026-06-18
**Depends on**: None

I092 activation note (2026-07-04): selected for autonomy permission analysis only. v1 message
injection remains the safe shape; direct scheduled tool execution, persistence, and cron-style
expansion remain out of scope until a non-bypass permission matrix is recorded.

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

- [ ] `delay` tool accepts `{ message: string, delay_secs: u64 }` and returns a `task_id`.
- [ ] `schedule` tool accepts `{ message: string, interval_secs: u64 }` and returns a `task_id`.
- [ ] `cancel_scheduled_task` tool accepts `{ task_id: string }` and cancels the task.
- [ ] `list_scheduled_tasks` tool returns all active scheduled tasks.
- [ ] Scheduled message injection triggers a new turn via `SessionOp::Submit`.
- [ ] Tasks are cancelled on session shutdown (CancellationToken).
- [ ] `cargo check --workspace` passes.
- [ ] `cargo test --workspace` passes.
- [ ] `cargo clippy --workspace -- -D warnings` passes.
